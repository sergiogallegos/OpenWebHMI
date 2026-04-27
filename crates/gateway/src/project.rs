//! Minimal Phase 1 project loader and driver publisher.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context};
use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_driver_rockwell::{RockwellConfig, RockwellDriver};
use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_tag_engine::TagStore;
use serde::Deserialize;
use tokio::time;
use tracing::{info, warn};

const STATUS_PREFIX: &str = "system/drivers";
const RECONNECT_BACKOFF: Duration = Duration::from_millis(250);

/// Parsed Phase 1 project file.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Project {
    /// Project schema version. Phase 1 only accepts `1`.
    pub schema_version: u32,
    /// Human-readable project name.
    pub name: String,
    /// Driver instances configured by the project.
    pub drivers: Vec<DriverConfig>,
    /// Driver-backed tags to publish into the gateway tag store.
    pub tags: Vec<TagConfig>,
}

/// Driver configuration block.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DriverConfig {
    /// Stable project-local driver id.
    pub id: String,
    /// Driver type. Phase 1 supports only `rockwell`.
    #[serde(rename = "type")]
    pub driver_type: String,
    /// Driver-specific JSON-like config.
    pub config: toml::Value,
}

/// Tag mapping from a project path to a driver address.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TagConfig {
    /// Gateway tag path exposed to runtime clients.
    pub path: String,
    /// Referenced driver id.
    pub driver: String,
    /// Driver-native tag address.
    pub address: String,
}

/// Load and validate a Phase 1 project file.
pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Project> {
    let path = path.as_ref();
    let text =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {:?}", path))?;
    let project = toml::from_str::<Project>(&text)
        .with_context(|| format!("failed to parse project TOML {:?}", path))?;
    validate(&project)?;
    Ok(project)
}

/// Start all project drivers and tag publishers.
pub fn spawn_project(project: Project, store: TagStore) -> anyhow::Result<()> {
    let tags_by_driver = group_tags_by_driver(&project);

    for driver in project.drivers {
        let tags = tags_by_driver.get(&driver.id).cloned().unwrap_or_default();
        if tags.is_empty() {
            continue;
        }

        let store = store.clone();
        tokio::spawn(async move {
            run_driver(driver, tags, store).await;
        });
    }

    Ok(())
}

fn validate(project: &Project) -> anyhow::Result<()> {
    if project.schema_version != 1 {
        bail!(
            "unsupported project schema_version {}; expected 1",
            project.schema_version
        );
    }

    let mut driver_ids = HashSet::new();
    for driver in &project.drivers {
        if driver.id.trim().is_empty() {
            bail!("driver id cannot be empty");
        }
        if !driver_ids.insert(driver.id.clone()) {
            bail!("duplicate driver id '{}'", driver.id);
        }
        if driver.driver_type != "rockwell" {
            bail!("unsupported driver type '{}'", driver.driver_type);
        }
    }

    for tag in &project.tags {
        if !driver_ids.contains(&tag.driver) {
            bail!(
                "tag '{}' references unknown driver '{}'",
                tag.path,
                tag.driver
            );
        }
        let expected_prefix = format!("{}/", tag.driver);
        if !tag.path.starts_with(&expected_prefix) {
            bail!(
                "tag path '{}' must start with '{}'",
                tag.path,
                expected_prefix
            );
        }
        if tag.address.trim().is_empty() {
            bail!("tag '{}' address cannot be empty", tag.path);
        }
    }

    Ok(())
}

fn group_tags_by_driver(project: &Project) -> HashMap<String, Vec<TagConfig>> {
    let mut grouped: HashMap<String, Vec<TagConfig>> = HashMap::new();
    for tag in &project.tags {
        grouped
            .entry(tag.driver.clone())
            .or_default()
            .push(tag.clone());
    }
    grouped
}

async fn run_driver(driver: DriverConfig, tags: Vec<TagConfig>, store: TagStore) {
    let status_path = status_path(&driver.id);
    publish_status(&store, &status_path, "connecting");

    loop {
        match connect_rockwell(&driver).await {
            Ok(rockwell) => {
                info!(driver_id = %driver.id, "driver connected");
                publish_status(&store, &status_path, "connected");
                run_subscription_until_disconnect(&driver.id, &tags, &store, rockwell).await;
                publish_status(&store, &status_path, "disconnected");
            }
            Err(err) => {
                warn!(driver_id = %driver.id, error = %err, "driver connect failed");
                publish_status(&store, &status_path, "disconnected");
            }
        }

        publish_status(&store, &status_path, "connecting");
        time::sleep(RECONNECT_BACKOFF).await;
    }
}

async fn connect_rockwell(driver: &DriverConfig) -> anyhow::Result<RockwellDriver> {
    let config: RockwellConfig = driver
        .config
        .clone()
        .try_into()
        .context("invalid Rockwell driver config")?;
    let config = serde_json::to_value(config).context("failed to encode Rockwell config")?;
    let mut rockwell = RockwellDriver::new();
    rockwell.connect(config).await?;
    Ok(rockwell)
}

async fn run_subscription_until_disconnect(
    driver_id: &str,
    tags: &[TagConfig],
    store: &TagStore,
    driver: RockwellDriver,
) {
    let addresses = tags
        .iter()
        .map(|tag| TagAddress::new(tag.address.clone()))
        .collect::<Vec<_>>();
    let path_by_address = tags
        .iter()
        .map(|tag| (tag.address.clone(), tag.path.clone()))
        .collect::<HashMap<_, _>>();

    let mut stream = match driver.subscribe(addresses).await {
        Ok(stream) => stream,
        Err(err) => {
            warn!(%driver_id, error = %err, "driver subscribe failed");
            publish_bad_for_tags(tags, store, err.to_string());
            return;
        }
    };

    while let Some(update) = stream.next().await {
        let Some(path) = path_by_address.get(&update.address.raw) else {
            continue;
        };

        store.publish(path, update.value, update.quality);
        if update.quality == Quality::Bad {
            publish_bad_for_tags(tags, store, "disconnected".to_string());
            return;
        }
    }

    publish_bad_for_tags(tags, store, "disconnected".to_string());
}

fn publish_bad_for_tags(tags: &[TagConfig], store: &TagStore, message: String) {
    for tag in tags {
        store.publish(&tag.path, TagValue::String(message.clone()), Quality::Bad);
    }
}

fn publish_status(store: &TagStore, path: &str, status: &str) {
    store.publish(path, TagValue::String(status.to_string()), Quality::Good);
}

fn status_path(driver_id: &str) -> String {
    format!("{STATUS_PREFIX}/{driver_id}/status")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_driver_references_and_path_prefixes() {
        let project = Project {
            schema_version: 1,
            name: "test".to_string(),
            drivers: vec![DriverConfig {
                id: "rockwell-1".to_string(),
                driver_type: "rockwell".to_string(),
                config: toml::Value::Table(toml::map::Map::new()),
            }],
            tags: vec![TagConfig {
                path: "other/Pressure".to_string(),
                driver: "rockwell-1".to_string(),
                address: "Pressure".to_string(),
            }],
        };

        assert!(validate(&project).is_err());
    }
}
