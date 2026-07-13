---
id: CODEX-CC
title: Designer editing integrity — script-source cross-contamination, uneditable JSON props, input races, reconnect
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CC — Designer editing integrity

## Brief

> A bundle of designer data-integrity bugs where the editor silently corrupts or discards operator edits. The headline one (a) is the most serious single frontend bug in the tree: switching scripts while a load is in flight can write script A's source into script B's artifact. The rest are editability, focus-race, reconnect, and unique-id defects. Fix each with a regression test that fails without the fix. These are correctness bugs, not features — no new UI surface.

### Goal

- Switching scripts A→B never writes A's source into B's `script_source` artifact, and never silently drops an in-flight edit.
- JSON-list props (Dropdown options, Tabs tabs, DataGrid columns, Button writeValue) stay editable through intermediate invalid-JSON keystrokes.
- A `NumericInput`/`Slider` being edited by an operator isn't wiped or jumped by a republished value.
- `DesignerClient` detects disconnect, reconnects, and never misroutes a correlated response after a failed send; `ScriptList` surfaces errors instead of throwing unhandled rejections.
- Duplicating a component twice produces unique ids (no React key collision, no `findNode` ambiguity).

### Context to read first

- `apps/designer/src/modules/ScriptEditor.tsx` — the load effect (lines 31-58) resolves `loadScriptSource` and sets `source`; the autosave effect (lines 60-78) fires 300 ms after `saveState` goes `unsaved` and writes `source` to `script.id`. The bug: on script switch the load effect resets `saveState` to `"saved"` (line 34) and the header renders the new `script.id` (line 96) immediately, but `source` (line 108 `value={source}`) still holds the *previous* script's text until the async load resolves. Typing during that window sets `saveState = "unsaved"` (line 116) with the wrong `source`; the autosave (line 67 `client.saveScriptSource(projectId, script.id, source)`) then writes A's text into B. And switching within 300 ms of a keystroke tears down the autosave timer (cleanup line 77) before it fires, discarding the edit.
- `apps/designer/src/modules/PropertyPanel.tsx` — the `objectList` textarea (lines 203-218) is fully controlled by `JSON.stringify(value)` (line 206) and on `JSON.parse` failure calls `onChange(value)` with the **old** value (line 212), so any keystroke that makes the JSON momentarily invalid snaps the textarea back. Also `draftView` resets from the `view` prop on every `view` change (effect lines 38-40), discarding edits typed during a save round-trip; and the parent passes an inline arrow `onChange` (`App.tsx` lines 330-337) whose identity changes every render, restarting the debounce effect (lines 42-52, `onChange` in deps).
- `packages/component-library/src/components/NumericInput.tsx` (lines 43-45) and `Slider.tsx` (lines 57-59) — `useEffect(() => setDraft(...), [value])` with no focus/dirty guard, so a value republished every 500 ms overwrites the draft mid-edit / jumps the slider thumb mid-drag.
- `packages/component-library/src/packs/material/index.tsx` — the material pack **duplicates** these components: `materialNumericInput` (from line 107, the `useEffect` at line 113) and `materialSlider` (from line 156, `useEffect` at line 165) copy the same buggy sync. Any fix to the base components must also apply here — or refactor the material pack to wrap the base component render rather than copy it (see gotchas).
- `apps/designer/src/lib/designerClient.ts` — `connect` (lines 122-137) sets connection state once and never updates it on close/reconnect; `send` (lines 323-328) **throws synchronously** when the socket isn't `OPEN`. Correlation queues (`pendingProjectLoads` FIFO line 92, `pendingSaves`/`pendingArtifactReads`/`pendingUserLists`) push the pending entry **before** calling `send` (e.g. `loadProject` lines 177-181, `saveArtifact` lines 218-228, `loadScriptSource` lines 234-242) — so a synchronous `send` throw leaves an orphaned pending entry that misroutes the next response. `rejectNext` (lines 396-407) on error only drains `pendingProjectLoads` and `pendingSaves`, **not** `pendingArtifactReads` or `pendingUserLists` — so a `loadScriptSource` for a missing script hangs forever.
- `apps/designer/src/modules/ScriptList.tsx` — `addScript` (lines 22-37), `renameScript` (lines 39-52), `deleteScript` (lines 54-65) all `await` client calls with **no `try/catch`**; a rejected/hanging promise is an unhandled rejection with no operator feedback.
- `apps/designer/src/lib/viewMutations.ts` — `withFreshIds` (lines 186-192) sets `id: `${node.id}-copy`` (line 189); duplicating a component twice yields two nodes with the same `-copy` id → React key collision and `findNodeInTree` (lines 194-205) returns the first match, so selection/edits target the wrong node.
- `apps/designer/src/App.tsx` — connection state (lines 30-85), the `connect` flow (lines 61-85), and the inline `onChange` arrows passed to the editor/`PropertyPanel` (lines 320-337).
- [`docs/agents/notes/binding-write-asymmetry.md`](../notes/binding-write-asymmetry.md) — the `tagPath?: string` write convention; the NumericInput/Slider fixes must not disturb it.

