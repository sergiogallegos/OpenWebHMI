---
id: CODEX-BA
title: website refresh — surface AN/AO/AS/AT/AQ/AR features on openwebhmi.com landing + docs + download pages
owner: codex
phase: 4
status: merged
created: 2026-05-27
last-update: 2026-05-29 claude [Opus 4.7]
merge-commit: a7ba50a
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

2026-05-27 codex [gpt-5]: Submitted. Refreshed `apps/website/src/pages/index.astro`, `docs.astro`, and `download.astro` only. Landing page now adds a `v1.x feature highlights` section using existing `.scope-list` styling with CFR21 audit-log framing first, plus theme/material-demo, historian, Raspberry Pi, and widget export/import cards; Phase 4 status text now names CI/release/docs/hardware-validation closeout. Docs page now adds a `Features & how-tos` section with six `${BLOB}` links to the new docs. Download page now clarifies Linux `.deb` + `.rpm` from CI and AppImage from release builds, and adds a Pi 4/5 prerequisite link. Verification: `pnpm --filter @openwebhmi/website build`, `git diff --check`, and local HTTP content checks against `http://127.0.0.1:4321/`, `/docs`, and `/download` passed; `rg -n "compliant|certified|validated for|meets 21 CFR" apps/website/src/pages` found no website over-claim wording. Browser-plugin visual verification could not run because the in-app browser backend reported `Browser is not available: iab`.

## Claude review

### 2026-05-29 18:00  claude [Opus 4.7]

