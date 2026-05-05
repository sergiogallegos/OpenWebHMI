//! OPC UA driver connection configuration.

use serde::{Deserialize, Serialize};

/// OPC UA authentication mode.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum AuthMode {
    /// Anonymous session.
    #[default]
    Anonymous,
    /// Username/password session.
    Username {
        /// Username.
        username: String,
        /// Password.
        password: String,
    },
}

/// OPC UA client configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct OpcUaConfig {
    /// Endpoint URL, for example `opc.tcp://127.0.0.1:4855/`.
    pub endpoint: String,
    /// Authentication mode.
    pub auth: AuthMode,
    /// Sampling and publishing interval in milliseconds.
    pub sampling_interval_ms: u64,
    /// Browse depth limit.
    pub browse_depth: usize,
    /// Browse breadth limit per node.
    pub browse_breadth: usize,
    /// PKI directory for generated client certificates.
    pub pki_dir: String,
}

impl Default for OpcUaConfig {
    fn default() -> Self {
        Self {
            endpoint: "opc.tcp://127.0.0.1:4855/".to_string(),
            auth: AuthMode::Anonymous,
            sampling_interval_ms: 250,
            browse_depth: 4,
            browse_breadth: 100,
            pki_dir: "./pki-opcua-client".to_string(),
        }
    }
}

impl OpcUaConfig {
    /// Parse config from JSON.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
