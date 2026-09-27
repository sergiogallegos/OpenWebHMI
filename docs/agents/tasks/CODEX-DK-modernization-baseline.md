---
id: CODEX-DK
title: Rust 1.98.1 and TypeScript baseline, web-first designer, architecture restart review
owner: codex
phase: 4
status: submitted
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DK — Modernization baseline and restart review

## Brief

Maintainer-directed review/update: adopt Rust 1.98.1, verify TypeScript and
rust-ethernet-ip currency, assess dependency-light reusable Rust services and a
web-first designer while preserving cross-platform desktop options, and recommend
the order of architecture/security/performance work before resuming development.

Deliver a version inventory, verified code findings, target dependency boundaries,
and an ordered migration in `docs/planning/2026-09-architecture-review.md`. Update
the build baseline and project direction without claiming the proposed core
extraction or production browser delivery is complete. Do not update all Cargo
dependencies, alter protocol pins, change licensing, or push automatically.

Acceptance: Rust 1.98.1 build/clippy/doc/fmt; full Rust and frontend suites three
consecutive green runs; frontend typecheck/build; frozen pnpm install; repository
validators. Record any unexecuted OS/browser/hardware/packaging checks explicitly.

## Codex log

### 2026-09-27 13:52 codex [GPT-6]

Started under direct maintainer instruction. Existing BB proposed 1.96; this newer
baseline supersedes that version direction without rewriting BB's brief or claiming
its example-test acceptance criteria. Verified rust-ethernet-ip already resolves
1.2.1; BK submission review and hardware gates are unchanged. Rust minimum is
explicitly raised to the compiler now tested, 1.98.1, rather than retaining 1.85.

TypeScript 7.0.2 for product apps/libraries; Astro stays on 5.9.3 after a trial of
6.0.2 exposed incompatible declared peer ranges. Node 24.21.0 LTS and pnpm 9.15.9
are the validation baseline. Browser designer build becomes default; optional
desktop packaging remains `build:desktop`; unused Tauri JS API removed. Core
dependency inversion is proposed and documented, not implemented by this task.

### 2026-09-27 13:55 codex [GPT-6]

Submitted as an uncommitted diff. Validation on macOS arm64, Rust 1.98.1,
Node 24.21.0, pnpm 9.15.9:

- `cargo build --workspace --all-features --locked`: passed after final Rust edits.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --workspace --all-features --locked`: three consecutive green runs,
  165 tests per run. Initial sandbox failure to bind a simulator socket was not
  counted; successful runs used authorized loopback networking.
- `cargo doc --workspace --no-deps --locked` and `cargo fmt --check`: passed.
- `pnpm -r typecheck` and `pnpm -r build`: passed with Node 24.21.0.
- `pnpm -r test`: three consecutive green runs, 128 tests per run.
- `pnpm install --frozen-lockfile`: passed; no peer mismatch after retaining
  TypeScript 5.9.3 for the existing Astro graph.
- License validator with self-test, agent-file validator, review local links and
  `git diff --check`: passed.

Rust lint migration: one manually fixed collapsible conditional in ProjectStore;
15 compiler-suggested fixes across 10 files (12 collapsible conditionals, two
array-chunk conversions, one is_multiple_of conversion). Reviewed all diffs and
formatted them; converted the touched gateway result_large_err allow to a reasoned
expect. No new behavior tests: Rust edits preserve behavior and use the existing
suite's three-run mechanical-change gate. TypeScript library emission demonstrably
failed with TS5011 before adding rootDir, then built successfully afterward.
Cargo.lock is unchanged. pnpm lock changes are the TypeScript compiler/platform
packages and removal of the unused Tauri API dependency, without broad updates.

Residual limits: existing deprecated frontend transitives and a designer chunk
above 500 kB remain; no performance speedup claimed. No physical PLC/24-hour soak,
Linux/Windows execution, interactive browser/offline smoke, desktop installer build,
or hosted CI run performed. Native shell compilation is covered by the Rust
workspace build. No commit or push performed. Security backlog, production web
hosting, framework majors and core extraction remain explicit follow-up work in
the linked architecture review.

## Claude review

## Verdict
