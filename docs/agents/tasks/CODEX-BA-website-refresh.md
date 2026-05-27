---
id: CODEX-BA
title: website refresh — surface AN/AO/AS/AT/AQ/AR features on openwebhmi.com landing + docs + download pages
owner: codex
phase: 4
status: open
created: 2026-05-27
last-update: 2026-05-27 claude [Opus 4.7]
---

# CODEX-BA — Website refresh (post AN..AZ feature-parity sweep)

## Brief

> The `apps/website/` Astro site at `openwebhmi.com` was last touched 2026-05-25 in commit `1f36e6b` (before the AN..AT feature-parity sweep landed). It's missing the CFR21 audit-log story (CODEX-AN), Theme Editor (CODEX-AO), Material widget pack (CODEX-AT), widget export/import (CODEX-AS), historian capacity story (CODEX-AQ), and Raspberry Pi deployment guide (CODEX-AR). Bring the three public pages in sync with what actually shipped. **Static site, no JS framework runtime — pure Astro/HTML/CSS edits.**

### Goal

A first-time visitor at `openwebhmi.com` and `openwebhmi.com/docs` can see the v1.x features that landed in the AN..AT sweep, and a docs-curious visitor finds the 6 new documentation pages. The CFR21 callout in particular should be visible on the landing page because pharma/medical/food-safety procurement starts with audit-log compliance.

### Context to read first

- `apps/website/README.md` — site overview, page list, dev/build commands.
- `apps/website/DEPLOY.md` — Cloudflare Workers deploy flow (auto-deploys on push to `main`; preview URLs on PRs).
- `apps/website/src/pages/index.astro` — landing page (the file with the most updates).
- `apps/website/src/pages/docs.astro` — docs index (Astro-data-driven from a `sections` array of `DocLink` entries).
- `apps/website/src/pages/download.astro` — build-from-source page; mentions Phase 4 installer aspirations.
- `apps/website/src/pages/about.astro` — current; no edits needed.
- The 6 new doc files this brief surfaces:
  - `docs/audit-log-cfr21-mapping.md` (CODEX-AN)
  - `docs/theme-editor.md` (CODEX-AO)
  - `docs/widget-export-import.md` (CODEX-AS)
  - `docs/widget-packs.md` (CODEX-AT)
  - `docs/historian.md` (CODEX-AQ)
  - `docs/deployment/raspberry-pi.md` (CODEX-AR)
- CODEX-AZ merge state (verify before touching `download.astro`): AZ took Path B — AppImage dropped from CI bundle targets, deferred to release-only/local builds.

### Files to create / modify

**`apps/website/src/pages/index.astro`** — modify three sections:

1. **"Where we are" Phase 4 bullet**: replace the single-line "Phase 4 — five drivers..., plugin SDK, real-hardware validation gate before 1.0" with a tighter Phase 4 summary plus a callout for the v1.x sweep additions. Keep the badge `<span class="badge">In progress</span>` shape.
2. **Add a new section after "Stack at a glance"** titled e.g. "v1.x feature highlights" with 4-6 cards (reuse the existing `.scope-list` or `.phase-list` visual pattern — no new CSS). Cards for:
   - Audit-log CFR21 Part 11 framing (hash chain + verify CLI + clause-mapping doc) — lead with this; link to `docs/audit-log-cfr21-mapping.md`.
   - Project themes + Material Design demo widget pack — link to `docs/theme-editor.md` and `docs/widget-packs.md`.
   - Historian capacity (SQLite 281 TB ceiling) + retention primitives — link to `docs/historian.md`.
   - Raspberry Pi 4/5 edge deployment — link to `docs/deployment/raspberry-pi.md`.
   - Widget export/import between projects — link to `docs/widget-export-import.md`.
3. **No other section changes.** "Why another platform?", "v1.0 scope", "Stack at a glance", "What we explicitly don't try to be", "Want to follow along?" — all stay as-is.

**`apps/website/src/pages/docs.astro`** — modify the `sections` array:

