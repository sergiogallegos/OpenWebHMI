---
id: CODEX-AY
title: ci: install Tauri Linux build deps in the Node job too (pnpm build step runs tauri build)
owner: codex
phase: 4
status: merged
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: 7505a62
---

# CODEX-AY — ci: install Tauri Linux build deps in the Node job too

## Brief

> CODEX-AU added the Tauri v2 Linux deps install step to the `Rust` job, but the same `pnpm -r --if-present build` step in the `Node` job also runs `apps/designer`'s `tauri build`, which fails with the identical `glib-sys` / `pkg-config` error. Apply the same install step to the `Node` job. One-line workflow change (mirror of AU). **Follow-up surfaced by AQ + AR merge CI run.**

### Goal

`pnpm -r --if-present build` succeeds for the whole workspace on CI. The Node job goes green for the first time (typecheck + lint + test already pass post-AX; the only remaining red is `build`).

### Context to read first

- `.github/workflows/ci.yml` — the workflow file. Failing step is the `node` job's `pnpm -r --if-present build` step.
- CODEX-AU task file — sister fix for the Rust job; the apt install block is the same.
- Failing CI run: https://github.com/sergiogallegos/OpenWebHMI/actions/runs/26430352703 — Node job, `pnpm -r --if-present build` step. The failure is in `apps/designer build` running `vite build && tauri build`.

### Files to create / modify

- **Modify** `.github/workflows/ci.yml` — add an `Install Tauri Linux build deps` step to the `node` job, placed **after** `actions/checkout@v4` and **before** `pnpm install`. The step is byte-identical to the one already in the `rust` job.

### Behavior

The new step in the `node` job runs:

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

Same comment as in the rust job: `# Tauri v2 Linux prereqs - update when src-tauri Cargo.toml's tauri pin changes.`

### Test requirements

The verification IS the CI run. After the change:

1. Push to `main`.
2. The `Node` job runs `pnpm -r --if-present build` to completion (passes or fails on actual Tauri build errors, not at the `glib-sys` system-library step).

Document the run url in the Codex log.

### Acceptance criteria

- [ ] `.github/workflows/ci.yml` `node` job has the install step in the same shape as the `rust` job.
- [ ] The `Node` CI job on the resulting push reaches the end of `pnpm -r --if-present build` (passes or fails on real findings, not at the `glib-sys` system-library step).
- [ ] Codex log notes whether any downstream Node-build failures remain; if so, flagged as separate follow-ups.

### Out of scope

- Caching the apt install (same reasoning as AU; ~10s install).
- Refactoring the workflow to DRY the install step between jobs (composite action or matrix). Worth considering for v1.1 polish if more jobs need the install; not in this brief.
- Touching the Tauri version itself.
- Fixing any *new* tauri-build failures that surface after the system-library step starts working (separate follow-ups).
- The Node 20 deprecation warning.

### Risks / gotchas

- **Don't DRY prematurely.** Two jobs, two install steps, identical content. Composite action / reusable workflow is appealing but adds indirection for marginal benefit at 2 sites. Wait for a 3rd consumer.
- **Tauri version drift** — if `apps/designer/src-tauri/Cargo.toml`'s `tauri` pin changes, both install steps need updating. The inline comment AU added flags this in the rust job; do the same in the node job.
- **`ubuntu-latest` rollback** — if it ever rolls back to 22.04, `libwebkit2gtk-4.1-dev` becomes `4.0-dev` in both jobs. Same risk as AU.
- **The `vite build` part of `pnpm --filter @openwebhmi/designer build` already works** (typecheck + test passed); only the `tauri build` part needs the system deps.

## Codex log

2026-05-26 codex [gpt-5]: Mirrored the byte-identical Tauri v2 Linux prereq apt-install block from the Rust job into the Node job before `pnpm install`. Local verification: `git diff --check` and `scripts/validate-agent-files` passed.

2026-05-26 codex [gpt-5]: Pushed `7505a62`; CI run https://github.com/sergiogallegos/OpenWebHMI/actions/runs/26431030607 verified the Node job now installs Tauri deps, passes install/typecheck/lint/test, and reaches `pnpm -r --if-present build`. The previous `glib-sys` / pkg-config blocker is gone. Downstream failure remains in `apps/designer` Linux AppImage bundling after the Tauri release binary, deb, and rpm are built: `failed to bundle project \`failed to run linuxdeploy\``. That is outside AY's scope and should be a separate follow-up if AppImage bundling stays in CI.

## Claude review

### 2026-05-26 19:00  claude [Opus 4.7]

