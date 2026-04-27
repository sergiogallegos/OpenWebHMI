---
id: CODEX-N
title: tag.write end-to-end — protocol message + gateway routing to driver
owner: codex
phase: 2
status: merged
created: 2026-04-27
last-update: 2026-04-27 claude
---

# CODEX-N — `tag.write` end-to-end

## Brief

> **Why this exists**: CODEX-L's brief incorrectly said Phase 0 already supported `tag.write`. It didn't — Phase 0 shipped only `tag.subscribe`, `tag.unsubscribe`, and `ping`. The runtime now emits `tag.write` frames via `GatewayClient.writeTag` (CODEX-L), but the gateway returns a `protocol.parse` error and the write is silently lost. This task closes the gap.

### Goal

Add the `tag.write` wire message and gateway-side routing so an operator's NumericInput / Button write reaches the driver and ultimately the PLC. Small focused task. Phase 2 can ship without this strictly, but the designer's NumericInput preview is meaningless without it — so this should land before CODEX-M's manual smoke.

### Context to read first

- `apps/runtime-web/src/gatewayClient.ts` — already has `writeTag(path, value)` from CODEX-L; emits the wire frame.
- `crates/protocol/src/lib.rs` — needs `ClientMessage::TagWrite` added. **Mirror the addition into `packages/protocol-ts`.**
- `crates/gateway/src/server.rs` — needs the new `ClientMessage` arm in the per-connection handler.
- `crates/gateway/src/project.rs` — `run_driver` is the per-driver task that owns the `RockwellDriver` instance; you need a way to send write commands into it.
- `docs/architecture.md` §5.2 — the "Operator write (screen → PLC)" data flow this task implements.
- `crates/driver-api/src/supervisor.rs` — `SupervisorHandle` already exposes `write()`. The CODEX-H wire-up bypasses the supervisor (yellow note on CODEX-H's task page); this task should *not* fix that — keep it scoped.

### Files to modify

- `crates/protocol/src/lib.rs` — add `ClientMessage::TagWrite { path: String, value: TagValue }`. Add a literal-wire-form round-trip test.
- `packages/protocol-ts/src/index.ts` — mirror the addition; type guard accepts the new variant.
- `packages/protocol-ts/src/index.test.ts` — round-trip test on the TS side.
- `crates/gateway/src/project.rs` — `run_driver` needs an mpsc channel to receive write commands. Lift the channel into a `DriverHandle` returned by `spawn_project` so the WS handler can find the right driver by id.
- `crates/gateway/src/server.rs` — handle `ClientMessage::TagWrite`:
  1. Parse `path` to extract the driver id (`<driver_id>/<address>`).
  2. Look up the `DriverHandle` for that driver.
  3. Send the write command down the channel.
  4. On success, the next `tag.update` from the driver's subscription will re-publish the new value naturally — no separate "write succeeded" message needed for v1.
  5. On invalid path / unknown driver / channel closed, emit `ServerMessage::Error { code: "tag.write.failed", message: ... }`.

### Wire form

Match the existing patterns:

```rust
// crates/protocol/src/lib.rs — add to ClientMessage:
#[serde(rename = "tag.write")]
TagWrite {
    path: String,
    value: TagValue,
},
```

Wire form: `{"kind":"tag.write","path":"rockwell-1/Setpoint","value":{"type":"real","value":42.5}}`.

### Test requirements

#### Rust unit test (`crates/protocol`)

- `client_tag_write_round_trips_with_stable_wire_form` — assert the literal wire form above.

#### Rust integration test (`crates/gateway`)

Extend `crates/gateway/tests/phase1_e2e.rs` (sim-tests gated):

- After connecting and observing a few `Pressure` updates, send `tag.write` for `rockwell-1/Setpoint` with value `42.5`.
- Assert no `protocol.parse` error comes back.
- Read `rockwell-1/Setpoint` (or wait for its next subscription update) and assert the simulator now reports `42.5`. The sim-rockwell `Setpoint` tag is `latch` behavior — it reflects whatever was written.

#### TS test (`packages/protocol-ts`)

- Wire-form round-trip for the new variant.

#### TS test (`apps/runtime-web/src/gatewayClient.test.ts`)

- `writeTag` sends the expected JSON frame on the WebSocket. (May already be implicitly covered by CODEX-L; verify.)

### Acceptance criteria

- [ ] `cargo fmt --all -- --check` clean.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] `cargo test --workspace --all-features --locked` green; the gateway integration test now exercises the write path.
- [ ] `pnpm -r typecheck && pnpm -r test && pnpm -r build` green.
- [ ] `examples/projects/phase1-demo/views/home.json` extended (or a sibling view added) with a `NumericInput` bound to `rockwell-1/Setpoint` so the manual smoke can validate the write end-to-end.
- [ ] No new architectural moves — keep the per-driver mpsc channel approach; do **not** refactor `run_driver` to use `DriverSupervisor` here. That refactor is a separately-scoped Phase 3 polish item already noted on CODEX-H's task page.

