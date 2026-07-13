---
id: CODEX-CD
title: Component theming + accessibility — CSS-variable adoption, configurable labels, keyboard/switch semantics
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CD — Component theming + accessibility

## Brief

> Bring the default component pack up to the theme contract and basic accessibility bar. Today nearly every core component hardcodes a light hex palette instead of the `themeToCss` CSS variables, so project themes and dark mode don't restyle them; interactive controls carry generic duplicated `aria-label`s; `ToggleSwitch` lacks switch semantics and `AlarmTable` misuses table roles; and the `Slider` only writes on pointer release, so keyboard/AT operators can move the thumb without ever writing the tag. Fix all four across the default pack (and the material pack), each with a test. Medium priority — no new components.

### Goal

- The default pack consumes the theme CSS variables (`--surface`, `--text-primary`, `--primary-color`, …) with hex fallbacks, so a project theme change / dark mode restyles it — matching Button and the material pack, which already do.
- Interactive controls derive their accessible name from a configurable prop so ten pumps on a screen don't all read as "Toggle switch".
- `ToggleSwitch` exposes `role="switch"` + `aria-checked`; `AlarmTable` uses valid row/cell roles.
- `Slider` commits on keyboard interaction (arrow keys) and blur, not only pointer release.

### Context to read first

- `packages/protocol-ts/src/theme.ts` (lines 81-97, `toCssVariables`) — the authoritative CSS-variable contract themes emit: `--primary-color`, `--secondary-color`, `--background`, `--surface`, `--text-primary`, `--text-secondary`, `--accent`, `--error`, `--warning`, `--font-family`, `--font-size-base`, `--spacing-unit`, `--border-radius`. Runtime injects these via `themeToCss` (`apps/runtime-web/src/App.tsx` line 173). Components must read **these exact names** with hex fallbacks.
- `packages/component-library/src/components/Button.tsx` (`buttonStyle`, lines 79-88) — the **reference pattern already done right**: `background: "var(--primary-color, #1f4e79)"`, `color: "var(--text-primary, #1f2933)"`, `borderColor: "var(--error, #b91c1c)"`. Every other component should look like this.
- `packages/component-library/src/components/Slider.tsx` — hardcoded palette in `styles` (lines 113-135: `background: "#ffffff"`, `color: "#1f2933"`, `border: "1px solid #cbd2d9"`); generic `aria-label="Slider"` (line 73); commits **only** on `onMouseUp`/`onTouchEnd` (lines 99-100) — arrow-key changes fire `onChange` (line 88) and update the draft but never `commit`; `props.tagPath || "value"` write fallback (line 66).
- `packages/component-library/src/components/NumericInput.tsx` — hardcoded `border: "1px solid #9aa5b1"`, `color: "#1f2933"` (lines 85-88); generic `aria-label="Numeric input"` (line 66); `props.tagPath || "value"` (line 59).
- `packages/component-library/src/components/ToggleSwitch.tsx` — the `<button>` (lines 39-47) is missing `role="switch"` + `aria-checked`; generic `aria-label="Toggle switch"` (line 40); hardcoded `#ffffff` knob/label styles (lines 63-91); `props.tagPath || "value"` (line 45).
- `packages/component-library/src/components/Dropdown.tsx` — hardcoded `styles.select` palette (lines 98-109); generic `aria-label="Dropdown"` (line 57); `props.tagPath || "value"` (line 66).
- `packages/component-library/src/components/AlarmTable.tsx` — hardcoded `styles` palette (`shell` lines 348-359: `background: "#ffffff"`, `color: "#1f2933"`); the header uses `role="row"` on a `<div>` (line 195) and each row is a `<button role="row">` (lines 202-213) whose `<span>` children are **not** `role="cell"`/`role="gridcell"` — invalid ARIA table structure.
- `packages/component-library/src/packs/material/index.tsx` — duplicates the same components: `materialButton` (30), `materialToggleSwitch` (69, `aria-label` line 78), `materialNumericInput` (107, `aria-label` line 131), `materialSlider` (156, `aria-label` lines 172/174, `onMouseUp`/`onTouchEnd` lines 192-193), `materialDropdown` (226, `aria-label` line 237). Fixes must reach here too.
- [`docs/agents/notes/binding-write-asymmetry.md`](../notes/binding-write-asymmetry.md) — the `tagPath?: string` write pattern is **correctly applied** across the writable components. The shared `|| "value"` fallback is the related nit below; the note's rule is "if neither `tagPath` nor a bound path is available, skip the write — don't write to a literal like `value`."
- CODEX-AO (`docs/agents/tasks/CODEX-AO-theme-editor-ui.md`) — the theme editor/contract this task consumes.

### Files to create / modify

