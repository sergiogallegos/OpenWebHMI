# packages/AGENTS.md

TypeScript library rules for `packages/{component-library,protocol-ts}`. Loaded automatically when work is under `packages/`. The root `AGENTS.md` applies in addition; the apps tree has its own `apps/AGENTS.md`.

## Per-package responsibilities

- **`packages/component-library`** — HMI components used by `apps/runtime-web` and authored in `apps/designer`. Each component has bindings (read path), optional write-back, vitest coverage, and a gallery entry. The 25-component v1 target is met (per CODEX-AC).
- **`packages/protocol-ts`** — TypeScript wire types paired with `crates/protocol`. The single source of truth for the WebSocket message shapes. Apps consume; they don't extend.

## Component contract

Every component in `component-library`:

1. Takes a `binding` prop for the read path (the tag whose value drives the visual).
2. If the component is an *input* (NumericInput, Slider, Dropdown, ToggleSwitch, etc.) it also takes a separate **`tagPath?: string`** prop for the write path. This is the v1 workaround for the binding system not exposing the bound path for write-back — see the project-specific gotchas section in `CLAUDE.md`.
3. Has a vitest unit test (`*.test.tsx`) covering read-path rendering, write-path side-effects where applicable, and quality propagation.
4. Has a gallery entry so the designer can list it.
5. Adds a numbered step to `apps/designer/README.md`'s manual-smoke checklist if the visual or interaction surface is new.

### Write-back pattern (load-bearing)

```tsx
// good — separate tagPath for write-back, falls through binding for read
<NumericInput binding={readBinding} tagPath="MyDriver/SetPoint" />

// bad — assumes "value" or hardcodes the write target
<NumericInput binding={readBinding} />   // will write to "value" — wrong
```

`NumericInput`, `Slider`, `Dropdown`, `ToggleSwitch` all follow this. New input components must too. The v1.1 architectural fix (exposing the bound path through the binding system) is tracked separately; until then, this pattern is the contract.

## Protocol contract

`packages/protocol-ts`:

- **Mirrors `crates/protocol`** message-for-message. A protocol change is a paired Rust + TS change in the same PR.
- **No drift** between Rust and TS shapes. If you add a field on one side, add it on the other in the same commit.
- **No business logic** lives here — only type definitions, discriminated unions, and pure parsers/serializers.
- **No `any`** in exported types. If a field is genuinely polymorphic, model it as a tagged union.

## Tooling

Same as apps: pnpm, Vitest, strict TypeScript, no `@ts-nocheck`. See `apps/AGENTS.md` for the tooling baseline; the only divergence here is that `packages/*` libraries are built as libraries (not bundled apps) — their `package.json` declares `main`/`module`/`types` for downstream consumption.

## Tests

- `pnpm -r test` runs vitest across all packages.
- `component-library` additionally has `pnpm build` (rollup or vite library mode); a broken build blocks downstream apps.
- Component tests use React Testing Library — no enzyme, no shallow-render adapters.
- Tag updates in tests use the same `protocol-ts` types as production; don't fake them with `any`.

## See also

- `AGENTS.md` (root) — codebase-wide rules.
- `apps/AGENTS.md` — TS apps.
- `VISION.md` — won't-merge policy.
- `crates/protocol/` — the Rust side of the protocol contract.
