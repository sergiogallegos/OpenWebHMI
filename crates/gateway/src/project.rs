//! Minimal Phase 1 project loader and driver publisher.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_driver_rockwell::{RockwellConfig, RockwellDriver};
use openwebhmi_project_store::{DriverConfig, Project, ProjectStore, TagConfig};
use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_tag_engine::TagStore;
use tokio::time;
use tracing::{info, warn};

const STATUS_PREFIX: &str = "system/drivers";
const RECONNECT_BACKOFF: Duration = Duration::from_millis(250);

/// Load and validate a Phase 1 project file.
pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Project> {
    let path = path.as_ref();
    let root = path
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    let project_id = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("phase1-demo");
    ProjectStore::open(root)?.load(project_id)
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
    let config: RockwellConfig =
        serde_json::from_value(driver.config.clone()).context("invalid Rockwell driver config")?;
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
    use openwebhmi_project_store::store::validate_project;

    #[test]
    fn validates_driver_references_and_path_prefixes() {
        let project = Project {
            id: "test".to_string(),
            schema_version: 1,
            name: "test".to_string(),
            version: 0,
            drivers: vec![DriverConfig {
                id: "rockwell-1".to_string(),
                driver_type: "rockwell".to_string(),
                config: serde_json::json!({}),
            }],
            tags: vec![TagConfig {
                path: "other/Pressure".to_string(),
                driver: "rockwell-1".to_string(),
                address: "Pressure".to_string(),
            }],
            views: Vec::new(),
        };

        assert!(validate_project(&project).is_err());
    }

    #[test]
    fn loads_phase1_demo_through_project_store() {
        let project =
            load("../../examples/projects/phase1-demo/project.toml").unwrap_or_else(|_| {
                load("examples/projects/phase1-demo/project.toml").expect("phase1 demo loads")
            });

        assert_eq!(project.id, "phase1-demo");
        assert_eq!(project.drivers[0].id, "rockwell-1");
        assert!(project
            .tags
            .iter()
            .any(|tag| tag.path == "rockwell-1/Pressure"));
    }
}
