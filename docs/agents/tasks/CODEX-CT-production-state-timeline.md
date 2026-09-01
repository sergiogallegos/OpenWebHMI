---
id: CODEX-CT
title: Production monitoring, targets, machine state, and timeline module
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CT — Production and state timeline

## Brief

### Goal

Ship the first manufacturing domain module: quality-aware production counts,
targets, cycle metrics, model context, machine state, and time-window timeline.

### Dependencies

- CODEX-CO, CODEX-CQ, CODEX-CP, and relevant CODEX-CS components.

### Required behavior

- reviewed machine output boundary; internal-station counts remain diagnostic;
- total/good/reject counter-change and reset facts;
- versioned target by equipment/model and explicit coverage gaps;
- machine state facts, interval projection, source quality, and timeline gaps;
- actual/target/delta, throughput, cycle metrics, and model mismatch projection;
- no formal line OEE or naive line-output sum.

### Tests

- counter delta/reset/wrap policy, out-of-order facts, bad/stale source, target gaps,
  boundary/internal station, interval clipping, and deterministic scenario tests;
- report/read-model pagination and backup/restore tests.

### Acceptance criteria

- [ ] Missing/bad counts or targets produce unavailable metrics, not zero.
- [ ] Boundary and aggregation rules are visible in configuration/UI.
- [ ] Machine timeline includes explicit quality gaps.
- [ ] Demo production screen works without PLC hardware.

### Out of scope

- Downtime classification, OEE, SPC, traceability, or synchronous line-flow assumptions.

## Codex log

## Claude review

## Verdict

