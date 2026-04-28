//! Public alarm types.

use openwebhmi_protocol::{Quality, TagValue};
use serde::{Deserialize, Serialize};

/// Static alarm definition loaded from a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlarmDefinition {
    /// Stable alarm id within a project.
    pub id: String,
    /// Human-readable alarm label.
    pub label: String,
    /// Alarm priority, where `1` is highest and `5` is lowest.
    pub priority: u8,
    /// Tag path evaluated by this alarm.
    pub tag_path: String,
    /// Condition that activates this alarm.
    pub condition: AlarmCondition,
    /// Message template. `{value}` is replaced with the current tag value.
    pub message: String,
    /// Disabled alarms are ignored.
    pub enabled: bool,
    /// Whether an operator ack is required before the alarm can clear.
    pub require_ack: bool,
}

/// Alarm condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AlarmCondition {
    /// Activates when numeric value is greater than `threshold`.
    HighLimit {
        /// High threshold.
        threshold: f64,
    },
    /// Activates when numeric value is less than `threshold`.
    LowLimit {
        /// Low threshold.
        threshold: f64,
    },
    /// Activates when value equals the configured value.
    Equals {
        /// Expected value.
        value: TagValue,
    },
    /// Activates when numeric value differs from setpoint by more than tolerance.
    Deviation {
        /// Expected setpoint.
        setpoint: f64,
        /// Maximum allowed absolute deviation.
        tolerance: f64,
    },
    /// Activates when a boolean tag equals `active_when`.
    Digital {
        /// Boolean active state.
        active_when: bool,
    },
}

/// Runtime alarm state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlarmState {
    /// Condition is not active and no ack is pending.
    Clear,
    /// Condition is active and unacknowledged.
    Active,
    /// Condition is active and acknowledged.
    Acked,
    /// Condition has cleared after an active/acked transition.
    Cleared,
}

impl AlarmState {
    /// Stable lowercase wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            AlarmState::Clear => "clear",
            AlarmState::Active => "active",
            AlarmState::Acked => "acked",
            AlarmState::Cleared => "cleared",
        }
    }
}

/// Active alarm snapshot retained by the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveAlarm {
    /// Alarm id.
    pub alarm_id: String,
    /// Current state.
    pub state: AlarmState,
    /// Activation timestamp.
    pub activated_at_ms: Option<u64>,
    /// Last transition timestamp.
    pub transitioned_at_ms: u64,
    /// Last tag value.
    pub value: TagValue,
    /// Last tag quality.
    pub quality: Quality,
}

/// Alarm event emitted to clients and journaled on transitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlarmEvent {
    /// Alarm id.
    pub alarm_id: String,
    /// Alarm label.
    pub label: String,
    /// Priority, where `1` is highest.
    pub priority: u8,
    /// New state.
    pub state: AlarmState,
    /// Evaluated tag path.
    pub tag_path: String,
    /// Current tag value.
    pub value: TagValue,
    /// Current tag quality.
    pub quality: Quality,
    /// Timestamp when this active episode started.
    pub activated_at_ms: Option<u64>,
    /// Timestamp when this event's transition occurred.
    pub transitioned_at_ms: u64,
    /// Actor for ack transitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,
    /// Optional transition note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Rendered alarm message.
    pub message: String,
}

/// Journaled state transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlarmTransition {
    /// Alarm id.
    pub alarm_id: String,
    /// Transition timestamp.
    pub ts_ms: u64,
    /// Previous state.
    pub from_state: AlarmState,
    /// New state.
    pub to_state: AlarmState,
    /// Actor.
    pub who: Option<String>,
    /// Note.
    pub note: Option<String>,
}
