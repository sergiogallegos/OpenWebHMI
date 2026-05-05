//! OpenWebHMI wire protocol types.
//!
//! All messages exchanged between the gateway and TS clients are JSON envelopes
//! with a `kind` discriminator. See `docs/architecture.md` §4.3.
//!
//! The TS counterpart of these types lives in `packages/protocol-ts`. Until
//! `ts-rs` (or `specta`) is wired in, the TS side is hand-maintained and
//! covered by integration tests.

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

pub use openwebhmi_project_store::{ArtifactKind, ChangeAction, View};

macro_rules! string_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Construct from any string-like input.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrow the underlying string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume and return the underlying string.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                Self(value.clone())
            }
        }

        impl From<&$name> for $name {
            fn from(value: &$name) -> Self {
                value.clone()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

string_newtype! {
    /// Fully qualified path to a tag in the runtime, for example `rockwell-1/Pressure`.
    ///
    /// Wire format is a transparent JSON string.
    TagPath
}

string_newtype! {
    /// Project-local driver instance id, for example `rockwell-1`.
    ///
    /// Wire format is a transparent JSON string.
    DriverId
}

/// User record exposed to administrator clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUser {
    /// Stable user id.
    pub id: String,
    /// Login username.
    pub username: String,
    /// Built-in role names.
    pub roles: Vec<String>,
}

/// Alarm state in the wire protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlarmState {
    /// Clear.
    Clear,
    /// Active and unacknowledged.
    Active,
    /// Active and acknowledged.
    Acked,
    /// Cleared after an active/acked transition.
    Cleared,
}

/// A historical tag sample returned by `history.result`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryPoint {
    /// Unix epoch milliseconds.
    pub ts_ms: u64,
    /// Historical value.
    pub value: TagValue,
    /// Historical quality.
    pub quality: Quality,
}

/// Audit query filter sent over the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuditQuery {
    /// Inclusive lower timestamp bound.
    pub from_ts_ms: Option<u64>,
    /// Inclusive upper timestamp bound.
    pub to_ts_ms: Option<u64>,
    /// Actor username filter.
    pub user: Option<String>,
    /// Event kinds, empty for all.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Maximum entries.
    pub limit: usize,
    /// Offset into matching entries.
    pub offset: usize,
}

/// Audit entry sent to clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry id.
    pub id: u64,
    /// Unix epoch milliseconds.
    pub ts_ms: u64,
    /// Actor username.
    pub user: Option<String>,
    /// Session id.
    pub session_id: Option<String>,
    /// Source peer address.
    pub source_ip: Option<String>,
    /// Event kind name.
    pub kind: String,
    /// Event payload.
    pub payload: serde_json::Value,
}

/// Backup import mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectImportMode {
    /// Replace artifacts present in the archive.
    Replace,
    /// Merge archive artifacts into the target project.
    Merge,
}

/// A tag's current value.
///
/// The runtime carries quality + timestamp alongside this value via
/// [`crate::ServerMessage::TagUpdate`].
///
/// This enum is intentionally exhaustive: adding a variant changes the v1 JSON
/// schema and should force callers to review value handling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum TagValue {
    /// Boolean.
    Bool(bool),
    /// Signed 64-bit integer.
    Int(i64),
    /// 64-bit float.
    Real(f64),
    /// UTF-8 string.
    String(String),
}

/// Quality of a tag value, OPC UA–style.
///
/// This enum is intentionally exhaustive: the v1 wire schema treats these as
/// the bounded quality states clients must handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    /// Trustworthy.
    Good,
    /// Untrustworthy (e.g. driver disconnected).
    Bad,
    /// Trust degraded but not invalidated (e.g. reconnect in progress).
    Uncertain,
    /// Last good value, but the source has not refreshed within its expected window.
    Stale,
}

