---
id: CODEX-EC
title: Prepare reproducible optional Python project environments
owner: codex
phase: 4
status: open
created: 2026-09-29
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-EC — Prepare reproducible optional Python project environments

## Brief

### Goal

Make Python dependencies explicit, reproducible and deployable offline without making Python mandatory for basic projects.

### Context and dependencies

DX project metadata; EB execution identities/jobs; DO minimal profiles; DR immutable revisions and rollback; ED deployment preflight.

Authoritative contract: [project engineering plan](../../planning/project-engineering.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as implemented or validated capability.

### Files to create or modify

Project dependency schemas and scripting environment adapter; CLI/Designer preparation UI; gateway activation integration; existing project/scripting tests and deployment docs.

### Required behavior

- Specify Python version constraints, resolved dependency versions/hashes and target OS/architecture metadata. Select packaging tooling with compatibility/license evidence rather than adding competing package managers.
- Create managed per-project environments keyed by resolved dependency/runtime identity through an explicit preparation operation. Opening or validating a project never downloads or runs installation hooks.
- Recreate target environments rather than copying .venv. Offer offline wheel bundles with integrity metadata and interpreter requirements, plus explicit use of preprovisioned environments.
- Preflight missing/incompatible packages, interpreter and disk space before activation; bound preparation resources and clean partial failures. No silent network fallback for offline mode.
- Tie environment identity to published revisions; retain prior environments for rollback and in-flight jobs with a documented drain/cancel/cleanup policy.
- Keep dependency state and secrets outside source; track dependency declarations and locks. No interpreter or installer is required when scripting is disabled.

### Acceptance and validation

- [ ] No-Python open/validate/minimal-build fixture passes; explicit script checks report a missing interpreter clearly.
- [ ] Offline preparation succeeds on recorded supported targets and fails cleanly for wrong architecture, hash, interpreter, missing wheel and insufficient capacity.
- [ ] Preparation failure leaves active revision/environment intact; upgrade and rollback preserve correct job/environment identities.
- [ ] Deterministic tests use controlled local packages and no public registry; actual Windows/macOS/Linux native-wheel proof is recorded separately.
- [ ] No dependency lock drift beyond chosen tooling; applicable Rust/TS checks and inbound license gate pass.

Use existing test files, deterministic synchronization and ephemeral ports. Behavior
regressions must fail against the pre-fix code. Run checks appropriate to the change
and required workspace validation; record unperformed manual, OS and hardware checks.

### Out of scope

No bundled ML stack, global Python mutation, automatic package execution on open, general-purpose dependency registry or guarantee that arbitrary packages are sandboxed.

### Risks and boundaries

Coordinate shared contract changes with the listed owners before implementation.
Preserve existing formats and wire behavior or provide a paired migration. Never
claim a preview, sandbox, offline package or rollback guarantee without its evidence.

## Codex log

## Claude review

## Verdict
