# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 1 — Vertical slice (PLC tag in browser, simulator-backed)

**🎉 Phase 1 complete.** Exit criterion met: a PLC tag from the Rockwell driver, sourced from `examples/sim-rockwell`, updates live in the browser through the gateway, with quality propagation on simulator restart. Hardware validation remains gated to pre-1.0 (per `docs/roadmap.md`).

*(no open Phase 1 tasks)*

## Phase 2 — Designer MVP

*(open this section's tasks once Phase 1 demo capture is recorded for the README)*

## Done

| Id | Title | Owner | Merge commit | Phase |
|---|---|---|---|---|
| CODEX-A | `packages/protocol-ts` — TS protocol types | codex | `75ccb9c` | 0 |
| CODEX-B | `crates/gateway` — WS gateway binary with sim provider | codex | `75ccb9c` | 0 |
| CODEX-C | `apps/runtime-web` — React+Vite client | codex | `75ccb9c` | 0 |
| CODEX-D | `.github/workflows/ci.yml` — Phase 0 CI | codex | `75ccb9c` | 0 |
| CODEX-E | `crates/driver-api` — Driver trait + types + supervisor | codex | `e19a3c2` | 1 |
| CODEX-G | `examples/sim-rockwell` — EtherNet/IP simulator harness | codex | `e19a3c2` | 1 |
| CODEX-F | `crates/driver-rockwell` — wrap `rust-ethernet-ip` 0.7.x | codex | `bc2d568` | 1 |
| CODEX-H | Phase 1 wire-up — gateway loads driver-rockwell, runtime-web shows PLC tag | codex | `ca481d4` | 1 |

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
