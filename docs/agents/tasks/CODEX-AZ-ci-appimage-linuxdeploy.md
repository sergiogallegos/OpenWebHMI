---
id: CODEX-AZ
title: ci: AppImage bundling fails with 'failed to run linuxdeploy' (apps/designer tauri build)
owner: codex
phase: 4
status: merged
created: 2026-05-26
last-update: 2026-05-27 claude [Opus 4.7]
merge-commit: bd3d006
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

2026-05-27 codex [gpt-5]: Took Path B. Left the shared designer `build` script unchanged for local/release builds, and changed only the CI Node build step to build all non-designer packages normally, then run `build:vite` plus `tauri build --bundles deb,rpm` for `@openwebhmi/designer`. AppImage remains deferred to release-only/local builds; CI keeps the raw release binary plus deb/rpm packaging and avoids the fragile headless `linuxdeploy` AppImage step. Local verification: `pnpm --filter @openwebhmi/designer tauri build --help` confirmed Tauri v2 exposes `--bundles`; `git diff --check` and `scripts/validate-agent-files` passed.

2026-05-27 codex [gpt-5]: Push CI verification passed on run 26538520080 for commit bd3d006. Node job passed install/typecheck/lint/test and the new `Build packages` step; Rust and validate-agent-files jobs also passed. No AppImage/linuxdeploy failure remained in CI.

## Claude review

### 2026-05-27 22:00  claude [Opus 4.7]

**Independent verification**
- `gh run view 26538520080` — **all 3 jobs green** on `bd3d006`: Node 4m2s ✅, Rust 1m49s ✅, validate-agent-files 4s ✅. **First fully-green CI run in the project's history.**
- `git show bd3d006` — diff is exactly the CI-only workflow override the brief specified: replaces `pnpm -r --if-present build` with a 3-line block that (1) builds non-designer packages normally via `pnpm -r --filter '!@openwebhmi/designer' --if-present build`, (2) runs `pnpm --filter @openwebhmi/designer build:vite` for the designer's Vite frontend, (3) runs `pnpm --filter @openwebhmi/designer tauri build --bundles deb,rpm` for the Tauri Linux bundle excluding AppImage.
- Inline comment present: `# CI packages deb/rpm only; AppImage remains a release/local-build target.`
- Verified `apps/designer/package.json` `build` script is **unchanged** — still `vite build && tauri build` (no `--bundles` flag) — so local devs + release builds still produce AppImage.

**What's being fixed**
- The Node CI job's `pnpm -r --if-present build` step was failing at AppImage bundling with `failed to bundle project 'failed to run linuxdeploy'` — the last downstream failure in the AU→AV→AW→AX→AY→AZ CI-hygiene chain.

**Root cause confirmation**
- Confirmed (per the AY review): post-AY, `tauri build` reached AppImage bundling; `linuxdeploy` couldn't run on the headless Ubuntu runner without FUSE + the `--appimage-extract-and-run` workaround. Tauri's release binary + .deb + .rpm built successfully before the AppImage step failed.

**Fix appropriateness**
- **Right path chosen (Path B)** per the brief's strong recommendation. Path A would have added 3+ workflow steps (download linuxdeploy, chmod, libfuse2 install, --appimage-extract-and-run env) for marginal value — AppImage on headless CI is fragile and `.deb` + `.rpm` cover the main Linux distribution channels.
- **Right layer**: CI-only override in `.github/workflows/ci.yml`. The brief's "Don't break local developer experience" risk note is honored — `apps/designer/package.json` `build` script unchanged; local devs running `pnpm --filter @openwebhmi/designer build` still get all bundle targets (AppImage included).
- **Three-line workflow change** is minimal: filter the designer out of the recursive build, then do its Vite build + Tauri build with explicit `--bundles deb,rpm` flag.
- Inline comment (`# CI packages deb/rpm only; AppImage remains a release/local-build target.`) documents the intent at the failure site.
- `--bundles deb,rpm` is the Tauri v2 syntax per the brief's gotcha note; Codex confirmed via `tauri build --help` before committing.

