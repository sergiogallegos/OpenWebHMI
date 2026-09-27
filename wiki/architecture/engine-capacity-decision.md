---
status: active
last-validated: 2026-09-27
---

# Engine reuse and capacity decision

## Summary

The accepted direction is an embeddable Rust engine consumed by the gateway and
other applications, a shared browser/optional-desktop designer, and separately
measured Edge and 50K-tag Medium workloads. This is a design decision; no engine
extraction or capacity certification is implied. The operational specification
lives in the [engine and capacity plan](../../docs/planning/engine-and-capacity.md).

## Current understanding

1. Ghostty provides a useful architectural precedent: GUI applications consume a
   shared library, and library release/API concerns can be separate from the app.
   Its [architecture](https://ghostty.org/docs/about) and newer
   [1.3 extraction update](https://ghostty.org/docs/install/release-notes/1-3-0)
   support that comparison; neither proves an OpenWebHMI implementation nor changes
   its license. Rust public APIs and a second consumer come before optional FFI.
2. IA's [sizing guide](https://inductiveautomation.com/resources/article/ignition-server-sizing-and-architecture-guide)
   supports classifying 50K tags / 50 clients as a medium reference workload, with
   separate historian rates and hardware caveats. Product tag count is insufficient
   to infer historian throughput, disk requirements or low-resource behavior.
3. The current [tag engine](../../crates/tag-engine/Cargo.toml) depends on
   [protocol](../../crates/protocol/Cargo.toml), which depends on project-store.
   Isolation requires an actual dependency inversion. Library targets alone do
   not demonstrate an embeddable foundation or a small dependency graph.
4. The [historian](../../crates/historian/src/store.rs) currently writes a transaction
   per sample and reads raw rows before aggregation. Keep SQLite as the default,
   but require measurements at the new data volume. A format-size ceiling does not
   establish acceptable ingestion, maintenance, query or recovery behavior.
5. One shared web presentation avoids three separate native designer implementations.
   Performance remains a testable condition. Optional Tauri packaging is not evidence
   of native-control rendering, low memory or fast startup.
6. Retain established HTTP/TLS/storage/protocol implementations at adapter boundaries.
   A pure std/core domain and narrow Tokio services reduce mandatory dependencies
   without adopting custom HTTP parsing or cryptography as a maintenance strategy.

## Evidence

- [Accepted goals/workloads/contracts](../../docs/planning/engine-and-capacity.md),
  [VISION](../../VISION.md), and [DL planning record](../../docs/agents/tasks/CODEX-DL-accepted-engine-capacity-plan.md).
- [Baseline review](../../docs/planning/2026-09-architecture-review.md) records
  current dependency counts, global services, security and driver integration gaps.
- [License policy](../../LICENSE-POLICY.md): AGPL engine/product, MPL wire packages.
  No source movement or license exception is authorized by the analogy alone.

## Open questions

- Whether the SQLite adapter meets Medium ingestion and month-sized query budgets:
  DS/CG/DT provide evidence and, if needed, a separate backend decision.
- Actual Edge memory/CPU and optional desktop responsiveness: DO/DT/DV hardware
  runs resolve these. Targets must remain labeled until measured.
- Any future C ABI or additional bindings: require a concrete consumer, ownership
  design and compatibility policy; not a prerequisite for the first Rust engine.

## Related pages

- [Architecture](../../docs/architecture.md)
- [Historian](../../docs/historian.md)
- [Task board](../../docs/agents/board.md)
