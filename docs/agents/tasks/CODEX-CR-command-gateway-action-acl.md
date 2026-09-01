---
id: CODEX-CR
title: Server-authoritative machine command gateway and action ACLs
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CR — Machine command gateway

## Brief

### Goal

Build one server-owned authorization, validation, dispatch, result, and audit
path for supervisory actions; web clients must not choose their own authority.

### Dependencies

- CODEX-BF server-authoritative write authorization and existing audit hardening.
- CODEX-CQ equipment scope for equipment-bound actions; a generic action contract
  can be designed first without machine-domain leakage.

### Required behavior

- action-level permission and project/equipment scope;
- server-owned allowlist mapping action to typed tag writes;
- value/range validation, optional confirmation/reason/signature hook;
- request/accepted/rejected/timeout result and correlation id;
- audit request, authorization result, dispatch result, and actor;
- command/request state separate from PLC-reported actual state;
- reconnect policy that never replays stale operator commands.

### Tests

- unauthorized/client-steered action, allowlist bypass, bad type/range, timeout,
  reconnect, audit, and command-versus-actual behavior tests;
- tests must fail against the primitive pre-feature tag-write path.

### Acceptance criteria

- [ ] No runtime/Designer message can nominate an unconfigured write target.
- [ ] Every accepted/rejected action has an auditable result.
- [ ] PLC permissives/interlocks remain authoritative and are never bypassed.
- [ ] Simulated commands use the identical gateway contract.

### Out of scope

- Safety logic, hard real-time control, recipe workflow, or protocol-specific commands.

## Codex log

## Claude review

## Verdict

