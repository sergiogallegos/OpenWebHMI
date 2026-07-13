---
id: CODEX-CB
title: Runtime subscription + render architecture — scale to hundreds of sub-second tags
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CB — Runtime subscription + render architecture

## Brief

> Re-architect the runtime-web value pipeline so a `tag.update` for one tag re-renders only the components bound to that tag, not the entire view tree. Today every update replaces one `Record<string, BoundValue>` held in a single `useState` at `App` level, which re-renders `App` → `ViewRenderer` → `renderNode` recursively across every node, with a freshly-allocated `context` object passed to each node defeating any memoization. Move to per-tag subscription granularity (an external store consumed via `useSyncExternalStore`, keyed per tag path), memoize node rendering, stabilize the `context` object, and batch bursts of `tag.update` frames per animation frame. **Structural refactor; the `ViewRenderer` public prop shape stays stable.**

### Goal

A 300-tag view refreshing at 500 ms drives roughly 600 `tag.update` frames/second. Today each frame re-renders the whole tree (300+ component instances) because all values live in one `useState` object at `App` and `context` is reallocated per node. After this task, a single `tag.update` re-renders only the component instance(s) whose bindings reference the changed path; a burst of frames arriving in one WebSocket message-loop turn is coalesced into one render pass. The demo HMI stays interactive (no input jank on a `NumericInput`/`Slider` while unrelated tags churn) and the `ViewRenderer` props contract is unchanged so `App.tsx` wiring needs no structural rewrite.

### Context to read first

- `apps/runtime-web/src/useTagBindings.ts` — the current single-`useState` aggregator. `boundValues` is one `Record<string, BoundValue | undefined>` (lines 12-14); every subscription callback does its own `setBoundValues((current) => ({ ...current, [path]: update }))` (lines 31-34) — one setState per tag per frame, each allocating a new object.
- `apps/runtime-web/src/ViewRenderer.tsx` — `renderNode` (lines 67-110) walks the tree recursively with no `React.memo`; the `<definition.Render>` call (lines 100-108) passes `context={{ mode: "runtime", liveValues: boundValues, ...runtime }}` (line 105) — a new object literal per node per render, plus `liveValues: boundValues` threads the whole value map into every node, so any value change invalidates every node's `context`.
- `apps/runtime-web/src/useViewSubscription.ts` — sibling hook; shows the `useState` + `useEffect` subscription pattern and cleanup contract to mirror (subscribe in effect, return unsubscribe).
- `apps/runtime-web/src/gatewayClient.ts` — the subscription API this store consumes. Key methods: `subscribe(path, callback)` (lines 152-168) returns an unsubscribe; `dispatchTagUpdate` (lines 434-450) fans one update out to all path callbacks; `lastTagUpdates` (line 97) caches the latest per-path value and replays it on new subscribe (lines 158-161); `markBindingsBad` (lines 557-573) pushes synthetic `quality: "bad"` updates on disconnect. The store must not lose these behaviors.
- `apps/runtime-web/src/App.tsx` — `useTagBindings(client, tagPaths)` at line 92 hands `boundValues` to `<ViewRenderer boundValues={...}>` at line 146. Confirm the refactor keeps that call site working (the prop may change type, but the wiring shape stays).
- [`docs/agents/notes/binding-write-asymmetry.md`](../notes/binding-write-asymmetry.md) — the write path (`writeTag`) is orthogonal to this read-path refactor; don't disturb the `tagPath?: string` write convention.

### Files to create / modify

1. **Create** a per-tag value store module (suggested `apps/runtime-web/src/tagValueStore.ts`) that wraps `GatewayClient` and exposes, per tag path, a `useSyncExternalStore`-compatible `subscribe(path, onStoreChange)` + `getSnapshot(path)` pair. Internally it holds the latest `BoundValue | undefined` per path, subscribes to `client.subscribe(path, …)` once per path (ref-counted so N components binding the same path share one gateway subscription), and notifies only the listeners registered for that path when its value changes. Batch: buffer incoming updates and flush notifications once per `requestAnimationFrame` (fall back to a microtask/`queueMicrotask` when `requestAnimationFrame` is unavailable, e.g. jsdom) so a burst of frames in one loop turn triggers one notification round.

