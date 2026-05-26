---
id: CODEX-AY
title: ci: install Tauri Linux build deps in the Node job too (pnpm build step runs tauri build)
owner: codex
phase: 4
status: open
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
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

## Claude review

## Verdict
