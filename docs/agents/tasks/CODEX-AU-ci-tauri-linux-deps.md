---
id: CODEX-AU
title: ci: install Tauri Linux build deps so the Rust job's clippy step succeeds
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
merge-commit: 4e9bc9b
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

### 2026-05-25 22:31 codex [gpt-5]
Status -> in-progress. Codex started the CI hygiene task after maintainer direction to handle AU and AV first.

### 2026-05-25 22:31 codex [gpt-5]
Status -> submitted. Added the `Install Tauri Linux build deps` step to the Rust CI job immediately after checkout and before Rust/cache setup. The local Tauri pin is `tauri = "2"` / `tauri-build = "2"` in `apps/designer/src-tauri/Cargo.toml`; the package list uses the Tauri v2 Debian/Ubuntu WebKitGTK 4.1 prerequisite set for the current `ubuntu-latest` runner family, with explicit `glib`, `gtk`, `libsoup`, and `pkg-config` packages for the failing build scripts. Local verification: `scripts/validate-agent-files` passes. CI verification, run URL, and job duration remain pending until the change is pushed and GitHub Actions runs.

### 2026-05-25 23:07 codex [gpt-5]
CI verification for run https://github.com/sergiogallegos/OpenWebHMI/actions/runs/26423144512: the Rust job reached and passed `cargo clippy --workspace --all-targets -- -D warnings`; the install step completed in 23 seconds and the Rust job ran for 5m40s total. The job then failed downstream in `cargo test --workspace --locked` on `crates/gateway/tests/integration.rs` test `websocket_gateway_forwards_script_events_by_project`, which timed out waiting for a message. That is beyond AU's system-dependency unblock scope.

## Claude review

### 2026-05-25 23:20  claude [Opus 4.7]

**Independent verification**
- `gh run view 26423144512` — confirmed the Rust job's `Install Tauri Linux build deps` step passed; install took 23s of the 5m40s total job time.
- Same run: `cargo fmt --all -- --check` passed; `cargo clippy --workspace --all-targets -- -D warnings` **passed** (this was the failing gate before AU).
- `git show 4e9bc9b -- .github/workflows/ci.yml` — diff is exactly the install step from the brief, placed after `actions/checkout@v4` and before the rustup/cache actions, with the comment the brief asked for (`# Tauri v2 Linux prereqs - update when src-tauri Cargo.toml's tauri pin changes.`).

**What's being fixed**
- The Rust CI job's `cargo clippy --workspace --all-targets` was failing at the `glib-sys` build script because the Ubuntu runner lacks the Tauri designer's Linux system libraries.

**Root cause confirmation**
- Confirmed: `glib-sys` is pulled in transitively from `apps/designer/src-tauri/`; the missing libraries (`glib-2.0`, `gtk-3`, `webkit2gtk-4.1`, `libsoup-3`, `pkg-config`) are not preinstalled on `ubuntu-latest`. Fix is environment-side, not code-side.

**Fix appropriateness**
- Lands at the right layer (workflow only; no `Cargo.toml` / `src-tauri/` changes). Step placement is correct: after checkout (needs the repo to know `src-tauri` exists), before cache (so the cache key isn't invalidated by missing libs).
- The Tauri version pin was correctly cited by Codex (`tauri = "2"` / `tauri-build = "2"`) and the package list matches Tauri v2's documented Linux prereqs for the Ubuntu 24.04 runner family.

**Test proof**
- The brief said the verification IS the CI run; CI run 26423144512 is the proof. Install step passed; clippy passed; reached `cargo test` (which fails downstream — see Residual risk).
- No new local tests; workflow-only change.

**Residual risk**
- `cargo test --workspace --locked` now fails downstream at `crates/gateway/tests/integration.rs::websocket_gateway_forwards_script_events_by_project` (timeout waiting for a message). This is **not** an AU regression — it was masked by the earlier `glib-sys` build failure. Tracked as CODEX-AW (gateway WS integration test timeout).
- Tauri version drift: if `apps/designer/src-tauri/Cargo.toml`'s `tauri` pin moves to v3+ later, the apt package list will need to change too. The inline comment in `ci.yml` flags this. No automated guard.
- Ubuntu 24.04 ships `libwebkit2gtk-4.1-dev`; if the GitHub `ubuntu-latest` ever rolls back to 22.04, the package name needs to be `4.0-dev`. Low likelihood.

**Strong points (✅)**
- Minimal, surgical change — exactly the one-line-class fix the brief specified, no scope creep.
- Inline comment in `ci.yml:18` matches the brief's risk note about Tauri version drift verbatim — future maintainers see the dependency contract.
- Codex correctly stopped at the scope boundary when the downstream `cargo test` failure surfaced. Reported it honestly with the test name and the run url, then explicitly noted "beyond AU's system-dependency unblock scope." Per CLAUDE.md "Honesty" rule, that's the bar.
- Tauri version was cited specifically (`tauri = "2"` / `tauri-build = "2"`) before picking the package list — answered the brief's "pick the right version's prereqs" instruction by checking the pin first.

**Findings**
- 🟢 The `Install Tauri Linux build deps` step took 23s on the runner; the brief estimated ~10s. Within the same order of magnitude, no action needed; documenting in case future cost-optimization care is wanted.
- 🟡 Workflow comment uses ASCII hyphens; the brief sample used em-dashes. Cosmetic only.
- 🟠 Real concerns — none. (The downstream `cargo test` failure is real but explicitly out of AU's scope, tracked as CODEX-AW.)
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `.github/workflows/ci.yml` `rust` job has the install step.
- ✅ Package list matches Tauri v2 Linux prerequisites; Tauri version cited in Codex log.
- ✅ Rust CI job reaches `cargo test` (passes/fails on real findings, not at build-script).
- ✅ Job duration noted (5m40s total; install 23s).

## Verdict

**Merged** at `4e9bc9b` (the implementation commit bundles AU + AV; bookkeeping in this commit).

What's NOT yet proven by this merge:
- `cargo test --workspace --locked` is still red — the gateway WS integration test timeout was *exposed* by AU but is not in AU's scope. Tracked as CODEX-AW.

Follow-ups opened: CODEX-AW (`tests: stabilize gateway WS script-event integration test`).
