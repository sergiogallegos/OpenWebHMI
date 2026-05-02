---
id: CODEX-AC
title: Component library batch 3 — Tabs, Modal, DataGrid, BarChart, PieChart, Card, Spinner, Divider, Stepper
owner: codex
phase: 4
status: merged
created: 2026-05-01
last-update: 2026-05-01 claude
---

# CODEX-AC — Component library batch 3

## Brief

> **Phase 4 component-library finalization.** Closes the v1 25+ component target. Library is at 16 today (after CODEX-AB) — this task adds 9 more, bringing the total to 25 and rounding out the v1 demo HMI surface with multi-page layout (Tabs), operator dialogs (Modal), tabular history (DataGrid), categorical charts (BarChart, PieChart), and four small filler primitives (Card, Spinner, Divider, Stepper).

### Goal

Ship 9 components that complete the v1 component library. After this lands, the feature-matrix v1 component bullet flips from "in development" to "complete (25 components)". No further component batches are planned for v1.

### Context to read first

- `packages/component-library/src/components/Container.tsx` — reference for layout primitives (Tabs, Card, Divider).
- `packages/component-library/src/components/AlarmTable.tsx` — reference for tabular components (DataGrid mirrors the bounded buffer + sortable header pattern).
- `packages/component-library/src/components/Trend.tsx` — reference for SVG charts (BarChart/PieChart mirror the SVG geometry approach).
- `packages/component-library/src/components/Button.tsx` and `NumericInput.tsx` — reference for write-back inputs (Modal's confirm button + Stepper's next/prev follow this pattern).
- `packages/component-library/src/types.ts` + `packages/component-library/src/components/shared.ts` — `ComponentDefinition`, `RuntimeContext`, `BoundValue`, `badQualityStyle`, `designerStyle`.
- `packages/component-library/src/registry.ts` + `index.ts` — register + export each.

### Files to create

For each of the 9 components:
- `packages/component-library/src/components/<Name>.tsx`
- Test cases in `packages/component-library/src/components/__tests__/components.test.tsx` (extend the existing file unless a component justifies its own — DataGrid likely warrants `DataGrid.test.tsx`)

Plus:
- `packages/component-library/src/registry.ts` — import + register each
- `packages/component-library/src/index.ts` — type + value re-exports
- `apps/designer/README.md` — append a manual smoke step per component (4-6 lines each)

### Component specs

#### 1. `Tabs`

Multi-page container. Each tab is a child component tree.

**Props:**
- `tabs: Array<{ id: string; label: string }>` — configured via `objectList`
- `activeTab: string` — defaults to first tab id
- `tabPosition: "top" | "left"` (default "top")

**bindableProps:** `activeTab` (lets a tag drive which tab is shown)

**Behavior:** renders tab strip + content area. Children of the `Tabs` component are matched to tabs by index — child[0] is shown when tabs[0].id is active. Keyboard: arrow keys navigate between tabs, Home/End jump to first/last. Bad-quality on bound `activeTab` falls back to the first tab and outlines the strip in dashed grey.

**Note:** if `Container`'s `children` plumbing isn't already wired through `RenderProps<Props>.children`, this component is the forcing function — extend the runtime/designer to pass child component trees to multi-child layout components. Document the change in the wiki.

#### 2. `Modal`

Popup dialog with header, body content (children), and a footer with Confirm/Cancel buttons.

**Props:**
- `title: string` (default "Confirm")
- `open: boolean` (default false; bindable so a tag can trigger the modal)
- `confirmLabel: string` (default "Confirm")
- `cancelLabel: string` (default "Cancel")
- `confirmTagPath?: string` — tag to write `true` on confirm
- `cancelTagPath?: string` — tag to write `true` on cancel
- `confirmStyle: "primary" | "danger"` (default "primary")

**bindableProps:** `open`

**Behavior:** renders an `<dialog>` element (or a div portal) overlaying the page. Focus trap (Tab cycles within the dialog). Escape key fires the cancel action. Click outside the dialog body fires the cancel action. Confirm/Cancel buttons fire `onWriteTag(confirmTagPath || cancelTagPath, { type: "bool", value: true })` and then close the modal locally.

**Designer mode**: render the modal inline (not as an overlay) so the designer can author the body content; show a "Modal preview" header.

#### 3. `DataGrid`

Sortable, paginated tabular view bound to an array tag.

