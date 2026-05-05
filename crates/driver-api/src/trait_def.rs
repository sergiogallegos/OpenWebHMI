//! Driver trait and update envelope.

use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::stream::{self, BoxStream};
use openwebhmi_protocol::{Quality, TagValue};
use tokio::time;

use crate::{Capabilities, DriverError, DriverMetadata, DriverResult, TagAddress, TagNode};

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// One driver-originated tag update.
#[derive(Debug, Clone, PartialEq)]
pub struct DriverUpdate {
    /// Driver-specific address that produced this update.
    pub address: TagAddress,
    /// Current value.
    pub value: TagValue,
    /// Current quality.
    pub quality: Quality,
    /// Unix epoch milliseconds from the driver or gateway.
    pub ts_ms: u64,
}

/// Stable contract implemented by every OpenWebHMI device driver.
// TODO(v1.1): revisit when async-fn-in-trait becomes object-safe.
#[async_trait]
pub trait Driver: Send + Sync + 'static {
    /// Static driver metadata.
    fn metadata(&self) -> DriverMetadata;

    /// Connect using driver-specific JSON configuration.
    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()>;

    /// Disconnect from the remote device.
    async fn disconnect(&mut self) -> DriverResult<()>;

    /// Browse remote tags under an optional path.
    async fn browse(&self, path: Option<&str>) -> DriverResult<Vec<TagNode>>;

    /// Read one tag.
    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue>;

    /// Write one tag.
    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()>;

    /// Return an owned driver handle usable by the default polling
    /// subscription.
    ///
    /// Drivers with cheap shared state can override this and return a boxed
    /// clone. Drivers with native subscription can leave this as `None` and
    /// override [`Driver::subscribe`] instead.
    fn clone_for_polling(&self) -> Option<Box<dyn Driver>> {
        None
    }

    /// Subscribe to updates for the supplied addresses.
    ///
    /// Drivers with native subscription should override this. The default is a
    /// polling stream at one second, implemented by repeatedly calling
    /// [`Driver::read`].
    ///
    /// Cancel-safe contract: implementations must return a stream whose
    /// `Stream::next` future can be dropped before completion without losing a
    /// tag update.
    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<BoxStream<'static, DriverUpdate>> {
        let driver = self
            .clone_for_polling()
            .ok_or_else(|| DriverError::UnsupportedType {
                device_type: "polling subscription requires clone_for_polling".to_string(),
            })?;

        let state = PollState {
            driver,
            addresses,
            interval: time::interval(DEFAULT_POLL_INTERVAL),
            pending: VecDeque::new(),
        };

        Ok(Box::pin(stream::unfold(state, |mut state| async move {
            loop {
                if let Some(update) = state.pending.pop_front() {
                    return Some((update, state));
                }

                state.interval.tick().await;
                let ts_ms = now_ms();
                for address in &state.addresses {
                    match state.driver.read(address).await {
                        Ok(value) => state.pending.push_back(DriverUpdate {
                            address: address.clone(),
                            value,
                            quality: Quality::Good,
                            ts_ms,
                        }),
                        Err(error) => {
                            tracing::debug!(
                                address = %address.raw,
                                %error,
                                "polling subscription read failed"
                            );
                        }
                    }
                }
            }
        })))
    }
}

struct PollState {
    driver: Box<dyn Driver>,
    addresses: Vec<TagAddress>,
    interval: time::Interval,
    pending: VecDeque<DriverUpdate>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

/// Helper for metadata construction in simple drivers.
pub fn make_metadata(
    vendor: impl Into<String>,
    family: impl Into<String>,
    crate_version: &'static str,
    capabilities: Capabilities,
) -> DriverMetadata {
    DriverMetadata {
        vendor: vendor.into(),
        family: family.into(),
        crate_version: crate_version.to_string(),
        capabilities,
    }
}
