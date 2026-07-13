---
id: CODEX-CK
title: CI hardening — multi-platform matrix, --all-features alignment, supply-chain gate, release workflow
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CK — CI hardening

## Brief

> `.github/workflows/ci.yml` has gaps that undercut the project's own commitments. It builds and tests on a single platform (`ubuntu-latest`) though README.md and VISION.md commit to Linux + macOS + Windows for the gateway and Win + Mac for the designer. Its Rust job runs `clippy --workspace --all-targets` and `test --workspace --locked` **without `--all-features`** (and clippy without `--locked`), so feature-gated driver paths are unlinted and untested — a mismatch with the CLAUDE.md test matrix. There is no supply-chain gate (`cargo-deny`/`cargo-audit`) despite VISION.md making MIT-compatible-license enforcement a hard merge contract, no release/tag workflow despite the roadmap's tagged-1.0.0 exit criterion, and no Dependabot config despite the strict `=`-pin policy that would benefit from grouped precise bumps. Close these gaps **incrementally and without regressing the just-turned-green CI** — expand platforms carefully (allow-failure first if needed), align the feature flags, add a `cargo-deny` license+advisory gate, and add a minimal tag→artifacts release workflow.

### Goal

CI builds and tests the Rust workspace on Linux, macOS, and Windows; the clippy and test invocations match CLAUDE.md's `--all-features --locked` matrix; a `cargo-deny` job enforces the MIT-compatible-license contract and advisory checks and is green; a minimal release workflow builds tagged artifacts; and Dependabot is configured to propose grouped, precise dependency bumps consistent with the `=`-pin policy. The existing green jobs stay green.

### Context to read first

- `.github/workflows/ci.yml` — the current file. Key lines:
  - `ci.yml:13-34` — the `rust` job: single `runs-on: ubuntu-latest`, `cargo clippy --workspace --all-targets -- -D warnings` (line ~33, **no `--all-features`, no `--locked`**), `cargo test --workspace --locked` (line ~34, **no `--all-features`**).
  - `ci.yml:36-65` — the `node` job: Tauri Linux prereqs, `pnpm` install/typecheck/lint/test, and the `Build packages` step that runs `tauri build --bundles deb,rpm` (line ~65). Note the comment "CI packages deb/rpm only; AppImage remains a release/local-build target" (line ~60) — **honor this; do not reintroduce AppImage/linuxdeploy in CI**.
  - `ci.yml:67-77` — the `validate-agent-files` job. Leave it alone.
- [`CLAUDE.md`](../../../CLAUDE.md) §"Review and merge lifecycle" — the canonical test matrix: `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `pnpm -r typecheck`, `pnpm -r test`. CI must mirror this.
- [`CLAUDE.md`](../../../CLAUDE.md) §"Dependency management" and §"No `--release` builds during development" — precise bumps only, bounded `Cargo.lock`, debug builds in the dev matrix. The release workflow is the one place `--release` is appropriate (shipping artifacts), not the dev matrix.
- [`VISION.md`](../../../VISION.md) §"License & dependency hygiene" (~34-39) — **no non-MIT-compatible dependencies** (MIT / Apache-2.0 / BSD / compatible only; no GPL/AGPL/SSPL); the `=`-pinned set (`tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`) stays pinned; `cargo update` (no args) never lands. This is the contract `cargo-deny` must encode.
- `README.md:33-35` — the platform commitment (Linux + macOS + Windows gateway; designer Win + Mac). `docs/roadmap.md` Phase 4 — the tagged-1.0.0 exit criterion the release workflow serves.
- `scripts/cross-compile-pi.sh` — existing Raspberry Pi cross-compile script; the release workflow can reference/reuse the aarch64 target notes but need not run Pi builds in CI.
- **The CI hygiene history — read before touching platforms** (these tasks are why CI is only now green; do not undo their decisions):
  - `docs/agents/tasks/CODEX-AU-*.md` through `CODEX-AZ-*.md` — the CI stabilization arc. CODEX-AZ specifically records the AppImage/linuxdeploy failure and the deb/rpm-only decision. Read AZ's verdict before changing the `node` job's bundling.
- The `Swatinem/rust-cache@v2` and `dtolnay/rust-toolchain@stable` actions already in use — reuse them; `rust-toolchain@stable` honors `rust-toolchain.toml` (currently pinned per CODEX-BB).

### Files to create / modify

1. **`.github/workflows/ci.yml` — Rust job feature alignment (do this first; lowest risk):**
   - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
   - `cargo test --workspace --all-features --locked`.
   - Keep `cargo fmt --all -- --check`. This alone closes the CLAUDE.md mismatch and may surface previously-unlinted feature-gated code — fix or `#[expect]` per CLAUDE.md, or flag in the Codex log if it needs a separate task.

