//! Script trigger registrations.

use openwebhmi_project_store::{ScriptConfig, ScriptTriggerConfig};
use openwebhmi_protocol::TagPath;

/// Runtime trigger registration for a script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerRegistration {
    /// Fire when a tag changes.
    OnTagChange {
        /// Subscribed tag path.
        path: TagPath,
    },
    /// Stub for timer triggers.
    OnTimer {
        /// Interval in milliseconds.
        every_ms: u64,
    },
    /// Stub for alarm triggers.
    OnAlarm {
        /// Alarm id.
        alarm_id: String,
    },
    /// Stub for button-click triggers.
    OnButtonClick {
        /// Button component id.
        component_id: String,
    },
}

impl TriggerRegistration {
    /// Extract active v1 tag-change paths from a project script config.
    pub fn tag_change_paths(script: &ScriptConfig) -> Vec<TagPath> {
        script
            .triggers
            .iter()
            .filter_map(|trigger| match trigger {
                ScriptTriggerConfig::OnTagChange { path } => Some(TagPath::from(path)),
                ScriptTriggerConfig::OnTimer { .. }
                | ScriptTriggerConfig::OnAlarm { .. }
                | ScriptTriggerConfig::OnButtonClick { .. } => None,
            })
            .collect()
    }
}
