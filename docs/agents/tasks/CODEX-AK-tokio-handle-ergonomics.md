---
id: CODEX-AK
title: Tokio handle ergonomics — AbortHandle map sweep, cancel-safety docs, per-connection drain
owner: codex
phase: 4
status: merged
created: 2026-05-05
last-update: 2026-05-05 claude
---

# CODEX-AK — Tokio handle ergonomics (Tier 2 from review pass 2)

## Brief

> Three follow-up items from the second-pass Rust 1.95 / edition 2024 review that complete the cancellation discipline AJ established. All three are **mechanical or documentation-only** changes that don't blur correctness with surface cleanup: (1) every `JoinHandle<()>` stored solely to call `.abort()` later becomes an `AbortHandle` — the same shape tokio uses internally, half the size, makes intent visible at the type level; (2) every `async fn` reachable from a `tokio::select!` arm gets a one-line cancel-safety doc comment so future contributors don't have to re-derive it; (3) the per-connection WebSocket handler drain that AJ's verdict flagged as the genuine v1.1 hardening item — extending the `JoinSet` + `CancellationToken` pattern AJ established at the gateway service layer down one level into the per-connection layer. **Scope-locked.** No `#[must_use]` sweep, no `#[track_caller]` sweep, no `pin!`-over-`Box::pin` sweep (audit found no candidates: every `Box::pin` in the workspace is a `BoxStream` return constructor, not a select-arm short-lived future). Tier 3 (newtypes / builders / `#[non_exhaustive]` / cheap-clone handles) and Tier 4 (`thiserror = "2"` / format-capture / missing-docs) follow as CODEX-AL / -AM.

### Goal

After this lands:

- Every `JoinHandle<()>` storage site whose only consumer is `.abort()` becomes `AbortHandle`. The eight call sites are listed below. Behavior is unchanged; the type signature gets honest about intent.
- Every `tokio::select!` arm in the workspace can be read in 30 seconds: each `async fn` reachable from a select arm has a one-line cancel-safety annotation in its doc comment. Future contributors don't re-litigate cancel-safety on every PR.
- `gateway/src/server.rs`'s connection-accepting loops register per-connection handlers in a `JoinSet` and pass the top-level `CancellationToken` into the handler. On shutdown, in-flight WebSocket connections receive a graceful close (drain window) before the gateway returns. The shutdown-drain test from AJ extends to cover the multi-task project-fixture variant.

### Context to read first

