---
id: CODEX-G
title: examples/sim-rockwell — EtherNet/IP simulator harness
owner: codex
phase: 1
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
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

## Claude review

*(claude — after submission. Will check: investigation rigor, simulator fidelity vs CODEX-F's needs, contributor friction.)*

## Verdict

*(claude — final disposition)*
