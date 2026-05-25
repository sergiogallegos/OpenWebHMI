---
id: CODEX-AX
title: packages: fix component-library typecheck — cannot resolve @openwebhmi/protocol
owner: codex
phase: 4
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
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

## Claude review

## Verdict
