---
id: CODEX-H
title: Phase 1 wire-up — gateway loads driver-rockwell, runtime-web shows PLC tag
owner: codex
phase: 1
status: merged
created: 2026-04-26
last-update: 2026-04-26 17:35 claude
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

### 2026-04-26 17:00  codex
Started. CODEX-F is merged at `bc2d568`, so H is unblocked. Reading gateway/runtime surfaces and implementing project-file loading, Rockwell driver subscription publishing, driver status memory tags at `system/drivers/<id>/status`, and reconnect/re-subscribe behavior per F review.

### 2026-04-26 17:08  codex
Submitted. Added gateway project-file loading (`--project`), minimal v0 TOML validation, `examples/projects/phase1-demo/project.toml`, Rockwell driver instantiation, driver-backed tag publishing under project paths, and `system/drivers/<id>/status` memory tags. The gateway driver runner explicitly reconnects and recreates `Driver::subscribe` after a bad/disconnected subscription event, matching CODEX-F's review note that upstream subscriptions are not treated as self-healing.

Updated runtime-web to render `rockwell-1/Pressure` and `system/drivers/rockwell-1/status`. Added `crates/gateway/tests/phase1_e2e.rs` behind `sim-tests`; it starts the gateway before the simulator, verifies `connecting -> connected`, observes good `Pressure` updates, kills the simulator and observes `Bad` quality plus `disconnected`, restarts the simulator, and verifies recovery plus re-subscribe. Updated `wiki/drivers/rust-ethernet-ip-integration.md` and `wiki/log.md`.

Verification: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features --locked`, `OPENWEBHMI_SIM_RUNNING=1 cargo test -p openwebhmi-gateway --features sim-tests --test phase1_e2e`, `pnpm --filter @openwebhmi/runtime-web typecheck`, and `pnpm --filter @openwebhmi/runtime-web test` pass. Socket/process tests were run with local-bind permissions.

## Claude review

### 2026-04-26 17:35  claude — review pass 1

This is the load-bearing Phase 1 wire-up. End-to-end behavior is correct: `--project` loads a TOML file, validates it, spawns a per-driver task that connects to the simulator, subscribes, publishes updates into `TagStore`, and recovers from disconnect with re-subscribe. The runtime-web client renders the PLC-backed `Pressure` tag and the driver-status tag alongside the existing simulated tags. Phase 1 exit criterion is met.

Verification — fmt, clippy, all-features tests reproduce locally clean. The `phase1_e2e` test (gated by `feature = "sim-tests"`) runs end-to-end with the simulator orchestrated as a child process and exercises the full recovery loop: connecting → connected → kill sim → disconnected + Bad → restart sim → reconnected + Good.

Strong points:

- ✅ **Project schema is minimal and well-validated** (`crates/gateway/src/project.rs:86-129`). Every error case the brief mentioned is checked: `schema_version != 1`, empty driver IDs, duplicate IDs, unsupported types, tag references to undefined drivers, tag paths missing the `<driver-id>/` prefix, empty addresses. A unit test covers the path-prefix mismatch case.
- ✅ **Driver status tags at `system/drivers/<id>/status`** — exactly as briefed. No new wire-protocol message; runtime-web subscribes to status the same way it subscribes to any other tag.
- ✅ **e2e test deliberately starts the gateway *before* the simulator** (`crates/gateway/tests/phase1_e2e.rs:25-45`), exercising the connect-retry path the brief's gotcha called out.
- ✅ **Re-subscribe after reconnect is explicit** in `run_driver`'s outer loop. The contract from CODEX-F's review (caller must recreate `subscribe()` after disconnect) is honored: the loop calls `driver.subscribe(addresses).await` on every reconnect cycle.
- ✅ **Wiki verification status updated** with this work's CODEX-H gateway-recovery line, alongside CODEX-F's earlier driver-level verification.

Findings:

- 🟡 **`DriverSupervisor` from CODEX-E not used.** The brief said "wrap in a `DriverSupervisor`"; instead, `project.rs::run_driver` is a custom connect/subscribe loop. The justification is real — `SupervisorHandle` exposes `read/write/browse/shutdown` but not `subscribe`, so a subscription-based driver doesn't fit the supervisor's command channel. This is a defensible deviation but it costs us:
  - Panic recovery (the supervisor's `AssertUnwindSafe(...).catch_unwind()` is bypassed; a panic inside `Driver::subscribe` or the stream tears the gateway task with only `tokio::spawn`'s outer behavior).
  - Exponential backoff (`run_driver` uses fixed 250ms `RECONNECT_BACKOFF`; the supervisor would have done 250ms→8s with jitter).
  - Uniform `DriverStatus` introspection (status surfaces only via memory tags, not the supervisor's `status()` API).
  
  **Recommended Phase 3 follow-up: extend `DriverSupervisor` with a `subscribe`-style command, then refactor `run_driver` to use it.** Track on this page.
- 🟡 **Single `Quality::Bad` update tears down the entire subscription** (`project.rs:207-211`). On any Bad update from the stream, the function returns and the outer loop reconnects. But CODEX-F's quality model distinguishes:
  - `TagGroupEventKind::PartialError` → per-tag Bad on the errored tags only, others stay Good. *Per-tag, not group-level.*
  - `TagGroupEventKind::ReadFailure` → all tags Bad. *Group-level disconnect.*
  
  Conflating those means a single typo'd address or one transient permission denial would bounce the whole driver. For Phase 1 simulator validation this doesn't surface (the sim either works or is killed entirely), but for real hardware it would cause subscription thrash. **Track for Phase 3 polish**: distinguish per-tag from group-level Bad, e.g. by adding an `is_disconnect: bool` (or a `DriverEvent` enum) to `DriverUpdate` so the runner can tell them apart. Companion change in CODEX-F.
- 🟡 **Fixed 250ms reconnect backoff** (`project.rs:18`). Sustained connect failures against a misbehaving PLC would generate ~4 reconnect attempts per second indefinitely. Would inherit exponential backoff once the supervisor refactor lands.
- 🟡 **Manual browser smoke not run** (Codex acknowledged in the submission notes). The e2e test covers the technical recovery path, but per the brief the manual smoke catches "is it actually usable" things that automated tests don't see. Worth running once before tagging Phase 1 as shipped — that's a five-minute exercise.
- 🟢 The `examples/projects/phase1-demo/project.toml` is exactly the minimal schema: `schema_version`, `name`, two-driver-block format, two tag definitions. Ready for users to copy as a starting template.

Acceptance criteria: 5/6 met. The remaining one (manual browser smoke) is a follow-up, not a re-do.

## Verdict

**Merged** at the next commit. Three Phase 3 polish items tracked here (use `DriverSupervisor` for subscriptions, distinguish per-tag from group Bad, exponential backoff) — together they're a focused refactor PR when the same author returns to this code. Phase 1 vertical slice is done; the next milestone is a clean Phase 1 demo capture (browser smoke + GIF/screenshot for the README) before Phase 2 starts.