**(a) CSS-variable adoption — the default pack:**
- Replace hardcoded surface/text/border/accent hex values with `var(--token, #fallback)` across `Slider.tsx`, `NumericInput.tsx`, `ToggleSwitch.tsx`, `Dropdown.tsx`, `AlarmTable.tsx`, and any other default-pack component still hardcoding the palette (grep the `src/components` dir for `#ffffff`, `#1f2933`, `#9aa5b1`, `#cbd2d9`, `#d9e2ec` etc. and map each to the closest token). Mapping guide: surfaces → `var(--surface, …)`, primary text → `var(--text-primary, …)`, secondary/meta text → `var(--text-secondary, …)`, borders → `var(--secondary-color, …)` or a neutral, primary actions → `var(--primary-color, …)`, error/alarm accents → `var(--error, …)`, warnings → `var(--warning, …)`. Keep every existing hex as the fallback so unthemed usage is pixel-identical.
- Preserve the shared helpers in `components/shared.ts` (`badQualityStyle`, `designerStyle`, `baseFont`) — if `baseFont` should map to `var(--font-family, …)`, do it there once rather than per component (state the decision in the log).

**(b) Configurable accessible labels:**
- Add a configurable label source (a `label?`/`name?` prop, or reuse an existing descriptive prop) to `NumericInput`, `Slider`, `ToggleSwitch`, `Button`, `Dropdown`; derive `aria-label` from it, falling back to the current generic string when unset. Add the prop to `propsSchema`/`defaultProps` so it's editable in the designer. Don't break existing views: unset ⇒ same generic label as today.

**(c) ARIA role corrections:**
- `ToggleSwitch`: add `role="switch"` and `aria-checked={value}` to the control (lines 39-47). Keep it a `<button>`; `role="switch"` on a button is valid.
- `AlarmTable`: give the table a valid structure. Either make the container `role="table"`/`role="grid"` with `role="row"` rows and `role="cell"`/`role="gridcell"` children (lines 195-213+), or drop the ARIA table roles and use semantic markup. A `<button role="row">` with non-cell `<span>` children is invalid — fix the mismatch. Preserve the click-to-select-row behavior and keyboard operability.

**(d) Slider keyboard commit:**
- `Slider` must also commit on `onKeyUp` and `onBlur` (in addition to `onMouseUp`/`onTouchEnd`, lines 99-100) so arrow-key/Home/End/PageUp changes and focus-loss write the tag. Respect `commitMode` (`"release"` vs `"live"`) — keyboard commit fits the release contract. Don't double-write when a pointer release and a blur both fire; guard against redundant identical writes if needed.

**(e) Related nit — the `|| "value"` write fallback (lower priority):**
- `NumericInput` (line 59), `Slider` (line 66), `Dropdown` (line 66), `ToggleSwitch` (line 45), and the material equivalents write to the literal path `"value"` when `tagPath` is unset. Per the binding-write-asymmetry note, an unresolved write target should **no-op** (skip the write) rather than emit `tag.write { path: "value" }`. Change the fallback so an unset/empty `tagPath` skips the write (and, in designer mode, may surface a config warning). Keep it minimal — this is a correctness nit, not a redesign.

**(f) Material pack:**
- Apply (a)-(e) to `packs/material/index.tsx` too, or (preferred) refactor the material entries to wrap/reuse the base component `Render` so theming/label/role/commit fixes live once. If wrapping is impractical for a given component, patch both and flag the duplication. State the decision in the log.

### Behavior

- Injecting a theme (or toggling `data-theme` dark) restyles the default-pack components — surfaces, text, borders, accents follow the CSS variables; with no theme present the fallbacks render identically to today.
- A `NumericInput`/`Slider`/`ToggleSwitch`/`Button`/`Dropdown` given a configurable label exposes it as the accessible name; unset falls back to the generic string.
- `ToggleSwitch` exposes `role="switch"` with `aria-checked` tracking the value. `AlarmTable` presents a valid role structure with operable, correctly-roled rows.
- Operating the `Slider` by keyboard (arrow keys, then blur or keyup) writes the tag; pointer drag still commits on release.
- An input control with no `tagPath` and no bound path no longer writes `tag.write { path: "value" }` — it skips the write.

### Test requirements

