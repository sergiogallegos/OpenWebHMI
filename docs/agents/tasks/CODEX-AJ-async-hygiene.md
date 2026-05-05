---
id: CODEX-AJ
title: Async hygiene — blocking SQLite, coordinated shutdown, scripting fan-in
owner: codex
phase: 4
status: open
created: 2026-05-05
last-update: 2026-05-05 claude
---

# CODEX-AJ — Async hygiene (Tier 1 from review pass 2)

## Brief

> Three runtime-health gaps surfaced by the second-pass Rust 1.95 / edition 2024 review against tokio/axum/ripgrep idioms. All three are correctness-or-perf concerns, not style: (1) blocking rusqlite calls happen inside async tasks, parking tokio worker threads on disk I/O; (2) the gateway's Ctrl-C handler exits without draining script/alarm/historian/backup spawns, so writes can be cut mid-row; (3) the per-script supervisor's tag-snapshot fan-in busy-polls all subscribed receivers every 10 ms, wasting wakes and silently swallowing broadcast lag. This brief is **scope-locked** to those three fixes — Tier 2 (`JoinSet` adoption inside subsystems, `#[must_use]`, cancel-safety doc sweep), Tier 3 (newtypes, builder split, `#[non_exhaustive]`, cheap-clone handles), and Tier 4 (`thiserror = "2"`, format-capture sweep, doc-deny consistency) follow as separate CODEX-AK / -AL / -AM tasks. Do not bleed into them.

### Goal

After this lands:

- No blocking SQLite call (`HistorianStore::*`, `AuditLog::*`, `AlarmJournal::*`, `UserStore::authenticate`) is invoked synchronously from inside an `async fn` or `tokio::spawn`-ed task. Every such call site is wrapped in `tokio::task::spawn_blocking` and `.await`-ed, with `JoinError` translated to a wire-level error.
- The gateway has a single top-level `tokio_util::sync::CancellationToken`. Every long-lived task spawned by `gateway/src/main.rs` (script-runtime watcher, alarm-runtime trampoline, historian recorder watcher, backup HTTP side-channel) selects on its work future + `cancellation.cancelled()`. On `ctrl_c`, `main` cancels the token and awaits a `JoinSet` drain with a bounded grace period before returning.
- The script-host supervisor no longer busy-polls. `recv_any` is replaced by a `tokio_stream::StreamMap<String, BroadcastStream<TagSnapshot>>` consumed via `next().await`. `BroadcastStream::Lagged` errors are surfaced as `tracing::warn!` with the script id, tag path, and skip count — turning a silent drop into an observability win.

### Context to read first

- `crates/historian/src/recorder.rs:114-128` — `run_one` calls sync `maybe_write` from inside `rx.recv().await`; this is the highest-write-rate offender (one task per recorded tag).
- `crates/historian/src/store.rs:24-74` — `HistorianStore` is `#[derive(Clone)]` (Arc inside); `write_sample` and `read` are sync rusqlite calls. Cheap to move into `spawn_blocking`. Same shape:
- `crates/audit-log/src/store.rs:16, 52, 63, 100` — `AuditLog` is `#[derive(Clone)]`; `append`, `append_at`, `query` are sync.
- `crates/alarm-engine/src/journal.rs:11, 36, 53` — `AlarmJournal` is `#[derive(Clone)]`; `write_transition`, `read_alarm` are sync.
- `crates/auth/src/users.rs:49, 234` — `UserStore` holds `Arc<Mutex<Connection>>`; `authenticate` runs bcrypt verification (CPU-bound, multi-ms) plus a SQLite query.
- `crates/alarm-engine/src/engine.rs:70-85` — `AlarmEngine::ack` is sync, but called from an async WS handler at `crates/gateway/src/server.rs:532`. **Wrap the call site, not the method** — do not restructure `AlarmEngine` itself.
- `crates/gateway/src/server.rs`:
  - `:41` — `static DEFAULT_AUDIT_LOG: OnceLock<AuditLog>` (already on modern std primitives — no change needed there).
  - `:373` — `auth.users.authenticate(...)` from inside the async login handler.
  - `:700-746` — `history.read` handler; `historian.read(...)` at `:729`.
  - `:1080-1126` — `audit.subscribe` (channel ok) and `audit.query`; the latter is at `:1108`.
  - `:1313-1322` — `append_audit` helper; **single chokepoint** for every audit append. Wrap once.
  - Every `AlarmEngine::ack` / `journal.write_transition` / `journal.read_alarm` reachable from a WS handler — search the file.