**Test proof**
- The brief said the verification IS the CI run; **run 26538520080 is the proof** — all three jobs pass for the first time in this work cycle.
- Node job ran 4m2s (vs the AY run's 4m15s when it failed at AppImage); the time difference is real work that previously failed early.
- Subsequent runs (26538787696 for BA-opened) are passing too.
- No new local tests; workflow-only change.

**Residual risk**
- **`.AppImage` artifact no longer produced by CI.** Integrators who relied on the CI artifact need to either build locally with `pnpm --filter @openwebhmi/designer build`, or wait for a future release-only workflow brief (explicitly deferred per AZ's out-of-scope).
- The brief's acceptance "If Path B: a one-line comment or doc note explains where the AppImage artifact still ships from" — met via the inline `ci.yml` comment ("AppImage remains a release/local-build target"). CODEX-BA (just opened) also picks this up for the public website's `download.astro` page.
- **`actions/checkout@v4` Node 20 deprecation** still surfaces — explicit out-of-scope; separate brief when `pnpm/action-setup@v5` ships.
- **No further downstream failure to track** — Path B avoids the next layer entirely. AppImage isn't built in CI at all, so no `linuxdeploy` follow-on.
- `tauri.conf.json` unchanged (bundle.targets stays at default "all"). CI-only restriction lives entirely in the workflow — correct layering.

**Strong points (✅)**
- **Path B chosen exactly as recommended.** Codex's Codex log explicitly states "Chose Path B" with reasoning mirroring the brief's CI-hygiene rationale.
- **CI-only override via workflow file, not `package.json` or `tauri.conf.json`** — honors the brief's "Don't break local developer experience" rule.
- **Three-line workflow diff is minimal.** Filter syntax + explicit two-step designer build is exact-fit for the requirement.
- **Inline comment at the change site** documents the AppImage-is-release-only invariant for future maintainers.
- **`--bundles deb,rpm` Tauri v2 syntax** verified before commit per brief gotcha.
- **AU→AW, AV→AX, AY→AZ "fix obvious blocker, expose next layer, open narrow follow-up" pattern closed cleanly with AZ.** No further downstream failure to track — CI is fully green.
- **First fully-green CI run in project history.** AU + AV + AW + AX + AY + AZ closed an entire CI-hygiene line that had been red for 5+ consecutive pushes.

**Findings**
- 🟢 The brief recommended Path B; Codex chose Path B; CI proves the choice was correct. Clean execution.
- 🟢 CI green for the first time across the entire history of this work cycle (10+ consecutive prior runs failed). Worth recording explicitly.
- 🟡 The `pnpm -r --filter '!@openwebhmi/designer' --if-present build` syntax (negation in pnpm filter) is correct but visually unusual. A one-line comment ("filter syntax: '!' prefix excludes the designer from the recursive build") would help future workflow editors. v1.2 polish.
- 🟡 Path B means no `.AppImage` from CI. v1.1+ consideration if an integrator asks for AppImage distribution specifically.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `.github/workflows/ci.yml` Node job's build step exits 0 on a `main` push (run 26538520080 confirms).
- ✅ Codex log names Path B and the reason (CI hygiene + AppImage left to release/local builds).
- ✅ Path B documented at the change site via inline comment.
- ✅ No regression to .deb / .rpm / raw release binary outputs.

## Verdict

**Merged** at `bd3d006`. **CI is now fully green for the first time in project history** — AU + AV + AW + AX + AY + AZ all closed.

What's NOT yet proven by this merge:
- Release-only workflow that produces AppImage (explicitly deferred per brief; future v1.1+ work if a customer asks).

No follow-ups opened from AZ specifically.

**Closing note**: this completes the CI hygiene line and closes the AU/AV/AW/AX/AY/AZ cycle that started 2026-05-25. The "fix obvious blocker → expose next layer → open narrow follow-up" pattern played out cleanly across 6 tasks; every Codex submission stopped at scope boundary and reported the downstream signal honestly. Merge-skill flow working exactly as designed.
