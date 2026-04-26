//! Driver metadata and capability flags.

use serde::{Deserialize, Serialize};

/// Static metadata reported by a driver implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverMetadata {
    /// Device vendor name.
    pub vendor: String,
    /// Protocol or family name, for example `EtherNet/IP-CIP`.
    pub family: String,
    /// Driver crate version.
    pub crate_version: String,
    /// Capabilities supported by this driver.
    pub capabilities: Capabilities,
}

/// Capability flags used by the gateway and designer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Driver can stream updates without supervisor polling.
    pub native_subscribe: bool,
    /// Driver can browse remote tags.
    pub browse: bool,
    /// Driver supports batch reads.
    pub batch_read: bool,
    /// Driver supports batch writes.
    pub batch_write: bool,
}

impl Capabilities {
    /// No optional capabilities.
    pub const NONE: Self = Self {
        native_subscribe: false,
        browse: false,
        batch_read: false,
        batch_write: false,
    };
}