/// Messages from a client to the gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[non_exhaustive]
pub enum ClientMessage {
    /// Authenticate with username/password credentials.
    #[serde(rename = "auth.login")]
    AuthLogin {
        /// Username.
        username: String,
        /// Plaintext password sent over TLS in production.
        password: String,
    },
    /// End the current client-side session.
    #[serde(rename = "auth.logout")]
    AuthLogout,
    /// Subscribe to alarm events.
    #[serde(rename = "alarm.subscribe")]
    AlarmSubscribe {
        /// Project id.
        project_id: String,
        /// Minimum priority, inclusive.
        #[serde(default)]
        priority_min: Option<u8>,
        /// Maximum priority, inclusive.
        #[serde(default)]
        priority_max: Option<u8>,
    },
    /// Acknowledge an alarm.
    #[serde(rename = "alarm.ack")]
    AlarmAck {
        /// Alarm id.
        alarm_id: String,
        /// Optional note.
        #[serde(default)]
        note: Option<String>,
    },
    /// Subscribe to script lifecycle/log/error events.
    #[serde(rename = "script.subscribe")]
    ScriptSubscribe {
        /// Project id.
        project_id: String,
    },
    /// Unsubscribe from script lifecycle/log/error events.
    #[serde(rename = "script.unsubscribe")]
    ScriptUnsubscribe {
        /// Project id.
        project_id: String,
    },
    /// Subscribe to live updates for one or more tag paths.
    #[serde(rename = "tag.subscribe")]
    TagSubscribe {
        /// Tag paths in `provider/path/with/slashes` form.
        paths: Vec<TagPath>,
    },
    /// Unsubscribe from previously-subscribed tag paths.
    #[serde(rename = "tag.unsubscribe")]
    TagUnsubscribe {
        /// Tag paths to unsubscribe from.
        paths: Vec<TagPath>,
    },
    /// Write one tag through its owning driver.
    ///
    /// Writes are serialized by the gateway's per-driver command queue;
    /// concurrent writes to the same tag are last-write-wins. A successful
    /// write has no separate acknowledgement in v1; the next
    /// [`ServerMessage::TagUpdate`] publishes the resulting value.
    #[serde(rename = "tag.write")]
    TagWrite {
        /// Full tag path including provider.
        path: TagPath,
        /// Value to write.
        value: TagValue,
    },
    /// Liveness ping. Gateway responds with [`ServerMessage::Pong`].
    #[serde(rename = "ping")]
    Ping,
    /// Subscribe to project change events.
    #[serde(rename = "project.subscribe")]
    ProjectSubscribe {
        /// Project id.
        project_id: String,
    },
    /// Unsubscribe from project change events.
    #[serde(rename = "project.unsubscribe")]
    ProjectUnsubscribe {
        /// Project id.
        project_id: String,
    },
    /// Load a project snapshot.
    #[serde(rename = "project.load")]
    ProjectLoad {
        /// Project id.
        project_id: String,
    },
    /// Read historical samples for one tag.
    #[serde(rename = "history.read")]
    HistoryRead {
        /// Optional request id echoed in the result.
        #[serde(default)]
        request_id: Option<String>,
        /// Tag path to query.
        tag_path: TagPath,
        /// Inclusive start time in Unix epoch milliseconds.
        t_start_ms: u64,
        /// Inclusive end time in Unix epoch milliseconds.
        t_end_ms: u64,
        /// Aggregation name.
        aggregation: String,
        /// Maximum points to return.
        max_points: u32,
    },
    /// Save a single project artifact.
    #[serde(rename = "project.save_artifact")]
    ProjectSaveArtifact {
        /// Optional request id echoed in save result.
        #[serde(default)]
        request_id: Option<String>,
        /// Project id.
        project_id: String,
        /// Artifact to save.
        artifact: ArtifactKind,
        /// Artifact body.
        body: serde_json::Value,
    },
    /// Read a single project artifact.
    #[serde(rename = "project.read_artifact")]
    ProjectReadArtifact {
        /// Optional request id echoed in artifact result.
        #[serde(default)]
        request_id: Option<String>,
        /// Project id.
        project_id: String,
        /// Artifact to read.
        artifact: ArtifactKind,
    },
    /// Delete a single project artifact.
    #[serde(rename = "project.delete_artifact")]
    ProjectDeleteArtifact {
        /// Optional request id echoed in delete result.
        #[serde(default)]
        request_id: Option<String>,
        /// Project id.
        project_id: String,
        /// Artifact to delete.
        artifact: ArtifactKind,
    },
    /// Request creation of a one-shot project backup download.
    #[serde(rename = "project.export")]
    ProjectExport {
        /// Request id echoed in the result.
        request_id: String,
        /// Project id.
        project_id: String,
        /// Include historian snapshot.
        include_historian: bool,
        /// Include alarm journal snapshot.
        include_alarm_journal: bool,
    },
    /// Request project import using the HTTP restore side-channel.
    #[serde(rename = "project.import")]
    ProjectImport {
        /// Request id echoed in progress/result events.
        request_id: String,
        /// Project id.
        project_id: String,
        /// Import conflict mode.
        mode: ProjectImportMode,
    },
    /// Open a view and request its current definition.
    #[serde(rename = "view.open")]
    ViewOpen {
        /// Project id.
        project_id: String,
        /// View id.
        view_id: String,
    },
    /// Close a view.
    #[serde(rename = "view.close")]
    ViewClose {
        /// Project id.
        project_id: String,
        /// View id.
        view_id: String,
    },
    /// List local users.
    #[serde(rename = "user.list")]
    UserList,
    /// Create or update a local user.
    #[serde(rename = "user.upsert")]
    UserUpsert {
        /// Username to create or update.
        username: String,
        /// Optional new password. Required for new users.
        #[serde(default)]
        password: Option<String>,
        /// Built-in role names.
        roles: Vec<String>,
    },
    /// Delete a local user.
    #[serde(rename = "user.delete")]
    UserDelete {
        /// User id.
        user_id: String,
    },
    /// Subscribe to audit events.
    #[serde(rename = "audit.subscribe")]
    AuditSubscribe {
        /// Request/subscription id.
        request_id: String,
    },
    /// Unsubscribe from audit events.
    #[serde(rename = "audit.unsubscribe")]
    AuditUnsubscribe {
        /// Request/subscription id.
        request_id: String,
    },
    /// Query audit entries.
    #[serde(rename = "audit.query")]
    AuditQuery {
        /// Request id echoed in the result.
        request_id: String,
        /// Query filter.
        query: AuditQuery,
    },
    /// Run a script entry point from the designer.
    #[serde(rename = "script.run")]
    ScriptRun {
        /// Optional request id echoed in the result.
        #[serde(default)]
        request_id: Option<String>,
        /// Script id.
        script_id: String,
        /// Trigger or entry point name.
        trigger: String,
        /// JSON arguments.
        #[serde(default)]
        args: serde_json::Value,
    },
}