2. **Create** a hook (suggested `apps/runtime-web/src/useTagValue.ts`) — `useTagValue(store, path): BoundValue | undefined` built on `useSyncExternalStore(store.subscribe(path), () => store.getSnapshot(path))`. A component instance that reads exactly one tag re-renders only when that tag's snapshot changes. `getSnapshot` must return a referentially stable value when unchanged (return the cached `BoundValue` object, not a fresh one) so `useSyncExternalStore` doesn't loop.

3. **Modify** `apps/runtime-web/src/ViewRenderer.tsx`:
   - Stop threading `boundValues` (the whole map) through `renderNode` and into every `context`. Instead resolve each node's bound values through the per-tag hook at the leaf that consumes them, or through a memoized per-node subscriber wrapper so that only nodes whose bound paths changed re-render.
   - Wrap the per-node render in a memoized component (`React.memo` with a stable prop set — `node`, resolved `props`, the stabilized `context`) so unrelated nodes skip re-render.
   - Stabilize `context`: build the `runtime` context object **once** (memoized/`useMemo` or module-stable) and reuse the same reference for every node. `context` must no longer carry `liveValues: boundValues` (that is the exact field defeating memoization); if a component genuinely needs cross-tag lookups, expose a stable accessor function on `context`, not the mutable map.
   - Keep `ViewRenderer`'s exported prop names and the `getComponentDefinition` / `bindingsForNode` / `bindingFromSource` behavior intact where they don't conflict with the above — the component contract (`props`, `bindings`, `context`) that every `ComponentDefinition.Render` consumes must still be satisfied.

4. **Modify** `apps/runtime-web/src/useTagBindings.ts` (or replace it with the store-backed path) — the `writeTag` passthrough (lines 45-47) stays; only the read/aggregation half changes. If `useTagBindings` is retained as a thin compatibility shim over the store, say so in the log.

5. **Do NOT** modify:
   - `gatewayClient.ts` subscription semantics (ref-count, `lastTagUpdates` replay, `markBindingsBad`) — consume them, don't rewrite them. A small additive method is acceptable if justified in the log; a behavior change is out of scope.
   - The `ComponentDefinition.Render` public signature in `packages/component-library` — the runtime satisfies its existing `context` contract; it does not get a breaking change from this task.
   - The write path (`tag.write`) or the `tagPath?: string` convention.

### Behavior

- One `tag.update` for path `A` re-renders only the component instance(s) bound to `A`. Instances bound to `B`, `C`, … do not re-render.
- A burst of `tag.update` frames delivered in one WebSocket message-loop turn (or one animation frame) is coalesced: each affected component re-renders at most once for that burst, not once per frame.
- Bad-quality propagation on disconnect (`markBindingsBad`) still reaches bound components (they render the stale/bad state).
- Replay-on-subscribe still works: a component mounting after a value already arrived immediately sees the last value.
- `ViewRenderer`'s public props are unchanged in name; `App.tsx` continues to wire the runtime without a structural rewrite.
- No `useSyncExternalStore` snapshot-identity loops (stable `getSnapshot`).

### Test requirements

- **Add to the existing runtime-web test suite** (Vitest). Do not fragment into many new files — extend the closest existing spec, or add one focused spec if none fits.
- **Render-count isolation test:** mount a view with at least two tag-bound components (e.g. two `NumericInput`/`Text` instances on different paths). Instrument render counts (a render-counting wrapper / `vi.fn` in the component body, or `@testing-library/react` with a counting render prop). Push a `tag.update` for path `A` through the fake gateway; assert the `A`-bound component's render count incremented and the `B`-bound component's did **not**. This test must **fail against the pre-refactor code** (where any update re-renders the whole tree) — run it against `main` once and confirm it fails before the fix, per the "your test is NOT VALID if it passes without the fix" rule.
- **Batching test:** deliver multiple `tag.update` frames for the same path (or several paths) within one flush window; assert the bound component re-rendered once, not once per frame. Drive the flush **deterministically** — inject/fake the `requestAnimationFrame` scheduler or use Vitest fake timers; **no `setTimeout`/wall-clock waits**.
- **Replay + bad-quality tests:** a component mounting after a value arrived shows it; a disconnect marks bound components bad. Reuse the existing fake `WebSocketLike`/`GatewayClient` test harness rather than inventing a new mock.
- `pnpm -r typecheck` and `pnpm -r test` clean. No `any` in the new store/hook (`unknown` + narrowing where a type is genuinely dynamic).

