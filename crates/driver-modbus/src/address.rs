//! Modbus address parsing and value encoding.

use std::fmt;
use std::str::FromStr;

use openwebhmi_driver_api::DriverError;
use openwebhmi_protocol::TagValue;
use thiserror::Error;

/// Modbus memory area and function-code family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Area {
    /// Coils, read by FC1 and written by FC5/FC15.
    Coils,
    /// Discrete inputs, read by FC2.
    Discrete,
    /// Input registers, read by FC4.
    Input,
    /// Holding registers, read by FC3 and written by FC6/FC16.
    Holding,
}

impl Area {
    /// Return true when the area stores bits rather than registers.
    pub fn is_bit_area(self) -> bool {
        matches!(self, Self::Coils | Self::Discrete)
    }

    /// Return true when the area is writable by this driver.
    pub fn is_writable(self) -> bool {
        matches!(self, Self::Coils | Self::Holding)
    }
}

impl FromStr for Area {
    type Err = ModbusAddressError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "coils" => Ok(Self::Coils),
            "discrete" => Ok(Self::Discrete),
            "input" => Ok(Self::Input),
            "holding" => Ok(Self::Holding),
            other => Err(ModbusAddressError::UnknownArea(other.to_string())),
        }
    }
}

impl fmt::Display for Area {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Coils => "coils",
            Self::Discrete => "discrete",
            Self::Input => "input",
            Self::Holding => "holding",
        })
    }
}

/// Data interpretation for one Modbus address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Boolean bit.
    Bool,
    /// Unsigned 16-bit register.
    U16,
    /// Signed 16-bit register.
    I16,
    /// Unsigned 32-bit integer, high word first.
    U32,
    /// Signed 32-bit integer, high word first.
    I32,
    /// Unsigned 32-bit integer, low word first.
    U32Le,
    /// Signed 32-bit integer, low word first.
    I32Le,
    /// IEEE-754 32-bit float, high word first.
    F32,
    /// IEEE-754 32-bit float, low word first.
    F32Le,
    /// IEEE-754 64-bit float, high word first.
    F64,
    /// UTF-8 string stored as two bytes per register.
    String,
}

impl DataType {
    /// Minimum register count required by this type.
    pub fn min_registers(self) -> u16 {
        match self {
            Self::Bool | Self::U16 | Self::I16 => 1,
            Self::U32 | Self::I32 | Self::U32Le | Self::I32Le | Self::F32 | Self::F32Le => 2,
            Self::F64 => 4,
            Self::String => 1,
        }
    }

    /// True when this type is valid for bit areas.
    pub fn valid_for_bits(self) -> bool {
        matches!(self, Self::Bool)
    }
}

impl FromStr for DataType {
    type Err = ModbusAddressError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "bool" => Ok(Self::Bool),
            "u16" => Ok(Self::U16),
            "i16" => Ok(Self::I16),
            "u32" => Ok(Self::U32),
            "i32" => Ok(Self::I32),
            "u32_le" => Ok(Self::U32Le),
            "i32_le" => Ok(Self::I32Le),
            "f32" => Ok(Self::F32),
            "f32_le" => Ok(Self::F32Le),
            "f64" => Ok(Self::F64),
            "string" => Ok(Self::String),
            other => Err(ModbusAddressError::UnknownDataType(other.to_string())),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Bool => "bool",
            Self::U16 => "u16",
            Self::I16 => "i16",
            Self::U32 => "u32",
            Self::I32 => "i32",
            Self::U32Le => "u32_le",
            Self::I32Le => "i32_le",
            Self::F32 => "f32",
            Self::F32Le => "f32_le",
            Self::F64 => "f64",
            Self::String => "string",
        })
    }
}

/// Parsed Modbus tag address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModbusAddress {
    /// Modbus unit/slave id.
    pub unit_id: u8,
    /// Memory area.
    pub area: Area,
    /// Zero-based coil or register address.
    pub address: u16,
    /// Number of coils/registers to read.
    pub count: u16,
    /// Value interpretation.
    pub data_type: DataType,
}

impl ModbusAddress {
    /// Parse the OpenWebHMI Modbus address syntax.
    pub fn parse(raw: &str) -> Result<Self, ModbusAddressError> {
        raw.parse()
    }

    /// Decode bits returned by FC1/FC2.
    pub fn decode_bits(&self, bits: &[bool]) -> Result<TagValue, DriverError> {
        if self.data_type != DataType::Bool {
            return Err(DriverError::UnsupportedType {
                device_type: self.data_type.to_string(),
            });
        }
        bits.first()
            .copied()
            .map(TagValue::Bool)
            .ok_or_else(|| DriverError::RemoteFault {
                code: "short-read".to_string(),
                message: "Modbus bit response was empty".to_string(),
            })
    }

