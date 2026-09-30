---
id: CODEX-EE
title: Add named project database access for Python scripts
owner: codex
phase: 5
status: open
created: 2026-09-29
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-EE — Add named project database access for Python scripts

## Brief

### Goal

Provide optional bounded database integration for application scripts, starting with a project-owned local SQLite adapter.

### Context and dependencies

V1 foundation and EB jobs/capabilities; EC environments; CL/CI explicit database deferral; DR revisions; CO coordination where module-owned storage is involved.

Authoritative contract: [project engineering plan](../../planning/project-engineering.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as implemented or validated capability.

### Files to create or modify

Optional database service adapter and Python bridge; project connection/permission schema; Designer configuration/stubs; existing scripting/database tests and docs.

### Required behavior

- Use named connections, parameterized queries, explicit read/write grants and secret references; start with project-owned SQLite.
- Define bounded result/statement sizes, timeouts, transaction and cancellation semantics, connection lifecycle and migration/backup responsibilities.
- Keep internal auth/project/historian/audit stores unavailable as arbitrary script databases; data access to core services uses their authorized APIs.
- Expose identical text configuration and diagnostics to agents/Designer with explicit isolated database fixtures for script tests.
- Scope external database and outbound HTTP adapters separately after the local adapter; do not introduce storage dependencies into pure domain code.

### Acceptance and validation

- [ ] Parameterized read/write fixture works against a temporary project-owned database; permission and transaction/cancellation tests pass.
- [ ] Result limits, malformed parameters, denied paths, secret redaction and internal-database access rejection are tested.
- [ ] Agent-equivalent source edits survive Designer round trips; explicit tests cannot reach production data.
- [ ] Optional feature build and required suites pass; deferred external adapters remain labelled deferred.

Use existing test files, deterministic synchronization and ephemeral ports. Behavior
regressions must fail against the pre-fix code. Run checks appropriate to the change
and required workspace validation; record unperformed manual, OS and hardware checks.

### Out of scope

No unrestricted internal gateway SQL, external DB fleet, general HTTP client, MES implementation or mandatory database service in the minimal engine.

### Risks and boundaries

Coordinate shared contract changes with the listed owners before implementation.
Preserve existing formats and wire behavior or provide a paired migration. Never
claim a preview, sandbox, offline package or rollback guarantee without its evidence.

## Codex log

## Claude review

## Verdict
