# OpenWebHMI — Architecture

> Status: **draft v0.1** — initial architecture for v1. Subject to revision until a Phase 1 vertical slice is shipped end-to-end. Decisions captured here are *committed*; rationale lives in `wiki/architecture/` (synthesized) and PRs (specific).

## 1. Mission

Open-source SCADA / HMI / MES platform of comparable capability to Inductive Automation **Ignition**, with **FactoryTalk Optix** as a secondary reference. Web-first runtime, cross-platform desktop designer, plant-floor connectivity. Designed so the community can extend drivers, components, and scripting libraries without forking.

**Primary v1 target:** parity with a meaningful subset of Ignition Edge / Standard. **Reference:** Optix (component model, project structure).

## 2. High-level topology

OpenWebHMI follows the **Ignition gateway-centric** model:

- A single **Gateway** (Rust binary) is the deployment unit. It owns the project, the tags, the drivers, the historian, the alarm engine, the scripting host, and authentication.
- Many **Designer** clients (Tauri desktop app, Win + Mac) connect to a gateway to author projects.
- Many **HMI Runtime** clients (browser) connect to the same gateway to render projects with live tag data.
- **Local mode** = one gateway + one client co-located on a single machine. Same code path as multi-client; just one process per role.

```
       ┌──────────────────────┐
       │ Designer (Tauri)     │── WebSocket ──┐
       │ React UI             │               │
       └──────────────────────┘               │
                                              ▼
       ┌──────────────────────┐         ┌─────────────────────────────────┐
       │ HMI Runtime          │── WS ──▶│         Gateway (Rust)          │
       │ (Browser, React)     │         │                                 │
       └──────────────────────┘         │  ┌───────────────────────────┐  │
                                        │  │ Tag Engine (in-memory)    │  │
       ┌──────────────────────┐         │  │ Pub/Sub                   │  │
       │ HMI Runtime …        │── WS ──▶│  ├───────────────────────────┤  │
       └──────────────────────┘         │  │ Driver Runtime            │  │
                                        │  │ (in-process plugins)      │  │
                                        │  ├───────────────────────────┤  │
                                        │  │ Alarm Engine              │  │
                                        │  ├───────────────────────────┤  │
                                        │  │ Historian (SQLite v1)     │  │
                                        │  ├───────────────────────────┤  │
                                        │  │ Scripting Host            │  │
                                        │  │ (Python via PyO3, in      │  │
                                        │  │  worker subprocesses)     │  │
                                        │  ├───────────────────────────┤  │
                                        │  │ Project Store             │  │
                                        │  ├───────────────────────────┤  │
                                        │  │ Auth / Sessions           │  │
                                        │  └───────────────────────────┘  │
                                        │           │            │        │
                                        │           ▼            ▼        │
                                        │   ┌──────────────┐  ┌────────┐  │
                                        │   │  Drivers     │  │ SQLite │  │
                                        │   │  rockwell,   │  │ files  │  │
                                        │   │  opc-ua, …   │  │        │  │
                                        │   └──────┬───────┘  └────────┘  │
                                        └──────────┼──────────────────────┘
                                                   ▼
                                              ┌─────────┐
                                              │  PLCs   │
                                              └─────────┘
```

## 3. Technology stack

| Layer | Tech | Rationale |
|---|---|---|
| Gateway runtime | **Rust** + Tokio | Single static binary, predictable latency for tag I/O, zero-GC, FFI-friendly. |
| Drivers | **Rust** (in-process plugins) | Tightest possible coupling to tag engine; reuse vendor crates (`rust-ethernet-ip`, `tokio-modbus`, etc.). |
| Scripting | **Python** via PyO3, run in worker subprocesses | Familiar to plant engineers (Ignition uses Jython); subprocess isolation sidesteps GIL contention and survives script crashes. |
| Wire protocol | **JSON over WebSocket** (v1) | Single duplex stream per client; debuggable; trivial TS interop. MessagePack/CBOR is a future optimization. |
| Designer / IDE | **Tauri** (Rust shell) + **React + TS** | Native binary, small footprint vs Electron, ships on Win + Mac. React for ecosystem (Monaco, react-konva, etc.). |
| HMI runtime | **React + TS** in a browser | Single rendering surface for v1; web-only is dramatically simpler than Vision-style desktop client. |
| Component library | **React** components, schema-driven | Same components used by designer (with adornments) and runtime (live). |
| Persistence | **SQLite** for project store, historian, auth | Zero-ops, embedded, single-file backups. Pluggable backend (Postgres + Timescale) deferred to v2. |
| Plugin distribution | crates.io (drivers), npm (components), Python packages (script libs) | Standard ecosystems; no custom registry. |

