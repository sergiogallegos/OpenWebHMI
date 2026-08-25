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

/// Convert an OpenWebHMI tag value into the target PLC tag's scalar type.
pub fn tag_to_plc_value(value: TagValue, target: Option<&PlcValue>) -> DriverResult<PlcValue> {
    // Preserve the legacy DINT/REAL defaults for callers without live metadata;
    // the driver supplies a pre-write read so normal writes never take this lossy path.
    match (value, target) {
        (TagValue::Bool(value), Some(PlcValue::Bool(_)) | None) => Ok(PlcValue::Bool(value)),
        (TagValue::Int(value), Some(PlcValue::Sint(_))) => {
            checked_integer(value, "SINT", PlcValue::Sint)
        }
        (TagValue::Int(value), Some(PlcValue::Int(_))) => {
            checked_integer(value, "INT", PlcValue::Int)
        }
        (TagValue::Int(value), Some(PlcValue::Dint(_)) | None) => {
            checked_integer(value, "DINT", PlcValue::Dint)
        }
        (TagValue::Int(value), Some(PlcValue::Lint(_))) => Ok(PlcValue::Lint(value)),
        (TagValue::Int(value), Some(PlcValue::Usint(_))) => {
            checked_integer(value, "USINT", PlcValue::Usint)
        }
        (TagValue::Int(value), Some(PlcValue::Uint(_))) => {
            checked_integer(value, "UINT", PlcValue::Uint)
        }
        (TagValue::Int(value), Some(PlcValue::Udint(_))) => {
            checked_integer(value, "UDINT", PlcValue::Udint)
        }
        (TagValue::Int(value), Some(PlcValue::Ulint(_))) => {
            checked_integer(value, "ULINT", PlcValue::Ulint)
        }
        (TagValue::Real(value), Some(PlcValue::Real(_)) | None) => Ok(PlcValue::Real(value as f32)),
        (TagValue::Real(value), Some(PlcValue::Lreal(_))) => Ok(PlcValue::Lreal(value)),
        (TagValue::String(value), Some(PlcValue::String(_)) | None) => Ok(PlcValue::String(value)),
        (value, Some(target)) => Err(DriverError::UnsupportedType {
            device_type: format!(
                "cannot write OpenWebHMI {} value to PLC {} tag",
                tag_value_type(&value),
                plc_value_type(target)
            ),
        }),
    }
}

fn checked_integer<T>(
    value: i64,
    target_type: &str,
    constructor: impl FnOnce(T) -> PlcValue,
) -> DriverResult<PlcValue>
where
    T: TryFrom<i64>,
{
    T::try_from(value)
        .map(constructor)
        .map_err(|_| DriverError::UnsupportedType {
            device_type: format!("OpenWebHMI Int value {value} is outside PLC {target_type} range"),
        })
}

fn tag_value_type(value: &TagValue) -> &'static str {
    match value {
        TagValue::Bool(_) => "Bool",
        TagValue::Int(_) => "Int",
        TagValue::Real(_) => "Real",
        TagValue::String(_) => "String",
    }
}

fn plc_value_type(value: &PlcValue) -> &'static str {
    match value {
        PlcValue::Bool(_) => "BOOL",
        PlcValue::Sint(_) => "SINT",
        PlcValue::Int(_) => "INT",
        PlcValue::Dint(_) => "DINT",
        PlcValue::Lint(_) => "LINT",
        PlcValue::Usint(_) => "USINT",
        PlcValue::Uint(_) => "UINT",
        PlcValue::Udint(_) => "UDINT",
        PlcValue::Ulint(_) => "ULINT",
        PlcValue::Real(_) => "REAL",
        PlcValue::Lreal(_) => "LREAL",
        PlcValue::String(_) => "STRING",
        PlcValue::Udt(_) => "UDT",
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
            tag_to_plc_value(TagValue::Bool(true), None).unwrap(),
            PlcValue::Bool(true)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(42), None).unwrap(),
            PlcValue::Dint(42)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Real(1.5), None).unwrap(),
            PlcValue::Real(1.5)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::String("ok".to_string()), None).unwrap(),
            PlcValue::String("ok".to_string())
        );
    }

    #[test]
    fn integer_values_follow_the_target_plc_type() {
        assert_eq!(
            tag_to_plc_value(TagValue::Int(100), Some(&PlcValue::Sint(0))).unwrap(),
            PlcValue::Sint(100)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(30_000), Some(&PlcValue::Int(0))).unwrap(),
            PlcValue::Int(30_000)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(70_000), Some(&PlcValue::Dint(0))).unwrap(),
            PlcValue::Dint(70_000)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(i64::MAX), Some(&PlcValue::Lint(0))).unwrap(),
            PlcValue::Lint(i64::MAX)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(200), Some(&PlcValue::Usint(0))).unwrap(),
            PlcValue::Usint(200)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Int(70_000), Some(&PlcValue::Udint(0))).unwrap(),
            PlcValue::Udint(70_000)
        );
    }

    #[test]
    fn integer_values_outside_the_target_range_are_rejected() {
        for (value, target, target_name) in [
            (200, PlcValue::Sint(0), "SINT"),
            (70_000, PlcValue::Int(0), "INT"),
            (i64::MAX, PlcValue::Dint(0), "DINT"),
            (-1, PlcValue::Usint(0), "USINT"),
        ] {
            let error = tag_to_plc_value(TagValue::Int(value), Some(&target)).unwrap_err();
            assert!(error.to_string().contains(target_name));
            assert!(error.to_string().contains(&value.to_string()));
        }
    }

    #[test]
    fn real_values_follow_the_target_precision() {
        let value = 1.234_567_890_123_45;
        assert_eq!(
            tag_to_plc_value(TagValue::Real(value), Some(&PlcValue::Lreal(0.0))).unwrap(),
            PlcValue::Lreal(value)
        );
        assert_eq!(
            tag_to_plc_value(TagValue::Real(value), Some(&PlcValue::Real(0.0))).unwrap(),
            PlcValue::Real(value as f32)
        );
    }
}
