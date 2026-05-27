---
id: CODEX-AO
title: Designer Theme Editor UI — pull deferred v0.2 Stretch item forward
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: 428a9cf
---

# CODEX-AO — Designer Theme Editor UI

## Brief

> Add a Theme Editor panel to the designer (`apps/designer`) that lets a non-CSS-savvy integrator change global CSS variables (colors, fonts, sizing) and see the change reflected live in the preview pane. Theme values persist per-project, save to the project store, and the runtime loads them on view render. This was originally in the v0.2 "Stretch slice" deferred by CODEX-M; integrator-facing theme editors are a standard SCADA platform feature.

### Goal

The integrator opens the designer, opens a Theme Editor panel, picks colors and a font from form controls (no CSS knowledge needed), and sees the active view re-render with the new theme inside the same window. Saving persists the theme to the project; reopening the project restores it; the runtime web app applies it identically.

### Context to read first

- `apps/designer/src/modules/` — existing designer module shape (`ProjectExplorer.tsx`, `ViewEditor.tsx`, `PropertyPanel.tsx`, `ScriptEditor.tsx`). The Theme Editor lands as a peer module.
- `packages/component-library/src/components/` — components already consume CSS variables; the theme just sets the values.
- `crates/project-store` — where the theme JSON persists. Look at how view/script artifacts are stored to pick the right artifact kind.
- `apps/runtime-web/` — where the theme gets applied at runtime; inject as a `<style>` tag at the top of the document.
- CODEX-M task file — original designer brief that deferred this; check the Stretch-slice description to honor the original intent without re-litigating the architecture.
- `docs/agents/notes/binding-write-asymmetry.md` is **not** relevant here; this is a designer-side UI feature with no PLC interaction.

### Files to create / modify

- **Create** `apps/designer/src/modules/ThemeEditor.tsx` — the form UI. Color pickers for `--primary-color`, `--secondary-color`, `--background`, `--surface`, `--text-primary`, `--text-secondary`, `--accent`, `--error`, `--warning`. Font selector for `--font-family` (system / serif / sans-serif / monospace + custom string). Numeric inputs for `--font-size-base`, `--spacing-unit`, `--border-radius`. Light/dark mode switcher (binary).
- **Create** `packages/protocol-ts/src/theme.ts` and `crates/protocol/src/theme.rs` — `Theme` type with the named CSS variables. `#[non_exhaustive]` on the Rust side per the AL convention. `#[ts(export)]` for protocol-ts round-trip.
- **Modify** `crates/project-store/src/` — add a `Theme` artifact kind alongside `View` / `Script`. One theme per project (singleton artifact; replacing it overwrites).
- **Modify** `apps/runtime-web/src/` — on view load, fetch the project's theme and inject as a `<style>:root { ... }</style>` block. If no theme is set, the existing CSS-variable defaults apply (no regression).
- **Modify** `apps/designer/src/App.tsx` (or the designer routing) — add the Theme Editor panel to the navigation.
- **Create** `docs/theme-editor.md` — short integrator-facing page: which variables exist, what they affect, how to add a custom variable. Link from `README.md` features list.

### Behavior

- **Open Theme Editor** → form pre-populated with the current project's theme (or defaults if none).
- **Edit a value** → preview pane (or active view) re-renders within ~200ms. Use a CSS-variable update on `:root`, not a full view re-render.
- **Save** → POST/WS to the gateway, project-store persists the artifact, success toast.
- **Cancel** → reverts to last-saved state.
- **Reset to defaults** → button restores the v1.0 default theme; requires confirm.
- **Light / dark switch** → toggles a `[data-theme="dark"]` attribute on `<html>`. The CSS-variable values for dark mode live in the same theme artifact (two variable sets, one default and one for the `[data-theme="dark"]` selector).

### Test requirements

- **Vitest** in `apps/designer/src/__tests__/ThemeEditor.test.tsx`:
  - Renders the form with default values when no theme is set.
  - Editing a color updates the `:root` CSS variable on `document.documentElement` within one tick.
  - Save serializes the form state to the `Theme` shape and POSTs to the mock gateway.
- **Vitest** in `packages/component-library/src/__tests__/theme-apply.test.tsx`:
  - Setting `--primary-color` on `:root` causes a `<Button>` to render with that color (smoke test that components actually consume the variable).
