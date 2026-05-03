//! Public driver API for OpenWebHMI device and PLC drivers.
//!
//! Drivers implement [`Driver`]. The gateway owns them through
//! [`DriverSupervisor`], which provides best-effort recovery from Rust panics
//! under `panic = "unwind"`.

#![deny(missing_docs)]

/// Driver address and browse-tree types.
pub mod address;
/// Driver error taxonomy.
pub mod error;
/// Driver metadata and capability flags.
pub mod metadata;
/// Driver supervision primitives.
pub mod supervisor;
/// Driver trait and update envelope.
pub mod trait_def;

#[cfg(feature = "mock")]
/// Test helper driver implementation for downstream crates.
pub mod mock;

pub use address::{TagAddress, TagNode};
pub use error::{DriverError, DriverResult};
pub use metadata::{Capabilities, DriverMetadata};
#[cfg(feature = "mock")]
pub use mock::{MockDriver, MockFailureMode};
pub use supervisor::{DriverStatus, DriverSupervisor, SupervisorHandle};
pub use trait_def::{Driver, DriverUpdate, make_metadata};

#[cfg(test)]
mod tests {
    use openwebhmi_protocol::Quality;

    use super::*;

    #[test]
    fn serde_round_trips_address_metadata_and_capabilities() {
        let address = TagAddress::new("Line1.MotorRPM");
        let address_json = serde_json::to_string(&address).unwrap();
        assert_eq!(
            serde_json::from_str::<TagAddress>(&address_json).unwrap(),
            address
        );

        let metadata = DriverMetadata {
            vendor: "OpenWebHMI".to_string(),
            family: "test".to_string(),
            crate_version: "0.0.1".to_string(),
            capabilities: Capabilities {
                native_subscribe: true,
                browse: true,
                batch_read: false,
                batch_write: false,
            },
        };
        let metadata_json = serde_json::to_string(&metadata).unwrap();
        assert_eq!(
            serde_json::from_str::<DriverMetadata>(&metadata_json).unwrap(),
            metadata
        );
    }

    #[test]
    fn error_quality_mapping_covers_all_variants() {
        assert_eq!(DriverError::NotConnected.into_quality(), Quality::Bad);
        assert_eq!(DriverError::Connecting.into_quality(), Quality::Uncertain);
        assert_eq!(
            DriverError::InvalidAddress("bad".to_string()).into_quality(),
            Quality::Bad
        );
        assert_eq!(
            DriverError::RemoteFault {
                code: "1".to_string(),
                message: "fault".to_string(),
            }
            .into_quality(),
            Quality::Bad
        );
        assert_eq!(
            DriverError::UnsupportedType {
                device_type: "X".to_string(),
            }
            .into_quality(),
            Quality::Bad
        );
        assert_eq!(
            DriverError::Io(std::io::Error::other("io")).into_quality(),
            Quality::Bad
        );
        assert_eq!(
            DriverError::Other(anyhow::anyhow!("other")).into_quality(),
            Quality::Bad
        );
    }
}
