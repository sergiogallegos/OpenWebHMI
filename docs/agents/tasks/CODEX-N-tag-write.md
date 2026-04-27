---
id: CODEX-N
title: tag.write end-to-end — protocol message + gateway routing to driver
owner: codex
phase: 2
status: open
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

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
