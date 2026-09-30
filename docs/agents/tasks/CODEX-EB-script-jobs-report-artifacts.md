---
id: CODEX-EB
title: Implement bounded Python jobs and PDF CSV report artifacts
owner: codex
phase: 4
status: open
created: 2026-09-29
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-EB — Implement bounded Python jobs and PDF CSV report artifacts

## Brief

### Goal

Provide optional project application scripts and explicitly invoked report jobs usable from the runtime, Designer and terminal tools.

### Context and dependencies

CL existing API/trigger ownership; BF/BT authorization; DX script metadata; DY test commands; DR revisions; coordinate EC environments before release.

Authoritative contract: [project engineering plan](../../planning/project-engineering.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as implemented or validated capability.

### Files to create or modify

crates/scripting and gateway job adapters; paired crates/protocol and packages/protocol-ts; apps/designer, apps/runtime-web; existing scripting/UI tests and documentation.

### Required behavior

- Define versioned script entry-point/input/output/capability metadata and structured diagnostics/API stubs; retain existing Python source compatibility or migrate explicitly.
- Separate short event handlers from longer jobs with bounded queues, explicit overload behavior, concurrency, timeouts, cancellation, shutdown and per-OS resource-limit reporting. Never call subprocess isolation a sandbox.
- Add authenticated UI job invocation, status/progress/result/error events and explicit CLI/Designer tests using mock tags and temporary data. Tag writes use the gateway write sink, never direct TagStore publication.
- Authorize each command with the script grant plus authenticated caller grant for UI jobs; scheduled work uses an explicit service identity. Authorize result access/cancel separately and reject spoofed identity.
- Produce basic CSV and PDF from deterministic sample data; select the PDF dependency through the license gate. Store bounded artifacts with authenticated downloads, expiry/cleanup and no arbitrary host-path exposure.
- Keep pure UI bindings/navigation independent of Python. Opening/validation/passive preview never executes scripts. Coordinate supported trigger/API tables with CL/CI; do not silently implement deferred database/HTTP/view hooks.

### Acceptance and validation

- [ ] Unauthorized and spoofed job/command/result access fails; UI invocation does not elevate privileges.
- [ ] Queue saturation, timeout, cancellation, worker crash, reconnect and shutdown are deterministic tests; report generation cannot starve event handlers.
- [ ] CLI and Designer run the same explicit script fixture, surface file-level errors and expose logs without requiring an agent or network.
- [ ] CSV contents and PDF content/layout are checked; paired runtime/Designer download smoke and unsupported OS enforcement limitations are recorded.
- [ ] Regression tests fail on pre-fix code; required Rust/TS suites and license checks pass.

Use existing test files, deterministic synchronization and ephemeral ports. Behavior
regressions must fail against the pre-fix code. Run checks appropriate to the change
and required workspace validation; record unperformed manual, OS and hardware checks.

### Out of scope

No ML platform, general messaging bus, native Python UI, arbitrary database access, provider SDK or remote OS provisioning.

### Risks and boundaries

Coordinate shared contract changes with the listed owners before implementation.
Preserve existing formats and wire behavior or provide a paired migration. Never
claim a preview, sandbox, offline package or rollback guarantee without its evidence.

## Codex log

## Claude review

## Verdict
