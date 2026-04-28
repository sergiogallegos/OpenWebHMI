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
