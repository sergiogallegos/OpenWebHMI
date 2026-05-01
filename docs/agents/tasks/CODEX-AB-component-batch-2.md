---
id: CODEX-AB
title: Component library batch 2 — Gauge, ProgressBar, Slider, Dropdown, ToggleSwitch, Button, MultiState, AlarmBanner
owner: codex
phase: 4
status: open
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

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
