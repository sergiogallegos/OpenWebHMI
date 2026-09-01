# Manufacturing Platform Plan

> Status: authoritative post-1.0 direction. This document does not add features
> to the v1.0 commitment. `VISION.md` remains authoritative for product
> boundaries, and `docs/roadmap.md` remains authoritative for release order.

OpenWebHMI is a modern open-source HMI/SCADA platform today. Its longer-term
direction is a modular industrial application and manufacturing platform built
on that foundation. A basic deployment must remain a single gateway with tags,
alarms, trends, scripts, and browser views; manufacturing capabilities are
optional modules that consume stable platform services.

Related evidence and delivery plans:

- [`reference-capability-analysis.md`](reference-capability-analysis.md) —
  anonymized source evidence, gap analysis, and adoption matrix.
- [`manufacturing-demo.md`](manufacturing-demo.md) — fictional starter/template
  and deterministic demo design.

## Commitment language

Planning documents use these terms consistently:

| State | Meaning |
|---|---|
| Implemented | Present in code and supported by proportionate verification. |
| Committed | Required for the named release but not necessarily complete. |
| Planned | Approved direction, sequenced after its prerequisites. |
| Exploratory | Worth investigating; no delivery commitment. |
| Deferred | Intentionally postponed until a stated dependency or evidence exists. |
| Community candidate | Suitable for an externally led implementation under platform contracts. |
| Out of scope | Conflicts with product, safety, licensing, or complexity boundaries. |

## Product scales

1. **Simple HMI:** PLC connectivity, tags, alarms, trends, scripts, and browser
   views. No manufacturing domain model or external database is required.
2. **Advanced machine HMI:** role-gated supervisory commands, machine
   diagnostics, recipes, maintenance content, and production counters.
3. **Cell or line monitoring:** equipment hierarchy, line visualization,
   production, downtime, fault analytics, and OEE.
4. **Manufacturing intelligence:** part/process data, SPC, energy, equipment
   health, traceability, and MES integration.

Scale 1 users must not pay the installation, storage, configuration, or UI
complexity cost of Scale 4.

## Capability taxonomy

### Core platform

- **Gateway and project model:** deployment, project artifacts, versioning,
  import/export, backup/restore, diagnostics, and migrations.
- **Connectivity:** drivers, browse/read/write/subscribe contracts, reconnect,
  poll groups, device health, source timestamps, and driver SDK.
- **Tags, expressions, and quality:** live values, aliases, memory values,
  calculations, quality propagation, and engineering metadata.
- **Event and data services:** immutable semantic facts, current-state
  projections, module-owned storage, retention, and reproducible calculations.
- **Historian:** time-series samples, aggregations, retention, downsampling,
  export, and optional future backends.
- **Alarm infrastructure:** definitions, lifecycle, journal, acknowledgment,
  associated data, and later shelving/analytics/notifications.
- **Security, commands, and audit:** identity, role/action/tag authorization,
  confirmations, reason capture, write results, and immutable audit records.
- **Scripting and integration:** isolated Python workers and governed system APIs.
- **Reporting infrastructure:** data sources, definitions, rendering, scheduling,
  distribution, and version metadata.
- **Visualization and Designer:** reusable web components, bindings, layouts,
  navigation, schemas, and designer contribution points.
- **Modules and templates:** installable capability packages, module manifests,
  schemas/migrations/routes/components, and cloneable starter projects.
- **Simulation:** deterministic clocks, scenarios, seeded randomness, explicit
  simulated quality, and reproducible integration-test fixtures.

### Optional manufacturing modules

