//! Beckhoff TwinCAT ADS client driver for OpenWebHMI.
//!
//! Address syntax is `<port>:<symbol>`, for example
//! `851:MAIN.fbMotor.fActualSpeed`.

pub mod address;
pub mod connection;
pub mod driver;
pub mod symbols;
#[cfg(windows)]
mod twincat_router;

pub use address::{AdsAddress, AdsAddressError};
pub use connection::{AdsBackend, AdsConfig, SourceAms};
pub use driver::AdsDriver;
pub use symbols::{AdsDataType, SymbolEntry, SymbolTable, decode_ads_value, encode_ads_value};
