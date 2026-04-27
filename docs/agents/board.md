# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 2 — Designer MVP

> Phase 2 is the highest-risk phase per the external-review pass: a drag/drop visual editor is genuinely a small IDE. Phase 2 is split into **Required** (form-based view editor, 6 essentials, no canvas) and **Stretch** (visual canvas + drag/drop). The five tasks below cover the Required slice.

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-M | `apps/designer` — Tauri shell, project explorer, form-based view editor | codex | open (L merged → unblocked) | 2026-04-27 claude | [tasks/CODEX-M-designer-shell.md](tasks/CODEX-M-designer-shell.md) |
| CODEX-N | `tag.write` end-to-end — protocol message + gateway routing to driver | codex | open (no blockers) | 2026-04-27 claude | [tasks/CODEX-N-tag-write.md](tasks/CODEX-N-tag-write.md) |

### Phase 2 dependency graph

```
CODEX-M  (designer)             ← unblocked, ready to start
CODEX-N  (tag.write end-to-end) ← unblocked, ready to start in parallel
```

CODEX-N is a small focused task that closes a brief-error from CODEX-L (Phase 0 didn't actually ship `tag.write` despite the L brief saying it did). It can run in parallel with CODEX-M; landing it before CODEX-M's manual smoke makes the designer's NumericInput preview meaningful.

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
| CODEX-L | `apps/runtime-web` — load views from gateway, render via component library | codex | *(this commit)* | 2 |

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
