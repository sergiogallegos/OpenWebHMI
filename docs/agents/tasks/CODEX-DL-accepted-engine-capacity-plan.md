---
id: CODEX-DL
title: Accepted engine foundation, capacity research and restart program
owner: codex
phase: 4
status: submitted
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DL — Accepted engine foundation, capacity research and restart program

## Brief

### Goal

Record accepted library-first reuse, shared desktop UI, 50K Medium target and low-resource profile; reconcile authoritative documents and create executable tasks.

### Context and dependencies

DK baseline; VISION.md; docs/planning/engine-and-capacity.md; wiki/architecture/engine-capacity-decision.md.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Separate vendor evidence, proposed budgets and verified capabilities; calculate 30-day storage from admitted samples/s.
- Align vision, architecture, roadmap, public claims, deployment docs and board; retain historical task briefs with explicit scope addenda.
- Commit and push the already validated baseline and planning progress as requested; do not claim open implementation tasks are shipped.

### Acceptance and validation

- [ ] Agent/license validators, local links, workload arithmetic and git diff checks pass.
- [ ] Website content typecheck/build and existing tests pass; record prior baseline verification separately.
- [ ] Commit/push result is recorded, with no fabricated review, hardware or performance evidence.

Run checks proportionate to the changes plus required workspace validation.
Behavior regressions must demonstrably fail without their fix. Mechanical code
changes require compile and three consecutive existing-suite passes. Use existing
test files, deterministic synchronization and ephemeral ports. Record hardware,
browser and cross-platform checks that were not performed by name.

### Out of scope

Do not implement adjacent tasks under this task's completion claim. No clustering,
MES expansion, license change, unbounded queues, telemetry, stub drivers or broad
Cargo dependency update. Preserve existing project/wire compatibility or ship an
explicit paired migration. Do not claim a performance gain without measurement.

## Codex log

### 2026-09-27 14:22 codex [GPT-6]

Accepted direction implemented as documentation/task updates under direct
maintainer instruction. Upstream Ignition sizing and Ghostty library architecture
were checked; workload/storage assumptions and unproven budgets are explicit.
DK's previously tested compiler/build changes are included in the requested
progress commit. DM–DV are executable follow-up briefs, not implemented features.
Validation and push evidence will be appended after completion.

### 2026-09-27 14:27 codex [GPT-6]

Validation passed: agent-file validator (125 briefs), license validator self-test,
53 newly added local link targets, 30-day sample/storage arithmetic, and
`git diff --check`. Re-ran `pnpm -r typecheck`, `pnpm -r build` and three consecutive
`pnpm -r test` runs (128 tests each) under Node 24.21.0 after the final website edits.
Website Astro checks report zero errors/warnings; the existing designer bundle-size
warning remains. There is no website-specific behavioral test suite; workspace
frontend suites passed. No visual browser smoke was performed.

DK records the earlier successful Rust build/clippy/doc/fmt and three consecutive
165-test runs; no Rust code changed during this planning follow-up. Cargo.lock is
unchanged. Engine extraction, 50K capacity, low-resource operation, desktop speed,
Linux/Windows execution and real PLC soak remain unproven. Ready for the requested
progress commit/push; independent task review remains pending.

### 2026-09-27 14:28 codex [GPT-6]

Committed at `5f03604` and successfully pushed to `origin/main` under explicit
maintainer instruction (remote advanced from `373be6e` to `5f03604`). This supersedes
the earlier uncommitted/no-push record. Status remains submitted: publishing progress
is not an independent review verdict or proof of the open implementation targets.

## Claude review

## Verdict
