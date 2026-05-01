//! MQTT driver implementation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::stream::{self, BoxStream, StreamExt};
use openwebhmi_driver_api::{
    make_metadata, Capabilities, Driver, DriverError, DriverMetadata, DriverResult, DriverUpdate,
    TagAddress, TagNode,
};
use openwebhmi_protocol::{Quality, TagValue};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, Publish, QoS};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

use crate::address::{MqttAddress, MqttAddressKind, PayloadType};
use crate::connection::{AuthMode, MqttConfig, MqttTransport};
use crate::sparkplug::{SparkplugKey, SparkplugState};

type Cache = Arc<RwLock<HashMap<String, DriverUpdate>>>;

/// MQTT client driver.
#[derive(Default)]
pub struct MqttDriver {
    client: Option<AsyncClient>,
    cache: Cache,
    tx: Option<broadcast::Sender<DriverUpdate>>,
    worker: Option<JoinHandle<()>>,
    config: Option<MqttConfig>,
}

impl MqttDriver {
    /// Construct a disconnected MQTT driver.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Driver for MqttDriver {
    fn metadata(&self) -> DriverMetadata {
        make_metadata(
            "MQTT",
            "MQTT + Sparkplug B",
            env!("CARGO_PKG_VERSION"),
            Capabilities {
                native_subscribe: true,
                browse: true,
                batch_read: false,
                batch_write: true,
            },
        )
    }

    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()> {
        let config = MqttConfig::from_value(config)
            .map_err(|err| DriverError::Other(anyhow::anyhow!("invalid MQTT config: {err}")))?;
        if !matches!(config.transport, MqttTransport::Tcp) {
            return Err(DriverError::UnsupportedType {
                device_type: "MQTT TLS/WebSocket config is reserved for v1 hardening".to_string(),
            });
        }
        let mut options = MqttOptions::new(&config.client_id, &config.host, config.port);
        options.set_keep_alive(Duration::from_secs(config.keep_alive_secs));
        if let AuthMode::Username { username, password } = &config.auth {
            options.set_credentials(username, password);
        }

        let (client, mut event_loop) = AsyncClient::new(options, 32);
        for topic in subscription_filters(&config) {
            client
                .subscribe(topic, QoS::AtLeastOnce)
                .await
                .map_err(map_client_error)?;
        }

        let (tx, _) = broadcast::channel(256);
        let worker_tx = tx.clone();
        let cache = Arc::clone(&self.cache);
        let mappings = topic_mappings(&config)?;
        let worker = tokio::spawn(async move {
            let mut sparkplug = SparkplugState::default();
            loop {
                match event_loop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        handle_publish(publish, &mappings, &cache, &worker_tx, &mut sparkplug)
                            .await;
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::warn!(%err, "MQTT event loop stopped");
                        break;
                    }
                }
            }
            mark_bad(&cache, &worker_tx).await;
        });

        self.client = Some(client);
        self.tx = Some(tx);
        self.worker = Some(worker);
        self.config = Some(config);
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        if let Some(worker) = self.worker.take() {
            worker.abort();
        }
        self.client = None;
        self.tx = None;
        self.config = None;
        Ok(())
    }

    async fn browse(&self, _path: Option<&str>) -> DriverResult<Vec<TagNode>> {
        let Some(config) = &self.config else {
            return Err(DriverError::NotConnected);
        };
        Ok(config
            .topics
            .iter()
            .map(|topic| TagNode {
                name: topic.address.clone(),
                address: Some(TagAddress::new(topic.address.clone())),
                data_type: Some(format!("{:?}", topic.payload_type)),
                children: Vec::new(),
            })
            .collect())
    }

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
        let cache = self.cache.read().await;
        cache
            .get(&address.raw)
            .map(|update| update.value.clone())
            .ok_or(DriverError::NotConnected)
    }

    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
        let client = self.client.as_ref().ok_or(DriverError::NotConnected)?;
        client
            .publish(
                &address.raw,
                QoS::AtLeastOnce,
                false,
                tag_value_to_payload(value),
            )
            .await
            .map_err(map_client_error)
    }

    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<BoxStream<'static, DriverUpdate>> {
        let Some(tx) = &self.tx else {
            return Err(DriverError::NotConnected);
        };
        let wanted = addresses
            .into_iter()
            .map(|address| address.raw)
            .collect::<HashSet<_>>();
        if wanted.is_empty() {
            return Ok(Box::pin(stream::empty()));
        }
        let rx = tx.subscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |item| {
                let wanted = wanted.clone();
                async move {
                    match item {
                        Ok(update) if wanted.contains(&update.address.raw) => Some(update),
                        _ => None,
                    }
                }
            })
            .boxed();
        Ok(Box::pin(stream))
    }
}

