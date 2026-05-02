---
id: CODEX-V
title: Route system.tag.write through the per-driver write queue
owner: codex
phase: 3
status: merged
created: 2026-04-28
last-update: 2026-04-28 claude
---

# CODEX-V — `system.tag.write` → per-driver write queue

## Brief

> **Phase 3 closeout follow-up.** CODEX-T shipped with a brief error in the original Claude-authored brief: it told Codex `system.tag.write` should call `TagStore::publish` directly. That's wrong for any tag backed by a driver — the write only updates the gateway's cached value, never reaches the PLC, and the next driver poll cycle overwrites it. The Phase 3 exit criterion ("a Python script that writes a derived setpoint based on two inputs") is not meaningfully met until this is fixed.

### Goal

Route `system.tag.write` through the same per-driver write mpsc that CODEX-N built for WS-side `tag.write`. Memory tags (paths whose driver-id prefix isn't in `DriverHandles`) fall through to `TagStore::publish` as before — that path is correct for derived/calculated tags that aren't backed by a driver.

After this lands, the Phase 1 demo's `derived-setpoint` script actually moves the simulator's Setpoint, and the manual smoke stops oscillating.

### Context to read first

- `crates/scripting/src/worker.rs:211-253` — current `handle_rpc`, `RpcMethod::TagWrite` calls `store.publish` directly (the line to fix).
- `crates/scripting/src/host.rs:101-118` — `ScriptHost::spawn` signature (needs the new sink param).
- `crates/gateway/src/project.rs:27-69` — `DriverHandles`, `DriverHandle::try_write`, `WriteCommand`, `WriteEnqueueError`. This is the existing write-queue API.
- `crates/gateway/src/server.rs:770-908` — how the WS-side `tag.write` handler routes through `DriverHandles`; mirror this logic.
- `crates/gateway/src/main.rs:72-118` — `spawn_script_runtime` is where the new sink gets injected.
- `crates/scripting/tests/host.rs` — five existing host tests that currently exercise the publish-only path; they need a fake sink that publishes back so the assertions still hold (or use memory-tag paths).

### Files to create / modify

- `crates/scripting/src/lib.rs` — re-export the new `TagWriteSink` trait and `TagWriteError` enum.
- `crates/scripting/src/sink.rs` — new module: `TagWriteSink` trait + `TagWriteError` enum + a `MemorySink` test impl that publishes to a `TagStore` (so existing tests can register it).
- `crates/scripting/src/host.rs` — `ScriptHost::spawn` accepts `Arc<dyn TagWriteSink>` (or generic). Plumb to workers.
- `crates/scripting/src/worker.rs` — `handle_rpc` `RpcMethod::TagWrite` calls the sink; on error returns the error string back over the RPC channel so the Python side raises a clean exception.
- `crates/scripting/tests/host.rs` — update existing tests to construct a sink (likely `MemorySink::new(store.clone())` so the assertions still work). Add one new test that asserts driver routing: register a sink that records writes, fire a script that calls `system.tag.write("rockwell-1/Setpoint", ...)`, assert the recorder saw the write.
- `crates/gateway/src/main.rs` — `spawn_script_runtime` builds a `GatewayTagWriteSink` from `DriverHandles + TagStore` and passes it into `ScriptHost::spawn`. Driver-prefixed paths go to `try_write`; otherwise fall through to `store.publish`.
- `crates/gateway/src/lib.rs` (or wherever appropriate) — `GatewayTagWriteSink` impl.

### `TagWriteSink` trait

```rust
pub trait TagWriteSink: Send + Sync + 'static {
    /// Enqueue a tag write. Returns synchronously after enqueue (or fallback publish);
    /// does NOT wait for the underlying PLC write to ack. Mirrors the WS-side semantics.
    fn enqueue(&self, path: &str, value: TagValue) -> Result<(), TagWriteError>;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TagWriteError {
    #[error("driver write queue full for '{0}'")]
    Busy(String),
    #[error("driver write channel closed for '{0}'")]
    Closed(String),
    #[error("unknown driver for path '{0}'")]
    UnknownDriver(String),
    #[error("{0}")]
    Other(String),
}
```

`UnknownDriver` is *not* an error case for memory tags — it's used when a path looks driver-prefixed (`<driver>/<address>`) but the driver-id isn't registered. Memory tags (no `/` in the path, or driver-id explicitly listed as a memory namespace) bypass the queue entirely and call `store.publish` inside the gateway sink impl.

### `GatewayTagWriteSink` semantics

```text
path           → action
─────────────────────────────────────────────────────────────────
"a/b"          → look up DriverHandles["a"]; if found:
                   try_write(WriteCommand { address: "b", value, ... })
                 if not found:
                   store.publish("a/b", value, Quality::Good)   ← memory tag
"a"            → store.publish("a", value, Quality::Good)       ← memory tag
"a/b/c"        → look up DriverHandles["a"]; address = "b/c"    ← multi-segment addresses
```

Use `path.split_once('/')` for the split; matches how `DriverHandle::try_write` already expects the address-without-prefix shape.

### Audit logging

The `info!(script_id, path, value, "script tag write")` line at `worker.rs:233-238` must still fire **before** the sink call (audit-on-attempt, not audit-on-success). On `TagWriteError::Busy/Closed/UnknownDriver`, additionally emit a `warn!` and propagate the error string back to Python so the `system.tag.write` call raises `RuntimeError` with the cause.

### Test requirements

- **Existing 5 host tests still pass.** Update them to inject a `MemorySink::new(store.clone())` so the publish path is still exercised.
- **New test** `tag_write_routes_to_driver_sink`: register a recording sink that captures `(path, value)` tuples on enqueue (and does NOT publish to TagStore). Fire a script that calls `system.tag.write("rockwell-1/Setpoint", 60.0)`. Assert the recorder saw exactly that path + value.
- **New test** `tag_write_busy_propagates_error_to_python`: sink that returns `Busy` for a specific path. Script catches the `RuntimeError` from `system.tag.write` and writes a sentinel to a memory tag. Assert sentinel is set (proves the script saw the exception).
- Memory-tag path: a script that writes to `mem/derived` (no driver registered) should still update TagStore via the fallback. Add a third small test or fold into the first.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-scripting` green (all existing tests + new ones).
- [ ] `cargo test --workspace --all-features --locked` green.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] Phase 1 demo manual smoke: with the simulator running and the runtime+designer connected, set Pressure > 100 in the simulator, observe Setpoint change to half-pressure, verify the value **persists** across at least 5 driver poll cycles (no oscillation back to the simulator's prior value).
- [ ] All public new items have rustdoc.

### Out of scope

- Async write-ack from the driver (script blocks until enqueue, not until PLC confirms). Acceptable for v1; matches WS-side semantics.
- Per-tag write rate limiting from scripts — Phase 4 polish.
- A `system.tag.write_async` variant that returns a future — post-1.0.
- Memory-tag namespacing convention (`mem/`, `system/`, etc.) — for now, "no driver registered for prefix" = memory tag. Document the v1 behavior.

### Risks / gotchas

- **Don't break the existing 5 host tests' assertions.** They use `wait_tag(&store, ...)` after the script writes. With the new routing, `store.publish` is no longer called by the script for driver-prefixed paths — only the recorder sink would see the write. Fix: have the test sink (`MemorySink`) call `store.publish` after recording, so it behaves like a driver that immediately reflects writes back into the cache. The crash/timeout/concurrent tests don't change semantics.
- **Path parsing must match the existing WS handler.** `split_once('/')` is what server.rs uses; mirror it exactly. Don't introduce a second parser.
- **Don't `await` inside `handle_rpc` for a long time.** `try_write` is sync and non-blocking — keep it that way. The script's RPC call already blocks in Python while waiting for the response; that's fine for enqueue-acknowledgement, but don't add an async `send().await` that could stall the worker reader loop.
- **`UnknownDriver` vs memory tag**: be explicit. `path.split_once('/').is_none()` → memory tag (publish). `Some((prefix, _)) if !DriverHandles.contains(prefix)` → also memory tag (publish, NOT UnknownDriver). `UnknownDriver` is reserved for a future case where we want to enforce explicit memory-tag namespaces. For v1, either route works correctly; document the choice.
- **Thread the audit log carefully.** It runs *before* the sink call so even rejected writes are audited. The warn-on-error fires *after* the sink returns Err.

## Codex log

*(codex — append working notes here)*

2026-04-28 18:50 MDT — codex — Status -> in-progress. Replacing the publish-only script write path with an injected `TagWriteSink`, then wiring gateway scripts to a sink that uses `DriverHandles::try_write` for driver-backed paths and `TagStore::publish` for memory tags.

2026-04-28 22:55 MDT — codex — Status -> submitted. Added `TagWriteSink`/`TagWriteError` and `MemorySink`, routed Python `system.tag.write` through the injected sink, and added `GatewayTagWriteSink` that uses `DriverHandles::try_write` for driver-backed paths with memory-tag fallback. Existing scripting host tests now inject `MemorySink`; added tests for recording driver-sink routing and Busy propagation to Python. Verified `cargo test -p openwebhmi-scripting`, `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `pnpm -r typecheck`, `pnpm -r test`. Manual smoke: sim-rockwell + gateway on `127.0.0.1:18080`, authenticated websocket subscribed to `rockwell-1/Pressure` and `rockwell-1/Setpoint`; last 5 Setpoint samples stayed >50 and tracked half-pressure across poll cycles, confirming no cache-only oscillation.

## Claude review

### 2026-04-28  claude — review pass 1

Spec-compliant. The brief was prescriptive and Codex hit every callout. The driver round-trip is now a real Phase 3 closeout, not a simulated one.

Strong points:
- ✅ **Trait + error enum + memory impl exactly as briefed.** `TagWriteSink` (`sink.rs:8-15`), `TagWriteError` with `Busy/Closed/UnknownDriver/Other` (`sink.rs:18-32`), `MemorySink` test impl that publishes back to TagStore (`sink.rs:36-52`).
- ✅ **Audit log fires before sink call** at `worker.rs:243-248` — `info!` with `script_id`, `path`, `value` runs unconditionally; `warn!` on rejection at `worker.rs:252-257`. Audit-on-attempt semantics preserved.
- ✅ **Error propagation to Python is real, not theoretical.** `worker.rs:258` returns `Err(err.to_string())`, the host writes `HostFrame::RpcError`, the Python `_bridge.py` raises `RuntimeError(error)`. The `tag_write_busy_propagates_error_to_python` test verifies end-to-end that a script's `try/except RuntimeError` catches the message string verbatim ("driver write queue full for 'rockwell-1/Setpoint'").
- ✅ **`GatewayTagWriteSink` mirrors the WS-side path** (`script_writes.rs:27-49`). `split_once('/')` matches what `server.rs` uses; multi-segment addresses handled correctly (`a/b/c` → driver=`a`, address=`b/c`); empty driver-id or empty address rejected by `split_tag_path` and routes to publish (defensive).
- ✅ **Memory-tag fallback works in two places**: (a) no `/` separator → publish, (b) prefix doesn't match a registered driver → publish. The unit test in `script_writes.rs` covers (b); the existing memory-tag scripts cover (a).
- ✅ **The new `tag_write_routes_to_driver_sink` test is the right shape.** It registers a `RecordingSink` that explicitly does NOT publish to TagStore, fires the script, then asserts both that the sink saw the write AND that `store.get("rockwell-1/Setpoint").is_none()`. That second assertion is the one that proves the routing is real — without it the test would pass even if writes were silently double-routed. Disciplined.
- ✅ **`MemorySink::new(store.clone())` injection in the existing 5 host tests** preserves their assertions without rewriting them. Crash, timeout, log, concurrent-RPC, and round-trip tests all retain their original semantics.
- ✅ **Live smoke validation** done by Codex with a real sim + gateway: Setpoint persisted across 5+ poll cycles, no oscillation. That's the closeout proof — the integration test passes by construction (no driver loop), but the live smoke is what shows the brief error from CODEX-T is actually fixed.
- ✅ **`spawn_script_runtime` plumbing in `main.rs:78-80, 96-122`** clones `DriverHandles` once for the sink and once for the WS server. Single source of truth for the driver write mpsc.
- ✅ **Acceptance**: 7 scripting tests pass (5 existing + 2 new); `cargo test --workspace`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo fmt --check`, `pnpm -r typecheck`, `pnpm -r test` all clean.

Findings:

- 🟢 **`UnknownDriver` enum variant is defined but never emitted by `GatewayTagWriteSink`.** The brief said v1 = "no driver registered → memory tag", and Codex implemented that. The variant remains for any future sink that wants to enforce explicit memory-tag namespaces. The enum being public commits us to it semantically — a v1.1 sink could start emitting it without a breaking change. Acceptable.
- 🟡 **`Closed` propagation lacks a dedicated test.** `Busy` is tested end-to-end through Python; `Closed` shares the exact same code path (single line in the match arm), so a regression would have to be deliberate to break only one. Tighten in a v1.1 polish pass.
- 🟡 **No unit test for the gateway sink's driver-routed branch.** The `script_writes.rs` test only covers memory-tag fallback. The driver-routed path is exercised by the live smoke and indirectly by the scripting `tag_write_routes_to_driver_sink` test (but that's a different sink impl). A small unit test that registers a fake `DriverHandle` with a recording mpsc receiver would lock in the gateway-specific routing. v1.1.
- 🟢 **Test sinks (`RecordingSink`, `BusySink`) live in `tests/host.rs`** rather than under `crates/scripting/src/` test helpers. Keeps the public API clean and the sink implementations self-contained per test. Right call.

Acceptance criteria — all five boxes verified, including the live manual smoke that this task was opened to fix.

## Verdict

**Merged at `9db307e`.** Phase 3 exit criterion now meaningfully met: the demo HMI's Python script writes a derived setpoint that actually reaches the simulator's PLC tag and persists across poll cycles. The brief error from CODEX-T is closed.

Two small v1.1 items added (Closed test, gateway-sink driver-branch unit test). The v1.1 backlog is now ~22 items; Phase 3 closes once CODEX-U lands.