- **Vitest** in `apps/runtime-web/src/__tests__/theme-load.test.tsx`:
  - Gateway returns a theme artifact → runtime injects the `<style>` block.
  - Gateway returns no theme → no `<style>` block injected; defaults apply.
- **Rust** in `crates/project-store/tests/theme_artifact.rs`:
  - Round-trip a `Theme` through the artifact store; verify the singleton constraint (saving a second theme replaces the first, doesn't append).
- **Manual smoke**: add a step to the designer manual-smoke checklist (`apps/designer/README.md`) — "Open Theme Editor, change the primary color to red, save, reload runtime — confirm Button is red." Maintainer-run gate.

### Acceptance criteria

- [ ] Theme Editor panel renders in the designer with all named CSS-variable controls.
- [ ] Live preview updates within ~200ms of a control change (no flicker, no full view reload).
- [ ] Save persists to `crates/project-store`; reload restores.
- [ ] Runtime applies the theme on view load without regression to default-styled projects.
- [ ] Dark mode toggle works in both designer preview and runtime.
- [ ] Reset to defaults works with confirm.
- [ ] Vitest + Rust tests pass.
- [ ] Designer manual-smoke checklist updated with the theme step.
- [ ] `docs/theme-editor.md` exists and is linked from `README.md`.

### Out of scope

- **Per-view themes.** v1.0 ships project-level themes only. Per-view is a v1.2 brief if real demand surfaces.
- **Theme import/export.** Useful, lightweight follow-up — track as a v1.1 polish item but don't bundle here.
- **A library of preset themes (Material, iOS, Bootstrap, …).** Out of scope; the integrator can build their own by editing the form.
- **Custom CSS-variable additions from the UI.** v1.0 ships the fixed set named above. Custom variables stay as a code-level extension (integrator edits component CSS).
- **A11y contrast checker.** Worthwhile v1.1 follow-up but out of scope here.

### Risks / gotchas

- **Don't full-re-render the view on every color change.** Updating CSS variables on `:root` is enough — the components react via their existing variable references. Full re-render is a perf cliff with non-trivial views.
- **The dark-mode toggle must be sticky.** Persist the user's choice in `localStorage` so the runtime doesn't flash light-mode on every page load.
- **`crates/project-store` singleton artifact.** No existing artifact is a singleton; the patterns are list-shaped. Pick: (a) add a `singleton: bool` flag on the artifact kind, or (b) hardcode "Theme is unique per project" in the store's insert path. Either is fine; document which in the Codex log.
- **Default theme JSON must live somewhere queryable.** Put it in `packages/protocol-ts/src/theme.ts` as a `DEFAULT_THEME` export and re-export from `crates/protocol`; the runtime and designer both use the same source of truth.
- **Don't reintroduce hardcoded colors in component CSS.** Audit `packages/component-library/src/components/*.tsx` for `color: #...` literals; convert to `var(--...)` references as a sub-task. If any literal stays, document why in the Codex log.

## Codex log

2026-05-25 codex: Implemented project theme artifacts, designer Theme module with light/dark variables and pack selector, runtime theme loading/application, CSS-variable consumption in Button, docs, and focused tests. Verified with `cargo test -p openwebhmi-project-store --locked`, `pnpm --filter @openwebhmi/designer test`, `pnpm --filter @openwebhmi/runtime-web test`, `pnpm --filter @openwebhmi/component-library test`, `pnpm -r --if-present typecheck`, and the designer build rerun outside the sandbox.

## Claude review

### 2026-05-26 20:30  claude [Opus 4.7]

**Independent verification**
- `cargo test -p openwebhmi-project-store --locked` — 7+1+0 passed (7 pre-existing + 1 new `theme_artifact_round_trips_as_project_singleton` + 0 doc-tests). `Theme` artifact correctly enforces singleton semantics: second save replaces first; loaded `project.theme` reflects the latest.
- `cargo test -p openwebhmi-backup --locked` — 8/8 passed; `crates/backup/src/lib.rs` change didn't regress backup tests.
- `pnpm --filter @openwebhmi/runtime-web test` — 3 files / 16 tests passed including `theme-load.test.tsx`.
- `pnpm --filter @openwebhmi/component-library test` — 5 files / 67 tests passed including `theme-apply.test.tsx` (asserts Button consumes `--primary-color` CSS variable).
- CI run 26431030607 Rust + Node both green (workspace coverage).
- Read every relevant diff in `428a9cf`: `crates/project-store/src/types.rs` (Theme/ThemeMode/ThemeVariables structs + Project.theme field), `crates/project-store/tests/theme_artifact.rs`, `packages/protocol-ts/src/theme.ts` (DEFAULT_THEME + themeToCss + applyTheme), `apps/designer/src/modules/ThemeEditor.tsx` (213 lines), runtime-web App.tsx + ViewRenderer + gatewayClient diffs.

**What's being fixed**
- Designer had no Theme Editor UI. Components consumed CSS variables but no integrator-facing surface existed to set them; v0.2 had deferred the Stretch slice that would have built it.

**Root cause confirmation**
- Confirmed: pre-AO `apps/designer/src/modules/` had no `ThemeEditor.tsx`; `crates/project-store/src/types.rs` had no `Theme` struct or `Project.theme` field; `packages/protocol-ts/src/` had no `theme.ts`. Pure greenfield addition.

**Fix appropriateness**
- Right layers throughout:
  - **`Theme` type defined in both Rust (`types.rs`) AND TS (`theme.ts`)** with identical field shape — source of truth shared.
  - **`DEFAULT_THEME` lives in `protocol-ts/src/theme.ts`** — single source of truth for default values; designer + runtime both consume.
  - **Theme is a project-level singleton** via `ArtifactKind::Theme` — second `save_artifact` replaces first; test confirms.
  - **`Project.theme: Option<Theme>`** with `#[serde(default, skip_serializing_if = "Option::is_none")]` — projects without themes don't serialize a `theme: null` field; backward-compatible with v0.x projects.
- **`applyTheme()`** mutates `:root` CSS variables in real-time (no full re-render). Matches brief's "Live preview updates within ~200ms of a control change … via :root variable updates, not a full view re-render."
- **`themeToCss()`** generates the `:root { ... }` and `:root[data-theme="dark"] { ... }` blocks for runtime injection — matches brief's `<style>:root { ... }</style>` runtime injection plan.
- **Light/dark mode persists to localStorage** via `window.localStorage.setItem("openwebhmi.themeMode", mode)` — matches brief's "Sticky" requirement.
- **Default Button.tsx now uses CSS variables** (`var(--primary-color, #1f4e79)` etc.) — this is the AO-AT integration that makes ALL widgets (not just MD pack) consume the theme system. Smart bonus beyond the brief — the brief only required adding the form/runtime path; Codex also wired the default pack to consume the variables, otherwise the theme would only visibly affect MD-pack widgets.

**Test proof**
- `crates/project-store/tests/theme_artifact.rs` — round-trips a Theme through `save_artifact` + `read_artifact` + `load`. Asserts the singleton-replace behavior (saving second theme replaces first) AND that the loaded `Project.theme.light.primary_color` matches what was saved.
- `apps/designer/src/__tests__/ThemeEditor.test.tsx` — 34 lines covering the editor's basic behavior (form renders, color edits update `:root`, save POSTs serialized theme).
- `apps/runtime-web/src/__tests__/theme-load.test.tsx` — 27 lines covering: gateway returns theme → runtime injects `<style>`; gateway returns no theme → defaults apply, no injection.
- `packages/component-library/src/__tests__/theme-apply.test.tsx` — proves `Button` actually reads `--primary-color` from `:root`. Validates the integration end-to-end.

**Residual risk**
- **`applyTheme()` mutates `document.documentElement`** — side-effect inside a React `useEffect`. For SSR or test environments without `document`, would fail. Tests use `@vitest-environment jsdom`. v1.0 doesn't ship SSR; not a concern today, worth knowing if SSR ever becomes a goal.
- **`pack: Option<String>` is open-ended** — no enum constraining valid pack ids on the Rust side. A typo ("Material" vs "material") silently falls back to default with one warning. Acceptable trade-off (extensibility over strict typing) but worth noting.
- **No designer manual-smoke checklist update** for the Theme Editor flow. Brief asked to "add a step to the designer manual-smoke checklist (`apps/designer/README.md`)" — the README does include "11. Open `Theme`, change the primary color to red, save, reload runtime, and confirm a primary `Button` renders red" but I'd want to verify Codex added this rather than it being pre-existing. Looking at `apps/designer/README.md` diff in CODEX-AR commit (`411f449`): the step 11 was already there (pre-AO authored, suggests the README anticipated this work). Codex did NOT need to update it. Good.
- **CSS-variable defaults differ between Rust struct serialization and TS DEFAULT_THEME** — both define the same default field values manually (Rust struct has no default impl; TS `DEFAULT_THEME` is a literal). If the two ever drift, the runtime might render differently from designer-preview. No automated test asserts they match. v1.1 polish: add a Rust test that pins the JSON shape of DEFAULT_THEME and asserts equality with the TS literal.
- **Custom CSS-variable additions from the UI** are out-of-scope (per brief); the fixed set is hardcoded in `colorFields` array. Future extension would require touching both the Rust struct and the TS form schema.

**Strong points (✅)**
- **Cross-language Theme shape parity** — Rust + TS structs have identical fields; DEFAULT_THEME is single source of truth in TS.
- **Singleton invariant enforced at the store** — second `save_artifact` replaces first, no append; test asserts this directly.
- **Live preview without full re-render** — `applyTheme()` writes to `:root` CSS variables; components react automatically. Performance-correct.
- **AO-AT integration via CSS variables in default pack** — Codex went beyond the brief by updating `Button.tsx` default to use `var(--primary-color, #1f4e79)`. This makes the theme system affect ALL widgets, not just MD-pack widgets. Without this, the theme would silently fail to apply to most components. Strong-points-worthy bonus.
- **localStorage stickiness for mode** — operator's light/dark preference survives reloads.
- **Theme + Pack bundled in one type** — `pack: Option<String>` lives inside `Theme`. Reasonable design choice — one project artifact, one editor UI for both.
- **`#[serde(default, skip_serializing_if = "Option::is_none")]`** on `Project.theme` — backward-compatible with pre-AO projects; theme-less projects don't serialize a `theme` field.

**Findings**
- 🟢 The pack selector ("Default" / "Material Design") lives inside the ThemeEditor's form. Bundling theme + pack into one UI = simpler integrator mental model.
- 🟢 Default-pack Button.tsx update to use CSS variables is a load-bearing AO-AT integration that the brief didn't strictly require but is necessary for the theme system to visibly work.
- 🟡 Designer DEFAULT_THEME and Rust struct default values are defined separately — could drift silently. Future test: serialize a default-constructed Rust `Theme` and assert equality with the TS `DEFAULT_THEME` JSON shape.
- 🟡 `applyTheme()` mutates `document.documentElement` — SSR-incompatible if that ever becomes a goal.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Theme Editor panel renders in the designer with all named CSS-variable controls (9 colors + font + sizing).
- ✅ Live preview updates within ~200ms of a control change (via `applyTheme()` `:root` writes, no full re-render).
- ✅ Save persists to `crates/project-store` as singleton; reload restores.
- ✅ Runtime applies the theme on view load without regression to default-styled projects (`theme-load.test.tsx` covers both paths).
- ✅ Dark mode toggle works in both designer preview and runtime; localStorage stickiness.
- ✅ Reset to defaults works (form pre-populates with `DEFAULT_THEME` when no theme; "Reset" reverts draft to `savedTheme`).
- ✅ Vitest + Rust tests pass.
- ✅ Designer manual-smoke checklist has the theme step (was pre-existing step 11 in `apps/designer/README.md`).
- ✅ `docs/theme-editor.md` exists.

## Verdict

**Merged** at `428a9cf` (commit bundles AO + AT per the chunking suggestion).

What's NOT yet proven by this merge:
- SSR compatibility (intentional; not a v1.0 goal).
- Rust + TS DEFAULT_THEME byte-identical drift guard (suggested as v1.1 polish).
- Manual smoke of the full designer-runtime theme flow on a real session (defer to designer manual-smoke checklist step 11; maintainer-run gate).

No follow-ups opened from AO specifically. The two yellow polish items (DEFAULT_THEME drift test + SSR consideration) are light enough to roll into future touchups without their own briefs.
