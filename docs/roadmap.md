# OpenWebHMI — Roadmap

> Phased plan to **OpenWebHMI 1.0**: an Ignition-class open-source SCADA/HMI platform. Each phase has a single, testable exit criterion. **Until physical PLC hardware is available to the project, phases 1 through 3 exit on a documented EtherNet/IP simulator** (see Phase 1). Real-hardware validation is a hard gate before tagging 1.0 — see [Real-hardware validation gate](#real-hardware-validation-gate-pre-10).

This is a **plan**, not a contract. Sequencing reflects what unlocks the next phase, not what would feel completion-shaped to ship in isolation. See [`docs/architecture.md`](architecture.md) for the system being built.

---

## Phase 0 — Foundations (target: ~1 month)

**Goal:** the gateway, designer shell, and runtime client all build, start, and exchange one message end-to-end. No real PLC yet.

Deliverables:
- Cargo workspace populated: `gateway`, `protocol`, `tag-engine`, `driver-api` crates with skeleton APIs and unit tests.
- pnpm workspace populated: `apps/runtime-web`, `apps/designer`, `packages/protocol-ts`, `packages/component-library` with skeleton apps.
- Gateway binary: starts, opens SQLite, binds WebSocket, accepts a connection, responds to a ping.
- TS protocol package: types generated from Rust via `ts-rs`.
- Tag engine: in-memory store, pub/sub, **simulated provider** producing a sin wave + counter at 1Hz.
- Web runtime: connect, log in (hardcoded creds), subscribe to `system/sim/sin`, display value live.
- Designer: Tauri app launches, connects to gateway, shows tag list.
- CI: GitHub Actions running `cargo test`, `cargo clippy`, `pnpm lint`, `pnpm typecheck` on push.

**Exit criterion:** a sin-wave value flows `simulated provider → tag engine → WebSocket → web runtime` and updates in the browser at 1Hz. The Tauri designer launches on Mac and Windows and successfully authenticates.

---

## Phase 1 — Vertical slice: PLC tag in browser (target: ~2 months)

**Goal:** prove the platform end-to-end with one real driver. This is the hardest, most de-risking phase. After this, every later phase is mostly additive.

> **No physical PLC available during this phase.** Phase 1 therefore exits on a **simulated EtherNet/IP target** (a `pylogix`-compatible Python simulator, a recorded session, or `driver-rockwell` pointed at a Rockwell software emulator like Studio 5000 Logix Emulate). The driver code path is the *real* code path — only the wire endpoint is simulated. Bench-PLC validation is a hard gate before 1.0 (see [Phase 4](#phase-4--10-release-target-3-months) and [Real-hardware validation gate](#real-hardware-validation-gate-pre-10)).

Deliverables:
- `driver-api` finalized: `Driver` trait, `TagAddress`, `TagValue`, `Quality`, error types, supervisor.
- `driver-rockwell` crate wrapping `rust-ethernet-ip` (>= 0.7). Implements `connect`, `read`, `write`, `subscribe` (using `subscribe_tag_group`), `browse`. Reconnect-with-backoff.
- Driver supervisor in gateway: spawn, monitor, restart on panic.
- Gateway-side driver config UI in the designer (form-based; no visual canvas yet): host, slot, route path, polling rate.
- Tag binding model: a project with one view containing one `ValueDisplay` component bound to a tag.
- Tag write path: a `Button` component writes a hardcoded value to a tag, gateway authorizes, value reaches the simulated PLC.
- Quality propagation visible in the UI: on simulator disconnect, the value display shows a bad-quality state.
- A standing **EtherNet/IP simulator harness** in `examples/sim-rockwell/` so contributors without hardware can run the full Phase 1 flow.
- `wiki/drivers/rust-ethernet-ip-integration.md` filled in: supported PLCs, data types, known limitations, upgrade workflow, simulator-vs-hardware caveats.

**Exit criterion:** with the simulator running, a tag value updates live in the browser at the configured poll rate. A button click writes back and the simulator confirms the new value. Killing the simulator shows bad quality within 5s; restarting it restores good quality automatically. Demo recorded for the README.

---

## Phase 2 — Designer MVP (target: ~3 months)

**Goal:** stop hand-editing project JSON. A user can build a real screen — first via a structured form-based UI, ideally with a visual canvas if scope allows.

> **Phase 2 is the highest-risk phase.** A drag/drop visual designer is genuinely a small IDE. To de-risk this, deliverables are split into **Required** (must ship to exit Phase 2) and **Stretch** (desirable, but explicitly allowed to slip into Phase 4 without blocking Phase 3 from starting). If everything stretches, Phase 4's "1.0 polish" absorbs the canvas work, and Phase 2 still delivers a usable authoring experience via the form-based UI.

### Required deliverables (must ship to exit Phase 2)

- **Project explorer**: tree of views, tags, alarms, scripts, drivers; create/rename/delete artifacts.
- **Tag browser** wired to live PLC tag introspection (Phase 1's `Driver::browse` if available, otherwise a manual import form).
- **Form-based view editor**: views are authored as structured artifacts (component tree edited via property panels), not by drag/drop. Adding a component is a "+ Add" button that spawns the component with default props at a default position; positioning is via numeric x/y/width/height fields in the property panel.
- **Property panel**: every component prop editable; tag-binding picker for bindable props.
- **Project save/load** to gateway. Versioning: each save bumps version; clients receive `project.changed`; runtime hot-reloads.
- **Component library v1, essentials only (6 components):** `Label, ValueDisplay, NumericInput, Indicator, Image, Container`.
- **Designer preview**: launches an embedded webview against the current saved project.

### Stretch deliverables (allowed to slip into Phase 4 if time-pressured)

- **Visual canvas (react-konva or equivalent)**: drag/drop, select, move, resize, delete.
- **Snap-to-grid**, **undo/redo**, **multi-select** + **align/distribute**.
- **Drag-from-tag-browser-onto-component** binding gesture.
- **4 additional components**: `Button, Rectangle, Line, ToggleSwitch`.
- **Theme editor UI** (the runtime supports themes; the designer authoring UI for them is the stretch).

### Exit criterion (Phase 2)

A user with no JSON editing builds a 5-screen HMI against the **simulated CompactLogix from Phase 1**, including navigation, value displays, manual write buttons, and one indicator showing simulator connection state. The whole project round-trips: save → reopen designer → render in runtime. Whether the user authored the screens via form-based UI or visual canvas does not change the criterion — both produce the same project artifacts.

---

## Phase 3 — Core SCADA features (target: ~3 months)

**Goal:** the platform is now SCADA, not just HMI. Alarms, history, scripting, auth.

Deliverables:
- **Alarm engine:**
  - Alarm config UI (per-tag conditions, priorities, messages).
  - State machine + persistence to SQLite.
  - `AlarmTable` component (live filtering, ack from UI).
  - Acknowledgement workflow + audit fields (who, when, note).
- **Historian:**
  - Per-tag logging config (rate, deadband).
  - SQLite-backed time-series store with `(tag_id, ts)` index.
  - Read API + aggregations (`raw, avg, min, max, count, first, last`).
  - `Trend` component (live + historical, multi-pen, zoom/pan).
- **Auth:**
  - Login UI for runtime; admin UI for users/roles in designer.
  - Roles: `Administrator, Designer, Operator, Viewer`.
  - Per-view ACLs.
  - TLS via rustls (cert paths in gateway config).
- **Scripting (Python via PyO3):**
  - Worker subprocess pool, JSON-RPC IPC.
  - Triggers: `on_tag_change`, `on_timer`, `on_alarm`, `on_button_click`.
  - `system.tag.*`, `system.alarm.*`, `system.db.*`, `system.util.*` libraries.
  - Script editor in designer (Monaco + Python syntax + auto-completion stubs for `system.*`).
  - Per-script resource limits.

**Exit criterion:** the simulator-backed HMI from Phase 2 now (a) raises a high-temperature alarm, (b) trends a process variable for the last 24h, (c) requires login with an `Operator` role to write tags, and (d) runs a Python script that writes a derived setpoint based on two inputs.

---

## Phase 4 — 1.0 release (target: ~3 months)

**Goal:** a small real plant could deploy this. Plugin SDK is real. Docs cover everything a contributor needs. **Picks up any Phase 2 stretch deliverables that slipped** (visual canvas, snap/undo/redo, the remaining 4 components, theme editor) before tagging 1.0.

Deliverables:
- **Second driver: `driver-opcua`** (committed v1 scope). Opens up Siemens, Schneider, Beckhoff, and most modern controllers via OPC UA, and lets us validate the driver-API contract against a fundamentally different protocol from Rockwell's EtherNet/IP. Modbus TCP is **deferred to post-1.0**; the v1 second driver is OPC UA, not "one of".
- **More components** (target: 25+ total): `Gauge, Pie/Bar/Line charts, AlarmBanner, MultiState, ProgressBar, Slider, Dropdown, Tabs, DataGrid`, etc.
- **Plugin SDK** (`packages/sdk` + Rust crate templates):
  - `cargo generate` template for a new driver crate.
  - `pnpm create` template for a new component package.
  - SDK docs page (`wiki/sdk/`).
- **Backup / restore:** project export to `.owhmi` archive; gateway-level backup including historian + alarm history.
- **Audit log:** every project change, alarm ack, manual tag write recorded with `(user, ts, action, target)`.
- **Reporting (lightweight):** scripted report generation to PDF/CSV via Python.
- **Docs site:** `docs.openwebhmi.org` (or repo-wiki for now) covering: install, designer tutorial, scripting tutorial, plugin authoring, deployment.
- **Performance baseline:** documented results for 1k / 5k live tag scenarios.

**Exit criterion:** tagged `1.0.0` release on GitHub. The full Ignition Edge analog: gateway, designer, runtime, alarms, history, scripting, two real drivers, plugin SDK, docs. A community contributor can submit a driver or component without core-team hand-holding.

### Real-hardware validation gate (pre-1.0)

Before tagging `1.0.0`, `driver-rockwell` **must** be validated end-to-end against real Rockwell hardware (CompactLogix or ControlLogix) for at least 24 hours of continuous operation. The validation run records: tag update rate vs configured poll rate, write round-trip latency p50/p95/p99, reconnect behavior across deliberate cable pulls, and any data-type or addressing edge case from `wiki/drivers/rust-ethernet-ip-integration.md` not exercised by the simulator. Findings land in `wiki/releases/1.0.0-validation-synthesis.md`.

If real hardware is still unavailable when the rest of Phase 4 completes, the release is held — we do not ship a 1.0 SCADA platform that has never seen a real PLC.

---

## Phase 5+ — Year 2 horizons (sketched, not committed)

After 1.0, sequencing is community-driven. Likely directions:

- **MES capabilities:** recipe management, batch execution (ISA-88-inspired), OEE calculation, traceability records.
- **Mobile / responsive runtime:** layouts that adapt; PWA for tablet.
- **Redundancy:** primary/backup gateway with state replication.
- **More drivers:** Siemens S7, Sparkplug B / MQTT, BACnet, EtherNet/IP for non-Rockwell, Modbus RTU.
- **Pluggable historian backends:** Timescale, Influx, Parquet/DuckDB.
- **Cloud-managed option:** hosted gateway service for those who don't want self-hosting.
- **AI assistance:** in-designer AI that suggests components, writes scripts, or auto-builds views from a tag list.

---

## Cross-cutting principles

These hold across every phase:

1. **Simulator validation before merge; hardware validation before 1.0.** Driver work merges to `main` after end-to-end validation against a documented simulator (the `examples/sim-rockwell/` harness for the Rockwell driver, equivalent harnesses for future drivers). Real-hardware validation against a physical PLC is a **hard gate before tagging 1.0** — see Phase 4's [Real-hardware validation gate](#real-hardware-validation-gate-pre-10). This rule replaces an earlier "real hardware before merge" stance, which was inconsistent with the simulator-first reality of Phase 1.
2. **Browser-first.** Until 1.0, every UI feature lands in the web runtime first. Desktop runtime is post-1.0.
3. **One process per gateway.** No microservices, no clustering before 1.0.
4. **Two-language rule for plugins.** Drivers are Rust; components are TS; scripts are Python. Don't add a fourth.
5. **Project artifacts are JSON files in a directory.** No proprietary blob format. Git-friendly.
6. **Wiki is updated when behavior is learned, not when a PR ships.** The wiki is for synthesized knowledge — see [`AGENTS.md`](../AGENTS.md).

---

## How phases compose

Each phase **builds on** the previous one without invalidating it. Phase 0's simulated provider continues to be useful for testing through 1.0 (driver-less CI). Phase 1's `driver-rockwell` is the canonical reference implementation that all future drivers (Phase 4+) are measured against. Phase 2's component schema is the contract that Phase 4's plugin SDK formalizes.

We do **not** front-load architecture for hypothetical future needs. If a Phase 5 capability requires reshaping a Phase 1 boundary, we reshape it then, not now.
