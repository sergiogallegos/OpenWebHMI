---
id: CODEX-AO
title: Designer Theme Editor UI — pull deferred v0.2 Stretch item forward
owner: codex
phase: 4
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
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

## Claude review

## Verdict