    /// Decode registers returned by FC3/FC4.
    pub fn decode_registers(&self, registers: &[u16]) -> Result<TagValue, DriverError> {
        if registers.len() < self.data_type.min_registers() as usize {
            return Err(DriverError::RemoteFault {
                code: "short-read".to_string(),
                message: format!(
                    "Modbus register response returned {} words, expected at least {}",
                    registers.len(),
                    self.data_type.min_registers()
                ),
            });
        }

        Ok(match self.data_type {
            DataType::Bool => TagValue::Bool(registers[0] != 0),
            DataType::U16 => TagValue::Int(registers[0] as i64),
            DataType::I16 => TagValue::Int(i16::from_be_bytes(registers[0].to_be_bytes()) as i64),
            DataType::U32 => TagValue::Int(words_to_u32([registers[0], registers[1]]) as i64),
            DataType::I32 => TagValue::Int(words_to_i32([registers[0], registers[1]]) as i64),
            DataType::U32Le => TagValue::Int(words_to_u32([registers[1], registers[0]]) as i64),
            DataType::I32Le => TagValue::Int(words_to_i32([registers[1], registers[0]]) as i64),
            DataType::F32 => {
                TagValue::Real(f32::from_bits(words_to_u32([registers[0], registers[1]])) as f64)
            }
            DataType::F32Le => {
                TagValue::Real(f32::from_bits(words_to_u32([registers[1], registers[0]])) as f64)
            }
            DataType::F64 => {
                let bytes = registers[..4]
                    .iter()
                    .flat_map(|word| word.to_be_bytes())
                    .collect::<Vec<_>>();
                TagValue::Real(f64::from_bits(u64::from_be_bytes(
                    bytes.try_into().expect("four registers make eight bytes"),
                )))
            }
            DataType::String => {
                let bytes = registers
                    .iter()
                    .flat_map(|word| word.to_be_bytes())
                    .take_while(|byte| *byte != 0)
                    .collect::<Vec<_>>();
                TagValue::String(String::from_utf8(bytes).map_err(|err| {
                    DriverError::UnsupportedType {
                        device_type: format!("invalid UTF-8 string register payload: {err}"),
                    }
                })?)
            }
        })
    }

    /// Encode a value for FC5/FC15 or FC6/FC16.
    pub fn encode_value(&self, value: TagValue) -> Result<EncodedValue, DriverError> {
        if !self.area.is_writable() {
            return Err(DriverError::InvalidAddress(format!(
                "{} is read-only",
                self.area
            )));
        }

        match self.area {
            Area::Coils => match value {
                TagValue::Bool(value) => Ok(EncodedValue::Bits(vec![value])),
                other => Err(DriverError::UnsupportedType {
                    device_type: format!("coils require bool, got {other:?}"),
                }),
            },
            Area::Holding => self.encode_registers(value).map(EncodedValue::Registers),
            Area::Discrete | Area::Input => unreachable!("read-only areas returned above"),
        }
    }

