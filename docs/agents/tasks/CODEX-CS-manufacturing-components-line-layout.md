---
id: CODEX-CS
title: Manufacturing component pack and interactive line layout
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CS — Manufacturing components and line layout

## Brief

### Goal

Create a first-party optional web component pack for equipment-aware operations
views, including a Designer-authored interactive line/cell layout.

### Dependencies

- CODEX-CO contribution contract and CODEX-CQ equipment/projection contracts.
- Existing component schemas, registry, widget export/import, themes, runtime,
  Designer property panel, and accessibility requirements.

### Initial components

- station tile, quality badge, target/actual indicator, KPI card, compact sparkline;
- state-timeline strip and Pareto only if their data contracts have landed;
- line-layout node/link/container with equipment binding and navigation target.

### Required behavior

- color plus label/glyph, quality/completeness display, touch and keyboard access;
- responsive SVG/web layout with deterministic serialization;
- runtime drill-down and Designer editing without protocol-specific addresses;
- component props stable across packs and module-disable behavior explicit.

### Acceptance criteria

- [ ] Gallery, Designer, runtime, schema, export/import, and accessibility tests exist.
- [ ] Layout renders degraded/stale equipment honestly.
- [ ] No customer drawing, logo, station name, or private asset enters fixtures.
- [ ] Bundle-size impact is measured and within the documented budget.

### Out of scope

- CAD import, 3D/digital twin, automatic process-flow inference, or OEE calculation.

## Codex log

## Claude review

## Verdict

