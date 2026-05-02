//! Beckhoff TwinCAT ADS client driver for OpenWebHMI.
//!
//! Address syntax is `<port>:<symbol>`, for example
//! `851:MAIN.fbMotor.fActualSpeed`.

pub mod address;
pub mod connection;
pub mod driver;
pub mod symbols;

pub use address::{AdsAddress, AdsAddressError};
pub use connection::{AdsConfig, SourceAms};
pub use driver::AdsDriver;
pub use symbols::{decode_ads_value, encode_ads_value, AdsDataType, SymbolEntry, SymbolTable};
