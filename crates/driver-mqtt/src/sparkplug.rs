//! Sparkplug B payload parsing.
//!
//! Implements the v1 subset needed from Sparkplug B 3.0.0 payloads: NBIRTH/DBIRTH
//! establish metric alias maps, and NDATA/DDATA resolve alias-only metrics.

use std::collections::HashMap;

use openwebhmi_protocol::{Quality, TagValue};
use prost::Message;
use thiserror::Error;

/// Sparkplug B protobuf payload.
#[derive(Clone, PartialEq, Message)]
pub struct Payload {
    /// Payload timestamp.
    #[prost(uint64, optional, tag = "1")]
    pub timestamp: Option<u64>,
    /// Payload metrics.
    #[prost(message, repeated, tag = "2")]
    pub metrics: Vec<Metric>,
    /// Sparkplug sequence number.
    #[prost(uint64, optional, tag = "3")]
    pub seq: Option<u64>,
}

/// Sparkplug B metric.
#[derive(Clone, PartialEq, Message)]
pub struct Metric {
    /// Metric name.
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,
    /// Metric alias.
    #[prost(uint64, optional, tag = "2")]
    pub alias: Option<u64>,
    /// Metric timestamp.
    #[prost(uint64, optional, tag = "3")]
    pub timestamp: Option<u64>,
    /// Sparkplug datatype enum value.
    #[prost(uint32, optional, tag = "4")]
    pub datatype: Option<u32>,
    /// Historical flag.
    #[prost(bool, optional, tag = "5")]
    pub is_historical: Option<bool>,
    /// Transient flag.
    #[prost(bool, optional, tag = "6")]
    pub is_transient: Option<bool>,
    /// Null flag.
    #[prost(bool, optional, tag = "7")]
    pub is_null: Option<bool>,
    /// Metric value.
    #[prost(oneof = "metric::Value", tags = "10, 11, 12, 13, 14, 15, 16")]
    pub value: Option<metric::Value>,
}

/// Metric oneof namespace.
pub mod metric {
    /// Sparkplug B metric value.
    #[allow(clippy::enum_variant_names)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// 32-bit integer-like value.
        #[prost(uint32, tag = "10")]
        IntValue(u32),
        /// 64-bit integer-like value.
        #[prost(uint64, tag = "11")]
        LongValue(u64),
        /// 32-bit float.
        #[prost(float, tag = "12")]
        FloatValue(f32),
        /// 64-bit float.
        #[prost(double, tag = "13")]
        DoubleValue(f64),
        /// Boolean.
        #[prost(bool, tag = "14")]
        BooleanValue(bool),
        /// String.
        #[prost(string, tag = "15")]
        StringValue(String),
        /// Raw bytes.
        #[prost(bytes, tag = "16")]
        BytesValue(Vec<u8>),
    }
}

/// Sparkplug namespace key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SparkplugKey {
    /// Group id.
    pub group_id: String,
    /// Edge node id.
    pub edge_node_id: String,
    /// Device id, empty for node-level metrics.
    pub device_id: String,
}

/// Resolved Sparkplug metric update.
#[derive(Debug, Clone, PartialEq)]
pub struct SparkplugMetricUpdate {
    /// Metric name.
    pub metric_name: String,
    /// Metric value.
    pub value: TagValue,
    /// Mapped quality.
    pub quality: Quality,
}

/// Sparkplug B alias-map state.
#[derive(Default)]
pub struct SparkplugState {
    aliases: HashMap<SparkplugKey, HashMap<u64, String>>,
}

