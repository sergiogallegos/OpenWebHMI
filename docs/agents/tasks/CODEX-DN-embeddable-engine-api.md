---
id: CODEX-DN
title: Expose embeddable engine and prove independent public-API consumer
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DN — Expose embeddable engine and prove independent public-API consumer

## Brief

### Goal

Make the gateway and a second application consume one public engine API, without requiring the gateway executable or network listener.

### Context and dependencies

DM; BX instance-owned services; BV configured paths; BI/BJ supervised drivers; BF/BT command authorization and expiry.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Define an engine builder, typed errors, capabilities, bounded subscriptions and commands, and snapshot/gap/resync semantics.
- Document caller-owned Tokio runtime, trusted low-level operations versus authorized operator commands, and shutdown drain/abort results. No implicit global logging, process exit or network listener.
- Refactor gateway composition to consume the engine; add a minimal external Rust host using only public APIs, selected driver/store adapters and separate data roots.
- Declare crate versioning/MSRV and a compatibility check; C ABI, Python bindings, WASM and dynamic ABI remain deferred. Engine license remains AGPL.

### Acceptance and validation

- [ ] Two engine instances coexist and stop independently; cancellation, startup rollback and failure isolation tests pass.
- [ ] Out-of-workspace consumer builds without GUI/HTTP/CLI dependencies, publishes/subscribes and handles a deterministic write outcome.
- [ ] Malformed commands, access denial, timeout, overload and shutdown behavior are covered by meaningful regressions; existing gateway wire clients remain compatible.

Run checks proportionate to the changes plus required workspace validation.
Behavior regressions must demonstrably fail without their fix. Mechanical code
changes require compile and three consecutive existing-suite passes. Use existing
test files, deterministic synchronization and ephemeral ports. Record hardware,
browser and cross-platform checks that were not performed by name.

### Out of scope

Do not implement adjacent tasks under this task's completion claim. No clustering,
MES expansion, license change, unbounded queues, telemetry, stub drivers or broad
Cargo dependency update. Preserve existing project/wire compatibility or ship an
explicit paired migration. Do not claim a performance gain without measurement.

## Codex log

## Claude review

## Verdict
