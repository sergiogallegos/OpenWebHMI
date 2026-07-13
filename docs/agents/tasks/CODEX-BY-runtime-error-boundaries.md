---
id: CODEX-BY
title: Runtime error boundaries — one throwing widget must not blank the operator HMI
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BY — Runtime error boundaries (contain a throwing widget to itself)

## Brief

> The operator runtime (`apps/runtime-web`) has **no React error boundary anywhere**. Component `Render` functions consume arbitrary designer-authored JSON props; if any one component throws during render, React unmounts the entire tree and the operator sees a blank screen — the worst possible SCADA failure mode (a live plant HMI going dark because one gauge got a malformed prop). Wrap each rendered node in a per-node error boundary that contains the failure to the offending widget and shows a fallback placeholder (widget id + error) while every sibling keeps rendering and updating live. Add a top-level boundary as the last line of defense. HIGH.

### Goal

A single component whose `Render` throws renders a bounded fallback placeholder in its own slot; the rest of the view stays live and keeps receiving tag updates. A catastrophic top-level failure shows a recoverable app-level fallback instead of a blank page. No operator-facing screen ever goes fully blank because of one bad widget.

### Context to read first

- `apps/runtime-web/src/ViewRenderer.tsx` — `renderNode` (lines 67–110) is where each `ComponentNode` becomes a `<definition.Render .../>` (lines 100–108). This is the wrap point. Note it recurses: `children` are rendered by `renderNode` at lines 96–98, so a per-node boundary naturally nests. The existing "Unknown component" fallback (lines 83–89) and `styles.unknown` (lines 177–185) are the visual precedent for a fallback placeholder.
- `apps/runtime-web/src/App.tsx` — `ViewRenderer` is mounted at lines 143–152; the app-level boundary wraps here (or in `main.tsx`).
- `apps/runtime-web/src/main.tsx` (lines 1–9) — the root render (`<React.StrictMode><App /></React.StrictMode>`); the top-level boundary can wrap `<App />` here.
- React error boundaries require a **class component** (`getDerivedStateFromError` + `componentDidCatch`) — there is no hook equivalent as of React 18. The runtime is React 18 (`react-dom/client` `createRoot`).
- CLAUDE.md frontend discipline: no `any` abuse (type the caught error as `unknown`/`Error`), remove listeners/subscriptions on unmount, deterministic tests.

### Files to create / modify

1. **Create** `apps/runtime-web/src/ErrorBoundary.tsx` — a reusable class error boundary:
   - Props: `children`, a `fallback` render prop or element (receives the caught `Error` and an optional `widgetId`/label), and an optional `onError` callback for logging.
   - State: `{ error: Error | null }`. `static getDerivedStateFromError(error)` sets it; `componentDidCatch(error, info)` logs (via the `onError` prop or `console.error`) — do not swallow silently.
   - **Reset semantics**: when the boundary's `resetKeys` (e.g. the widget's `node.id`, or a changing prop) change, clear `error` so a transient bad prop that later becomes valid can recover on the next update rather than staying stuck on the fallback forever. Implement via `getDerivedStateFromProps` comparing a stored key, or a `componentDidUpdate` reset — state the approach in the Codex log.
   - Type the error as `Error` (or `unknown` narrowed) — no `any`.
2. **Modify** `apps/runtime-web/src/ViewRenderer.tsx` — wrap the returned `<definition.Render>` in `renderNode` (lines 100–108) with `<ErrorBoundary>`:
   - Fallback shows the failing widget's `node.id` and `node.kind` plus the error message, styled like the existing `styles.unknown` placeholder (a contained, visibly-degraded slot — not a blank gap).
   - Keep the existing `key={node.id}` behavior; the boundary should carry a `resetKeys`/key tied to `node.id` so identity is stable across re-renders and the boundary state doesn't bleed between different nodes when the tree reshapes.
   - The boundary wraps **each** node (the recursion at lines 96–98 means children get their own boundaries automatically), so a throwing child is contained within its parent's subtree without taking out the parent. Confirm the nesting behaves that way.
3. **Modify** `apps/runtime-web/src/App.tsx` (or `main.tsx`) — wrap `<ViewRenderer .../>` (App.tsx lines 143–152) or `<App />` (main.tsx lines 5–9) in a top-level `<ErrorBoundary>` with an app-level fallback (a "The screen failed to render — reload" recoverable message, not a blank page). Prefer wrapping at the `ViewRenderer` mount so the app chrome (connection status, etc.) survives a total view failure; add the `main.tsx` boundary too as the outermost catch-all.

