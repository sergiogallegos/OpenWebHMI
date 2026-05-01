//! MQTT tag address parsing.

use thiserror::Error;

/// Payload decoder for generic MQTT topics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadType {
    /// UTF-8 string payload.
    #[default]
    Utf8String,
    /// Signed 64-bit integer encoded as decimal UTF-8.
    RawIntBe,
    /// Signed 64-bit integer encoded as little-endian bytes.
    RawIntLe,
    /// 64-bit float encoded as decimal UTF-8.
    RawFloatBe,
    /// 64-bit float encoded as little-endian bytes.
    RawFloatLe,
    /// Sparkplug metric protobuf payload.
    SparkplugMetric,
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
}