2. **`.github/workflows/ci.yml` — multi-platform matrix (do this carefully, incrementally):**
   - Convert the `rust` job to a `strategy.matrix.os: [ubuntu-latest, macos-latest, windows-latest]` running at least `cargo build --workspace --all-features --locked` and `cargo test --workspace --all-features --locked`.
   - The Tauri Linux prereqs (`ci.yml:18-27`) are Linux-only — gate that step to `runs-on == ubuntu-latest` (`if:` condition) so macOS/Windows don't fail on `apt-get`.
   - **Land platform expansion behind `continue-on-error: true` (allow-failure) first if macOS/Windows reveal unknown breakage**, so the merge doesn't regress the green Linux gate. Record in the Codex log which platforms are required-green vs. allow-failure at merge time, and open a follow-up note for any platform left allow-failure.
   - The `node`/designer Tauri bundling stays Linux-only deb/rpm in CI (per CODEX-AZ). Do **not** add macOS/Windows `tauri build --bundles` to CI in this task — matrix expansion here is about the Rust workspace build+test, not desktop packaging.

3. **`.github/workflows/ci.yml` (or a separate `.github/workflows/supply-chain.yml`) — supply-chain gate:**
   - A `cargo-deny` job (use `EmbarkStudios/cargo-deny-action` or install the binary) running `licenses` + `advisories` (+ `bans`/`sources` as appropriate).
   - Must be green at merge — this makes the VISION.md license contract enforced-in-CI instead of manual.

4. **`deny.toml`** (new, repo root):
   - `[licenses]` allowlist: MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016/Unicode-3.0, Zlib, and any other permissive licenses actually present in the current tree — **derive the list from `cargo deny check licenses` output against the real lockfile, don't guess**. Deny GPL/AGPL/SSPL/copyleft. Add narrowly-scoped, commented per-crate exceptions only where a genuinely permissive crate carries a nonstandard SPDX expression; document each exception's justification inline.
   - `[advisories]`: deny known vulnerabilities; use `ignore` with a comment + rationale only where a pinned (`=`) dependency can't yet move and the advisory is assessed non-exploitable in this usage — tie any such ignore to the `=`-pin discipline.
   - `[bans]`: at minimum flag multiple-versions where cheap; don't over-constrain.

5. **`.github/workflows/release.yml`** (new): minimal `on: push: tags: ['v*']` workflow that builds release artifacts (gateway binary per platform at least; designer bundles optional/Linux-first) and attaches them to a GitHub Release. `--release` is appropriate here (shipping artifacts). Keep it minimal — a working skeleton that produces the gateway binary for the three platforms is sufficient; full signed/notarized designer bundles can be a later task (note the deferral).

6. **`.github/dependabot.yml`** (new): cargo + npm (+ github-actions) ecosystems, **grouped** updates, weekly cadence. Configure so it proposes bumps that fit the `=`-pin policy — do not let it mass-bump the `=`-pinned set; group/patch-limit as needed and document the intent in a comment. Dependabot proposes; the merge discipline (precise bumps, bounded lockfile) still applies at review.

### Behavior

- Every push/PR: Rust build+test on Linux, macOS, Windows (required-green where stable, allow-failure only where a platform is newly flaky and tracked); clippy+test use `--all-features --locked`; `cargo-deny` gate green; `node` job unchanged (deb/rpm Linux bundling); `validate-agent-files` unchanged.
- Pushing a `v*` tag: the release workflow builds artifacts and creates/updates a GitHub Release.
- Dependabot opens grouped dependency PRs on schedule without violating the `=`-pin policy.
- **The current green Linux gate never regresses.** If a change would turn a required job red without a corresponding real fix, it lands allow-failure with a tracking note instead.

### Test requirements

