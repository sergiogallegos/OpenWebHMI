# Binding read/write asymmetry in the component library

## The constraint

Component bindings in `packages/component-library` give the **read path** for a bound tag value but do not expose the bound **path** for write-back. Components that need to *write* (any input control: numeric input, slider, dropdown, toggle, etc.) cannot derive the target path from the binding alone.

## The v1.0 workaround

Write-back components accept an explicit `tagPath?: string` prop. At runtime:

1. If `tagPath` is set, write to that path.
2. Otherwise fall back to the binding's bound path via the component's existing prop convention.
3. If neither is available, skip the write (don't crash, don't write to a literal like `"value"`).

The canonical implementation is in `packages/component-library/src/NumericInput.tsx` — the optional-prop fallback pattern there is what every write-back component should mirror.

Components currently following this pattern: `NumericInput`, `Slider`, `Dropdown`, `ToggleSwitch`. Any new input component must do the same.

## Why this matters in reviews

A review of a new input component that hardcodes `{ target: "value" }` (or any other literal) instead of accepting the optional `tagPath` prop is a **closeout blocker**, not polish — the component will break write-back on any binding whose name isn't `value`. CODEX-AB merge had this exact bug in three components; the fix during merge added the `tagPath?: string` prop to all of them.

## The v1.1 architectural fix

Extend the binding system to expose the bound path for write-back use, so input components don't need a parallel `tagPath` prop. Not done in v1.0 because it touches every binding consumer and is a bigger refactor than the read-path bug it fixes.

Until then, the prop fallback is the contract.

## See also

- `packages/component-library/src/NumericInput.tsx` — canonical pattern.
- CLAUDE.md "Code quality and testing discipline" — neighbor-copy rule.