**Languages explicitly excluded for v1:** Go, C#, C++. Adding them increases contributor friction without unlocking capability that Rust + TS + Python don't already cover. Revisit only if a specific module hits a hard limit in Rust.

## 4. Components

### 4.1 Gateway (`crates/gateway`)

A single Rust binary. Boot sequence:

1. Load `gateway.toml` config (bind address, TLS cert paths, project store path, log level).
2. Open SQLite handles for project store, historian, auth.
3. Initialize tag engine.
4. Discover and load drivers per project config.
5. Load enabled projects; instantiate views, alarms, scripts.
6. Bind WebSocket + HTTP listeners.
7. Begin serving clients.

**Crash boundary.** A panic in the gateway tears down the process. Project store and historian use SQLite write-ahead logging so an in-flight write isn't silently corrupted. **Containment is asymmetric**: script failures are reliably isolated because scripts run in worker subprocesses (see §4.7); driver failures are only partially isolated, because drivers run in-process. Rust driver-task panics under `panic = "unwind"` are supervised and restarted (see §4.4), but native faults, FFI crashes, undefined behavior in `unsafe` code, `panic = "abort"` builds, and resource exhaustion can still terminate the gateway. Whole-process crash recovery is therefore an OS-level concern (systemd, Docker `restart: always`, Kubernetes liveness probe), not an in-process guarantee.

**Process model.** Single OS process. Tokio multi-thread runtime. Drivers run on the same runtime. Scripts run in **separate worker subprocesses** managed by the scripting host.

### 4.2 Tag Engine (`crates/tag-engine`)

The single source of truth for live tag values inside the gateway.

- **Tag identity:** `{provider}/{path/with/slashes}`. Providers: `rockwell-1`, `opc-1`, `memory`, `system`, etc.
- **Value envelope:** `{ value: TagValue, quality: Quality, timestamp: u64 }` — mirrors OPC UA semantics.
- **Quality enum:** `Good | Bad(reason) | Uncertain(reason) | Stale`.
- **Data types:** `Bool, Int8/16/32/64, Uint8/16/32/64, Real32, Real64, String, Bytes, Array<T>, Struct(UDT)`.
- **Subscriptions:** clients (and internal consumers like alarm engine, historian, scripts) subscribe to tag paths or globs (`rockwell-1/Line1/*`). Engine publishes on change (report-by-exception).
- **Sources of values:**
  - **Driver tags** — backed by a driver instance, polled or subscribed via the driver's protocol.
  - **Memory tags** — gateway-owned, settable from scripts or operator inputs.
  - **Expression tags** — derived (`a + b`, `now()`, etc.); recomputed when inputs change.
  - **Reference tags** — alias to another tag.

The tag engine is **not** the historian. History is written by a separate component subscribing to the engine.

### 4.3 Protocol (`crates/protocol` + `packages/protocol-ts`)

Wire schema shared between gateway and TS clients. Single source of truth in Rust; TS types generated via `ts-rs` (or `specta`).

Message kinds (illustrative, finalized in Phase 0):

```
client → gateway:
  auth.login           { username, password }
  project.subscribe    { projectId }
  view.open            { viewId }
  tag.subscribe        { paths: [...] }
  tag.unsubscribe      { paths: [...] }
  tag.write            { path, value }
  alarm.ack            { alarmId, note }
  script.invoke        { scriptId, args }

gateway → client:
  auth.result          { sessionId, roles }
  project.changed      { projectId, version }
  view.definition      { viewId, tree }
  tag.update           { path, value, quality, ts }
  alarm.event          { alarmId, kind, ... }
  script.result        { invocationId, result }
  error                { code, message }
```

