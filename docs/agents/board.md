# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 4 — 1.0 release (in progress)

**Driver scope expansion (2026-04-30).** Phase 4 now ships **four new drivers** (in addition to Rockwell from Phase 1): OPC UA, Modbus TCP/RTU, MQTT (incl. Sparkplug B), and Beckhoff ADS. Each lands as a real driver crate + simulator harness + simulator-driven CI integration tests + wiki entry + designer manual smoke step. Real-hardware validation remains the pre-1.0 gate. Drivers are independent — they can run in parallel.

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-AJ | Async hygiene — blocking SQLite + coordinated shutdown + scripting fan-in | codex | open | 2026-05-05 claude | [`CODEX-AJ-async-hygiene.md`](tasks/CODEX-AJ-async-hygiene.md) |

### Phase 4 dependency graph

```
(v1.0 feature ladder complete — AE/AF/AI/AG/AD/AH/AC merged)
        │
        └── quality follow-ups (review pass 2 — Rust 1.95 idioms vs tokio/axum/ripgrep)
                ├── CODEX-AJ  async hygiene (Tier 1: blocking SQLite, shutdown coord, scripting fan-in)  ← open
                ├── CODEX-AK  tokio handle ergonomics (JoinSet inside subsystems, #[must_use], cancel-safety docs)
                ├── CODEX-AL  API polish (newtypes, builder/Config split, #[non_exhaustive], cheap-clone handles)
                └── CODEX-AM  stdlib + deps modernization (thiserror 2.x, format-capture, missing-docs consistency)
```

**🎉 Phase 4 v1.0 feature ladder complete.** AE (audit log), AF (backup/restore), AI (historian import closeout) all merged. Driver slice was AG (toolchain) → AD (ADS validation) → AH (ADS native notifications). Component slice closed at AC (25 components). Remaining v1.0 closeout items live outside the agent-task ladder: plugin SDK, performance baseline, pre-1.0 hardware-validation 24h soak gate, plus the v1.1 polish list flagged across AE/AF/AI verdicts (SessionExpired hook, wiki/protocol/* pages, SQL-side audit-log filter pushdown, alarm-journal merge dedupe, designer session disconnect on replace).

**Quality sweep (review pass 2 — Rust 1.95 + edition 2024 idioms vs tokio/axum/ripgrep).** CODEX-AG was the mechanical edition migration; this is the deferred idiom modernization, sliced four ways. CODEX-AJ (Tier 1, runtime-health gaps) leads; AK/AL/AM open after AJ merges to avoid blurring correctness fixes with surface cleanup.

## Phase 3 — Core SCADA features

**🎉 Phase 3 code-complete.** All seven tasks merged (O, Q, S, R, P, T, U) plus the V closeout follow-up. The demo HMI now has the full SCADA stack: alarms + history + auth + alarm UI + trends + scripting + Monaco script editor with live error surfacing. Awaiting **manual-smoke validation** of the full 19-step checklist in [`apps/designer/README.md`](../../apps/designer/README.md) (covers Phase 2 and Phase 3 together) before tagging `v0.3.0`.

*(no open Phase 3 tasks)*

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
| CODEX-R | `AlarmTable` component + designer alarm config | codex | `29be0e9` | 3 |
| CODEX-P | `Trend` component — multi-pen historical + live chart | codex | `151afdb` | 3 |
| CODEX-T | `crates/scripting` — CPython 3.11+ host + worker subprocesses + system.* RPC | codex | `f8b74a9` | 3 |
| CODEX-V | Route `system.tag.write` through per-driver write queue | codex | `9db307e` | 3 |
| CODEX-U | Designer script editor — Monaco + Python syntax + system.* stubs | codex | `cfa1cc0` | 3 |
| CODEX-X | `crates/driver-modbus` — Modbus TCP + RTU client driver | codex | `16f11bb` | 4 |
| CODEX-W | `crates/driver-opcua` — OPC UA client driver | codex | `6a8c2e2` | 4 |
| CODEX-Y | `crates/driver-mqtt` — MQTT (generic + Sparkplug B) driver | codex | `6a8c2e2` | 4 |
| CODEX-AA | Close v1 scope gaps in `driver-mqtt` (TLS + WS + json_path + binary BE) | codex | `560227e` | 4 |
| CODEX-AB | Component library batch 2 — 8 new components (Gauge, ProgressBar, Slider, Dropdown, ToggleSwitch, Button, MultiState, AlarmBanner) | codex | `3e88046` | 4 |
| CODEX-AC | Component library batch 3 — 9 new components (Tabs, Modal, DataGrid, BarChart, PieChart, Card, Spinner, Divider, Stepper); v1 25-component target met | codex | `6bff505` | 4 |
| CODEX-Z | `crates/driver-ads` — Beckhoff TwinCAT (ADS) client driver; implementation-merged but not production-validated (see CODEX-AD) | codex | `f34ebd6` | 4 |
| CODEX-AG | Workspace toolchain modernization — pin Rust 1.95.0 + edition 2024 | codex | `9a96871` | 4 |
| CODEX-AD | ADS validation hardening — TwinCAT-router FFI backend + hardware-validated against CX-23F092 | codex | `54ec808` | 4 |
| CODEX-AH | Native ADS device notifications via TcAdsDll FFI — closes AD's polling regression | codex | `2be075e` | 4 |
| CODEX-AE | `crates/audit-log` — SQLite security event journal + query/subscribe wire protocol + gateway hooks | codex | `83aac91` | 4 |
| CODEX-AF | `crates/backup` — project export/import + gateway HTTP side-channel; historian/alarm import deferred to CODEX-AI | codex | `83aac91` | 4 |
| CODEX-AI | Historian + alarm-journal import path in `crates/backup` — closes AF's v1.0 gap | codex | `79e0ec4` | 4 |

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
