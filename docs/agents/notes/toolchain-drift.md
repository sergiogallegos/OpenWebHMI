# Toolchain drift

## The pinning

OpenWebHMI pins two toolchains:

- **Rust** — version is in `rust-toolchain.toml` at the repo root. CI installs the toolchain and components from the file with `rustup show`; it does not install a separate floating stable toolchain. Current pin: see `rust-toolchain.toml`; the codebase relies on edition 2024 features (let chains, `#[expect]`).
- **Node** — version is in `.node-version`, consumed by `actions/setup-node`. `pnpm` is pinned in root `package.json` (`packageManager`), consumed by `pnpm/action-setup`.

Local development should match both. Mismatches surface as either build errors that don't appear in CI, or — worse — green builds locally that fail in CI on different lints.

## Why this matters in reviews

When a submission reports verification failures that don't reproduce locally, **first** ask: what Rust version did Codex run? What Node version? Drift between Codex's environment and the reviewer's environment is the single most common source of "works on my machine" disagreement.

The honest review move is to:

1. Run the matrix with the pinned toolchains, not whatever's on PATH.
2. If the failure reproduces, it's a real bug; flag it normally.
3. If the failure doesn't reproduce, name the toolchain mismatch explicitly in the residual-risk section. Don't pretend it's resolved.

Codex's verification claim should include the toolchain versions used. If it doesn't, asking is cheaper than guessing.

## Common drift sources

- **Codex environment doesn't honor `rust-toolchain.toml`** — Codex's sandbox sometimes runs whatever stable is installed, not the pinned version. If the pinned version uses an edition-2024 feature that doesn't compile on older stable, the error will surface as a parse error, not an obvious version issue.
- **Local Node ≠ CI Node** — pnpm 9 vs pnpm 10 changes lockfile-resolution behavior. Use the exact `packageManager` version; major upgrades need a separate lockfile/build-script review.
- **macOS vs Linux** — most often a tokio file-descriptor or fs-watch behavior difference; surfaces in `crates/historian` or `crates/project-store` tests under load. Three consecutive runs catches most of these.

## See also

- `rust-toolchain.toml` — Rust pin.
- `.github/workflows/ci.yml` — Node + pnpm pins.
- CLAUDE.md "Code quality and testing discipline" — testing rules including the three-consecutive-runs convention for known-flaky tests.