/// Messages from the gateway to a client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[non_exhaustive]
pub enum ServerMessage {
    /// Authentication result.
    #[serde(rename = "auth.result")]
    AuthResult {
        /// JWT session token, or `None` on failure.
        session_token: Option<String>,
        /// User id, or `None` on failure.
        user_id: Option<String>,
        /// Built-in role names.
        roles: Vec<String>,
        /// Error message, or `None` on success.
        error: Option<String>,
    },
    /// Alarm transition event.
    #[serde(rename = "alarm.event")]
    AlarmEvent {
        /// Alarm id.
        alarm_id: String,
        /// Label.
        label: String,
        /// Priority.
        priority: u8,
        /// State.
        state: AlarmState,
        /// Tag path.
        tag_path: TagPath,
        /// Current value.
        value: TagValue,
        /// Current quality.
        quality: Quality,
        /// Activation timestamp.
        activated_at_ms: Option<u64>,
        /// Transition timestamp.
        transitioned_at_ms: u64,
        /// Actor.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        who: Option<String>,
        /// Optional note.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        /// Rendered message.
        message: String,
    },
    /// A tag's value has changed (or its first value is being delivered after subscribe).
    #[serde(rename = "tag.update")]
    TagUpdate {
        /// Full tag path including provider.
        path: TagPath,
        /// Current value.
        value: TagValue,
        /// Current quality.
        quality: Quality,
        /// Unix epoch milliseconds when the value was published.
        ts: u64,
    },
    /// Liveness response to [`ClientMessage::Ping`].
    #[serde(rename = "pong")]
    Pong,
    /// A protocol-level error.
    #[serde(rename = "error")]
    Error {
        /// Stable machine-readable error code.
        code: String,
        /// Human-readable message.
        message: String,
    },
    /// Full project snapshot.
    #[serde(rename = "project.snapshot")]
    ProjectSnapshot {
        /// Project JSON payload.
        project: serde_json::Value,
        /// Project version.
        version: u64,
    },
    /// Historical points for one tag.
    #[serde(rename = "history.result")]
    HistoryResult {
        /// Optional request id from the read request.
        request_id: Option<String>,
        /// Tag path queried.
        tag_path: TagPath,
        /// Returned points.
        points: Vec<HistoryPoint>,
    },
    /// Project artifact changed.
    #[serde(rename = "project.changed")]
    ProjectChanged {
        /// Project id.
        project_id: String,
        /// Project version.
        version: u64,
        /// Changed artifact.
        artifact: ArtifactKind,
        /// Change action.
        action: ChangeAction,
    },
    /// Result of a project artifact save.
    #[serde(rename = "project.save_result")]
    ProjectSaveResult {
        /// Optional request id echoed from client.
        request_id: Option<String>,
        /// Project id.
        project_id: String,
        /// New project version.
        version: u64,
    },
    /// Body of a project artifact read.
    #[serde(rename = "project.artifact")]
    ProjectArtifact {
        /// Optional request id echoed from client.
        request_id: Option<String>,
        /// Project id.
        project_id: String,
        /// Artifact read.
        artifact: ArtifactKind,
        /// Artifact body, or `null` when missing.
        body: serde_json::Value,
    },
    /// Result of a project artifact delete.
    #[serde(rename = "project.delete_result")]
    ProjectDeleteResult {
        /// Optional request id echoed from client.
        request_id: Option<String>,
        /// Project id.
        project_id: String,
        /// Artifact deleted.
        artifact: ArtifactKind,
    },
    /// Backup archive is ready for HTTP download.
    #[serde(rename = "project.export_ready")]
    ProjectExportReady {
        /// Request id.
        request_id: String,
        /// One-shot download URL.
        download_url: String,
        /// Archive size in bytes.
        size_bytes: u64,
    },
    /// Restore progress update.
    #[serde(rename = "project.import_progress")]
    ProjectImportProgress {
        /// Request id.
        request_id: String,
        /// Current phase.
        phase: String,
        /// Completion estimate, 0-100.
        percent: u8,
    },
    /// Restore completion result.
    #[serde(rename = "project.import_result")]
    ProjectImportResult {
        /// Request id.
        request_id: String,
        /// Whether import completed.
        ok: bool,
        /// Error text when `ok` is false.
        error: Option<String>,
    },
    /// View definition for an opened view.
    #[serde(rename = "view.definition")]
    ViewDefinition {
        /// Project id.
        project_id: String,
        /// View id.
        view_id: String,
        /// Project version that produced this view.
        version: u64,
        /// View schema.
        view: View,
    },
    /// Local user list.
    #[serde(rename = "user.list")]
    UserList {
        /// Users visible to administrators.
        users: Vec<AuthUser>,
    },
    /// Script run completed.
    #[serde(rename = "script.result")]
    ScriptResult {
        /// Optional request id from the run request.
        request_id: Option<String>,
        /// Script id.
        script_id: String,
        /// Result payload.
        result: serde_json::Value,
    },
    /// Script run failed.
    #[serde(rename = "script.error")]
    ScriptError {
        /// Optional request id from the run request.
        request_id: Option<String>,
        /// Script id.
        script_id: String,
        /// Error message.
        message: String,
    },
    /// Streaming script lifecycle/log/error event.
    #[serde(rename = "script.event")]
    ScriptEvent {
        /// Project id.
        project_id: String,
        /// Script id.
        script_id: String,
        /// Event kind: `status`, `log`, or `error`.
        event_kind: String,
        /// Optional status value for `status` events.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        /// Optional message for `log` and `error` events.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Live audit event.
    #[serde(rename = "audit.event")]
    AuditEvent {
        /// Audit entry.
        entry: AuditEntry,
    },
    /// Audit query result.
    #[serde(rename = "audit.query_result")]
    AuditQueryResult {
        /// Request id.
        request_id: String,
        /// Matching entries.
        entries: Vec<AuditEntry>,
        /// Total matches before pagination.
        total: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use openwebhmi_project_store::{Binding, BindingSource, Component};

    #[test]
    fn client_subscribe_round_trips_with_stable_wire_form() {
        let m = ClientMessage::TagSubscribe {
            paths: vec!["system/sim/sin".into()],
        };
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"tag.subscribe","paths":["system/sim/sin"]}"#
        );
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn client_unsubscribe_round_trips() {
        let m = ClientMessage::TagUnsubscribe {
            paths: vec!["a/b".into(), "c/d".into()],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn client_tag_write_round_trips_with_stable_wire_form() {
        let m = ClientMessage::TagWrite {
            path: "rockwell-1/Setpoint".into(),
            value: TagValue::Real(42.5),
        };
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"tag.write","path":"rockwell-1/Setpoint","value":{"type":"real","value":42.5}}"#
        );
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn auth_messages_wire_form_is_stable() {
        let login = ClientMessage::AuthLogin {
            username: "admin".into(),
            password: "secret".into(),
        };
        assert_eq!(
            serde_json::to_string(&login).unwrap(),
            r#"{"kind":"auth.login","username":"admin","password":"secret"}"#
        );

        let result = ServerMessage::AuthResult {
            session_token: Some("jwt".into()),
            user_id: Some("u1".into()),
            roles: vec!["Administrator".into()],
            error: None,
        };
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"auth.result","session_token":"jwt","user_id":"u1","roles":["Administrator"],"error":null}"#
        );
    }

