---
status: active
last-validated: 2026-05-05
---

# Async runtime hygiene

## Summary
OpenWebHMI's gateway keeps blocking SQLite and bcrypt work off Tokio worker threads, drains service and per-connection tasks through a shared cancellation token, and uses event-driven script trigger fan-in instead of polling.

## Current understanding
1. Historian recorder writes run through `tokio::task::spawn_blocking`, so per-tag recorder tasks do not execute `rusqlite` writes on the async worker thread. Source: `crates/historian/src/recorder.rs`; regression coverage: `crates/historian/tests/store.rs`.
2. Gateway WebSocket handlers wrap blocking auth, historian read, audit query, alarm ack, backup export, and HTTP restore work in `spawn_blocking`; `JoinError` is translated to wire-level or HTTP errors instead of propagating with `?` out of the handler loop. Source: `crates/gateway/src/server.rs`.
3. Audit appends use an outer `tokio::spawn` around the inner `spawn_blocking`, preserving the existing fire-and-forget audit path without queueing the WebSocket hot path on the blocking pool. Source: `crates/gateway/src/server.rs`.
4. Alarm-engine journal transition writes are `spawn_blocking` wrapped from the alarm subscription task; the alarm state machine remains local to the async task. Source: `crates/alarm-engine/src/engine.rs`.
5. Gateway startup owns a single `tokio_util::sync::CancellationToken` and a service-level `JoinSet`; Ctrl-C or a test token cancels long-lived top-level tasks and drains them for `SHUTDOWN_GRACE = 5s` before aborting stragglers. Source: `crates/gateway/src/main.rs`.
6. Script trigger fan-in uses `tokio_stream::StreamMap<String, BroadcastStream<TagSnapshot>>`; broadcast lag logs `script_id`, tag path, and skipped count at `warn!` instead of being silently dropped. Source: `crates/scripting/src/host.rs`.
7. Per-connection WebSocket handlers are registered in the gateway service `JoinSet` and receive the shared shutdown token; on shutdown, the writer attempts a WebSocket close frame with code `1001` and reason `gateway shutting down` before the drain timeout. Source: `crates/gateway/src/main.rs`, `crates/gateway/src/server.rs`; regression coverage: `shutdown_token_sends_websocket_close_frame`.
8. Cancel-only task maps use `tokio::task::AbortHandle` rather than storing `JoinHandle<()>` when the only operation is later cancellation. Source: `crates/alarm-engine/src/engine.rs`, `crates/historian/src/recorder.rs`, `crates/gateway/src/server.rs`, `crates/driver-api/src/supervisor.rs`.

## Evidence
- Code: `crates/gateway/src/main.rs`, `crates/gateway/src/server.rs`, `crates/historian/src/recorder.rs`, `crates/alarm-engine/src/engine.rs`, `crates/scripting/src/host.rs`.
- Tests: `recorder_does_not_block_current_thread_runtime`, `fan_in_lag_is_logged_with_script_and_path`, `shutdown_token_drains_gateway_tasks`, `shutdown_token_drains_project_gateway_tasks`, `shutdown_token_sends_websocket_close_frame`.
- Validation: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo doc --workspace --no-deps`, `cargo test --workspace --all-features --locked`, `pnpm -r typecheck`, and `pnpm -r test` passed on 2026-05-05; the workspace test command was run three consecutive times.

## Open questions
- Manual Ctrl-C smoke under real simulator/runtime load still needs a maintainer run with `PRAGMA integrity_check;` on history, alarm, and audit SQLite databases.
- Store actor services may eventually centralize SQLite backpressure, but CODEX-AJ intentionally used call-site `spawn_blocking` wrappers only.
- `WorkerProc::invoke_tag_change` is documented as not cancel-safe; dropping it mid-await may leave an in-flight script invocation running until completion or worker exit. Explicit trigger cancellation is a v1.1 design question.

## Related pages
- `wiki/architecture/audit-log.md`
- `wiki/architecture/backup-restore.md`
- `docs/agents/tasks/CODEX-AJ-async-hygiene.md`
- `docs/agents/tasks/CODEX-AK-tokio-handle-ergonomics.md`
