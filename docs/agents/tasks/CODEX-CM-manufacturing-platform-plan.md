---
id: CODEX-CM
title: Evidence-based manufacturing platform roadmap and task program
owner: codex
phase: 5
status: submitted
created: 2026-09-01
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-CM — Manufacturing platform plan

## Brief

### Goal

Turn the two local, read-only reference implementations into an anonymized,
evidence-based post-1.0 plan without changing v1.0 scope or copying their stacks,
identifiers, data, or assets.

### Context to read first

- `VISION.md`, `AGENTS.md`, `CLAUDE.md`, `docs/agents/README.md`
- `README.md`, `docs/architecture.md`, `docs/roadmap.md`,
  `docs/feature-matrix.md`, and the current board
- the reference repositories' canonical specs, backlogs, source, and tests
- the maintainer decision to include the OEE reference's live working tree while
  leaving both reference repositories unmodified

### Deliverables

- authoritative manufacturing capability/core-module boundary and roadmap;
- anonymized evidence/gap/adoption matrix;
- starter/demo project plan;
- dependency graph, storage/quality/event/command/module recommendations;
- Linux/macOS/Windows contract reconciliation;
- executable CODEX briefs for the planned foundation/manufacturing sequence;
- README, roadmap, feature-matrix, board, and log links kept consistent.

### Verification

- run Markdown/link validation available in the repository;
- run `scripts/validate-agent-files`;
- grep public changes for private project/customer identifiers, addresses, tags,
  alarm wording, and brand names;
- inspect `git diff --check` and all changed files.

### Acceptance criteria

- [ ] v1.0 commitments are unchanged and manufacturing work is gated after them.
- [ ] Implemented, committed, planned, exploratory, deferred, and out-of-scope
      language is explicit.
- [ ] Core and optional modules, dependencies, data quality, semantic facts,
      storage, commands, equipment, packaging, and cross-platform rules are clear.
- [ ] Reference findings distinguish implemented code from aspirational specs.
- [ ] No private reference content enters public files.
- [ ] Planned work has dependency-ordered CODEX briefs and board rows.

### Out of scope

- Manufacturing feature implementation.
- Modifying either reference repository.
- Committing or pushing without a separate maintainer request.
- Resolving product decisions explicitly left open by the plan.

## Codex log

### 2026-09-01  codex [gpt-5]

Discovery completed against all three repositories. The OEE reference working
tree was included by maintainer direction and remained read-only. Documentation
and task-program work is in progress.

### 2026-09-01  codex [gpt-5]

Submitted as an uncommitted documentation/task-program diff. Added the
authoritative manufacturing plan, anonymized reference capability/adoption
analysis, starter/demo design, roadmap and feature-matrix integration, Linux
Designer contract reconciliation, and CODEX-CN..CY dependency-ordered briefs.
Advanced modules remain exploratory/deferred without executable board tasks.

Verification run: `scripts/validate-agent-files` (103 task files), local
Markdown-link scan across 23 affected files, `git diff --check`, JSON parsing via
the link-scan Node process, and a private-identifier grep across the new planning
and task files. No Rust/TypeScript behavior changed, so the full code test matrix
was not run.

### 2026-09-01  codex [gpt-5]

Maintainer clarification incorporated after submission: the standard product
distribution must bundle a working demo locally by default. The demo is now an
explicit product-ready release gate with offline first-run `Open Demo` and
`Create from template` entry points, not an optional later download.

### 2026-09-01  codex [gpt-5]

Demo deployment clarification incorporated: the same project now has a default
offline deterministic profile and an opt-in real Rockwell PLC profile. Views and
manufacturing modules share one logical contract; a failed real connection never
falls back to simulation. CODEX-DA owns the generic controller package, mode
integration, licensing review, supported firmware matrix, and hardware proof.

### 2026-09-01  codex [gpt-5]

First-showcase clarification incorporated: CODEX-DB now owns reproducible
deployment and video proof on an Omarchy Linux workstation, demonstrating the
Gateway, Linux Designer, browser runtime, real Rockwell profile, offline profile,
and honest disconnect behavior. Omarchy is explicitly a first showcase target,
not a core dependency or exclusive supported distribution.

### 2026-09-27 14:22 codex [GPT-6] — accepted scope reconciliation

The accepted engine/capacity plan is the prerequisite foundation for this post-v1 manufacturing plan. Browser-first CN proof replaces mandatory desktop installers; DV owns optional native packaging. Manufacturing scope stays post-v1.

### 2026-09-29 22:37 codex [GPT-6] — project engineering scope coordination

The [project engineering plan](../../planning/project-engineering.md) records
maintainer-directed scope clarification. Original brief and status are preserved.

V1 paired component SDK must cover declared properties/events/bindings, scoped CSS, version compatibility and the same preview/runtime renderer. Prove a custom TS component through source authoring, manual property edit, export/import and runtime render. No executable HTML/JS embedded in view JSON, new XML format or automatic install on open; use the existing component SDK ownership rather than a duplicate task.

### 2026-09-29 22:38 codex [GPT-6] — SDK ownership correction

The preceding scope addendum assigned v1 component SDK implementation to CM in
error. CM remains the submitted Phase 5 manufacturing planning task; its scope and
status do not change. The previously untracked v1 component extension work is now
owned by CODEX-EF. CO remains the later general module/storage contract. The shared
project/preview/runtime requirements apply to future manufacturing components.

## Claude review

## Verdict
