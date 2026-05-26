---
id: CODEX-AZ
title: ci: AppImage bundling fails with 'failed to run linuxdeploy' (apps/designer tauri build)
owner: codex
phase: 4
status: open
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
---

# CODEX-AZ — ci: AppImage bundling fails with 'failed to run linuxdeploy'

## Brief

> CODEX-AY unblocked the Node job past `glib-sys`, but `apps/designer`'s `tauri build` now fails downstream at AppImage bundling with `failed to bundle project 'failed to run linuxdeploy'`. The Tauri release binary, `.deb`, and `.rpm` all build successfully — only the `.AppImage` artifact fails. Pick a scope-bounded fix and document the choice. **Follow-up surfaced by AY merge CI run.**

### Goal

`pnpm -r --if-present build` reaches exit code 0 on the Node CI job, going fully green for the first time. Two acceptable fix paths — Codex picks one and documents the choice in the Codex log:

- **Path A — Install `linuxdeploy` + AppImage prereqs on the runner** so the existing Tauri AppImage bundler runs successfully. Keeps the AppImage artifact available from CI runs.
- **Path B — Exclude AppImage from the CI bundle targets**, keeping `deb` + `rpm` + the raw release binary. AppImage stays available for release builds (outside CI) if a release workflow needs it.

Strong recommendation: **Path B** for CI hygiene. AppImage on CI is value-marginal — `.deb` + `.rpm` cover the Linux distribution channels most integrators ship through, and AppImage bundling on headless CI runners is historically fragile (FUSE requirements, `linuxdeploy` self-extracting binaries needing `--appimage-extract-and-run`, etc.). Path A keeps the artifact at the cost of CI complexity.

### Context to read first

- `.github/workflows/ci.yml` — Node job's `pnpm -r --if-present build` step. AY's install block precedes it.
- `apps/designer/src-tauri/tauri.conf.json` — the Tauri config. The `bundle.targets` field (if present) lists which formats `tauri build` produces; default is `"all"` which includes AppImage on Linux.
- `apps/designer/package.json` — confirm `build` script is `vite build && tauri build` (or similar); if the brief's Path B is taken, the change might be to pass `--bundles deb,rpm` to the `tauri build` invocation.
- Failing CI run: https://github.com/sergiogallegos/OpenWebHMI/actions/runs/26431030607 — Node job, `pnpm -r --if-present build` step. The failure line is `apps/designer build: failed to bundle project 'failed to run linuxdeploy'`. The .deb + .rpm + raw release binary all built successfully before AppImage failed.
- CODEX-AY task file for the AU/AY pattern of "fix obvious blocker → next layer surfaces → narrow follow-up brief."
- Tauri AppImage bundling notes: https://v2.tauri.app/distribute/appimage/ — canonical reference for AppImage prerequisites and `--bundles` flag syntax.

### Files to create / modify

**If Path A (install linuxdeploy + prereqs):**

- **Modify** `.github/workflows/ci.yml` — add a step after the existing Tauri deps install that:
  - Downloads the `linuxdeploy` AppImage release binary (e.g. via `wget`).
  - `chmod +x` it and either install to `/usr/local/bin` OR set `--appimage-extract-and-run` env so it works without FUSE.
  - Confirms FUSE is installed (`sudo apt-get install -y libfuse2`) — needed for AppImage extraction on Ubuntu 22.04+; the runner may already have it.

**If Path B (exclude AppImage from CI bundle):**

- **Modify** `apps/designer/package.json` — change the `build` script to `vite build && tauri build --bundles deb,rpm` (or similar; verify Tauri v2 syntax).
- **OR Modify** `apps/designer/src-tauri/tauri.conf.json` — set `bundle.targets = ["deb", "rpm"]` for the CI-relevant build path.
- **OR Modify** `.github/workflows/ci.yml` — change the Node job's `pnpm -r --if-present build` invocation to skip AppImage targets via a workspace-level flag if pnpm supports it.

Decide *one* approach for Path B and apply it minimally.

### Behavior

After the fix, `pnpm -r --if-present build` exits 0. The Node CI job goes fully green.

- Path A: `.AppImage` artifact appears in `apps/designer/src-tauri/target/release/bundle/appimage/`.
- Path B: `.AppImage` artifact does NOT appear in CI runs, but `.deb` and `.rpm` still do.

### Test requirements

The verification IS the CI run.

1. Push to `main`.
2. The Node job's `pnpm -r --if-present build` step exits 0.
3. Document the run url and the chosen path in the Codex log.

If Path B: optionally verify a local `tauri build` (no `--bundles` flag) still produces an AppImage outside CI, confirming the artifact is available for release builds.

### Acceptance criteria

- [ ] `.github/workflows/ci.yml` Node job's `pnpm -r --if-present build` step exits 0 on a `main` push.
- [ ] Codex log names which Path was taken (A or B) and why (defaults to B per the brief's recommendation; A acceptable with a stated reason).
- [ ] If Path B: a one-line comment or doc note explains where the AppImage artifact still ships from (release-only workflow, manual local build, or "deferred until requested").
- [ ] No regression to .deb / .rpm / raw release binary outputs.

### Out of scope

- Adding a separate release workflow that produces AppImage. If Path B is taken, defer release-only AppImage to a future brief unless a customer asks.
- Bumping the Tauri version.
- Migrating from AppImage to alternatives (Flatpak, Snap, .pkg, etc.).
- Fixing any *new* failures that surface after `pnpm -r --if-present build` exits 0 — separate follow-ups.
- The Node 20 deprecation warning.
- DRY-ing the Tauri install steps across rust/node jobs (still 2 sites; wait per AY's "wait for a 3rd consumer" rule).

### Risks / gotchas

- **`linuxdeploy` FUSE requirement.** Path A on Ubuntu 22.04+ may need `libfuse2` installed; FUSE 3 doesn't satisfy `linuxdeploy`'s self-extracting AppImage. The `--appimage-extract-and-run` flag works around this and is the cleaner CI option if Path A is chosen.
- **`tauri build --bundles` flag syntax** is Tauri-version-specific. Verify against the pinned Tauri version (`apps/designer/src-tauri/Cargo.toml`) — Tauri v2 supports `--bundles deb,rpm`; older v1 used `--bundle` (singular).
- **Don't break local developer experience.** If Path B modifies `package.json`'s `build` script, local developers running `pnpm --filter @openwebhmi/designer build` should still get all bundle targets (AppImage included) by default for their personal workflow — the CI-only restriction belongs in the workflow file, not in the package.json shared script. Prefer modifying `.github/workflows/ci.yml` to pass the `--bundles` flag as a CI-only override.
- **Honest scope discipline (AU/AV/AY pattern).** When the AppImage bundle starts working, the next downstream failure may surface (e.g. signing, metadata, AppImage version checks). Stop at AZ's boundary and open a new follow-up.
- **The deprecated `actions/checkout@v4` Node 20 warning** is unrelated and stays out of scope (separate brief when v5 ships).

## Codex log

## Claude review

## Verdict
