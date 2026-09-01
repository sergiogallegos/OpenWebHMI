---
id: CODEX-CV
title: Reproducible report definitions, rendering, and export framework
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CV — Reporting framework

## Brief

### Goal

Evolve basic scripted output into a platform report contract that modules can
contribute to without each creating a rendering stack.

### Dependencies

- CODEX-CO module contributions and at least one real data source from CODEX-CT
  or CODEX-CU. Inspect existing Python subprocess and backup/security boundaries.

### Required behavior

- versioned report definition and governed data-source API;
- date/time/equipment/model/run filters and UTC/local metadata;
- deterministic PDF and CSV with query/policy/schema/version metadata;
- preview, empty/failure states, authorization, audit, and export from views;
- later-compatible scheduler/distribution seam without requiring internet/email;
- backup/export of definitions and module contribution lifecycle.

### Acceptance criteria

- [ ] A report can be reproduced from persisted inputs and recorded versions.
- [ ] PDF/CSV outputs use the same filtered dataset and tested escaping/timezones.
- [ ] Renderer choice is licensed, cross-platform, and does not require a fourth language.
- [ ] Basic HMI users do not run a report service unless enabled.

### Out of scope

- Full visual report designer, Excel, SMTP/SMS implementation, or adopting a
  reference project's PDF library without an independent architecture decision.

## Codex log

## Claude review

## Verdict

