---
id: CODEX-V
title: Route system.tag.write through the per-driver write queue
owner: codex
phase: 3
status: open
created: 2026-04-28
last-update: 2026-04-28 claude
---

# CODEX-V — `system.tag.write` → per-driver write queue

## Brief

> **Phase 3 closeout follow-up.** CODEX-T shipped with a brief error I authored: the brief told Codex `system.tag.write` should call `TagStore::publish` directly. That's wrong for any tag backed by a driver — the write only updates the gateway's cached value, never reaches the PLC, and the next driver poll cycle overwrites it. The Phase 3 exit criterion ("a Python script that writes a derived setpoint based on two inputs") is not meaningfully met until this is fixed.

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

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
