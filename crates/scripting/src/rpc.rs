//! JSON frame types for the CPython worker channel.

use openwebhmi_protocol::{Quality, TagPath, TagValue};
use serde::{Deserialize, Serialize};

/// A tag snapshot passed to a script trigger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagChangeArgs {
    /// Full tag path.
    pub tag_path: TagPath,
    /// Current value.
    pub value: TagValue,
    /// Current quality.
    pub quality: Quality,
    /// Unix epoch milliseconds.
    pub ts_ms: u64,
}

/// Frames written by the Rust host to a Python worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HostFrame {
    /// Invoke a registered trigger handler.
    #[serde(rename = "trigger")]
    Trigger {
        /// Correlation id.
        id: String,
        /// Trigger name.
        trigger: String,
        /// Trigger arguments.
        args: TagChangeArgs,
    },
    /// Successful response to a worker RPC.
    #[serde(rename = "rpc.result")]
    RpcResult {
        /// Correlation id from the worker request.
        id: String,
        /// Result payload.
        result: serde_json::Value,
    },
    /// Failed response to a worker RPC.
    #[serde(rename = "rpc.error")]
    RpcError {
        /// Correlation id from the worker request.
        id: String,
        /// Error message.
        error: String,
    },
    /// Ask the worker to stop.
    #[serde(rename = "shutdown")]
    Shutdown,
}

/// Frames written by a Python worker to the Rust host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum WorkerFrame {
    /// Worker has loaded the user script and is ready for triggers.
    #[serde(rename = "ready")]
    Ready {
        /// Script id.
        script_id: String,
    },
    /// Worker requests a gateway-side `system.*` call.
    #[serde(rename = "rpc")]
    Rpc {
        /// Correlation id.
        id: String,
        /// Method name.
        method: RpcMethod,
        /// Method arguments.
        args: serde_json::Value,
    },
    /// Trigger invocation completed successfully.
    #[serde(rename = "trigger.done")]
    TriggerDone {
        /// Correlation id from the trigger frame.
        id: String,
    },
    /// Trigger invocation failed.
    #[serde(rename = "trigger.error")]
    TriggerError {
        /// Correlation id from the trigger frame.
        id: String,
        /// Error message or traceback.
        error: String,
    },
    /// Worker emitted a script error outside a trigger.
    #[serde(rename = "script.error")]
    ScriptError {
        /// Error message.
        message: String,
        /// Optional traceback.
        #[serde(default)]
        traceback: Option<String>,
    },
}

/// RPC methods exposed by the Python `system.*` module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcMethod {
    /// `system.tag.read(path)`.
    #[serde(rename = "tag.read")]
    TagRead,
    /// `system.tag.write(path, value)`.
    #[serde(rename = "tag.write")]
    TagWrite,
    /// `system.util.now()`.
    #[serde(rename = "util.now")]
    UtilNow,
    /// `system.util.log(message)`.
    #[serde(rename = "util.log")]
    UtilLog,
}

/// Arguments for `tag.read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagReadArgs {
    /// Full tag path.
    pub path: TagPath,
}

/// Arguments for `tag.write`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagWriteArgs {
    /// Full tag path.
    pub path: TagPath,
    /// Value to publish.
    pub value: TagValue,
}

/// Arguments for `util.log`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtilLogArgs {
    /// Message to write to the gateway log.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_response_ids_are_preserved_out_of_order() {
        let first = HostFrame::RpcResult {
            id: "req-1".into(),
            result: serde_json::json!({"ok": true}),
        };
        let second = HostFrame::RpcResult {
            id: "req-2".into(),
            result: serde_json::json!({"ok": false}),
        };

        let second_json = serde_json::to_string(&second).unwrap();
        let first_json = serde_json::to_string(&first).unwrap();

        assert_eq!(
            serde_json::from_str::<HostFrame>(&second_json).unwrap(),
            second
        );
        assert_eq!(
            serde_json::from_str::<HostFrame>(&first_json).unwrap(),
            first
        );
    }
}
