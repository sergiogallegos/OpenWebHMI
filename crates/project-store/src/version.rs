//! Version and change event types.

use serde::{Deserialize, Serialize};

use crate::store::ArtifactKind;

/// Project artifact change action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeAction {
    /// Artifact was created.
    Created,
    /// Artifact was updated.
    Updated,
    /// Artifact was deleted.
    Deleted,
}

/// Change event broadcast by [`crate::ProjectStore`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectChange {
    /// Project id.
    pub project_id: String,
    /// New project version.
    pub version: u64,
    /// Artifact that changed.
    pub artifact: ArtifactKind,
    /// Change action.
    pub action: ChangeAction,
}