**Independent verification**
- `git show 7505a62 -- .github/workflows/ci.yml` — diff is byte-identical to the install step in the rust job, placed after `actions/checkout@v4` and before `pnpm/action-setup@v4`, with the same `# Tauri v2 Linux prereqs - update when src-tauri Cargo.toml's tauri pin changes.` comment AU established.
- `gh run view 26431030607` — Node job: ✅ Install Tauri Linux build deps, ✅ pnpm/action-setup@v4, ✅ actions/setup-node@v4, ✅ pnpm install --frozen-lockfile, ✅ pnpm -r --if-present typecheck, ✅ pnpm -r --if-present lint, ✅ pnpm -r --if-present test, ❌ pnpm -r --if-present build (fails at AppImage bundling — see Residual risk).
- `gh run view --job 77804158796 --log-failed` — confirmed failure is `apps/designer build: failed to bundle project 'failed to run linuxdeploy'`. The Tauri release binary + .deb + .rpm built successfully before the AppImage step failed.
- Rust job ✅ in 1m43s; validate-agent-files ✅ in 4s.

**What's being fixed**
- The Node CI job's `pnpm -r --if-present build` step was failing at the `glib-sys` system-library check because `apps/designer`'s `tauri build` needs the same Linux prereqs the Rust job got in CODEX-AU but the Node job never got.

**Root cause confirmation**
- Confirmed: pre-AY Node job had no apt install step, identical failure shape as the pre-AU Rust job. Sister fix.

**Fix appropriateness**
- Right layer (workflow only; no Cargo.toml or src-tauri changes). Mirror placement of the AU install step (after checkout, before next setup).
- Comment preserved verbatim from AU — future maintainers see the same Tauri-pin invariant flagged in both jobs.
- Codex correctly resisted the urge to DRY into a composite action at this stage (the brief's "Don't DRY prematurely" risk note was honored).

**Test proof**
- The brief said the verification IS the CI run; run 26431030607 is the proof. The Node job's previously-failing `glib-sys` gate is now passing; the build step proceeds through Tauri release binary + .deb + .rpm.
- No new local tests; workflow-only change.

**Residual risk**
- **AppImage bundling fails downstream with `failed to run linuxdeploy`.** This is **not** an AY regression — it was masked by the earlier `glib-sys` block. Tracked as CODEX-AZ. The .deb + .rpm + raw binary all build successfully, so the only artifact missing from a Linux release is AppImage. AZ has two scope options: install linuxdeploy + FUSE on the runner, OR exclude AppImage from the CI bundle targets (keep release-only).
- **Both install steps now duplicate** in `rust` and `node` jobs. Per the brief's "Wait for a 3rd consumer" rule, don't DRY yet. If a 3rd job needs the deps, refactor to a composite action then.

**Strong points (✅)**
- Surgical change, exactly the one-line-class fix the brief specified.
- Codex correctly stopped at the scope boundary when the AppImage failure surfaced. Reported it honestly with the exact error string (`failed to run linuxdeploy`), named what *did* build (.deb + .rpm + release binary), and proposed it as a separate follow-up. Per CLAUDE.md "Honesty" rule, that's the bar — same pattern as AU/AV stopping at their respective downstream boundaries.
- Inline comment matches the AU pattern verbatim — review can trust both jobs share the same Tauri-pin invariant note.

**Findings**
- 🟢 The Node job ran in 4m15s post-AY (vs 1m10s pre-AY when it failed fast at install). The extra time is real work the job is now doing — not regression overhead.
- 🟢 The pattern of "fix the obvious blocker → next layer surfaces → open follow-up" has now repeated 3 times (AU→AW, AV→AX, AY→AZ). The merge skill's flow is working as designed.
- 🟠 Real concerns — none. (AppImage failure is real but explicitly out of AY's scope, tracked as AZ.)
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `.github/workflows/ci.yml` Node job has the install step in the same shape as the Rust job.
- ✅ Node CI job reaches the end of `pnpm -r --if-present build` (passes Tauri binary/.deb/.rpm; fails at AppImage downstream).
- ✅ Codex log notes the downstream failure (`failed to run linuxdeploy`) and flags it as separate follow-up work.

## Verdict

**Merged** at `7505a62`.

What's NOT yet proven by this merge:
- AppImage bundle artifact (.AppImage) — fails at `linuxdeploy` invocation; tracked as CODEX-AZ. .deb + .rpm + raw binary all build successfully.
- Full Node CI job green (still red on the AppImage step).

Follow-ups opened: CODEX-AZ (`ci: AppImage bundling fails with 'failed to run linuxdeploy'`).
