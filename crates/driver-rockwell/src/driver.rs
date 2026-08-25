//! Rockwell driver implementation.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::stream::{self, BoxStream};
use openwebhmi_driver_api::{
    Capabilities, Driver, DriverError, DriverMetadata, DriverResult, DriverUpdate, TagAddress,
    TagNode, make_metadata,
};
use openwebhmi_protocol::{Quality, TagValue};
use rust_ethernet_ip::{EtherNetIpError, TagGroupEvent, TagGroupEventKind, TagGroupValueResult};
use tokio::sync::Mutex;

use crate::config::RockwellConfig;
use crate::eip_client::{EipClientLike, RealEipClient, TagGroupSubscriptionLike};
use crate::types::{plc_to_tag_value, tag_to_plc_value};

static NEXT_GROUP_ID: AtomicU64 = AtomicU64::new(1);

type SharedClient = Arc<Mutex<Box<dyn EipClientLike>>>;

/// Rockwell CompactLogix/ControlLogix driver backed by `rust-ethernet-ip`.
#[derive(Default)]
pub struct RockwellDriver {
    client: Option<SharedClient>,
    config: Option<RockwellConfig>,
}

impl RockwellDriver {
    /// Construct a disconnected driver.
    pub fn new() -> Self {
        Self::default()
    }

    #[doc(hidden)]
    pub fn with_client(client: Box<dyn EipClientLike>, config: RockwellConfig) -> Self {
        Self {
            client: Some(Arc::new(Mutex::new(client))),
            config: Some(config),
        }
    }

    fn client(&self) -> DriverResult<SharedClient> {
        self.client.clone().ok_or(DriverError::NotConnected)
    }
}

#[async_trait]
impl Driver for RockwellDriver {
    fn metadata(&self) -> DriverMetadata {
        make_metadata(
            "Rockwell",
            "EtherNet/IP-CIP",
            env!("CARGO_PKG_VERSION"),
            Capabilities::new(true, false, true, true),
        )
    }

    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()> {
        let config = RockwellConfig::from_value(config)
            .map_err(|err| DriverError::Other(anyhow::anyhow!("invalid Rockwell config: {err}")))?;
        let client = RealEipClient::connect(&config)
            .await
            .map_err(map_eip_error)?;
        self.client = Some(Arc::new(Mutex::new(Box::new(client))));
        self.config = Some(config);
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        self.client = None;
        Ok(())
    }

    async fn browse(&self, _path: Option<&str>) -> DriverResult<Vec<TagNode>> {
        Err(DriverError::Other(anyhow::anyhow!(
            "browse not implemented; use external tag-list import"
        )))
    }

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
        let client = self.client()?;
        let mut guard = client.lock().await;
        let value = guard.read_tag(&address.raw).await.map_err(map_eip_error)?;
        plc_to_tag_value(value)
    }

    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
        let client = self.client()?;
        let mut guard = client.lock().await;
        let current = guard.read_tag(&address.raw).await.map_err(map_eip_error)?;
        let plc_value = tag_to_plc_value(value, Some(&current))?;

        guard
            .write_tag(&address.raw, plc_value)
            .await
            .map_err(map_eip_error)
    }

    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<BoxStream<'static, DriverUpdate>> {
        if addresses.is_empty() {
            return Ok(Box::pin(stream::empty()));
        }

        let client = self.client()?;
        let config = self.config.clone().unwrap_or_default();
        let group_name = format!(
            "openwebhmi-{}",
            NEXT_GROUP_ID.fetch_add(1, Ordering::Relaxed)
        );
        let tag_names = addresses
            .iter()
            .map(|address| address.raw.clone())
            .collect::<Vec<_>>();
        let tag_refs = tag_names.iter().map(String::as_str).collect::<Vec<_>>();

        let subscription = {
            let mut guard = client.lock().await;
            guard
                .upsert_tag_group(&group_name, &tag_refs, config.poll_rate_ms)
                .await
                .map_err(map_eip_error)?;
            guard
                .subscribe_tag_group(&group_name)
                .await
                .map_err(map_eip_error)?
        };

        let state = SubscriptionState {
            subscription,
            requested: addresses,
            pending: Vec::new(),
        };

        Ok(Box::pin(stream::unfold(state, |mut state| async move {
            loop {
                if let Some(update) = state.pending.pop() {
                    return Some((update, state));
                }

                let event = state.subscription.wait_for_update().await?;
                state.pending = event_to_updates(event, &state.requested);
                state.pending.reverse();
            }
        })))
    }
}

