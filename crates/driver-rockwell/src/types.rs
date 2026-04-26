//! Type conversions between OpenWebHMI and `rust-ethernet-ip`.

use openwebhmi_driver_api::{DriverError, DriverResult};
use openwebhmi_protocol::TagValue;
use rust_ethernet_ip::PlcValue;

/// Convert an upstream PLC value into an OpenWebHMI tag value.
pub fn plc_to_tag_value(value: PlcValue) -> DriverResult<TagValue> {
    match value {
        PlcValue::Bool(value) => Ok(TagValue::Bool(value)),
        PlcValue::Sint(value) => Ok(TagValue::Int(value.into())),
        PlcValue::Int(value) => Ok(TagValue::Int(value.into())),
        PlcValue::Dint(value) => Ok(TagValue::Int(value.into())),
        PlcValue::Lint(value) => Ok(TagValue::Int(value)),
        PlcValue::Usint(value) => Ok(TagValue::Int(value.into())),
        PlcValue::Uint(value) => Ok(TagValue::Int(value.into())),
        PlcValue::Udint(value) => Ok(TagValue::Int(value.into())),
        PlcValue::Ulint(value) => {
            i64::try_from(value)
                .map(TagValue::Int)
                .map_err(|_| DriverError::UnsupportedType {
                    device_type: "ULINT value exceeds OpenWebHMI signed integer range".to_string(),
                })
        }
        PlcValue::Real(value) => Ok(TagValue::Real(value.into())),
        PlcValue::Lreal(value) => Ok(TagValue::Real(value)),
        PlcValue::String(value) => Ok(TagValue::String(value)),
        PlcValue::Udt(_) => Err(DriverError::UnsupportedType {
            device_type: "UDT".to_string(),
        }),
    }
}

/// Convert an OpenWebHMI tag value into an upstream PLC value.
pub fn tag_to_plc_value(value: TagValue) -> DriverResult<PlcValue> {
    match value {
        TagValue::Bool(value) => Ok(PlcValue::Bool(value)),
        TagValue::Int(value) => {
            i32::try_from(value)
                .map(PlcValue::Dint)
                .map_err(|_| DriverError::UnsupportedType {
                    device_type: "OpenWebHMI Int outside DINT range".to_string(),
                })
        }
        TagValue::Real(value) => Ok(PlcValue::Real(value as f32)),
        TagValue::String(value) => Ok(PlcValue::String(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plc_values_convert_to_tag_values() {
        assert_eq!(
            plc_to_tag_value(PlcValue::Real(1.5)).unwrap(),
            TagValue::Real(1.5)
        );
        assert_eq!(
            plc_to_tag_value(PlcValue::Dint(42)).unwrap(),
            TagValue::Int(42)
        );
        assert_eq!(
            plc_to_tag_value(PlcValue::String("ok".to_string())).unwrap(),
            TagValue::String("ok".to_string())
        );
    }

    #[test]
    fn tag_values_convert_to_plc_values() {
        assert_eq!(
            tag_to_plc_value(TagValue::Bool(true)).unwrap(),
            PlcValue::Bool(true)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(42)).unwrap(),
            PlcValue::Dint(42)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Real(1.5)).unwrap(),
            PlcValue::Real(1.5)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::String("ok".to_string())).unwrap(),
            PlcValue::String("ok".to_string())
        );
    }
}
