//! OPC UA driver implementation.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::stream::{self, BoxStream, StreamExt};
use opcua::client::{
    ClientBuilder, IdentityToken, MonitoredItem, OnSubscriptionNotification, Session,
};
use opcua::crypto::SecurityPolicy;
use opcua::types::{
    AttributeId, BrowseDescription, BrowseDescriptionResultMask, BrowseDirection, DataValue,
    DateTime, EndpointDescription, MessageSecurityMode, MonitoredItemCreateRequest, MonitoringMode,
    MonitoringParameters, NodeClass, NodeClassMask, NodeId, ObjectId, ReadValueId, ReferenceTypeId,
    StatusCode, TimestampsToReturn, UserTokenPolicy, Variant, WriteValue,
};
use openwebhmi_driver_api::{
    Capabilities, Driver, DriverError, DriverMetadata, DriverResult, DriverUpdate, TagAddress,
    TagNode, make_metadata,
};
use openwebhmi_protocol::{Quality, TagValue};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::address::OpcUaAddress;
use crate::connection::{AuthMode, OpcUaConfig};

/// OPC UA client driver.
#[derive(Default)]
pub struct OpcUaDriver {
    session: Option<Arc<Session>>,
    event_loop: Option<JoinHandle<StatusCode>>,
    config: Option<OpcUaConfig>,
}

impl OpcUaDriver {
    /// Construct a disconnected OPC UA driver.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Driver for OpcUaDriver {
    fn metadata(&self) -> DriverMetadata {
        make_metadata(
            "OPC Foundation",
            "OPC UA",
            env!("CARGO_PKG_VERSION"),
            Capabilities::new(true, true, true, true),
        )
    }

    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()> {
        let config = OpcUaConfig::from_value(config)
            .map_err(|err| DriverError::Other(anyhow::anyhow!("invalid OPC UA config: {err}")))?;
        let mut client = ClientBuilder::new()
            .application_name("OpenWebHMI OPC UA Client")
            .application_uri("urn:openwebhmi:driver-opcua")
            .create_sample_keypair(true)
            .trust_server_certs(true)
            .verify_server_certs(false)
            .pki_dir(&config.pki_dir)
            .session_retry_initial(Duration::from_millis(250))
            .session_retry_max(Duration::from_secs(8))
            .client()
            .map_err(|errs| DriverError::Other(anyhow::anyhow!(errs.join("; "))))?;
        let endpoint: EndpointDescription = (
            config.endpoint.as_str(),
            SecurityPolicy::None.to_str(),
            MessageSecurityMode::None,
            UserTokenPolicy::anonymous(),
        )
            .into();
        let identity = match &config.auth {
            AuthMode::Anonymous => IdentityToken::Anonymous,
            AuthMode::Username { username, password } => {
                IdentityToken::UserName(username.clone(), password.clone().into())
            }
        };
        let (session, event_loop) = client
            .connect_to_matching_endpoint(endpoint, identity)
            .await
            .map_err(map_opcua_error)?;
        let handle = event_loop.spawn();
        tokio::time::timeout(Duration::from_secs(5), session.wait_for_connection())
            .await
            .map_err(|_| DriverError::NotConnected)?;

        self.session = Some(session);
        self.event_loop = Some(handle);
        self.config = Some(config);
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if let Some(handle) = self.event_loop.take() {
            handle.abort();
        }
        self.session = None;
        self.config = None;
        Ok(())
    }

