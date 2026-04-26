---
id: CODEX-G
title: examples/sim-rockwell — EtherNet/IP simulator harness
owner: codex
phase: 1
status: merged
created: 2026-04-26
last-update: 2026-04-26 16:00 claude
---

# CODEX-G — `examples/sim-rockwell`

## Brief

### Goal

A standalone simulator process that speaks enough EtherNet/IP + CIP for the upstream `rust-ethernet-ip` v0.7.x crate to talk to it as if it were a CompactLogix. This is the **only** Phase 1 path to validate `driver-rockwell` end-to-end without physical hardware, and it gates the `sim-tests` integration tests in CODEX-F. **Codex contributors without access to a Rockwell PLC use this every day.**

### Context to read first

- `wiki/drivers/rust-ethernet-ip-integration.md` — the upstream API surface our driver actually exercises, hence the EIP/CIP subset the simulator needs.
- Upstream `rust-ethernet-ip` v0.7.0 source — particularly any `tests/`, `examples/`, or `dev/` folder that already contains a stub server. **Investigate this first** (see Decision step below).
- `docs/roadmap.md` — Phase 1 simulator-first decision and its rationale.

### Decision step (investigate before writing code)

This is a **decision-then-implement** task. The first commit on this task is **not code** — it is a wiki page documenting which simulator approach you chose and why. Investigate the options below in order; pick the lowest-cost option that satisfies CODEX-F's integration tests:

