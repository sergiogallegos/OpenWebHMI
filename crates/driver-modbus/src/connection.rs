//! Modbus driver configuration.

use serde::{Deserialize, Serialize};

/// Driver connection and polling configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct ModbusConfig {
    /// Transport-specific endpoint.
    #[serde(flatten)]
    pub connection: Connection,
    /// Poll cycle in milliseconds for subscription streams.
    pub poll_rate_ms: u64,
    /// Connection timeout in milliseconds.
    pub connection_timeout_ms: u64,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            connection: Connection::default(),
            poll_rate_ms: 250,
            connection_timeout_ms: 5_000,
        }
    }
}

impl ModbusConfig {
    /// Parse config from a generic JSON driver config.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

/// Modbus transport selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "lowercase")]
pub enum Connection {
    /// Modbus TCP endpoint.
    Tcp {
        /// Hostname or IP address.
        host: String,
        /// TCP port, normally 502.
        #[serde(default = "default_tcp_port")]
        port: u16,
    },
    /// Modbus RTU serial endpoint.
    Rtu {
        /// Serial device path, for example `/dev/ttyUSB0`.
        device: String,
        /// Serial baud rate.
        #[serde(default = "default_baud")]
        baud: u32,
        /// Serial parity.
        #[serde(default)]
        parity: Parity,
        /// Data bits.
        #[serde(default = "default_data_bits")]
        data_bits: u8,
        /// Stop bits.
        #[serde(default)]
        stop_bits: StopBits,
    },
}

impl Default for Connection {
    fn default() -> Self {
        Self::Tcp {
            host: "127.0.0.1".to_string(),
            port: default_tcp_port(),
        }
    }
}

impl Connection {
    /// Return a human-readable endpoint label.
    pub fn endpoint_label(&self) -> String {
        match self {
            Self::Tcp { host, port } => format!("{host}:{port}"),
            Self::Rtu { device, .. } => device.clone(),
        }
    }
}

/// Serial parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    /// No parity bit.
    #[default]
    None,
    /// Odd parity.
    Odd,
    /// Even parity.
    Even,
}

/// Serial stop-bit count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StopBits {
    /// One stop bit.
    #[default]
    One,
    /// Two stop bits.
    Two,
}

fn default_tcp_port() -> u16 {
    502
}

fn default_baud() -> u32 {
    9_600
}

fn default_data_bits() -> u8 {
    8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tcp_config_defaults() {
        let config = ModbusConfig::from_value(serde_json::json!({
            "transport": "tcp",
            "host": "192.168.1.10"
        }))
        .unwrap();

        assert_eq!(
            config.connection,
            Connection::Tcp {
                host: "192.168.1.10".to_string(),
                port: 502
            }
        );
        assert_eq!(config.poll_rate_ms, 250);
    }

    #[test]
    fn parses_rtu_config() {
        let config = ModbusConfig::from_value(serde_json::json!({
            "transport": "rtu",
            "device": "/dev/ttyUSB0",
            "baud": 19200,
            "parity": "even",
            "stop_bits": "two"
        }))
        .unwrap();

        assert!(matches!(
            config.connection,
            Connection::Rtu {
                baud: 19200,
                parity: Parity::Even,
                stop_bits: StopBits::Two,
                ..
            }
        ));
    }
}
