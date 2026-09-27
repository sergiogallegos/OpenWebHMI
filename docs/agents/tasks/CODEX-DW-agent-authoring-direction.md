---
id: CODEX-DW
title: Record open project and agent-assisted Designer architecture
owner: codex
phase: 4
status: submitted
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DW — Record open project and agent-assisted Designer architecture

## Brief

### Goal

Align product direction and implementation ownership for a Designer that interoperates with external editors and terminal agents.

### Context and dependencies

DL accepted architecture; DR publication contract; project-store implementation evidence.

Authoritative contract: [agent authoring plan](../../planning/agent-authoring.md).
Read root/scoped AGENTS.md and VISION.md before implementation.

### Required behavior

- Document portable source, schemas/CLI, local and remote authoring, conflicts and publication boundaries.
- Keep planned capabilities distinct from current file storage; no model provider or MCP dependency.
- Update vision, architecture, roadmap, board and wiki with implementation ownership.

### Acceptance and validation

- [ ] Documentation links, task validator and diff checks pass.
- [ ] No CLI, synchronization, performance or runtime implementation is claimed.

Run appropriate repository checks. Behavior regressions must fail against pre-fix
code; use deterministic synchronization and ephemeral ports. Paired Designer/runtime
contract changes land together. Record unperformed browser, OS and hardware checks.

### Out of scope

No mandatory AI provider, built-in chat or MCP server, automatic publication,
implicit script execution/PLC writes, dynamic executable plugin loader, private
SQLite editing API, license change or broad dependency updates. Do not claim
planned features are implemented. Keep optional authoring tools out of Edge runtime.

## Codex log

### 2026-09-27 15:32 codex [GPT-6]

Recorded under the accepted Designer direction. Current ProjectStore code was inspected;
file-based artifacts do not establish external-edit synchronization. DX/DY/DZ remain
open implementation tasks. Documentation validation is recorded after execution.

### 2026-09-27 15:34 codex [GPT-6]

Added the accepted positioning to the README and website source: visual authoring
or external coding-agent assistance over one open project. Copy explicitly marks
the workflow planned and makes no market-first, speedup or universal-compatibility
claim. No CLI, synchronization or runtime code was implemented.

Validation passed: `scripts/validate-agent-files` (129 briefs), license validator
self-test, 37 new local Markdown link targets and `git diff --check`. With Node
24.21.0, `pnpm -r typecheck`, `pnpm -r build` and three consecutive `pnpm -r test`
runs passed (128 tests per run). Existing designer bundle-size warning remains.
Rust code and dependency lockfiles are unchanged; Rust suites were not rerun for
this documentation/website-copy change. No interactive visual/browser smoke,
real-agent workflow, OS matrix, PLC or publication acceptance test was performed.
Website source update is not evidence of production website deployment.

## Claude review

## Verdict
