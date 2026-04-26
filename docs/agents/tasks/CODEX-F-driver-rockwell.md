---
id: CODEX-F
title: crates/driver-rockwell — wrap rust-ethernet-ip 0.7.x
owner: codex
phase: 1
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
blocked-by: CODEX-E
---

# CODEX-F — `crates/driver-rockwell`

## Brief

> **Blocked by [CODEX-E](CODEX-E-driver-api.md)** (the `Driver` trait and types). May be partially scaffolded in parallel, but cannot land until E lands.

### Goal

The first real OpenWebHMI driver. Wraps the upstream `rust-ethernet-ip` crate (v0.7.x, see [wiki page](../../../wiki/drivers/rust-ethernet-ip-integration.md)) behind the `Driver` trait. Targets Rockwell CompactLogix and ControlLogix.

### Context to read first

- `wiki/drivers/rust-ethernet-ip-integration.md` — **the** reference for how this wrapper should behave. It enumerates the upstream API surface, validated targets, quality mapping, known limitations, and upgrade workflow.
- `crates/driver-api/src/*` (delivered by CODEX-E).
- Upstream crate at `v0.7.0` (commit `592bfa716309e3388cf8143c4095622d6302a7f6`): https://github.com/sergiogallegos/rust-ethernet-ip — particularly the README and any `examples/` it ships.
- `docs/architecture.md` §4.4.

### Files to create

- `crates/driver-rockwell/Cargo.toml` — declares `rust-ethernet-ip = { workspace = true }`, `openwebhmi-driver-api = { path = "../driver-api" }`, plus tokio/serde from workspace.
- `crates/driver-rockwell/src/lib.rs` — re-exports.
- `crates/driver-rockwell/src/driver.rs` — `RockwellDriver` struct implementing `Driver`.
- `crates/driver-rockwell/src/config.rs` — `RockwellConfig` (host, slot, route, poll_rate_ms, connection_timeout_ms).
- `crates/driver-rockwell/src/types.rs` — `TagValue` ↔ `PlcValue` conversions.
- `crates/driver-rockwell/src/eip_client.rs` — internal `EipClientLike` trait abstracting the upstream `EipClient` for testability.
- `crates/driver-rockwell/tests/unit.rs` — unit tests using a mock `EipClientLike`.
- `crates/driver-rockwell/tests/integration.rs` — integration tests against the simulator from CODEX-G (gated by `#[cfg(feature = "sim-tests")]`).

Add `crates/driver-rockwell` to workspace `members`.

### Configuration

```rust
pub struct RockwellConfig {
    pub host: String,            // e.g. "192.168.1.10"
    pub slot: u8,                // backplane slot for ControlLogix; 0 for CompactLogix
    pub route: Option<String>,   // CIP route path, e.g. "1,0"
    pub poll_rate_ms: u32,       // default 250
    pub connection_timeout_ms: u32, // default 5000
}
```

Parsed from `serde_json::Value` in `Driver::connect`.

### Trait implementation

| `Driver` method | Backed by |
|---|---|
| `metadata()` | hard-coded `DriverMetadata { vendor: "Rockwell", family: "EtherNet/IP-CIP", crate_version: env!("CARGO_PKG_VERSION"), capabilities: { native_subscribe: true, browse: false, batch_read: true, batch_write: true } }` |
| `connect(config)` | parse `RockwellConfig`, instantiate `EipClient`, drive bring-up |
| `disconnect()` | drop client, transition state |
| `browse(_)` | return `Err(DriverError::Other(anyhow!("browse not implemented; use external tag-list import")))` for v1 |
| `read(addr)` | `eip.read_tag(&addr.raw)` → `PlcValue` → `TagValue` |
| `write(addr, value)` | `TagValue` → `PlcValue` → `eip.write_tag(&addr.raw, ...)`. Apply read-modify-write workaround for `STRING` tags and UDT array element members per the wiki page. |
| `subscribe(addrs)` | `upsert_tag_group` + `subscribe_tag_group`; map `TagGroupEventKind` → `DriverUpdate` per the wiki page's quality-mapping table |

