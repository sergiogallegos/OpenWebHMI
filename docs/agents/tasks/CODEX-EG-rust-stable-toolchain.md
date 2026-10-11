---
id: CODEX-EG
title: Upgrade Rust and Cargo to verified stable 1.99
owner: codex
phase: 4
status: open
created: 2026-10-11
last-update: 2026-10-11 codex [GPT-6]
---

# CODEX-EG — Upgrade Rust and Cargo to verified stable 1.99

## Brief

### Goal

Upgrade the tested Rust/Cargo toolchain from 1.98.1 to verified stable 1.99.0 (or a newer stable patch verified when executing). Cargo ships with Rust; it is not a separately upgraded application dependency.

### Evidence and sources

The [official Rust announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) confirms 1.99.0 released on 2026-10-01. Current pins are `rust-toolchain.toml` and workspace `rust-version` in `Cargo.toml`. CI reads the toolchain file, installs rustfmt/clippy, and runs `cargo fmt --all -- --check`; that check passed on 2026-10-11 before any toolchain change.

### Required work

- Verify latest stable from official release metadata at execution; pin the exact tested version. Update compiler/MSRV declarations deliberately, root/scoped guidance and active build documentation together. Preserve edition 2024.
- Record `rustc --version`, `cargo --version`, rustfmt and clippy versions. Review Cargo 1.99 profile/CI behavior changes against existing debug validation; do not introduce release-only correctness or performance claims.
- Keep dependency versions and Cargo.lock unchanged unless a specific compiler compatibility issue is demonstrated and separately explained. EH owns general dependency upgrades.
- Resolve new compiler/clippy diagnostics narrowly. Preserve documented assertion policies and convert touched allow attributes per repository rules.
- Supersede BB's obsolete downgrade direction without claiming its unrelated assert_matches example is complete. DK's submitted baseline remains historical evidence, not proof of this upgrade.

### Acceptance and validation

- [ ] Exact stable toolchain and tested minimum are consistent across manifests, CI and active instructions.
- [ ] Run full root Rust build/clippy/test/doc/fmt matrix with locked resolution; existing suites pass three consecutive runs for mechanical changes. Run frontend checks if paired surfaces change.
- [ ] Record platform/native-packaging and hardware checks not run; report any formatter diff separately from semantic edits.

### Out of scope

General crate upgrades, protocol changes, architecture extraction and pushing without explicit direction.

## Codex log

## Claude review

## Verdict
