---
id: CODEX-BB
title: Rust 1.96 toolchain bump — 1.95.0 → 1.96.0 + one assert_matches! example conversion
owner: codex
phase: 4
status: open
created: 2026-05-29
last-update: 2026-05-29 claude [Opus 4.7]
---

# CODEX-BB — Rust 1.96 toolchain bump (Tier 5 mechanical cleanup)

## Brief

> Bump the workspace Rust toolchain from `1.95.0` to `1.96.0`, absorb whatever clippy fallout appears, and convert **one representative** `match { ... => panic!() }` test site to the newly-stabilized `assert_matches!` macro as an example for future contributors. **Scope-locked, mirrors CODEX-AM's shape.** No broader test refactoring, no dep bumps, no `core::range::Range` adoption, no `LazyCell`/`LazyLock` `From<T>` ergonomic conversions.

### Goal

`rust-toolchain.toml` pins `channel = "1.96.0"`. The full validation matrix (`cargo build`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`) is clean. CI Rust job green on the resulting push. One test in `crates/audit-log/tests/hash_chain.rs` uses `core::assert_matches::assert_matches!` to demonstrate the v1.96 idiom for future contributors.

### Context to read first

- `rust-toolchain.toml` — the channel pin. Currently `1.95.0`.
- `Cargo.toml` workspace `[workspace.package]`: `rust-version = "1.85"` (MSRV — **not** changed by this brief), `edition = "2024"` (also unchanged).
- Rust 1.96 release notes: https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/ — newly stabilized items.
- `core::assert_matches::assert_matches!` rustdoc — the macro signature and matching examples.
- `crates/audit-log/tests/hash_chain.rs:74-81` (the `migrates_pre_hash_schema_and_records_migration_event` test) — the canonical `match { Variant => assert_eq!(...), other => panic!("...{other:?}") }` site that becomes the example conversion.
- [`docs/agents/notes/toolchain-drift.md`](../notes/toolchain-drift.md) — environment-mismatch discipline.
- CODEX-AM task file (`docs/agents/tasks/CODEX-AM-stdlib-deps-modernization.md`) — the canonical shape for Tier-4-style narrow mechanical cleanups. BB follows the same shape.
- CODEX-AG task file — the original toolchain pin / edition migration. Useful for the "what could surface during a minor-version toolchain bump" mental model.

### Files to create / modify

1. **Modify** `rust-toolchain.toml`:
   ```toml
   [toolchain]
   channel = "1.96.0"
   components = ["rustfmt", "clippy"]
   ```
   (Change is the single channel line.)

2. **Modify** `crates/audit-log/tests/hash_chain.rs`:
   - Add `use core::assert_matches::assert_matches;` at the top alongside the existing `use` statements.
   - In `migrates_pre_hash_schema_and_records_migration_event` (~line 74-81), replace the `match { ... => assert_eq!, other => panic! }` block with a single `assert_matches!(&entries[0].kind, AuditEvent::MigrationCompleted { rows } if *rows == 50);` invocation (or equivalent guard-clause shape).
   - Do NOT touch any other test in this file. The conversion is an **example pattern**, not a sweep. Other tests stay as they are.

3. **Modify** crate `lib.rs` files if and only if new clippy lints fire — minimal `#[expect(...)]` annotations with a one-line reason. Don't `#[allow]` per CLAUDE.md "Rust code quality" discipline; use `#[expect]`. If a new lint is genuinely correct and the fix is one-line, fix the code instead of suppressing.

4. **Do NOT** modify:
   - `Cargo.toml` workspace `rust-version` (stays at `1.85`; this brief doesn't move the MSRV).
   - `Cargo.toml` workspace `edition` (stays at `2024`).
   - `Cargo.lock` (no dep changes; the toolchain bump may regenerate transitively but the diff should be bounded).
   - `.github/workflows/ci.yml` (already uses `dtolnay/rust-toolchain@stable` which honors `rust-toolchain.toml`).
   - Any other test files in `crates/audit-log/tests/` or elsewhere — the `assert_matches!` conversion is one site only.

### Behavior

- `cargo build --workspace --all-features --locked` — clean.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — clean. New lints (if any) addressed with `#[expect(...)]` or code fix per scope rules.
- `cargo test --workspace --all-features --locked` — clean; the converted `migrates_pre_hash_schema_and_records_migration_event` test passes with the new `assert_matches!` shape.
- `cargo doc --workspace --no-deps --locked` — clean (no missing-docs regressions).
- CI Rust job green on push.
- `Cargo.lock` drift bounded — toolchain bump alone shouldn't churn the lockfile. If it does, **stop and investigate** (per CLAUDE.md "Cargo.lock diff is bounded" rule).

### Test requirements

- Full validation matrix runs locally three times (per CLAUDE.md "Three consecutive runs for known-flaky integration tests" — toolchain bumps can surface subtle codegen differences worth confirming determinism).
- The converted test demonstrably still catches the same bug class (insert the wrong `rows` value into the migration's `MigrationCompleted` event; revert; confirm `assert_matches!` fails with a useful Debug message).
- CI Rust job green on the pushed commit.

### Acceptance criteria

- [ ] `rust-toolchain.toml` `channel = "1.96.0"`.
- [ ] Workspace `Cargo.toml` `rust-version = "1.85"` and `edition = "2024"` **unchanged**.
- [ ] Full validation matrix clean (build + clippy + test + doc) three consecutive runs locally.
- [ ] CI Rust job green on the resulting push.
- [ ] `crates/audit-log/tests/hash_chain.rs::migrates_pre_hash_schema_and_records_migration_event` uses `assert_matches!`; all 5 hash_chain tests still pass.
- [ ] No other tests touched. No `core::range::Range` adoption. No `From<T>` ergonomic conversions for `LazyCell` / `LazyLock` / `AssertUnwindSafe`.
- [ ] `Cargo.lock` drift bounded (toolchain bump alone — no unrelated package version moves).
- [ ] Codex log states whether any new clippy lints fired and how each was addressed (`#[expect]` with reason vs code fix).

### Out of scope

- **Sweeping all `match { ... => panic!() }` test patterns to `assert_matches!`.** One example conversion only. Future contributors apply the pattern drift-on-touch.
- **Adopting `core::range::Range` family.** New type, not a drop-in for `std::ops::Range`. Brief on-demand if a specific hot path needs Copy ranges.
- **`From<T>` ergonomic conversions for `LazyCell` / `LazyLock` / `AssertUnwindSafe`.** Drift-on-touch only.
- **Dependency version bumps.** No `cargo update` of any kind. Cargo security fixes from 1.96 (CVE-2026-5222, CVE-2026-5223) apply automatically via the toolchain bump; no Cargo.toml changes needed.
- **MSRV bump.** Workspace `rust-version = "1.85"` stays; the toolchain pin moves to 1.96 but the codebase doesn't start using 1.96-specific features beyond the one `assert_matches!` example.
- **WebAssembly target changes** (1.96 makes undefined symbols errors in the wasm linker). OpenWebHMI doesn't target wasm.
- **Tauri / src-tauri toolchain implications.** Tauri builds against the workspace toolchain; if Tauri-build surfaces a 1.96-specific issue, that's a separate brief.
- **Any clippy lint sweeps** beyond what new 1.96 lints surface. Don't fix pre-existing `#[allow(...)]` annotations as a side-effect.
- **Restructuring the assert_matches example.** One test site, narrowest possible diff.

### Risks / gotchas

- **New clippy lints under `-D warnings`.** Minor Rust versions often add lints. Likely candidates in 1.96 territory: `redundant_pattern_matching` extensions, `manual_range_contains` variants, new `dead_code` heuristics. Use `#[expect(<lint>, reason = "...")]` per CLAUDE.md "Rust code quality" rule — not `#[allow]`.
- **`assert_matches!` import path is `core::assert_matches::assert_matches!`** — note `core`, not `std`. The macro is re-exported from `std::assert_matches` per stdlib convention, so `use std::assert_matches::assert_matches;` also works. Match whichever idiom the test file's other imports use.
- **Workspace MSRV vs toolchain pin distinction**: `rust-version = "1.85"` (workspace MSRV) declares the minimum the *consumers* of the crates need; `rust-toolchain.toml` `channel = "1.96.0"` is what *contributors* use to build. The toolchain bump doesn't break MSRV consumers unless code starts using 1.86+ features unconditionally — and `assert_matches!` stabilized in 1.96, so the converted test technically requires MSRV ≥ 1.96 to compile. **This is acceptable for test code** because tests aren't part of the published API surface, but worth confirming the workspace test-feature configuration doesn't propagate the MSRV requirement to consumers. Inspect the audit-log Cargo.toml [dev-dependencies] vs published features carefully.
- **`Cargo.lock` drift is bounded but not always zero** under toolchain bumps. The 1.96 cargo may pick newer compatible versions of `proc-macro2`, `syn`, `quote`, or `serde_derive` (proc-macro transitives). If the lockfile churns beyond proc-macro infrastructure, **stop and investigate** — that's a red flag per CLAUDE.md "Cargo.lock diff is bounded" rule.
- **`cargo doc` may surface new rustdoc warnings** about reference resolution or intra-doc-link formatting. Address with the same `#[expect]` discipline as clippy lints.
- **CI cache invalidation**: `Swatinem/rust-cache@v2` keys on the toolchain version; first CI run post-bump will be slow (cache rebuild). Subsequent runs use the new cache. No action needed; just expect longer first build.
- **The `assert_matches!` example is a v1.96-floor test** — it won't compile on older Rust. The workspace's `rust-version = "1.85"` is for the *library* crate API surface; tests aren't subject to the same MSRV declaration. Confirm `cargo test -p openwebhmi-audit-log` passes; don't need to bump workspace MSRV.
- **Honesty discipline (per CLAUDE.md)**: Codex log states **exactly** how many new clippy lints fired, what each was, and how addressed (one-line `#[expect]` with reason vs code fix). Don't say "no fallout" if there was any, however small.
- **Don't reach for `core::range::Range`**. The new `Copy + IntoIterator` range types are tempting for hot paths but require explicit imports and don't replace existing `0..n` syntax. Separate brief if a specific hot path benefits.
- **Don't sweep**. The one converted test is an example, not a stylistic mandate. Other tests using `match { ... => panic!() }` stay as they are until drift-on-touch.

## Codex log

## Claude review

## Verdict
