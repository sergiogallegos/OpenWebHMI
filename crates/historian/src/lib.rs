//! SQLite-backed tag historian.

#![deny(missing_docs)]

/// Aggregation functions for historical queries.
pub mod aggregations;
/// Recorder task that subscribes to live tags.
pub mod recorder;
/// SQLite storage API.
pub mod store;

pub use aggregations::Aggregation;
pub use recorder::{HistoryTagConfig, RecorderHandle, spawn_recorder};
pub use store::{HistorianStore, HistoryPoint};
