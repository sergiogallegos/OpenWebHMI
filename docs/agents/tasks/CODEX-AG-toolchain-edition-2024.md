---
id: CODEX-AG
title: Workspace toolchain modernization — pin Rust 1.95.0 + edition 2024
owner: codex
phase: 4
status: open
created: 2026-05-02
last-update: 2026-05-02 claude
---

# CODEX-AG — Rust 1.95 + edition 2024 sweep

## Brief

> **Phase 4 dev-experience cleanup, ahead of v1.0.** The workspace currently floats on `channel = "stable"` in `rust-toolchain.toml`, holds `edition = "2021"`, and pins `rust-version = "1.80"`. That means Codex, CI, and the maintainer can land on different rustc minors and we deliberately avoid every language feature added since July 2024 — including the entire 2024 edition (stable since Rust 1.85, Feb 2025). Lock the toolchain to **Rust 1.95.0** (latest stable, released 2026-04-14), bump the workspace to **edition 2024**, raise MSRV to **1.85**, and run the mechanical migration. No behavior changes; this is a toolchain bump, not a refactor.

### Goal

After this lands:

- `rustup show` resolves to `rustc 1.95.0` for everyone touching the repo.
- `cargo build --workspace` compiles on edition 2024 with no warnings beyond what was present before.
- The full test matrix (`cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`) is green on three consecutive runs.
- CI uses the same toolchain as local — no more "passes locally, fails in CI" or vice-versa from rustc drift.

### Context to read first

- `rust-toolchain.toml` — currently `channel = "stable"` with no version pin. This is the file that needs the version lock.
- `Cargo.toml` (workspace root) — `[workspace.package]` has `edition = "2021"` and `rust-version = "1.80"`. These are the two lines that change globally; all 19 member crates use `edition.workspace = true` and inherit, so individual `Cargo.toml`s do **not** need touching.
- `.github/workflows/ci.yml:18` — `dtolnay/rust-toolchain@stable` with no `toolchain:` input. The action reads `rust-toolchain.toml` automatically, so pinning that file pins CI. Verify this still holds after the change; if not, add an explicit `toolchain: 1.95.0` input.
- The Rust 2024 edition migration guide — read sections **"Tail expression scopes"**, **"Never type fallback"**, **"`gen` keyword reservation"**, **"Match ergonomics with `&` patterns"**, **"Macro fragment specifiers"**, and **"Prelude additions (`Future`, `IntoFuture`)"** before running `cargo fix --edition`. These are the changes most likely to touch this codebase.

### Files to modify

- `rust-toolchain.toml` — change `channel = "stable"` to `channel = "1.95.0"`. Keep the `components = ["rustfmt", "clippy"]` line.
- `Cargo.toml` (workspace root) — `[workspace.package]`:
  - `edition = "2021"` → `edition = "2024"`
  - `rust-version = "1.80"` → `rust-version = "1.85"`
- Any source files that `cargo fix --edition --workspace --all-features` rewrites. Run it once, commit the diff, then run `cargo build --workspace --all-features` and verify it compiles. **Hand-audit the diff** — do not trust `cargo fix` blindly. Specifically look for:
  - Closures that captured by reference and now capture by move (the precise-capture change). If a `move` keyword was inserted, sanity-check the closure's lifetime story.
  - `?`-on-`Result<(), _>` patterns where never-type fallback shifted from `()` to `!`. The compiler will tell you if this breaks; the fix is usually a turbofish or an explicit `: ()` annotation.
  - Any identifier named `gen` (now reserved). None expected, but grep to be sure.
- `.github/workflows/ci.yml` — only if the existing `dtolnay/rust-toolchain@stable` action does not honor `rust-toolchain.toml`. Verify by reading the action docs or by inspecting CI logs after the first push. If it doesn't, add `toolchain: 1.95.0` under the `with:` block (and keep `components: rustfmt, clippy`).
- `wiki/index.md` (or wherever the dev-environment setup lives — check for an existing page first) — one-line note that the workspace requires Rust 1.85 minimum and pins 1.95.0 via `rust-toolchain.toml`.
- `CLAUDE.md` — update the "Project-specific gotchas" section to drop the Node-25 / rustc-drift line if it's now obsolete, or amend it to point at the pinned toolchain.

### Out of scope (explicit)

