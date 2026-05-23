# apps/AGENTS.md

TypeScript application rules for `apps/{designer,runtime-web,website}`. Loaded automatically when work is under `apps/`. The root `AGENTS.md` applies in addition; this file is the TS-scoped tightening for the apps tree. For shared TS libraries see `packages/AGENTS.md`.

## Tooling

- **Package manager**: pnpm only. Never `npm` or `yarn`. `pnpm install`, `pnpm -r typecheck`, `pnpm -r test`, `pnpm -r build`.
- **Bundler**: Vite for `runtime-web` and `designer`. Astro for `website`. Do not introduce webpack or rollup configs.
- **Test runner**: Vitest. No jest, no mocha.
- **Type checking**: `tsc --noEmit` via `pnpm typecheck`. Strict mode is on in every `tsconfig.json` — don't loosen it.
- **No `@ts-nocheck`, no `@ts-ignore` without `// @ts-expect-error: <reason>` instead** (and only when there's a documented reason).

## Per-app responsibilities

- **`apps/runtime-web`** — browser HMI runtime. Loads views from the gateway, renders via `packages/component-library`, holds the WebSocket connection. No designer code. No build-time view authoring.
- **`apps/designer`** — Tauri desktop designer. Holds the project explorer, view editor, script editor (Monaco), driver config UI. Talks to a local or remote gateway via the same protocol as the runtime. No runtime-only concerns leak in.
- **`apps/website`** — public marketing + docs (`openwebhmi.com`). Astro. No runtime code. Builds independently of the rest of the workspace.

## React conventions

- React 18 (function components + hooks). No class components in new code.
- No `useEffect` for fetching data that the WebSocket connection already provides — subscribe to the connection's tag stream instead.
- Component files use `.tsx`. Pure logic in `.ts`. Tests colocated as `*.test.ts(x)`.
- Component props are explicit TypeScript interfaces, not inferred from `defaultProps`.

## Wire protocol

- The WebSocket protocol lives in `packages/protocol-ts` (paired with `crates/protocol`).
- Apps **consume** the protocol types; they do not extend them. Protocol changes are a paired Rust + TS change in the same PR (see root `AGENTS.md` git section).
- No ad-hoc REST endpoints. The gateway speaks one wire protocol (JSON over WebSocket); apps use it.

## Designer-specific gotchas

- **Designer changes that alter the runtime contract require a paired runtime change in the same PR.** Don't ship a new component config schema in the designer without runtime support; don't ship runtime rendering for a config the designer doesn't author yet.
- **Tauri shell** (`apps/designer/src-tauri/`) is the only place Rust code lives outside `crates/`. Keep the Rust there minimal — heavy logic belongs in a gateway crate.
- **Manual smoke step**: any user-visible designer change adds a numbered step to `apps/designer/README.md`'s manual-smoke checklist. The checklist is the pre-tag gate for `v0.2.0` / `v0.3.0` / future tags.

## Runtime-specific gotchas

- **No blocking work on the WebSocket thread.** Tag rendering must stay smooth at 50 concurrent clients × 10K tags. Profile before adding work to the tag update path.
- **Quality propagation matters.** When a driver goes bad → good → bad, the component shows it. The `Quality` enum is part of the protocol; don't drop it on the way to render.

## Build & test

- `pnpm -r typecheck` and `pnpm -r test` must pass before submission.
- `apps/designer` has `pnpm build:vite` and `pnpm build` (Tauri); both are part of the validation matrix for designer changes.
- `apps/website` builds standalone — its failures should not block runtime/designer work.
- Vitest tests use deterministic time (`vi.useFakeTimers()` + `vi.advanceTimersByTime()`); no `setTimeout`-based waits.

## See also

- `AGENTS.md` (root) — codebase-wide rules.
- `packages/AGENTS.md` — shared TS libraries (component-library, protocol-ts).
- `VISION.md` — won't-merge policy.
- `docs/architecture.md` — system topology and protocol boundaries.
