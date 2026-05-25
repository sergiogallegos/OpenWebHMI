---
id: CODEX-AU
title: ci: install Tauri Linux build deps so the Rust job's clippy step succeeds
owner: codex
phase: 4
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

# CODEX-AU — ci: install Tauri Linux build deps

## Brief

> The `Rust` CI job has been failing for at least 5 consecutive pushes because `cargo clippy --workspace --all-targets` recursively builds the Tauri designer, which needs `glib-2.0`, `gtk-3`, `webkit2gtk-4.1`, and `libsoup-3` system libraries that the GitHub-hosted Ubuntu runner doesn't preinstall. Add one `apt-get install` step before the clippy + test steps so the build can find the libraries. One-line workflow change.

### Goal

`cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` complete on the Ubuntu CI runner. The Rust job goes green for the first time in 5+ pushes.

### Context to read first

- `.github/workflows/ci.yml` — the workflow file. Failing step is the `Rust` job's `cargo clippy` invocation.
- Last failure log (one-liner): `The system library 'glib-2.0' required by crate 'glib-sys' was not found.` — `glib-sys` is pulled in transitively via the Tauri designer (`apps/designer/src-tauri/`).
- Tauri's Linux prerequisites: https://v2.tauri.app/start/prerequisites/#linux — canonical list of system libraries needed to build a Tauri app on Linux. The current pinned Tauri version dictates which exact packages are needed; check `apps/designer/src-tauri/Cargo.toml` for the `tauri` version pin and cross-reference the Tauri docs for that version's Linux deps.
- [`docs/agents/notes/toolchain-drift.md`](../notes/toolchain-drift.md) — environment-mismatch discipline.

### Files to create / modify

- **Modify** `.github/workflows/ci.yml` — add a `Install Tauri Linux build deps` step to the `rust` job, placed **after** `actions/checkout@v4` and **before** the `cargo` cache action. (Caching the deps install isn't necessary; `apt-get install` is ~10 seconds on the runner.)

### Behavior

The new step runs:

```yaml
- name: Install Tauri Linux build deps
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libglib2.0-dev \
      libgtk-3-dev \
      libwebkit2gtk-4.1-dev \
      libsoup-3.0-dev \
      pkg-config
```

The exact package list mirrors Tauri's v2 Linux prerequisites for the pinned Tauri version. If Tauri 1.x is still in use (unlikely — check `apps/designer/src-tauri/Cargo.toml`), substitute `libwebkit2gtk-4.0-dev` for `4.1-dev` and `libsoup2.4-dev` for `libsoup-3.0-dev`.

### Test requirements

The verification IS the CI run. After the change:

1. Push to a branch (or to `main` per OpenWebHMI's no-PR convention).
2. The `Rust` job runs `cargo clippy` and `cargo test` to completion (pass or fail on actual code findings — but no longer fails at the build-script step for `glib-sys`).

No new local tests; the workflow change is verified by CI itself. Document the run url in the Codex log.

### Acceptance criteria

- [ ] `.github/workflows/ci.yml` `rust` job has the install step.
- [ ] Package list matches the pinned Tauri version's Linux prerequisites (cite the Tauri version in the Codex log).
- [ ] The `Rust` CI job on the resulting push reaches `cargo test` (passes or fails on real findings, not at the build-script step).
- [ ] Job duration noted in the Codex log (sanity-check vs the pre-fix ~1-minute failure time).

### Out of scope

- Caching the apt install. ~10s install time doesn't justify the cache complexity.
- Touching the Tauri version itself.
- Fixing any *new* clippy/test failures that surface after the build-script step starts working. Those are real findings and get their own follow-up.
- Linux build deps for runtimes that don't currently target Linux (e.g. Windows-only drivers). Only the deps needed to make `cargo clippy --workspace --all-targets` succeed.
- The `Node` CI job (covered by CODEX-AV) and the `validate-agent-files` job (already green).

### Risks / gotchas

- **Tauri version drift.** If `apps/designer/src-tauri/Cargo.toml`'s `tauri` pin changes later, the apt package list may need to change too. Add a one-line comment in `ci.yml` above the install step: `# Tauri vN Linux prereqs — update when src-tauri Cargo.toml's tauri pin changes`.
- **`libwebkit2gtk-4.1-dev` availability.** Ubuntu 22.04 ships `4.0`; 24.04 ships `4.1`. Check what `ubuntu-latest` currently resolves to (24.04 as of 2026-05) and pick the matching dep. If the runner is still 22.04, use `4.0-dev`.
- **Running clippy may surface real warnings now.** Once `glib-sys` builds, clippy will lint the Tauri-touching code. If those lints fire, that's CODEX-AU finding new work, not failing — open a separate follow-up brief. Don't expand AU to fix downstream lint findings.
- **`sudo` on GitHub runners.** Standard, supported, no surprises.
- **The validator pre-commit hook is unaffected** — workflow-only change; `docs/agents/` not touched.

## Codex log

## Claude review

## Verdict