### Acceptance criteria

- [ ] Per-tag value store exists; N components binding one path share one `client.subscribe` (ref-counted), and each path notifies only its own listeners.
- [ ] `useSyncExternalStore`-based hook with a referentially-stable `getSnapshot` (no update loop).
- [ ] `context` object is stabilized (one reference reused across nodes) and no longer carries the full `liveValues` map; per-node render is memoized.
- [ ] Single-update isolation proven by a render-count test that **fails without the refactor**.
- [ ] Burst batching proven by a deterministic test (faked scheduler / fake timers, no wall-clock waits).
- [ ] Replay-on-subscribe and disconnect bad-quality behaviors preserved (tests).
- [ ] `ViewRenderer` public prop names unchanged; `App.tsx` wiring still compiles without a structural rewrite.
- [ ] `pnpm -r typecheck` + `pnpm -r test` clean; no `any` abuse; all subscriptions/listeners removed on unmount.
- [ ] Codex log states the batching mechanism chosen (rAF vs microtask) and the jsdom fallback, and whether `useTagBindings` was retained as a shim or replaced.

### Out of scope

- **Rewriting `gatewayClient.ts` subscription internals.** Consume `subscribe` / `lastTagUpdates` / `markBindingsBad` as they are. Additive-only if justified.
- **Changing the `ComponentDefinition.Render` public contract** in `packages/component-library`. This is a runtime wiring refactor, not a component-API break.
- **Write-path / `tagPath` changes** (owned by the component-library briefs).
- **Virtualization / windowing of off-screen components.** A separate optimization if a specific screen needs it; not required to hit the render-isolation goal.
- **Designer preview render path.** This task is runtime-web only; the designer's live preview is a separate surface.
- **Trend/history batching.** `Trend` reads history via a different path (`onReadHistory`); leave it alone unless it shares the value store trivially.

### Risks / gotchas

- **`useSyncExternalStore` snapshot identity.** `getSnapshot` must return the same object reference when the value hasn't changed, or React 18 throws "getSnapshot should be cached" / loops. Cache the per-path `BoundValue` and only replace it when the value actually changes.
- **Ref-counted subscription teardown.** When the last component for a path unmounts, the store must call the gateway unsubscribe; when the first mounts, subscribe once. Off-by-one here leaks gateway subscriptions or unsubscribes a still-watched path. Mirror `gatewayClient`'s own `wasEmpty`/`size === 0` ref-count shape (lines 152-187).
- **Batching vs test determinism.** `requestAnimationFrame` doesn't exist/behave predictably in jsdom. Make the scheduler injectable (default `requestAnimationFrame`, override in tests) or fall back to `queueMicrotask`; drive tests with fake timers/scheduler, never `setTimeout` sleeps (repo "no flaky tests" rule).
- **`context.liveValues` consumers.** Some components may read `context.liveValues` for cross-tag logic today. Grep the component library before removing the field; if a real consumer exists, replace it with a stable accessor on `context` rather than re-threading the mutable map. Document any consumer found.
- **Memoization correctness.** `React.memo` with an object prop that's rebuilt each render is a no-op. The whole point is the stabilized `context` + a `node` reference that only changes when the node's definition/props change. If the memo comparison is wrong, the test will still catch cross-tag re-renders — trust the render-count test.
- **Don't regress the write path.** `writeTag` (useTagBindings 45-47) and the `tagPath` fallback are untouched by this task; keep them working.
- **Honesty discipline.** The Codex log states exactly what re-renders after the change (measured render counts), the batching mechanism, and any behavior that was *not* preserved or was deferred.

## Codex log

## Claude review

## Verdict