- `crates/gateway/src/main.rs:99-202` — current spawn pattern for backup HTTP side-channel and script-runtime watcher; `:122-129` is the only existing shutdown signal. The `pending::<()>` fragment at `:217-224` is the explicit "no shutdown channel exists" hack — replace it.
- `crates/scripting/src/host.rs:343-406` — supervisor `select!` (lines 351-384) and `recv_any` (lines 387-406). Note the `biased` keyword on the select; preserve control/worker priority over fan-in.
- The cancel-safety contracts: `tokio::sync::broadcast::Receiver::recv` (cancel-safe), `tokio_stream::wrappers::BroadcastStream::next` (cancel-safe). Re-verify before changing the supervisor.

### Files to modify

- **`crates/historian/src/recorder.rs`** — wrap the two `maybe_write` call sites (line 118 startup snapshot, line 122 in-loop) in `tokio::task::spawn_blocking`. The `historian` handle is `Clone`; clone it into the closure. The `state: &mut FilterState` borrow does not cross `.await` — keep `state` updates on the async side; pass owned `snapshot` + `config` into the blocking task.
- **`crates/gateway/src/server.rs`** — wrap every direct sync-store call from inside an async handler:
  - `historian.read(...)` at `:729`.
  - `audit_log.query(...)` at `:1108`.
  - `append_audit` (`:1313`) — wrap the inner `audit_log.append(...)` once. Per CLAUDE.md the wire path treats audit-append failures as fire-and-forget (`warn!` and drop); the wrap should `tokio::spawn(async move { spawn_blocking(...).await })` so the WS handler's hot path doesn't block on the blocking-pool queue.
  - `auth.users.authenticate(...)` at `:373`.
  - The alarm-`ack` call (search the file for `engine.lock` + `.ack(`) and any `journal.write_transition(...)` / `journal.read_alarm(...)` reachable from a WS handler.
- **`crates/gateway/Cargo.toml`** — add `tokio-util = { version = "0.7", default-features = false, features = ["rt"] }` (the `rt` feature is the one that gates `CancellationToken`). Crate-local; do **not** promote to `workspace.dependencies`.
- **`crates/gateway/src/main.rs`** — introduce the top-level `CancellationToken`:
  - `let shutdown = CancellationToken::new();` near the start of `main()`.
  - `let mut tasks: JoinSet<()> = JoinSet::new();` for service-level spawns (backup HTTP, script-runtime watcher, alarm-runtime trampoline, anything else this file spawns long-lived).
  - Refactor every `tokio::spawn` in this file to `tasks.spawn(...)`. Each inner future selects on its existing work + `shutdown.clone().cancelled_owned()`.
  - The existing `tokio::select! { result = run_server, signal = ctrl_c }` becomes:
    1. `select!` exits.
    2. `info!("shutdown signal received");`
    3. `shutdown.cancel();`
    4. Drain: `let drained = tokio::time::timeout(SHUTDOWN_GRACE, async { while tasks.join_next().await.is_some() {} }).await;` with `const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);` at the module top.
    5. On timeout, `warn!("shutdown drain timed out after {:?}", SHUTDOWN_GRACE);` then `tasks.abort_all();` and a final brief loop to swallow the abort errors.
  - Drop the `pending::<()>` placeholder at `:217-224` and any twin in the `Some(project_store)` branch — both become normal `tasks.spawn` arms with a `select! { _ = shutdown.cancelled() => {}, _ = work => {} }` body, or remove the spawn entirely and call `engine.abort()` directly at drain time. Document the choice in code (one-line comment) and in the Codex log.
- **`crates/scripting/Cargo.toml`** — confirm `tokio-stream = { version = "0.1", features = ["sync"] }` is a dep (the `sync` feature gates `BroadcastStream`). Add it if missing; otherwise reuse.
- **`crates/scripting/src/host.rs`** — replace `recv_any` with a `StreamMap`-driven fan-in:
  - Where the supervisor builds `let mut receivers = ...` (line 344-348), build instead a `tokio_stream::StreamMap<String, tokio_stream::wrappers::BroadcastStream<TagSnapshot>>` keyed by tag path.
  - The third `select!` arm becomes:
    ```rust
    Some((path, item)) = stream_map.next() => {
        match item {
            Ok(snapshot) => { /* existing trigger-invoke logic */ }
            Err(BroadcastStreamRecvError::Lagged(n)) => {
                warn!(
                    script_id = %runtime.script_id,
                    %path,
                    skipped = n,
                    "script tag fan-in lagged"
                );
            }
        }
    }
    ```
  - Delete `recv_any` and its 10 ms `tokio::time::sleep`.
  - Preserve `biased` ordering: control kill/shutdown wins, worker exit second, fan-in third.

### Behavior

**SQLite wrapping** — every wrapped site looks like:

