//! Tag-based alarm engine for OpenWebHMI.

#![deny(missing_docs)]

/// Alarm condition evaluation.
pub mod conditions;
/// Runtime state machine and subscription handling.
pub mod engine;
/// SQLite alarm transition journal.
pub mod journal;
/// Public alarm types.
pub mod types;

pub use conditions::{ConditionError, evaluate};
pub use engine::{AlarmEngineHandle, spawn_alarm_engine};
pub use journal::{AlarmJournal, JournalEntry};
pub use types::{
    ActiveAlarm, AlarmCondition, AlarmDefinition, AlarmEvent, AlarmState, AlarmTransition,
};