- CI is validated by running on a branch/PR — this task's "tests" are the workflow runs themselves. In the Codex log, record:
  - The `cargo-deny` result (paste the license summary; confirm no GPL/AGPL/SSPL and that `deny.toml`'s allowlist was derived from the real tree, not guessed).
  - The per-platform Rust job results (which platforms are green, which are allow-failure and why).
  - Whether `--all-features` clippy surfaced new lints and how each was addressed.
  - A `git tag` dry-run or a test tag on a fork/branch showing the release workflow produces artifacts (or, if a real tag isn't run, say so explicitly — no "I tested it" without running it).
- Locally, before pushing: `cargo deny check` passes against the current lockfile; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` and `cargo test --workspace --all-features --locked` pass on the dev machine (at least the host platform).
- No secrets committed. Release workflow uses `GITHUB_TOKEN`/`secrets`, not hardcoded credentials.

### Acceptance criteria

- [ ] Rust CI builds and tests on `ubuntu-latest`, `macos-latest`, `windows-latest` (required-green where stable; any allow-failure platform tracked with a follow-up note).
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` and `cargo test --workspace --all-features --locked` in CI — matches CLAUDE.md.
- [ ] A `cargo-deny` job runs licenses + advisories and is green; `deny.toml` exists with an allowlist derived from the real tree, denies GPL/AGPL/SSPL, and documents any exception/ignore inline.
- [ ] `.github/workflows/release.yml` builds tagged (`v*`) artifacts and attaches them to a GitHub Release (gateway binary across the three platforms at minimum).
- [ ] `.github/dependabot.yml` configures grouped cargo + npm (+ actions) updates consistent with the `=`-pin policy.
- [ ] The `node` job's Linux deb/rpm bundling and the `validate-agent-files` job are unchanged; no AppImage/linuxdeploy reintroduced.
- [ ] Existing green jobs do not regress; the Codex log records the pre/post job status for each.
- [ ] Codex log documents the `cargo-deny` license summary, per-platform results, any new `--all-features` clippy findings, and the release-workflow verification (run or explicitly deferred).

### Out of scope

- **Desktop bundle packaging expansion** (macOS `.dmg`, Windows `.msi`, signing/notarization) in the release workflow — a working gateway-binary release skeleton is enough; note bundle signing as a follow-up.
- **AppImage / linuxdeploy in CI** — explicitly excluded per CODEX-AZ. Do not reintroduce.
- **Code coverage tooling** (`cargo-llvm-cov`, Codecov) — desirable but a separate task; mention as a follow-up if convenient, don't build it here.
- **Bumping any dependency** as part of wiring Dependabot — Dependabot *proposes*; this task configures it, it does not land bumps. If `--all-features` clippy surfaces a fix that needs a dep bump, flag it for a separate brief.
- **Changing the `=`-pinned set** or `rust-toolchain.toml` — pins stay; toolchain bumps are their own briefs (see CODEX-BB).
- **Raspberry Pi / aarch64 builds in the main CI matrix** — the cross-compile script exists for release/local use; adding a Pi job to every-PR CI is out of scope (reference it in release notes at most).
- **A full OpenMetrics/observability CI** — unrelated (see CODEX-CJ).

### Risks / gotchas

- **Don't regress the just-green CI.** CODEX-AU..AZ spent real effort getting to green. Platform expansion is the highest-risk change — land it allow-failure first if macOS/Windows reveal breakage, and only promote to required-green once a run confirms it. A red required job on `main` is worse than a missing platform.
- **Windows/macOS surprises.** Path separators, line endings, `bundled` SQLite (rusqlite) C-toolchain availability, and driver crates' system deps can differ. `--all-features` on Windows may pull driver code paths that assume POSIX. Expect the first cross-platform run to surface something; triage rather than force-green.
- **`--all-features` may surface new clippy lints** that the single-feature Linux job never exercised (feature-gated driver modules). Fix or `#[expect(..., reason = "...")]` per CLAUDE.md — do not blanket-`#[allow]`.
- **`deny.toml` allowlist must be derived, not guessed.** Run `cargo deny check licenses` against the actual lockfile and allowlist exactly the permissive licenses present. A too-broad allowlist defeats the VISION.md contract; a too-narrow one turns CI red on a legitimate permissive crate. Every entry and every `ignore` gets a comment.
- **Dependabot vs. `=`-pins.** Default Dependabot would try to bump the load-bearing `=`-pinned crates and blow the bounded-lockfile rule. Group/ignore the pinned set so its PRs stay reviewable and the pins stay honored; document the config intent inline.
- **Release workflow `--release` is the exception, not a license to use release builds elsewhere.** The dev matrix stays debug per CLAUDE.md; only the artifact-shipping workflow builds `--release`.
- **Honesty in the log.** If the release workflow wasn't actually triggered by a real tag, say "release workflow authored but not yet run against a real tag" — don't claim it produces artifacts on faith. Same for any allow-failure platform: name it.
- **`cargo-deny` advisory DB is network-fetched.** That's fine in CI (not the air-gapped gateway runtime) — VISION.md's "no third-party calls" is about the *product at runtime*, not the CI pipeline. Don't confuse the two.

## Codex log

## Claude review

## Verdict
