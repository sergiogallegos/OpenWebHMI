---
id: CODEX-K
title: packages/component-library — 6 essential components for Phase 2
owner: codex
phase: 2
status: merged
created: 2026-04-26
last-update: 2026-04-27 claude
---

# CODEX-K — `packages/component-library` v1

## Brief

### Goal

The 6 essential components that compose every Phase 2 demo HMI. Each component is a React function that:
1. Renders in the **runtime** (live, bound to tags).
2. Renders in the **designer** with adornments (selection outline, drag-handles when a canvas exists).
3. Declares a **propsSchema** that drives the designer's property panel automatically.
4. Declares which props are **bindable** (can read from tags).

This task does *not* include the visual canvas or drag/drop — those are Phase 2 stretch. Components must be authored to render correctly *now* and be canvas-compatible *later* without rewrites.

### Context to read first

- `docs/architecture.md` §4.12 (Component Library) — the contract.
- `docs/feature-matrix.md` §3 (HMI / visualization) — what users will recognize as "Ignition-equivalent".
- `crates/project-store/src/types.rs` (CODEX-J) — the `Component` struct that references your components by `kind`.
- `apps/runtime-web/src/App.tsx` — the runtime's current shape; CODEX-L wires the library in.

### Files to create

- `packages/component-library/package.json` — name `@openwebhmi/component-library`, depends on `react`, `@openwebhmi/protocol`. DevDeps: `typescript`, `vitest`, `@testing-library/react`, `jsdom`.
- `packages/component-library/tsconfig.json` — strict, JSX react-jsx.
- `packages/component-library/src/index.ts` — exports.
- `packages/component-library/src/types.ts` — `ComponentDefinition`, `PropSchema`, `BindableProp`, `RuntimeContext`, `DesignerContext`.
- `packages/component-library/src/registry.ts` — `componentRegistry: Record<string, ComponentDefinition>`.
- `packages/component-library/src/components/Label.tsx`
- `packages/component-library/src/components/ValueDisplay.tsx`
- `packages/component-library/src/components/NumericInput.tsx`
- `packages/component-library/src/components/Indicator.tsx`
- `packages/component-library/src/components/Image.tsx`
- `packages/component-library/src/components/Container.tsx`
- `packages/component-library/src/components/__tests__/*.test.tsx` — vitest + RTL.

Add `packages/component-library` to `pnpm-workspace.yaml`.

### Component contract

```ts
import type { TagValue, Quality } from "@openwebhmi/protocol";

export type ComponentDefinition<Props = unknown> = {
  kind: string;                      // "Label", "ValueDisplay", ...
  defaultProps: Props;
  propsSchema: PropSchema<Props>;    // JSON-Schema-ish, drives the property panel
  bindableProps: (keyof Props & string)[];  // which props accept tag bindings
  Render: React.FC<RenderProps<Props>>;
  ThumbnailIcon?: React.FC;          // small icon for the designer palette
};

export type RenderProps<Props> = {
  props: Props;
  bindings: Record<string, BoundValue | undefined>;
  context: RuntimeContext | DesignerContext;
};

export type BoundValue = {
  value: TagValue;
  quality: Quality;
  ts: number;
};

export type RuntimeContext = {
  mode: "runtime";
  onWriteTag: (path: string, value: TagValue) => void;
};

export type DesignerContext = {
  mode: "designer";
  isSelected: boolean;
  // Future canvas hooks land here without touching components.
};
```

### The 6 components

| Component | Purpose | Key props | Bindable props |
|---|---|---|---|
| **Label** | static text | `text`, `color`, `fontSize` | `text` |
| **ValueDisplay** | render a tag's current value with quality + timestamp | `format` (`"number" \| "string" \| "auto"`), `decimals`, `unit` | `value` |
| **NumericInput** | operator-editable number that writes back to a tag | `min`, `max`, `step`, `disabled` | `value` (read), `value` (write — paired) |
| **Indicator** | colored dot/badge driven by a tag — maps `boolean` to on/off colors, optionally maps a string to one of N colors | `onColor`, `offColor`, `mapping` (string → color) | `state` |
| **Image** | static or tag-driven image (URL prop, optionally bound) | `src`, `width`, `height`, `fit` (`"contain" \| "cover"`) | `src` |
| **Container** | layout container — flex or grid; renders `children` from the view tree | `direction` (`"row" \| "column"`), `gap`, `padding`, `background` | (none — Container is layout-only) |

Each component must:
- Render with sensible defaults when no binding is attached (designer preview without a gateway connection).
- Render a **bad-quality visual** (subtle red border / muted color / "?" indicator) when its bound value has `quality !== "good"`. Don't crash; don't hide the value.
- Be keyboard-accessible where it makes sense (NumericInput, Indicator). Add ARIA labels.
- Render within 16ms for typical prop sets.

### Tests

Vitest + React Testing Library. Cover, per component:

- Renders with default props.
- Renders with a bound `Quality::Good` value.
- Renders the bad-quality visual when bound to `Quality::Bad`.
- Renders correctly when the binding is missing entirely (no value at all).

Plus, for `NumericInput`: typing a value and pressing Enter calls `context.onWriteTag` with the new value.

### Storybook

