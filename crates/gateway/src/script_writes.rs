//! Gateway-backed tag write sink for Python scripts.

use openwebhmi_driver_api::TagAddress;
use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_scripting::{TagWriteError, TagWriteSink};
use openwebhmi_tag_engine::TagStore;

use crate::project::{DriverHandles, WriteCommand, WriteEnqueueError};

/// Routes script-driven tag writes through driver queues or memory tags.
#[derive(Clone)]
pub struct GatewayTagWriteSink {
    driver_handles: DriverHandles,
    store: TagStore,
}

impl GatewayTagWriteSink {
    /// Create a sink from the gateway's active driver handles and tag store.
    pub fn new(driver_handles: DriverHandles, store: TagStore) -> Self {
        Self {
            driver_handles,
            store,
        }
    }
}

impl TagWriteSink for GatewayTagWriteSink {
    fn enqueue(&self, path: &str, value: TagValue) -> Result<(), TagWriteError> {
        let Some((driver_id, address)) = split_tag_path(path) else {
            self.store.publish(path, value, Quality::Good);
            return Ok(());
        };

        let Some(driver) = self.driver_handles.get(driver_id) else {
            self.store.publish(path, value, Quality::Good);
            return Ok(());
        };

        driver
            .try_write(WriteCommand {
                address: TagAddress::new(address.to_string()),
                value,
            })
            .map_err(|err| match err {
                WriteEnqueueError::Busy => TagWriteError::Busy(path.to_string()),
                WriteEnqueueError::Closed => TagWriteError::Closed(path.to_string()),
            })
    }
}

fn split_tag_path(path: &str) -> Option<(&str, &str)> {
    let (driver_id, address) = path.split_once('/')?;
    (!driver_id.is_empty() && !address.is_empty()).then_some((driver_id, address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_paths_publish_to_store() {
        let store = TagStore::new();
        let sink = GatewayTagWriteSink::new(DriverHandles::new(), store.clone());

        sink.enqueue("mem/derived", TagValue::Real(42.0)).unwrap();

        assert_eq!(
            store.get("mem/derived").unwrap().value,
            TagValue::Real(42.0)
        );
    }
}