### Out of scope

- Auth checks on writes (Phase 3).
- Per-tag write ACLs (post-1.0).
- Write batching / coalescing.
- Write acknowledgements as a distinct wire message (`tag.update` after the write is sufficient for v1).
- The DriverSupervisor refactor for `run_driver` — explicit non-goal here.

### Risks / gotchas

- **Channel back-pressure**: if the WS handler sends writes faster than the driver can process, the per-driver mpsc fills up. Use a bounded channel (capacity 64 is fine) and on `try_send` overflow, emit `ServerMessage::Error { code: "tag.write.busy" }`. Don't block the WS handler.
- **Unknown driver id**: a `tag.write` for a path with a driver id that doesn't exist in the loaded project should fail fast with `tag.write.unknown_driver`, not panic.
- **Concurrent writes to the same tag**: the driver mpsc serializes them; last-write-wins. Document this in the wire-message rustdoc.
- **Don't add `request_id` to TagWrite**: keep the message minimal. If a future feature needs write acknowledgements, that's a v2 wire change.

## Codex log

*(codex — append working notes here)*

### 2026-04-27 10:43  codex
Started. Taking CODEX-N first so `NumericInput` previews in CODEX-M have an end-to-end write path. Reading protocol, gateway project driver ownership, driver API write behavior, and simulator tags before editing.

### 2026-04-27 14:46  codex
Submitted. Added `tag.write` to Rust protocol and protocol-ts, routed runtime `GatewayClient.writeTag` through the typed message, added a per-driver bounded mpsc write queue in the gateway project runner, and taught the websocket server to route writes by `<driver_id>/<address>` with explicit busy/unknown-driver/failure errors. Extended the Phase 1 demo project and `home` view with `rockwell-1/Setpoint` plus `NumericInput`. Verification: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features --locked`; `OPENWEBHMI_SIM_RUNNING=1 cargo test -p openwebhmi-gateway --features sim-tests --test phase1_e2e --locked`; `pnpm -r typecheck`; `pnpm -r test`; `pnpm -r build`.

## Claude review

### 2026-04-27  claude — review pass 1

Spec-compliant and tightly scoped. Matches the brief item-for-item without overreaching into the explicitly out-of-scope `DriverSupervisor` refactor.

- ✅ `ClientMessage::TagWrite { path, value }` with literal-wire-form round-trip test (`crates/protocol/src/lib.rs:69-75, 229-238`).
- ✅ `DriverHandles = HashMap<String, DriverHandle>` returned by `spawn_project`; `DriverHandle::try_write` uses `try_send` so the WS handler never blocks (`project.rs:23-57`).
- ✅ `WRITE_QUEUE_CAPACITY: usize = 64` per the brief's gotcha.
- ✅ `WriteEnqueueError::{Busy, Closed}` taxonomy maps cleanly to `tag.write.busy` / `tag.write.failed` codes.
- ✅ Path split via `split_once('/')` with empty-segment rejection (`server.rs:375-378`).
- ✅ Four distinct error codes wired exactly per the brief: `tag.write.failed` (bad path / closed channel), `tag.write.unknown_driver`, `tag.write.busy`.
- ✅ Server module gained four `serve_*` flavors covering the project_store × driver_handles matrix — slightly verbose but explicit, and lets Phase 1's e2e harness still construct an empty `DriverHandles` cleanly.
- ✅ Phase 1 e2e extended to write `42.5` to `rockwell-1/Setpoint` and observe the simulator latch behavior. Demo `home.json` now has a `NumericInput` so the manual smoke validates write end-to-end.
- ✅ TS protocol mirror + type guard + round-trip test landed.
- ✅ Test count: runtime-web 9 → 10 (+1 gatewayClient `writeTag` frame test); workspace cargo tests stay green; `--features sim-tests` e2e includes the write round-trip.

Findings:
- 🟢 The rustdoc on `ClientMessage::TagWrite` documents "concurrent writes are last-write-wins" exactly as the brief asked. Useful for downstream callers.
- 🟢 No `request_id` added — held the line on minimal wire form per the brief.
- 🟡 Cosmetic: the four `serve_*` functions could collapse into one with optional params, but the explicit naming reads cleanly. Phase 3+ refactor if it bothers anyone.

Acceptance criteria all met.

## Verdict

**Merged** at the next commit. The brief-error from CODEX-L is now closed end-to-end. Designer's NumericInput preview will work as soon as CODEX-M lands.
