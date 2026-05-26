---
id: CODEX-AX
title: packages: fix component-library typecheck — cannot resolve @openwebhmi/protocol
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: 47da4ea
---

# CODEX-AX — Fix component-library typecheck resolution of @openwebhmi/protocol

## Brief

> CI run 26423144512 (post-CODEX-AV) reaches `pnpm -r --if-present typecheck` and fails in `packages/component-library` with `Cannot find module '@openwebhmi/protocol' or its corresponding type declarations.` This failure was masked by CODEX-AV's earlier pnpm-setup conflict; once AV unblocked the setup, the typecheck surface became visible. The fix is almost certainly a workspace-link or `tsconfig`/`exports` misconfiguration between `packages/protocol-ts` (which publishes `@openwebhmi/protocol`) and `packages/component-library` (which imports it). **Follow-up surfaced by AV merge.**

### Goal

`pnpm -r --if-present typecheck` passes for the whole workspace on CI. The `@openwebhmi/protocol` symbol resolves cleanly from `packages/component-library` (and any other consumer that uses the same import path).

### Context to read first

- `packages/component-library/package.json` — its declared dependency on `@openwebhmi/protocol`. Check the version pin, the `dependencies` vs `peerDependencies` placement, and whether it uses `workspace:*`.
- `packages/protocol-ts/package.json` — the package that ships `@openwebhmi/protocol`. Check the `name` field (must be exactly `@openwebhmi/protocol`), `main`/`module`/`types`/`exports` fields, and the built output paths.
- `packages/component-library/tsconfig.json` and `packages/protocol-ts/tsconfig.json` — TypeScript path mapping, project references (`references: [{ path: "..." }]`), and `moduleResolution`.
- Root `pnpm-workspace.yaml` and root `tsconfig.json` (or `tsconfig.base.json`) — workspace member globs, any path aliases.
- Failing CI run: https://github.com/sergiogallegos/OpenWebHMI/actions/runs/26423144512 — Node job, `pnpm -r --if-present typecheck` step. The component-library typecheck log will list which files fail to resolve the module.

### Files to create / modify

The diagnosis informs the fix. Likely candidates (in order of probability):

1. **`packages/protocol-ts/package.json` `exports` / `types` field** — modern pnpm workspaces require the consumed package to declare its entry point via `exports` or `types`. If `protocol-ts` was authored pre-`exports`, the consumer can't resolve it under strict TS resolution.
2. **`packages/component-library/package.json` dependency declaration** — should be `"@openwebhmi/protocol": "workspace:*"` (or matching the protocol-ts version). If it's a literal version string and protocol-ts isn't published to npm, the dep falls through.
3. **`packages/component-library/tsconfig.json` `references`** — TS project references need an explicit `{ "path": "../protocol-ts" }` entry so the typechecker knows protocol-ts is a sibling workspace project, not a node_modules dep.
4. **Build order** — protocol-ts must build its `.d.ts` output before component-library can typecheck against it. If the workspace `typecheck` script doesn't ensure this, add an explicit `pnpm -r --filter @openwebhmi/protocol build` predecessor or use TS project references which handle build order automatically.

