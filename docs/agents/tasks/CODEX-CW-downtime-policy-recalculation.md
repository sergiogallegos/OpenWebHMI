---
id: CODEX-CW
title: Versioned downtime policy, corrections, and recalculation module
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CW — Downtime policy and recalculation

## Brief

### Goal

Model downtime/loss as policy-dependent intervals separate from machine state
and alarms, with operator reason capture and immutable supervisor corrections.

### Dependencies

- CODEX-CT production/state facts, CODEX-CU fault association, CODEX-CR action
  permissions/audit, CODEX-CV reporting data-source contract.

### Required behavior

- category vocabulary is project-configurable and versioned;
- planned/external, Availability/Performance treatment, reason requirement;
- automatic candidate plus operator reason and authorized correction revisions;
- immutable raw facts/corrections; atomic recalculation writes versioned results;
- interval clipping/union and machine/internal-station attribution;
- prior results retained for audit/comparison; unknown remains visible.

### Tests

- overlapping intervals, simultaneous causes, policy version selection, corrections,
  failed atomic recalculation, quality gaps, waiting/operator treatment, and audit.

### Acceptance criteria

- [ ] State, fault, downtime event, and loss policy remain distinct.
- [ ] Policy changes never mutate raw facts or earlier results.
- [ ] Missing/bad source windows cannot become confident classified downtime.
- [ ] No default policy is presented as universally correct.

### Out of scope

- OEE formula/rollup, customer-specific categories, or automatic alarm-to-downtime mapping.

## Codex log

## Claude review

## Verdict

