---
id: CODEX-D
title: .github/workflows/ci.yml — Phase 0 CI
owner: codex
phase: 0
status: in-progress
created: 2026-04-26
last-update: 2026-04-26 13:41 codex
---

# CODEX-D — Phase 0 CI workflow

## Brief

### Goal

GitHub Actions workflow that exercises the Rust workspace and the pnpm workspace on every push to `main` and every PR. Catches regressions before review burden lands on Claude.

### Context to read first

- `Cargo.toml` (workspace root) — current members.
- `pnpm-workspace.yaml` — current packages.
- `rust-toolchain.toml` — pinned channel.
- `docs/contributing.md` §4.4 — the local commands the CI mirrors.

### Files to create

- `.github/workflows/ci.yml`.

### Behavior

Single workflow file with two jobs running in parallel: `rust` and `node`.

#### Triggers

```yaml
on:
  push:
    branches: [main]
  pull_request:
```

#### `rust` job

- Runs on `ubuntu-latest`.
- Steps:
  1. `actions/checkout@v4`
  2. `dtolnay/rust-toolchain@stable` with `components: rustfmt, clippy`
  3. `Swatinem/rust-cache@v2`
  4. `cargo fmt --all -- --check`
  5. `cargo clippy --workspace --all-targets -- -D warnings`
  6. `cargo test --workspace --locked`

#### `node` job

- Runs on `ubuntu-latest`.
- Steps:
  1. `actions/checkout@v4`
  2. `pnpm/action-setup@v4` with `version: 9`
  3. `actions/setup-node@v4` with `node-version: 20` and `cache: pnpm`
  4. `pnpm install --frozen-lockfile`
  5. `pnpm -r typecheck` (skip per-package if no `typecheck` script — use `--if-present`)
  6. `pnpm -r test --if-present`
  7. `pnpm -r build --if-present`

> Note: as of this brief, only Rust crates exist in `members`; pnpm `packages:` already lists `packages/protocol-ts` and `apps/runtime-web` but those packages haven't landed yet. The `node` job must therefore tolerate a workspace whose `packages:` patterns currently match no `package.json` files (this happens until CODEX-A and CODEX-C ship). If `pnpm install` errors on this, use `--if-present`-equivalent guards or set `pnpm install` to ignore missing packages. **Do not** break the workflow with strict checks that would force `node` to fail until A and C land — that creates a deadlock where CODEX-A can't be merged because CI is red.

### Concurrency

Add a concurrency group so superseded PR runs are cancelled:

```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

### Acceptance criteria

- [ ] File at `.github/workflows/ci.yml`.
- [ ] On a clean checkout of `main`, both jobs pass green.
- [ ] After CODEX-A merges, the `node` job runs `pnpm test` and it passes (vitest from A).
- [ ] After CODEX-B merges, the `rust` job's `cargo test --workspace` includes the gateway integration test and it passes.
- [ ] Workflow runs in under 5 minutes on a cached run.

### Out of scope

- Release / publish workflows.
- Code coverage reporting.
- Matrix testing across OSes (Linux only for v1).
- Tauri build job (deferred to Phase 2).
- License / dependency audit (deferred).

### Risks / gotchas

- **Sequencing.** The `rust` job will fail until CODEX-B lands, because `crates/gateway` is referenced in `[workspace] members` but its `Cargo.toml` does not yet exist. The `node` job will fail until CODEX-A lands (and CODEX-C, since `apps/runtime-web` is in `pnpm-workspace.yaml`). Therefore CI **goes red on first push and stays red until A, B, and C have all landed.** That is acceptable for the bootstrap; it would not be acceptable in a healthy project. Note the temporary red period in your `## Codex log` entry. Do not work around the issue by removing strict checks — the right fix is the other tasks landing.
- `cargo test --locked` requires `Cargo.lock` to be committed at the repo root. As of this brief, `Cargo.lock` is **not** tracked. Either commit `Cargo.lock` as part of this task (recommended for an application repo) or drop `--locked` from the test command. Pick one and document the choice in your `## Codex log` entry.
- `pnpm install --frozen-lockfile` requires `pnpm-lock.yaml` at the repo root. Same situation. If the lockfile doesn't exist, run `pnpm install --no-frozen-lockfile` once locally (after CODEX-A creates the first real package) to generate it, then commit it as part of this task. If you land CI before A, drop `--frozen-lockfile`.
- The `pnpm` workspace currently lists packages whose directories don't exist yet. `pnpm install` should still succeed (it just finds zero matches per glob) on pnpm 9.x.
- Don't skip rustfmt/clippy. They're cheap and the project is small; let them stay strict from day 1.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 13:41  codex
I identified as codex and read `docs/agents/README.md`, `board.md`, and this task brief. I started CODEX-D after A/B/C files existed locally so the CI workflow can target real workspace members.

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
