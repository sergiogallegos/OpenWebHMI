---
id: CODEX-DS
title: Implement bounded durable historian ingestion and thirty-day retention
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DS — Implement bounded durable historian ingestion and thirty-day retention

## Brief

### Goal

Make SQLite historian ingestion/retention meet documented correctness contracts and test Medium feasibility before choosing another backend.

### Context and dependencies

DN service ports; CG bounded SQL aggregation/read owner; DT harness; BR atomic persistence and BV configured storage.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Introduce a narrow storage interface and batched transaction writer with bounded queue/spool, explicit admission/durability acknowledgements and overload policy.
- Preserve quality/source timestamps and distinct same-tag samples sharing a millisecond; define migration from the current tag_id/timestamp key and test it.
- Wire opt-in age/size retention safely with independent alarm/audit policy, minimum free-space monitoring, backup coordination and explicit gap/failure signaling.
- Test numeric storage/index/partition alternatives only with before/after evidence. Do not lower fsync guarantees to pass benchmarks.
- Seed a 30-day-sized Medium dataset and report ingest/query/maintenance/restore/disk footprint. If SQLite fails, record evidence and a backend decision request; do not add QuestDB silently.

### Acceptance and validation

- [ ] Crash after acknowledged durability loses no accepted sample; power/process-failure approximation limits are documented.
- [ ] Disk full, slow disk, queue overflow, duplicate timestamp, retention boundary and concurrent query/backup tests prove explicit behavior.
- [ ] Run 2,500 samples/s sustained and 5,000/s burst plus the month-sized query gate on recorded hardware; failures remain visible and do not certify Medium support.

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
