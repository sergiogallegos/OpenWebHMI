---
id: CODEX-AT
title: Material Design widget pack — demo subset proving theme-pack architecture
owner: codex
phase: 4
status: submitted
created: 2026-05-25
last-update: 2026-05-25 codex [gpt-5]
---

# CODEX-AT — Material Design widget pack (demo subset)

## Brief

> Ship a Material-Design-styled variant of 6–8 representative widgets to prove the "two widget packs, switchable per project" architecture that mature SCADA platforms commonly ship (a legacy pack plus a modern pack). **Demo-scope:** not full 25-component parity. Goal is to validate the parallel-pack pattern, give marketing a "ships with two looks" demo, and surface the architectural cost so v1.2 can scope a full second pack honestly.

### Goal

A project can opt into a Material Design pack via a single configuration change (theme manifest field, project setting, or runtime flag — Codex picks the right knob). The 6–8 chosen widgets render with Material elevation, ripple effects, motion, and the MD3 color system instead of the default v1.0 styling. Existing default-styled projects are unaffected (no visual regression).

Picks for the demo subset (representative of the 25 v1.0 components):

- `Button` — interaction primitive; ripple is the canonical MD effect.
- `ToggleSwitch` — second interaction primitive; MD switch shape is distinctive.
- `NumericInput` — input field with MD floating-label or filled style.
- `Slider` — continuous input; MD slider has distinctive thumb + track.
- `Card` — surface primitive; MD elevation contracts live here.
- `Dropdown` — menu primitive; MD menu animation contracts live here.
- `Gauge` — visualization primitive; demonstrates that data-viz components fit the pattern.
- `Modal` — overlay primitive; MD elevation + scrim contracts.

### Context to read first

- `packages/component-library/src/components/` — current v1.0 components.
- `packages/component-library/src/registry.ts` — type-id → component lookup. The pack-switching mechanism extends or layers this.
- `packages/component-library/src/types.ts` — component prop shapes. MD variants share the same prop API (identical contract) — only the rendering changes.
- The theme infrastructure delivered by **CODEX-AO** (Theme Editor UI). This brief **depends on CODEX-AO** if AO ships first; the MD pack consumes the same CSS-variable surface AO defines. If AT lands first, it must define the shared CSS-variable contract and AO consumes it.
- `material-web` (`@material/web` on npm) — Google's official MD3 web components. Codex picks: (a) reuse `material-web` components inside the wrapper layer, (b) reimplement using Tailwind/CSS-Modules to match MD3 spec, (c) hand-roll a subset. Document the choice + reasoning in the Codex log.

### Files to create / modify

- **Create** `packages/component-library/src/packs/material/` — the MD pack:
  - `Button.tsx`, `ToggleSwitch.tsx`, `NumericInput.tsx`, `Slider.tsx`, `Card.tsx`, `Dropdown.tsx`, `Gauge.tsx`, `Modal.tsx` — MD-styled implementations of the same component prop API.
  - `tokens.css` — the MD3 design tokens as CSS variables (elevation levels, motion timing, type scale).
  - `index.ts` — exports the pack as a `{ [typeId]: Component }` map.
- **Modify** `packages/component-library/src/registry.ts` — extend to support pack selection. Pattern: `registry.get(typeId, packId)` falls back to the default pack if the MD pack doesn't have an entry for that type.
- **Modify** `crates/protocol/src/` (and TS mirror) — add a `pack: Option<String>` field to the project / theme schema. `None` means default pack. `Some("material")` means the MD pack.
- **Modify** `apps/designer/src/modules/ThemeEditor.tsx` (if CODEX-AO shipped) — add a pack selector ("Default" / "Material Design"). If AO hasn't shipped, add the pack selector as a temporary stand-alone control in the project-settings module.
- **Modify** `apps/runtime-web/src/` — on project load, read the pack id and bind the registry to that pack.
- **Create** `docs/widget-packs.md` — explains the two packs, what's in each, how to switch, what's covered in the MD pack vs falling back to default.
- **Create** `docs/agents/notes/widget-pack-architecture.md` — agent note: the contract for adding new packs, what prop API stability means, what intentionally falls back to default, how packs interact with the theme variables from CODEX-AO.

### Behavior

- **Default pack unchanged**. A v1.0 project with `pack: None` renders identically to today — no visual regression.
- **MD pack opt-in**. Setting `pack: Some("material")` on the project flips the registry binding. Widgets that exist in the MD pack render MD-style; widgets that don't fall back transparently to the default pack with a one-time console warning (`material pack: NumericInput rendered with default style; not in pack`).
- **Prop API parity is the load-bearing contract.** The MD `Button` takes the same props as the default `Button`. No new props specific to MD3 (no `elevation`, no `ripple-enabled`) — those go in `tokens.css` and apply to every MD-pack widget uniformly. The motivation: integrators switch packs without touching project files.
- **Demo project**: include a small example project (`examples/material-demo/`) that flips the pack and renders one view per widget category, so marketing has something to screenshot.