**Out of scope for this task.** Storybook is a Phase 2 stretch; component-level rendering is verified by RTL tests for now.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/component-library build` exits 0; emits `dist/index.js` + types.
- [ ] `pnpm --filter @openwebhmi/component-library typecheck` exits 0.
- [ ] `pnpm --filter @openwebhmi/component-library test` runs vitest, all per-component tests above pass.
- [ ] `componentRegistry` exports all 6 components keyed by their `kind`.
- [ ] Each component has TSDoc on its exported `ComponentDefinition`.
- [ ] No DOM imports in the registry/types layer; only the components themselves touch the DOM.

### Out of scope

- Visual canvas / drag-drop adornments (Phase 2 stretch — components must be canvas-compatible but don't ship canvas behavior).
- Storybook stories (deferred).
- Theme support (Phase 2 stretch / Phase 4).
- The remaining 4 components (`Button, Rectangle, Line, ToggleSwitch`) — Phase 2 stretch.
- Server-side rendering / hydration.

### Risks / gotchas

- **Don't over-engineer the schema language.** `propsSchema` is JSON-Schema-ish but does *not* need to be full JSON Schema. Just enough to render: `{ type: "string" | "number" | "boolean" | "color" | "select", options?: string[], default?: unknown }`.
- **`Container.children` reads from the view tree, not from a `children` prop.** The runtime renderer (CODEX-L) walks the view tree; Container is a layout primitive, not a normal React `{children}` component.
- **Bad-quality visual must be visible without being noisy.** Subtle border + muted text. Don't blink. Operators turn off blinking alarms; we shouldn't accidentally re-invent that.
- **NumericInput write semantics**: write only on Enter / blur, *not* on every keystroke. Otherwise typing 1234 writes 1, then 12, then 123, then 1234.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 20:09  codex
Started CODEX-K. Read `docs/agents/README.md`, this task brief, `docs/architecture.md` section 4.12, `docs/feature-matrix.md` section 3, current `@openwebhmi/protocol` types, workspace config, and runtime app shape. `crates/project-store/src/types.rs` is not present in this checkout yet, so implementation follows the explicit component contract in this brief.

### 2026-04-26 20:16  codex
Submitted CODEX-K. Added `@openwebhmi/component-library` with shared component contract types, six React component definitions, registry exports, generated `dist/` output, and Vitest/React Testing Library coverage for defaults, good bindings, bad-quality visuals, missing bindings, and NumericInput Enter writes. Verification passed: `pnpm --filter @openwebhmi/component-library build`, `typecheck`, and `test`.

## Claude review

### 2026-04-27  claude — review pass 1

Spec-compliant. All 6 components present, registry exports them keyed by `kind`, `ComponentDefinition` contract matches the brief exactly. Each component declares `propsSchema`, `bindableProps`, `defaultProps`, and a single `Render` function — designer property panel can drive itself off these without per-component code in CODEX-M.

Strong points:
- ✅ **NumericInput writes on Enter or blur, not on every keystroke** (`NumericInput.tsx:73-79`). Per the brief's gotcha. Plus min/max clamping with `Number.isFinite` guard against NaN.
- ✅ **Bad-quality visual is consistent across components** via `badQualityStyle()` in `shared.ts`. Subtle red border, "?" indicator on Indicator. Doesn't blink. Per the brief.
- ✅ **`shared.ts` extracts the cross-component duplication** (badQualityStyle, designerStyle, tagValueToNumber, tagValueFromNumber). Keeps individual components clean.
- ✅ **Test coverage is thorough** — every component covers default props, bound-good, bound-bad, and missing-binding paths. NumericInput additionally asserts the Enter-then-write semantics with `userEvent.type("123.5{Enter}")`.
- ✅ **Container `direction` / `gap` / `padding` / `background` props** map directly to the layout primitives the runtime needs. Container reads `children` from React (passed by the renderer in CODEX-L), not from a prop.

Findings:
- 🟡 **`BindableProp<Props = any>`** uses `any` (`types.ts:15`). For library generics this is fine because it's existential, but `unknown` would be marginally safer. Cosmetic.
- 🟡 **Container's bad-quality test** (`components.test.tsx:311-316`) is named "renders without a bad-quality visual because it has no bindable props" but only asserts the *designer dashed* border. It doesn't *not* assert the bad-quality red border. Functional behavior is correct (Container has no `bindableProps`, so a bad binding can't reach it), but the test would benefit from an explicit "bad style is NOT applied" assertion.
- 🟡 **`dist/` was committed** alongside source. The repo's `.gitignore` excludes `dist/` so the next checkout will not have it, but the committed snapshot carries build output. One-time cleanup at the next merge.
- 🟢 The `propsSchema` field-type vocabulary (`string | number | boolean | color | select`) is exactly what the brief asked for — minimal, not full JSON Schema. Designer property panel UI in CODEX-M can render each from a single switch statement.

Acceptance criteria all met. No canvas, no drag/drop — held the Phase 2 Required line correctly.

## Verdict

**Merged** at `24c1ac7`. Cosmetic notes tracked here. Strong submission — the components + tests are designer-ready and runtime-ready, which is exactly the gate before CODEX-L can start.
