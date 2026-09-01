---
id: CODEX-CX
title: Quality-aware hierarchical OEE module
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CX — Hierarchical OEE

## Brief

### Goal

Implement OEE as a versioned, reproducible manufacturing calculation with
visible inputs and quality—not as three disconnected gauges.

### Dependencies

- CODEX-CQ, CODEX-CT, CODEX-CW, CODEX-CV, and their stable persisted contracts.

### Required behavior

- A/P/Q/OEE per equipment boundary, model, shift/run/custom window;
- effective planned-production time and target/ideal-cycle version;
- physical machine boundary counts; internal stations diagnostic only;
- unioned included downtime and attributed causes;
- versioned calculation policy/run with immutable inputs and retained results;
- data completeness/quality and unavailable factor propagation;
- line rollup only with explicit project policy; descriptive averages labeled.

### Tests

- mathematical edge cases and bounds; missing targets/counts; bad quality; partial
  coverage; overlapping downtime; internal station non-summation; policy versions;
  recalculation reproducibility; cross-timezone/shift windows.

### Acceptance criteria

- [ ] Every KPI exposes source window, boundary, target/policy version, and completeness.
- [ ] The same persisted facts and versions reproduce the same result.
- [ ] Unavailable inputs never become confident zero or 100% values.
- [ ] No protocol or database-vendor logic appears in the OEE engine.

### Out of scope

- Universal line OEE, operator performance scoring, SPC, MES scheduling, or
  customer-specific Availability/Performance policy.

## Codex log

## Claude review

## Verdict