- **Vitest + `@testing-library/react`, added to the closest existing component-library specs** (there is a `material-pack.test.tsx` and per-component tests — extend, don't fragment). Every test must **fail without the fix** (run once on `main`).
- **(a) Theming:** render a component inside a container that sets the CSS variables (e.g. a wrapper with `style={{ "--surface": "#101418", "--text-primary": "#e5e7eb" }}`), assert the component's computed/inline style consumes the variable (assert the style string contains `var(--surface`, or resolve via `getComputedStyle` on a styled node). A pre-fix run (hardcoded hex) fails this.
- **(b) Labels:** render with a configurable label prop; assert `getByRole(...).getAttribute("aria-label")` (or accessible name) is the configured value; render without it and assert the generic fallback.
- **(c) Roles:** assert `ToggleSwitch` is `getByRole("switch")` with `aria-checked` reflecting the value; assert `AlarmTable` exposes valid `role="table"`/`grid` + rows + cells (query by role) with no `button[role="row"]`-without-cells mismatch.
- **(d) Keyboard commit:** render `Slider` in runtime mode with a spy `onWriteTag`; fire an arrow-key change then `keyUp`/`blur`; assert `onWriteTag` was called with the new value. A pre-fix run (commit only on mouseup) fails this.
- **(e) No-op fallback:** render a writable control with no `tagPath` and no bound path; trigger a write interaction; assert `onWriteTag` was **not** called with path `"value"` (asserted for the base and material variants).
- Cover **both** the default pack and the material pack (or, if refactored to share, one path plus a smoke test that the material entry inherits the behavior).
- `pnpm -r typecheck`, `pnpm -r test`, and the component-library `build` clean. No `any` abuse; no removed listeners left dangling.

### Acceptance criteria

- [ ] Default-pack components consume the theme CSS variables (`--surface`/`--text-primary`/`--primary-color`/… per `toCssVariables`) with hex fallbacks; unthemed rendering is unchanged.
- [ ] Configurable accessible label on NumericInput/Slider/ToggleSwitch/Button/Dropdown, generic fallback preserved, prop added to schema.
- [ ] `ToggleSwitch` has `role="switch"` + `aria-checked`; `AlarmTable` has a valid role structure (no `button[role="row"]` with non-cell children).
- [ ] `Slider` commits on `onKeyUp`/`onBlur` as well as pointer release; keyboard operation writes the tag; no redundant double-writes.
- [ ] `|| "value"` fallback replaced with a skip-the-write (no-op) when no target is resolvable; optional designer-mode config warning.
- [ ] Fixes applied to the material pack (or the pack refactored to reuse base renders — flagged either way).
- [ ] Each fix has a test that fails without the fix; theming/label/role/keyboard/no-op all covered for base and material; typecheck + test + build clean.
- [ ] Codex log states the token-mapping decisions (which hex → which variable), the `baseFont`→`--font-family` decision, and the material refactor-vs-duplicate choice.

### Out of scope

- **New theme tokens or extending `toCssVariables`.** Consume the existing 13 variables; don't invent new ones.
- **A full WCAG audit / focus-ring redesign / contrast tuning.** This task fixes the specific role, label, and keyboard-commit gaps named above, not every a11y concern in the pack.
- **Redesigning `AlarmTable` into a real data grid** or virtualization. Fix the ARIA role mismatch and keep behavior; a grid rewrite is separate.
- **Changing the `tagPath?: string` write convention** or the binding system. (e) only changes what happens when the target is *unresolvable* (skip vs write `"value"`).
- **Runtime render architecture** (CODEX-CB) and **designer editing integrity** (CODEX-CC). The NumericInput/Slider focus-guard lives in CODEX-CC; this task's Slider change is the keyboard-commit + theming, not the focus race — coordinate if both touch Slider (rebase order is a merge concern, flag it).
- **Gauge/Trend/Card deep restyle** beyond swapping their hardcoded palette to variables — apply the same variable-adoption mechanically; don't redesign their visuals.

### Risks / gotchas

- **Fallback identity.** Every `var(--token, #hex)` must keep the *current* hex as the fallback so unthemed rendering is byte-identical. A theming test proves variables are consumed; a visual-regression risk is that a wrong fallback shifts the default look — keep fallbacks exactly as the hex they replace.
- **Inline-style CSS variables in React.** `style={{ "--surface": "#101418" }}` needs a typed cast (`as React.CSSProperties`) because TS doesn't allow arbitrary custom properties by default; the *consumption* side (`background: "var(--surface, #fff)"`) is a plain string and fine. Handle the test-wrapper typing without `any`.
- **`role="switch"` semantics.** A switch needs `aria-checked` (not `aria-pressed`); keep it a `<button type="button">`. Screen readers announce state from `aria-checked`, so it must track `value`, including the bound value, not just local state.
- **AlarmTable keyboard operability.** If rows become `role="row"` on non-button elements, preserve click-and-keyboard selection (the current `<button>` gave that for free). Either keep buttons *inside* cells or add `tabIndex`/`onKeyDown` (Enter/Space) to the row. Don't regress operability while fixing roles.
- **Slider double-write.** Adding `onKeyUp`/`onBlur` commits alongside `onMouseUp` can fire two writes for one interaction (mouseup then blur). Guard against emitting an identical redundant write (compare against last-committed value) so the gateway isn't spammed. In `"live"` mode the throttle already exists (lines 91-97) — keyboard commit belongs to the release semantics.
- **The `|| "value"` change interacts with existing views.** Some demo views may rely on an implicit `value` path today. Grep the demo project/views before making the skip a hard no-op; if a real view depends on it, note it and prefer the no-op + designer warning so misconfiguration is visible rather than silently writing a literal.
- **Material pack duplication (again).** Same repeat-offender risk as CODEX-CC(c). Prefer wrapping the base render so theming/label/role/commit fixes can't diverge. Whatever the choice, flag it.
- **Honesty discipline.** The Codex log lists exactly which components were re-themed, the hex→token map, which got configurable labels, and whether the material pack was refactored to share or patched twice — no "done everywhere" hand-waves.

## Codex log

## Claude review

## Verdict