    fn encode_registers(&self, value: TagValue) -> Result<Vec<u16>, DriverError> {
        match (self.data_type, value) {
            (DataType::Bool, TagValue::Bool(value)) => Ok(vec![u16::from(value)]),
            (DataType::U16, TagValue::Int(value)) if (0..=u16::MAX as i64).contains(&value) => {
                Ok(vec![value as u16])
            }
            (DataType::I16, TagValue::Int(value))
                if (i16::MIN as i64..=i16::MAX as i64).contains(&value) =>
            {
                Ok(vec![value as i16 as u16])
            }
            (DataType::U32, TagValue::Int(value)) if (0..=u32::MAX as i64).contains(&value) => {
                Ok(u32_to_words(value as u32).to_vec())
            }
            (DataType::I32, TagValue::Int(value))
                if (i32::MIN as i64..=i32::MAX as i64).contains(&value) =>
            {
                Ok(u32_to_words(value as i32 as u32).to_vec())
            }
            (DataType::U32Le, TagValue::Int(value)) if (0..=u32::MAX as i64).contains(&value) => {
                let [hi, lo] = u32_to_words(value as u32);
                Ok(vec![lo, hi])
            }
            (DataType::I32Le, TagValue::Int(value))
                if (i32::MIN as i64..=i32::MAX as i64).contains(&value) =>
            {
                let [hi, lo] = u32_to_words(value as i32 as u32);
                Ok(vec![lo, hi])
            }
            (DataType::F32, TagValue::Real(value)) => {
                Ok(u32_to_words((value as f32).to_bits()).to_vec())
            }
            (DataType::F32Le, TagValue::Real(value)) => {
                let [hi, lo] = u32_to_words((value as f32).to_bits());
                Ok(vec![lo, hi])
            }
            (DataType::F64, TagValue::Real(value)) => Ok(value
                .to_bits()
                .to_be_bytes()
                .chunks_exact(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                .collect()),
            (DataType::String, TagValue::String(value)) => {
                let mut bytes = value.into_bytes();
                if bytes.len() % 2 != 0 {
                    bytes.push(0);
                }
                Ok(bytes
                    .chunks_exact(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                    .collect())
            }
            (data_type, value) => Err(DriverError::UnsupportedType {
                device_type: format!("{data_type} cannot encode {value:?}"),
            }),
        }
    }
}

impl FromStr for ModbusAddress {
    type Err = ModbusAddressError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let segments = raw.split('/').collect::<Vec<_>>();
        if segments.len() != 3 {
            return Err(ModbusAddressError::InvalidShape(raw.to_string()));
        }

        let unit_id = segments[0]
            .parse::<u8>()
            .map_err(|_| ModbusAddressError::InvalidUnit(segments[0].to_string()))?;
        if !(1..=247).contains(&unit_id) && unit_id != 255 {
            return Err(ModbusAddressError::InvalidUnit(segments[0].to_string()));
        }

        let area = segments[1].parse::<Area>()?;
        let address_parts = segments[2].split(':').collect::<Vec<_>>();
        if address_parts.is_empty() || address_parts.len() > 3 {
            return Err(ModbusAddressError::InvalidShape(raw.to_string()));
        }

        let address = address_parts[0]
            .parse::<u16>()
            .map_err(|_| ModbusAddressError::InvalidAddress(address_parts[0].to_string()))?;
        let mut explicit_count = None;
        let mut data_type = None;

        match address_parts.as_slice() {
            [_address] => {}
            [_address, maybe_count_or_type] => {
                if let Ok(count) = maybe_count_or_type.parse::<u16>() {
                    explicit_count = Some(count);
                } else {
                    data_type = Some(maybe_count_or_type.parse::<DataType>()?);
                }
            }
            [_address, count, type_name] => {
                explicit_count = Some(
                    count
                        .parse::<u16>()
                        .map_err(|_| ModbusAddressError::InvalidCount((*count).to_string()))?,
                );
                data_type = Some(type_name.parse::<DataType>()?);
            }
            _ => unreachable!("address part length checked above"),
        }

        let data_type = data_type.unwrap_or_else(|| {
            if area.is_bit_area() {
                DataType::Bool
            } else {
                DataType::U16
            }
        });
        if area.is_bit_area() && !data_type.valid_for_bits() {
            return Err(ModbusAddressError::InvalidTypeForArea { area, data_type });
        }

        let count = explicit_count.unwrap_or_else(|| {
            if area.is_bit_area() {
                1
            } else {
                data_type.min_registers()
            }
        });
        if count == 0 {
            return Err(ModbusAddressError::InvalidCount("0".to_string()));
        }
        if !area.is_bit_area() && count < data_type.min_registers() {
            return Err(ModbusAddressError::CountTooSmall {
                data_type,
                min: data_type.min_registers(),
                actual: count,
            });
        }
        let max = if area.is_bit_area() { 2000 } else { 125 };
        if count > max {
            return Err(ModbusAddressError::CountTooLarge {
                area,
                max,
                actual: count,
            });
        }

        Ok(Self {
            unit_id,
            area,
            address,
            count,
            data_type,
        })
    }
}

/// Encoded payload for Modbus write operations.
#[derive(Debug, Clone, PartialEq)]
pub enum EncodedValue {
    /// Coil bits.
    Bits(Vec<bool>),
    /// Holding registers.
    Registers(Vec<u16>),
}

/// Address parse failure.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModbusAddressError {
    /// Address did not contain `<unit>/<area>/<addr...>`.
    #[error("invalid Modbus address shape: {0}")]
    InvalidShape(String),
    /// Unit id is not 1..247 or 255.
    #[error("invalid Modbus unit id: {0}")]
    InvalidUnit(String),
    /// Unknown memory area.
    #[error("unknown Modbus area: {0}")]
    UnknownArea(String),
    /// Coil/register address was not a u16.
    #[error("invalid Modbus address offset: {0}")]
    InvalidAddress(String),
    /// Count was not a positive u16.
    #[error("invalid Modbus count: {0}")]
    InvalidCount(String),
    /// Unknown data type.
    #[error("unknown Modbus datatype: {0}")]
    UnknownDataType(String),
    /// Datatype cannot be applied to this area.
    #[error("datatype {data_type} is invalid for {area}")]
    InvalidTypeForArea {
        /// Memory area.
        area: Area,
        /// Data type.
        data_type: DataType,
    },
    /// Count is too small for the requested type.
    #[error("datatype {data_type} needs at least {min} registers, got {actual}")]
    CountTooSmall {
        /// Data type.
        data_type: DataType,
        /// Minimum registers.
        min: u16,
        /// Actual count.
        actual: u16,
    },
    /// Count exceeds the Modbus function-code limit for the area.
    #[error("{area} count exceeds Modbus limit {max}: {actual}")]
    CountTooLarge {
        /// Memory area.
        area: Area,
        /// Maximum coils/registers.
        max: u16,
        /// Actual count.
        actual: u16,
    },
}

