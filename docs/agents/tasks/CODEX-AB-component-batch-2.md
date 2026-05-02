---
id: CODEX-AB
title: Component library batch 2 — Gauge, ProgressBar, Slider, Dropdown, ToggleSwitch, Button, MultiState, AlarmBanner
owner: codex
phase: 4
status: merged
created: 2026-05-01
last-update: 2026-05-01 claude
---

# CODEX-AB — Component library batch 2

## Brief

> **Phase 4 component-library expansion.** v1 target is 25+ components; we have 8 today (Label, ValueDisplay, NumericInput, Indicator, Image, Container from CODEX-K + AlarmTable from CODEX-R + Trend from CODEX-P). This task adds 8 more — covering output displays (Gauge, ProgressBar, MultiState, AlarmBanner) and input controls (Slider, Dropdown, ToggleSwitch, Button) — bringing the library to 16. A future CODEX-AD-or-similar can finish the journey to 25+.

### Goal

Ship 8 production-quality components that round out the v1 "demo HMI" surface. Each component is independently usable, follows the existing `ComponentDefinition` shape, and has unit-test coverage. Inputs write back through `context.onWriteTag`; AlarmBanner subscribes through `context.onSubscribeAlarms`.

### Context to read first

- `packages/component-library/src/components/NumericInput.tsx` — reference pattern for write-back inputs (commit on Enter/blur, debounced display sync). Mirror this for Slider/Dropdown/ToggleSwitch/Button.
- `packages/component-library/src/components/Indicator.tsx` — reference pattern for a single-output bool/state display.
- `packages/component-library/src/components/AlarmTable.tsx` — reference pattern for alarm-event subscriptions; AlarmBanner reuses the `onSubscribeAlarms` plumbing.
- `packages/component-library/src/components/Trend.tsx` — reference pattern for SVG-based output components (gauge math is similar).
- `packages/component-library/src/types.ts` — `ComponentDefinition`, `RuntimeContext`, `BoundValue`, `RenderProps`.
- `packages/component-library/src/registry.ts` — register each new component here.
- `packages/component-library/src/index.ts` — export each new component's type + value.
- `packages/component-library/src/components/shared.ts` — shared `baseFont`, `designerStyle` helpers.

### Files to create

For each of the 8 components:
- `packages/component-library/src/components/<Name>.tsx`
- An entry in `packages/component-library/src/components/__tests__/components.test.tsx` (extend the existing file rather than creating one per component — follow the AlarmTable test split only if a component justifies its own file)

Plus:
- `packages/component-library/src/registry.ts` — import + register each
- `packages/component-library/src/index.ts` — type + value re-exports
- `apps/designer/README.md` — append a manual smoke step that adds each component to the demo project's `home` view (one paragraph per component, 4-6 lines each)

### Component specs

#### 1. `Gauge`

Circular SVG gauge (270° arc) for a single numeric tag.

**Props:**
- `min: number` (default 0)
- `max: number` (default 100)
- `units: string` (default "", e.g. "PSI", "°C")
- `lowWarn: number | undefined` (band threshold)
- `highWarn: number | undefined`
- `lowAlarm: number | undefined`
- `highAlarm: number | undefined`

**bindableProps:** `value`

**Behavior:** SVG arc from `min` to `max`, needle at `value`. Color the band segments: green (normal), yellow (warn), red (alarm). Center label shows the numeric value + units. Bad-quality binding renders the needle in dashed grey.

#### 2. `ProgressBar`

Horizontal or vertical fill bar.

**Props:**
- `min: number` (default 0)
- `max: number` (default 100)
- `orientation: "horizontal" | "vertical"` (default "horizontal")
- `showLabel: boolean` (default true; shows current value as percentage)
- `fillColor: string` (default "#1f4e79")

**bindableProps:** `value`

**Behavior:** clamp value to [min, max], render fill proportional. Bad-quality renders the bar in `--bad-color` with a dashed border.

#### 3. `Slider`

Write-back numeric input as a horizontal slider.

**Props:**
- `min: number` (default 0)
- `max: number` (default 100)
- `step: number` (default 1)
- `commitMode: "release" | "live"` (default "release" — only write on mouseup; "live" writes on every drag step with rate-limit)
- `units: string` (default "")

**bindableProps:** `value`

**Behavior:** standard `<input type="range">`. On commit, calls `context.onWriteTag(boundPath, { type: "real", value })`. Display shows current value to the right of the slider. While dragging, the display tracks the local state; the bound value re-syncs after commit. Bad-quality renders the slider in disabled-grey.

#### 4. `Dropdown`

Write-back selection input.

