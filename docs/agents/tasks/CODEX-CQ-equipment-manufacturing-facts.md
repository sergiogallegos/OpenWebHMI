---
id: CODEX-CQ
title: Optional equipment model and manufacturing fact/projection layer
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CQ — Equipment model and manufacturing facts

## Brief

### Goal

Introduce the minimal protocol-neutral identity and semantic-data foundation
required by production, downtime, OEE, fault analytics, and line visualization.

### Dependencies

- CODEX-CO module/storage contract.
- Existing tag quality, historian, alarm journal, project identifiers, and backup.

### Required behavior

- optional hierarchy nodes for site/area/line/machine/station/device with stable ids;
- projects may begin at machine/line level without dummy ancestors;
- append-only fact envelope with event/equipment ids, schema version, source and
  observation timestamps, quality, origin, idempotency, and optional context;
- rebuildable current projections and explicit quality/completeness gaps;
- initial fact types for state, count, model, quality, and device connection;
- internal stations remain distinct from physical machines; no naive count sum.

### Tests

- hierarchy/reference and schema migration tests;
- idempotent append, ordered/out-of-order fact, projection rebuild, quality-gap,
  and independent-equipment isolation tests;
- backup/restore and cross-platform SQLite tests.

### Acceptance criteria

- [ ] Tag history, alarm journal, facts, projections, and derived results remain distinct.
- [ ] Communication failure never produces plausible default domain values.
- [ ] One failed device does not invalidate unrelated equipment.
- [ ] No manufacturing concept enters driver or tag-engine APIs unnecessarily.

### Out of scope

- OEE/downtime policy, enterprise rollups, PostgreSQL, or general event sourcing.

## Codex log

## Claude review

## Verdict

