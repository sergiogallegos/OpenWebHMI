---
id: CODEX-DM
title: Extract pure domain and storage-independent wire schema
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DM — Extract pure domain and storage-independent wire schema

## Brief

### Goal

Remove storage/network/UI dependencies from pure engine types/rules and the wire protocol while preserving JSON compatibility and the approved license map.

### Context and dependencies

DL; crates/protocol, tag-engine, project-store and alarm-engine; coordinate BX. No security fix waits for this task.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Extract pure tag identity/value/quality and alarm rules with injected time; default domain graph uses std/core only.
- Keep MPL wire DTOs self-contained and map to AGPL domain types in an AGPL adapter; do not relicense copied storage code. Preserve public re-exports where possible.
- Move shared project shape ownership out of concrete persistence; no SQLite, TOML, filesystem or HTTP requirements in protocol/domain builds. Optional serialization must be explicit.

### Acceptance and validation

- [ ] Independent minimal package build and cargo tree prove absence of storage/transport dependencies; add a boundary check that fails on the old graph.
- [ ] Shared old/new wire fixtures and existing Rust/TS tests prove unchanged messages; new behavior regressions fail before fixes.
- [ ] Pure alarm transition/clock tests and migration/license validators pass; no no_std or stable C ABI claim.

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

### 2026-09-27 15:32 codex [GPT-6] — open authoring scope addendum

The [agent authoring plan](../../planning/agent-authoring.md) adds external editors
and terminal agents as peer authoring clients. DX owns portable source/offline
schemas, DY the CLI/local service, DZ reconciliation and conflicts. Coordinate
shared format/validation with DM and browser delivery with DQ; DR remains the only
publication authority for both CLI and Designer. Original brief/status preserved;
no external-edit feature is implemented by this addendum.

## Claude review

## Verdict
