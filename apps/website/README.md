# `@openwebhmi/website` — marketing + docs site

The public-facing OpenWebHMI website. Built with [Astro 5](https://astro.build/) as static HTML; no JS framework runtime, no SSR. The output deploys to any static host (GitHub Pages, Cloudflare Pages, Vercel, Netlify) for free.

## Pages

| Path | Purpose |
|---|---|
| `/` | Landing — hero, why, scope envelope, stack comparison, phase status. |
| `/docs` | Documentation index. Links to canonical markdown in the repo. A first-class docs site (search, versioned reference) ships in Phase 4. |
| `/download` | "How to build from source" today. Pre-built installers ship in Phase 4. |
| `/about` | Mission, contribution model, AI-collaboration workflow, license, maintainer. |
| `/404` | Standard 404. |

## Dev

```bash
pnpm install
pnpm --filter @openwebhmi/website dev
# → http://localhost:4321
```

`astro check` runs as part of `build` so type errors block deploy.

## Build

```bash
pnpm --filter @openwebhmi/website build
# → apps/website/dist/
```

The `dist/` directory is a static site ready to upload anywhere. Per host:

| Host | Notes |
|---|---|
| GitHub Pages | Push `dist/` to a `gh-pages` branch (or use `peaceiris/actions-gh-pages`). |
| Cloudflare Pages | Connect the repo, set build command `pnpm --filter @openwebhmi/website build`, output dir `apps/website/dist`. |
| Vercel | Same shape; framework preset = Astro. |

## Brand assets

The favicon and OpenGraph image in `public/` are copies from the project's [`brand/`](../../brand/) directory. If the brand assets change, sync them with:

```bash
cp brand/openwebhmi-favicon-blue.svg apps/website/public/favicon.svg
cp brand/openwebhmi-mark-blue.svg    apps/website/public/og-image.svg
```

(A small build script in Phase 4 will automate this.)

## Why Astro, not Next.js / Docusaurus / VitePress

- **Static-first.** This is a marketing + docs site, not an app. SSG output is faster, cheaper, and more reliable than SSR.
- **No framework lock-in.** Astro can host React / Vue / Svelte islands when something genuinely needs interactivity. We don't yet, but the option is there.
- **MDX-ready.** When we want to render the canonical `docs/` markdown into the website (rather than linking to GitHub), Astro Content Collections handle this cleanly.
- **Boring HTML.** No client-side router, no hydration cost on a static page. Good for Lighthouse, good for SEO, good for low-spec plant-floor browsers.

## Roadmap (this site, not the platform)

- **v0.1** — landing + docs index linking to GitHub markdown. *(this commit)*
- **v0.2** — render `docs/*.md` directly via Astro Content Collections; full-text search via Pagefind.
- **v0.3** — embedded interactive demo (browser runtime hitting a hosted gateway) — Phase 2+ deliverable.
- **v1.0** — versioned API reference (rustdoc + TS reference), tutorial series, blog/changelog at `/blog`.

## Out of scope

- Authentication (Phase 3 platform feature, not a website concern).
- User accounts / dashboards.
- Backend services for the website itself — keep it static.