impl SparkplugState {
    /// Apply a BIRTH payload and update aliases.
    pub fn apply_birth(
        &mut self,
        key: SparkplugKey,
        bytes: &[u8],
    ) -> Result<Vec<SparkplugMetricUpdate>, SparkplugError> {
        let payload = Payload::decode(bytes)?;
        let mut aliases = HashMap::new();
        let updates = payload
            .metrics
            .into_iter()
            .filter_map(|metric| {
                if let (Some(alias), Some(name)) = (metric.alias, metric.name.clone()) {
                    aliases.insert(alias, name);
                }
                metric_to_update(metric).transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.aliases.insert(key, aliases);
        Ok(updates)
    }

    /// Apply a DATA payload and resolve aliases through the latest BIRTH map.
    pub fn apply_data(
        &self,
        key: &SparkplugKey,
        bytes: &[u8],
    ) -> Result<Vec<SparkplugMetricUpdate>, SparkplugError> {
        let payload = Payload::decode(bytes)?;
        let aliases = self.aliases.get(key);
        payload
            .metrics
            .into_iter()
            .filter_map(|mut metric| {
                if metric.name.is_none() {
                    let alias = metric.alias?;
                    let Some(name) = aliases.and_then(|map| map.get(&alias)).cloned() else {
                        tracing::warn!(alias, ?key, "Sparkplug DATA before BIRTH or unknown alias");
                        return None;
                    };
                    metric.name = Some(name);
                }
                metric_to_update(metric).transpose()
            })
            .collect()
    }
}

fn metric_to_update(metric: Metric) -> Result<Option<SparkplugMetricUpdate>, SparkplugError> {
    let Some(metric_name) = metric.name else {
        return Ok(None);
    };
    if metric.is_null.unwrap_or(false) {
        return Ok(None);
    }
    let value = match metric.value.ok_or(SparkplugError::MissingValue)? {
        metric::Value::IntValue(value) => TagValue::Int(value as i64),
        metric::Value::LongValue(value) if value <= i64::MAX as u64 => TagValue::Int(value as i64),
        metric::Value::LongValue(value) => TagValue::String(value.to_string()),
        metric::Value::FloatValue(value) => TagValue::Real(value as f64),
        metric::Value::DoubleValue(value) => TagValue::Real(value),
        metric::Value::BooleanValue(value) => TagValue::Bool(value),
        metric::Value::StringValue(value) => TagValue::String(value),
        metric::Value::BytesValue(value) => TagValue::String(format!("{value:?}")),
    };
    let quality = if metric.is_transient.unwrap_or(false) {
        Quality::Uncertain
    } else {
        Quality::Good
    };
    Ok(Some(SparkplugMetricUpdate {
        metric_name,
        value,
        quality,
    }))
}

/// Sparkplug B parse failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SparkplugError {
    /// Protobuf decode failed.
    #[error("Sparkplug payload decode failed: {0}")]
    Decode(#[from] prost::DecodeError),
    /// Metric had no value.
    #[error("Sparkplug metric had no value")]
    MissingValue,
}

/// Encode a simple Sparkplug payload for simulator/tests.
pub fn encode_payload(metrics: Vec<Metric>) -> Vec<u8> {
    Payload {
        timestamp: None,
        metrics,
        seq: None,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(edge: &str) -> SparkplugKey {
        SparkplugKey {
            group_id: "group".to_string(),
            edge_node_id: edge.to_string(),
            device_id: "device".to_string(),
        }
    }

    #[test]
    fn birth_then_data_resolves_alias() {
        let mut state = SparkplugState::default();
        let birth = encode_payload(vec![Metric {
            name: Some("Pressure".to_string()),
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(10.0)),
            ..Default::default()
        }]);
        state.apply_birth(key("edge"), &birth).unwrap();
        let data = encode_payload(vec![Metric {
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(11.5)),
            ..Default::default()
        }]);
        let updates = state.apply_data(&key("edge"), &data).unwrap();
        assert_eq!(updates[0].metric_name, "Pressure");
        assert_eq!(updates[0].value, TagValue::Real(11.5));
    }

    #[test]
    fn data_before_birth_is_ignored() {
        let data = encode_payload(vec![Metric {
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(11.5)),
            ..Default::default()
        }]);
        let updates = SparkplugState::default()
            .apply_data(&key("edge"), &data)
            .unwrap();
        assert!(updates.is_empty());
    }

    #[test]
    fn alias_maps_are_scoped_by_edge() {
        let mut state = SparkplugState::default();
        let birth_a = encode_payload(vec![Metric {
            name: Some("Pressure".to_string()),
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(10.0)),
            ..Default::default()
        }]);
        let birth_b = encode_payload(vec![Metric {
            name: Some("Temperature".to_string()),
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(20.0)),
            ..Default::default()
        }]);
        state.apply_birth(key("edge-a"), &birth_a).unwrap();
        state.apply_birth(key("edge-b"), &birth_b).unwrap();
        let data = encode_payload(vec![Metric {
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(21.0)),
            ..Default::default()
        }]);
        let updates = state.apply_data(&key("edge-b"), &data).unwrap();
        assert_eq!(updates[0].metric_name, "Temperature");
    }
}
