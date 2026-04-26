# OpenWebHMI — Feature Matrix

> Side-by-side feature catalog of **Inductive Automation Ignition** (primary reference) and **FactoryTalk Optix** (secondary reference) against the OpenWebHMI plan.
>
> **Status legend** for OpenWebHMI:
> - 🟢 **v1** — committed for 1.0 release (Phases 0–4, see [`roadmap.md`](roadmap.md))
> - 🟡 **post-1.0** — planned but after 1.0 (Phase 5+)
> - 🔵 **maybe** — community-driven; if someone wants it, we'd accept a PR
> - ⚪ **not planned** — explicitly out of scope; safety, regulatory, or strategic reasons
>
> "Ignition" / "Optix" columns: ✅ = ships in product, ➕ = ships in extension/module, ❌ = not available, ❔ = uncertain.
>
> This matrix is the **scope contract** for 1.0. If a feature is marked v1 here and slips, it's a roadmap conversation, not a quiet drop.

---

## 1. Connectivity (drivers)

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Rockwell EtherNet/IP (CompactLogix, ControlLogix) | ✅ | ✅ | 🟢 v1 (Phase 1, via `rust-ethernet-ip`) |
| Rockwell Micro800 / Micrologix | ✅ | ❔ | 🟡 post-1.0 (depends on `rust-ethernet-ip` support) |
| Siemens S7 (S7-300/400/1200/1500) | ✅ | ✅ | 🟡 post-1.0 |
| Modbus TCP | ✅ | ✅ | 🟡 post-1.0 |
| Modbus RTU (serial) | ✅ | ✅ | 🟡 post-1.0 |
| OPC UA client | ✅ | ✅ | 🟢 v1 (Phase 4) |
| OPC UA server (expose gateway tags) | ✅ | ✅ | 🟡 post-1.0 |
| OPC DA client (legacy) | ➕ | ❌ | ⚪ not planned |
| Omron FINS / EtherNet/IP | ➕ | ❔ | 🔵 maybe |
| BACnet/IP | ➕ | ❔ | 🔵 maybe (HVAC adjacency) |
| MQTT (generic) | ➕ | ✅ | 🟡 post-1.0 |
| MQTT Sparkplug B | ➕ (Cirrus Link) | ✅ | 🟡 post-1.0 |
| DNP3 (utility / SCADA) | ➕ | ❌ | ⚪ not planned |
| ASCII / custom serial | ➕ | ✅ | 🔵 maybe |
| Beckhoff TwinCAT (ADS) | ➕ | ❔ | 🔵 maybe |
| GE / Emerson DeltaV | ➕ | ❔ | 🔵 maybe |
| Built-in driver SDK for community drivers | ✅ (Module SDK) | ✅ | 🟢 v1 (Phase 4) |

## 2. Tags

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Tag providers (multiple, scoped) | ✅ | ❔ | 🟢 v1 |
| OPC tag (driven by driver) | ✅ | ✅ | 🟢 v1 |
| Memory tag (gateway-owned, settable) | ✅ | ✅ | 🟢 v1 |
| Expression tag (derived) | ✅ | ✅ | 🟢 v1 (Phase 3) |
| Reference tag (alias) | ✅ | ❔ | 🟢 v1 |
| Query tag (DB-backed) | ✅ | ❔ | 🟡 post-1.0 |
| User-Defined Types (UDTs / Object Types) | ✅ | ✅ (Information Model) | 🟢 v1 |
| Tag history (per-tag logging config) | ✅ | ✅ | 🟢 v1 (Phase 3) |
| Tag-level alarms (built into the tag) | ✅ | ✅ | 🟢 v1 (Phase 3) |
| Tag scaling / engineering units | ✅ | ✅ | 🟢 v1 |
| Tag deadband (report-by-exception threshold) | ✅ | ✅ | 🟢 v1 |
| Quality propagation (Good/Bad/Uncertain) | ✅ | ✅ | 🟢 v1 |
| Tag tree / browse | ✅ | ✅ | 🟢 v1 |
| Bulk tag editor | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Tag import/export (CSV/JSON) | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Tag groups (poll rate scheduling) | ✅ | ✅ | 🟢 v1 |