**Independent verification**
- `pnpm --filter @openwebhmi/website build` — clean (Astro + `astro check` exit 0; 5 pages built in 617ms).
- `git show a7ba50a` — read all three modified page files end-to-end (`index.astro` +70 lines, `docs.astro` +42 lines, `download.astro` +16 lines).
- `rg -n "compliant|certified|validated for|meets 21 CFR" apps/website/src/pages` — **zero matches** (independently confirmed Codex's claim — no website over-claim wording made it through).
- `rg -n "Material-inspired|Material Design demo" apps/website/src/pages` — both phrasings present, no "MD3-compliant" claim.
- `rg -n "hardware smoke|operator validation" apps/website/src/pages` — Pi disclaimer honored.

**What's being fixed**
- The public-facing Astro site at `openwebhmi.com` was last touched 2026-05-25 (commit `1f36e6b`), before the v1.x feature-parity sweep landed. Six new features and their canonical docs had no surface on the marketing site.

**Root cause confirmation**
- Confirmed: pre-BA `index.astro` had no "v1.x feature highlights" section; `docs.astro` `sections` array had only "Project overview", "For contributors", "For AI agents"; `download.astro` Linux installer line said only ".deb/.rpm" with no AppImage clarification post-AZ.

**Fix appropriateness**
- **Right pages, right scope**: index + docs + download modified; about + 404 unchanged per brief. No README or repo-side doc changes.
- **No new dependencies, no new components, no new CSS classes** — reuses `.scope-list` for the new feature-highlights section (the brief explicitly named this pattern as acceptable). Honors "Static-first, no JS framework runtime" rationale documented in the site's own README.
- **`${BLOB}` template consistently used** for all 6 + 1 new GitHub links (matches the existing `docs.astro` convention).
- **Phase 4 status text rewrite** is accurate and tightened: "five drivers are in place for the v1.0 envelope; the current closeout work is CI hygiene, release packaging, website/docs refresh, and the real-hardware validation gate before 1.0. Recent v1.x additions include audit-log Part 11 framing, themes, widget transfer, historian retention planning, and Raspberry Pi deployment notes." Names the actual current state without overpromising.
- **Card order on the landing page** leads with the CFR21 audit-log — exactly what the brief asked for ("pharma/medical/food-safety procurement starts with audit-log compliance").

**Test proof**
- Astro build is the verification per the brief; 5/5 pages built clean; no type errors.
- Codex's HTTP content checks against `127.0.0.1:4321/`, `/docs`, `/download` passed locally.
- Codex's `rg` honesty audit found no over-claim wording — I re-ran the same audit + an extended one (`Material-inspired` + `hardware smoke|operator validation`) and confirmed.
- Browser visual verification not run (Codex noted this honestly: "Browser is not available: iab"). Acceptable — Cloudflare Workers will auto-deploy this commit and preview-URL render is the final visual gate.

**Residual risk**
- **Cloudflare Workers auto-deploys on push to main**, so the live site updates immediately. The Astro build is green, so the deploy should succeed; if the rendered site has visual layout issues with the new section, the maintainer would need to spot them post-deploy. Browser-visual verification deferred per Codex's honest "iab not available" disclosure.
- **The 5 new landing-page cards each have an external `↗` link.** Search engine crawlers count these as outbound; not a problem for v1.x marketing scope, but if Astro Content Collections lands in v0.2 (per site README's own roadmap), the same content could become internal pages.
- **No screenshot in the repo** to show what the new layout looks like; the visual will only be verifiable once the Cloudflare preview / production URL renders. Standard for the project's iterate-on-push convention.
- **No new docs/agents/notes/ file** to capture the honesty conventions (CFR21 framing, Material-inspired framing, Pi disclaimer) as a durable reference for future website touches. Codex flagged the conventions in the Codex log inline; a future v1.1 polish could lift these into `docs/agents/notes/website-honesty-conventions.md`.

**Strong points (✅)**
- **CFR21 framing is gold-standard honest**: card text says "tamper-evident SHA-256 hash chain, verify CLI, and an engineering coverage map with every clause marked partial or out-of-scope." Matches the doc's own "engineering coverage map, not a legal certification" discipline exactly. NO "compliant" or "certified" anywhere on the website — Codex's rg audit confirms this independently.
- **"Material-inspired demo widgets"** — matches the AT verdict's reframing recommendation precisely. Brief's risk note ("'Material Design' framing is also load-bearing") fully honored.
- **Raspberry Pi card says "honest hardware-smoke boundaries for operator validation"** — explicit acknowledgement that the doc itself is honest about not being hardware-verified. Brief's risk note honored.
- **Three pages, three small focused edits** — no scope creep into about.astro, 404.astro, README.md, or repo-side docs. AS/AT pattern of "minimum diff that satisfies the contract" continues.
- **Phase 4 description honestly says "real-hardware validation gate before 1.0"** — doesn't claim the gate is passed.
- **Reuses `.scope-list` styling** — no new CSS, no new components. Site stays static, fast, low-spec-browser-friendly per its own design rationale.
- **Codex's rg-based honesty self-audit in the Codex log** ("`rg -n "compliant|certified|validated for|meets 21 CFR" apps/website/src/pages` found no website over-claim wording") is exactly the discipline the brief asked for. Documented proactively rather than waiting for review to find it.
- **Browser-visual not run, honestly disclosed**: "Browser is not available: iab" — names the specific environment limitation rather than claiming visual verification succeeded.

**Findings**
- 🟢 The card-link styling (text link with `↗` arrow after the bullet content) reuses the existing pattern from the "Want to follow along?" CTA section. Visually consistent.
- 🟢 The Phase 4 text rewrite names "website/docs refresh" as part of the closeout — meta-acknowledges that BA itself is part of the closeout. Self-referential but accurate.
- 🟡 No `docs/agents/notes/website-honesty-conventions.md` to capture the CFR21/Material/Pi framing rules for future website touches. v1.1 polish if the website gets significant content edits again.
- 🟡 Browser visual not verified; deferred to Cloudflare preview render. If layout breaks visually but the Astro build passes, the maintainer would need to catch it post-deploy. Low likelihood given the reused `.scope-list` styling.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `index.astro`: "Where we are" Phase 4 bullet refreshed; new "v1.x feature highlights" section with **5 cards** (brief said 4-6; Codex picked 5 by bundling themes + Material into one card). CFR21 is the first card.
- ✅ `docs.astro`: new "Features & how-tos" section with 6 `DocLink` entries pointing at canonical `docs/*.md` files.
- ✅ `download.astro`: AppImage line clarified ("Linux .deb + .rpm from CI builds, AppImage from release builds"); Pi prerequisite added.
- ✅ `about.astro` unchanged.
- ✅ `pnpm --filter @openwebhmi/website build` exits 0.
- ✅ No new dependencies in `apps/website/package.json`.
- ✅ No new CSS classes; reuses `.scope-list`.
- ✅ All new GitHub links use `${BLOB}` template.

## Verdict

**Merged** at `a7ba50a`. **Phase 4 fully closed** with this merge (Website refresh was the last open task).

What's NOT yet proven by this merge:
- Visual layout in the deployed Cloudflare preview/production (auto-deploys on push; verifiable post-merge by visiting `openwebhmi.com`).
- Lighthouse score impact on the landing page from the added 5 cards (low likelihood of regression; the cards are pure HTML/CSS with no JS).
- Long-term docs.astro maintenance pattern as more features ship — the `sections` array data-driven approach scales naturally, but at v1.x+10 features the page may benefit from grouping or a sub-index.

No follow-ups opened from BA specifically. Two yellow polish items (no `docs/agents/notes/website-honesty-conventions.md`, no browser-visual gate) are light enough to roll into future touchups without their own briefs.

**Closing note**: BA closes the post-AN..AT marketing-surface sync and completes Phase 4 (quality follow-ups AJ/AK/AL/AM + feature-parity AN/AO/AQ/AR/AS/AT + CI hygiene AU/AV/AW/AX/AY/AZ + website BA — all 22 tasks merged, AP rejected for VISION.md scope discipline). The honesty discipline — CFR21 framing without "compliant" claims, Material-inspired without MD3 claims, Pi documented without hardware-claims — held cleanly from brief to docs to website. That's the procurement-grade Honesty rule applied end-to-end across the surface.
