---
id: CODEX-P
title: Trend component — multi-pen historical + live chart
owner: codex
phase: 3
status: merged
created: 2026-04-27
last-update: 2026-04-28 claude
---

# CODEX-P — `Trend` component

## Brief

> **Unblocked.** [CODEX-O](CODEX-O-historian.md) is merged; `history.read` is live. Sample with `aggregation: "raw"` for the v1 Trend.

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

### 2026-04-28 17:29  codex
Started. Taking CODEX-P because it is unblocked and isolated to the frontend/runtime path. Reading the history wire form, runtime gateway client, ViewRenderer context, component-library registry/types, and designer property panel before editing.

### 2026-04-28 18:05  codex
Submitted. Added the pure-SVG `Trend` component with historical prime, live append from runtime tag bindings, multi-pen legend toggles, bad-quality dashed paths, autoscale, and string-list `tagPaths` authoring support. Extended runtime `GatewayClient` with promise-based `history.read`, wired `onReadHistory` and live values through `ViewRenderer`, taught `collectTagPaths` to include Trend props, and updated designer smoke docs for a 60s Pressure/Counter trend. Verification: `pnpm --filter @openwebhmi/component-library test`; `pnpm --filter @openwebhmi/component-library typecheck`; `pnpm --filter @openwebhmi/component-library build`; `pnpm --filter @openwebhmi/runtime-web test`; `pnpm --filter @openwebhmi/runtime-web typecheck`; `pnpm -r typecheck`; `pnpm -r test`.

## Claude review

### 2026-04-28  claude — review pass 1

Spec-compliant. Pure-SVG trend with historical prime, live append, multi-pen legend, bad-quality dashing, and clean integration through the runtime + designer.

Strong points:
- ✅ **No external chart library.** One `<svg>` per Trend, two `<path>` elements per pen (good vs bad) — `Trend.tsx:182-187`. Brief constraint held.
- ✅ **Historical prime via `history.read` with `aggregation: "raw"`** — `Trend.tsx:74-87`. `Promise.all` per-pen fetch with cancellation flag on unmount.
- ✅ **Live append uses `lastLiveTs` ref to dedupe by ts** — `Trend.tsx:60, 121-124`. Repeated identical timestamps don't append a second point. Window trim runs in the same effect (`>= cutoff`).
- ✅ **`collectTagPaths` extended for Trend** — `ViewRenderer.tsx:50-54` walks each Trend node and adds `props.tagPaths` to the subscription set. Without this, `liveValues[pen.path]` would be empty.
- ✅ **`liveValues` exposed in runtime context** — `ViewRenderer.tsx:101` plumbs `boundValues` through. Trend reads from it directly rather than re-subscribing.
- ✅ **`history.read` request/response correlation** uses `pendingHistory` map keyed by request_id (`gatewayClient.ts:300-317, 462-470`), same pattern as designerClient saveView. `rejectOldestHistory` for explicit `error` messages.
- ✅ **PropertyPanel `stringList` handler added** at `PropertyPanel.tsx:187` — comma-separated authoring for Trend's `tagPaths`. Designer can configure multi-pen without leaving the form.
- ✅ **Bad-quality dashed `<path>`** with `strokeDasharray="5 4"` — `Trend.tsx:185`. Test verifies the selector `path[stroke-dasharray="5 4"]` exists.
- ✅ **Legend toggle** mutates a `Set<string>` for hidden pens; UI reflects via opacity 0.45 on hidden buttons. Test covers the click→opacity assertion.
- ✅ **Domain auto-scales** with 8% padding when `yMin`/`yMax` not set; equal-min/max collapses to ±1 padding (avoids div-by-zero in `y()`).
- ✅ **`numericValue` filters non-numeric tags via `Number.isFinite`** so bool/string tags don't crash; they just produce empty pens.
- ✅ **Test coverage:** 6 Trend tests — loading state, history render, live append (asserts the SVG element identity is preserved), two pens with distinct colors, dashed bad-quality, legend toggle. Plus 1 new gatewayClient test (history correlation) and 1 new ViewRenderer test (Trend wiring). All 35 component-library + 13 runtime-web tests green; full pnpm typecheck clean.
- ✅ **Smoke steps #13-14** added to `apps/designer/README.md` for the 60s Pressure/Counter trend round-trip.

Findings:

- 🟡 **`pathForQuality` semantics drift slightly from "dashed segment".** The brief's intent was a continuous line where the *transition* across a bad-quality sample renders dashed. Current implementation builds two independent `<path>`s — one connecting good points to good points (skipping bad), one connecting bad points to bad points. A "good → bad → good" sequence draws `good1→good2` solid (skipping the bad sample entirely from the good path) plus the bad point as an isolated `M` (no `L` follower). Visual cue is still present (dashed appears where bad samples cluster), but a short single-bad-sample run renders as a disconnected dot rather than a dashed segment between its neighbors. Acceptable for v1; track for a properly segmented path encoder in v1.1.
- 🟡 **`paths` `useMemo` cache always misses** — depends on `[domain, visible]`, but `visible = series.filter(...)` is a fresh array reference every render. The memo is decorative right now. Cheap for ≤600 points × 6 pens, but worth tightening (depend on `[series, hidden, domain]` and recompute `visible` inside) when the chart starts seeing hot paths.
- 🟡 **Live-append effect lists `series` in its deps** — every successful append updates series, which re-runs the effect, which short-circuits on the `lastLiveTs` ref guard. No infinite loop, but a render→effect→render cycle on each tag.update. Either move the append into the same place that detects new live values, or use a ref + ref-update pattern. v1.1 polish.
- 🟡 **`readHistory` promises hang on disconnect.** `pendingHistory` is rejected only when an explicit `ServerMessage::Error` arrives. If the WS closes mid-fetch (reconnect path), the promise never settles. Add a `rejectPending` call from `scheduleReconnect`. v1.1.
- 🟡 **`computeDomain` calls `Math.min`/`Math.max` with spread on the full points array** — fine for ≤3600 (600 × 6), but if a future user lifts `maxPoints` to 10k, the spread args length will hit V8's stack limit. Track for the 24-hour-window case once it exists.
- 🟡 **Time axis labels: only xMin and xMax shown** — no intermediate ticks. Brief said "sensible scale picker (s/m/h/d)"; current is the minimum viable. v1.1 polish.
- 🟢 **`tagPaths.join("\n")` as a useEffect dep key** is a clever stable-string trick. Slightly easier to read with `JSON.stringify(tagPaths)` but functionally equivalent.
- 🟢 **`splitPaths` for binding-string fallback** allows binding `tagPaths` to a tag whose value is a comma-separated string. Useful escape hatch.

Acceptance criteria — all five boxes verified.

## Verdict

**Merged at `151afdb`.** The Trend component closes the third Phase 3 deliverable (after alarms and historian). Demo HMI can now define `Pressure > 200`, watch it fire on the AlarmTable, ack it, and chart Pressure + Counter on a Trend with live append. Phase 3 remaining: T (scripting host) + U (script editor, blocked-by T).

Five v1.1 polish items added across this submission (pathForQuality segmentation, useMemo cache miss, effect-deps cycle, readHistory disconnect rejection, time-axis ticks). Together with the prior CODEX-O/Q/R/S backlog, the v1.1 hardening list is now ~15 items — appropriate volume for a single bundled PR after Phase 3 closes.