## 3. HMI / visualization

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Web runtime (browser, no install) | ✅ Perspective | ✅ WebPresentation | 🟢 v1 |
| Desktop runtime (native window) | ✅ Vision (Java) | ✅ | 🟡 post-1.0 (Tauri-based, reuses web runtime) |
| Mobile-responsive layouts | ✅ | ✅ | 🟡 post-1.0 |
| Native mobile apps (iOS/Android) | ✅ Perspective app | ❔ | 🔵 maybe (PWA likely sufficient) |
| Drag/drop visual designer | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Component property panel | ✅ | ✅ | 🟢 v1 |
| Tag-to-prop binding picker | ✅ | ✅ | 🟢 v1 |
| Expression bindings | ✅ | ✅ | 🟢 v1 (Phase 3) |
| Themes (project-level) | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Styles / style classes | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Templates / reusable view fragments | ✅ Templates / Embedded views | ✅ Widgets | 🟢 v1 (Phase 2) |
| Component library (built-in) | ✅ (large) | ✅ (large) | 🟢 v1 — 25+ components target |
| Custom component plugins | ✅ Module SDK | ✅ | 🟢 v1 (Phase 4 SDK) |
| Embedded views / containers | ✅ | ✅ | 🟢 v1 |
| Multi-monitor / multi-window | ✅ Vision | ✅ | 🟡 post-1.0 |
| Localization (multi-language) | ✅ | ✅ | 🟡 post-1.0 |
| Right-to-left support | ✅ | ❔ | 🟡 post-1.0 |
| Animations / transitions | ✅ | ✅ | 🟢 v1 (basic) |
| SVG / vector graphics | ✅ | ✅ | 🟢 v1 |
| Custom CSS injection | ✅ | ❔ | 🟢 v1 |

## 4. Alarms

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Tag-level alarm definitions | ✅ | ✅ | 🟢 v1 (Phase 3) |
| State machine (active / acked / cleared / shelved) | ✅ | ✅ | 🟢 v1 |
| Priority / severity | ✅ | ✅ | 🟢 v1 |
| Alarm journal (history) | ✅ | ✅ | 🟢 v1 |
| AlarmTable / live alarm view | ✅ | ✅ | 🟢 v1 |
| Acknowledgement with note | ✅ | ✅ | 🟢 v1 |
| Alarm shelving | ✅ | ✅ | 🟡 post-1.0 |
| Pipeline notifications (email) | ✅ | ✅ | 🟡 post-1.0 |
| Pipeline notifications (SMS, voice) | ✅ (Voice module) | ❔ | 🔵 maybe |
| On-call schedules / escalation | ✅ | ❔ | 🟡 post-1.0 |
| Alarm rosters | ✅ | ❔ | 🟡 post-1.0 |
| Alarm associated data | ✅ | ✅ | 🟢 v1 |

## 5. History / trending

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Tag historian (configurable per-tag) | ✅ | ✅ | 🟢 v1 (Phase 3, SQLite) |
| Multiple history providers / splitter | ✅ | ❔ | 🟡 post-1.0 |
| Live + historical trend chart | ✅ Easy Chart | ✅ | 🟢 v1 |
| Multi-pen trends | ✅ | ✅ | 🟢 v1 |
| Trend zoom / pan / cursor | ✅ | ✅ | 🟢 v1 |
| Aggregations (avg, min, max, etc.) | ✅ | ✅ | 🟢 v1 |
| Continuous downsampling tables | ✅ | ❔ | 🟡 post-1.0 |
| Pluggable backend (Timescale, Influx) | ➕ | ❔ | 🟡 post-1.0 |
| History export (CSV, Parquet) | ✅ | ❔ | 🟡 post-1.0 |