- **CODEX-AJ verdict** (`docs/agents/tasks/CODEX-AJ-async-hygiene.md`'s `## Verdict` section). The two flagged v1.1 follow-ups (per-connection drain + multi-task fixture for the shutdown test) are scoped into items #3 here.
- **`wiki/architecture/async-runtime-hygiene.md`** "Open questions" section — formal record of what AK is closing.
- **AbortHandle map sweep — eight sites:**
  - `crates/alarm-engine/src/engine.rs:21` — `AlarmEngineHandle::handles: HashMap<String, JoinHandle<()>>`. Only `.abort()` is ever called; `update_definitions` and `abort` both consume by `.abort()`.
  - `crates/historian/src/recorder.rs:27` — `RecorderHandle::handles: HashMap<String, JoinHandle<()>>`. Same shape; `update_configs` and `abort` consume by `.abort()`.
  - `crates/gateway/src/server.rs:317-321` — five per-connection subscription maps: `subscriptions`, `project_subscriptions`, `alarm_subscriptions`, `script_subscriptions`, `audit_subscriptions`. Each is `HashMap<String, JoinHandle<()>>`; the consumer is always `.abort()` on unsubscribe.
  - `crates/driver-api/src/supervisor.rs:51` — `SupervisorTask::task: JoinHandle<()>`. Only consumer is `.abort()` in `Drop`. Single field, not a map; same conversion to `AbortHandle`.
- **Cancel-safety annotation — eight `tokio::select!` sites** (audit each `async fn` reachable from any arm):
  1. `scripting/src/host.rs:364` — control.recv / worker.wait / snapshots.next (post-AJ form). The two helpers `worker.wait()` and the StreamMap iterator are the candidates here.
  2. `driver-ads/examples/hardware-smoke.rs:248` — example-only; **annotate but don't gate scope on it.**
  3. `gateway/src/main.rs:153` — run_server / ctrl_c / shutdown.cancelled. `run_server`, `tokio::signal::ctrl_c`, `CancellationToken::cancelled` — all already-documented by their authors. Cite their guarantees in our `run_gateway` doc comment.
  4. `gateway/src/main.rs:178` — `spawn_cancellable` helper. The two arms are `shutdown.cancelled()` (cancel-safe; tokio_util documents) and the user-provided `work: F`. Document that the contract is "the work future MUST be cancel-safe at every yield point or accept that drop-mid-await is a no-op for the work it represents."
  5. `gateway/src/main.rs:245, 317, 384` — `spawn_script_runtime` / `spawn_alarm_runtime` / `spawn_history_recorder` watchers. Each selects on `shutdown.cancelled()` + `changes.recv()`. Both arms are tokio-cancel-safe; annotate accordingly.
  6. `gateway/src/project.rs:240` — driver subscription routing. Selects on `stream.next()` (driver subscribe stream) + `write_rx.recv()` (mpsc) + driver write futures. The driver-side `subscribe()` return-stream cancel-safety is not currently documented in `driver-api`'s `Driver` trait — that's the highest-value annotation in this sweep.
- **Per-connection drain (item #3):**
  - `crates/gateway/src/server.rs` — `serve_with_project_store_driver_handles_and_auth` (non-TLS path) and the TLS path inside `crates/gateway/src/main.rs:421-499` (`run_server`). Both are accept-loops that spawn per-connection handlers without a top-level `JoinSet` registration and without a `shutdown.cancelled()` arm in the handler.
  - `crates/gateway/src/main.rs:435-468` — the TLS accept loop's per-connection `connections.spawn(...)` is a *local* `JoinSet`, not the service-level one. Decide: lift it to the service `JoinSet` or document why the local one is correct.
  - `apps/designer/README.md:46` — the manual-smoke step from AJ ("Ctrl-C the gateway under load; confirm clean exit within 6 s") becomes the verification surface for item #3.

### Files to modify

**Item 1 — AbortHandle map sweep:**

- `crates/alarm-engine/src/engine.rs` — change `handles: HashMap<String, JoinHandle<()>>` to `handles: HashMap<String, AbortHandle>`. `spawn_path` (currently `-> JoinHandle<()>`) returns `AbortHandle`: at the spawn site, write `let handle = tokio::spawn(...); handle.abort_handle()`. The `JoinHandle` is dropped at the end of the statement; the task continues running and is cancellable via the abort handle. `update_definitions` and `abort` both already use `.abort()`, no further changes.
- `crates/historian/src/recorder.rs` — identical shape. `spawn_one` returns `AbortHandle`. `RecorderHandle::handles` is `HashMap<String, AbortHandle>`. `update_configs` and `abort` unchanged in body.
- `crates/gateway/src/server.rs:317-321` — five subscription maps converted. Look for the call sites that store/remove from these maps (a few hundred lines below) and adjust the spawn pattern: `let handle = tokio::spawn(...); map.insert(key, handle.abort_handle());`. The `.abort()` call at unsubscribe time stays as-is.
- `crates/driver-api/src/supervisor.rs:51` — `task: JoinHandle<()>` becomes `task: AbortHandle`. The `Drop` impl's `.abort()` works on `AbortHandle` identically.

Add `use tokio::task::AbortHandle;` where needed.

**Item 2 — Cancel-safety doc annotations:**

For each `async fn` reachable from a `tokio::select!` arm in the workspace, add a single-line annotation to its doc comment. The format is:

```rust
/// Cancel-safe: dropping the returned future before completion discards no work.
```

or

```rust
/// NOT cancel-safe: dropping the returned future mid-await may lose <specific resource>.
/// Callers using this in `tokio::select!` should reserve the resource via <pattern>.
```

**Specific surfaces to annotate (the inventory):**

- `crates/driver-api/src/trait_def.rs` — `Driver::subscribe` return type is `BoxStream<'static, DriverUpdate>`. The trait docs should state the cancel-safety contract for `Stream::next()` on the returned stream — i.e., the implementer's contract: "Implementations MUST produce a stream whose `Stream::next` is cancel-safe; dropping the resulting future before it resolves MUST NOT lose a tag update." This is a contract addition, not a behavior change.
- `crates/scripting/src/host.rs` — `WorkerProc::wait` (the `worker.wait()` arm at line 379). Annotate cancel-safety for the `select!` consumer.
- `crates/scripting/src/host.rs` — `WorkerProc::invoke_tag_change` is *not* in a select arm directly (it's awaited inside the snapshots-arm body), but is the "if cancellation lands here" surface. Annotate cancel-safety regardless: dropping mid-await loses the in-flight script invocation.
- `crates/gateway/src/project.rs` — the per-driver routing function (around `:240`). Annotate the function's outer doc with a cancel-safety statement covering `tokio::select!` semantics (which arm wins, what happens to in-flight writes).
- `crates/gateway/src/main.rs` — `run_gateway`, `spawn_cancellable`, `drain_tasks`. One-line annotations.
- Trait re-exports: where we re-export tokio types that have documented cancel-safety, our re-export docs should cite the upstream guarantee rather than re-deriving it.

**Do not** change behavior to *make* a function cancel-safe. If you find a `async fn` reachable from a select arm that is genuinely *not* cancel-safe and where dropping mid-await would lose data, **document the hazard, log a `// TODO(v1.1)` next to the doc comment, and report the function in the Codex log.** Do not "fix" it — that's an architectural change that needs its own brief.

**Item 3 — Per-connection WebSocket handler drain:**

- `crates/gateway/src/main.rs:421-499` (`run_server`) — both TLS and non-TLS paths spawn per-connection handlers. Lift these spawns to the service-level `JoinSet` *or* keep a local `JoinSet` and have it await on the same `CancellationToken`. The cleaner shape: pass `shutdown: CancellationToken` and `tasks: &mut JoinSet<()>` from `run_gateway` into `run_server`; per-connection handlers register with `tasks` and select on `shutdown.cancelled()` to send a WebSocket close-frame and return.
- `crates/gateway/src/server.rs` — `handle_connection` and `serve_with_project_store_driver_handles_and_auth` (non-TLS). The per-connection handler's main loop is the `while let Some(item) = incoming.next().await` at server.rs:323. Wrap that loop in a `tokio::select! { _ = shutdown.cancelled() => { /* close frame + return */ }, _ = main_loop => {} }`.
- WebSocket close-frame conventions: send a `Close { code: 1001 (Going Away), reason: "gateway shutting down" }` frame before dropping the connection. tokio-tungstenite's `WebSocketStream::close(...)` is the API.
- The `SHUTDOWN_GRACE = 5s` constant from `main.rs:25` is the umbrella drain budget — per-connection close frames must be sent within ~1s so the rest of the budget covers the broadcast HTTP and other tasks.

**Tests:**

- `crates/gateway/tests/shutdown_drain.rs` (or whichever existing file holds the AJ test) — extend with a **multi-task project-fixture variant** that activates script + alarm + historian + backup HTTP. Keep the cross-platform shape (token cancel, not SIGINT). Same `SHUTDOWN_GRACE + 1s` deadline.
- New test: `gateway/tests/shutdown_drain_per_connection.rs` (or extension of the existing one) — opens a real WebSocket connection to a running gateway test fixture, cancels the token, asserts the connection receives a close frame within ~1s and the gateway returns within `SHUTDOWN_GRACE + 1s`. Use the existing test-only `run_gateway` entry point.

### Behavior

**AbortHandle map sweep** — pure type-level change. Functionally:

```rust
// before
let handle: JoinHandle<()> = tokio::spawn(async move { /* work */ });
handles.insert(key, handle);
// later: handles.remove(&key).map(|h| h.abort());

// after
let handle = tokio::spawn(async move { /* work */ });
handles.insert(key, handle.abort_handle());
// later: handles.remove(&key).map(|h| h.abort());
```

The task continues running after `JoinHandle` is dropped; `AbortHandle` is the cancel-only view. Saves a small amount of memory per task; signals intent. No behavior change.

**Cancel-safety annotation pattern** — reuse the wording tokio uses in `tokio/src/sync/mpsc/bounded.rs:330-336` (which I cited in the original review pass 2). The doc comment is the contract; the test for it is the existing per-call-site usage.

**Per-connection drain semantics:**

1. Token cancels (via `ctrl_c` or test entry).
2. `run_gateway` calls `shutdown.cancel()` after the outer `select!` exits.
3. Service-level tasks (script watcher, alarm watcher, historian recorder, backup HTTP) — each selects on `shutdown.cancelled()` and exits cleanly. *(Already done by AJ.)*
4. **New:** per-connection handlers each select on `shutdown.cancelled()`. On signal, send a WebSocket close frame (1001, "gateway shutting down"), drop the connection, exit the handler.
5. `drain_tasks` awaits the JoinSet drain with `SHUTDOWN_GRACE` timeout.
6. Stragglers (handlers that ignored the cancel signal) get `abort_all`'d after the timeout.

### Test requirements

- **Existing matrix green**: `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check` — three consecutive runs each.
- **AJ shutdown-drain test extended** with a project-fixture variant that loads a real example project (e.g., `examples/sim-rockwell` if it has the required artifacts; otherwise build a minimal fixture in `crates/gateway/tests/fixtures/`). Asserts: gateway runs for ~50 ms with all four service tasks active, token cancel triggers drain, gateway returns within `SHUTDOWN_GRACE + 1s`.
- **Per-connection drain test:** opens a WebSocket to a running test gateway, sends one valid message (e.g., authentication), then while the connection is open, cancels the token. Asserts: client receives a close frame with code 1001 within 1 s, no panic, gateway returns within `SHUTDOWN_GRACE + 1s`.
- **Cancel-safety annotation acceptance** is **doc compilation only**: `cargo doc --workspace --no-deps` must succeed (the annotations don't introduce any rustdoc warnings).
- **`pnpm -r typecheck` + `pnpm -r test`** green (regression sanity; should be unaffected).
- **Manual smoke (record findings in the Codex log):** the `apps/designer/README.md:46` step from AJ, repeated with an open browser tab connected to the runtime — Ctrl-C the gateway, confirm the browser shows a clean disconnect within ~1 s and the gateway exits within 6 s.

### Acceptance criteria

- [ ] Every `JoinHandle<()>` storage site listed above is now `AbortHandle`. `cargo build --workspace` succeeds with no functional changes.
- [ ] No new clippy warnings under `-D warnings`.
- [ ] Every `async fn` reachable from a `tokio::select!` arm in the workspace has a single-line cancel-safety annotation in its doc comment. The eight inventoried `select!` sites are all documented; for each arm, the annotation matches the underlying primitive's documented behavior (cancel-safe / not).
- [ ] Per-connection WebSocket handlers receive a `shutdown.cancelled()` signal and emit a 1001 close frame within ~1 s. In-flight connections drain cleanly without `abort_all`.
- [ ] AJ shutdown-drain test extended with the multi-task project-fixture variant.
- [ ] New per-connection drain test passes on three consecutive runs.
- [ ] `cargo doc --workspace --no-deps` succeeds with zero new rustdoc warnings.
- [ ] `apps/designer/README.md:46` smoke step verified manually with an active browser session.

### Out of scope (explicit)

- **No `#[must_use]` sweep.** Codebase audit found few candidates with real return-ignored hazards; deferred to AL or skipped entirely. Don't add `#[must_use]` opportunistically — it generates noise without value.
- **No `#[track_caller]` sweep.** Same: the sync-bridge / blocking-helper surfaces tokio uses these for don't have many analogs in our codebase.
- **No `pin!`-over-`Box::pin` sweep.** Audit confirmed: all 12 `Box::pin` sites in the workspace construct `BoxStream<'static, ...>` for `Driver::subscribe`'s trait return type. None are short-lived select-arm futures. **Non-issue.** Don't change them.
- **No newtype introduction**, no driver-config builder split, no `#[non_exhaustive]` annotations, no cheap-clone handle refactor. CODEX-AL.
- **No `thiserror = "2"` bump, no `format!` capture-syntax sweep, no missing-docs lint consistency.** CODEX-AM.
- **No restructuring of `async fn`s to make them cancel-safe.** Item #2 is documentation-only. If an `async fn` is genuinely cancel-unsafe and reachable from a select arm, document the hazard with a `// TODO(v1.1)` and report it; don't fix it inline (that's its own brief).
- **No JoinSet conversion of internal handle maps where per-key abort is the access pattern.** `HashMap<String, AbortHandle>` is the cleaner shape than `JoinSet` for that access pattern; tokio's own `JoinSet` is for collective awaiting/aborting, not per-key keyed cancel.
- **No changes to driver `subscribe` implementations.** Item #2 adds a *contract* in the trait docs; existing impls must already conform (their `BoxStream` returns are built on tokio primitives that are cancel-safe). Don't audit each driver's stream construction.

### Risks / gotchas

- **`AbortHandle` requires `tokio::task::AbortHandle` import.** Trivial. Confirm it's `pub use`'d through `tokio::task::AbortHandle` and not gated behind a feature.
- **Dropping `JoinHandle` immediately is correct but easy to miss.** The pattern `handles.insert(key, tokio::spawn(...).abort_handle())` works because the `JoinHandle` returned by `tokio::spawn` is dropped at end-of-statement; the task lives on. Document this in a one-line code comment at the first AbortHandle map insertion site (the others can reference the first).
- **Cancel-safety annotation discipline.** Only document what's actually true. Don't claim cancel-safety based on intuition; check the underlying primitive's docs (tokio's mpsc::Receiver::recv is cancel-safe; tokio::time::sleep is cancel-safe; broadcast::Receiver::recv is cancel-safe; mpsc::Sender::send is **not** cancel-safe). The five-minute audit per call site is the discipline; don't shortcut it.
- **Per-connection drain WebSocket close frame timing.** tokio-tungstenite's `close()` returns a future that completes when the close handshake finishes (or the connection drops). Allow up to 1 s for the handshake; anything longer indicates a buggy client and should fall through to the umbrella `SHUTDOWN_GRACE` drain.
- **Test fixture for project-loaded shutdown.** Building a minimal project fixture in `crates/gateway/tests/fixtures/` is preferable to depending on `examples/sim-rockwell`'s artifact layout (which can change). The fixture only needs: one tag, one alarm, one historian config, and one script (the script can be a no-op stub).
- **`run_server` signature changes.** Lifting the per-connection JoinSet to the service level changes `run_server`'s signature (it now needs `shutdown` and `tasks` parameters). This ripples through `run_gateway`. Keep the change minimal — pass references, don't restructure the call graph beyond what item #3 requires.
- **Don't propagate cancel-safety annotations into TS-side or Python-side docs.** This brief is Rust-only; the protocol-ts package and Python script worker don't have an analog to tokio's cancel-safety contract.

## Codex log

<!-- Codex appends status transitions and notes here. -->

2026-05-05 codex Status -> submitted. Converted the listed cancel-only task storage to `AbortHandle`: alarm engine path tasks, historian recorder tasks, gateway per-connection subscription forwarders, audit subscription forwarder, and the driver supervisor abort-on-drop handle. Extended gateway shutdown into per-connection WebSocket handlers by passing the service `CancellationToken` and service `JoinSet` through `run_server`; handlers now send close code 1001 (`gateway shutting down`) on token cancellation. Added cancel-safety docs for the inventoried select surfaces and documented `WorkerProc::invoke_tag_change` as NOT cancel-safe with a v1.1 TODO. Added project-fixture shutdown test and WebSocket close-frame shutdown test. Validation green: `cargo fmt --all --check` three runs, `cargo clippy --workspace --all-targets --all-features -- -D warnings` three runs, `cargo test --workspace --all-features --locked` three consecutive runs, `cargo doc --workspace --no-deps`, `pnpm -r typecheck`, and `pnpm -r test`. Manual browser/SQLite integrity smoke not run in this environment.

## Claude review

### Strong points

- ✅ **AbortHandle conversion at all 8 sites is uniform and minimal.** `AlarmEngineHandle::handles`, `RecorderHandle::handles`, the five per-connection subscription maps in `gateway/src/server.rs:401-405`, and `SupervisorHandle::task` all switched cleanly. The pattern `tokio::spawn(...).abort_handle()` is used consistently; no behavior change in the normal path.
- ✅ **Pattern documented at the first insertion site.** `crates/alarm-engine/src/engine.rs:175` carries the comment `// Dropping a JoinHandle does not cancel its task; AbortHandle is the cancel-only handle.` — exactly as the brief asked. Future contributors discover the constraint at the relevant location, not via archaeology.
- ✅ **`Driver::subscribe` cancel-safety contract added** (`crates/driver-api/src/trait_def.rs:65-68`). This was the brief's "highest-value annotation in this sweep" and Codex landed it precisely: "implementations must return a stream whose `Stream::next` future can be dropped before completion without losing a tag update." Trait-level contract; binding on every driver impl.
- ✅ **Cancel-safety annotations on every reachable `async fn`.** `run_gateway`, `spawn_cancellable`, `drain_tasks`, `spawn_script_runtime`, `spawn_alarm_runtime`, `spawn_history_recorder`, `run_server`, `run_subscription_until_disconnect`, `run_ready_worker`, `WorkerProc::wait`, `serve_with_project_store_driver_handles_auth_and_shutdown`, `handle_connection`, plus the inline annotation on the example's select. Wording is consistent and cites the underlying primitive's behavior.
- ✅ **`WorkerProc::invoke_tag_change` correctly identified as NOT cancel-safe** with both the rationale ("dropping mid-await may leave an in-flight script invocation running until the worker reports completion or exits") and a `// TODO(v1.1): add explicit trigger cancellation if scripts gain long-lived handlers` marker (`crates/scripting/src/worker.rs:174-177`). Exactly matches the brief's "do NOT fix inline; document the hazard" requirement.
- ✅ **Per-connection drain implementation is solid**:
  - Writer task selects on `writer_shutdown.cancelled()` vs `out_rx.recv()` with `biased` ordering (shutdown wins).
  - On shutdown, sends `CloseFrame { code: CloseCode::Away (1001), reason: "gateway shutting down" }` with a 1-second `tokio::time::timeout` around the `sink.send` so a stuck handshake can't block drain (server.rs:347-359).
  - Main handler loop selects on `shutdown.cancelled()` vs `incoming.next()` (server.rs:413-419) with a `shutdown_requested` flag for the cleanup path.
  - End-of-handler logic gives the writer a 1-second drain window when `shutdown_requested` so the close frame has time to deliver before `writer.abort()` (server.rs:1343-1352). When the connection ends naturally (no shutdown), writer is aborted immediately as before.
- ✅ **`ServerRunConfig` struct introduced** (main.rs:187-194) to keep `run_server`'s signature manageable after adding `shutdown` + `tasks` parameters. Cleaner than 8-positional-args; clean refactor.
- ✅ **Clean back-compat**: existing `serve_with_project_store_driver_handles_and_auth` API preserved (now delegates to the shutdown-aware variant via a private `serve_inner_with_shutdown` helper); new `serve_with_project_store_driver_handles_auth_and_shutdown` exposes the shutdown-aware path. No breakage for crates that depend on the old signature.
- ✅ **TLS path uses the service-level JoinSet.** Previously the TLS accept loop owned a local `connections: JoinSet<()>`; now it `tasks.spawn(...)` into the shared service JoinSet (main.rs:481-501). Both per-connection drain paths (TLS and non-TLS) register with the same JoinSet; symmetric drain behavior.
- ✅ **Wiki page updated comprehensively** (`wiki/architecture/async-runtime-hygiene.md`). Two new "Current understanding" entries: per-connection drain (#7) and AbortHandle pattern (#8). The per-connection drain "Open question" from AJ is removed (resolved). Evidence section adds the new test names and the `cargo doc` + `pnpm` validations.
- ✅ **Two new tests added**: `shutdown_token_drains_project_gateway_tasks` covers the multi-task project-fixture variant the brief asked for, and `shutdown_token_sends_websocket_close_frame` verifies the 1001 close-frame contract. Both pass on Codex's three consecutive runs.

### Bonuses beyond brief

- ✅ **`SupervisorHandle::Drop` impl added** (`crates/driver-api/src/supervisor.rs:202-208`). The brief asked for a type-only conversion (`task: JoinHandle<()>` → `task: AbortHandle`). Codex made it `task: Option<AbortHandle>` so `shutdown(mut self)` can `take()` the handle in the normal path, AND added a `Drop` impl that aborts when the handle is dropped without explicit shutdown. **This is a real behavioral improvement**: previously, dropping a `SupervisorHandle` without calling `shutdown()` left the supervised task running silently (since `JoinHandle::drop` doesn't abort). Now the task is properly aborted on drop. **Pattern noted**: this is the second time this sprint Codex has expanded scope on a `JoinHandle → AbortHandle` conversion (first was extending `AlarmRuntime::evaluate_snapshot/apply/transition` to async in AJ); both expansions were net-positive.
- ✅ **Writer task drains gracefully on shutdown.** Before this brief, the writer was *always* `.abort()`-ed at handler-end, even though by that point the close frame might still be in flight. Codex's split — `tokio::time::timeout(1s, &mut writer)` first, then `abort()` only if it didn't drain — gives the close frame its handshake window without compromising the umbrella `SHUTDOWN_GRACE` budget.

### Findings

- 🟡 **`SupervisorHandle::shutdown(mut self)` no longer awaits the task.** Original behavior: `let _ = self.task.await;` waited for the task to complete after sending Shutdown. New behavior: `let _ = self.task.take();` drops the AbortHandle without awaiting. The mpsc reply (`let _ = rx.await;`) provides ordering — the task acknowledges Shutdown before the reply lands — so behavior is approximately preserved. **But there's a tiny window** where the supervisor task might be doing post-reply cleanup when the AbortHandle is dropped. Today the task's normal exit has no post-reply cleanup, so this is fine; if post-reply cleanup is ever added, the await-vs-take distinction becomes load-bearing. v1.1 polish: change `shutdown` to `await` the task to completion via a separate oneshot or by holding a `JoinHandle`.
- 🟡 **No regression test for the `Driver::subscribe` cancel-safety contract.** The trait doc asserts implementations must guarantee drop-mid-await safety, but no test exercises this for any of the five driver impls. Their `BoxStream` constructors are built on tokio primitives (broadcast::Receiver, mpsc::Receiver) that do guarantee this in practice, but a regression test that drops the stream future mid-await and asserts no update is lost would harden the contract. Polish item.
- 🟡 **Multi-task project-fixture test does not open an actual WebSocket connection.** `shutdown_token_drains_project_gateway_tasks` covers service-level drain with all four service tasks active; `shutdown_token_sends_websocket_close_frame` covers per-connection drain. The integration of the two (multi-task + open WS connection draining together under the same token cancel) is implicitly covered but not directly asserted. Acceptable test decomposition; flagging for transparency.
- 🟡 **`scripting/src/worker.rs` not in the brief's "Files to modify" list.** Brief listed only the five select! sites and the trait doc. Codex correctly traced reachability — `WorkerProc::wait` is the `worker.wait()` arm at host.rs:379, and `WorkerProc::invoke_tag_change` is reachable from within the `snapshots.next()` arm body. Annotations are correctly placed. Not a finding, just a sign of careful audit; I'll mention it because the brief omitted worker.rs.
- 🟡 **`while let Some(item) = incoming.next().await` rewritten** to a `loop { let item = tokio::select! { ... }; let Some(item) = item else { break; }; }` (server.rs:407-419). Necessary to add the shutdown arm. Codex correctly handles both shutdown-requested and stream-end paths via the `shutdown_requested` flag. Real behavioral change but correctly implemented.
- 🟡 **Manual Ctrl-C + `PRAGMA integrity_check` smoke deferred** (same as AJ). Codex correctly didn't fake it. Maintainer-action follow-up.

### Independent verification

- `cargo fmt --all --check` — ✅ clean.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — ✅ clean.
- `cargo test --workspace --all-features --locked` — ✅ green (single workspace run on my side; Codex documents three consecutive runs).
- `cargo doc --workspace --no-deps` — ✅ green; no new rustdoc warnings from the cancel-safety annotations.
- Read every changed Rust source file (gateway/main.rs, gateway/server.rs, driver-api/trait_def.rs, driver-api/supervisor.rs, scripting/host.rs, scripting/worker.rs, alarm-engine/engine.rs, historian/recorder.rs, gateway/project.rs, driver-ads/examples/hardware-smoke.rs) plus the wiki update; cross-checked against the brief's call-site list.

### Acceptance-criteria tally

- [x] Every `JoinHandle<()>` storage site listed is now `AbortHandle`. Eight sites converted.
- [x] No new clippy warnings under `-D warnings`.
- [x] Every `async fn` reachable from a `tokio::select!` arm has a single-line cancel-safety annotation. Eight `select!` sites all annotated; trait-level `Driver::subscribe` contract added.
- [x] Per-connection WebSocket handlers receive a `shutdown.cancelled()` signal and emit a 1001 close frame within ~1 s.
- [x] AJ shutdown-drain test extended with the multi-task project-fixture variant (`shutdown_token_drains_project_gateway_tasks`).
- [x] New per-connection drain test passes on three consecutive runs (`shutdown_token_sends_websocket_close_frame`).
- [x] `cargo doc --workspace --no-deps` succeeds with zero new rustdoc warnings.
- [~] `apps/designer/README.md:46` smoke step verified manually with active browser session — **deferred** to maintainer hardware/runtime session, same as AJ.

## Verdict

**Merged.** AbortHandle conversion at all 8 sites; cancel-safety annotations on every async fn reachable from a `tokio::select!` arm including the `Driver::subscribe` trait-level contract; per-connection WebSocket handler drain with 1001 close frame and 1-second handshake window inside the umbrella `SHUTDOWN_GRACE` budget. The `WorkerProc::invoke_tag_change` cancel-unsafe surface is documented with a `TODO(v1.1)` marker rather than fixed inline, exactly per the brief.

Codex extended scope on one point — adding `Drop` to `SupervisorHandle` so an unforgotten handle aborts its task instead of leaking — and was right to. The brief asked for a type conversion; the underlying behavior bug (drop-without-shutdown leaks the task) was inherited from the previous `JoinHandle` version and was worth fixing. **Pattern noted**: third sprint occurrence of Codex catching an in-scope improvement the brief missed (after AJ's alarm-engine async refactor and AD's TLS fourth-option).

Two genuine v1.1 follow-ups surface: (1) `SupervisorHandle::shutdown` no longer awaits the task to drain — the mpsc reply provides ordering, but post-reply cleanup would be racy if added; (2) no regression test exercises the `Driver::subscribe` cancel-safety contract directly. Neither blocks the merge; both are tracked under the broader v1.1 polish list rather than as their own briefs. The wiki page is now the canonical reference for the workspace's async hygiene posture.