| Family | Scope |
|---|---|
| Equipment | Enterprise/site/area/line/machine/station/device identity and relationships. |
| Machine operations | Auto/manual/setup patterns, permissives, command/actual state, diagnostics. |
| Production | Counts, targets, takt/cycle, product/model, machine state, timelines, bottlenecks. |
| Downtime and losses | State intervals, reason capture, corrections, policy versions, Pareto. |
| OEE | A/P/Q, boundary counts, targets, planned time, rollups, policy and recalculation. |
| Fault analytics | Fault history, frequency, duration, Pareto, MTBF/MTTR, flood/chatter context. |
| Part/process data | Cycle records, results, measurements, recipes, operator/shift context. |
| Recipes/changeover | Versioned parameter sets, validation, compare, approval, PLC handshake, audit. |
| Maintenance | Runtime/cycle counters, PM schedules, service history, documents, CMMS hooks. |
| Line visualization | Interactive equipment layout, state flow, alarms, quality, drill-down. |
| SPC/quality | Measurement definitions, control charts, capability indices, statistical rules. |
| Energy/sustainability | Utility measurements, state/part normalization, versioned emissions factors. |
| Equipment health | Transparent baselines, EWMA/drift/rate rules, interpretable anomaly evidence. |
| Traceability | Optional identity, station/operation history, components, lots, rework, genealogy. |
| Andon/communication | Calls, escalation, response time, notes, shift handoff, integration hooks. |
| MES integration | Work/order/routing/WIP interfaces and later execution workflows; not ERP. |

## Dependency direction

```text
Drivers
  -> tags + quality + device health
  -> historian + alarm journal + semantic fact services
  -> equipment + manufacturing modules
  -> views + reports + operator workflows
```

Drivers and the tag engine do not depend on production, OEE, downtime, or any
other manufacturing module. A module may consume tags, alarms, history,
security, scripting, and reporting contracts; it must not know how EtherNet/IP,
ADS, OPC UA, Modbus, or MQTT reconnects.

### Feature dependency graph

| Capability | Required first |
|---|---|
| Project templates | Stable project schema, import/export migration, asset packaging. |
| Manufacturing modules | Module manifest/contribution contract and module-owned storage. |
| Equipment model | Stable identifiers, project schema migration, reference integrity. |
| Semantic facts | Equipment identity, quality envelope, idempotency, clock/source metadata. |
| Production | Equipment, counter/state facts, target configuration, projections. |
| State timeline | State facts, interval projection, quality gaps, time-window queries. |
| Fault analytics | Alarm journal/fault mapping, equipment identity, quality gaps. |
| Downtime | State/fault facts, reason workflow, versioned policy, corrections, recalculation. |
| OEE | Production, planned time/targets, downtime policy, boundary definition, quality. |
| Line layout | Equipment identity, runtime navigation, state/quality projections. |
| Recipes | Command gateway, action ACLs, revisions, validation, PLC apply handshake. |
| Part/process data | Part/cycle event, measurement model, module storage. |
| SPC | Measurement model, rational subgroup rules, statistical engine, history. |
| Energy per part | Meter samples, state intervals, production events, model context. |
| Equipment health | Historian, signal definitions, baselines, interpretable rule engine. |
| Traceability | Part identity, operation/station model, immutable part events, search. |
| MES execution | Equipment/product/order models, commands, audit, integration contracts. |

## Reusable data architecture

OpenWebHMI needs a small semantic-fact layer, not a general event-sourcing
framework. Four data shapes remain distinct:

| Shape | Examples | Store/owner |
|---|---|---|
| Live sample | Pressure, mode bit, count value | Tag engine; optional historian sample. |
| Operational fact | State changed, count changed, part completed, model changed | Append-only module fact store. |
| Current projection | Current machine state, active model, latest count | Rebuildable module projection. |
| Derived result | Downtime interval, target attainment, OEE interval | Versioned policy/calculation output. |

Candidate fact types include `MachineStateChanged`, `ProductionCountChanged`,
`PartCompleted`, `RejectRecorded`, `FaultActivated`, `FaultCleared`,
`ModelChanged`, `RecipeChanged`, `ShiftChanged`, `MeasurementRecorded`,
`EnergySampled`, and `DeviceDisconnected`. Fact envelopes require:

- stable event and equipment identifiers;
- source and observation timestamps;
- source quality and diagnostic detail;
- idempotency key where a collector can retry;
- real/simulated origin;
- optional run/shift/product context;
- schema version.