### Files to create / modify

**(a) Script-source cross-contamination — `apps/designer/src/modules/ScriptEditor.tsx`:**
- Track the id of the script whose source is currently loaded (e.g. a `loadedScriptId` state/ref) distinct from the selected `script.id`.
- Gate the autosave effect on `loadedScriptId === script.id` — never autosave while the loaded source belongs to a different script than the one selected.
- Don't render the editor as editable / don't let `onChange` flip `saveState` to `unsaved` until the source for the *current* `script.id` has actually loaded. Do not reset `saveState` to `"saved"` on switch until the correct source arrives.
- On switch with a pending edit, either flush the pending autosave for the outgoing script first, or preserve the edit so it isn't silently discarded. Choose one and state which in the log.

**(b) Uneditable JSON props + draftView reset — `apps/designer/src/modules/PropertyPanel.tsx` + `apps/designer/src/App.tsx`:**
- Give the `objectList` `PropInput` local raw-draft text state; render the raw draft, not `JSON.stringify(value)`, while editing. Parse on blur (or on valid parse) and only call `onChange` with valid parsed JSON. On invalid intermediate text, keep the draft — never call `onChange(value)` with the stale value.
- Fix the `draftView` reset (effect lines 38-40): don't clobber in-progress edits when the `view` prop updates from a save round-trip. Distinguish "selection changed" (reset is correct) from "same selection, view object re-identified by a save" (preserve draft). Reset on `selectedComponentId` change, not on every `view` identity change.
- Stabilize the parent `onChange` handlers in `App.tsx` (lines 320-337) with `useCallback` so the `PropertyPanel` debounce effect isn't restarted every render.

**(c) Focus/dirty guard — `packages/component-library/src/components/NumericInput.tsx` + `Slider.tsx` + the material pack:**
- Skip the `value → draft` sync effect while the control is focused (NumericInput) / being dragged (Slider). Track focus/drag with a ref set on `onFocus`/`onPointerDown`/`onBlur`/`onPointerUp` (or the existing DOM events) and short-circuit the effect when dirty.
- Apply the identical fix in `packages/component-library/src/packs/material/index.tsx` (`materialNumericInput` ~line 113, `materialSlider` ~line 165). **Preferred:** refactor the material pack entries to wrap/reuse the base component `Render` (or share a common hook) instead of copying the logic, so this class of bug can't diverge again. If wrapping is impractical, apply the same guard to both copies and flag the duplication in the log as tech debt.

**(d) Reconnect + correlation integrity — `apps/designer/src/lib/designerClient.ts` + `apps/designer/src/modules/ScriptList.tsx` + `apps/designer/src/App.tsx`:**
- Add disconnect detection and reconnect to `DesignerClient`; update a connection-state signal on open/close/reconnect (the runtime-web `gatewayClient.ts` `ConnectionState` + `onStateChange` pattern, lines 43-55 / 138-143 / 582-587, is the reference shape to mirror).
- Push the pending correlation entry **only after** `send` succeeds — wrap `send` so a throw doesn't leave an orphaned pending entry (or catch and reject the just-created promise). Apply to every enqueue site (`loadProject`, `saveArtifact`, `loadScriptSource`, `listUsers`/`upsertUser`/`deleteUser`, `deleteArtifact`).
- Make `rejectNext` (lines 396-407) drain **all** pending kinds on error, including `pendingArtifactReads` and `pendingUserLists`, so a `loadScriptSource` for a missing script rejects instead of hanging. Preserve the existing `rejectPending` on-close behavior (lines 409-424).
- Wrap `ScriptList` `addScript`/`renameScript`/`deleteScript` in `try/catch` and surface the error (a status/alert region), no unhandled rejections.