#[derive(Debug, Clone)]
struct TopicMapping {
    address: String,
    payload_type: PayloadType,
}

fn subscription_filters(config: &MqttConfig) -> HashSet<String> {
    let mut filters = HashSet::new();
    for topic in &config.topics {
        filters.insert(topic.topic.clone().unwrap_or_else(|| topic.address.clone()));
        if matches!(topic.payload_type, PayloadType::SparkplugMetric) {
            if let Ok(parsed) = MqttAddress::parse(&topic.address) {
                if let MqttAddressKind::Sparkplug {
                    group_id,
                    edge_node_id,
                    device_id,
                    ..
                } = parsed.kind
                {
                    filters.insert(format!("spB/v1.0/{group_id}/NBIRTH/{edge_node_id}"));
                    filters.insert(format!(
                        "spB/v1.0/{group_id}/DBIRTH/{edge_node_id}/{device_id}"
                    ));
                    filters.insert(format!(
                        "spB/v1.0/{group_id}/DDATA/{edge_node_id}/{device_id}"
                    ));
                }
            }
        }
    }
    filters
}

fn topic_mappings(config: &MqttConfig) -> DriverResult<HashMap<String, Vec<TopicMapping>>> {
    let mut mappings: HashMap<String, Vec<TopicMapping>> = HashMap::new();
    for topic in &config.topics {
        let parsed = MqttAddress::parse(&topic.address)
            .map_err(|err| DriverError::InvalidAddress(err.to_string()))?;
        let subscribe_topic = topic.topic.clone().unwrap_or_else(|| parsed.topic.clone());
        mappings
            .entry(subscribe_topic)
            .or_default()
            .push(TopicMapping {
                address: topic.address.clone(),
                payload_type: topic.payload_type,
            });
        if matches!(topic.payload_type, PayloadType::SparkplugMetric) {
            if let MqttAddressKind::Sparkplug {
                group_id,
                edge_node_id,
                device_id,
                ..
            } = parsed.kind
            {
                mappings
                    .entry(format!(
                        "spB/v1.0/{group_id}/DDATA/{edge_node_id}/{device_id}"
                    ))
                    .or_default()
                    .push(TopicMapping {
                        address: topic.address.clone(),
                        payload_type: topic.payload_type,
                    });
            }
        }
    }
    Ok(mappings)
}

async fn handle_publish(
    publish: Publish,
    mappings: &HashMap<String, Vec<TopicMapping>>,
    cache: &Cache,
    tx: &broadcast::Sender<DriverUpdate>,
    sparkplug: &mut SparkplugState,
) {
    let topic = publish.topic.clone();
    if let Some((key, _birth_kind)) = parse_birth_topic(&topic) {
        let result = sparkplug.apply_birth(key, &publish.payload);
        if let Err(err) = result {
            tracing::warn!(%err, topic, "failed to apply Sparkplug birth");
        }
        return;
    }
    let Some(topic_mappings) = mappings.get(&topic) else {
        return;
    };
    for mapping in topic_mappings {
        let updates = decode_mapping(mapping, &topic, &publish.payload, sparkplug);
        for (address, value, quality) in updates {
            publish_update(cache, tx, address, value, quality).await;
        }
    }
}

fn decode_mapping(
    mapping: &TopicMapping,
    topic: &str,
    payload: &[u8],
    sparkplug: &mut SparkplugState,
) -> Vec<(String, TagValue, Quality)> {
    if mapping.payload_type == PayloadType::SparkplugMetric {
        return decode_sparkplug(mapping, topic, payload, sparkplug);
    }
    match decode_payload(mapping.payload_type, payload) {
        Ok(value) => vec![(mapping.address.clone(), value, Quality::Good)],
        Err(err) => {
            tracing::warn!(%err, address = %mapping.address, "failed to decode MQTT payload");
            Vec::new()
        }
    }
}

fn decode_sparkplug(
    mapping: &TopicMapping,
    topic: &str,
    payload: &[u8],
    sparkplug: &mut SparkplugState,
) -> Vec<(String, TagValue, Quality)> {
    let Some(key) = parse_data_topic(topic) else {
        return Vec::new();
    };
    let metric_name = match MqttAddress::parse(&mapping.address) {
        Ok(MqttAddress {
            kind: MqttAddressKind::Sparkplug { metric_name, .. },
            ..
        }) => metric_name,
        _ => return Vec::new(),
    };
    match sparkplug.apply_data(&key, payload) {
        Ok(updates) => updates
            .into_iter()
            .filter(|update| update.metric_name == metric_name)
            .map(|update| (mapping.address.clone(), update.value, update.quality))
            .collect(),
        Err(err) => {
            tracing::warn!(%err, topic, "failed to decode Sparkplug DATA");
            Vec::new()
        }
    }
}