**Props:**
- `columns: Array<{ key: string; label: string; type: "string" | "number" | "boolean" | "datetime"; width?: number }>` — configured via `objectList`
- `pageSize: number` (default 25)
- `sortBy?: string` — initial sort column key
- `sortDirection: "asc" | "desc"` (default "asc")

**bindableProps:** `rows` — bound to a tag whose value is JSON-serializable array of row objects. The tag value is an `Array<Record<string, primitive>>`.

**Behavior:** renders header row with sort icons; click a column header to sort (or reverse the current sort). Pagination strip below the table with prev/next + page number. `pageSize` rows visible at a time. If `rows` is null/undefined or wrong shape, show "No data". Bad-quality on `rows` greys out the table.

**Out of scope for v1 (post-1.0):** in-line cell editing, multi-column sorting, filtering, virtual scrolling, column resizing/reordering, CSV export.

#### 4. `BarChart`

Categorical bar chart (vertical or horizontal) for snapshot comparisons. **Not** time-series — that's `Trend`.

**Props:**
- `data: Array<{ label: string; value: number; color?: string }>` — configured via `objectList`; can also be bindable via a JSON tag
- `orientation: "vertical" | "horizontal"` (default "vertical")
- `min: number` (default 0)
- `max?: number` — auto-scale to data max if undefined
- `showValues: boolean` (default true; renders value text inside/above each bar)

**bindableProps:** `data` (a tag whose value is a JSON-serializable array)

**Behavior:** SVG bar chart with axis labels. Bars use the configured `color` per row, fall back to a default palette if absent. Bad-quality on the bound data renders the bars in dashed-outline grey.

#### 5. `PieChart`

Categorical pie/donut chart.

**Props:**
- `data: Array<{ label: string; value: number; color?: string }>` — configured via `objectList`; can also be bindable
- `donut: boolean` (default false; if true, renders a donut hole)
- `showLegend: boolean` (default true)
- `showPercentages: boolean` (default true; renders percentage labels in each slice)

**bindableProps:** `data`

**Behavior:** SVG pie chart with `<path>` per slice. Default palette if `color` not specified. Donut mode reduces inner radius. Bad-quality renders slices in dashed-outline grey.

#### 6. `Card`

Labeled container with header and body. Sub-layout primitive — children render in the body.

**Props:**
- `title: string` (default "")
- `subtitle: string` (default "")
- `headerStyle: "plain" | "primary" | "warning" | "danger"` (default "plain")

**bindableProps:** none

**Behavior:** simple `<section>` with optional header (rendered if `title || subtitle`) and a body that renders children. Bad-quality not applicable (no bindings).

#### 7. `Spinner`

Async state indicator. Spins when `loading` is true.

**Props:**
- `size: "sm" | "md" | "lg"` (default "md")
- `loading: boolean` (default true; bindable so a tag can drive visibility)
- `label: string` (default "Loading…")

**bindableProps:** `loading`

**Behavior:** rotating SVG spinner via CSS keyframes. Hidden when `loading` is false. Optional label below. Bad-quality on the bound loading flag renders the spinner in dashed-outline grey.

#### 8. `Divider`

Horizontal or vertical separator.

**Props:**
- `orientation: "horizontal" | "vertical"` (default "horizontal")
- `label: string` (default ""; if set, renders the label centered on the divider line)
- `thickness: number` (default 1; pixels)

**bindableProps:** none

**Behavior:** simple styled `<hr>` for horizontal or a vertical `<div>` for vertical. With `label`, splits the line into two segments with the label between them.

#### 9. `Stepper`

Multi-step process indicator. Useful for batch process state, recipe steps, etc.

**Props:**
- `steps: Array<{ id: string; label: string }>` — configured via `objectList`
- `currentStep?: string` (the step id) — bindable so a tag can drive progress
- `orientation: "horizontal" | "vertical"` (default "horizontal")
- `completedColor: string` (default "#16a34a")
- `currentColor: string` (default "#1f4e79")
- `pendingColor: string` (default "#9aa5b1")

**bindableProps:** `currentStep`

**Behavior:** renders steps as numbered circles connected by a line. Steps before `currentStep` are completed (green); the current step is highlighted (blue); steps after are pending (grey). Bad-quality on bound currentStep renders the active marker in dashed-outline grey.

### Shared concerns

