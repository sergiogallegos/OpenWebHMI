---
id: CODEX-A
title: packages/protocol-ts — TS protocol types
owner: codex
phase: 0
status: merged
created: 2026-04-26
last-update: 2026-04-26 13:55 claude
merge-commit: 75ccb9c
---

# CODEX-A — `packages/protocol-ts` (TypeScript protocol types)

## Brief

### Goal

Hand-maintained TS counterparts of the types in `crates/protocol/src/lib.rs`. Wire form must round-trip with the Rust serde output exactly. No codegen yet (`ts-rs` / `specta` is a post-Phase-0 concern).

### Context to read first

- `crates/protocol/src/lib.rs` — the Rust source of truth, especially the `#[serde(...)]` attributes and the `tests` module that asserts literal wire forms.
- `docs/architecture.md` §4.3 (Protocol).
- `pnpm-workspace.yaml` — `packages/protocol-ts` is already registered.

### Files to create

- `packages/protocol-ts/package.json` — name `@openwebhmi/protocol`, `private: true`, `version: 0.0.1`. Build with `tsc`. Scripts: `build` (`tsc`), `test` (`vitest run`), `typecheck` (`tsc --noEmit`).
- `packages/protocol-ts/tsconfig.json` — `target: ES2022`, `module: ESNext`, `moduleResolution: bundler`, `strict: true`, `declaration: true`, `outDir: dist`, `rootDir: src`.
- `packages/protocol-ts/src/index.ts` — exported types + type guards.
- `packages/protocol-ts/src/index.test.ts` — vitest round-trip tests.

Do **not** modify `pnpm-workspace.yaml`; the package is already listed there.

### Type contract — must match Rust wire form exactly

JS has one numeric type, so `int` and `real` both map to `number`. The discriminator field names below are the literal JSON keys.

```ts
export type TagValue =
  | { type: "bool";   value: boolean }
  | { type: "int";    value: number  }   // i64 in Rust; JS number is fine for v1
  | { type: "real";   value: number  }
  | { type: "string"; value: string  };

export type Quality = "good" | "bad" | "uncertain" | "stale";

export type ClientMessage =
  | { kind: "tag.subscribe";   paths: string[] }
  | { kind: "tag.unsubscribe"; paths: string[] }
  | { kind: "ping" };

export type ServerMessage =
  | { kind: "tag.update"; path: string; value: TagValue; quality: Quality; ts: number }
  | { kind: "pong" }
  | { kind: "error"; code: string; message: string };
```

Plus type guards (defensive — handle `unknown` and unknown `kind` gracefully):

```ts
export function isClientMessage(x: unknown): x is ClientMessage;
export function isServerMessage(x: unknown): x is ServerMessage;
export function isTagValue(x: unknown): x is TagValue;
export function isQuality(x: unknown): x is Quality;
```

### Test requirements (vitest)

The tests in `crates/protocol/src/lib.rs` assert exact wire-form strings. Mirror those in TS so a regression on either side fails fast. Required cases:

- `JSON.stringify({ kind: "tag.subscribe", paths: ["system/sim/sin"] })` equals `'{"kind":"tag.subscribe","paths":["system/sim/sin"]}'`.
- `JSON.stringify({ kind: "ping" })` equals `'{"kind":"ping"}'`.
- `JSON.stringify({ type: "real", value: 2.5 })` equals `'{"type":"real","value":2.5}'`.
- `JSON.stringify("good")` equals `'"good"'`.
- For each `ServerMessage` variant: parse a literal JSON string from the Rust test fixtures and assert the type guard returns `true`.
- Type guards return `false` for: `null`, `undefined`, `{}`, `{ kind: "unknown" }`, `{ kind: "tag.update" }` (missing required fields), arrays, primitives.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/protocol build` exits 0 and emits `dist/index.js` + `dist/index.d.ts`.
- [ ] `pnpm --filter @openwebhmi/protocol typecheck` exits 0.
- [ ] `pnpm --filter @openwebhmi/protocol test` runs vitest, all cases above pass.
- [ ] No runtime dependencies (only devDependencies: `typescript`, `vitest`).
- [ ] Each exported symbol has a TSDoc comment.

### Out of scope

- Code generation (`ts-rs`, `specta`).
- React or DOM imports.
- Bundler config (esbuild, rollup) — `tsc` only.
- Versioning beyond `0.0.1`.

### Risks / gotchas

- `JSON.stringify` field order depends on insertion order in modern engines; the Rust serde output orders fields by their struct definition. If a test fails due to field order, re-construct the object with fields in the order Rust emits them (kind first, then payload fields in declaration order).
- TypeScript will infer `number` for both `int` and `real` — that's deliberate and correct for v1. Don't try to encode i64 vs f64 distinction in the TS type system.

## Codex log

*(codex — append your working notes here, timestamped per `README.md` conventions)*

### 2026-04-26 13:41  codex
I identified as codex and read `docs/agents/README.md`, `board.md`, this task brief, and the Rust protocol source. I started CODEX-A in parallel with CODEX-B because they have no code dependency.

### 2026-04-26 13:48  codex
Submitted. Implemented `packages/protocol-ts` with `@openwebhmi/protocol`, strict `tsconfig`, exported protocol types, TSDoc, defensive type guards, and vitest wire-form/guard tests. Verification: `pnpm --filter @openwebhmi/protocol build`, `typecheck`, and `test` pass; root `pnpm -r --if-present test/typecheck/build` also passes.

## Claude review

### 2026-04-26 13:55  claude — review pass 1

Spec-compliant. Code is clean and well-typed.

- ✅ Type contract matches `crates/protocol/src/lib.rs` wire form exactly.
- ✅ Type guards properly defensive: reject `null`/`undefined`/arrays/primitives, unknown `kind`, missing required fields, non-finite numbers.
- ✅ 9 vitest cases pass; literal wire-form assertions match the Rust serde output.
- 🟡 Minor: `isTagValue` accepts non-integer numbers like `3.14` for `type: "int"`. Mirrors what Rust serde produces (i64 → JSON number → JS number), so it's technically correct. `Number.isSafeInteger` tightening is a Phase 1 hardening only if integer-only call sites surface.

No follow-up tasks required.

## Verdict

**Merged** at `75ccb9c`. The minor int-vs-real validation note is tracked here only.
