//! ADS symbol metadata and primitive value mapping.

use std::collections::BTreeMap;

use openwebhmi_driver_api::{DriverError, DriverResult, TagAddress, TagNode};
use openwebhmi_protocol::TagValue;

/// ADS/TwinCAT primitive type supported by OpenWebHMI v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdsDataType {
    /// `BOOL`
    Bool,
    /// `SINT`
    Sint,
    /// `USINT` / `BYTE`
    Usint,
    /// `INT`
    Int,
    /// `UINT` / `WORD`
    Uint,
    /// `DINT`
    Dint,
    /// `UDINT` / `DWORD`
    Udint,
    /// `LINT`
    Lint,
    /// `ULINT` / `LWORD`
    Ulint,
    /// `REAL`
    Real,
    /// `LREAL`
    Lreal,
    /// `STRING(n)`
    String,
}

impl AdsDataType {
    /// Infer a supported primitive from ADS base type and TwinCAT type name.
    pub fn from_ads(base_type: u32, typ: &str) -> Option<Self> {
        let normalized = typ
            .split('(')
            .next()
            .unwrap_or(typ)
            .trim()
            .to_ascii_uppercase();
        match (base_type, normalized.as_str()) {
            (33, _) | (_, "BOOL") => Some(Self::Bool),
            (16, _) | (_, "SINT") => Some(Self::Sint),
            (17, _) | (_, "USINT" | "BYTE") => Some(Self::Usint),
            (2, _) | (_, "INT") => Some(Self::Int),
            (18, _) | (_, "UINT" | "WORD") => Some(Self::Uint),
            (3, _) | (_, "DINT") => Some(Self::Dint),
            (19, _) | (_, "UDINT" | "DWORD") => Some(Self::Udint),
            (20, _) | (_, "LINT") => Some(Self::Lint),
            (21, _) | (_, "ULINT" | "LWORD") => Some(Self::Ulint),
            (4, _) | (_, "REAL") => Some(Self::Real),
            (5, _) | (_, "LREAL") => Some(Self::Lreal),
            (30, _) => Some(Self::String),
            (_, typ) if typ.starts_with("STRING") => Some(Self::String),
            _ => None,
        }
    }
}

/// One browsable TwinCAT symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    /// Symbol name.
    pub name: String,
    /// ADS index group.
    pub index_group: u32,
    /// ADS index offset.
    pub index_offset: u32,
    /// TwinCAT type name.
    pub type_name: String,
    /// Value size in bytes.
    pub size: usize,
    /// ADS base type.
    pub base_type: u32,
    /// Decoded OpenWebHMI primitive type, if supported.
    pub data_type: Option<AdsDataType>,
}

/// Symbol table used for browse and type-aware decode.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    by_name: BTreeMap<String, SymbolEntry>,
}

impl SymbolTable {
    /// Build a table from symbol entries.
    pub fn new(entries: impl IntoIterator<Item = SymbolEntry>) -> Self {
        Self {
            by_name: entries
                .into_iter()
                .map(|entry| (entry.name.clone(), entry))
                .collect(),
        }
    }

    /// Return a symbol by name.
    pub fn get(&self, name: &str) -> Option<&SymbolEntry> {
        self.by_name.get(name)
    }

    /// Return all entries in sorted name order.
    pub fn entries(&self) -> impl Iterator<Item = &SymbolEntry> {
        self.by_name.values()
    }

    /// Build a flat browse tree for v1.
    pub fn browse(&self, prefix: Option<&str>) -> Vec<TagNode> {
        self.entries()
            .filter(|entry| prefix.map(|p| entry.name.starts_with(p)).unwrap_or(true))
            .map(|entry| TagNode {
                name: entry.name.clone(),
                address: Some(TagAddress::new(entry.name.clone())),
                data_type: Some(entry.type_name.clone()),
                children: Vec::new(),
            })
            .collect()
    }
}

/// Convert `ads::symbol::Symbol` values to an OpenWebHMI symbol table.
pub fn table_from_ads(symbols: Vec<ads::symbol::Symbol>) -> SymbolTable {
    SymbolTable::new(symbols.into_iter().map(|symbol| SymbolEntry {
        data_type: AdsDataType::from_ads(symbol.base_type, &symbol.typ),
        name: symbol.name,
        index_group: symbol.ix_group,
        index_offset: symbol.ix_offset,
        type_name: symbol.typ,
        size: symbol.size,
        base_type: symbol.base_type,
    }))
}