## 6. Scripting

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Embedded scripting language | ✅ Jython 2.7 | ✅ C# (NetLogic) + JS | 🟢 v1 — **CPython 3.11+** via PyO3 |
| Gateway-scope scripts | ✅ | ✅ | 🟢 v1 |
| Client / view-scope scripts | ✅ | ✅ | 🟢 v1 |
| Tag-change event handlers | ✅ | ✅ | 🟢 v1 |
| Timer-based scripts | ✅ | ✅ | 🟢 v1 |
| Alarm-event scripts | ✅ | ✅ | 🟢 v1 |
| Component-event scripts (button click etc.) | ✅ | ✅ | 🟢 v1 |
| Project-shared library / functions | ✅ | ✅ | 🟢 v1 |
| `system.*` standard library | ✅ extensive | ✅ extensive | 🟢 v1 — `system.tag`, `system.alarm`, `system.db`, `system.http`, `system.util` |
| Script editor with syntax + autocomplete | ✅ | ✅ | 🟢 v1 (Monaco) |
| Script debugger | ✅ | ✅ | 🟡 post-1.0 |
| Scripts in worker subprocesses (crash-isolated) | ❌ in-process Jython | ❔ | 🟢 v1 — **explicit design choice** |
| Script package manager | ❌ | ❔ | 🔵 maybe |

## 7. Database / data

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Connection pool to external DB (Postgres, MySQL, MSSQL) | ✅ | ✅ | 🟡 post-1.0 |
| Named queries (parameterized SQL bindings) | ✅ | ❔ | 🟡 post-1.0 |
| Transaction groups (DB ↔ tag bidirectional sync) | ✅ | ❔ | 🔵 maybe |
| `system.db.query` from scripts | ✅ | ✅ | 🟢 v1 (against gateway DB; external in post-1.0) |

## 8. Security

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Username / password (built-in users) | ✅ | ✅ | 🟢 v1 (Phase 3, bcrypt + JWT) |
| Roles / role-based ACLs | ✅ | ✅ | 🟢 v1 |
| Per-view ACLs | ✅ | ✅ | 🟢 v1 |
| Per-tag ACLs (write authorization) | ✅ | ✅ | 🟡 post-1.0 |
| Active Directory / LDAP | ✅ | ✅ | 🟡 post-1.0 |
| OIDC / SSO | ✅ | ❔ | 🟡 post-1.0 |
| SAML | ✅ | ❔ | 🔵 maybe |
| 2FA / MFA | ✅ | ❔ | 🟡 post-1.0 |
| TLS for all wire traffic | ✅ | ✅ | 🟢 v1 (rustls) |
| Audit log (who did what, when) | ✅ | ✅ | 🟢 v1 (Phase 4) |
| Audit log export | ✅ | ❔ | 🟡 post-1.0 |
| Per-project user isolation | ✅ | ❔ | 🟡 post-1.0 |

## 9. Designer / IDE

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Cross-platform (Win + Mac + Linux) | ✅ Win/Mac/Linux | Win + Linux runtime, Win-only Studio | 🟢 v1 — Win + Mac (Linux post-1.0) |
| Visual canvas | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Project resources tree | ✅ | ✅ | 🟢 v1 |
| Tag browser (live PLC browse) | ✅ | ✅ | 🟢 v1 (Phase 1) |
| Built-in script editor | ✅ | ✅ | 🟢 v1 |
| Multi-developer concurrent editing | ✅ (locking) | ❔ | 🟡 post-1.0 |
| Undo/redo | ✅ | ✅ | 🟢 v1 |
| Project diff / version control friendly | partial | partial | 🟢 v1 — JSON-on-disk, diff-friendly |
| Built-in git integration | ❌ | ❔ | 🔵 maybe |
| Live preview (test in designer) | ✅ | ✅ | 🟢 v1 (Phase 2) |
| Hot reload to running clients | ✅ | ✅ | 🟢 v1 |
| Property bindings UI | ✅ | ✅ | 🟢 v1 |

## 10. Gateway / runtime

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Runs on Win / Mac / Linux | ✅ | ✅ | 🟢 v1 |
| Runs in Docker | ✅ | ✅ | 🟢 v1 |
| Single-binary deployment | ❌ (JVM bundle) | ❌ | 🟢 v1 — single Rust binary + SQLite |
| Backup / restore | ✅ Gateway Backup | ✅ | 🟢 v1 (Phase 4) |
| Project export / import | ✅ | ✅ | 🟢 v1 (`.owhmi` archive) |
| Multiple isolated projects | ✅ | ❔ | 🟡 post-1.0 |
| Redundant / failover gateway | ✅ Standard+ | ✅ | 🟡 post-1.0 |
| Gateway network (gateway-to-gateway) | ✅ | ❔ | 🟡 post-1.0 |
| Modules / extension hot-load | ✅ Module SDK | ✅ | 🟡 post-1.0 (v1 ships compiled-in plugins) |
| Resource quotas (per-project CPU/mem) | ❔ | ❔ | 🟡 post-1.0 |