fn words_to_u32(words: [u16; 2]) -> u32 {
    u32::from_be_bytes([
        (words[0] >> 8) as u8,
        words[0] as u8,
        (words[1] >> 8) as u8,
        words[1] as u8,
    ])
}

fn words_to_i32(words: [u16; 2]) -> i32 {
    words_to_u32(words) as i32
}

fn u32_to_words(value: u32) -> [u16; 2] {
    let bytes = value.to_be_bytes();
    [
        u16::from_be_bytes([bytes[0], bytes[1]]),
        u16::from_be_bytes([bytes[2], bytes[3]]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_area_defaults() {
        assert_eq!(
            ModbusAddress::parse("1/coils/0").unwrap(),
            ModbusAddress {
                unit_id: 1,
                area: Area::Coils,
                address: 0,
                count: 1,
                data_type: DataType::Bool,
            }
        );
        assert_eq!(
            ModbusAddress::parse("255/holding/100").unwrap(),
            ModbusAddress {
                unit_id: 255,
                area: Area::Holding,
                address: 100,
                count: 1,
                data_type: DataType::U16,
            }
        );
    }

    #[test]
    fn parses_all_areas_and_types() {
        for area in ["coils", "discrete"] {
            let parsed = ModbusAddress::parse(&format!("1/{area}/0:bool")).unwrap();
            assert_eq!(parsed.area.to_string(), area);
            assert_eq!(parsed.data_type, DataType::Bool);
        }

        for data_type in [
            "bool", "u16", "i16", "u32", "i32", "u32_le", "i32_le", "f32", "f32_le", "f64",
            "string",
        ] {
            let area = if data_type == "bool" {
                "holding"
            } else {
                "input"
            };
            let parsed = ModbusAddress::parse(&format!("1/{area}/10:{data_type}")).unwrap();
            assert_eq!(parsed.data_type.to_string(), data_type);
        }
    }

    #[test]
    fn parses_explicit_count_and_type() {
        let parsed = ModbusAddress::parse("1/holding/100:2:f32").unwrap();
        assert_eq!(parsed.count, 2);
        assert_eq!(parsed.data_type, DataType::F32);

        let string = ModbusAddress::parse("2/holding/200:5:string").unwrap();
        assert_eq!(string.count, 5);
        assert_eq!(string.data_type, DataType::String);
    }

    #[test]
    fn rejects_malformed_addresses() {
        assert!(ModbusAddress::parse("holding/0").is_err());
        assert!(ModbusAddress::parse("0/holding/0").is_err());
        assert!(ModbusAddress::parse("1/bad/0").is_err());
        assert!(ModbusAddress::parse("1/coils/0:u16").is_err());
        assert!(ModbusAddress::parse("1/holding/0:1:f32").is_err());
        assert!(ModbusAddress::parse("1/holding/0:126").is_err());
        assert!(ModbusAddress::parse("1/coils/0:2001").is_err());
    }

    #[test]
    fn decodes_endianness_and_strings() {
        let be = ModbusAddress::parse("1/holding/0:2:f32").unwrap();
        let le = ModbusAddress::parse("1/holding/0:2:f32_le").unwrap();
        assert_eq!(
            be.decode_registers(&[0x4148, 0x0000]).unwrap(),
            TagValue::Real(12.5)
        );
        assert_eq!(
            le.decode_registers(&[0x0000, 0x4148]).unwrap(),
            TagValue::Real(12.5)
        );

        let text = ModbusAddress::parse("1/holding/0:3:string").unwrap();
        assert_eq!(
            text.decode_registers(&[0x4f4b, 0x2100, 0]).unwrap(),
            TagValue::String("OK!".to_string())
        );
    }

    #[test]
    fn encodes_write_values() {
        let coil = ModbusAddress::parse("1/coils/0").unwrap();
        assert_eq!(
            coil.encode_value(TagValue::Bool(true)).unwrap(),
            EncodedValue::Bits(vec![true])
        );

        let f32_le = ModbusAddress::parse("1/holding/10:f32_le").unwrap();
        assert_eq!(
            f32_le.encode_value(TagValue::Real(12.5)).unwrap(),
            EncodedValue::Registers(vec![0x0000, 0x4148])
        );
    }
}
