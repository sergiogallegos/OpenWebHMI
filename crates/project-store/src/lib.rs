//! Persistent OpenWebHMI project storage.

#![deny(missing_docs)]

/// Storage API and filesystem persistence.
pub mod store;
/// Project, tag, driver, and view schema types.
pub mod types;
/// Project versioning and change event types.
pub mod version;

pub use store::{ArtifactKind, ProjectStore};
pub use types::{
    Binding, BindingSource, Component, DriverConfig, Project, ProjectSummary, TagConfig, View,
};
pub use version::{ChangeAction, ProjectChange};
