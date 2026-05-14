//! Beckhoff TwinCAT ADS client driver for OpenWebHMI.
//!
//! Address syntax is `<port>:<symbol>`, for example
//! `851:MAIN.fbMotor.fActualSpeed`.

#![deny(missing_docs)]

/// ADS address parsing.
pub mod address;
/// ADS connection configuration.
pub mod connection;
/// ADS driver implementation.
pub mod driver;
/// ADS symbol table and value codec helpers.
pub mod symbols;
#[cfg(windows)]
mod twincat_router;

pub use address::{AdsAddress, AdsAddressError};
pub use connection::{AdsBackend, AdsConfig, SourceAms};
pub use driver::AdsDriver;
pub use symbols::{AdsDataType, SymbolEntry, SymbolTable, decode_ads_value, encode_ads_value};