Tag history remains optimized for sampled values. The alarm journal remains the
alarm lifecycle authority. The manufacturing fact store records semantic domain
facts. Projections and derived results can be rebuilt without mutating facts.

### Storage strategy

- SQLite remains the required embedded default.
- Modules own versioned schemas through a platform migration contract; they do
  not edit unrelated core tables.
- A single-machine project can enable manufacturing modules in SQLite without
  installing another service.
- PostgreSQL is a planned optional backend for larger relational/event workloads,
  not a required dependency.
- Timescale, Influx, DuckDB/Parquet, and enterprise historians remain backend or
  archive integrations selected by workload, not a mandatory multi-database stack.
- Reports declare the persisted inputs and policy versions used to reproduce them.

## Equipment hierarchy

The planned common model is:

```text
Enterprise -> Site -> Area -> Line -> Machine -> Station -> Device
```

Projects may start at any level. A single machine project does not need dummy
enterprise/site objects. Every node has a stable id, display name, type, optional
parent, metadata, and tag/fact associations. A station is an internal boundary
inside a machine unless explicitly modeled as a separate physical machine.

Rollups must declare their boundary and aggregation rule. Counts from internal
stations are diagnostic and are never naively summed into machine output.
Overlapping station downtime is unioned at the machine boundary. Line OEE is not
defined until a project selects an explicit line policy; an average of machine
OEE values must be labeled descriptive, not presented as authoritative line OEE.

Adoption is phased:

1. optional equipment artifact and designer tree;
2. machine/station bindings for production, faults, and line layout;
3. area/site rollups only after machine semantics are stable.

## Data-quality contract

Higher-level values carry quality, observation time, completeness, and reasons.
At minimum they distinguish `Good`, `Uncertain`, `Bad`, `Stale`, and
`Unavailable`. Manufacturing calculations follow these rules:

- communication failure never becomes `false`, `0`, empty text, or simulated data;
- last-known values retain their original timestamp and become stale;
- a calculation lists missing/bad inputs and cannot report confident output;
- one failed device does not invalidate independent equipment;
- time windows expose quality gaps rather than deleting them;
- reports show calculation/policy version and data-completeness metadata;
- simulation and real acquisition are selected explicitly and visually distinct.

## Machine-command architecture

The existing tag-write path is a primitive, not the final machine-command
contract. A planned central command service accepts an action such as
`machine.start` or `recipe.apply`, resolves server-owned configuration, and
enforces:

1. authenticated action permission;
2. project/equipment scope;
3. explicit tag/command allowlist;
4. typed value and range validation;
5. optional confirmation, reason, or electronic-signature hook;
6. command request audit before dispatch;
7. bounded dispatch with an explicit accepted/rejected/timeout result;
8. command state distinct from PLC-reported actual state.

The PLC remains authoritative for permissives, interlocks, operation, and all
safety functions. OpenWebHMI is supervisory and never safety-rated.

## Alarm, fault, state, and downtime boundaries

These concepts remain separate:

- **Machine state:** what the equipment is doing.
- **Alarm/fault:** an abnormal condition with lifecycle and operator response.
- **Downtime event:** a production-time interval that may or may not have a fault.
- **Loss classification:** versioned policy describing business/OEE treatment.

A stopped machine may have no alarm. An alarm may not reduce production.
Blocked/starved loss may be external. Operator waiting may affect Performance
rather than Availability. Downtime/OEE modules reference alarm and state facts;
they do not replace the alarm engine.

## Module packaging

The current Driver trait and TypeScript component registry are useful extension
seams, but OpenWebHMI does not yet have a general module SDK. The planned module
contract must define optional contributions for:

- gateway services and lifecycle hooks;
- project artifact schemas and migrations;
- SQLite storage migrations and optional backend adapters;
- protocol messages and HTTP/WebSocket routes without ungoverned collisions;
- runtime components, routes, and navigation;
- Designer editors, palettes, and validation;
- scripts/system APIs;
- report data sources/templates;
- backup/export participation;
- health and diagnostics.

