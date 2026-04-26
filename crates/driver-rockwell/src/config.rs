//! Rockwell driver configuration.

use serde::{Deserialize, Serialize};

/// Configuration for a Rockwell EtherNet/IP connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RockwellConfig {
    /// PLC host or `host:port` endpoint.
    pub host: String,
    /// Backplane slot for ControlLogix; `0` for common CompactLogix setups.
    pub slot: u8,
    /// Optional CIP route path such as `1,0`.
    pub route: Option<String>,
    /// Subscription poll rate in milliseconds.
    pub poll_rate_ms: u32,
    /// Connection timeout in milliseconds.
    pub connection_timeout_ms: u32,
}

impl Default for RockwellConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            slot: 0,
            route: None,
            poll_rate_ms: 250,
            connection_timeout_ms: 5_000,
        }
    }
}

impl RockwellConfig {
    /// Parse config from the generic driver JSON payload.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Return the socket address string expected by `rust-ethernet-ip`.
    pub fn endpoint(&self) -> String {
        if self.host.contains(':') {
            self.host.clone()
        } else {
            format!("{}:44818", self.host)
        }
    }
}
