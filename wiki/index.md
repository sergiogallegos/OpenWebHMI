# Wiki Index

This file catalogs the current wiki pages. Add a one-line entry when a new page lands; update the line if the page's scope changes.

Status values:
- `seed` — placeholder
- `active` — current best understanding
- `needs-review` — possibly stale
- `historical` — superseded, kept for context

## Core

- [README.md](README.md) — What this wiki is for and how it differs from `docs/` and `README.md`. `active`
- [log.md](log.md) — Append-only chronological activity log. `active`
- Development environment — The workspace pins Rust 1.95.0 via `rust-toolchain.toml` and declares Rust 1.85 as the minimum supported toolchain.

## Architecture

- [architecture/project-store-on-disk-format.md](architecture/project-store-on-disk-format.md) — Phase 2 project-store directory layout, SQLite metadata role, versioning, and atomic artifact saves. `active`
- [architecture/audit-log.md](architecture/audit-log.md) — SQLite-backed security audit journal, gateway hooks, administrator-only query/subscribe protocol, and remaining scale questions. `active`
- [architecture/backup-restore.md](architecture/backup-restore.md) — `.owhmi` archive core, manifest schema, path traversal checks, historian/alarm SQLite snapshot export/import semantics, audit hooks, and gateway HTTP side-channel flow. `active`

Planned high-value pages:
- `architecture/protocol-evolution.md` — How the WebSocket message schema is versioned. `seed`
- `architecture/tag-id-stability.md` — Tag identity rules and rename behavior. `seed`
- `architecture/scripting-isolation-tradeoffs.md` — Why scripts run in worker subprocesses; what we lose. `seed`

## Drivers

- [drivers/rust-ethernet-ip-integration.md](drivers/rust-ethernet-ip-integration.md) — How `driver-rockwell` wraps the `rust-ethernet-ip` crate, upgrade workflow, supported devices, verified simulator behavior, known limitations. `active`
- [drivers/modbus-integration.md](drivers/modbus-integration.md) — How `driver-modbus` wraps `tokio-modbus` 0.16.1 for Modbus TCP/RTU client mode, address syntax, datatype mapping, simulator validation, and limitations. `active`
- [drivers/opcua-integration.md](drivers/opcua-integration.md) — How `driver-opcua` wraps `async-opcua` 0.18.0 for OPC UA client mode, NodeId syntax, simulator validation, and limitations. `active`
- [drivers/mqtt-integration.md](drivers/mqtt-integration.md) — How `driver-mqtt` wraps `rumqttc` 0.25.1 for generic MQTT and Sparkplug B data-feed mode, alias-map handling, simulator validation, and limitations. `active`
- [drivers/ads-integration.md](drivers/ads-integration.md) — How `driver-ads` uses the Rust `ads` 0.4.4 backend and Windows `TcAdsDll`/TwinCAT-router backend for Beckhoff ADS addresses, AMS configuration, native notifications, symbol metadata, primitive values, and TwinCAT validation. `active`
- [drivers/ads-tls-decision.md](drivers/ads-tls-decision.md) — CODEX-AD decision to support Secure ADS through a Windows `TcAdsDll`/TwinCAT-router backend while keeping `ads-rs` for plain ADS/TCP. `active`
- [drivers/ads-sim-decision.md](drivers/ads-sim-decision.md) — CODEX-AD decision to defer a CI ADS simulator until it can reuse or implement real AMS/ADS frames instead of a fake dialect. `active`

## Designer

*(seed — populated during Phase 2)*

Planned:
- `designer/canvas-library-evaluation.md` — react-konva vs alternatives, decision record. `seed`
- `designer/property-binding-ux.md` — Tag-binding picker design. `seed`

## Runtime

*(seed — populated during Phase 0/1)*

Planned:
- `runtime/component-rendering-perf.md` — Re-render strategy for thousands of subscribed tags. `seed`

## Scripting

*(seed — populated during Phase 3)*

Planned:
- `scripting/worker-subprocess-pool-design.md` — Subprocess lifecycle, IPC, restart policy. `seed`
- `scripting/system-stdlib-surface.md` — `system.*` API contract. `seed`

## Investigations

- [investigations/sim-rockwell-strategy-2026-04-26.md](investigations/sim-rockwell-strategy-2026-04-26.md) — Decision to adapt upstream `rust-ethernet-ip` 0.7.0 `plc_sim` for the Phase 1 Rockwell simulator harness. `active`

Planned:
- `investigations/ignition-feature-deep-dive-<area>.md` — Per-area study of how Ignition implements something. `seed`
- `investigations/optix-information-model-applicability.md` — Whether Optix's Information Model is worth porting concepts from. `seed`

## Releases

*(populated when validation runs occur, one page per release)*

Planned:
- `releases/0.1.0-validation-synthesis.md` — Phase 0 simulator-only validation. `seed`
- `releases/1.0.0-validation-synthesis.md` — Real-hardware gate per `roadmap.md`. `seed`

## Conventions

- Filenames: lowercase-with-dashes; date suffix `-YYYY-MM-DD` when the page is a snapshot in time.
- Add new pages here when they become durable references — not when they're aspirational. Aspirational ideas go in issues.
- Prefer updating an existing page over creating a near-duplicate.