Single duplex WebSocket per client. Messages are JSON envelopes: `{ id?, kind, payload }`. Request/response correlation via `id`.

### 4.4 Driver model (`crates/driver-api` + `crates/driver-*`)

Drivers are **in-process Rust crates** implementing a stable trait:

```rust
#[async_trait]
pub trait Driver: Send + Sync {
    fn metadata(&self) -> DriverMetadata;
    async fn connect(&mut self, config: serde_json::Value) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;

    async fn browse(&self, path: Option<&str>) -> Result<Vec<TagNode>>;

    async fn read(&self, address: &TagAddress) -> Result<TagValue>;
    async fn write(&self, address: &TagAddress, value: TagValue) -> Result<()>;

    /// Optional: drivers that support native subscription override this.
    async fn subscribe(&mut self, addresses: Vec<TagAddress>)
        -> Result<BoxStream<'static, TagUpdate>>;
}
```

**First driver: `driver-rockwell`** wraps the `rust-ethernet-ip` crate (v0.7+ on crates.io). It exposes the crate's `EipClient`, `RoutePath`, `PlcValue`, and `*_tag_group` subscription API behind the `Driver` trait. Upgrades are a version bump in `Cargo.toml`; integration nuances and known limitations live in `wiki/drivers/rust-ethernet-ip-integration.md`.

**Driver containment (and what containment cannot give us).** Drivers run as in-process trusted code — they are not sandboxed (they need raw socket access). The supervisor catches **Rust panics in driver tasks** under the default `panic = "unwind"` runtime: each driver instance runs inside a `tokio::spawn` task with `catch_unwind`-style supervision; on panic the driver is marked `Faulted`, all its tags transition to `Bad(driver_faulted)`, and the supervisor attempts restart with exponential backoff.

The supervisor **does not** protect against:
- Native crashes — `SIGSEGV`, `SIGBUS`, `SIGABRT` from FFI dependencies or C extensions.
- Builds compiled with `panic = "abort"` (`unwind` is required for in-process recovery).
- Undefined behavior in `unsafe` blocks (memory corruption can survive past the unwind point).
- Stack overflow.
- Resource exhaustion (file descriptor leaks, unbounded memory growth).

Any of those will terminate the gateway process. **In-process restart-on-Rust-panic is a best-effort mitigation, not a containment guarantee.** Production deployments must rely on an OS-level supervisor (systemd, Docker `restart: always`, Kubernetes liveness probe) for whole-process crash recovery. *Community drivers must be vetted for memory safety, FFI hygiene, and resource discipline before being recommended in the registry.*

**Future drivers (Phase 4+):** OPC UA, Modbus TCP, MQTT (Sparkplug B), Siemens S7, BACnet, Ethernet/IP for non-Rockwell devices.

### 4.5 Alarm Engine (`crates/alarm-engine`)

- Alarm definitions are tag-bound: `tag`, `condition` (high, low, deviation, equality, digital, etc.), `priority`, `messages`, `setpoint(s)`.
- State machine: `Clear → Active → Acked → Cleared` (Ignition convention). Optional `Shelved` state.
- Subscribes to tag engine; transitions on edge.
- Persists alarm history to SQLite.
- Notifications (email, webhook, SMS, MQTT publish) deferred to Phase 3+.

### 4.6 Historian (`crates/historian`)

- Subscribes to a configurable subset of tags ("logged" tags).
- Writes `(tag_id, ts, value, quality)` rows to SQLite, indexed by `(tag_id, ts)`.
- Read API: `read_history(tag, start, end, aggregation, max_points)` — aggregations: `raw, avg, min, max, sum, count, first, last`.
- Downsampling: query-time only in v1. Continuous downsampling tables in v2.
- v2 backend: pluggable (Timescale, Influx, DuckDB).

### 4.7 Scripting Host (`crates/scripting`)

