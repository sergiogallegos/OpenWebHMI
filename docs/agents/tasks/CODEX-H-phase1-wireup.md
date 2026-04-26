---
id: CODEX-H
title: Phase 1 wire-up — gateway loads driver-rockwell, runtime-web shows PLC tag
owner: codex
phase: 1
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
blocked-by: CODEX-E, CODEX-F, CODEX-G
---

# CODEX-H — Phase 1 end-to-end wire-up

## Brief

> **Blocked by [CODEX-E](CODEX-E-driver-api.md), [CODEX-F](CODEX-F-driver-rockwell.md), and [CODEX-G](CODEX-G-sim-rockwell.md).** Cannot start in earnest until E + F + G are merged. A scaffold-only first commit (gateway accepts a driver config block but does nothing with it yet) is fine in parallel with F.

### Goal

Compose the Phase 1 deliverable: the gateway loads `driver-rockwell` from a project file, connects to the `sim-rockwell` simulator, publishes the simulator's tags into the existing `TagStore`, and the runtime-web client renders a PLC-backed tag in the browser. **This is the Phase 1 exit-criterion demo.**

### Context to read first

- `docs/roadmap.md` — Phase 1 exit criterion. The bar this task clears.
- `crates/driver-api/*` (CODEX-E).
- `crates/driver-rockwell/*` (CODEX-F).
- `examples/sim-rockwell/*` (CODEX-G).
- Existing `crates/gateway/*` and `apps/runtime-web/*` from Phase 0.

### Files to create / modify

- `crates/gateway/Cargo.toml` — add `openwebhmi-driver-api`, `openwebhmi-driver-rockwell`.
- `crates/gateway/src/project.rs` — minimal project file loader (see schema below).
- `crates/gateway/src/main.rs` — accept `--project <path>`; if present, load it and instantiate drivers.
- `crates/gateway/src/server.rs` — keep the existing simulator at `system/sim/*`; route driver tags under `<provider>/...` per architecture §4.2.
- `examples/projects/phase1-demo/project.toml` — the demo project file.
- `apps/runtime-web/src/App.tsx` — add a third row showing a PLC-backed tag (`rockwell-1/Pressure` from the simulator).
- `crates/gateway/tests/phase1_e2e.rs` — end-to-end test that orchestrates simulator + gateway + WS client.

### Project file schema (v0)

```toml
# project.toml — minimal Phase 1 schema
schema_version = 1
name = "phase1-demo"

[[drivers]]
id = "rockwell-1"
type = "rockwell"

[drivers.config]
host = "127.0.0.1"
slot = 0
poll_rate_ms = 250
connection_timeout_ms = 5000

[[tags]]
path = "rockwell-1/Pressure"
driver = "rockwell-1"
address = "Pressure"

[[tags]]
path = "rockwell-1/Counter"
driver = "rockwell-1"
address = "Counter"
```

Validation rules:
- `schema_version = 1` is required; future versions get explicit migration paths.
- Every `tags[].driver` must reference a defined `drivers[].id`.
- `tags[].path` must start with `<driver-id>/` for driver-backed tags.

### Gateway behavior changes

1. On boot, if `--project <path>` is given:
   - Parse the project file (return an error and exit non-zero if invalid; do not start with a partial project).
   - For each driver: instantiate via a small registry (Phase 1 has only `"rockwell" → RockwellDriver`), wrap in a `DriverSupervisor` from CODEX-E, call `connect`.
   - For each tag: subscribe the supervisor stream and pipe `DriverUpdate` events into `TagStore::publish` under the project-defined path.
2. Without `--project`, the gateway behaves exactly as in Phase 0 (sim-only).
3. Driver status surfaces as `system/drivers/<id>/status` memory tags, `Quality::Good` always, value is `TagValue::String` of `"connected" | "connecting" | "disconnected" | "faulted:<reason>"`. (Lets the UI show PLC connection state without inventing a new wire-protocol message.)

### Runtime-web changes

Minimal: extend `App.tsx` to render a third row for `rockwell-1/Pressure` plus a `system/drivers/rockwell-1/status` indicator. No styling work; UI is throwaway demo per Phase 1 scope.

### End-to-end test (`crates/gateway/tests/phase1_e2e.rs`, gated by `feature = "sim-tests"`)

- Orchestrate: spawn `sim-rockwell` (use its binary or library API depending on CODEX-G's choice); spawn the gateway with `--project examples/projects/phase1-demo/project.toml`; connect a WS client.
- Subscribe to `rockwell-1/Pressure`; assert ≥2 `tag.update` messages with `Quality::Good` arrive within 3 seconds.
- Subscribe to `system/drivers/rockwell-1/status`; assert it transitions through `connecting` → `connected`.
- Kill the simulator; assert pressure quality transitions to `Bad` within 5 seconds AND the driver status tag transitions to `disconnected`.
- Restart simulator; assert recovery within 8 seconds.

### Acceptance criteria

- [ ] `cargo build --workspace` exits 0.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] `cargo test --workspace` exits 0 (the e2e test runs only with `--features sim-tests`).
- [ ] Manual smoke per `docs/roadmap.md` Phase 1 exit criterion: with sim + gateway + runtime-web running, the browser shows `Pressure` updating ~1Hz; killing the simulator flips quality to bad and the driver-status tag to `disconnected`; restarting it restores both within 5–8 seconds.
- [ ] `docs/feature-matrix.md` row for OPC UA → leave as v1; row for Rockwell EtherNet/IP → still v1; no scope drift in the matrix.
- [ ] `wiki/log.md` appended; `wiki/drivers/rust-ethernet-ip-integration.md` "Independent verification status" rows updated where applicable.

### Out of scope

- Visual designer (Phase 2).
- Project file hot-reload.
- Tag scaling / engineering units / deadband (Phase 1 publishes raw values).
- More than one driver instance (the schema supports it; we just don't exercise it in the demo project).
- Auth (Phase 3).

### Risks / gotchas

- **Project file format will evolve.** Don't paint into a corner with a maximalist v0 schema. The structure above is deliberately minimal.
- The end-to-end test depends on simulator startup ordering. If the gateway connects before the simulator is bound, the supervisor must retry — this exercises CODEX-E's backoff. Make the test deliberately reverse the start order to validate.
- Manual smoke vs automated test: keep both. The e2e test catches regressions; the manual smoke catches "is it actually usable" things tests don't see.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission. Will check: project schema cleanliness, driver-status-as-tag pattern fitness, recovery timing.)*

## Verdict

*(claude — final disposition)*
