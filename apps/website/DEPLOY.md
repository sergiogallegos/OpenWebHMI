# Deploying `apps/website` to Cloudflare (Workers + static assets)

The OpenWebHMI website deploys via **Cloudflare Workers with static assets** (the unified flow that replaced standalone Pages in late 2024). Free static hosting at the edge, auto-deploys on every push to `main`, preview URLs on every PR. DNS for `openwebhmi.com` lives in the same Cloudflare account, so the custom-domain hookup is one click.

No GitHub Actions deploy workflow is needed — Cloudflare Workers Builds does the build + deploy itself. Type errors are gated by the existing CI (`.github/workflows/ci.yml`'s `node` job runs `pnpm -r typecheck` and `pnpm -r build`).

The deploy is configured by [`apps/website/wrangler.toml`](wrangler.toml). It declares no Worker script — only `[assets]` — so the deploy is pure static content.

## Build settings to copy into the dashboard

The "Set up your application" form Cloudflare shows after **Connect to Git**:

| Field | Value |
|---|---|
| Project name | `openwebhmi` (or leave the auto-suggested `OpenWebHMI` — Cloudflare lowercases it) |
| Build command | `pnpm install --frozen-lockfile && pnpm --filter @openwebhmi/website build` |
| Deploy command | `npx wrangler deploy --config apps/website/wrangler.toml` |
| Builds for non-production branches | ✅ keep checked (gives preview URLs on every PR) |

> **Note**: There is no separate "Build output directory" or "Root directory" field in the Workers flow — the wrangler config in the repo handles output paths. Just paste the two commands and let `wrangler.toml` do the rest.

## Step-by-step setup (one-time)

### 1. Connect the GitHub repo

1. Sign in to <https://dash.cloudflare.com>.
2. Sidebar → **Workers & Pages**.
3. Click **Create** → choose **Connect to a Git repository** (or **Import a repository** in some account variants).
4. Authorize Cloudflare for your GitHub account when prompted. Single-repo access is fine; pick **`sergiogallegos/OpenWebHMI`**.
5. Select the repo → **Begin setup** (or **Set up application**).

### 2. Fill in the build form

Use the table above. Specifically:

- **Project name**: `openwebhmi`. This becomes your `*.workers.dev` subdomain (`openwebhmi.<account>.workers.dev`) before the custom domain takes over.
- **Build command**: `pnpm install --frozen-lockfile && pnpm --filter @openwebhmi/website build`. Two stages: install the workspace, then run Astro's build for the `@openwebhmi/website` package only.
- **Deploy command**: `npx wrangler deploy --config apps/website/wrangler.toml`. Wrangler reads the config from that path; `[assets].directory = "./dist"` resolves to `apps/website/dist`.
- **Builds for non-production branches**: leave checked. PRs get preview deploys at unique URLs.

You don't need to expand **Advanced settings** for v1.

### 3. Save and deploy

Click **Deploy**. The first build runs about 2–3 minutes:
- ~90 s — `pnpm install --frozen-lockfile`
- ~30 s — `astro check` (TypeScript check)
- ~20 s — `astro build` (static output)
- ~20 s — `wrangler deploy` upload

Once green, Cloudflare gives you a temporary URL like `https://openwebhmi.<account>.workers.dev`. Open it; verify the landing page renders.

### 4. Wire up the custom domain `openwebhmi.com`

1. From the project page, **Settings** → **Triggers** → **Custom domains** → **Add Custom Domain**.
   *(Some Cloudflare account variants surface this as "Domains & Routes". Same destination.)*
2. Enter `openwebhmi.com` → **Add Custom Domain**.
3. Cloudflare detects you own the domain (it's already in your account) and proposes adding a `CNAME` record pointing to the Worker. Click **Activate**.
4. Cloudflare creates the DNS record and issues a Let's Encrypt cert. Typical: 30 seconds to 5 minutes.
5. Add `www.openwebhmi.com` the same way and accept Cloudflare's offered redirect to the apex.

### 5. Verify

- `https://openwebhmi.com` loads the landing page.
- `/docs`, `/download`, `/about` render.
- `/favicon.svg` returns the blue mark.
- `/some-nonexistent-path` falls through to the `404` page (because `wrangler.toml` sets `not_found_handling = "404-page"`).
- Browser shows valid TLS (Let's Encrypt cert issued by Cloudflare).
- DevTools → Network → reload → `cf-cache-status: HIT` on the second request.

## Auto-deploy behavior

After setup, every push to `main` triggers a production deploy. Every PR triggers a preview deploy with a unique URL — Cloudflare comments the URL on the PR.

## Gotchas

- **Don't omit `--frozen-lockfile`** in the install step. Without it, pnpm may quietly mutate `pnpm-lock.yaml` mid-build and the deploy state diverges from `main`.
- **Don't `cd apps/website` in the build command.** The `--filter` flag selects the workspace member; `cd` works on local CI but Cloudflare resets between command segments in some variants. The `--filter` form is portable.
- **Wrangler resolves `[assets].directory` relative to the wrangler.toml file**, not the cwd. So `directory = "./dist"` correctly means `apps/website/dist`.
- **`compatibility_date`** must be set in `wrangler.toml`. If you bump it later, test in a preview branch first.
- **404 routing**: `not_found_handling = "404-page"` makes Cloudflare serve `/404/index.html` (Astro's directory format) on missing routes. If you change `astro.config.mjs` to `format: "file"`, also verify `/404.html` exists in dist.
- **TLS cert window**: don't enable Cloudflare's "Always Use HTTPS" toggle for the apex domain before the TLS cert issues — there's a brief window where the redirect targets HTTPS but the cert isn't yet valid. Wait until the Custom Domains panel shows "Active" with a valid cert.
- **File limit**: free tier allows 20,000 files per deploy. We have ~50 — irrelevant for years.
- **Build time limit**: 25 minutes free, 30 paid. Our build is 3 minutes — safe.

## Rollback

Project page → **Deployments** → click an older successful build → **Rollback to this deployment**. Instant.

## Future subdomains (Phase 4)

The `docs.openwebhmi.com` (rustdoc) and `demo.openwebhmi.com` (hosted demo gateway) subdomains referenced in `docs.astro` and `download.astro` are Phase 4 deliverables:

- `docs.openwebhmi.com` → another Workers project deploying `cargo doc --workspace --no-deps` output, built by a `.github/workflows/rustdoc.yml` workflow that runs cargo doc and uploads via Wrangler.
- `demo.openwebhmi.com` → a Cloudflare Worker reverse-proxying to a hosted gateway (or a Cloudflare Tunnel back to a self-hosted gateway). Out of scope today.

## Why this works without classic Pages

Cloudflare's late-2024 product change merged Pages into Workers. Static sites that used to deploy via Pages now deploy via Workers + an `[assets]` binding. Behavior is identical for our use case (free, edge-cached, auto-TLS, preview URLs on PRs) — the only difference is one small `wrangler.toml` file in the repo instead of dashboard-only config.