- Embeds **CPython** via PyO3 in a pool of **worker subprocesses**.
- Why subprocesses, not in-process: GIL contention would bottleneck many concurrent scripts; a script that segfaults a C extension would otherwise crash the gateway. Subprocess restart is cheap and keeps the gateway alive.
- IPC: stdin/stdout JSON-RPC between gateway and worker.
- Triggers: `on_tag_change(tag)`, `on_timer(interval)`, `on_alarm(alarm)`, `on_button_click(button)`, `on_view_open(view)`, `on_view_close(view)`.
- Standard library exposed to scripts:
  ```python
  system.tag.read(path)        # → TagValue
  system.tag.write(path, val)
  system.tag.subscribe(path, callback)
  system.alarm.ack(alarm_id, note)
  system.db.query(sql, params) # gateway-managed connection pool
  system.http.get/post(...)
  system.util.now() / .log() / .send_message()
  ```
- Script source lives in the project; runs with the project's permissions, not the operator's.
- Resource limits per script: CPU ms budget, memory ceiling, network egress allowlist (Phase 3 hardening).

### 4.8 Project Store (`crates/project-store`)

- A **project** is the unit of authoring and deployment.
- Contents: views (HMI screen tree), tag definitions, alarm configs, driver configs, scripts, themes, assets (images, fonts).
- On-disk format: a directory of canonical JSON files (one per artifact) under `<project-store-root>/<project-id>/`. Diff-friendly. Optionally `git init` for project-level history.
- Indexed by SQLite for fast metadata lookup.
- Versioning: every save increments project version; clients receive `project.changed` and reload affected views.
- Hot reload: granular by artifact (changing one view does not reload the whole project).
- Export/import: `.owhmi` archive (zipped project directory) for moving between gateways.

### 4.9 Auth (`crates/auth`)

- v1: username/password (bcrypt), session tokens (JWT), 4 built-in roles: `Administrator, Designer, Operator, Viewer`.
- Per-project role bindings and per-view ACLs.
- TLS-only on production deployments (TLS terminated by gateway via `rustls`).
- v2+: OIDC / SSO, AD/LDAP, audit log export.

### 4.10 Designer / IDE (`apps/designer`)

Tauri shell (Rust) + React UI (TS). Cross-platform: Windows + macOS.

Modules:
- **Connection** — pick gateway, log in.
- **Project explorer** — tree of views, tags, alarms, scripts, drivers.
- **Tag browser** — view + edit tag definitions; browse PLC tags via `Driver::browse()` and bulk-import.
- **Visual canvas** — drag/drop components, property panel, tag binding picker. Backed by react-konva (canvas) for v2; Phase 1 ships a form-based bindings UI without canvas.
- **Script editor** — Monaco with Python syntax + intellisense for `system.*` exposed via TS-generated stubs.
- **Alarm config**, **driver config**, **theme editor**.
- **Run / preview** — opens an embedded webview pointing at a test runtime.

The designer talks to the gateway over the **same WebSocket protocol** as the runtime, just with `Designer` role privileges (write-project, run-tests, etc.).

### 4.11 HMI Runtime (`apps/runtime-web`)

A React SPA loaded from the gateway HTTP endpoint. On boot:
1. Auth (login form or session cookie).
2. Fetch current project metadata (active version).
3. Open WebSocket; subscribe to current view.
4. Render component tree; subscribe to bound tags.
5. As user navigates, mount/unmount views → subscribe/unsubscribe tags.

Operator inputs (button click, value entry) become `tag.write` messages; gateway authorizes against role + view ACL.

### 4.12 Component Library (`packages/component-library`)

- Standard React components: `Label, Button, ValueDisplay, Indicator, Gauge, Trend, AlarmTable, NumericInput, Image, Container, ToggleSwitch`, etc.
- Each component declares: `propsSchema` (JSON schema, drives the property panel), `defaultProps`, `render` (runtime), `designerRender` (with adornments), `bindableProps` (which props accept tag bindings).
- Plugin packages publish additional components; designer discovers them via package manifest.

## 5. Data flow

### 5.1 Live tag read (PLC → screen)

