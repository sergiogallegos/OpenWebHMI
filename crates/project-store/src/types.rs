//! Project and view schema types.

use serde::{Deserialize, Serialize};

/// A loaded project with metadata, drivers, tags, and views.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Stable project id, derived from its directory name.
    pub id: String,
    /// Project schema version. Phase 2 accepts `1`.
    pub schema_version: u32,
    /// Human-readable project name.
    pub name: String,
    /// Monotonic project version from the metadata index.
    pub version: u64,
    /// Driver instances.
    pub drivers: Vec<DriverConfig>,
    /// Driver-backed tags.
    pub tags: Vec<TagConfig>,
    /// Alarm definitions.
    #[serde(default)]
    pub alarms: Vec<AlarmConfig>,
    /// Project-level theme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    /// Python script definitions.
    #[serde(default)]
    pub scripts: Vec<ScriptConfig>,
    /// View definitions.
    pub views: Vec<View>,
}

/// Project summary returned by list operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSummary {
    /// Stable project id.
    pub id: String,
    /// Current project version.
    pub version: u64,
    /// Last modification timestamp as Unix epoch seconds.
    pub last_modified: i64,
}

/// Project-level CSS variable theme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    /// Light mode variables.
    pub light: ThemeVariables,
    /// Dark mode variables.
    pub dark: ThemeVariables,
    /// Default active mode.
    #[serde(default)]
    pub mode: ThemeMode,
    /// Optional project-level component pack id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack: Option<String>,
}

/// Theme mode persisted with a project theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    /// Light theme.
    #[default]
    Light,
    /// Dark theme.
    Dark,
}

/// Named CSS variables exposed by the designer theme editor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeVariables {
    /// Primary color.
    pub primary_color: String,
    /// Secondary color.
    pub secondary_color: String,
    /// Page background.
    pub background: String,
    /// Surface background.
    pub surface: String,
    /// Primary text color.
    pub text_primary: String,
    /// Secondary text color.
    pub text_secondary: String,
    /// Accent color.
    pub accent: String,
    /// Error color.
    pub error: String,
    /// Warning color.
    pub warning: String,
    /// Font family.
    pub font_family: String,
    /// Base font size in pixels.
    pub font_size_base: u32,
    /// Spacing unit in pixels.
    pub spacing_unit: u32,
    /// Border radius in pixels.
    pub border_radius: u32,
}

/// Driver configuration block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriverConfig {
    /// Stable project-local driver id.
    pub id: String,
    /// Driver type, for example `rockwell`.
    #[serde(rename = "type")]
    pub driver_type: String,
    /// Driver-specific configuration.
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Tag mapping from a project path to a driver address.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagConfig {
    /// Gateway tag path exposed to runtime clients.
    pub path: String,
    /// Referenced driver id.
    pub driver: String,
    /// Driver-native tag address.
    pub address: String,
    /// Optional historian recording configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,
}

/// Optional tag history recording configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Minimum interval between recorded samples.
    #[serde(default)]
    pub rate_ms: Option<u64>,
    /// Numeric deadband; non-numeric values ignore it.
    #[serde(default)]
    pub deadband: Option<f64>,
}

/// Project alarm definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlarmConfig {
    /// Stable alarm id.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Priority, where `1` is highest and `5` is lowest.
    pub priority: u8,
    /// Tag path this alarm evaluates.
    pub tag_path: String,
    /// Activation condition.
    pub condition: AlarmConditionConfig,
    /// Message template.
    pub message: String,
    /// Whether this alarm is enabled.
    pub enabled: bool,
    /// Whether an operator acknowledgement is required before clear.
    pub require_ack: bool,
}

/// Project alarm condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AlarmConditionConfig {
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
    /// Activates when value equals configured value.
    Equals {
        /// Expected value.
        value: serde_json::Value,
    },
    /// Activates when numeric value differs from setpoint by more than tolerance.
    Deviation {
        /// Expected setpoint.
        setpoint: f64,
        /// Maximum allowed absolute deviation.
        tolerance: f64,
    },
    /// Activates when boolean value equals `active_when`.
    Digital {
        /// Active boolean value.
        active_when: bool,
    },
}

/// Project script definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// Stable script id.
    pub id: String,
    /// Path to the Python source file. Relative paths are resolved from the
    /// project directory when the project is loaded.
    pub path: String,
    /// Whether this script is active.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Trigger registrations for this script.
    #[serde(default)]
    pub triggers: Vec<ScriptTriggerConfig>,
    /// Optional per-handler timeout override in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handler_timeout_ms: Option<u64>,
}

/// Script trigger configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScriptTriggerConfig {
    /// Fire when a tag changes.
    OnTagChange {
        /// Subscribed tag path.
        path: String,
    },
    /// Placeholder for timer triggers.
    OnTimer {
        /// Interval in milliseconds.
        every_ms: u64,
    },
    /// Placeholder for alarm triggers.
    OnAlarm {
        /// Alarm id.
        alarm_id: String,
    },
    /// Placeholder for designer/runtime button-click triggers.
    OnButtonClick {
        /// Button component id.
        component_id: String,
    },
}

/// HMI view definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct View {
    /// Unique view id within the project.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Roles allowed to write tags from this view. When omitted, role defaults apply.
    #[serde(
        default,
        rename = "allowedRoles",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_roles: Option<Vec<String>>,
    /// Root component of the view tree.
    pub root: Component,
    /// View schema version. Phase 2 accepts `1`.
    pub schema_version: u32,
}

/// Component node in a view tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    /// Unique component id within this view.
    pub id: String,
    /// Component kind from the component library registry.
    pub kind: String,
    /// Component-specific props.
    #[serde(default)]
    pub props: serde_json::Value,
    /// Bindings for component props. If multiple bindings target the same prop,
    /// the last binding wins.
    #[serde(default)]
    pub bindings: Vec<Binding>,
    /// Child component nodes for containers.
    #[serde(default)]
    pub children: Vec<Component>,
}

/// A binding from a component prop to a source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    /// Prop name on the component.
    pub prop: String,
    /// Binding source.
    pub source: BindingSource,
}

/// Source of a component binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BindingSource {
    /// Read the value from a tag path.
    Tag {
        /// Tag path.
        path: String,
    },
    /// Placeholder for Phase 3 expression evaluation.
    Expression {
        /// Expression source code.
        code: String,
    },
    /// Constant JSON value.
    Constant {
        /// Constant value.
        value: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProjectMetaFile {
    pub(crate) schema_version: u32,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) drivers: Vec<DriverConfigToml>,
    #[serde(default)]
    pub(crate) tags: Vec<TagConfig>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub(crate) struct DriverConfigToml {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) driver_type: String,
    pub(crate) config: toml::Value,
}

impl DriverConfigToml {
    pub(crate) fn into_driver_config(self) -> anyhow::Result<DriverConfig> {
        Ok(DriverConfig {
            id: self.id,
            driver_type: self.driver_type,
            config: serde_json::to_value(self.config)?,
        })
    }
}