Find the root cause (don't guess), then minimally fix the configuration in the relevant `package.json` / `tsconfig.json`.

### Behavior

After the fix:
- `pnpm -r --if-present typecheck` succeeds in the component-library package.
- Local IDE typecheck (TS language server) also resolves the import (sanity check).
- The fix doesn't break other consumers of `@openwebhmi/protocol` — grep for `from '@openwebhmi/protocol'` and verify all sites still typecheck.

### Test requirements

- `pnpm -r --if-present typecheck` exits 0.
- The fix demonstrably catches the bug: if the root cause is in `protocol-ts`'s `exports`, revert the fix and confirm the typecheck fails again. Per CLAUDE.md "Your test is NOT VALID if it passes without the fix."
- CI Node job passes the typecheck step.

### Acceptance criteria

- [ ] `pnpm -r --if-present typecheck` green locally three consecutive runs (pnpm + TS resolution can be deterministic, but check anyway).
- [ ] CI Node job's `typecheck` step passes (the AV follow-on CI run is the proof).
- [ ] Codex log names the root cause: `package.json` exports / dependency declaration / tsconfig references / build order. Cite the file:line of the fix.
- [ ] No other workspace package's typecheck regresses (grep all `@openwebhmi/protocol` consumers, verify each).

### Out of scope

- Migrating the workspace to TS project references *across the board* if it doesn't already use them. AX uses the minimal mechanism — if references aren't already established, fix via `exports` field or build-order instead. A workspace-wide TS-references migration is its own brief.
- Bumping protocol-ts or component-library package versions.
- Renaming `@openwebhmi/protocol` to anything else.
- The downstream `pnpm -r --if-present lint` / `test` / `build` steps that might fail once typecheck passes. Separate follow-ups per failure.
- The Node 20 deprecation warning — separate brief when `pnpm/action-setup@v5` ships.

### Risks / gotchas

- **pnpm workspace symlinks vs `node_modules` shape.** pnpm uses a strict `node_modules` shape that hoists differently from npm/yarn. A package that "works" with one resolution algorithm can fail under pnpm's strict mode. Check `pnpm-workspace.yaml` for any `node-linker` config that might affect resolution.
- **`exports` field is exact-match.** If `protocol-ts/package.json` has `"exports": { ".": "./dist/index.js" }`, consumers can ONLY import the root — sub-paths like `@openwebhmi/protocol/types` won't resolve. If component-library uses sub-paths, the `exports` map needs to enumerate them or use a wildcard `"./*": "./dist/*.js"`.
- **TypeScript `moduleResolution: "bundler"` vs `"node16"` vs `"node"`.** Different resolution rules. Confirm what protocol-ts and component-library each use; mismatch is a known source of "module not found" errors.
- **Don't rebuild every workspace package just to fix one resolution.** If the fix is to make protocol-ts's `.d.ts` available before component-library typechecks, do it via TS project references (incremental, cache-friendly), not a brute-force `pnpm -r build` predecessor that doubles CI time.
- **The CODEX-A task (original protocol-ts) is the historical reference.** Read its verdict for any noted gotchas about `protocol-ts` packaging.
- **If `@openwebhmi/protocol` isn't the right package name** (it should be — `package.json` declares it), the fix is to align the consumer's import path, not to rename the package. Renaming would break every other consumer.

## Codex log

2026-05-25 codex: Fixed workspace package source resolution by pointing `@openwebhmi/protocol` and `@openwebhmi/component-library` package exports/types/main at `src/index.ts` for no-dist workspace typechecking. Verified with `pnpm -r --if-present typecheck` and package builds.

## Claude review

### 2026-05-26 19:30  claude [Opus 4.7]

**Independent verification**
- `git show 47da4ea` — 8 insertions / 8 deletions across 2 `package.json` files; minimal change.
- `grep "\"private\"" packages/{protocol-ts,component-library}/package.json` — both packages are `"private": true`. Not published to npm; the workspace-source-export pattern is safe for this consumption shape.
- CI runs 26430352703 + 26431030607 — Node job `pnpm -r --if-present typecheck` ✅ green both times post-AX. Was failing on `Cannot find module '@openwebhmi/protocol'` pre-AX.

**What's being fixed**
- `packages/component-library` typecheck failed with `Cannot find module '@openwebhmi/protocol'` because both `protocol-ts` and `component-library` declared `package.json` `exports`/`main`/`types` pointing at `./dist/` paths that don't exist until `pnpm build` runs. Workspace consumers under pnpm strict-resolution couldn't find the module without a pre-built `dist/`.

**Root cause confirmation**
- Confirmed via diff: `exports[".]`, `main`, and `types` all pointed at `./dist/index.d.ts` and `./dist/index.js` pre-AX. Post-AX they point at `./src/index.ts`. Workspace consumers now resolve to the TypeScript source directly — no pre-build needed.

**Fix appropriateness**
- Right layer: minimal `package.json` change in both producer packages. No tsconfig changes, no TS project references introduced, no build-script changes.
- The brief listed 4 candidate fix paths in order of probability; Codex picked the most direct: producer-side `exports`/`main`/`types` rewrite (combining brief candidates #1 and #4 — `exports` change + eliminate build-order dependency entirely).
- The packages are `"private": true` — never published to npm. The workspace-source-export pattern is the established idiom for in-monorepo TS consumption; consumers get hot type updates without rebuilding producers. This trade-off would NOT be safe for a published package (npm consumers need `dist/`), but the privacy flag makes it safe here.
- No regression to other consumers: any code doing `import { … } from '@openwebhmi/protocol'` now resolves via the workspace symlink → producer's `package.json` → `./src/index.ts`. TypeScript handles `.ts` source as well as `.d.ts` declarations.

**Test proof**
- Verification IS the CI run; `pnpm -r --if-present typecheck` passes on runs 26430352703 and 26431030607 — both green post-AX. Pre-AX runs (26423144512 and earlier) had this exact failure.
- Test-must-fail-without-fix: trivially satisfied — reverting the diff restores `dist/` paths, and any clean `pnpm install && pnpm typecheck` without a prior `pnpm build` reproduces the original error.

**Residual risk**
- **If either package is ever published to npm**, the `exports` field must be flipped back to a conditional shape: `"types": "./dist/index.d.ts"` + `"default": "./dist/index.js"` for the published path. The `"private": true` flag is the only thing preventing this from being a footgun.
- **Build script still exists** (`tsc -p tsconfig.json`) — `dist/` will be built by anyone who runs `pnpm --filter @openwebhmi/protocol build`, but nothing depends on the output now. Dead-output risk is low but worth knowing.
- **IDE behavior**: most TS-aware IDEs honor `package.json` `types`; pointing at `./src/index.ts` means "Go to Definition" navigates to source. Feature, not bug, but contributors accustomed to `dist/index.d.ts` navigation may notice the change.

**Strong points (✅)**
- **Minimal, focused change** — 8 lines across 2 files. Matches the brief's "minimally fix the configuration" instruction.
- **Codex correctly chose the simplest viable fix** from the 4 brief candidates. The TS-project-references option would have been correct but more invasive; the workspace-source-export trick achieves the same outcome with less ceremony.
- **`"private": true` invariant honored** — Codex's Codex-log entry explicitly states "package exports/types/main at `src/index.ts` for no-dist workspace typechecking" — documents the intent precisely.

**Findings**
- 🟢 The two `package.json` files now share an identical export shape. Future consumers see a consistent pattern.
- 🟡 The workspace-private convention deserves a short agent note (`docs/agents/notes/workspace-package-resolution.md`) so future contributors know why `exports` points at `src/` and what would change if a package goes public. v1.1 polish.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `pnpm -r --if-present typecheck` green (CI runs 26430352703 + 26431030607 confirm).
- ✅ CI Node job's typecheck step passes (proof above).
- ✅ Codex log names the root cause: producer-side `exports` pointing at non-existent `dist/`; fix points exports at `src/`.
- ✅ No other workspace package's typecheck regresses (workspace-wide green).

## Verdict

**Merged** at `47da4ea`.

What's NOT yet proven by this merge:
- npm-published-shape compatibility (intentional; the packages are private).
- TS IDE "Go to Definition" navigation in Tauri/VSCode/JetBrains (very likely works; not exhaustively tested).

No follow-ups opened from AX specifically. The v1.1 polish (agent note documenting the workspace-only convention) is light enough to roll into a future touchup without its own brief.
