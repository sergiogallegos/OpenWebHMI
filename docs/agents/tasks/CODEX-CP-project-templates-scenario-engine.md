---
id: CODEX-CP
title: Versioned project templates and deterministic scenario engine
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CP — Project templates and scenario engine

## Brief

### Goal

Let the Designer create an independent project from a versioned built-in
template and run deterministic, explicitly simulated scenarios without hardware.

### Dependencies

- Stable project archive/schema migration behavior.
- CODEX-CO only if template manifests declare optional modules; the initial Basic
  Machine HMI template may proceed without manufacturing modules.

### Required behavior

- template manifest, compatibility validation, assets, and required module list;
- `Create from template` workflow with regenerated project/component identifiers;
- seeded randomness and controllable logical clock;
- scenario reset/replay and explicit simulated origin/quality;
- normal, fault, stale/bad/disconnected, and guarded command scenarios;
- no automatic fallback from failed real acquisition to simulation.

### Tests

- deterministic output without sleeps/wall-clock waits;
- import/create collision and migration tests;
- real-versus-simulated separation regression test;
- Linux/macOS/Windows Designer creation and gateway startup coverage.

### Acceptance criteria

- [ ] Blank and Basic Machine HMI templates create runnable independent projects.
- [ ] Replaying a seed produces identical ordered facts/updates.
- [ ] Missing/incompatible modules abort before project creation.
- [ ] Export/import and backup preserve template-created projects.

### Out of scope

- OEE, SPC, energy, traceability, or the full manufacturing demo.

## Codex log

## Claude review

## Verdict

