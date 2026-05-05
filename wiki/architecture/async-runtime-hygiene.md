---
status: active
last-validated: 2026-05-05
---

# Async runtime hygiene

## Summary
OpenWebHMI's gateway keeps blocking SQLite and bcrypt work off Tokio worker threads, drains top-level service tasks through a shared cancellation token, and uses event-driven script trigger fan-in instead of polling.

## Current understanding
1. Historian recorder writes run through `tokio::task::spawn_blocking`, so per-tag recorder tasks do not execute `rusqlite` writes on the async worker thread. Source: `crates/historian/src/recorder.rs`; regression coverage: `crates/historian/tests/store.rs`.
2. Gateway WebSocket handlers wrap blocking auth, historian read, audit query, alarm ack, backup export, and HTTP restore work in `spawn_blocking`; `JoinError` is translated to wire-level or HTTP errors instead of propagating with `?` out of the handler loop. Source: `crates/gateway/src/server.rs`.
3. Audit appends use an outer `tokio::spawn` around the inner `spawn_blocking`, preserving the existing fire-and-forget audit path without queueing the WebSocket hot path on the blocking pool. Source: `crates/gateway/src/server.rs`.
4. Alarm-engine journal transition writes are `spawn_blocking` wrapped from the alarm subscription task; the alarm state machine remains local to the async task. Source: `crates/alarm-engine/src/engine.rs`.
5. Gateway startup owns a single `tokio_util::sync::CancellationToken` and a service-level `JoinSet`; Ctrl-C or a test token cancels long-lived top-level tasks and drains them for `SHUTDOWN_GRACE = 5s` before aborting stragglers. Source: `crates/gateway/src/main.rs`.
6. Script trigger fan-in uses `tokio_stream::StreamMap<String, BroadcastStream<TagSnapshot>>`; broadcast lag logs `script_id`, tag path, and skipped count at `warn!` instead of being silently dropped. Source: `crates/scripting/src/host.rs`.

## Evidence
- Code: `crates/gateway/src/main.rs`, `crates/gateway/src/server.rs`, `crates/historian/src/recorder.rs`, `crates/alarm-engine/src/engine.rs`, `crates/scripting/src/host.rs`.
- Tests: `recorder_does_not_block_current_thread_runtime`, `fan_in_lag_is_logged_with_script_and_path`, `shutdown_token_drains_gateway_tasks`.
- Validation: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features --locked` passed on 2026-05-05; the workspace test command was run three consecutive times.

## Open questions
- Manual Ctrl-C smoke under real simulator/runtime load still needs a maintainer run with `PRAGMA integrity_check;` on history, alarm, and audit SQLite databases.
- Per-connection WebSocket handler tasks spawned below `gateway/src/main.rs` are not yet part of the top-level `JoinSet`; broader connection-drain semantics belong to a future shutdown hardening task.
- Store actor services may eventually centralize SQLite backpressure, but CODEX-AJ intentionally used call-site `spawn_blocking` wrappers only.

## Related pages
- `wiki/architecture/audit-log.md`
- `wiki/architecture/backup-restore.md`
- `docs/agents/tasks/CODEX-AJ-async-hygiene.md`
