# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 1 — Vertical slice (PLC tag in browser, simulator-backed)

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-F | `crates/driver-rockwell` — wrap `rust-ethernet-ip` 0.7.x | codex | open (E + G now merged → unblocked) | 2026-04-26 claude | [tasks/CODEX-F-driver-rockwell.md](tasks/CODEX-F-driver-rockwell.md) |
| CODEX-H | Phase 1 wire-up — gateway loads driver-rockwell, runtime-web shows PLC tag | codex | open (blocked-by F) | 2026-04-26 claude | [tasks/CODEX-H-phase1-wireup.md](tasks/CODEX-H-phase1-wireup.md) |

## Done

| Id | Title | Owner | Merge commit | Phase |
|---|---|---|---|---|
| CODEX-A | `packages/protocol-ts` — TS protocol types | codex | `75ccb9c` | 0 |
| CODEX-B | `crates/gateway` — WS gateway binary with sim provider | codex | `75ccb9c` | 0 |
| CODEX-C | `apps/runtime-web` — React+Vite client | codex | `75ccb9c` | 0 |
| CODEX-D | `.github/workflows/ci.yml` — Phase 0 CI | codex | `75ccb9c` | 0 |
| CODEX-E | `crates/driver-api` — Driver trait + types + supervisor | codex | `e19a3c2` | 1 |
| CODEX-G | `examples/sim-rockwell` — EtherNet/IP simulator harness | codex | `e19a3c2` | 1 |

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
