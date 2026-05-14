//! OPC UA client driver for OpenWebHMI.

#![deny(missing_docs)]

/// OPC UA NodeId address parsing.
pub mod address;
/// OPC UA connection configuration.
pub mod connection;
/// OPC UA driver implementation.
pub mod driver;

pub use address::{NodeIdForm, OpcUaAddress, OpcUaAddressError};
pub use connection::{AuthMode, OpcUaConfig};
pub use driver::OpcUaDriver;