struct SubscriptionState {
    subscription: Box<dyn TagGroupSubscriptionLike>,
    requested: Vec<TagAddress>,
    pending: Vec<DriverUpdate>,
}

impl Drop for SubscriptionState {
    fn drop(&mut self) {
        self.subscription.stop();
    }
}

pub(crate) fn event_to_updates(
    event: TagGroupEvent,
    requested: &[TagAddress],
) -> Vec<DriverUpdate> {
    match event.kind {
        TagGroupEventKind::Data | TagGroupEventKind::PartialError => event
            .snapshot
            .values
            .into_iter()
            .filter_map(|value| map_value_result(value, event.snapshot.sampled_at))
            .collect(),
        _ => {
            let ts_ms = system_time_ms(event.snapshot.sampled_at);
            requested
                .iter()
                .cloned()
                .map(|address| DriverUpdate {
                    address,
                    value: TagValue::String(
                        event
                            .error
                            .clone()
                            .unwrap_or_else(|| "tag group read failed".to_string()),
                    ),
                    quality: Quality::Bad,
                    ts_ms,
                })
                .collect()
        }
    }
}

fn map_value_result(value: TagGroupValueResult, sampled_at: SystemTime) -> Option<DriverUpdate> {
    let address = TagAddress::new(value.tag_name);
    let ts_ms = system_time_ms(sampled_at);
    if let Some(error) = value.error {
        return Some(DriverUpdate {
            address,
            value: TagValue::String(error),
            quality: Quality::Bad,
            ts_ms,
        });
    }

    match value.value {
        Some(value) => match plc_to_tag_value(value) {
            Ok(value) => Some(DriverUpdate {
                address,
                value,
                quality: Quality::Good,
                ts_ms,
            }),
            Err(error) => Some(DriverUpdate {
                address,
                value: TagValue::String(error.to_string()),
                quality: Quality::Bad,
                ts_ms,
            }),
        },
        None => None,
    }
}

pub(crate) fn map_eip_error(error: EtherNetIpError) -> DriverError {
    match error {
        EtherNetIpError::TagNotFound(tag) | EtherNetIpError::Tag(tag) => {
            DriverError::InvalidAddress(tag)
        }
        EtherNetIpError::ReadError { status, message } if is_invalid_address_status(status) => {
            DriverError::InvalidAddress(message)
        }
        EtherNetIpError::CipError { code, message } if is_invalid_address_status(code) => {
            DriverError::InvalidAddress(message)
        }
        EtherNetIpError::DataTypeMismatch { expected, actual } => DriverError::UnsupportedType {
            device_type: format!("expected {expected}, got {actual}"),
        },
        EtherNetIpError::Udt(message) => DriverError::UnsupportedType {
            device_type: message,
        },
        EtherNetIpError::Io(error) => DriverError::Io(error),
        EtherNetIpError::Timeout(_)
        | EtherNetIpError::Connection(_)
        | EtherNetIpError::ConnectionLost(_) => DriverError::NotConnected,
        EtherNetIpError::Permission(message) => DriverError::RemoteFault {
            code: "permission".to_string(),
            message,
        },
        EtherNetIpError::WriteError { status, message }
        | EtherNetIpError::ReadError { status, message }
        | EtherNetIpError::CipError {
            code: status,
            message,
        } => DriverError::RemoteFault {
            code: format!("0x{status:02X}"),
            message,
        },
        EtherNetIpError::StringTooLong { .. }
        | EtherNetIpError::InvalidString { .. }
        | EtherNetIpError::Protocol(_)
        | EtherNetIpError::InvalidResponse { .. }
        | EtherNetIpError::Subscription(_)
        | EtherNetIpError::Utf8(_)
        | EtherNetIpError::Other(_)
        | EtherNetIpError::Unsupported { .. } => DriverError::Other(anyhow::anyhow!(error)),
        _ => DriverError::Other(anyhow::anyhow!(error)),
    }
}

fn is_invalid_address_status(status: u8) -> bool {
    matches!(status, 0x04 | 0x05)
}

fn system_time_ms(ts: SystemTime) -> u64 {
    ts.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