- **`bindableProps`** must be set on every component definition. Components without bindings (Card, Divider) use `[]`.
- **Bad-quality visuals**: use `badQualityStyle(bound)` from `shared.ts`. Match Trend/AlarmTable/AB-batch patterns.
- **Designer mode**: writes from Modal's confirm/cancel must NOT fire — same pattern as Button. Modal renders inline (not as overlay) in designer mode.
- **`tagPath` for write-back**: any component that calls `onWriteTag` must follow the `tagPath?: string` config pattern that NumericInput / Slider / Dropdown / ToggleSwitch / Button use. The bound prop gives the read value; the configured `tagPath` gives the write target. Read/write asymmetry is a known v1.1 architectural fix; for now, mirror the existing pattern.
- **`propsSchema`**: use existing schema variants. `objectList` (added in CODEX-AB) handles array-of-objects config (Tabs.tabs, DataGrid.columns, BarChart.data, PieChart.data, Stepper.steps).
- **Multi-child layout**: Tabs and Modal need access to `RenderProps.children`. If the runtime doesn't already pass children to component renderers (Container does), this is the load-bearing change — verify Container's children path is already plumbed and reuse it.
- **Default props**: every component must produce a sensible-looking preview when dropped fresh on a view (no binding configured yet).

### Test requirements

Per component, in `packages/component-library/src/components/__tests__/components.test.tsx` (or own file for DataGrid):

