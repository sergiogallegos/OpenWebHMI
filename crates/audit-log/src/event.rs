//! Audit event and persisted entry types.

use openwebhmi_auth::Role;
use openwebhmi_protocol::{TagPath, TagValue};
use serde::{Deserialize, Serialize};

/// One persisted audit entry with actor metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Monotonic SQLite row id.
    pub id: u64,
    /// Unix epoch timestamp in milliseconds.
    pub ts_ms: u64,
    /// Authenticated actor username, if any.
    pub user: Option<String>,
    /// Session id or token identifier, when available.
    pub session_id: Option<String>,
    /// Source peer address, when captured by the gateway.
    pub source_ip: Option<String>,
    /// Structured event payload.
    pub kind: AuditEvent,
}

/// Security-relevant event payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum AuditEvent {
    /// Login attempt result.
    AuthLogin {
        /// Attempted username.
        username: String,
        /// Whether authentication succeeded.
        success: bool,
        /// Failure reason, if any.
        reason: Option<String>,
    },
    /// Logout by an authenticated user.
    AuthLogout {
        /// Username.
        username: String,
    },
    /// Session expired before use.
    SessionExpired {
        /// Username if the expired token could be associated.
        username: String,
    },
    /// Tag write attempt.
    TagWrite {
        /// Full tag path.
        path: TagPath,
        /// Requested value.
        value: TagValue,
        /// Whether the write was accepted.
        success: bool,
        /// Write source.
        source: WriteSource,
        /// Failure text, if any.
        error: Option<String>,
    },
    /// Project artifact save.
    ProjectSave {
        /// Project id.
        project_id: String,
        /// Artifact kind.
        artifact_kind: String,
    },
    /// Project export.
    ProjectExport {
        /// Project id.
        project_id: String,
        /// Whether historian data was included.
        includes_historian: bool,
        /// Whether alarm journal data was included.
        includes_alarm_journal: bool,
        /// Archive size.
        archive_size_bytes: u64,
    },
    /// Project import.
    ProjectImport {
        /// Project id.
        project_id: String,
        /// Import mode.
        mode: String,
        /// Archive size.
        archive_size_bytes: u64,
    },
    /// User administration action.
    UserAdmin {
        /// Actor username.
        actor: String,
        /// Action.
        action: UserAdminAction,
        /// Target username or user id.
        target_user: String,
    },
}

/// Origin of a tag write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteSource {
    /// WebSocket/runtime UI.
    WebSocket,
    /// Python script host.
    Script,
}

/// User administration action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserAdminAction {
    /// User created.
    Created,
    /// User deleted.
    Deleted,
    /// Role set changed.
    RoleChanged {
        /// Previous primary role, when known.
        from: Role,
        /// New primary role.
        to: Role,
    },
    /// Password changed.
    PasswordChanged,
}

impl AuditEvent {
    /// Stable event kind discriminant stored alongside JSON payload.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::AuthLogin { .. } => "AuthLogin",
            Self::AuthLogout { .. } => "AuthLogout",
            Self::SessionExpired { .. } => "SessionExpired",
            Self::TagWrite { .. } => "TagWrite",
            Self::ProjectSave { .. } => "ProjectSave",
            Self::ProjectExport { .. } => "ProjectExport",
            Self::ProjectImport { .. } => "ProjectImport",
            Self::UserAdmin { .. } => "UserAdmin",
        }
    }
}