**(e) Unique duplicate ids — `apps/designer/src/lib/viewMutations.ts`:**
- Generate a unique id in `withFreshIds` (lines 186-192) — e.g. a monotonic suffix, `crypto.randomUUID()`, or an existing-id-collision check — so duplicating twice yields distinct ids and children get fresh ids too (the recursion at line 190 must not reintroduce collisions). Ensure ids are unique across the whole tree, not just locally.

### Behavior

- Switch A→B while B's source load is in flight, type into the editor: the autosave never targets B with A's text, and A's text is never written to B's `script_source`. Switching back to A preserves (or has already saved) A's edit — no silent loss.
- Typing a Dropdown option / Tabs tab / DataGrid column / Button writeValue through a momentarily-invalid JSON state leaves the textarea showing what was typed; `onChange` fires only with valid JSON.
- A `NumericInput` with focus, or a `Slider` mid-drag, is not overwritten/jumped by a value republished every 500 ms; on blur/release the latest external value re-syncs.
- Killing the gateway connection updates the designer connection state and triggers reconnect; a `send` that fails while disconnected doesn't corrupt the next response's routing; a `loadScriptSource` for a missing script rejects (surfaced), not hangs; `ScriptList` actions show an error instead of throwing.
- Duplicating a component twice yields three components with three distinct ids.

### Test requirements

