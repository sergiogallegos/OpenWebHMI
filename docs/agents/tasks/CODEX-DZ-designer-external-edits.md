---
id: CODEX-DZ
title: Synchronize Designer drafts with external project edits
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-DZ — Synchronize Designer drafts with external project edits

## Brief

### Goal

Let engineers and agents edit the same project while preserving source, dirty buffers and production revisions.

### Context and dependencies

DX format; DY local authoring service; DQ browser delivery; DR shared draft/revision protocol. CC owns existing editor integrity fixes.

Authoritative contract: [agent authoring plan](../../planning/agent-authoring.md).
Read root/scoped AGENTS.md and VISION.md before implementation.

### Required behavior

- Reconcile content-identified validated snapshots with revision checks; watcher notifications are hints, with recovery/rescan for missed or duplicate events.
- Support multi-file create/rename/delete transactions and show partial/invalid edits as diagnostics while retaining the last valid preview.
- Use a known base for dirty-buffer comparison; preserve overlapping versions for explicit resolution, and make undo revision-aware.
- Apply the same draft events for remote CLI submissions, local files and visual edits. Keep UI selection/editing stable when unrelated artifacts change.
- Add one end-to-end terminal-to-Designer-to-runtime fixture: author, validate, preview, external update, conflict, explicit publish and rollback.

### Acceptance and validation

- [ ] Deterministic event-driven tests cover external save, partial writes, Git conflict markers, duplicates/missed events, rename/delete and disconnect/reconnect without wall-clock sleeps.
- [ ] Concurrent visual/external edits and undo do not silently overwrite each other; rejected snapshots leave the accepted draft intact.
- [ ] Production stays unchanged on source edits; browser evidence on Linux/macOS/Windows and explicit authorized publish/rollback are recorded separately from unit tests.

Run appropriate repository checks. Behavior regressions must fail against pre-fix
code; use deterministic synchronization and ephemeral ports. Paired Designer/runtime
contract changes land together. Record unperformed browser, OS and hardware checks.

### Out of scope

No mandatory AI provider, built-in chat or MCP server, automatic publication,
implicit script execution/PLC writes, dynamic executable plugin loader, private
SQLite editing API, license change or broad dependency updates. Do not claim
planned features are implemented. Keep optional authoring tools out of Edge runtime.

## Codex log

### 2026-09-29 22:37 codex [GPT-6] — project engineering scope coordination

The [project engineering plan](../../planning/project-engineering.md) records
maintainer-directed scope clarification. Original brief and status are preserved.

Acceptance includes keeping the Designer open during agent-equivalent file edits and preserving supported source through manual saves. Cover invalid intermediate multi-file writes, Python formatting, Git merge conflicts and last-valid preview. Surface shared EB test/job diagnostics without executing code on open or passive preview.

## Claude review

## Verdict
