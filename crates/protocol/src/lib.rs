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

/// A tag's current value.
///
/// The runtime carries quality + timestamp alongside this value via
/// [`crate::ServerMessage::TagUpdate`].
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
pub enum ClientMessage {
    /// Subscribe to live updates for one or more tag paths.
    #[serde(rename = "tag.subscribe")]
    TagSubscribe {
        /// Tag paths in `provider/path/with/slashes` form.
        paths: Vec<String>,
    },
    /// Unsubscribe from previously-subscribed tag paths.
    #[serde(rename = "tag.unsubscribe")]
    TagUnsubscribe {
        /// Tag paths to unsubscribe from.
        paths: Vec<String>,
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
        path: String,
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
}

/// Messages from the gateway to a client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ServerMessage {
    /// A tag's value has changed (or its first value is being delivered after subscribe).
    #[serde(rename = "tag.update")]
    TagUpdate {
        /// Full tag path including provider.
        path: String,
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
    fn client_ping_serializes_minimally() {
        let m = ClientMessage::Ping;
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, r#"{"kind":"ping"}"#);
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
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

    fn sample_view() -> View {
        View {
            id: "home".into(),
            title: "Home".into(),
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