The first implementation may compile first-party modules into the gateway. Hot
loading is not required to prove the contract. Enabling a module is explicit per
project; disabling it preserves or deliberately archives its data.

## Cross-platform contract

Gateway and Designer are first-class on Linux, macOS, and Windows. Every new
module must document and test:

- filesystem paths and atomic replacement;
- SQLite/native library behavior;
- process spawning and shutdown;
- TLS/certificate stores;
- packaging and migrations;
- browser/webview behavior;
- optional native integrations.

Platform-specific integrations are optional adapters behind clean contracts.
They cannot become a core dependency or prevent basic authoring/runtime on any
supported desktop OS. Current public docs that call Linux Designer post-1.0 are
documentation drift against `VISION.md` and must be reconciled before v1.0.

## Roadmap horizons

### v1.0 — committed scope and hardening

No manufacturing module implementation is added. Finish security, driver,
data-integrity, frontend, docs-honesty, multi-platform CI/release, plugin SDK,
performance, and real-hardware validation work already on the board.

### Foundation release after v1.0 — planned

- general module contribution and module-storage contracts;
- project-template distribution and deterministic scenario engine;
- server-authoritative action/command gateway;
- optional equipment hierarchy;
- semantic manufacturing facts, projections, quality/completeness;
- manufacturing web component pack and interactive line layout;
- Basic Machine HMI template and first useful manufacturing demo slice.

The first product-ready distribution after this foundation must bundle the
initial demo locally and expose it in the first-run/new-project workflow. A
working default demo is a release gate, not an optional gallery download.
That same shipped demo is the canonical source for website screenshots,
documentation, release posts, and YouTube/tutorial material.
It ships with an offline deterministic profile and an opt-in real Rockwell PLC
profile using the same logical project contract; real communication failure is
never masked by simulated values.
The first showcase is planned on an Omarchy Linux workstation to demonstrate
Gateway, Designer, browser runtime, real Rockwell connectivity, and offline
simulation on an open-source desktop OS. Omarchy remains a validated showcase
configuration, not a required dependency or replacement for general Linux,
macOS, and Windows support.

### Manufacturing release — planned

- production counters/targets and machine state timeline;
- fault analytics built on alarm/fact infrastructure;
- richer report definitions and reproducible exports;
- versioned downtime classification, corrections, and recalculation;
- OEE only after boundary, target, planned-time, downtime, and quality contracts.

### Later modules — exploratory or deferred

- recipes/changeover and maintenance are strong next candidates after the
  command/storage contracts;
- part/process data precedes SPC and traceability;
- SPC requires reviewed statistical definitions and rational subgrouping;
- energy precedes carbon, with configurable versioned factors;
- equipment health begins with transparent statistical rules;
- traceability, Andon, and MES integration follow proven identity/event models;
- full MES execution, advanced ML, and remaining-useful-life predictions remain
  deferred until scope and data justify them.

## Highest-value additions after v1.0

1. Official project templates and deterministic simulation.
2. Module/storage contribution contract.
3. Server-authoritative action and write gateway.
4. Optional equipment hierarchy.
5. Semantic facts/projections with quality propagation.
6. Production counters, targets, and state timeline.
7. Manufacturing component pack and line layout.
8. Alarm/fault analytics.
9. Reproducible report definitions and export.
10. Versioned downtime followed by OEE.

## Architectural decisions still open

These are deliberately not guessed in implementation briefs:

- exact manifest/API shape for first-party versus third-party modules;
- whether equipment identity lives in the core project schema or a built-in
  optional module artifact;
- SQLite schema-per-module naming and migration ownership;
- first optional PostgreSQL abstraction boundary;
- exact report definition/rendering technology;
- public names for manufacturing module bundles;
- which advanced feature expands the demo first after OEE;
- whether energy is first-party or a reference plugin.

Decisions must preserve the dependency direction, embedded default, public
privacy boundary, and three-language limit.