- Add a new section `"Features & how-tos"` between "Project overview" and "For contributors":
  - Audit log CFR21 mapping — `${BLOB}/docs/audit-log-cfr21-mapping.md` — audience: `"everyone"`
  - Theme editor — `${BLOB}/docs/theme-editor.md` — audience: `"everyone"`
  - Widget export/import — `${BLOB}/docs/widget-export-import.md` — audience: `"everyone"`
  - Widget packs (Material Design demo) — `${BLOB}/docs/widget-packs.md` — audience: `"everyone"`
  - Historian capacity + retention — `${BLOB}/docs/historian.md` — audience: `"everyone"`
  - Raspberry Pi deployment — `${BLOB}/docs/deployment/raspberry-pi.md` — audience: `"everyone"`
- No other section changes. The "Project overview" / "For contributors" / "For AI agents" sections stay as-is.

**`apps/website/src/pages/download.astro`** — two narrow edits:

1. **"Coming in Phase 4 (1.0)" list**: clarify the Linux installer line. Pre-AZ it said "Linux .deb/.rpm". Post-AZ: ".deb + .rpm shipped from CI; AppImage available from release-only workflow (deferred from CI per [CODEX-AZ](https://github.com/sergiogallegos/OpenWebHMI/blob/main/docs/agents/tasks/CODEX-AZ-ci-appimage-linuxdeploy.md))" — or whatever brevity-equivalent fits the existing card style. Don't link directly to the agent-tasks-dir from a public marketing page; rephrase as "Linux: .deb + .rpm in CI builds; AppImage from release builds" and skip the link.
2. **Prerequisites section**: add a `<li>` for "Raspberry Pi 4 / 5 (64-bit Pi OS Bookworm) — see [Raspberry Pi deployment guide](docs/deployment/raspberry-pi.md)" near the Tauri prerequisites item, OR add a new "Edge deployment" section after the "Build from source" section. Codex picks whichever fits the page flow better.

**`apps/website/src/pages/about.astro`** — **NO CHANGES.** Current; "How it's built" + status + license + maintainer all accurate.

**`apps/website/src/pages/404.astro`** — **NO CHANGES.** Standard 404.

### Behavior

After the change:
- `https://openwebhmi.com/` shows the 4-6 v1.x feature cards prominently. Specifically, CFR21 audit-log framing is visible without scrolling past the fold (or as close to the fold as the page structure allows).
- `https://openwebhmi.com/docs` lists the 6 new doc files under a "Features & how-tos" heading.
- `https://openwebhmi.com/download` correctly reflects the AppImage CI-vs-release split.
- The site builds cleanly: `pnpm --filter @openwebhmi/website build` exits 0 with no Astro type errors (the build step runs `astro check`).
- Cloudflare Workers auto-deploys on push to `main`; preview URL appears on the PR (if a PR is opened).
- No new dependencies. No new components. No new CSS classes. Reuse `.scope-list` / `.dont-list` / `.phase-list` / `.doc-grid` visual patterns.

### Test requirements

- `pnpm --filter @openwebhmi/website build` exits 0. (This runs `astro check && astro build`.)
- `pnpm --filter @openwebhmi/website dev` then visit `http://localhost:4321` and visually confirm:
  - Landing page CFR21 + theme + Pi + widget-export cards render.
  - Docs page shows the new "Features & how-tos" section with 6 entries.
  - Download page Linux installer line correctly clarifies AppImage.
- (Optional) `pnpm -r --if-present typecheck` to confirm no TS regression in shared types.
- No new vitest needed — the website is static Astro; existing pages don't have unit tests and this brief doesn't add a testable surface.

### Acceptance criteria

