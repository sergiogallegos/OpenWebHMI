//! Modbus TCP/RTU driver for OpenWebHMI.
//!
//! Address syntax is `<unit>/<area>/<addr>[:<count>][:<datatype>]`,
//! for example `1/holding/100:2:f32`.

pub mod address;
pub mod connection;
pub mod driver;

pub use address::{Area, DataType, ModbusAddress, ModbusAddressError};
pub use connection::{Connection, ModbusConfig, Parity, StopBits};
pub use driver::ModbusDriver;
