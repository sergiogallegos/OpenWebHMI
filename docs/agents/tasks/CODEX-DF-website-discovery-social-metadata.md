---
id: CODEX-DF
title: Website discoverability, social previews, and launch metadata
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DF — Website discovery and launch metadata

## Brief

### Goal

Make public website and demo links produce accurate search results and useful,
versioned social previews without adding telemetry or third-party runtime calls.

### Dependencies

- CODEX-DC for final page positioning and claim language.
- CODEX-DE for stable homepage/demo routes and media placement.
- CODEX-CZ for approved social images and capture metadata.
- A published CODEX-DB video URL before adding video-specific metadata.

### Required behavior

- emit an absolute canonical URL and `og:url` for every public page;
- replace the generic social image with CODEX-CZ-approved raster previews that
  meet current Open Graph/Twitter dimensions and remain readable when cropped;
- provide page-specific titles, descriptions, preview images, and alt text for
  Home, Demo, Docs, Download, and About;
- add sitemap and robots output appropriate for the static production site;
- add minimal standards-based structured data for the open-source software and
  website, limited to claims supported by current artifacts and `VISION.md`;
- add video structured data only after a stable public thumbnail, URL, upload
  date, duration, and transcript/caption source exist;
- ensure preview assets contain no customer data, credentials, private network
  details, unreleased features, or unlicensed marks;
- document a repeatable preview-validation and release-staleness check.

### Tests and verification

- run website typecheck and production build;
- parse built HTML to verify one canonical URL and the required Open Graph,
  Twitter, and structured-data fields per public route;
- validate sitemap URLs against generated routes and check internal links;
- inspect representative link previews using local/static tooling where possible;
- verify the built site makes no analytics, tracking-pixel, or unapproved
  third-party network request during startup or normal browsing;
- validate structured data with a standards-compatible validator and record any
  external validation that could not run in the submission environment.

### Acceptance criteria

- [ ] Every public page has a correct absolute canonical URL and page-specific metadata.
- [ ] Homepage and Demo social cards use approved real-product captures, not the generic logo card.
- [ ] Sitemap, robots, Open Graph, Twitter, and structured-data output match generated routes.
- [ ] Video metadata is omitted until every required published-video field is real.
- [ ] No metadata overstates shipped scope or leaks private/demo-hardware details.
- [ ] No telemetry or third-party runtime dependency is introduced.

### Out of scope

- SEO guarantees, paid advertising, analytics, tracking, or visitor profiling.
- Uploading social images/video or deploying the site without explicit authorization.
- Inventing ratings, pricing, testimonials, customers, download counts, or release dates.

### Risks and gotchas

- Relative Open Graph image paths are inconsistently handled by crawlers; generated
  metadata must use the configured production site URL.
- Structured data is another public claim surface and follows the same
  shipped-versus-planned honesty rules as visible page copy.

## Codex log

## Claude review

## Verdict
