---
id: CODEX-EA
title: Reconcile project engineering scripting and deployment direction
owner: codex
phase: 4
status: submitted
created: 2026-09-29
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-EA — Reconcile project engineering scripting and deployment direction

## Brief

### Goal

Record the accepted product direction consistently without claiming implementation.

### Context and dependencies

VISION.md; engine/capacity and agent-authoring plans; architecture, roadmap, design-system roadmap and stack rationale.

Authoritative contract: [project engineering plan](../../planning/project-engineering.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as implemented or validated capability.

### Files to create or modify

VISION.md; docs/planning/project-engineering.md; related architecture/plans; docs/agents board, log and task addenda.

### Required behavior

- Retain AGPL and accepted compliant reuse; explain internal engine modularity and pure-domain dependency goals.
- Specify Tauri-first desktop measurement with optional native-UI reconsideration, Git-ready source/terminal-agent editing, scoped Python services and deployment.
- Open implementation tasks with dependencies and evidence; preserve prior task briefs, statuses and review history.

### Acceptance and validation

- [x] Plans and design documents agree on scope, formats, permissions and delivery sequence.
- [x] Task frontmatter, board rows and append-only log pass repository validation; local Markdown links resolve.
- [x] Changes are planning only; no scripting, deployment, browser, OS or hardware implementation is claimed.

Use existing test files, deterministic synchronization and ephemeral ports. Behavior
regressions must fail against the pre-fix code. Run checks appropriate to the change
and required workspace validation; record unperformed manual, OS and hardware checks.

### Out of scope

No product code, license change, implementation completion claim, remote publication or task merge verdict.

### Risks and boundaries

Coordinate shared contract changes with the listed owners before implementation.
Preserve existing formats and wire behavior or provide a paired migration. Never
claim a preview, sandbox, offline package or rollback guarantee without its evidence.

## Codex log

### 2026-09-29 22:37 codex [GPT-6] — planning submission

Recorded maintainer-approved AGPL reuse, internal engine modularity, measured Tauri
strategy, Git-ready terminal/Designer authoring, optional application scripting and
deployment direction. Opened EB/EC/ED/EE and appended scope coordination to existing
tasks. Submission is the working-tree diff; implementation and independent review
remain pending. Validation results are appended after execution.

### 2026-09-29 22:39 codex [GPT-6] — validation

Passed scripts/validate-agent-files (135 task files), scripts/validate-licenses,
git diff --check and a temporary documentation check covering 28 changed/new
Markdown files and 171 local links. Confirmed required scope, balanced code fences
and preservation of existing task briefs. EF now owns the component SDK contract;
CM ownership was corrected in an appended entry. No product code changed; runtime
build/test suites, browser/OS smoke, hardware and deployment tests were not run.
These implementation gates remain in the open tasks. Local commit only; no push.

## Claude review

## Verdict