### Quality mapping

Implement exactly the table in `wiki/drivers/rust-ethernet-ip-integration.md` §"Quality mapping". Surface the upstream `TagGroupEventKind::PartialError` per-tag, not as a whole-group failure.

### Reconnect behavior

- On `EipClient` connection loss (detected by an upstream error class — TBD; investigate during implementation and document in your `## Codex log`):
  - Transition all tags to `Quality::Bad("disconnected")` until reconnect succeeds.
  - During in-flight reconnect, transition tags to `Quality::Uncertain("reconnecting")`.
  - Backoff handled by the `DriverSupervisor` from CODEX-E; this driver only reports state and lets the supervisor decide retry timing.

### Test requirements

#### Unit tests (`tests/unit.rs`, no network, no simulator)

Use a `MockEipClient: EipClientLike` that records calls and returns scripted responses.

- `Driver::read` happy path: `EipClient::read_tag` returns `PlcValue::Real(1.5)` → driver returns `TagValue::Real(1.5)`.
- `Driver::write` happy path for each `PlcValue` variant.
- `Driver::write` for STRING with the read-modify-write workaround active.
- `Driver::read` when `EipClient::read_tag` returns the upstream "no such tag" error → driver returns `DriverError::InvalidAddress(...)`.
- Quality mapping: each `TagGroupEventKind` variant → expected `Quality`.

#### Integration tests (`tests/integration.rs`, gated by `feature = "sim-tests"`)

Run against the simulator from CODEX-G. Skipped by default in CI; opt-in via `cargo test -p openwebhmi-driver-rockwell --features sim-tests` and a `OPENWEBHMI_SIM_RUNNING=1` env var.

- Connect to simulator (`127.0.0.1:44818` by convention).
- Read a tag named `Counter` (provided by the sim) and assert it increments across two reads.
- Write a tag named `Setpoint` and read it back.
- Subscribe to a tag group of two tags and observe at least 3 updates within 2 seconds.
- Kill the simulator (the test harness controls it) and assert quality transitions to `Bad`. Restart and assert it returns to `Good` within 5 seconds.

### Acceptance criteria

- [ ] `cargo build -p openwebhmi-driver-rockwell` exits 0.
- [ ] `cargo clippy -p openwebhmi-driver-rockwell --all-targets -- -D warnings` exits 0.
- [ ] `cargo test -p openwebhmi-driver-rockwell` (unit tests) exits 0.
- [ ] `cargo test -p openwebhmi-driver-rockwell --features sim-tests` exits 0 with the sim from CODEX-G running.
- [ ] Wiki page `wiki/drivers/rust-ethernet-ip-integration.md` updated: in the §"Independent verification status" table, flip the rows that are now verified (with this PR's commit ref).
- [ ] `wiki/log.md` appended with one line.
- [ ] All public items have rustdoc.

### Out of scope

- Driver registry (Phase 4).
- Auto-tag-import from a Studio 5000 export file (post-1.0).
- `browse()` implementation (depends on whether upstream exposes introspection at v0.7.x — leave for next upstream version).
- Real-hardware validation — that's the Phase 4 hardware gate, not Phase 1.

### Risks / gotchas

- Upstream API may not expose every behavior we need. If a gap is found, **document it in `wiki/drivers/rust-ethernet-ip-integration.md` Open Questions section** and either work around it, or open an upstream issue and link from your `## Codex log`. Do not silently work around — the wiki entry is load-bearing.
- The internal `EipClientLike` trait is the testability seam. Don't put logic in it that isn't strictly the upstream surface; the wrapper logic stays in `RockwellDriver` so it's covered by mocked unit tests.
- Don't `unwrap()` upstream errors. Map every error path explicitly to a `DriverError` variant — the goal is that *all* upstream failure modes flow through the supervisor with the right quality semantics.
- Reconnect coordination: when `subscribe_tag_group` is active and the connection drops, do we need to re-call `subscribe_tag_group` after reconnect, or does the upstream auto-resume? **This is an open question** in the wiki. Resolve it during implementation and update the wiki.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
