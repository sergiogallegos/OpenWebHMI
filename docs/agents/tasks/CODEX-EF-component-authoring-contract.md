---
id: CODEX-EF
title: Prove paired custom component authoring and packaging
owner: codex
phase: 4
status: open
created: 2026-09-29
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-EF — Prove paired custom component authoring and packaging

## Brief

### Goal

Complete the v1 custom component authoring contract for manual and text/agent
workflows without changing the runtime contract independently of the Designer.

### Context and dependencies

DX source/schema/catalogs; DY scaffolding and explicit validation; DZ reconciliation;
DR revision authority; ED export/import. Existing component registry, widget packs,
AS widget transfer and the Phase 4 plugin SDK roadmap are the starting points.
CO remains the separate post-v1 module/storage contract; CM is planning only.

Read root/scoped AGENTS.md, VISION.md and the authoritative
[project engineering plan](../../planning/project-engineering.md).

### Files to create or modify

packages/component-library and existing SDK scaffolding if present; paired
Designer/runtime registration; component manifests and source schemas; existing
component and editor tests; SDK documentation and example package.

### Required behavior

- Specify versioned manifests with declared properties, bindings/events, assets,
  scoped styles and engine/renderer compatibility requirements.
- Provide one documented TypeScript/TSX component package with CSS; use the same
  renderer in preview and runtime and preserve supported fields during visual edits.
- Export offline catalog/schema information for terminal tools. Missing/incompatible
  components produce actionable errors rather than being dropped silently.
- Define explicit package build/install steps and artifact identity. Project open,
  validation and passive preview never install or execute unreviewed package code;
  preview uses already installed trusted components. Do not add executable HTML/JS
  fields to view JSON or a second XML representation.
- Coordinate source declarations and built artifacts with ED export/import/preflight;
  no dynamic plugin ABI or implicit network package resolution.

### Acceptance and validation

- [ ] An example component is created from the SDK, built/installed explicitly,
  placed manually, bound through a source edit and rendered in preview/runtime.
- [ ] Designer saves preserve externally authored supported props, bindings and IDs.
- [ ] Missing versions/assets, unknown fields and incompatible manifests fail
  clearly; export/import preserves the component identity and requirements.
- [ ] Paired existing Designer/runtime tests prove equivalent behavior and catch
  a pre-fix failure; optional agent-equivalent fixtures need no LLM.
- [ ] Applicable Rust/TS checks, package build and dependency-license gate pass;
  browser and three-platform manual checks are recorded separately.

### Out of scope

No arbitrary web-app round-tripping, XML view format, executable JSON properties,
Python browser runtime, third-party product engine SDK, driver SDK replacement or
post-v1 module-owned storage work.

### Risks and boundaries

Custom components are trusted executable extensions. Validation is structural and
does not establish code safety. Keep exact renderer compatibility and package
identity explicit across drafts, published revisions and offline transfers.

## Codex log

## Claude review

## Verdict