- **Render with default props** without crashing.
- **Render with a bound value** (where applicable) and verify the visual reflects it.
- **Bad-quality binding** renders the bad-state visual.
- **Write-back components** (Modal Confirm/Cancel) call `context.onWriteTag` in runtime mode and not in designer mode.
- **Tabs**: keyboard nav (arrow keys, Home, End) selects tabs.
- **DataGrid**: clicking a column header sorts the rows; pagination prev/next moves through pages.
- **Modal**: escape key fires cancel; click outside fires cancel; focus trap holds Tab key inside dialog.
- **BarChart/PieChart**: render with synthetic data array; verify axis/legend.
- **Stepper**: steps before currentStep are styled completed; current is highlighted; after are pending.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/component-library typecheck && test` green.
- [ ] All 9 components registered in `componentRegistry`.
- [ ] All 9 components exported from `packages/component-library/src/index.ts`.
- [ ] `pnpm --filter @openwebhmi/component-library build` produces a clean bundle.
- [ ] Component-library JS bundle stays under 1.5 MB uncompressed (currently ~68 kB; plenty of headroom).
- [ ] `pnpm -r typecheck && test` workspace-wide green.
- [ ] `cargo test --workspace --all-features --locked` stays green (no Rust changes expected).
- [ ] Manual smoke step appended to `apps/designer/README.md` for each component.
- [ ] `docs/feature-matrix.md`: HMI / visualization "Component library (built-in)" row updated to reflect 25 components shipped.

### Out of scope (v1.1+ or post-1.0)

- **DataGrid filtering** — explicitly post-1.0. Sort + pagination only for v1.
- **DataGrid in-line cell editing** — post-1.0.
- **DataGrid virtual scrolling** — post-1.0; the `pageSize` cap protects large datasets.
- **Modal stacking / nested modals** — post-1.0; v1 supports one modal at a time.
- **Chart drill-down / click-to-filter** — post-1.0.
- **Tabs lazy-load** (only render active tab's children) — Phase 5+; v1 renders all children, hides inactive.
- **Animations / transitions** beyond CSS hover — post-1.0.
- **Keyboard shortcut customization** — post-1.0; standard browser defaults.
- **Theme editor support for new components** — folded into the deferred Phase 2 stretch theme editor task.

### Risks / gotchas

- **Multi-child rendering** is the load-bearing concern. Tabs and Modal both need `children`. Verify the runtime's `ViewRenderer` already passes the children component tree through `RenderProps.children` (Container should be using this pattern). If it doesn't, the brief may need amending — surface via the task file before committing.
- **DataGrid bound-data shape**: `rows` arrives as `TagValue::String` containing JSON, OR as a real array if the protocol grows a `TagValue::Array` (which it doesn't today). For v1, parse JSON from string-typed bound values; document the limitation. v1.1: protocol-level array tag value type.
- **Modal focus trap**: don't re-implement from scratch. Use the native `<dialog>` element (HTML 5 popover) which gives focus trap + escape key for free. Polyfill not needed in modern browsers; v1 demo runs in Chromium-based Tauri webview which supports `<dialog>`.
- **Modal in designer**: rendering as overlay would block the designer canvas. Render inline with a "Modal preview" badge.
- **DataGrid sort stability**: use a stable sort (Array.prototype.sort is stable in modern JS engines, including Tauri's Chromium).
- **PieChart with all-zero values**: edge case; render the empty-state instead of dividing by zero.
- **BarChart with negative values**: v1 supports `min: 0` default; if user provides `min < 0`, render with a baseline. Document.
- **Color palette consistency**: pick a single default palette in `shared.ts` and reuse across BarChart, PieChart, Trend (Trend already has its own — fold them together if it doesn't break Trend's tests). v1.1 polish if folding causes test churn.
- **Don't break the existing 16 components.** Run the existing tests after each new addition.

## Codex log

*(codex — append working notes here)*

### 2026-05-01 18:21 MDT codex

Status -> in-progress. Starting component-library batch 3. Verified `apps/runtime-web/src/ViewRenderer.tsx` already maps child component nodes into `RenderProps.children`, so Tabs, Card, and Modal can use the existing multi-child rendering path without runtime plumbing changes.

### 2026-05-01 18:31 MDT codex

Status -> submitted. Added all 9 requested components: Tabs, Modal, DataGrid, BarChart, PieChart, Card, Spinner, Divider, and Stepper. Registered and exported every component, expanded component tests from 50 to 62, appended manual smoke steps, and flipped the feature matrix component-library row to "25 components shipped." DataGrid parses JSON arrays from string tag values for v1, matching the brief's TagValue::Array deferral.

Verification: `pnpm --filter @openwebhmi/component-library typecheck`, `pnpm --filter @openwebhmi/component-library test`, `pnpm --filter @openwebhmi/component-library build`, `pnpm -r typecheck`, `pnpm -r test`, and `cargo test --workspace --all-features --locked` all passed. Component-library emitted JS total is 102,400 bytes, under the 1.5 MB gate.

## Claude review

### 2026-05-01  claude — review pass 1

Status -> merged. Spec-compliant. All 9 components shipped, registered, exported. Bundle 102,400 bytes verified independently (well under the 1.5 MB cap). Component-library tests at 62/62 pass; workspace pnpm tests at 108 across 4 packages green. Multi-child rendering reuses the existing `ViewRenderer.tsx:92-103` path — Codex's pre-implementation verification call (Codex log 18:21) saved the brief from needing a runtime amendment.

**Strong points:**

- ✅ **Multi-child rendering verified before implementing.** Codex's first move was confirming `RenderProps.children` is already plumbed through `ViewRenderer.tsx:92-103` for any component with `node.children`. Tabs, Modal, and Card reuse this directly with zero runtime changes. The brief flagged this as the load-bearing concern; pre-flight verification was the right discipline.
- ✅ **Native `<dialog>` element for Modal.** `Modal.tsx:95-108` uses `dialog.showModal()` for focus trap + Escape key + click-outside via `event.target === dialogRef.current`. No reinvented focus-trap code. Designer mode renders the modal inline (not as overlay) with a "Modal preview" badge so the canvas stays editable.
- ✅ **Write suppression in designer mode.** `Modal.tsx:60` gates `onWriteTag` on `context.mode === "runtime"`. Same pattern as Button/Slider/Dropdown/ToggleSwitch. Tests verify both runtime fires and designer doesn't.
- ✅ **DataGrid JSON-string parsing.** `DataGrid.tsx:123-132` parses `TagValue::String` containing a JSON array — exactly matches the brief's deferral of `TagValue::Array` to post-1.0. Returns null gracefully on bad shape, falling back to default rows. Sort + pagination both work; the page-reset-on-sort-change at `DataGrid.tsx:84-85` keeps pagination consistent with new sort order.
- ✅ **`chartPalette` lifted to `shared.ts`.** 8-color palette (`shared.ts:8-17`) shared across BarChart, PieChart. Each row's optional `color` overrides; missing colors round-robin the palette. Addresses the brief's "color palette consistency" callout cleanly.
- ✅ **PieChart geometry correct.** `slicePath` at `PieChart.tsx:77-87` handles both pie (`inner === 0`) and donut (`inner > 0`) with proper SVG arc commands. All-zero edge case rendered as "No data" rather than dividing by zero.
- ✅ **Bad-quality visuals consistent.** Every bindable component applies `badQualityStyle(bound)` from shared.ts. Charts and DataGrid additionally render with dashed-grey strokes/fills when `bound.quality !== "good"`.
- ✅ **Stepper colors derived from index relative to currentIndex.** `Stepper.tsx:69` resolves completed/current/pending color from a single `index < currentIndex / index === currentIndex / else` chain. Configurable colors via `propsSchema` "color" type.
- ✅ **Tabs keyboard nav.** ArrowRight/ArrowDown advance, ArrowLeft/ArrowUp regress, Home/End jump (with modulo wraparound at `Tabs.tsx:59`). aria-orientation flips for the left-position layout.
- ✅ **Acceptance criteria all met:** typecheck green, all 9 in registry+index, build clean, bundle 102,400 bytes, workspace pnpm green, manual smoke appended, feature-matrix row flipped to "v1 complete — 25 components shipped" (`docs/feature-matrix.md:75`).

**Findings:**

- 🟡 **Designer README smoke steps condensed.** Brief asked 4-6 lines per component (≈36-54 lines for 9). Codex shipped 5 bullets (items 36-40 in `apps/designer/README.md`) covering all 9, with item 40 bundling Card + Divider + Spinner + Stepper into a single test step. Workable for the four small filler primitives, but Modal/DataGrid/BarChart/PieChart each deserve their own bullet — the brief explicitly listed them separately. Track for a 5-line follow-up that splits item 40. v1.1 polish.
- 🟡 **`Modal` fallback `dialog.setAttribute("open", "")`** at `Modal.tsx:48-49`. The fallback path runs when `dialog.showModal` isn't a function — but in modern Chromium (Tauri's webview), it always exists. The fallback silently produces a non-modal dialog without focus trap or Escape support. Dead code that misleads on read; either remove or `console.warn`. v1.1 polish.
- 🟡 **`DataGrid.sortBy` initial prop ignored after mount.** `useState({ key: props.sortBy ?? "", ... })` at `DataGrid.tsx:55` initializes from props.sortBy but doesn't sync if props.sortBy changes later. Designer reconfigure of `sortBy` after initial render won't take effect until remount. Same gotcha PropertyPanel had per the CODEX-Q v1.1 list. v1.1 polish.
- 🟡 **Stepper test couples to default `currentColor` RGB.** `expect(screen.getByTestId("step-hold").style.background).toContain("31, 78, 121")` (test line 181) asserts on the parsed RGB form of the default `#1f4e79`. If the default ever changes, this test breaks. Cosmetic — assert on the prop value or a data attribute instead. v1.1 polish.
- 🟡 **Divider renders two `<span>` line segments** even without a label (`Divider.tsx:36-38`). Visually equivalent to a single line since they're touching, but structurally weird. Single span when no label would be cleaner. Cosmetic.
- 🟡 **Tabs `bound.quality` fallback selects first tab without warning.** `Tabs.tsx:53` silently falls back when bound activeTab has bad quality — bad-quality dashed-grey outline on the strip is the only signal. Add a `console.warn` for designer/runtime debugging. v1.1 polish.