- **Do not touch `async-trait`.** The `Driver` trait in `crates/driver-api/src/trait_def.rs` is used as `Box<dyn Driver>` (see `supervisor.rs:75`, `trait_def.rs:55`/`112`, `mock.rs:111`). Native `async fn` in traits stabilized in Rust 1.75 but is **not** object-safe — replacing `#[async_trait]` here would require redesigning the supervisor's polymorphism story. Leave a `// TODO(v1.1): revisit when async-fn-in-trait becomes object-safe` near the `#[async_trait]` attribute on the `Driver` trait and move on.
- **No idiom-modernization sweep.** Do not rewrite `format!("{}", x)` to `format!("{x}")`, do not swap `if let Some(x) = ... else { return }` to `let-else`, do not introduce `LazyLock` (no `once_cell` or `lazy_static` is used in source today). If `cargo fix --edition` rewrites something idiomatic-but-not-required, that's fine; do not hand-edit beyond what `cargo fix` produces.
- **No dependency bumps.** Cargo.lock will update naturally from the rustc bump (some build-script-driven crates may rev). Do not run `cargo update` or bump pinned `=x.y.z` deps.
- **No clippy-rule additions or removals.** Stay on the existing `-D warnings` gate. If 1.95 introduces new lints that fire, fix them minimally; don't allowlist.

### Test requirements

- `cargo build --workspace --all-features` — clean compile on edition 2024.
- `cargo test --workspace --all-features --locked` — green on **three consecutive runs** (we have a known flake history; the toolchain bump must not regress flakiness).
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `cargo fmt --check` — clean. (rustfmt's edition-2024 defaults differ slightly from 2021; if it wants to reformat, accept the reformat as part of this task.)
- `pnpm -r typecheck` and `pnpm -r test` — green (these don't depend on rustc, but run them anyway as a regression check).
- The `examples/twincat-smoke/` and `examples/sim-*` example crates still build (`cargo build -p sim-rockwell` etc.).

### Acceptance criteria

- [ ] `rustc --version` reports 1.95.0 inside the repo (via `rust-toolchain.toml`).
- [ ] Workspace compiles on edition 2024 with no new warnings.
- [ ] Full test matrix green on three consecutive runs.
- [ ] `pnpm -r typecheck` + `pnpm -r test` green (regression sanity).
- [ ] CI passes on the PR (verify CI is using 1.95.0 — read the CI log).
- [ ] `async-trait` is **not** removed from `crates/driver-api`. The TODO marker is in place.
- [ ] One-line dev-env note exists in the wiki pointing at the pinned toolchain.

### Risks / gotchas

- **Three Codex tasks are open against this branch's tip:** CODEX-AD (ADS validation), CODEX-AE (audit log), CODEX-AF (backup). When this lands, those branches will need to rebase. Land AG **first**, then post hand-off notes telling Codex to rebase AD/AE/AF onto the new toolchain before submitting. The maintainer (claude) will coordinate the rebase notice; Codex does not need to track this in AG itself.
- **`tokio-modbus = "=0.16.1"`, `async-opcua = "=0.18.0"`, `rumqttc = "=0.25.1"`, `rumqttd = "=0.20.0"`, `prost = "=0.13.5"`, `jsonpath-rust = "=1.0.4"`, `ads = "=0.4.4"`** are version-pinned with `=`. Some of these may emit deprecation warnings under rustc 1.95 that they didn't under 1.93. If clippy fires on dependency-side issues that we can't fix, **do not** unpin the dep — instead, scope an `#[allow(...)]` to the narrowest call site with a comment referencing this task and the upstream issue.
- **PyO3 0.22 + Python 3.11/3.12 ABI on Windows.** The `crates/scripting` crate links to libpython. Edition bumps don't usually disturb this, but verify `cargo build -p openwebhmi-scripting` still links cleanly. If Codex hits a linker error, document the rustc version + pyo3 version + Python version in the Codex log and stop — that's a real upstream issue, not edition-bump fallout.
- **rustfmt 2024 edition changes are visible.** The 2024 edition flips a couple of rustfmt defaults (notably around let-chains formatting and trailing commas in match arms). Expect a non-trivial whitespace-only diff across the codebase from `cargo fmt`. Land it; don't hand-edit.
- **Tauri / `apps/designer/src-tauri`.** Tauri's macro expansions under edition 2024 have historically been slow to catch up to new rustc releases. If `cargo build -p openwebhmi-designer` fails with a macro error, report the exact tauri version + rustc version in the Codex log and pause. The maintainer will decide whether to upgrade tauri or revert AG.
- **Don't `cargo update`.** The temptation is real because the bump invites "while we're here, refresh deps." Resist — this brief is scope-locked to toolchain + edition. Dep refresh is a separate task.

## Codex log

<!-- Codex appends status transitions and notes here. -->

## Claude review

<!-- Claude reviews after submission. -->

## Verdict

<!-- Final disposition. -->