### Test requirements

- **Vitest** in `packages/component-library/src/__tests__/material-pack.test.tsx`:
  - Render each MD-pack widget with default props; assert no console errors, basic DOM shape (e.g. MD Button has a `.mdc-button` or equivalent class).
  - Pack-fallback: render a widget type the MD pack doesn't have with `packId: "material"`; assert it renders via the default pack and one warning fires.
  - Prop API parity: for each MD-pack widget, assert the props accepted match the default pack widget's props (use TypeScript types in the test).
- **Storybook stories** (or a `packages/component-library/src/__stories__/` equivalent) for each MD-pack widget; sets up the visual-regression surface for future changes. Out of scope to add Storybook to the repo if it isn't already; otherwise, screenshots in `docs/widget-packs.md` are the substitute.
- **Visual smoke**: `examples/material-demo/` renders cleanly in the runtime with `pack: Some("material")`; manual maintainer step.

### Acceptance criteria

- [ ] MD pack ships with the 8 components named above (Button, ToggleSwitch, NumericInput, Slider, Card, Dropdown, Gauge, Modal).
- [ ] Default pack and projects with `pack: None` render unchanged.
- [ ] Pack selection persists on the project; reload restores.
- [ ] Pack-fallback works with a one-time warning per type.
- [ ] `examples/material-demo/` project exists and renders.
- [ ] `docs/widget-packs.md` and `docs/agents/notes/widget-pack-architecture.md` exist.
- [ ] Vitest tests pass three consecutive runs.
- [ ] Prop API parity verified by TS types (compile-time check).

### Out of scope

- **Full 25-widget MD pack.** v1.2 brief if this demo lands cleanly and marketing wants the full pack. The cost of finishing the remaining 17 widgets should be visible after this brief lands.
- **MD2 / pre-MD3.** MD3 (Material You) is the target. MD2 is end-of-life.
- **Custom-pack authoring documentation.** Document the contract in the agent note; full third-party-pack authoring guide is post-1.x.
- **Per-widget pack overrides.** v1.0 supports project-level pack only. "This one Button is MD even though the project pack is default" is out of scope.
- **Migrating existing projects to MD pack automatically.** Opt-in only. No mass-migration helper.
- **Tauri-side native MD components.** Web/CSS only.

### Risks / gotchas

- **Pick the implementation strategy carefully.** Three options for the MD pack:
  1. **Wrap `@material/web` (Google's MD3 web components).** Fastest to ship but adds a substantial dep, and `@material/web` is Lit-based which doesn't compose perfectly with React. Document the React interop strategy.
  2. **Reimplement with Tailwind / CSS Modules** matching MD3 spec. Most control, no large deps, but a lot of CSS to write and maintain.
  3. **Hand-roll minimal MD styling** without claiming full MD3 spec compliance. Cheapest, but the "Material Design" claim then becomes "MD-inspired" — marketing-wise weaker.

   Codex picks one and documents in the Codex log. Strong default recommendation: **option 2** (Tailwind / CSS Modules per MD3 spec) for control and stability; reach for `@material/web` only if it accelerates substantially and the React interop is clean.

- **Don't reshape the default pack to support packs.** The registry abstraction can be additive. If supporting packs requires refactoring every default-pack widget, the scope has crept — stop and rework the registry-only diff.
- **Ripple / motion / elevation are MD3's distinguishing features.** A "Material" pack without them looks generic. Budget the implementation time for these or descope a widget that won't get them properly.
- **`docs/widget-packs.md` must set expectations honestly.** "8 of 25 widgets are MD-styled; the rest fall back to default" is correct positioning. "OpenWebHMI ships a full Material Design widget library" is overpromising.
- **Theme variables from CODEX-AO are the shared contract.** MD3 has its own color tokens (`--md-sys-color-primary`, etc.). The pack's `tokens.css` must either (a) consume AO's variables directly, or (b) define its own and document why AO's surface doesn't fit. If (b), there are now two color systems — that's a maintenance debt; recommend (a).
- **Designer preview is the demo surface.** Make sure the pack works in the designer's preview pane, not just the runtime. Otherwise the integrator can't see what they're shipping.

## Codex log

2026-05-25 codex: Implemented additive component pack registry with default fallback warning, Material demo pack for Button/ToggleSwitch/NumericInput/Slider/Card/Dropdown/Gauge/Modal, theme-level pack persistence, runtime pack binding, designer pack selector, example project, docs, and material pack tests. Chose hand-rolled React/CSS using the existing prop contracts and Theme Editor CSS variables instead of `@material/web` to avoid Lit/React interop and new dependency surface. Verified with `pnpm --filter @openwebhmi/component-library test`, `pnpm --filter @openwebhmi/runtime-web test`, `pnpm --filter @openwebhmi/designer test`, `pnpm -r --if-present typecheck`, and build verification.

## Claude review

## Verdict