- [ ] `apps/website/src/pages/index.astro`: "Where we are" Phase 4 bullet refreshed; new "v1.x feature highlights" section with 4-6 cards including CFR21 first.
- [ ] `apps/website/src/pages/docs.astro`: new "Features & how-tos" section with 6 `DocLink` entries pointing at canonical `docs/*.md` files on GitHub.
- [ ] `apps/website/src/pages/download.astro`: AppImage line clarified per CODEX-AZ Path B; Pi mention added (in Prerequisites or a new section, whichever fits).
- [ ] `apps/website/src/pages/about.astro` unchanged.
- [ ] `pnpm --filter @openwebhmi/website build` exits 0.
- [ ] No new dependencies in `apps/website/package.json`.
- [ ] No new CSS classes; reuse existing visual patterns.
- [ ] All new GitHub links use `${BLOB}` template (the existing pattern in docs.astro) for consistency.

### Out of scope

- **Migrating `docs/*.md` to Astro Content Collections** (the website README calls this v0.2 work; CODEX-BA is just a content refresh, not the Content Collections migration).
- **Adding full-text search via Pagefind** (v0.2 website roadmap item).
- **Embedded interactive demo** (v0.3 website roadmap item; Phase 2+ deliverable).
- **Versioned API reference** (v1.0 website roadmap item).
- **Marketing copy rewrites** beyond the new feature cards. "Why another platform?" and "What we explicitly don't try to be" stay as-is.
- **README.md changes** — the repo README is already current (CODEX-AQ + CODEX-AR commits updated it). This brief is purely the website.
- **Blog / changelog page** at `/blog` — v1.0 website roadmap; not in this brief.
- **Repo-side docs file changes** — the 6 new doc files were authored in earlier merges; CODEX-BA only references them from the website.
- **CFR21 marketing claims beyond what the doc supports.** The website should say "engineering coverage map for 21 CFR Part 11 (every clause marked partial or out-of-scope; none claimed satisfied)" or similar honest framing — NOT "CFR21 Part 11 compliant." Mirror the discipline of the `docs/audit-log-cfr21-mapping.md` doc.

### Risks / gotchas

- **CFR21 over-promising is the biggest risk.** The audit-log doc itself is honest ("This is an engineering coverage map, not a legal certification"); the website needs to mirror that. Do NOT say "compliant", "certified", "validated for", or "meets". Use phrasing like "engineering coverage map for", "partial coverage of", "tamper-evident hash chain providing engineering evidence for". A regulated customer's procurement officer or auditor will notice false claims.
- **"Material Design" framing is also load-bearing.** Per the CODEX-AT verdict, the widget pack is "Material-inspired demo" (Option C / hand-rolled), NOT strict MD3 compliance. The website should say "opt-in Material Design demo widget pack" or "Material-inspired widget pack" — matching the docs/widget-packs.md phrasing.
- **Don't link to agent-tasks-dir paths from a public marketing page.** `docs/agents/tasks/CODEX-*.md` is internal collaboration; don't expose it on the marketing site. Public users see `docs/*.md` only.
- **Don't add JS frameworks or runtime hydration.** The site is static-first per its own design rationale. New cards should be pure Astro (no React/Vue/Svelte islands needed for this brief).
- **Don't sync brand assets unnecessarily.** The site already has `public/favicon.svg` + `public/og-image.svg`; the website README documents the sync command for when brand assets change. Brand hasn't changed; don't run the sync.
- **`docs/deployment/raspberry-pi.md` is honest about not being hardware-verified.** Mirror that honesty if writing copy: "Raspberry Pi 4/5 deployment guide (cross-compile path documented; hardware smoke deferred to operator validation)" — not "Pi deployment supported".
- **Cloudflare Workers auto-deploy** on push to `main` means the live site updates immediately when CODEX-BA merges. No staging gate. Per AY's pattern of "fix obvious blocker → next layer surfaces", if the build breaks (e.g. broken markdown link in a DocLink), the site stays on the previous deploy but the build fails loudly.
- **Page weight discipline.** The landing page is a marketing surface; adding 6 new cards is fine but don't bloat it past the "low-spec plant-floor browsers" Lighthouse target the site README calls out as a goal.
- **The website Pages roadmap (v0.2 Content Collections, v0.3 demo, v1.0 versioned API)** is already documented in `apps/website/README.md`. Don't reshape that roadmap from this brief.

## Codex log

## Claude review

## Verdict
