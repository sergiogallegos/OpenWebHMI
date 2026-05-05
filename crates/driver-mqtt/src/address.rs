//! MQTT tag address parsing.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Payload decoder for generic MQTT topics.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum PayloadType {
    /// UTF-8 string payload.
    #[default]
    Utf8String,
    /// Signed 64-bit integer encoded as decimal UTF-8.
    Utf8Int,
    /// 64-bit float encoded as decimal UTF-8.
    Utf8Float,
    /// Signed 64-bit integer encoded as 8 big-endian bytes.
    RawIntBe,
    /// Signed 64-bit integer encoded as little-endian bytes.
    RawIntLe,
    /// 64-bit float encoded as 8 big-endian bytes.
    RawFloatBe,
    /// 64-bit float encoded as little-endian bytes.
    RawFloatLe,
    /// JSON payload decoded with a JSONPath expression.
    JsonPath {
        /// JSONPath expression, for example `$.outer.inner`.
        path: String,
    },
    /// Sparkplug metric protobuf payload.
    SparkplugMetric,
}

impl Serialize for PayloadType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Utf8String => serializer.serialize_str("utf8_string"),
            Self::Utf8Int => serializer.serialize_str("utf8_int"),
            Self::Utf8Float => serializer.serialize_str("utf8_float"),
            Self::RawIntBe => serializer.serialize_str("raw_int_be"),
            Self::RawIntLe => serializer.serialize_str("raw_int_le"),
            Self::RawFloatBe => serializer.serialize_str("raw_float_be"),
            Self::RawFloatLe => serializer.serialize_str("raw_float_le"),
            Self::SparkplugMetric => serializer.serialize_str("sparkplug_metric"),
            Self::JsonPath { path } => {
                #[derive(Serialize)]
                struct JsonPathWire<'a> {
                    json_path: &'a str,
                }

                JsonPathWire { json_path: path }.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for PayloadType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PayloadWire {
            Name(String),
            JsonPath { json_path: JsonPathWire },
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum JsonPathWire {
            Path(String),
            Object { path: String },
        }

        match PayloadWire::deserialize(deserializer)? {
            PayloadWire::Name(name) => match name.as_str() {
                "utf8_string" => Ok(Self::Utf8String),
                "utf8_int" => Ok(Self::Utf8Int),
                "utf8_float" => Ok(Self::Utf8Float),
                "raw_int_be" => Ok(Self::RawIntBe),
                "raw_int_le" => Ok(Self::RawIntLe),
                "raw_float_be" => Ok(Self::RawFloatBe),
                "raw_float_le" => Ok(Self::RawFloatLe),
                "sparkplug_metric" => Ok(Self::SparkplugMetric),
                other => Err(serde::de::Error::unknown_variant(
                    other,
                    &[
                        "utf8_string",
                        "utf8_int",
                        "utf8_float",
                        "raw_int_be",
                        "raw_int_le",
                        "raw_float_be",
                        "raw_float_le",
                        "json_path",
                        "sparkplug_metric",
                    ],
                )),
            },
            PayloadWire::JsonPath { json_path } => {
                let path = match json_path {
                    JsonPathWire::Path(path) | JsonPathWire::Object { path } => path,
                };
                Ok(Self::JsonPath { path })
            }
        }
    }
}

/// Parsed MQTT address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttAddress {
    /// Address kind.
    pub kind: MqttAddressKind,
    /// Concrete MQTT topic.
    pub topic: String,
}

/// MQTT address kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MqttAddressKind {
    /// Generic MQTT topic.
    Generic,
    /// Sparkplug B metric address.
    Sparkplug {
        /// Sparkplug group id.
        group_id: String,
        /// Edge node id.
        edge_node_id: String,
        /// Device id.
        device_id: String,
        /// Metric name.
        metric_name: String,
    },
}

impl MqttAddress {
    /// Parse a generic topic or Sparkplug B `spB/v1.0/.../DDATA/...` address.
    pub fn parse(raw: &str) -> Result<Self, MqttAddressError> {
        if raw.is_empty() || raw.starts_with('/') || raw.ends_with('/') {
            return Err(MqttAddressError::InvalidShape(raw.to_string()));
        }
        if raw.starts_with("spB/") {
            return parse_sparkplug(raw);
        }
        Ok(Self {
            kind: MqttAddressKind::Generic,
            topic: raw.to_string(),
        })
    }
}

fn parse_sparkplug(raw: &str) -> Result<MqttAddress, MqttAddressError> {
    let parts = raw.split('/').collect::<Vec<_>>();
    if parts.len() != 7 || parts[0] != "spB" || parts[1] != "v1.0" || parts[3] != "DDATA" {
        return Err(MqttAddressError::InvalidSparkplug(raw.to_string()));
    }
    if parts.iter().any(|part| part.is_empty()) {
        return Err(MqttAddressError::InvalidSparkplug(raw.to_string()));
    }
    Ok(MqttAddress {
        kind: MqttAddressKind::Sparkplug {
            group_id: parts[2].to_string(),
            edge_node_id: parts[4].to_string(),
            device_id: parts[5].to_string(),
            metric_name: parts[6].to_string(),
        },
        topic: raw.to_string(),
    })
}

/// MQTT address parse failure.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum MqttAddressError {
    /// Address shape is invalid.
    #[error("invalid MQTT address shape: {0}")]
    InvalidShape(String),
    /// Sparkplug B address shape is invalid.
    #[error("invalid Sparkplug B metric address: {0}")]
    InvalidSparkplug(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generic_topics() {
        let address = MqttAddress::parse("factory/line1/temperature").unwrap();
        assert_eq!(address.kind, MqttAddressKind::Generic);
    }

    #[test]
    fn parses_sparkplug_metric_address() {
        let address = MqttAddress::parse("spB/v1.0/group/DDATA/edge/device/Pressure").unwrap();
        assert_eq!(
            address.kind,
            MqttAddressKind::Sparkplug {
                group_id: "group".to_string(),
                edge_node_id: "edge".to_string(),
                device_id: "device".to_string(),
                metric_name: "Pressure".to_string()
            }
        );
    }

    #[test]
    fn rejects_malformed_sparkplug_addresses() {
        assert!(MqttAddress::parse("spB/v1.0/group/NBIRTH/edge/Pressure").is_err());
        assert!(MqttAddress::parse("spB/v1.0/group/DDATA/edge/device").is_err());
        assert!(MqttAddress::parse("/bad").is_err());
    }

    #[test]
    fn json_path_payload_type_round_trips() {
        let payload_type: PayloadType =
            serde_json::from_value(serde_json::json!({ "json_path": "$.outer.inner" })).unwrap();
        assert_eq!(
            payload_type,
            PayloadType::JsonPath {
                path: "$.outer.inner".to_string()
            }
        );
        assert_eq!(
            serde_json::to_value(payload_type).unwrap(),
            serde_json::json!({ "json_path": "$.outer.inner" })
        );
    }
}
