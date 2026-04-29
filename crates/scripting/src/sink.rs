//! Tag write routing for script-driven writes.

use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_tag_engine::TagStore;
use thiserror::Error;

/// Destination for `system.tag.write` calls from Python scripts.
pub trait TagWriteSink: Send + Sync + 'static {
    /// Enqueue or publish a tag write.
    ///
    /// This returns after the write has been accepted by the sink. Driver
    /// sinks should mirror websocket write semantics and not wait for the PLC
    /// to acknowledge the write.
    fn enqueue(&self, path: &str, value: TagValue) -> Result<(), TagWriteError>;
}

/// Error returned when a script tag write cannot be accepted.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TagWriteError {
    /// Driver write queue is full.
    #[error("driver write queue full for '{0}'")]
    Busy(String),
    /// Driver write channel is closed.
    #[error("driver write channel closed for '{0}'")]
    Closed(String),
    /// Driver was required but not registered.
    #[error("unknown driver for path '{0}'")]
    UnknownDriver(String),
    /// Other sink-specific error.
    #[error("{0}")]
    Other(String),
}

/// In-memory sink used by tests and memory-tag-only embeddings.
#[derive(Clone)]
pub struct MemorySink {
    store: TagStore,
}

impl MemorySink {
    /// Create a sink that publishes accepted writes into `store`.
    pub fn new(store: TagStore) -> Self {
        Self { store }
    }
}

impl TagWriteSink for MemorySink {
    fn enqueue(&self, path: &str, value: TagValue) -> Result<(), TagWriteError> {
        self.store.publish(path, value, Quality::Good);
        Ok(())
    }
}