```rust
let historian = historian.clone(); // cheap; Arc inside
let result = tokio::task::spawn_blocking(move || {
    historian.read(&tag_path, t_start_ms, t_end_ms, aggregation, max_points)
})
.await;
let result = match result {
    Ok(inner) => inner, // anyhow::Result<Vec<HistoryPoint>>
    Err(join_err) => {
        tracing::error!(error = %join_err, "history.read task panicked");
        send_error(&out_tx, "history.unavailable", "internal error".into());
        continue;
    }
};
match result {
    Ok(points) => try_send_message(...),
    Err(err) => send_error(&out_tx, "history.read", err.to_string()),
}
```

For **fire-and-forget audit appends**, the wrapping is:

```rust
let audit_log = audit_log.clone();
tokio::spawn(async move {
    let join = tokio::task::spawn_blocking(move || {
        audit_log.append(user, session_id, peer, event)
    })
    .await;
    match join {
        Ok(Err(err)) => warn!(error = %err, "failed to append audit event"),
        Err(join_err) => warn!(error = %join_err, "audit append task panicked"),
        Ok(Ok(())) => {}
    }
});
```

This keeps the WS handler off the blocking-pool queue entirely.

**Shutdown** — every spawned task either:
- selects on its existing work future + `shutdown.cancelled()`, exiting cleanly when the latter fires; or
- is shaped as a stream loop that exits when its source closes — wrap the whole loop in `tokio::select! { _ = work => {}, _ = shutdown.cancelled() => {} }` at the spawn site.

`SHUTDOWN_GRACE = Duration::from_secs(5)`. Tasks that haven't returned by then are `abort_all()`-ed; this is logged at `warn!` level with the count of stragglers.

**`recv_any` replacement** — semantics that **must** be preserved:
- Control messages (`Kill { response }`, `Shutdown`) still take priority via `biased`.
- Worker subprocess exit (`worker.wait()`) still beats fan-in.
- Trigger errors still kill the worker and propagate to `runtime.events` as `ScriptEvent::Error`.
- Subscription closure (any single `BroadcastStream` ending) does **not** kill the supervisor — currently `recv_any` `bail!`s on closure of any one receiver, which is too aggressive. With `StreamMap`, a closed stream is removed from the map; if the map is empty, the supervisor stays alive (it has no work to do until a control message arrives, which is the correct behavior).

**New behavior**: broadcast lag is now visible at `warn!` level. Today the `Err(TryRecvError::Lagged(_)) => {}` arm at host.rs:398 silently drops; this is the observability improvement the reviewer (Codex) called out as required.

### Test requirements

- **Existing matrix green**: `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check` — three consecutive runs each.
- **New unit/integration tests**:
  - **Recorder non-blocking regression** (`crates/historian/tests/recorder_spawn_blocking.rs` or extend the existing recorder test). On a single-thread runtime (`#[tokio::test(flavor = "current_thread")]`), publish a flood of N updates while concurrently driving an unrelated `tokio::time::sleep(Duration::from_millis(0))` future; assert the unrelated future completes within a deadline (e.g., 200 ms). Pre-fix this test would hang or time out; post-fix it passes cleanly.
  - **Supervisor lag visibility** (`crates/scripting/tests/host_lag_visibility.rs`). Spin up a supervisor with a slow worker and a fast publisher; assert a `tracing::warn!` line containing `"lagged"` is emitted (use `tracing-test` or a custom layer). `crates/scripting/tests/host.rs` is the template.
  - **Gateway shutdown drain** (`crates/gateway/tests/shutdown_drain.rs`). Spawn the gateway with a project that activates script runtime + alarm runtime + historian recorder + backup HTTP; trigger shutdown via the cancellation token directly through a test-only entry point (do **not** rely on signal delivery — Windows doesn't have SIGINT); assert `main()` returns cleanly within `SHUTDOWN_GRACE + 1s`.
- **Manual smoke (record findings in the Codex log)**: after the change, run the gateway with a demo project + runtime-web client connected, generate write/ack load for 30 s, hit Ctrl-C, and confirm:
  - Process exits within 6 s.
  - No "panicked at..." in stderr.
  - SQLite integrity intact: `sqlite3 openwebhmi-history.sqlite "PRAGMA integrity_check;"` reports `ok`; same for alarms + audit.

### Acceptance criteria

- [ ] Every blocking SQLite call from an async context is `spawn_blocking`-wrapped (historian read+write_sample, audit append+query, alarm journal write_transition+read_alarm, user authenticate).
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] `gateway/src/main.rs` has a single top-level `CancellationToken`; every previously bare `tokio::spawn` is now via `JoinSet` and selects on cancellation.
- [ ] Ctrl-C drain works under load; gateway exits within `SHUTDOWN_GRACE + ~1s` (manual smoke recorded).
- [ ] `recv_any` is gone. Script-host supervisor uses `StreamMap`. Broadcast lag is logged at `warn!` with `script_id` and skip count.
- [ ] Three new tests added (recorder non-blocking, supervisor lag visibility, gateway shutdown drain). Existing tests still pass on three consecutive runs.
- [ ] `pending::<()>` placeholders at `gateway/src/main.rs:217-224` (and any twin) are removed.
- [ ] `apps/designer/README.md` smoke checklist gets one new step: "Ctrl-C the gateway under load; confirm clean exit within 6 s and SQLite integrity_check reports ok on all three DBs."

