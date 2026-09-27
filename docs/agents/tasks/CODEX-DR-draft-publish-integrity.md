---
id: CODEX-DR
title: Add revision-checked drafts and atomic project publishing
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DR — Add revision-checked drafts and atomic project publishing

## Brief

### Goal

Prevent incomplete authoring work from silently changing operator runtime screens.

### Context and dependencies

BC/BR persistence safety; CC editor fixes; DM schema boundary; DN engine; DQ delivery integration.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Define versioned draft, published revision, preview and rollback semantics with paired Rust/TS messages and project-format migration.
- Use optimistic revision checks for concurrent editors; show actionable conflicts without overwriting another editor silently.
- Validate a complete revision before atomic publish, authorize the action, journal its result, and let runtime clients switch coherently.
- Define crash/restart/backup behavior and compatibility for existing projects that currently save/hot-reload immediately; no PLC writes from a passive design preview.

### Acceptance and validation

- [ ] Two-editor conflicts, partial-save failures, unauthorized publish, crash recovery, rollback and old-project migration tests fail before the new behavior.
- [ ] Runtime keeps the prior published revision through incomplete/failed edits; publish switches all relevant artifacts together.
- [ ] Paired designer/runtime smoke and wire fixtures prove preview isolation and explicit result feedback.

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