    #[test]
    fn alarm_ack_and_event_wire_form_is_stable() {
        let ack = ClientMessage::AlarmAck {
            alarm_id: "pressure-high".into(),
            note: Some("checked".into()),
        };
        assert_eq!(
            serde_json::to_string(&ack).unwrap(),
            r#"{"kind":"alarm.ack","alarm_id":"pressure-high","note":"checked"}"#
        );

        let event = ServerMessage::AlarmEvent {
            alarm_id: "pressure-high".into(),
            label: "High pressure".into(),
            priority: 2,
            state: AlarmState::Active,
            tag_path: "rockwell-1/Pressure".into(),
            value: TagValue::Real(250.0),
            quality: Quality::Good,
            activated_at_ms: Some(10),
            transitioned_at_ms: 10,
            who: None,
            note: None,
            message: "Pressure high: 250".into(),
        };
        assert_eq!(
            serde_json::to_string(&event).unwrap(),
            r#"{"kind":"alarm.event","alarm_id":"pressure-high","label":"High pressure","priority":2,"state":"active","tag_path":"rockwell-1/Pressure","value":{"type":"real","value":250.0},"quality":"good","activated_at_ms":10,"transitioned_at_ms":10,"message":"Pressure high: 250"}"#
        );
    }

    #[test]
    fn client_ping_serializes_minimally() {
        let m = ClientMessage::Ping;
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, r#"{"kind":"ping"}"#);
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn history_read_wire_form_is_stable() {
        let m = ClientMessage::HistoryRead {
            request_id: Some("r1".into()),
            tag_path: "rockwell-1/Pressure".into(),
            t_start_ms: 10,
            t_end_ms: 20,
            aggregation: "avg".into(),
            max_points: 100,
        };
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"history.read","request_id":"r1","tag_path":"rockwell-1/Pressure","t_start_ms":10,"t_end_ms":20,"aggregation":"avg","max_points":100}"#
        );
        assert_eq!(serde_json::from_str::<ClientMessage>(&json).unwrap(), m);
    }

    #[test]
    fn audit_query_and_result_wire_form_is_stable() {
        let query = ClientMessage::AuditQuery {
            request_id: "audit-r1".into(),
            query: AuditQuery {
                from_ts_ms: Some(10),
                to_ts_ms: None,
                user: Some("admin".into()),
                kinds: vec!["AuthLogin".into()],
                limit: 50,
                offset: 0,
            },
        };
        assert_eq!(
            serde_json::to_string(&query).unwrap(),
            r#"{"kind":"audit.query","request_id":"audit-r1","query":{"from_ts_ms":10,"to_ts_ms":null,"user":"admin","kinds":["AuthLogin"],"limit":50,"offset":0}}"#
        );

        let result = ServerMessage::AuditQueryResult {
            request_id: "audit-r1".into(),
            entries: vec![AuditEntry {
                id: 1,
                ts_ms: 10,
                user: Some("admin".into()),
                session_id: None,
                source_ip: Some("127.0.0.1:8080".into()),
                kind: "AuthLogin".into(),
                payload: serde_json::json!({
                    "type": "AuthLogin",
                    "username": "admin",
                    "success": true,
                    "reason": null
                }),
            }],
            total: 1,
        };
        assert_eq!(
            serde_json::from_str::<ServerMessage>(&serde_json::to_string(&result).unwrap())
                .unwrap(),
            result
        );
    }

    #[test]
    fn server_tag_update_round_trips() {
        let m = ServerMessage::TagUpdate {
            path: "system/sim/sin".into(),
            value: TagValue::Real(0.5),
            quality: Quality::Good,
            ts: 1_714_128_000_000,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn server_error_round_trips() {
        let m = ServerMessage::Error {
            code: "auth.required".into(),
            message: "session expired".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn tag_value_round_trips_for_all_variants() {
        let cases = vec![
            TagValue::Bool(true),
            TagValue::Bool(false),
            TagValue::Int(-42),
            TagValue::Int(0),
            TagValue::Real(12.25),
            TagValue::String("hello".into()),
            TagValue::String(String::new()),
        ];
        for v in cases {
            let json = serde_json::to_string(&v).unwrap();
            let back: TagValue = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn tag_value_wire_form_is_adjacently_tagged() {
        let v = TagValue::Real(2.5);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, r#"{"type":"real","value":2.5}"#);
    }

    #[test]
    fn quality_round_trips() {
        for q in [
            Quality::Good,
            Quality::Bad,
            Quality::Uncertain,
            Quality::Stale,
        ] {
            let json = serde_json::to_string(&q).unwrap();
            let back: Quality = serde_json::from_str(&json).unwrap();
            assert_eq!(q, back);
        }
    }

    #[test]
    fn quality_wire_form_is_lowercase() {
        assert_eq!(serde_json::to_string(&Quality::Good).unwrap(), r#""good""#);
        assert_eq!(
            serde_json::to_string(&Quality::Uncertain).unwrap(),
            r#""uncertain""#
        );
    }

    #[test]
    fn project_save_artifact_wire_form_is_stable() {
        let m = ClientMessage::ProjectSaveArtifact {
            request_id: Some("r1".into()),
            project_id: "demo".into(),
            artifact: ArtifactKind::View { id: "home".into() },
            body: serde_json::json!({"id":"home"}),
        };
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            r#"{"kind":"project.save_artifact","request_id":"r1","project_id":"demo","artifact":{"kind":"view","id":"home"},"body":{"id":"home"}}"#
        );
    }

    #[test]
    fn project_read_artifact_wire_form_is_stable() {
        let m = ClientMessage::ProjectReadArtifact {
            request_id: Some("script-r1".into()),
            project_id: "demo".into(),
            artifact: ArtifactKind::ScriptSource {
                id: "derived-setpoint".into(),
            },
        };
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            r#"{"kind":"project.read_artifact","request_id":"script-r1","project_id":"demo","artifact":{"kind":"script_source","id":"derived-setpoint"}}"#
        );

        let result = ServerMessage::ProjectArtifact {
            request_id: Some("script-r1".into()),
            project_id: "demo".into(),
            artifact: ArtifactKind::ScriptSource {
                id: "derived-setpoint".into(),
            },
            body: serde_json::json!({"source":"import system\n"}),
        };
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"project.artifact","request_id":"script-r1","project_id":"demo","artifact":{"kind":"script_source","id":"derived-setpoint"},"body":{"source":"import system\n"}}"#
        );
    }

    #[test]
    fn project_backup_wire_form_is_stable() {
        let export = ClientMessage::ProjectExport {
            request_id: "backup-r1".into(),
            project_id: "demo".into(),
            include_historian: true,
            include_alarm_journal: false,
        };
        assert_eq!(
            serde_json::to_string(&export).unwrap(),
            r#"{"kind":"project.export","request_id":"backup-r1","project_id":"demo","include_historian":true,"include_alarm_journal":false}"#
        );

        let import = ClientMessage::ProjectImport {
            request_id: "restore-r1".into(),
            project_id: "demo".into(),
            mode: ProjectImportMode::Replace,
        };
        assert_eq!(
            serde_json::to_string(&import).unwrap(),
            r#"{"kind":"project.import","request_id":"restore-r1","project_id":"demo","mode":"replace"}"#
        );

        let ready = ServerMessage::ProjectExportReady {
            request_id: "backup-r1".into(),
            download_url: "/api/projects/demo/backup/token".into(),
            size_bytes: 42,
        };
        assert_eq!(
            serde_json::to_string(&ready).unwrap(),
            r#"{"kind":"project.export_ready","request_id":"backup-r1","download_url":"/api/projects/demo/backup/token","size_bytes":42}"#
        );
    }

    #[test]
    fn view_tree_round_trips() {
        let view = sample_view();
        let json = serde_json::to_string(&view).unwrap();
        assert_eq!(serde_json::from_str::<View>(&json).unwrap(), view);
    }

    #[test]
    fn view_open_and_definition_round_trip() {
        let open = ClientMessage::ViewOpen {
            project_id: "demo".into(),
            view_id: "home".into(),
        };
        assert_eq!(
            serde_json::to_string(&open).unwrap(),
            r#"{"kind":"view.open","project_id":"demo","view_id":"home"}"#
        );

        let definition = ServerMessage::ViewDefinition {
            project_id: "demo".into(),
            view_id: "home".into(),
            version: 7,
            view: sample_view(),
        };
        let json = serde_json::to_string(&definition).unwrap();
        let back = serde_json::from_str::<ServerMessage>(&json).unwrap();
        assert_eq!(back, definition);
    }

    #[test]
    fn script_lifecycle_wire_form_is_stable() {
        let run = ClientMessage::ScriptRun {
            request_id: Some("r1".into()),
            script_id: "derived-setpoint".into(),
            trigger: "on_tag_change".into(),
            args: serde_json::json!({"tag_path":"rockwell-1/Pressure"}),
        };
        assert_eq!(
            serde_json::to_string(&run).unwrap(),
            r#"{"kind":"script.run","request_id":"r1","script_id":"derived-setpoint","trigger":"on_tag_change","args":{"tag_path":"rockwell-1/Pressure"}}"#
        );

        let result = ServerMessage::ScriptResult {
            request_id: Some("r1".into()),
            script_id: "derived-setpoint".into(),
            result: serde_json::json!(null),
        };
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"script.result","request_id":"r1","script_id":"derived-setpoint","result":null}"#
        );

        let error = ServerMessage::ScriptError {
            request_id: Some("r1".into()),
            script_id: "derived-setpoint".into(),
            message: "boom".into(),
        };
        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            r#"{"kind":"script.error","request_id":"r1","script_id":"derived-setpoint","message":"boom"}"#
        );
    }

    #[test]
    fn script_streaming_wire_form_is_stable() {
        let subscribe = ClientMessage::ScriptSubscribe {
            project_id: "phase1-demo".into(),
        };
        assert_eq!(
            serde_json::to_string(&subscribe).unwrap(),
            r#"{"kind":"script.subscribe","project_id":"phase1-demo"}"#
        );

        let unsubscribe = ClientMessage::ScriptUnsubscribe {
            project_id: "phase1-demo".into(),
        };
        assert_eq!(
            serde_json::to_string(&unsubscribe).unwrap(),
            r#"{"kind":"script.unsubscribe","project_id":"phase1-demo"}"#
        );

        let event = ServerMessage::ScriptEvent {
            project_id: "phase1-demo".into(),
            script_id: "derived-setpoint".into(),
            event_kind: "error".into(),
            status: None,
            message: Some("NameError: name 'sytem' is not defined".into()),
        };
        assert_eq!(
            serde_json::to_string(&event).unwrap(),
            r#"{"kind":"script.event","project_id":"phase1-demo","script_id":"derived-setpoint","event_kind":"error","message":"NameError: name 'sytem' is not defined"}"#
        );
        assert_eq!(
            serde_json::from_str::<ServerMessage>(&serde_json::to_string(&event).unwrap()).unwrap(),
            event
        );
    }

    fn sample_view() -> View {
        View {
            id: "home".into(),
            title: "Home".into(),
            allowed_roles: None,
            schema_version: 1,
            root: Component {
                id: "root".into(),
                kind: "Container".into(),
                props: serde_json::json!({}),
                bindings: Vec::new(),
                children: vec![Component {
                    id: "nested".into(),
                    kind: "Container".into(),
                    props: serde_json::json!({}),
                    bindings: Vec::new(),
                    children: vec![Component {
                        id: "pressure".into(),
                        kind: "ValueDisplay".into(),
                        props: serde_json::json!({"format":"number"}),
                        bindings: vec![Binding {
                            prop: "value".into(),
                            source: BindingSource::Tag {
                                path: "rockwell-1/Pressure".into(),
                            },
                        }],
                        children: Vec::new(),
                    }],
                }],
            },
        }
    }
}
