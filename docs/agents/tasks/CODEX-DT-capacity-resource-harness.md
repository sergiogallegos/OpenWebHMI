---
id: CODEX-DT
title: Establish Edge Standard Medium capacity and resource evidence
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DT — Establish Edge Standard Medium capacity and resource evidence

## Brief

### Goal

Create a reproducible correctness-aware benchmark harness before optimization, then certify only the profiles actually proven.

### Context and dependencies

DL workload contract; CJ observability. Harness starts immediately; final profile evidence depends on DM–DS, BW/CB/CG, driver correctness, CK/CN.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Version workload fixtures for 500/10K/50K active tags, real accepted updates, payload mix, alarms, writes, subscriptions and history admission rates.
- Separate engine-only, simulator/wire, physical PLC, storage and browser latency; count sequences/gaps and verify final states, quality, command results and admitted history.
- Report p50/p95/p99 latency, RSS/total process memory, CPU, queue age, drops, disk/WAL growth, storage-per-sample and query/restore time with environment/build metadata.
- Add month-sized preseeded datasets, 72-hour mixed-load soak, 60-second burst/recovery and real low-resource device runs; do not equate accelerated time with real-time endurance.
- Build a measured performance baseline in the repository before BW/CB/DS optimizations. Release profiling is allowed for this defined investigation; correctness suites remain debug.

### Acceptance and validation

- [ ] Deterministic fixture/count/unit tests fail for dropped/reordered samples or bad calculations; performance measurements do not introduce flaky wall-clock unit tests.
- [ ] Record baseline failures honestly; no target is declared supported without its workload, limits, data integrity and hardware evidence.
- [ ] Compare same workload/hardware/build configuration before and after changes; collect browser and gateway measurements independently.

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