```
PLC ─ EtherNet/IP ─▶ driver-rockwell ─▶ TagEngine.publish(path, value, quality, ts)
                                          │
                                          ├─▶ Historian (logs if tag is logged)
                                          ├─▶ AlarmEngine (evaluates conditions)
                                          ├─▶ Scripting (fires on_tag_change handlers)
                                          └─▶ WS subscribers (HMI clients) ─▶ React render
```

### 5.2 Operator write (screen → PLC)

```
Click ─▶ tag.write WS message ─▶ Gateway authorizes (role + ACL)
                                  │
                                  ▼
                              TagEngine routes to driver
                                  │
                                  ▼
                          driver-rockwell writes via EtherNet/IP
                                  │
                                  ▼
                          PLC echoes / next poll → §5.1 propagates new value
```

### 5.3 Project change (designer → all clients)

```
Designer save ─▶ project.save WS ─▶ ProjectStore.commit(artifact)
                                     │
                                     ├─▶ version++
                                     └─▶ broadcast project.changed
                                          │
                                          ▼
                                  HMI clients reload affected views
```

## 6. Failure modes & resilience

| Failure | Behavior |
|---|---|
| Driver disconnect | Driver reports `Bad(disconnected)` quality on its tags. Supervisor reconnects with exponential backoff. HMI shows bad-quality visual. |
| PLC slow / busy | Driver retries per its config; tag quality → `Uncertain(timeout)`. |
| Driver Rust panic (`panic = "unwind"`) | Supervisor catches, marks driver `Faulted`, restarts with backoff. Gateway stays up. |
| Driver native fault (segfault, FFI abort, `unsafe` UB, `panic = "abort"` build) | Gateway process terminates. Recovery is the OS-level supervisor's job (systemd, Docker, k8s). On restart, clients reconnect and resubscribe. |
| Script error | Caught by worker; written to script log; tag value unaffected. Worker process restarts on segfault. |
| Gateway crash | Clients reconnect with backoff and resubscribe. SQLite WAL guarantees no torn writes. |
| Client disconnect | Gateway drops subscriptions; tag engine reduces driver poll set if no other subscribers. |
| Project corruption | Project store keeps the last-known-good version; corrupted save is rejected before `project.changed` fires. |

## 7. Security boundaries

- **Gateway is the trust boundary.** All PLC writes go through gateway authorization — no client talks to a PLC directly.
- **Drivers are trusted code** (in-process). Community drivers must be vetted before being recommended.
- **Scripts are semi-trusted.** Run in worker subprocesses; resource limits (CPU, memory, network) enforced by the host. They have full `system.*` access — they're authored by the project owner, not arbitrary users.
- **Operators are untrusted.** Authenticated; every write checked against role + view ACL.
- **Wire transport:** TLS via rustls in production. WSS + HTTPS only.

## 8. Out of scope for v1.0

- Mobile apps / native iOS/Android runtime
- Redundant / failover gateways
- Cluster / horizontal scale (single gateway only)
- Loads above ~5,000 live tags (target; will validate)
- MES-grade workflow engine (recipes, batch, OEE — Phase 5)
- Vision-style desktop runtime client (web-only in v1)
- Real-time control loops (we do supervisory control, not deterministic real-time)
- Safety-rated functions (we are *not* IEC 61508 / SIL-rated)

## 9. Open architectural questions

Tracked in `wiki/architecture/` as they arise. Initial list:

- Tag-write authorization: per-tag ACLs vs view-only ACLs.
- Project versioning: own metadata vs delegate to git from day one.
- Component plugin trust: signed packages vs informal allowlist.
- Designer ↔ gateway disconnected editing: should the designer support local-only project editing without a live gateway?
- Multi-tenant gateway: should one gateway host multiple isolated projects, or is one-gateway-per-project the model?

## 10. Naming

- Repo & project: **OpenWebHMI**.
- Gateway binary: `owhmi-gateway`.
- Designer app: `OpenWebHMI Designer`.
- File extension for project archive: `.owhmi`.

These can change before 1.0 if a better identity emerges.

---

See also: [`docs/roadmap.md`](roadmap.md), [`docs/feature-matrix.md`](feature-matrix.md), [`docs/contributing.md`](contributing.md), [`wiki/`](../wiki/).
