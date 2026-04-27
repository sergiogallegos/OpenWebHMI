# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 2 — Designer MVP

> Phase 2 is the highest-risk phase per the external-review pass: a drag/drop visual editor is genuinely a small IDE. Phase 2 is split into **Required** (form-based view editor, 6 essentials, no canvas) and **Stretch** (visual canvas + drag/drop). The five tasks below cover the Required slice.

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-I | `crates/project-store` — gateway-side project storage with versioning | codex | open | 2026-04-26 claude | [tasks/CODEX-I-project-store.md](tasks/CODEX-I-project-store.md) |
| CODEX-J | View schema + protocol additions for view-tree authoring | codex | open | 2026-04-26 claude | [tasks/CODEX-J-view-schema.md](tasks/CODEX-J-view-schema.md) |
| CODEX-K | `packages/component-library` — 6 essential components | codex | open | 2026-04-26 claude | [tasks/CODEX-K-component-library-v1.md](tasks/CODEX-K-component-library-v1.md) |
| CODEX-L | `apps/runtime-web` — load views from gateway, render via component library | codex | open (blocked-by I, J, K) | 2026-04-26 claude | [tasks/CODEX-L-runtime-view-loading.md](tasks/CODEX-L-runtime-view-loading.md) |
| CODEX-M | `apps/designer` — Tauri shell, project explorer, form-based view editor | codex | open (blocked-by I, J, K, L) | 2026-04-26 claude | [tasks/CODEX-M-designer-shell.md](tasks/CODEX-M-designer-shell.md) |

### Phase 2 dependency graph

```
CODEX-I  (project-store)        no blockers — start anytime
CODEX-J  (view schema)          no blockers — start anytime
CODEX-K  (component library)    no blockers — start anytime
CODEX-L  (runtime view loading) blocked-by I, J, K
CODEX-M  (designer)             blocked-by I, J, K, L (exit criterion)
```

I, J, and K can start in parallel. L starts when all three land. M starts when L lands and is the Phase 2 exit demo.

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

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
