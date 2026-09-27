---
id: CODEX-DU
title: Upgrade frontend tool families with compatibility and footprint checks
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DU — Upgrade frontend tool families with compatibility and footprint checks

## Brief

### Goal

Bring remaining frontend frameworks/tooling onto supported releases without a blind all-package update.

### Context and dependencies

DK version inventory; DQ Monaco integration; CB/BY runtime quality; existing React component suite.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Recheck latest supported versions at execution; checkpoint Vite/React plugin/Vitest/jsdom/Monaco together where compatibility requires it, then React/types/test adapters, Astro/compiler API, and pnpm separately.
- Preserve strict typechecking, frozen lockfile builds and official supported peer ranges; do not suppress conflicts or introduce unused state/chart/canvas libraries.
- Keep the website independent and use the compiler API version supported by Astro; the current TS5 exception is temporary and documented.
- Measure app/editor bundle sizes, dependency/feature changes and representative interactions; review pnpm lifecycle-build policy before major migration.

### Acceptance and validation

- [ ] Each compatible group passes clean frozen install, build, typecheck, existing tests and appropriate browser smoke before the next group.
- [ ] No unrelated Cargo changes or automatic protocol upgrades; inbound license/advisory and bundle reports are recorded.
- [ ] No inferred speedup from version numbers; publish before/after values if claiming improvement.

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