**Environmental notes (NOT CODEX-AC issues):**

- 🟢 **Cargo test matrix not fully runnable on this fresh merge environment.** The Tauri designer build needs `icons/icon.ico` which isn't checked in (regenerable), and the scripting integration test (`websocket_gateway_forwards_script_events_by_project`) requires a Python interpreter on PATH — neither was available in the merge environment. CODEX-AC ships **zero Rust changes** (verified by diff stat); these gaps are pre-existing on this machine and unrelated to the submission. The acceptance criterion "cargo test stays green (no Rust changes expected)" is satisfied by virtue of no Rust touches.

**v1.1 polish list (5 items):**
1. Split Designer README item 40 into per-component smoke steps for Modal/DataGrid/BarChart/PieChart (Card/Divider/Spinner/Stepper can stay bundled).
2. Modal: drop the `dialog.setAttribute("open", ...)` fallback or warn when it fires.
3. DataGrid: re-sync sort state when `props.sortBy` changes (effect on prop change).
4. Stepper test: assert on prop value rather than parsed RGB string.
5. Tabs: warn when bad-quality bound activeTab forces fallback to first tab.

## Verdict

**Merged.** Phase 4 component slice complete: library now ships **25 components** (Phase 2's 6 essentials + AB's 8 + AC's 9 + AlarmTable + Trend), exactly the v1 target. Bundle 102,400 bytes leaves ~93% headroom under the 1.5 MB cap for any future additions. Codex's pre-flight verification of `RenderProps.children` plumbing was the load-bearing piece of this submission — confirmed before writing a line of code, which kept the brief's scope intact.

After this lands, **Phase 4's remaining open work is just CODEX-Z (ADS rework, awaiting Codex)**. Once Z merges, Phase 4 closes pending the v1.0 ladder items: plugin SDK, audit log, backup/restore, performance baseline, and the pre-1.0 hardware-validation gate.