## 11. Reporting

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Visual report designer | ✅ Reporting module | ✅ | 🟡 post-1.0 |
| Scheduled report generation | ✅ | ✅ | 🟡 post-1.0 |
| Report distribution (email, file, printer) | ✅ | ✅ | 🟡 post-1.0 |
| Scripted report (Python → PDF/CSV) | ✅ | ❔ | 🟢 v1 (Phase 4, basic) |
| Tag-history-based reports | ✅ | ✅ | 🟡 post-1.0 |

## 12. MES / Industry 4.0

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Recipe management | ➕ Sepasoft | ✅ | 🟡 post-1.0 (Phase 5+) |
| OEE calculation | ➕ Sepasoft | ❔ | 🟡 post-1.0 |
| Track and Trace | ➕ Sepasoft | ❔ | 🟡 post-1.0 |
| SPC (Statistical Process Control) | ➕ Sepasoft | ❔ | 🔵 maybe |
| Batch execution (ISA-88) | ➕ Sepasoft | ❔ | 🟡 post-1.0 |
| Predictive maintenance hooks | ❔ | ❔ | 🔵 maybe |
| Energy monitoring | ❔ | ❔ | 🔵 maybe |
| Sequential Function Charts (SFC) | ✅ | ❔ | 🔵 maybe |

## 13. Specialized / niche

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| WebDev (custom HTTP endpoints / REST API) | ✅ WebDev module | ❔ | 🟡 post-1.0 |
| Voice notification module | ✅ | ❔ | 🔵 maybe |
| GIS / map integration | ❌ | ❔ | 🔵 maybe |
| Cloud-managed gateway service | ✅ Cloud Edition | ❔ | 🔵 maybe (Year 2+) |
| Edge gateway / lightweight runtime | ✅ Edge | ✅ | 🟡 post-1.0 |
| Safety-rated functions (SIL) | ❌ | ❌ | ⚪ not planned |
| Hard real-time control | ❌ | ❌ | ⚪ not planned (we are supervisory) |

## 14. Licensing / openness

| Feature | Ignition | Optix | OpenWebHMI |
|---|:-:|:-:|---|
| Open source | ❌ (closed source, free unlimited dev) | ❌ (closed source, free dev) | 🟢 v1 — **MIT** |
| Source-available components | partial | partial | 🟢 v1 — entire stack |
| Free runtime license | ❌ (per-server license) | ➕ tiered | 🟢 v1 — runtime is free |
| Community-publishable plugins | ✅ Exchange | ✅ | 🟢 v1 (Phase 4 SDK) |

---

## 15. Where we are deliberately *different*

We are not trying to be a 1:1 Ignition clone. Some explicit divergences:

| Area | Ignition / Optix | OpenWebHMI choice | Why |
|---|---|---|---|
| Scripting language | Jython 2.7 (Ignition) / C# (Optix) | **CPython 3.11+** | Modern Python, real ecosystem (numpy, pandas), no language EOL exposure. |
| Script isolation | In-process | **Worker subprocesses** | Survives script crashes; sidesteps GIL contention. |
| Project file format | Proprietary blob | **Directory of canonical JSON** | Diff-friendly; works with any VCS. |
| Designer install | Java desktop app | **Tauri (Rust + web)** | Smaller download, faster startup, Mac-native. |
| Runtime install | JVM bundle / .NET | **Single Rust binary + SQLite** | Zero-dependency deploy. |
| Module distribution | Custom format | **crates.io / npm / PyPI** | No custom registry to maintain. |
| Pricing | Per-server license | **Free, MIT-licensed** | Open source. |

---

## 16. How this matrix evolves

When a feature row's status changes (🟡 → 🟢, 🔵 → 🟡, etc.), update this file in the same PR that ships the change, and add a one-line entry to `wiki/log.md`. New feature rows are added when planning a new phase or accepting a community proposal — never silently.