- **Vitest, added to the closest existing designer/component-library spec files** — don't fragment. Every test must **fail against pre-fix code** (run once on `main` to confirm) per the "test is NOT VALID if it passes without the fix" rule.
- **(a)** With a fake `DesignerClient` whose `loadScriptSource` resolves on a controllable deferred: select A (loaded), switch to B (load pending), type, resolve B's load; assert `saveScriptSource` was **never** called with `(projectId, "B", <A's text>)`, and the A edit is not silently lost. Drive the deferred resolution explicitly — **no `setTimeout` waits**; use fake timers for the 300 ms autosave.
- **(b)** Render `PropertyPanel` with an `objectList` prop; fire keystrokes that pass through invalid JSON (e.g. deleting a closing `]`) then back to valid; assert the textarea retained the typed text throughout and `onChange` fired only with valid parsed arrays. Separately assert a `view`-prop re-identify (same selection) does not wipe an in-progress draft.
- **(c)** Render `NumericInput` in runtime mode, focus it, set the draft, then push a new bound `value`; assert `draft` (the input's displayed value) is unchanged while focused and re-syncs on blur. Analogous drag test for `Slider`. Cover **both** the base component and the material-pack variant (if not refactored to share).
- **(d)** Fake socket: enqueue a `loadScriptSource`, deliver an `error` frame; assert the promise rejects (not hangs). Enqueue with the socket closed; assert no orphaned pending entry misroutes the next response. Assert a disconnect flips the connection-state signal and reconnect is attempted (deterministic — inject the socket, no wall-clock waits). `ScriptList` action error surfaces without an unhandled rejection.
- **(e)** Duplicate a node twice via `viewMutations`; assert all resulting ids are unique and `findNode` resolves each to the intended node.
- `pnpm -r typecheck` + `pnpm -r test` clean. No `any` abuse (`unknown` + narrowing for dynamic JSON). All listeners/subscriptions/timers removed on unmount.

### Acceptance criteria

- [ ] (a) Autosave gated on the loaded script id matching the selected id; A's source can never be written to B; in-flight edits flushed or preserved on switch (state which in the log).
- [ ] (b) `objectList` prop editable through invalid intermediate JSON; `onChange` only with valid JSON; `draftView` no longer clobbered by save-round-trip `view` re-identify; parent `onChange` handlers stabilized with `useCallback`.
- [ ] (c) Focus/dirty guard on NumericInput and Slider draft sync, applied to the material pack too (or the pack refactored to reuse the base render — flagged either way).
- [ ] (d) `DesignerClient` reconnect + connection-state updates; pending entry pushed only after `send` succeeds; `rejectNext` drains all pending kinds; `ScriptList` actions `try/catch` with surfaced errors.
- [ ] (e) `withFreshIds` produces tree-unique ids; double-duplicate yields distinct ids.
- [ ] Each fix has a regression test that fails without the fix; `pnpm -r typecheck` + `pnpm -r test` clean; no `any` abuse; deterministic tests (no `setTimeout` waits).
- [ ] Codex log states the (a) flush-vs-preserve choice and whether the material pack was refactored to share or patched in two places.

### Out of scope

- **Collaboration locking / OT / CRDT.** The "last write wins" warning (App.tsx ~line 93) stays; this task fixes single-editor corruption, not multi-editor concurrency.
- **Redesigning the property panel schema system** or adding new prop types. Fix editability of the existing `objectList`, don't re-architect `PropInput`.
- **Monaco/editor version or dependency changes.** The Node-22 monaco gating is settled elsewhere; don't touch it.
- **Gateway-side script artifact storage.** This is a client-integrity task; the gateway contract is unchanged.
- **Runtime-web render architecture** (owned by CODEX-CB). Don't refactor the runtime value pipeline here; the NumericInput/Slider focus guard is a component-local fix that both surfaces share.
- **Reworking the whole `DesignerClient` correlation model** into request-ids everywhere. Fix the FIFO/enqueue-ordering and the rejection-coverage bugs; a full request-id redesign is a separate brief if warranted.

### Risks / gotchas

- **(a) is the load-bearing fix — get the id gate exactly right.** The corruption happens because `saveState` and the header trust `script.id` while `source` still holds the old script. The invariant to assert: *never autosave unless the source in state was loaded for the currently-selected id.* A `loadedScriptId` ref compared against `script.id` inside the autosave effect is the minimal shape. Test the in-flight-load race explicitly with a controllable deferred, not a timer.
- **Material pack duplication is a repeat offender.** NumericInput/Slider bugs exist in two places because the pack copied the render. Prefer wrapping the base `Render` so the fix lives once. If the pack's material styling makes wrapping awkward, at minimum share the focus-guard hook. Whatever you choose, a divergence here will re-break silently — flag it.
- **`useCallback` stabilization must not stale-close.** Stabilizing `App.tsx` `onChange` handlers is required for the debounce, but they close over `selectedComponentId`/`saveView`; include the right deps so they don't capture stale state. The `PropertyPanel` debounce effect currently lists `onChange` in deps (line 52) precisely because it wasn't stable — fix the cause, keep the dep.
- **`send` throwing is load-bearing behavior.** `send` throws by design when disconnected (line 325). The fix isn't to stop it throwing — it's to not leave an orphaned pending entry when it does. Push-after-success (or catch-and-reject-the-just-created-promise) at every enqueue site. Miss one and that path still misroutes.
- **`rejectNext` FIFO semantics.** On an `error` frame the client currently rejects the oldest project-load or save. Extending it to artifact-reads/user-lists must keep a deterministic order and not double-reject an already-settled promise. Mirror the `rejectPending` drain style (lines 409-424) but for the single-error case.
- **Unique ids across the whole tree.** `withFreshIds` recurses (line 190); a naive `${id}-copy` reused per child reintroduces collisions. Generate genuinely unique ids (uuid or a tree-wide collision check), and confirm nested children of a duplicated subtree are all fresh.
- **jsdom focus behavior.** `document.activeElement` and focus/blur events work in jsdom but require the element to be in the document and, for some, a real focus() call. Use `@testing-library/react` `fireEvent.focus`/`.blur` and `userEvent` where needed; assert on the input's displayed value, not internal state.
- **Honesty discipline.** The Codex log states which bugs got behavior tests vs compile-only verification (all five should get behavior tests), the flush-vs-preserve decision for (a), and the material-pack refactor-vs-duplicate decision for (c).

## Codex log

## Claude review

## Verdict
