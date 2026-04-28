# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 3 — Core SCADA features

> Phase 3 is the largest phase by scope: alarms + history + trends + auth + scripting + script editor. Seven tasks. Most can run in parallel — the dependency graph below shows which ones gate others. Phase 3 exit criterion (per `docs/roadmap.md`): the demo HMI raises a high-temperature alarm, trends a process variable for 24h, requires login with an `Operator` role to write tags, and runs a Python script that writes a derived setpoint based on two inputs.

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-P | `Trend` component — multi-pen historical + live chart | codex | open | 2026-04-27 claude | [tasks/CODEX-P-trend-component.md](tasks/CODEX-P-trend-component.md) |
| CODEX-R | `AlarmTable` component + designer alarm config | codex | open (Q merged → unblocked) | 2026-04-28 claude | [tasks/CODEX-R-alarm-ui.md](tasks/CODEX-R-alarm-ui.md) |
| CODEX-T | `crates/scripting` — CPython 3.11+ host + worker subprocesses + system.* RPC | codex | open (no blockers) | 2026-04-27 claude | [tasks/CODEX-T-scripting-host.md](tasks/CODEX-T-scripting-host.md) |
| CODEX-U | Designer script editor — Monaco + Python syntax + system.* stubs | codex | open (blocked-by T) | 2026-04-27 claude | [tasks/CODEX-U-script-editor.md](tasks/CODEX-U-script-editor.md) |

### Phase 3 dependency graph

```
CODEX-P  (Trend component)       ← unblocked
CODEX-R  (AlarmTable + UI)       ← unblocked (Q merged)
CODEX-T  (scripting host)        ← no blockers
CODEX-U  (script editor)         blocked-by T
```

**Three can start in parallel: P, R, T.** Backend trio O+Q+S is complete — historian + alarms + auth all in. CODEX-R now plumbs the alarm UI on top of CODEX-Q, and should use the verified session's username as `who` on ack (per the CODEX-Q review note).

## Phase 2 — Designer MVP

**🎉 Phase 2 code-complete.** All Required-slice tasks (I/J/K/L/M + the CODEX-N follow-up) merged. Awaiting **manual-smoke validation** of the 10-step checklist in [`apps/designer/README.md`](../../apps/designer/README.md) before tagging `v0.2.0`. The Stretch slice (visual canvas, drag/drop, snap-to-grid, undo/redo, theme editor UI, four more components) is deferred to Phase 4 per the de-risked plan.

*(no open Phase 2 tasks)*

## Phase 1 — Vertical slice (PLC tag in browser, simulator-backed)

**🎉 Phase 1 complete.** Released as [`v0.1.0`](https://github.com/sergiogallegos/OpenWebHMI/releases/tag/v0.1.0). Exit criterion met: a PLC tag from the Rockwell driver, sourced from `examples/sim-rockwell`, updates live in the browser through the gateway, with quality propagation on simulator restart. Hardware validation remains gated to pre-1.0 (per `docs/roadmap.md`).

*(no open Phase 1 tasks)*

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
| CODEX-I | `crates/project-store` — gateway-side project storage with versioning | codex | `24c1ac7` | 2 |
| CODEX-J | View schema + protocol additions for view-tree authoring | codex | `24c1ac7` | 2 |
| CODEX-K | `packages/component-library` — 6 essential components | codex | `24c1ac7` | 2 |
| CODEX-L | `apps/runtime-web` — load views from gateway, render via component library | codex | `921e3d9` | 2 |
| CODEX-N | `tag.write` end-to-end — protocol message + gateway routing to driver | codex | `f40b780` | 2 |
| CODEX-M | `apps/designer` — Tauri shell, project explorer, form-based view editor | codex | `bc5bd38` | 2 |
| CODEX-O | `crates/historian` — tag time-series storage + read API | codex | `9db711e` | 3 |
| CODEX-S | `crates/auth` — local users + roles + JWT sessions + per-view ACLs | codex | `7eff30a` | 3 |
| CODEX-Q | `crates/alarm-engine` — definitions + state machine + journal | codex | `ff56780` | 3 |

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
