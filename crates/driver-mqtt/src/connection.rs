//! MQTT driver connection configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::address::PayloadType;

/// MQTT authentication mode.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum AuthMode {
    /// Anonymous connection.
    #[default]
    Anonymous,
    /// Username/password connection.
    Username {
        /// Username.
        username: String,
        /// Password.
        password: String,
    },
}

/// MQTT transport mode.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MqttTransport {
    /// Plain TCP MQTT.
    #[default]
    Tcp,
    /// TLS using system trust roots.
    Tls,
    /// WebSocket MQTT.
    WebSocket,
}

/// Configured topic/tag mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicConfig {
    /// OpenWebHMI tag address.
    pub address: String,
    /// MQTT subscription topic/filter. Defaults to `address`.
    #[serde(default)]
    pub topic: Option<String>,
    /// Payload decoder.
    #[serde(default)]
    pub payload_type: PayloadType,
}

/// MQTT driver configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MqttConfig {
    /// Broker host.
    pub host: String,
    /// Broker port.
    pub port: u16,
    /// Client id.
    pub client_id: String,
    /// Keep-alive seconds.
    pub keep_alive_secs: u64,
    /// Transport mode.
    pub transport: MqttTransport,
    /// Allow insecure TLS verification. Logged as a deployment warning when enabled.
    pub tls_insecure: bool,
    /// Optional PEM CA certificate path for TLS or WSS.
    pub ca_cert_path: Option<PathBuf>,
    /// WebSocket path. Defaults to `/mqtt`.
    pub ws_path: Option<String>,
    /// Authentication mode.
    pub auth: AuthMode,
    /// Configured topic/tag mappings.
    pub topics: Vec<TopicConfig>,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1883,
            client_id: "openwebhmi-mqtt".to_string(),
            keep_alive_secs: 5,
            transport: MqttTransport::Tcp,
            tls_insecure: false,
            ca_cert_path: None,
            ws_path: None,
            auth: AuthMode::Anonymous,
            topics: Vec::new(),
        }
    }
}

impl MqttConfig {
    /// Parse config from JSON.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
