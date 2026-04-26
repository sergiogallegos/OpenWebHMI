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
    /// Liveness ping. Gateway responds with [`ServerMessage::Pong`].
    #[serde(rename = "ping")]
    Ping,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