    async fn browse(&self, path: Option<&str>) -> DriverResult<Vec<TagNode>> {
        let session = self.session()?;
        let config = self.config.as_ref().ok_or(DriverError::NotConnected)?;
        let root = match path {
            Some(path) => parse_address(path)?,
            None => ObjectId::ObjectsFolder.into(),
        };
        browse_level(&session, root, config.browse_depth, config.browse_breadth).await
    }

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
        let session = self.session()?;
        let node_id = parse_address(&address.raw)?;
        let values = session
            .read(
                &[ReadValueId::new_value(node_id)],
                TimestampsToReturn::Both,
                0.0,
            )
            .await
            .map_err(map_status)?;
        let data_value = values
            .into_iter()
            .next()
            .ok_or_else(|| DriverError::RemoteFault {
                code: "opcua.empty-read".to_string(),
                message: "OPC UA read returned no results".to_string(),
            })?;
        if !data_value.status().is_good() {
            return Err(map_status(data_value.status()));
        }
        data_value_to_tag_value(data_value)
    }

    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
        let session = self.session()?;
        let node_id = parse_address(&address.raw)?;
        let status = session
            .write(&[WriteValue {
                node_id,
                attribute_id: AttributeId::Value as u32,
                index_range: Default::default(),
                value: DataValue {
                    value: Some(tag_value_to_variant(value)?),
                    status: Some(StatusCode::Good),
                    source_timestamp: Some(DateTime::now()),
                    ..Default::default()
                },
            }])
            .await
            .map_err(map_status)?
            .into_iter()
            .next()
            .unwrap_or(StatusCode::BadNoData);
        if status.is_good() {
            Ok(())
        } else {
            Err(map_status(status))
        }
    }

    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<BoxStream<'static, DriverUpdate>> {
        if addresses.is_empty() {
            return Ok(Box::pin(stream::empty()));
        }

        let session = self.session()?;
        let config = self.config.as_ref().ok_or(DriverError::NotConnected)?;
        let (tx, rx) = mpsc::unbounded_channel();
        let sub_id = session
            .create_subscription(
                Duration::from_millis(config.sampling_interval_ms),
                100,
                20,
                addresses.len() as u32,
                0,
                true,
                SubscriptionForwarder { tx: tx.clone() },
            )
            .await
            .map_err(map_status)?;
        let items = addresses
            .into_iter()
            .map(|address| {
                let node_id = parse_address(&address.raw)?;
                Ok(MonitoredItemCreateRequest::new(
                    ReadValueId::new_value(node_id),
                    MonitoringMode::Reporting,
                    MonitoringParameters {
                        sampling_interval: config.sampling_interval_ms as f64,
                        queue_size: 10,
                        discard_oldest: true,
                        ..Default::default()
                    },
                ))
            })
            .collect::<DriverResult<Vec<_>>>()?;
        let results = session
            .create_monitored_items(sub_id, TimestampsToReturn::Both, items)
            .await
            .map_err(map_status)?;
        for result in results {
            if !result.result.status_code.is_good() {
                return Err(map_status(result.result.status_code));
            }
        }

        Ok(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx).boxed(),
        ))
    }
}

impl OpcUaDriver {
    fn session(&self) -> DriverResult<Arc<Session>> {
        self.session.clone().ok_or(DriverError::NotConnected)
    }
}

#[derive(Clone)]
struct SubscriptionForwarder {
    tx: mpsc::UnboundedSender<DriverUpdate>,
}

impl OnSubscriptionNotification for SubscriptionForwarder {
    fn on_data_value(&mut self, notification: DataValue, item: &MonitoredItem) {
        let ts_ms = now_ms();
        let node_id = &item.item_to_monitor().node_id;
        let address = TagAddress::new(format!("{node_id}"));
        let quality = status_to_quality(notification.status());
        let value = data_value_to_tag_value(notification)
            .unwrap_or_else(|err| TagValue::String(err.to_string()));
        let _ = self.tx.send(DriverUpdate {
            address,
            value,
            quality,
            ts_ms,
        });
    }
}

/// Map an OPC UA status code to OpenWebHMI quality.
pub fn status_to_quality(status: StatusCode) -> Quality {
    if status.is_good() {
        Quality::Good
    } else if status.is_uncertain() {
        Quality::Uncertain
    } else {
        Quality::Bad
    }
}

fn parse_address(raw: &str) -> DriverResult<NodeId> {
    OpcUaAddress::parse(raw)
        .and_then(|address| address.to_node_id())
        .map_err(|err| DriverError::InvalidAddress(err.to_string()))
}

fn data_value_to_tag_value(data_value: DataValue) -> DriverResult<TagValue> {
    let value = data_value.value.ok_or_else(|| DriverError::RemoteFault {
        code: "opcua.no-value".to_string(),
        message: "OPC UA DataValue did not contain a value".to_string(),
    })?;
    variant_to_tag_value(value)
}

