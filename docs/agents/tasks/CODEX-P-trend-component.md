---
id: CODEX-P
title: Trend component — multi-pen historical + live chart
owner: codex
phase: 3
status: open
created: 2026-04-27
last-update: 2026-04-27 claude
blocked-by: CODEX-O
---

# CODEX-P — `Trend` component

## Brief

> **Blocked by [CODEX-O](CODEX-O-historian.md).** Needs `history.read` to fetch initial points.

### Goal

A multi-pen line chart that shows the last N minutes of one or more tags, with live updates appended as new `tag.update` events arrive. Users see process behavior at a glance — the trend is the most-asked-for SCADA component after the value display.

### Context to read first

- `packages/component-library/src/types.ts` — `ComponentDefinition`, `BoundValue`.
- `packages/component-library/src/components/ValueDisplay.tsx` — the simplest binding pattern.
- `apps/runtime-web/src/useTagBindings.ts` — how live values arrive.
- The `history.read` wire form from CODEX-O.

### Files to create

- `packages/component-library/src/components/Trend.tsx`
- `packages/component-library/src/components/__tests__/components.test.tsx` — extend with Trend tests.
- `apps/runtime-web/src/lib/historyClient.ts` (or extend `gatewayClient.ts`) — `readHistory(tag_path, range, agg, max_points): Promise<HistoryPoint[]>`.

Add `Trend` to `componentRegistry` in `packages/component-library/src/registry.ts`.

### Component contract

```ts
type TrendProps = {
  tagPaths: string[];               // bindable per-pen
  windowSeconds: number;            // e.g. 300 = last 5 minutes
  maxPoints: number;                // chart resolution; default 600
  yMin?: number;
  yMax?: number;                    // auto-scale if missing
  showLegend: boolean;
};
```

`bindableProps`: `tagPaths` (special — multi-tag binding; the property panel renders an array editor).

### Behavior

- On mount: fetch `history.read` for each tag in `tagPaths` over `[now - windowSeconds, now]`. Render lines.
- Subscribe to live `tag.update` for each path; append points; drop points older than the window.
- Use `react-svg` (no external chart library for v1 — keep dependency surface minimal). One `<svg>` per Trend instance, one `<path>` per pen.
- Bad-quality samples render with a dashed segment.
- Legend: tag path + current value + color swatch. Toggleable per-pen visibility.
- Keyboard: `← / →` pan one bucket; `+ / -` zoom in/out (post-1.0 polish; v1 = static window).

### Test requirements

- Renders with no data (initial fetch in flight) — shows "loading…".
- Renders fetched history correctly (mock the history client).
- Receives a live `tag.update` and appends without re-rendering existing path elements unnecessarily (use a `useRef` for the SVG path data).
- Multi-pen: 2 pens render distinct colors; legend lists both.
- Bad-quality segment renders dashed.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/component-library test` green with new Trend tests.
- [ ] `pnpm --filter @openwebhmi/component-library typecheck` clean.
- [ ] `componentRegistry.Trend` exported.
- [ ] No external chart library — pure SVG.
- [ ] Manual smoke (added to `apps/designer/README.md`): in the designer, add a Trend with `windowSeconds=60`, bind to `rockwell-1/Pressure` and `rockwell-1/Counter`, save, see lines update live in the runtime preview.

### Out of scope

- Zoom/pan UI (Phase 4).
- Cross-hair cursor with value readout (Phase 4).
- Export PNG / CSV (post-1.0).
- Logarithmic scale (post-1.0).
- Stacked / area mode (post-1.0).

### Risks / gotchas

- **Don't recompute the full SVG path on every tag.update** — incremental append + window trim is cheap; full recompute thrashes layout.
- **`maxPoints` × pens** can land >5000 SVG path commands. Browsers cope, but pre-decimate via `useMemo` keyed by the data ref.
- **Time axis labels** need a sensible scale picker (s / m / h / d). Don't ship "1234567890ms" labels.
- **Avoid `Date` formatting in render path** — format axis labels once when data changes.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
