//! Live tag recorder.

use std::collections::HashMap;

use openwebhmi_protocol::TagValue;
use openwebhmi_tag_engine::{TagSnapshot, TagStore};
use tokio::task::JoinHandle;
use tracing::warn;

use crate::store::HistorianStore;

/// Per-tag history recording configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryTagConfig {
    /// Tag path to record.
    pub path: String,
    /// Minimum interval between recorded samples.
    pub rate_ms: u64,
    /// Numeric deadband. Non-numeric values ignore it.
    pub deadband: f64,
}

/// Join handle for a group of tag recorder tasks.
pub struct RecorderHandle {
    tag_store: TagStore,
    historian: HistorianStore,
    handles: HashMap<String, JoinHandle<()>>,
    configs: HashMap<String, HistoryTagConfig>,
}

impl RecorderHandle {
    /// Replace the logged-tag set, aborting removed tags and spawning added tags.
    pub fn update_configs(&mut self, configs: Vec<HistoryTagConfig>) {
        let next = configs_by_path(configs);
        let removed = self
            .handles
            .keys()
            .filter(|path| !next.contains_key(*path))
            .cloned()
            .collect::<Vec<_>>();
        for path in removed {
            if let Some(handle) = self.handles.remove(&path) {
                handle.abort();
            }
        }
        for (path, config) in next {
            if let Some(existing) = self.handles.get(&path) {
                if self.configs.get(&path) == Some(&config) {
                    continue;
                }
                existing.abort();
                self.handles.remove(&path);
            }
            if !self.handles.contains_key(&path) {
                self.handles.insert(
                    path.clone(),
                    spawn_one(
                        self.tag_store.clone(),
                        self.historian.clone(),
                        config.clone(),
                    ),
                );
            }
            self.configs.insert(path, config);
        }
        self.configs
            .retain(|path, _| self.handles.contains_key(path));
    }

    /// Stop all recorder tasks.
    pub fn abort(self) {
        for (_, handle) in self.handles {
            handle.abort();
        }
    }
}

/// Spawn recorders for configured tags.
pub fn spawn_recorder(
    tag_store: TagStore,
    historian: HistorianStore,
    configs: Vec<HistoryTagConfig>,
) -> RecorderHandle {
    let configs = configs_by_path(configs);
    let handles = configs
        .values()
        .cloned()
        .map(|config| {
            let path = config.path.clone();
            (
                path,
                spawn_one(tag_store.clone(), historian.clone(), config),
            )
        })
        .collect();
    RecorderHandle {
        tag_store,
        historian,
        handles,
        configs,
    }
}

fn spawn_one(
    tag_store: TagStore,
    historian: HistorianStore,
    config: HistoryTagConfig,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        run_one(tag_store, historian, config).await;
    })
}

async fn run_one(tag_store: TagStore, historian: HistorianStore, config: HistoryTagConfig) {
    let mut rx = tag_store.subscribe(&config.path);
    let mut state = FilterState::default();
    if let Some(snapshot) = tag_store.get(&config.path) {
        maybe_write(&historian, &config, &mut state, snapshot).await;
    }
    loop {
        match rx.recv().await {
            Ok(snapshot) => maybe_write(&historian, &config, &mut state, snapshot).await,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(path = %config.path, skipped, "historian recorder lagged");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

#[derive(Default)]
struct FilterState {
    last_logged_ts: Option<u64>,
    last_numeric: Option<f64>,
}

async fn maybe_write(
    historian: &HistorianStore,
    config: &HistoryTagConfig,
    state: &mut FilterState,
    snapshot: TagSnapshot,
) {
    if let Some(last_ts) = state.last_logged_ts {
        if snapshot.ts.saturating_sub(last_ts) < config.rate_ms {
            return;
        }
    }
    if let Some(value) = numeric_value(&snapshot.value) {
        if let Some(last) = state.last_numeric {
            if (value - last).abs() < config.deadband {
                return;
            }
        }
        state.last_numeric = Some(value);
    }
    let path = snapshot.path;
    let ts = snapshot.ts;
    let value = snapshot.value;
    let quality = snapshot.quality;
    let historian = historian.clone();
    let result =
        tokio::task::spawn_blocking(move || historian.write_sample(&path, ts, &value, quality))
            .await;
    let write = match result {
        Ok(write) => write,
        Err(err) => {
            warn!(path = %config.path, error = %err, "historian recorder write task failed");
            return;
        }
    };
    if let Err(err) = write {
        warn!(path = %config.path, error = %err, "failed to write history sample");
        return;
    }
    state.last_logged_ts = Some(ts);
}

fn numeric_value(value: &TagValue) -> Option<f64> {
    match value {
        TagValue::Int(value) => Some(*value as f64),
        TagValue::Real(value) => Some(*value),
        TagValue::Bool(_) | TagValue::String(_) => None,
    }
}

/// Diff helper used by gateway hot-reload integration.
pub fn configs_by_path(configs: Vec<HistoryTagConfig>) -> HashMap<String, HistoryTagConfig> {
    configs
        .into_iter()
        .map(|config| (config.path.clone(), config))
        .collect()
}
