//! Audit event and persisted entry types.

use openwebhmi_auth::Role;
use openwebhmi_protocol::{TagPath, TagValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    /// Previous entry hash in the append-only chain.
    #[serde(with = "optional_hash_hex")]
    pub prev_hash: Option<[u8; 32]>,
    /// This entry's SHA-256 hash.
    #[serde(with = "hash_hex")]
    pub hash: [u8; 32],
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
    /// One-way schema migration completed for the audit journal.
    MigrationCompleted {
        /// Rows backfilled into the tamper-evident hash chain.
        rows: u64,
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
            Self::MigrationCompleted { .. } => "MigrationCompleted",
        }
    }
}

/// Encode a SHA-256 audit-chain hash as lowercase hexadecimal.
pub fn encode_hash(hash: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in hash {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Decode a lowercase or uppercase hexadecimal SHA-256 audit-chain hash.
pub fn decode_hash(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", value.len()));
    }
    let mut out = [0_u8; 32];
    let bytes = value.as_bytes();
    for idx in 0..32 {
        let high = decode_nibble(bytes[idx * 2])?;
        let low = decode_nibble(bytes[idx * 2 + 1])?;
        out[idx] = (high << 4) | low;
    }
    Ok(out)
}

fn decode_nibble(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(format!("invalid hex byte {byte}")),
    }
}

mod hash_hex {
    use super::*;

    pub(crate) fn serialize<S>(hash: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode_hash(hash))
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        decode_hash(&value).map_err(serde::de::Error::custom)
    }
}

mod optional_hash_hex {
    use super::*;

    pub(crate) fn serialize<S>(hash: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match hash {
            Some(hash) => serializer.serialize_some(&encode_hash(hash)),
            None => serializer.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        value
            .as_deref()
            .map(decode_hash)
            .transpose()
            .map_err(serde::de::Error::custom)
    }
}