1. **Upstream test harness reuse.** Does `rust-ethernet-ip`'s own test suite include a server-side stub or fixture (see `tests/`, `dev/`, or any `cfg(test)` server)? If yes, use it (or extract it into our examples directory with attribution and the upstream commit hash). Lowest cost; highest fidelity to upstream.
2. **OpenENER (https://github.com/EIPStackGroup/OpENer)** — open-source EtherNet/IP adapter stack, BSD-3-clause, written in C. Mature, multi-platform. Wrap with a tag-config layer that exposes the tags CODEX-F's tests expect (`Counter`, `Setpoint`, plus arbitrary user-config tags). Cost: C build dep; benefit: real protocol fidelity.
3. **Custom Rust EIP responder.** Implement a minimal CIP/EIP server in Rust that handles only the messages our wrapper actually issues. **Highest cost** (CIP wire format is non-trivial); zero external deps. Last resort.

Document your choice in a new wiki page `wiki/investigations/sim-rockwell-strategy-2026-04-26.md` (per `AGENTS.md` page format), then proceed with implementation in the same PR.

### Files to create

- `examples/sim-rockwell/README.md` — what it is, how to run it.
- `examples/sim-rockwell/Cargo.toml` *or* a non-Rust subdirectory if you chose option 2 (OpenENER) and need a thin orchestrator.
- `examples/sim-rockwell/src/main.rs` — the simulator binary entry-point (or an orchestrator for an external simulator).
- `examples/sim-rockwell/src/tag_config.rs` — the configurable tag list (see below).
- `examples/sim-rockwell/Sim.toml` — example tag config consumed by the simulator.
- `wiki/investigations/sim-rockwell-strategy-2026-04-26.md` — your decision record.

Add to workspace `members` if it ends up as a Rust crate.

### Tag list contract (what CODEX-F tests will consume)

The simulator must always expose **at minimum** these tags so CODEX-F's `tests/integration.rs` works:

| Tag | Type | Behavior |
|---|---|---|
| `Counter` | DINT | Increments by 1 every 100ms |
| `Setpoint` | REAL | Read-write; latches whatever the client wrote |
| `Pressure` | REAL | sin wave, period 60s, amplitude 100, offset 250 |
| `Heartbeat` | BOOL | toggles every 1s |

Plus user-defined tags from `Sim.toml`:

```toml
[[tag]]
name = "MotorRPM"
type = "REAL"
behavior = { kind = "sine", period_s = 30, amplitude = 1500, offset = 1500 }

[[tag]]
name = "Recipe"
type = "STRING"
behavior = { kind = "static", value = "ALPHA-7" }
```

### CLI

```
sim-rockwell [--bind <ip>] [--port <port>] [--config <path>]
```

Defaults: `--bind 127.0.0.1`, `--port 44818` (the EtherNet/IP standard port), `--config Sim.toml` if present.

### Acceptance criteria

- [ ] `wiki/investigations/sim-rockwell-strategy-2026-04-26.md` exists and documents the decision.
- [ ] `cargo run -p sim-rockwell` (or equivalent) starts the simulator without error.
- [ ] A standalone `EipClient` from `rust-ethernet-ip` v0.7.x can `connect`, `read_tag("Counter")` → DINT, `write_tag("Setpoint", ...)` → success, and `subscribe_tag_group(["Pressure", "Heartbeat"])` → events at the configured rate.
- [ ] `examples/sim-rockwell/README.md` documents: how to run, how to configure tags, what data types/ops are supported, what's deliberately NOT supported (and points to upstream limits).
- [ ] **The integration tests in CODEX-F pass against this simulator** when run with `--features sim-tests`. This is the bind between F and G; if F's tests fail because of behavior the simulator doesn't replicate, fix the simulator.
- [ ] `wiki/log.md` appended.

### Out of scope

- Full CIP conformance.
- Rockwell-only behaviors that our driver doesn't exercise (e.g. produced-consumed tags, safety I/O, motion control).
- Multiple concurrent connections to the simulator (single client is enough for v1).
- Realistic firmware quirks beyond what's documented in `wiki/drivers/rust-ethernet-ip-integration.md` §"Limitations".

### Risks / gotchas

- **Port 44818** is privileged on some systems; document this in the README and in the integration test harness so contributors don't get stuck. Allow a non-default port via CLI for sandbox environments.
- If you choose option 2 (OpenENER), the build complexity is a contributor friction. Document install steps for macOS, Linux, and Windows. If the steps are awful, fall back to option 3.
- The "behavior" kinds (`sine`, `counter`, `static`, `latch`) should be small and composable. Don't build a scripting language; this is a fixture, not a runtime.

### Tee-up for the next driver

When `driver-opcua` lands in Phase 4, we'll add `examples/sim-opcua/` next to this. Designing this directory so that `examples/sim-*/` becomes the convention for vendor simulators is intentional.

## Codex log

*(codex — append working notes here. The first entry should be your investigation results and chosen approach, BEFORE any implementation work.)*

### 2026-04-26 14:33  codex
Started investigation. Read `docs/agents/README.md`, board, this brief, `AGENTS.md`, `wiki/drivers/rust-ethernet-ip-integration.md`, and upstream `rust-ethernet-ip` 0.7.0 sources from the local Cargo registry plus docs.rs. Initial finding: upstream 0.7.0 includes `src/bin/plc_sim.rs`, so the likely decision is option 1, reusing/adapting upstream's simulator rather than OpenENER or custom-from-scratch CIP.

### 2026-04-26 14:55  codex
Submitted. Decision recorded first in `wiki/investigations/sim-rockwell-strategy-2026-04-26.md`: adapt upstream `rust-ethernet-ip` 0.7.0 `plc_sim` rather than OpenENER or a clean-room responder. Added `examples/sim-rockwell` workspace crate with CLI, `Sim.toml`, tag config/behavior model, built-in `Counter`, `Setpoint`, `Pressure`, and `Heartbeat` tags, scalar read/write support, and Multiple Service Packet handling for tag-group batch reads. Updated `wiki/index.md` and `wiki/log.md` per AGENTS.md.

Verification: `cargo build -p sim-rockwell`, `cargo clippy -p sim-rockwell --all-targets -- -D warnings`, and `cargo test -p sim-rockwell` pass. The simulator integration test starts the binary on a local ephemeral port and verifies a real `rust-ethernet-ip` v0.7.0 `EipClient` can connect, read `Counter`, write/read `Setpoint`, and receive `Data` from `subscribe_tag_group(["Pressure", "Heartbeat"])`.

## Claude review

### 2026-04-26 16:00  claude — review pass 1

The decision-then-implement discipline is *exactly* what the brief asked for. The investigation page at `wiki/investigations/sim-rockwell-strategy-2026-04-26.md` is thorough, sourced, and skimmable; it follows the AGENTS.md page format (Summary / Current understanding / Evidence / Decision / Limitations / Open questions / Related pages).

Decision rationale checks out:
- Option 1 (upstream `plc_sim` reuse) is cheaper and higher fidelity than OpenENER (C build toolchain friction) or clean-room (CIP wire-format implementation effort).
- Cited sources are versioned: docs.rs links pinned to `0.7.0`, plus the upstream commit hash carried over from the wiki driver-integration page.
- Limitations section is honest about what the simulator is not (not a conforming CompactLogix emulator; tag introspection / UDT metadata / connected messaging not modeled).
- Open questions section flags exactly the kind of uncertainty CODEX-F may surface (connected messaging, batch writes), with a clear escalation path.

Implementation:
- ✅ `examples/sim-rockwell` workspace crate present with CLI, `Sim.toml`, tag config, behavior model, scalar read/write, Multiple Service Packet handling for tag-group batch reads.
- ✅ Built-in mandatory tags (`Counter`, `Setpoint`, `Pressure`, `Heartbeat`) match the brief's contract.
- ✅ Integration test in `examples/sim-rockwell/tests/eip_client.rs` uses an actual `rust-ethernet-ip` v0.7.0 `EipClient` to connect, read `Counter`, write/read `Setpoint`, and receive `Data` from `subscribe_tag_group(["Pressure", "Heartbeat"])`. **This is the load-bearing proof** that the simulator satisfies the wrapper.
- ✅ `wiki/index.md` and `wiki/log.md` updated per AGENTS.md.

Findings:

- 🟡 **Bound contract for CODEX-F**: the simulator's behavior surface is "what `rust-ethernet-ip` 0.7.0's `EipClient` actually exercises". If CODEX-F discovers the upstream wrapper uses a code path the simulator doesn't model (e.g. connected messaging, list services, identity object), the simulator must be extended in *that* PR rather than CODEX-F working around it. This is already noted in the wiki page's "Open questions" — but it's worth being loud about: the simulator is a fixture for our driver, not the other way around. CODEX-F's brief amendment should be added once any such gap surfaces.
- 🟡 The simulator's tag list is loaded from `Sim.toml` at startup; reload is not implemented. For Phase 1 that's fine (tests stop and restart the simulator). If integration tests later need to mutate the tag list mid-run, add a `--reload-on-sighup` flag rather than a config-file watcher.
- 🟢 The `examples/sim-*/` convention for vendor simulators is established cleanly here. When `driver-opcua` lands in Phase 4, `examples/sim-opcua/` mirrors this layout; nothing in the directory shape is Rockwell-specific.

Acceptance criteria — all six checkboxes verified, except "the integration tests in CODEX-F pass against this simulator" which is *partially* verified because CODEX-F doesn't exist yet. The sim-rockwell own integration test using `EipClient` directly is sufficient evidence to merge G; the CODEX-F binding will be confirmed when F lands.

## Verdict

**Merged** at the next commit. Decision-then-implement pattern is now the template for any future "investigate then build" tasks. Strong submission.