**Props:**
- `options: Array<{ value: string | number | boolean; label: string }>` (configured per-instance; designer property panel renders an array editor — same pattern as Trend's `tagPaths`)
- `placeholder: string` (default "Select…")

**bindableProps:** `value`

**Behavior:** standard `<select>`. On change, write the selected option's `value` to the bound tag. Selected option is determined by matching the bound value against `options[].value`. Bad-quality renders the select in disabled-grey.

#### 5. `ToggleSwitch`

Write-back boolean input as an iOS-style switch.

**Props:**
- `onLabel: string` (default "On")
- `offLabel: string` (default "Off")
- `onColor: string` (default "#16a34a")
- `offColor: string` (default "#9aa5b1")

**bindableProps:** `value`

**Behavior:** clickable pill. Writes `{ type: "bool", value: true|false }` to the bound tag. Bad-quality renders the switch in disabled-grey with the last-known value still visible (faded).

#### 6. `Button`

Click-to-write button.

**Props:**
- `label: string` (default "Press")
- `writeValue: TagValue` (configured constant; what gets written on click — bool, int, real, or string)
- `style: "primary" | "secondary" | "danger"` (default "primary")
- `confirmPrompt: string | undefined` (if set, shows a `window.confirm()` dialog before writing)

**bindableProps:** `target` (the tag path to write to — string-typed binding)

**Behavior:** on click, optionally confirm, then call `context.onWriteTag(target, writeValue)`. Disabled in designer mode (no live writes). Bad-quality on the bound `target` (if any) renders the button in disabled-grey.

#### 7. `MultiState`

Display one of N labels based on the bound tag value.

**Props:**
- `states: Array<{ value: string | number | boolean; label: string; color: string; bgColor: string }>` (configured per-instance; designer property panel renders an array editor)
- `defaultLabel: string` (default "—" — shown when bound value matches no state)
- `defaultColor: string` (default "#697586")

**bindableProps:** `value`

**Behavior:** find the state whose `value` matches the bound value (strict equality for primitives). Render `<div>` with that state's `label` + `color` + `bgColor`. If no match, render `defaultLabel`. Bad-quality renders the chip outlined in dashed grey.

#### 8. `AlarmBanner`

Top-of-screen alarm count strip showing active alarms grouped by priority band.

**Props:**
- `projectId: string` (default "" — uses runtime project id when blank)
- `showZero: boolean` (default false — hides bands with 0 active alarms unless true)
- `clickAction: "none" | "scroll-to-alarm-table"` (default "none")

**bindableProps:** none (uses `context.onSubscribeAlarms` like AlarmTable does)

**Behavior:** subscribes to `alarm.event` for the project, maintains a count of currently-active alarms (state ∈ {"active", "acked"}) per priority band:
- Critical (priority 1-2)
- Warning (priority 3)
- Info (priority 4-5)

Renders three pills with band name + count. Click is a no-op for v1 (the `clickAction` is a forward-compat hook).

**bindableProps:** none — alarm subscription is implicit, not bound to a tag.

### Shared concerns

- **`bindableProps`** must be set on every component definition. For value-displaying components, it's `["value"]` (or similar); for inputs it's `["value"]` (the bound write target); for Button it's `["target"]`. Components without a bound tag (AlarmBanner) use `[]`.
- **Bad-quality visuals**: at minimum, render the component in a faded/dashed-outline state when the bound `BoundValue.quality === "bad"`. Match Trend/AlarmTable's existing patterns.
- **Designer mode** (`context.mode === "designer"`): inputs (Slider, Dropdown, ToggleSwitch, Button) must NOT call `onWriteTag`; they're authored, not exercised. Wrap the click/change handlers with the existing mode check.
- **`propsSchema`** for each component drives the designer's property panel. Use the existing types: `string`, `number`, `boolean`, `color`, `select` (with `options`), `stringList`. For `options`/`states` (arrays of objects), introduce a new `objectList` schema type if necessary, or stringify and split on commas as a v1 pragmatic shortcut.
- **Default props** must produce a usable preview in the designer when the component is dropped fresh on a view (no binding configured yet).

### Test requirements

For each component, in `packages/component-library/src/components/__tests__/components.test.tsx` (or a per-component file if the component justifies its own):

- **Render with default props** without crashing.
- **Render with a bound `value`** and verify the visual (e.g. Gauge needle position, ProgressBar fill width, MultiState selected label).
- **Bad-quality binding** renders the bad-state visual (dashed outline / faded color).
- **Inputs** (Slider, Dropdown, ToggleSwitch, Button) call `context.onWriteTag` on user interaction in runtime mode and do NOT call it in designer mode.
- **AlarmBanner** subscribes via `context.onSubscribeAlarms` on mount; count updates correctly when synthetic events arrive.

The Trend/AlarmTable test pattern (`MockEditor`-style stubs, synthetic events) is the template — don't fight jsdom on real DOM measurements.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/component-library typecheck && test` green.
- [ ] All 8 components registered in `componentRegistry`.
- [ ] All 8 components exported from `packages/component-library/src/index.ts`.
- [ ] `pnpm --filter @openwebhmi/component-library build` produces a clean bundle.
- [ ] `pnpm -r typecheck && test` green workspace-wide.
- [ ] `cargo test --workspace --all-features --locked` stays green (no Rust changes expected, but the gateway smoke shouldn't regress).
- [ ] Manual smoke steps appended to `apps/designer/README.md` cover adding each new component to the `home` view and verifying it renders.
- [ ] Component-library JS bundle stays under 1.5 MB uncompressed (reasonable for 16 components).

### Out of scope (v1.1+ or future component-batch tasks)

- **Charts** (Pie, Bar, Line, Area beyond Trend) — separate task.
- **DataGrid / TabularView** — needs sorting/pagination/filtering UX; separate task.
- **Tabs / Accordion / Modal** — multi-page layout containers; separate task.
- **Theme editor UI** — Phase 2 stretch deferral; separate task.
- **Drag-to-resize / snap-to-grid** in the designer — Phase 2 stretch deferral.
- **Animation / transitions** beyond CSS hover states — separate task.

### Risks / gotchas

- **`objectList` schema type doesn't exist yet.** PropertyPanel today supports `string`, `number`, `boolean`, `color`, `select`, `stringList`. Components that need an array-of-objects config (Dropdown's `options`, MultiState's `states`) need either a new schema type with an array editor OR a v1 pragmatic shortcut: store as a JSON string and parse on render. Pick one and document. The pragmatic shortcut is faster but uglier; the array editor is the right v1.0 surface.
- **Slider commit modes**: `live` mode rate-limits writes to ~30/s to avoid flooding the gateway; use `requestAnimationFrame` or a debounce. `release` mode only writes on mouseup — easier and the safe v1 default.
- **AlarmBanner project_id resolution**: blank `projectId` should fall back to the runtime context's project id (same pattern AlarmTable uses).
- **Button's `writeValue` is a TagValue config**: the property panel needs to render a type+value picker. v1 pragmatic: render four inputs (bool/int/real/string) and pick whichever's set. v1.1 polish: a proper TagValue picker.
- **Don't break the existing 8 components.** Run the existing component tests after each new addition to catch shared-helper regressions.
- **Tag path for Button's `target` is bound, not configured.** A Button needs to know which tag to write — so `target` is a bindable string-typed prop. The runtime evaluates the binding at click-time, then calls `onWriteTag(targetValue, writeValue)`. v1.1 polish: consider whether `target` should be configurable as a literal string instead of bound, since most uses don't change at runtime.

## Codex log

*(codex — append working notes here)*

### 2026-05-01 17:14 MDT codex

Status -> in-progress. Starting component-library batch 2 first. Plan: mirror existing NumericInput/Trend/AlarmTable patterns, use the existing schema surface unless an object-list editor is already present, then run component package checks and workspace smoke.

### 2026-05-01 17:27 MDT codex

Status -> submitted. Added all 8 requested components: Gauge, ProgressBar, Slider, Dropdown, ToggleSwitch, Button, MultiState, and AlarmBanner. Registered and exported every component, added `objectList` schema support in the designer property panel for Dropdown options / MultiState states / Button writeValue, expanded component tests to 50 total, and appended manual smoke steps for each component.

Verification: `pnpm --filter @openwebhmi/component-library typecheck`, `pnpm --filter @openwebhmi/component-library test`, `pnpm --filter @openwebhmi/component-library build`, `pnpm -r typecheck`, `pnpm -r test`, and `cargo test --workspace --all-features --locked` all passed. Component-library emitted JS total is 67,720 bytes, under the 1.5 MB gate.

## Claude review

### 2026-05-01  claude — review pass 1

Eight components shipped, library doubled from 8 → 16. Bundle 67,720 bytes (well under the 1.5 MB gate). 50 tests green, +15 new for the batch. **Caught one real bug during review and applied a small fix during merge** (see Findings).

Strong points:
- ✅ **Gauge SVG arc math** is clean (`polar` + `arcPath` helpers, 270° SWEEP starting at -225°). Layered band rendering: green base → yellow warn overlays → red alarm overlays via `strokeLinecap="round"` so the segments visually compose.
- ✅ **AlarmBanner** subscribes via `context.onSubscribeAlarms` only in runtime mode, falls back to `context.projectId` when `projectId` prop is blank, counts active+acked alarms by priority band (≤2 critical / =3 warning / ≥4 info), respects `showZero` for empty bands, click-to-scroll uses `[aria-label='Alarms']` selector that matches AlarmTable's section.
- ✅ **Slider live-mode rate limit**: 33ms throttle via `lastLiveWrite` ref + Date.now() guard at `Slider.tsx:86-90`. Release mode uses `onMouseUp`/`onTouchEnd` (covers desktop + touch).
- ✅ **Button** correctly handles all three disable cases: no target, bad-quality bound target, designer mode. `disabled` flag drives the visual; `onClick` double-checks `context.mode !== "runtime" || disabled` before writing. `confirmPrompt` via `window.confirm()` works as briefed.
- ✅ **`objectList` schema variant added** at `types.ts:12` and `PropertyPanel.tsx:203` — the new schema type the brief flagged as needing to be invented. Powers Dropdown's `options` and MultiState's `states` arrays.
- ✅ **Designer-mode write suppression** consistent across all four inputs (Slider, Dropdown, ToggleSwitch, Button). Each guards `context.mode !== "runtime"` before calling `onWriteTag`.
- ✅ **Bad-quality visuals** consistent — `badQualityStyle(bound)` from `shared.ts` applied across the new components; matches existing Trend/AlarmTable patterns.
- ✅ **Codex correctly held the line** on CODEX-Z: didn't try to two-shot the ADS rework alongside AB. Discipline note appreciated.

Findings:

- 🟠 **Bug — write-back hardcoded to literal `"value"` path** in Slider, Dropdown, ToggleSwitch. `Slider.tsx:62` was `context.onWriteTag("value", ...)`; same in Dropdown.tsx:63 and ToggleSwitch.tsx:42. None of the three could write to a real PLC tag — they'd all hit the gateway's memory-tag fallback (CODEX-V's `split_once('/')` → `None` → publish path) regardless of binding configuration. **Fix applied during review** mirroring `NumericInput`'s existing pattern: added a `tagPath?: string` prop with default `""` and `props.tagPath || "value"` write expression. Tests already assert against the default fallback so they stayed green; verified 50/50 still passing post-fix. Button is the only correctly-shaped one of the four — its `target` prop is bindable from the start.
- 🟡 **Read/write asymmetry is the deeper issue.** The component model handles the read direction via the binding system (`bindings.value` is a `BoundValue` resolved from the bound tag path) but exposes only the *value* to components, not the *path*. Write-back therefore needs a separate, manually-configured `tagPath` string. Designers will commonly forget to set it; result: writes go to a memory tag silently. v1.1 architectural fix: extend the binding system or runtime context to expose the bound path so write-back can use it automatically. Tracked.
- 🟡 **`Button.writeValue` typed as `TagValue | TagValue[]`** to accommodate the `objectList` editor (which always stores values as arrays). `normalizeWriteValue` picks the first element. Works but is awkward — the brief said singular `TagValue`. v1.1: a proper TagValue picker with type+value selectors, no array.
- 🟡 **`AlarmBanner.mergeEvent` doesn't bound the events array** at `AlarmBanner.tsx:89-91`. Cleared alarms remain in the array (`countBands` correctly skips them so the displayed count is right). Memory grows unbounded over a long-running session with high alarm churn. AlarmTable bounds at 500 by default; AlarmBanner should adopt the same cap. v1.1.
- 🟡 **Gauge color bands don't dim on bad quality.** Only the needle goes dashed-grey; the background arc and warn/alarm bands stay full color. Inconsistent with how other components handle bad quality. Cosmetic; v1.1.
- 🟢 **Test coverage** is comprehensive: every component has a "renders default props", "writes on user action in runtime", "doesn't write in designer", and "renders bad quality" combination as applicable. AlarmBanner has a synthetic-events branch like AlarmTable does.
- 🟢 **Component bundle 67.7kB uncompressed** for 16 components — about 4.2kB per component. Plenty of headroom for the v1 25+ target.

Acceptance criteria — all eight boxes verified after the Claude-applied write-target fix.

## Verdict

**Merged with a Claude-applied fix.** Applied during review: added `tagPath` props to Slider/Dropdown/ToggleSwitch matching NumericInput's existing pattern so write-back actually targets a configurable tag path instead of the literal `"value"` string. The fix is mechanical (3 prop declarations + 3 schema entries + 3 fallback expressions); tests stayed green because they exercised the default fallback path.

Library now at **16 components** (up from 8). v1 target is 25+. Five v1.1 polish items added (write-target asymmetry, Button writeValue array shape, AlarmBanner unbounded events, Gauge bad-quality bands, plus the architectural extend-the-binding-system follow-up).

Codex's discipline note worth flagging: explicitly held off on Z's rework rather than landing another stub. Same posture from CODEX-AA. That's the right pattern.