fn variant_to_tag_value(value: Variant) -> DriverResult<TagValue> {
    match value {
        Variant::Boolean(value) => Ok(TagValue::Bool(value)),
        Variant::SByte(value) => Ok(TagValue::Int(value as i64)),
        Variant::Byte(value) => Ok(TagValue::Int(value as i64)),
        Variant::Int16(value) => Ok(TagValue::Int(value as i64)),
        Variant::UInt16(value) => Ok(TagValue::Int(value as i64)),
        Variant::Int32(value) => Ok(TagValue::Int(value as i64)),
        Variant::UInt32(value) => Ok(TagValue::Int(value as i64)),
        Variant::Int64(value) => Ok(TagValue::Int(value)),
        Variant::UInt64(value) if value <= i64::MAX as u64 => Ok(TagValue::Int(value as i64)),
        Variant::Float(value) => Ok(TagValue::Real(value as f64)),
        Variant::Double(value) => Ok(TagValue::Real(value)),
        Variant::String(value) => Ok(TagValue::String(value.to_string())),
        other => Err(DriverError::UnsupportedType {
            device_type: format!("{other:?}"),
        }),
    }
}

fn tag_value_to_variant(value: TagValue) -> DriverResult<Variant> {
    Ok(match value {
        TagValue::Bool(value) => Variant::Boolean(value),
        TagValue::Int(value) if (i32::MIN as i64..=i32::MAX as i64).contains(&value) => {
            Variant::Int32(value as i32)
        }
        TagValue::Int(value) => Variant::Int64(value),
        TagValue::Real(value) => Variant::Double(value),
        TagValue::String(value) => Variant::String(value.into()),
    })
}

async fn browse_level(
    session: &Arc<Session>,
    root: NodeId,
    depth: usize,
    breadth: usize,
) -> DriverResult<Vec<TagNode>> {
    if depth == 0 {
        return Ok(Vec::new());
    }
    let results = session
        .browse(
            &[BrowseDescription {
                node_id: root,
                browse_direction: BrowseDirection::Forward,
                reference_type_id: ReferenceTypeId::HierarchicalReferences.into(),
                include_subtypes: true,
                node_class_mask: (NodeClassMask::OBJECT | NodeClassMask::VARIABLE).bits(),
                result_mask: BrowseDescriptionResultMask::all().bits(),
            }],
            breadth as u32,
            None,
        )
        .await
        .map_err(map_status)?;
    let Some(result) = results.into_iter().next() else {
        return Ok(Vec::new());
    };
    if !result.status_code.is_good() {
        return Err(map_status(result.status_code));
    }

    let mut nodes = Vec::new();
    for reference in result
        .references
        .unwrap_or_default()
        .into_iter()
        .take(breadth)
    {
        let node_id = reference.node_id.node_id.clone();
        let address =
            if reference.node_id.server_index == 0 && reference.node_id.namespace_uri.is_null() {
                Some(TagAddress::new(format!("{node_id}")))
            } else {
                None
            };
        let children = if reference.node_class == NodeClass::Object {
            Box::pin(browse_level(session, node_id, depth - 1, breadth)).await?
        } else {
            Vec::new()
        };
        nodes.push(TagNode {
            name: reference.display_name.text.to_string(),
            address,
            data_type: Some(format!("{:?}", reference.node_class)),
            children,
        });
    }
    Ok(nodes)
}

fn map_status(status: StatusCode) -> DriverError {
    if status == StatusCode::BadNodeIdUnknown || status == StatusCode::BadAttributeIdInvalid {
        DriverError::InvalidAddress(status.to_string())
    } else {
        DriverError::RemoteFault {
            code: format!("0x{:08X}", status.bits()),
            message: status.to_string(),
        }
    }
}

fn map_opcua_error(error: opcua::types::Error) -> DriverError {
    map_status(error.status())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use openwebhmi_driver_api::Driver;

    #[test]
    fn quality_mapping_covers_good_bad_uncertain() {
        assert_eq!(status_to_quality(StatusCode::Good), Quality::Good);
        assert_eq!(status_to_quality(StatusCode::Bad), Quality::Bad);
        assert_eq!(status_to_quality(StatusCode::Uncertain), Quality::Uncertain);
    }

    #[tokio::test]
    async fn metadata_matches_opcua_capabilities() {
        let driver = OpcUaDriver::new();
        let metadata = driver.metadata();
        assert!(metadata.capabilities.native_subscribe);
        assert!(metadata.capabilities.browse);
        assert!(metadata.capabilities.batch_read);
        assert!(metadata.capabilities.batch_write);
    }
}
