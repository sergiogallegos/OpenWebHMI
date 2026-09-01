---
id: CODEX-CU
title: Alarm maturity and equipment-aware fault analytics
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CU — Alarm and fault analytics

## Brief

### Goal

Extend the single existing alarm subsystem with mature lifecycle controls and
equipment-aware fault analytics; do not create a second alarm engine.

### Dependencies

- v1 alarm correctness tasks CODEX-CE/CF/CA and CODEX-CQ equipment/facts.
- Use existing alarm journal, associated data, audit, and AlarmTable/Banner.

### Required behavior

- complete equipment/source association and quality gaps;
- shelving with bounded duration/reason/audit after lifecycle design review;
- frequency, duration, longest active, Pareto, chatter, and flood metrics;
- MTTR and MTBF with documented inclusion/window rules;
- filters and reproducible CSV/report data source;
- mapping from alarm occurrence to a fault fact without making it downtime/OEE.

### Tests

- lifecycle/restart, shelving expiry, chatter/flood windows, overlapping/open
  occurrences, data gaps, equipment filters, and metric-definition tests.

### Acceptance criteria

- [ ] Existing alarm journal remains authoritative.
- [ ] Analytics state definitions and denominators are documented and tested.
- [ ] Alarm/fault does not automatically become downtime or Availability loss.
- [ ] No wall-clock waits in analytics/lifecycle tests.

### Out of scope

- Notification rosters, SMS/voice, downtime policy, or customer alarm databases.

## Codex log

## Claude review

## Verdict

