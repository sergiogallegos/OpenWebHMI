---
id: CODEX-DO
title: Implement minimal engine and Edge Standard Medium feature profiles
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-DO — Implement minimal engine and Edge Standard Medium feature profiles

## Brief

### Goal

Make unused services absent from the build/runtime graph and prove a small headless Edge deployment.

### Context and dependencies

DM/DN; DT baseline; CK supply-chain and build matrix; accepted profile definitions.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Replace workspace-wide Tokio full and accidental driver server-feature leakage with per-crate features verified in isolation.
- Make driver families, historian and Python adapters selectable; disabled configured services fail explicitly at startup. No mandatory Python, Node, Tauri or desktop libraries in the minimal deployed engine.
- Record normal/build/test/platform dependency closures, duplicates, enabled features, licenses, package and installed footprint; remove unused dependencies without reimplementing secure primitives.
- Preserve full product builds and reviewed protocol pins. Feature combinations must not bypass authentication/write policy when transport is enabled.

### Acceptance and validation

- [ ] Minimal, each individual driver/service and full builds pass independently; enforce forbidden dependency edges with a check demonstrated against the old graph.
- [ ] Edge target workload runs on recorded ARM64/x86-64 low-resource hardware with RSS/CPU/disk/queue evidence and remote browser.
- [ ] Fresh-machine startup without Python/Node/desktop packages works when those features are disabled; record unsupported combinations.

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

### 2026-09-29 22:37 codex [GPT-6] — project engineering scope coordination

The [project engineering plan](../../planning/project-engineering.md) records
maintainer-directed scope clarification. Original brief and status are preserved.

Python workers, package preparation and authoring services remain optional. Add EC coordination to minimal-build evidence: a scripting-disabled Edge build requires neither interpreter nor package installer. Report domain versus orchestration versus adapter dependencies separately.

## Claude review

## Verdict
