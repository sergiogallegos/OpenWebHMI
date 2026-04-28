//! Alarm condition evaluation.

use openwebhmi_protocol::TagValue;

use crate::types::AlarmCondition;

/// Evaluate a condition against one tag value.
pub fn evaluate(condition: &AlarmCondition, value: &TagValue) -> Result<bool, ConditionError> {
    match condition {
        AlarmCondition::HighLimit { threshold } => Ok(numeric(value)? > *threshold),
        AlarmCondition::LowLimit { threshold } => Ok(numeric(value)? < *threshold),
        AlarmCondition::Equals { value: expected } => Ok(value == expected),
        AlarmCondition::Deviation {
            setpoint,
            tolerance,
        } => Ok((numeric(value)? - *setpoint).abs() > *tolerance),
        AlarmCondition::Digital { active_when } => match value {
            TagValue::Bool(value) => Ok(value == active_when),
            _ => Err(ConditionError::ExpectedBool),
        },
    }
}

/// Condition evaluation error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConditionError {
    /// Numeric condition received a non-numeric tag value.
    #[error("condition requires a numeric tag value")]
    ExpectedNumeric,
    /// Digital condition received a non-boolean tag value.
    #[error("digital condition requires a boolean tag value")]
    ExpectedBool,
}

fn numeric(value: &TagValue) -> Result<f64, ConditionError> {
    match value {
        TagValue::Int(value) => Ok(*value as f64),
        TagValue::Real(value) => Ok(*value),
        TagValue::Bool(_) | TagValue::String(_) => Err(ConditionError::ExpectedNumeric),
    }
}