### Behavior

- A component whose `Render` throws renders its fallback placeholder in-place; siblings render normally and continue to receive `tag.update`-driven re-renders.
- The fallback names the offending widget (`node.id` + `node.kind`) and shows the error message, so an integrator can identify the bad component without opening devtools.
- If the throwing widget's inputs later become valid (e.g. a malformed bound value corrects), the boundary resets and the widget renders live again on the next update.
- A top-level throw shows an app-level recoverable fallback, never a blank document body.

### Test requirements

- **Sibling stays live while one widget throws** (add to the runtime-web test suite — Vitest + React Testing Library, the existing frontend test stack; add to the closest existing ViewRenderer/App test file, don't fragment): render a view with two sibling widgets where one's `Render` throws unconditionally and the other reflects a bound value. Assert the throwing widget's fallback (with its `node.id`) is present **and** the sibling renders its value; then push a new bound value and assert the sibling re-renders while the fallback persists. This test must fail against the pre-fix code (an unguarded throw propagates and unmounts the tree / fails the render). Use a deterministic test double for the throwing component — no timers, no wall-clock waits.
- **Boundary reset**: a widget that throws on first render but not after a prop change recovers (fallback replaced by live content) once its `resetKeys` change. Deterministic — drive the prop change explicitly.
- **Top-level fallback**: a total render failure shows the app-level fallback, not an empty container.
- `pnpm -r typecheck` and `pnpm -r test` clean; no new `any`.

### Acceptance criteria

- [ ] `apps/runtime-web/src/ErrorBoundary.tsx` exists: class boundary, typed error (no `any`), logs via `onError`/`console.error`, resets on `resetKeys` change.
- [ ] `renderNode` wraps each `<definition.Render>` in a per-node boundary with a fallback naming `node.id` + `node.kind` + error message, styled as a contained degraded slot.
- [ ] A top-level boundary wraps the view mount (and a catch-all in `main.tsx`) with a recoverable app-level fallback — never a blank body.
- [ ] Sibling-stays-live test passes and demonstrably fails against pre-fix code; boundary-reset and top-level-fallback tests pass; all deterministic (no timers).
- [ ] `pnpm -r typecheck` and `pnpm -r test` clean; no new `any`; no subscription/listener leaks introduced.

### Out of scope

- **Retry/telemetry backends** — logging via `console.error`/`onError` is enough for v1; shipping errors to a server is a later brief.
- **Designer-side (Tauri) error boundaries** — this task is the operator runtime only.
- **Changing component `Render` implementations to not throw** — the boundary is the containment layer; hardening individual components is drift-on-touch, not this task.
- **A global error-toast / notification system** — the per-node fallback + app-level fallback is the scope; a cross-cutting notification surface is separate.
- **Suspense / async-error handling** — error boundaries catch render-phase throws; async/event-handler errors are a different mechanism and out of scope here.

### Risks / gotchas

- **Error boundaries only catch render-phase (and lifecycle) errors** — not errors in event handlers, async callbacks, or `setTimeout`. Don't oversell the boundary as catching everything; the test that throws must throw during `Render`, not in a click handler. State this limitation in the log.
- **`React.StrictMode` double-invokes render in dev** — a component that throws will be invoked twice; ensure the test asserts on the settled fallback, not intermediate double-render artifacts. This is a dev-only concern but can confuse test expectations.
- **Reset-key identity.** If the boundary's reset key isn't stable per node, a re-render of the parent can spuriously reset a boundary (hiding a persistent failure) or fail to reset one that should recover. Tie the key to `node.id` and test both directions (persists when it should, resets when inputs change).
- **Nesting correctness.** Because `renderNode` recurses, a naive wrap could double-wrap or leave a gap. Confirm a throwing *child* is contained without taking down its *parent*, and a throwing *parent* still renders its own fallback (its children never mount) — the test should cover a nested case, not just flat siblings.
- **Fallback must not itself throw.** The fallback renders `node.id`/`node.kind`/`error.message` — all strings already in hand; keep it dependency-free so it can't be the thing that throws inside the boundary.
- **Honesty (CLAUDE.md).** State exactly which render-phase errors are caught vs not, whether the sibling-live test was run against pre-fix code to confirm it fails, and whether the top-level boundary was verified to leave app chrome intact.

## Codex log

## Claude review

## Verdict