fn parse_birth_topic(topic: &str) -> Option<(SparkplugKey, &'static str)> {
    let parts = topic.split('/').collect::<Vec<_>>();
    if parts.len() == 5 && parts[0] == "spB" && parts[1] == "v1.0" && parts[3] == "NBIRTH" {
        return Some((
            SparkplugKey {
                group_id: parts[2].to_string(),
                edge_node_id: parts[4].to_string(),
                device_id: String::new(),
            },
            "NBIRTH",
        ));
    }
    if parts.len() == 6 && parts[0] == "spB" && parts[1] == "v1.0" && parts[3] == "DBIRTH" {
        return Some((
            SparkplugKey {
                group_id: parts[2].to_string(),
                edge_node_id: parts[4].to_string(),
                device_id: parts[5].to_string(),
            },
            "DBIRTH",
        ));
    }
    None
}

fn parse_data_topic(topic: &str) -> Option<SparkplugKey> {
    let parts = topic.split('/').collect::<Vec<_>>();
    if parts.len() == 6 && parts[0] == "spB" && parts[1] == "v1.0" && parts[3] == "DDATA" {
        return Some(SparkplugKey {
            group_id: parts[2].to_string(),
            edge_node_id: parts[4].to_string(),
            device_id: parts[5].to_string(),
        });
    }
    None
}

async fn publish_update(
    cache: &Cache,
    tx: &broadcast::Sender<DriverUpdate>,
    address: String,
    value: TagValue,
    quality: Quality,
) {
    let update = DriverUpdate {
        address: TagAddress::new(address.clone()),
        value,
        quality,
        ts_ms: now_ms(),
    };
    cache.write().await.insert(address, update.clone());
    let _ = tx.send(update);
}

async fn mark_bad(cache: &Cache, tx: &broadcast::Sender<DriverUpdate>) {
    let updates = cache.read().await.values().cloned().collect::<Vec<_>>();
    for mut update in updates {
        update.quality = Quality::Bad;
        let _ = tx.send(update);
    }
}

fn decode_payload(payload_type: PayloadType, payload: &[u8]) -> DriverResult<TagValue> {
    match payload_type {
        PayloadType::Utf8String => Ok(TagValue::String(
            std::str::from_utf8(payload)
                .map_err(|err| DriverError::Other(anyhow::anyhow!(err)))?
                .to_string(),
        )),
        PayloadType::RawIntBe => Ok(TagValue::Int(
            std::str::from_utf8(payload)
                .map_err(|err| DriverError::Other(anyhow::anyhow!(err)))?
                .parse::<i64>()
                .map_err(|err| DriverError::Other(anyhow::anyhow!(err)))?,
        )),
        PayloadType::RawFloatBe => Ok(TagValue::Real(
            std::str::from_utf8(payload)
                .map_err(|err| DriverError::Other(anyhow::anyhow!(err)))?
                .parse::<f64>()
                .map_err(|err| DriverError::Other(anyhow::anyhow!(err)))?,
        )),
        PayloadType::RawIntLe if payload.len() == 8 => {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(payload);
            Ok(TagValue::Int(i64::from_le_bytes(bytes)))
        }
        PayloadType::RawFloatLe if payload.len() == 8 => {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(payload);
            Ok(TagValue::Real(f64::from_le_bytes(bytes)))
        }
        PayloadType::SparkplugMetric => Err(DriverError::UnsupportedType {
            device_type: "Sparkplug payloads are decoded via Sparkplug mappings".to_string(),
        }),
        other => Err(DriverError::UnsupportedType {
            device_type: format!("invalid payload length for {other:?}"),
        }),
    }
}

fn tag_value_to_payload(value: TagValue) -> Vec<u8> {
    match value {
        TagValue::Bool(value) => value.to_string().into_bytes(),
        TagValue::Int(value) => value.to_string().into_bytes(),
        TagValue::Real(value) => value.to_string().into_bytes(),
        TagValue::String(value) => value.into_bytes(),
    }
}

fn map_client_error(error: rumqttc::ClientError) -> DriverError {
    DriverError::RemoteFault {
        code: "mqtt.client".to_string(),
        message: error.to_string(),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
