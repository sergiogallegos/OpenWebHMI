---
id: CODEX-L
title: apps/runtime-web — load views from gateway, render via component library
owner: codex
phase: 2
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
blocked-by: CODEX-I, CODEX-J, CODEX-K
---

# CODEX-L — runtime-web view loading + dynamic rendering

## Brief

> **Blocked by [CODEX-I](CODEX-I-project-store.md), [CODEX-J](CODEX-J-view-schema.md), [CODEX-K](CODEX-K-component-library-v1.md).** Cannot start until the project-store crate ships, the view schema is finalized, and the 6 components exist.

### Goal

The web runtime stops being a hardcoded 4-row demo and becomes a real HMI runtime that loads view definitions from the gateway and renders them via the component library. After this lands, an HMI is *defined by its project file*, not by handwritten React.

### Context to read first

- `apps/runtime-web/src/App.tsx` — the current Phase 1 shape; this task replaces its body.
- `apps/runtime-web/src/gatewayClient.ts` — extend with `view.*` protocol support.
- `crates/project-store/src/types.rs` (CODEX-I) and the TS mirror.
- `packages/component-library/src/registry.ts` (CODEX-K).
- `docs/architecture.md` §4.11 (HMI Runtime).

### Files to create / modify

- `apps/runtime-web/package.json` — depend on `@openwebhmi/component-library`.
- `apps/runtime-web/src/App.tsx` — rewrite: project + view loading, no hardcoded tags.
- `apps/runtime-web/src/ViewRenderer.tsx` — recursive renderer over a `View.root` tree.
- `apps/runtime-web/src/useViewSubscription.ts` — hook that opens a view and tracks the live `View` definition.
- `apps/runtime-web/src/useTagBindings.ts` — hook that, given a list of tag paths, subscribes and returns the latest `BoundValue`s keyed by path.
- `apps/runtime-web/src/gatewayClient.ts` — add `openView(projectId, viewId, callback)` returning unsubscriber; routes `view.definition` events.
- `apps/runtime-web/src/__tests__/ViewRenderer.test.tsx` — render a small synthetic view tree, assert correct components mount + receive bindings.

### Behavior

#### Boot

1. Read `import.meta.env.VITE_PROJECT_ID` (default: `"phase1-demo"`).
2. Read `import.meta.env.VITE_INITIAL_VIEW` (default: `"home"`).
3. Connect the gateway client.
4. Send `project.subscribe { project_id }`. (Server pushes `project.changed` whenever the project version bumps.)
5. Send `view.open { project_id, view_id: initial_view }`. Server replies with `view.definition`.
6. Render the view via `ViewRenderer`.

#### Tag binding lifecycle

`ViewRenderer` walks the view tree, collects every `binding.source.kind === "tag"` path, and passes them to `useTagBindings`. The hook subscribes to all paths at once and returns a `Record<string, BoundValue | undefined>`. As the view tree changes (project hot reload), the hook diffs paths and adjusts subscriptions.

#### View hot reload

When `project.changed` arrives with a non-`"deleted"` action that affects a currently-open view, the runtime sends a new `view.open` for it. The replacement `view.definition` causes `ViewRenderer` to re-render with the new tree. Bindings that no longer exist unsubscribe automatically; new bindings subscribe.

#### Tag write path

When a component calls `context.onWriteTag(path, value)`, the hook routes the call to `gatewayClient.writeTag(path, value)`. The gateway already supports `tag.write` from Phase 0; no protocol change here.

#### Connection / loading states

- **Connecting**: render a minimal loading state.
- **Connected, no view loaded yet**: render "loading view…".
- **View loaded, gateway disconnected**: keep rendering the last view tree, but show a banner. Bindings hold their last `BoundValue` with `quality` flipped to `"bad"` (the `gatewayClient` already does this when the WebSocket drops).
- **View not found** (server returns `error` with code `view.not_found`): render a clear error page with the view id and project id.

### Test requirements

`ViewRenderer.test.tsx`:
- Render a synthetic 2-level view tree (Container > [Label, ValueDisplay]) and assert each component receives the correct props + bindings.
- Update the view tree (simulate hot reload), assert the renderer reflects the change without unmounting unrelated subtrees.
- A view tree with a binding that has no live value (yet) renders the component with `bindings[prop] === undefined`.
- A view tree with a `Quality::Bad` bound value passes `quality: "bad"` through to the component.

`gatewayClient` extensions get unit tests in `gatewayClient.test.ts`:
- `openView` registers a callback and routes `view.definition` to it.
- Calling the unsubscriber stops further `view.definition` deliveries.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/runtime-web typecheck` exits 0.
- [ ] `pnpm --filter @openwebhmi/runtime-web test` runs vitest, all tests above pass.
- [ ] `pnpm --filter @openwebhmi/runtime-web build` produces a `dist/` bundle.
- [ ] **Manual smoke**: with the gateway running with `--project examples/projects/phase1-demo/project.toml` and a `home` view definition added, the page renders that view's components live, bound to the same `rockwell-1/Pressure` etc tags from Phase 1. Killing the simulator flips affected components into bad-quality visual; restarting it restores them.
- [ ] No hardcoded tag paths or component types in `App.tsx`. Everything comes from the view definition.

### Out of scope

- Multi-page navigation between views (Phase 2 stretch).
- Modal dialogs / popups.
- Visual canvas (Phase 2 stretch).
- Authentication / login UI (Phase 3).

### Risks / gotchas

- **Don't memoize `useTagBindings` by path-list identity.** Memoize by sorted-path-list contents — tree-walks produce arrays in different orders, you don't want subscription thrash on re-render.
- **Hot reload diffing**: when a view definition changes, the renderer should preserve component instances where possible (React's reconciliation does this if `key` props are stable). Use the component `id` from the view tree as the React `key`.
- **`onWriteTag` should not block the render path.** Components dispatch writes asynchronously; the renderer must keep up with incoming `tag.update` events while a write is in flight.
- **Don't introduce a state library** (Redux, Zustand) for this scope. `useState` + the existing client subscriptions are sufficient. Phase 4 plugin SDK can revisit if components need cross-tree communication.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
