//! CPython scripting host.
//!
//! User scripts run in normal `python3` worker subprocesses and communicate
//! with the Rust gateway using newline-delimited JSON frames on stdin/stdout.
//! The gateway process never embeds CPython directly, preserving the crash
//! boundary described in `docs/architecture.md` §4.7.

#![deny(missing_docs)]

/// Script host orchestration.
pub mod host;
/// JSON frame types for the host/worker channel.
pub mod rpc;
/// Tag write sink abstraction.
pub mod sink;
/// Script trigger types.
pub mod triggers;
/// Worker subprocess wrapper.
pub mod worker;

pub use host::{
    DEFAULT_TIMEOUT, ScriptEvent, ScriptHost, ScriptHostHandle, ScriptHostOptions, ScriptStatus,
};
pub use sink::{MemorySink, TagWriteError, TagWriteSink};
pub use triggers::TriggerRegistration;
