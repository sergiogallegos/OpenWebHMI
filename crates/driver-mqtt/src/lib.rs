//! MQTT and Sparkplug B driver for OpenWebHMI.

#![deny(missing_docs)]

/// MQTT address parsing.
pub mod address;
/// MQTT connection configuration.
pub mod connection;
/// MQTT driver implementation.
pub mod driver;
/// Sparkplug B payload and state helpers.
pub mod sparkplug;

pub use address::{MqttAddress, MqttAddressError, MqttAddressKind, PayloadType};
pub use connection::{AuthMode, MqttConfig, MqttTransport, TopicConfig};
pub use driver::MqttDriver;
