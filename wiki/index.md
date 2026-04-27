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

## Architecture

- [architecture/project-store-on-disk-format.md](architecture/project-store-on-disk-format.md) — Phase 2 project-store directory layout, SQLite metadata role, versioning, and atomic artifact saves. `active`

Planned high-value pages:
- `architecture/protocol-evolution.md` — How the WebSocket message schema is versioned. `seed`
- `architecture/tag-id-stability.md` — Tag identity rules and rename behavior. `seed`
- `architecture/scripting-isolation-tradeoffs.md` — Why scripts run in worker subprocesses; what we lose. `seed`

## Drivers

- [drivers/rust-ethernet-ip-integration.md](drivers/rust-ethernet-ip-integration.md) — How `driver-rockwell` wraps the `rust-ethernet-ip` crate, upgrade workflow, supported devices, verified simulator behavior, known limitations. `active`

Planned:
- `drivers/opcua-integration.md` — Once Phase 4 OPC UA work begins. `seed`

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
