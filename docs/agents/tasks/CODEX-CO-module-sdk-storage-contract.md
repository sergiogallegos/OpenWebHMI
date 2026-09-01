---
id: CODEX-CO
title: Module SDK and module-owned storage contract
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CO — Module SDK and storage contract

## Brief

### Goal

Define and prove the smallest general module contract needed by optional
manufacturing capabilities while keeping first-party modules compiled in and
SQLite as the zero-ops default.

### Dependencies

- Begin after v1.0 hardening and the promised driver/component plugin SDK is honest.
- Read the existing Driver trait, component registry/widget packs, project store,
  backup, protocol, gateway lifecycle, and Designer extension seams.

### Required behavior

- versioned module manifest and dependency validation;
- project artifact/schema contribution and migration ownership;
- namespaced SQLite migrations plus backup/restore participation;
- governed gateway service/routes/protocol contribution;
- runtime components/routes/navigation and Designer editor/palette contribution;
- module health/diagnostics and enable/disable lifecycle;
- explicit compatibility and uninstall/data-preservation behavior.

### Proof module

Use a tiny fictional first-party module with one artifact, one table, one route,
one runtime component, one Designer editor, backup/restore, and health status.
Do not use OEE as the architecture test.

### Acceptance criteria

- [ ] Basic HMI projects load without any optional module.
- [ ] A module can be enabled, migrated, backed up, restored, and disabled safely.
- [ ] Namespace/version conflicts fail before partial mutation.
- [ ] Linux, macOS, and Windows tests pass.
- [ ] Hot loading and third-party native code are not falsely claimed.

### Out of scope

- Dynamic library loading, custom marketplace, PostgreSQL implementation, or a
  manufacturing domain model.

## Codex log

## Claude review

## Verdict

