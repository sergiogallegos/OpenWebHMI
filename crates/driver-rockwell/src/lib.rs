//! Rockwell EtherNet/IP driver implementation for OpenWebHMI.
//!
//! The crate wraps `rust-ethernet-ip` behind the stable
//! `openwebhmi-driver-api` trait.

#![deny(missing_docs)]

mod config;
mod driver;
mod types;

#[doc(hidden)]
pub mod eip_client;
#[doc(hidden)]
pub mod test_support;

pub use config::RockwellConfig;
pub use driver::RockwellDriver;
pub use types::{plc_to_tag_value, tag_to_plc_value};
