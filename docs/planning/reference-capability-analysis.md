# Reference Capability Analysis

> Public, anonymized synthesis. Local reference repositories were inspected as
> engineering evidence only. No private names, addresses, tags, alarms, values,
> screenshots, branding, or customer process details belong in OpenWebHMI.

## Evidence method

The analysis compared source, tests, canonical specifications, active backlogs,
and the current working tree of two local reference implementations. A feature
described in a specification but still open in its backlog is not labeled
implemented. Technology choices are not treated as portability recommendations.

## Reference machine-HMI findings

The reference WPF HMI validates several broadly useful concepts:

- zero-UI domain seams between PLC drivers, logical tags, quality, and screens;
- a single authorization/audit gateway for machine writes;
- command state distinct from actual PLC state;
- logical-to-symbol tag mapping and protocol-independent screens;
- complete alarm occurrence identity and transition persistence;
- store-and-forward behavior around database failures;
- role-aware, touch-first navigation and persistent machine controls;
- shifts crossing midnight, UTC persistence, local display;
- reusable machine diagnostics for I/O, axes, robots, vision, safety status;
- maintenance documents and operator handoff as pragmatic HMI features.

Its source contains implemented core services and broad screen/component
coverage, but its active backlog still identifies unfinished recipe workflow,
report UI wiring, security audit coverage, hardware validation, alarm commands,
equipment faceplates, communications diagnostics, and deterministic scenarios.
The distinction matters: OpenWebHMI should reuse proven concepts without
claiming every reference feature is production-complete.

Do not migrate:

- WPF or Windows-only hosting;
- its database or PDF libraries solely because the reference selected them;
- project-specific roles, tag maps, screen names, imagery, or branding;
- buffered PLC writes after reconnect without an explicit stale-command policy;
- PC-side safety or permissive authority.

## Reference OEE application findings

The reference OEE web application provides stronger evidence for manufacturing
analytics and data integrity:

- one independently supervised acquisition session per physical controller;
- transition/counter-change persistence instead of every poll;
- immutable raw facts and idempotency keys;
- explicit Good/Stale/Disconnected/BadData quality without substitute values;
- physical machines separated from internal PLC stations;
- machine output defined at a reviewed boundary; station counts are not summed;
- overlapping station downtime unioned at machine level;
- target rates integrated over effective planned-production time;
- versioned downtime/alarm policies, immutable corrections, recalculation runs;
- model alignment that becomes indeterminate on bad/missing observations;
- run/session contexts and verified archive-before-reset behavior;
- deterministic simulation visibly separated from live read-only acquisition;
- version metadata in PDF/CSV reports.

The current working tree also validates a separately gated, transition-only
alarm acquisition path. It does not justify coupling alarms directly to OEE:
classification and direct OEE treatment remain separate policy decisions.

Keep project-specific:

- exact equipment counts/topology, addresses, station identities, PLC mappings,
  alarm databases, product models, customer classifications, and deployment;
- assumptions specific to semi-automatic workflow;
- native Windows packaging and the reference application's multi-process stack;
- customer approval/evidence vocabulary in end-user OpenWebHMI features.

## Migration/adoption matrix

| Capability | Evidence | OpenWebHMI today | Gap | Target | Horizon | Treatment |
|---|---|---|---|---|---|---|
| Quality-bearing live tags | Both references | Implemented primitive | Higher-level completeness | Core data services | Foundation | Extend existing quality; do not duplicate. |
| PLC driver seam | Both references | Implemented `Driver` trait | General module lifecycle | Connectivity/module SDK | v1 closeout/foundation | Keep Rust protocol ownership. |
| Central command gateway | Machine-HMI | Role + view ACL tag writes | Action ACL, allowlist, result/actual | Security/commands | Foundation | Rewrite for server-owned web architecture. |
| Alarm lifecycle | Machine-HMI | Alarm engine + journal | shelving, delays, analytics | Alarm core | v1.x/manufacturing | Extend one alarm engine. |
| Alarm/fault analytics | Both references | Basic journal/UI | Pareto, duration, MTBF/MTTR, gaps | Fault analytics | Manufacturing | Build read models over journal/facts. |
| Historian/store-and-forward | Both references | SQLite historian | retention/backends/domain events | Data services | Foundation/later | Preserve SQLite; add contracts. |
| Equipment hierarchy | OEE reference | None | Stable physical/internal identity | Equipment module | Foundation | Optional artifact; no dummy hierarchy. |
| Semantic facts/projections | OEE reference | Tag/alarm/history primitives | Domain facts, idempotency, rebuild | Manufacturing data | Foundation | Small explicit layer, not giant event sourcing. |
| Production counters/targets | Both references | Generic tags/charts | Domain configuration/projections | Production | Manufacturing | Rewrite as protocol-neutral module. |
| State timeline | Both references | Trend primitives | Interval model + quality gaps | Production UI | Manufacturing | Reuse chart/component infrastructure. |
| Downtime classification | Both references | None | reasons, versions, corrections | Downtime | Manufacturing | Separate from alarms and state. |
| Hierarchical OEE | OEE reference | Roadmap only | boundaries, policies, recalculation | OEE | Manufacturing | Build after production/downtime. |
| Run/session windows | OEE reference | None | generic analysis scope | Production/OEE | Manufacturing | Generalize beyond customer FAT workflow. |
| Line overview/layout | Both references | SVG/components | equipment-aware interactive component | Manufacturing UI pack | Foundation | Web-native component + demo. |
| Recipe workflow | Machine-HMI | Post-1.0 roadmap | revisions, compare, handshake | Recipes | Later planned | Requires command gateway and audit. |
| Reporting | Both references | Basic scripted target | definitions, metadata, schedule | Reporting core | Manufacturing | Do not bind to reference PDF library. |
| Maintenance/documents | Machine-HMI | Generic assets only | document index, counters, service log | Maintenance | Later planned | Small pragmatic module first. |
| Part/process records | Both references | None | cycle/measurement schema | Part data | Later | Keep separate from genealogy. |
| SPC | Reference designs | None | statistical definitions/engine | Quality/SPC | Exploratory | Implement only with validated math/tests. |
| Energy/carbon | New platform synthesis | Tag history can sample | context normalization/factor versions | Energy | Exploratory | Configurable factors; no universal claims. |
| Equipment health | New platform synthesis | Historian inputs | baselines/rules/evidence | Health | Exploratory | Statistics before ML. |
| Traceability | Reference design only | Roadmap only | part identity/operations/search | Traceability | Deferred | Optional; after part-data foundation. |
| Andon/team communication | Reference design only | None | call/escalation workflow | Andon | Deferred | Integration hooks, not core dependency. |
| MES execution | Reference design only | Explicitly not v1 | orders/routes/WIP contracts | MES | Deferred | Integration boundary before execution. |

## Core versus module classification

Put a capability in core only when multiple unrelated modules need the same
mechanism and a simple HMI benefits from the contract without enabling the
domain. Thus quality envelopes, migrations, audit, report infrastructure,
module lifecycle, and command authorization are core. OEE categories, downtime
policy, production targets, measurement definitions, and emissions factors are
module-owned.

Machine-control faceplates and line nodes are component-pack patterns. Operator
reason capture and recipe approval are domain workflows. Exact station logic,
fault mappings, targets, and shift policy remain application configuration.

