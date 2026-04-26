//! Driver error taxonomy.

use openwebhmi_protocol::Quality;
use thiserror::Error;

/// Result alias for driver operations.
pub type DriverResult<T> = Result<T, DriverError>;

/// Stable error classes returned by drivers.
#[derive(Debug, Error)]
pub enum DriverError {
    /// Driver is not currently connected.
    #[error("driver is not connected")]
    NotConnected,
    /// Connection is currently in progress.
    #[error("driver is connecting")]
    Connecting,
    /// Address syntax is invalid for this driver.
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    /// Remote device refused, timed out, or reported a fault.
    #[error("remote fault {code}: {message}")]
    RemoteFault {
        /// Machine-readable remote fault code.
        code: String,
        /// Human-readable remote fault message.
        message: String,
    },
    /// Device type cannot be mapped to `TagValue`.
    #[error("unsupported device type: {device_type}")]
    UnsupportedType {
        /// Driver-specific device type name.
        device_type: String,
    },
    /// TCP or OS-level I/O failure.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Driver-specific escape hatch.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl DriverError {
    /// Map an error to the quality that dependent tags should report.
    pub fn into_quality(&self) -> Quality {
        match self {
            Self::Connecting => Quality::Uncertain,
            Self::NotConnected
            | Self::InvalidAddress(_)
            | Self::RemoteFault { .. }
            | Self::UnsupportedType { .. }
            | Self::Io(_)
            | Self::Other(_) => Quality::Bad,
        }
    }
}