### Out of scope (explicit)

- **No `JoinSet` adoption inside drivers, the historian-recorder per-tag task map, or anywhere below the gateway main.rs service layer.** That's CODEX-AK. The internal recorder still uses its existing `HashMap<String, JoinHandle<()>>`.
- **No `#[must_use]` or `#[track_caller]` annotation sweep.** CODEX-AK.
- **No cancel-safety doc sweep across driver/supervisor `async fn`s.** CODEX-AK. (You will need to *think* about cancel-safety to write the wrappers and new `select!` arms — that's fine. Don't add doc comments to other modules.)
- **No `reserve()` / `Permit` adoption** in the gateway write queue or anywhere else. The earlier review listed this as a "consider" item but Codex (reviewing the review) downgraded it: no concrete loss-mode identified. Skip until one is.
- **No newtype introduction (`TagPath`, `DriverId`)**, no driver-config builder split, no `#[non_exhaustive]` annotations, no cheap-clone handle refactor of `ScriptHost`/`TagStore`/etc. CODEX-AL.
- **No `thiserror = "2"` bump, no `format!` capture-syntax sweep, no missing-docs lint consistency.** CODEX-AM.
- **No restructuring of `HistorianStore` / `AuditLog` / `AlarmJournal` / `UserStore` into actor-style services.** Wrappers at the call site only. (Considered: actor-style centralizes backpressure but introduces a new RPC protocol; out-of-proportion for this brief and would blur correctness with ergonomics.)
- **No changes to the `select!` arms in `recv_any`'s caller other than the third arm.** Control and worker arms remain as-is.
- **No `tokio-stream` introduction outside the script host.** If the gateway WS writer (currently a manual `out_rx.recv()` loop) tempts you, leave it.

### Risks / gotchas

- **`spawn_blocking` panics**. If a closure panics, `.await` returns `JoinError::Panic`. SQLite stores all use `?` and return `anyhow::Result`, so panics should be rare — but `expect("...")` exists in some places, and a corrupted DB still hits `?` cleanly. Convert `JoinError` to a wire-level error; do **not** propagate via `?` directly out of the WS handler (that would close the connection).
- **`spawn_blocking` is bounded** (default 512 threads). Audit-log append is the highest-frequency wrapped path (every WS write). The brief mandates the `tokio::spawn(spawn_blocking(...))` outer-spawn pattern for fire-and-forget appends precisely so the WS hot path doesn't queue on the blocking pool. **Do not** replace this with a synchronous `.await` — it would couple WS latency to blocking-pool depth.
- **`AlarmEngine::ack` is `&self` and runs sync work inline.** The current call site is `engine.lock().ack(...)` (server.rs:~532). Move the *whole* lock+ack into the blocking closure: clone the `Arc<std::sync::Mutex<AlarmEngine>>`, capture it in `spawn_blocking`, take the lock there, call `.ack(...)`. Keeps the std-Mutex off the runtime.
- **`tokio_util` is gateway-only.** Add it to `crates/gateway/Cargo.toml`, not workspace deps.
- **`tokio-stream::wrappers::BroadcastStream`** semantics: the stream item is `Result<T, BroadcastStreamRecvError>`. Channel closure ends the stream cleanly (no error item). Only `Lagged` is surfaced as an error. Don't add a closure-handler arm; handle map-empty by letting the `select!` bias keep the supervisor alive.
- **Cross-platform shutdown test.** SIGINT doesn't exist on Windows. Use a test-only helper that takes the `CancellationToken` and triggers it directly; do not invoke `tokio::signal::ctrl_c()` in tests.
- **`#[tokio::test(flavor = "current_thread")]`** is the right shape for the recorder non-blocking regression. On a multi-thread runtime, `spawn_blocking` always works because there are spare worker threads. Single-thread is what catches the regression.
- **Don't bump tokio-stream past 0.1 / tokio-util past 0.7.** Workspace deps are pinned; the brief is scope-locked to what already resolves.
- **Don't refactor `append_audit` into a method.** It's a free function for a reason (no receiver, OnceLock-backed); the wrap is one site.

## Codex log

<!-- Codex appends status transitions and notes here. -->

## Claude review

## Verdict
