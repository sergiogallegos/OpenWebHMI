---
id: CODEX-ED
title: Add target-aware project publishing and offline deployment
owner: codex
phase: 4
status: open
created: 2026-09-29
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-ED — Add target-aware project publishing and offline deployment

## Brief

### Goal

Expose preview, selected-gateway publication and offline project transfer through one Designer/CLI workflow.

### Context and dependencies

DX portable format; DY CLI/local preview; DQ routing; DR sole atomic publisher; EC Python readiness; BE backup transport safety; CK release artifacts. Read Ignition references in the project engineering plan as workflow references only.

Authoritative contract: [project engineering plan](../../planning/project-engineering.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as implemented or validated capability.

### Files to create or modify

Designer target/export/import/setup flows; shared CLI adapters; gateway preflight/project transfer APIs; protocol Rust/TS pairs; existing deployment and backup tests/docs.

### Required behavior

- Define named target profiles without resolved secrets; check schema/engine/component versions, drivers/services, Python readiness, storage and expected revision.
- Present resource manifest/diff and explicit authorized publication; distinguish project artifacts from gateway-wide users, credentials, certificates, connections and history.
- Assemble deterministic versioned .owhmi exports and safely import bounded packages with referenced assets/dependencies. Reuse backup/path protections; no traversal or silent gateway-wide replacement.
- Invoke DR for activation/rollback; add post-publish readiness and revision verification without PLC writes. Report interruptions, failed/stale/denied operations and retained prior revision accurately.
- Provide separate gateway setup guidance for selected OS/architecture with verified release/package retrieval, prerequisites and connection checks. Clearly distinguish download/setup from publishing a project.
- Offer equivalent structured CLI results for optional agents and automation; permissions remain shared. Document supported offline, local and remote-installed-gateway workflows.

### Acceptance and validation

- [ ] Clean checkout -> preview -> export -> compatible offline import -> publish -> verify -> rollback fixture passes without an LLM or internet.
- [ ] Denied/stale publication, missing dependencies/secrets, version mismatch, partial upload, corrupt/path-escaping package and insufficient capacity leave prior production intact.
- [ ] Export/import resource manifest preserves project content and never includes resolved secrets or silently overwrites gateway-wide state.
- [ ] Paired Designer/CLI fixtures use the same DR authority; target setup/download and browser workflow evidence is recorded on supported OSes.
- [ ] Regression tests fail before the change; required Rust/TS checks pass; installation and project deployment are documented separately.

Use existing test files, deterministic synchronization and ephemeral ports. Behavior
regressions must fail against the pre-fix code. Run checks appropriate to the change
and required workspace validation; record unperformed manual, OS and hardware checks.

### Out of scope

No competing publication authority, automatic remote OS provisioning, privilege escalation, cloud/fleet management, implicit live writes or assertion that rollback reverses external side effects.

### Risks and boundaries

Coordinate shared contract changes with the listed owners before implementation.
Preserve existing formats and wire behavior or provide a paired migration. Never
claim a preview, sandbox, offline package or rollback guarantee without its evidence.

## Codex log

## Claude review

## Verdict