/// Decode raw ADS bytes into an OpenWebHMI tag value.
pub fn decode_ads_value(data_type: AdsDataType, bytes: &[u8]) -> DriverResult<TagValue> {
    match data_type {
        AdsDataType::Bool => Ok(TagValue::Bool(bytes.first().copied().unwrap_or(0) != 0)),
        AdsDataType::Sint => {
            require(bytes, 1).map(|b| TagValue::Int(i8::from_le_bytes([b[0]]) as i64))
        }
        AdsDataType::Usint => require(bytes, 1).map(|b| TagValue::Int(b[0] as i64)),
        AdsDataType::Int => {
            require(bytes, 2).map(|b| TagValue::Int(i16::from_le_bytes([b[0], b[1]]) as i64))
        }
        AdsDataType::Uint => {
            require(bytes, 2).map(|b| TagValue::Int(u16::from_le_bytes([b[0], b[1]]) as i64))
        }
        AdsDataType::Dint => require(bytes, 4)
            .map(|b| TagValue::Int(i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as i64)),
        AdsDataType::Udint => require(bytes, 4)
            .map(|b| TagValue::Int(u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as i64)),
        AdsDataType::Lint => require(bytes, 8)
            .map(|b| TagValue::Int(i64::from_le_bytes(b[..8].try_into().expect("len")))),
        AdsDataType::Ulint => require(bytes, 8)
            .map(|b| TagValue::Int(u64::from_le_bytes(b[..8].try_into().expect("len")) as i64)),
        AdsDataType::Real => require(bytes, 4)
            .map(|b| TagValue::Real(f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64)),
        AdsDataType::Lreal => require(bytes, 8)
            .map(|b| TagValue::Real(f64::from_le_bytes(b[..8].try_into().expect("len")))),
        AdsDataType::String => {
            let end = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(bytes.len());
            String::from_utf8(bytes[..end].to_vec())
                .map(TagValue::String)
                .map_err(|err| DriverError::RemoteFault {
                    code: "ads-string-utf8".to_string(),
                    message: err.to_string(),
                })
        }
    }
}

/// Encode an OpenWebHMI tag value into TwinCAT little-endian bytes.
pub fn encode_ads_value(
    data_type: AdsDataType,
    value: TagValue,
    size: usize,
) -> DriverResult<Vec<u8>> {
    let bytes = match (data_type, value) {
        (AdsDataType::Bool, TagValue::Bool(value)) => vec![u8::from(value)],
        (AdsDataType::Sint, TagValue::Int(value)) => vec![(value as i8) as u8],
        (AdsDataType::Usint, TagValue::Int(value)) => vec![value as u8],
        (AdsDataType::Int, TagValue::Int(value)) => (value as i16).to_le_bytes().to_vec(),
        (AdsDataType::Uint, TagValue::Int(value)) => (value as u16).to_le_bytes().to_vec(),
        (AdsDataType::Dint, TagValue::Int(value)) => (value as i32).to_le_bytes().to_vec(),
        (AdsDataType::Udint, TagValue::Int(value)) => (value as u32).to_le_bytes().to_vec(),
        (AdsDataType::Lint, TagValue::Int(value)) => value.to_le_bytes().to_vec(),
        (AdsDataType::Ulint, TagValue::Int(value)) => (value as u64).to_le_bytes().to_vec(),
        (AdsDataType::Real, TagValue::Real(value)) => (value as f32).to_le_bytes().to_vec(),
        (AdsDataType::Lreal, TagValue::Real(value)) => value.to_le_bytes().to_vec(),
        (AdsDataType::String, TagValue::String(value)) => {
            if value.len() + 1 > size {
                return Err(DriverError::UnsupportedType {
                    device_type: format!(
                        "STRING payload length {} exceeds ADS size {size}",
                        value.len()
                    ),
                });
            }
            let mut bytes = vec![0; size];
            bytes[..value.len()].copy_from_slice(value.as_bytes());
            bytes
        }
        (data_type, value) => {
            return Err(DriverError::UnsupportedType {
                device_type: format!("cannot write {value:?} to {data_type:?}"),
            });
        }
    };
    Ok(bytes)
}

fn require(bytes: &[u8], len: usize) -> DriverResult<&[u8]> {
    if bytes.len() < len {
        return Err(DriverError::RemoteFault {
            code: "ads-short-read".to_string(),
            message: format!("expected {len} bytes, got {}", bytes.len()),
        });
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_primitives() {
        assert_eq!(
            decode_ads_value(AdsDataType::Bool, &[1]).unwrap(),
            TagValue::Bool(true)
        );
        assert_eq!(
            decode_ads_value(AdsDataType::Dint, &42_i32.to_le_bytes()).unwrap(),
            TagValue::Int(42)
        );
        assert_eq!(
            decode_ads_value(AdsDataType::Real, &12.5_f32.to_le_bytes()).unwrap(),
            TagValue::Real(12.5)
        );
        assert_eq!(
            decode_ads_value(AdsDataType::String, b"hello\0\0").unwrap(),
            TagValue::String("hello".to_string())
        );
    }

    #[test]
    fn encodes_primitives() {
        assert_eq!(
            encode_ads_value(AdsDataType::Bool, TagValue::Bool(true), 1).unwrap(),
            vec![1]
        );
        assert_eq!(
            encode_ads_value(AdsDataType::Dint, TagValue::Int(42), 4).unwrap(),
            42_i32.to_le_bytes()
        );
    }

    #[test]
    fn builds_browse_nodes() {
        let table = SymbolTable::new([SymbolEntry {
            name: "MAIN.Speed".to_string(),
            index_group: 1,
            index_offset: 2,
            type_name: "REAL".to_string(),
            size: 4,
            base_type: 4,
            data_type: Some(AdsDataType::Real),
        }]);
        let nodes = table.browse(Some("MAIN"));
        assert_eq!(nodes.len(), 1);
        assert_eq!(
            nodes[0]
                .address
                .as_ref()
                .map(|address| address.raw.as_str()),
            Some("MAIN.Speed")
        );
    }
}
