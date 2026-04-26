//! Mock driver implementation for downstream tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use openwebhmi_protocol::TagValue;

use crate::{
    trait_def::make_metadata, Capabilities, Driver, DriverError, DriverMetadata, DriverResult,
    TagAddress, TagNode,
};

/// Failure mode injected into [`MockDriver`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockFailureMode {
    /// Reads return `NotConnected` until the counter reaches zero.
    NotConnectedReads(usize),
}

/// Simple in-memory driver for tests.
#[derive(Clone)]
pub struct MockDriver {
    values: Arc<Mutex<HashMap<TagAddress, TagValue>>>,
    failure_mode: Arc<Mutex<Option<MockFailureMode>>>,
}

impl MockDriver {
    /// Construct a mock driver with static values.
    pub fn new(values: HashMap<TagAddress, TagValue>) -> Self {
        Self {
            values: Arc::new(Mutex::new(values)),
            failure_mode: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the current failure mode.
    pub fn set_failure_mode(&self, failure_mode: Option<MockFailureMode>) -> DriverResult<()> {
        *self
            .failure_mode
            .lock()
            .map_err(|_| DriverError::Other(anyhow::anyhow!("mock failure lock poisoned")))? =
            failure_mode;
        Ok(())
    }
}

#[async_trait]
impl Driver for MockDriver {
    fn metadata(&self) -> DriverMetadata {
        make_metadata(
            "OpenWebHMI",
            "mock",
            env!("CARGO_PKG_VERSION"),
            Capabilities::NONE,
        )
    }

    async fn connect(&mut self, _config: serde_json::Value) -> DriverResult<()> {
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        Ok(())
    }

    async fn browse(&self, _path: Option<&str>) -> DriverResult<Vec<TagNode>> {
        let values = self
            .values
            .lock()
            .map_err(|_| DriverError::Other(anyhow::anyhow!("mock value lock poisoned")))?;
        Ok(values
            .keys()
            .map(|address| TagNode {
                name: address.raw.clone(),
                address: Some(address.clone()),
                data_type: None,
                children: Vec::new(),
            })
            .collect())
    }

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
        if let Some(MockFailureMode::NotConnectedReads(remaining)) = &mut *self
            .failure_mode
            .lock()
            .map_err(|_| DriverError::Other(anyhow::anyhow!("mock failure lock poisoned")))?
        {
            if *remaining > 0 {
                *remaining -= 1;
                return Err(DriverError::NotConnected);
            }
        }

        self.values
            .lock()
            .map_err(|_| DriverError::Other(anyhow::anyhow!("mock value lock poisoned")))?
            .get(address)
            .cloned()
            .ok_or_else(|| DriverError::InvalidAddress(address.raw.clone()))
    }

    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
        self.values
            .lock()
            .map_err(|_| DriverError::Other(anyhow::anyhow!("mock value lock poisoned")))?
            .insert(address.clone(), value);
        Ok(())
    }

    fn clone_for_polling(&self) -> Option<Box<dyn Driver>> {
        Some(Box::new(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use futures_util::StreamExt;
    use openwebhmi_protocol::{Quality, TagValue};
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn default_subscribe_polls_mock_driver_values() {
        let address = TagAddress::new("Counter");
        let driver = MockDriver::new(HashMap::from([(address.clone(), TagValue::Int(7))]));
        let mut stream = driver.subscribe(vec![address.clone()]).await.unwrap();

        let update = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("polling fallback should emit promptly")
            .expect("stream should stay open");

        assert_eq!(update.address, address);
        assert_eq!(update.value, TagValue::Int(7));
        assert_eq!(update.quality, Quality::Good);
        assert!(update.ts_ms > 0);
    }
}
