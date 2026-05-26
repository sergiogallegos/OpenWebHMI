//! Widget export/import wire shape.

use openwebhmi_project_store::Binding;
use serde::{Deserialize, Serialize};

/// Stable single-widget export format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExportedWidget {
    /// Export schema version. Version 1 is the v1.x compatibility floor.
    pub schema_version: u32,
    /// Component-library type id.
    pub widget_type: String,
    /// Component props serialized exactly as stored in the view.
    pub props: serde_json::Value,
    /// Component bindings serialized with full source paths.
    #[serde(default)]
    pub bindings: Vec<Binding>,
    /// Child component nodes for container-like widgets.
    #[serde(default)]
    pub children: Vec<ExportedWidgetChild>,
    /// Unix epoch milliseconds when the file was exported.
    pub exported_at: u64,
    /// Informational OpenWebHMI version string.
    pub openwebhmi_version: String,
}

/// Child component shape nested inside an exported widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExportedWidgetChild {
    /// Component-library type id.
    pub widget_type: String,
    /// Component props serialized exactly as stored in the view.
    pub props: serde_json::Value,
    /// Component bindings serialized with full source paths.
    #[serde(default)]
    pub bindings: Vec<Binding>,
    /// Nested children.
    #[serde(default)]
    pub children: Vec<ExportedWidgetChild>,
}
