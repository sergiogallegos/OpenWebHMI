# Deploying `apps/website` to Cloudflare Pages

The OpenWebHMI website deploys via **Cloudflare Pages**: free static hosting, auto-detects the pnpm monorepo, deploys on every push to `main`, opens preview URLs on every PR. DNS for `openwebhmi.com` is in the same Cloudflare account, so the custom-domain hookup is one click.

No GitHub Actions deploy workflow is needed — Cloudflare Pages does the build + deploy itself. Type errors are gated by the existing CI (`.github/workflows/ci.yml`'s `node` job runs `pnpm -r typecheck` and `pnpm -r build`).

## Build settings to copy into the dashboard

| Field | Value |
|---|---|
| Project name | `openwebhmi` (or `openwebhmi-website`) |
| Production branch | `main` |
| Framework preset | **Astro** (auto-detected) |
| Build command | `pnpm --filter @openwebhmi/website build` |
| Build output directory | `apps/website/dist` |
| Root directory | *(leave blank — must run from repo root)* |
| Environment variable | `NODE_VERSION` = `20` |

> **Critical**: Root directory must be **blank** (= repo root). pnpm needs the workspace root to install. If you set Root directory = `apps/website`, install will fail with "no pnpm-workspace.yaml found".

## Step-by-step setup (one-time)

### 1. Create the Pages project

1. Sign in to <https://dash.cloudflare.com>.
2. Sidebar → **Workers & Pages**.
3. Click **Create** → **Pages** tab → **Connect to Git**.
4. **Authorize Cloudflare** for your GitHub account when prompted. Grant access to **`sergiogallegos/OpenWebHMI`** (you can grant access to a single repo rather than all repos).
5. Select the repository → **Begin setup**.

### 2. Configure build

Fill in the form using the table above. Specifically:

- **Project name**: `openwebhmi` — this becomes your `*.pages.dev` subdomain (`openwebhmi.pages.dev`) before the custom domain takes over.
- **Production branch**: `main`.
- **Framework preset**: select **Astro**. Cloudflare may auto-detect from `astro.config.mjs`; if not, pick it manually so the right Node version + cache strategy applies.
- **Build command**: `pnpm --filter @openwebhmi/website build`. This invokes `astro check && astro build` (per the website's `package.json`).
- **Build output directory**: `apps/website/dist`. Astro's default output. Don't add a leading `/`.
- **Root directory**: leave **blank**. Cloudflare runs `pnpm install` from this directory; the install needs to see `pnpm-workspace.yaml` at the repo root.

### 3. Environment variables

Click **Environment variables (advanced)** and add:

| Name | Value | Scope |
|---|---|---|
| `NODE_VERSION` | `20` | Production + Preview |

Cloudflare reads `packageManager: pnpm@9.0.0` from `package.json` automatically — no `PNPM_VERSION` needed.

### 4. Save and deploy

Click **Save and Deploy**. The first build takes about 2–3 minutes:
- ~90s: pnpm install
- ~30s: `astro check` (TypeScript)
- ~20s: `astro build` (static output)

Once green, Cloudflare gives you a temporary URL like `https://openwebhmi.pages.dev`. Verify it loads.

### 5. Wire up the custom domain `openwebhmi.com`

1. In the project page, click **Custom domains** → **Set up a custom domain**.
2. Enter `openwebhmi.com` → **Continue**.
3. Cloudflare detects you own the domain (it's in the same account) and proposes adding a `CNAME` record pointing to the Pages project. Click **Activate**.
4. Cloudflare creates the DNS record and issues a Let's Encrypt cert. Typical: 30 seconds to 5 minutes.
5. **Add `www` as a redirect**: back in **Custom domains**, add `www.openwebhmi.com`. Cloudflare offers to redirect `www.openwebhmi.com → openwebhmi.com` automatically; accept it.

### 6. Verify

- `https://openwebhmi.com` loads the landing page.
- `https://openwebhmi.com/docs`, `/download`, `/about` render.
- `https://openwebhmi.com/favicon.svg` returns the blue mark.
- Browser shows valid TLS (Let's Encrypt cert issued by Cloudflare).
- Open DevTools → Network → reload → look for `cf-cache-status: HIT` on the second request. (First request will be `MISS`; subsequent are `HIT`.)

## Auto-deploy behavior

After setup, every push to `main` triggers a production deploy. Every PR triggers a preview deploy with a unique URL like `https://<branch-hash>.openwebhmi.pages.dev` — Cloudflare comments the URL on the PR.

## Gotchas

- **Don't `cd apps/website` in the build command.** Cloudflare runs the command from the install directory; pnpm's `--filter` selects the workspace member.
- **pnpm version**: Cloudflare reads `packageManager: pnpm@9.0.0` from `package.json` automatically. Don't pin a different version in environment variables.
- **`astro check` failures fail the build**, which is intentional. If a deploy fails on a type error, fix it locally with `pnpm --filter @openwebhmi/website typecheck` and push the fix.
- **Don't enable Cloudflare's "Always Use HTTPS" toggle for the apex domain before the TLS cert issues** — there's a brief window where the redirect targets HTTPS but the cert isn't yet valid. Wait until the Custom Domains panel shows "Active".
- **File limit**: free tier allows 20,000 files per deploy. We're at ~50 — irrelevant for years.
- **Build time limit**: 25 minutes free, 30 paid. Our build is 3 minutes — safe.
- **If install fails with `Cannot find lockfile`**: confirm Root directory is empty in the build settings. Cloudflare runs install from that directory; the workspace root has the lockfile.
- **Node 20 vs 22**: `NODE_VERSION=20` is the LTS at time of writing and matches our CI. Bump to 22 only after the GitHub Actions CI bumps too.

## Promoting a preview to production

Cloudflare auto-promotes the latest `main` commit. To roll back to a previous build: project page → **Deployments** → click the older build → **Manage deployment** → **Rollback to this deployment**. Instant.

## Disabling deploys temporarily

If you need to push a doc-only fix that shouldn't trigger a redeploy: Cloudflare Pages doesn't have per-commit skip flags. The simplest path is to land the change on a feature branch, accept the preview URL only, and only fast-forward `main` when you're ready for production.

## Future subdomains (Phase 4)

The `docs.openwebhmi.com` (rustdoc) and `demo.openwebhmi.com` (hosted demo gateway) subdomains referenced in `docs.astro` and `download.astro` are Phase 4 deliverables. When we're ready:

- `docs.openwebhmi.com` → another Cloudflare Pages project pointing at a `cargo doc --workspace --no-deps` output, deployed by a `.github/workflows/rustdoc.yml` workflow that runs cargo doc and uploads the artifact.
- `demo.openwebhmi.com` → a Cloudflare Worker reverse-proxying to a hosted gateway (or a Cloudflare Tunnel back to a self-hosted gateway). Out of scope today.

## Update workflow

When you change the website:
1. Push to a feature branch → Cloudflare creates a preview deploy → comment on the PR.
2. Open a PR → CI's `node` job runs typecheck + build (catches errors before the preview deploys).
3. Merge to `main` → Cloudflare promotes to production at `openwebhmi.com` within ~3 minutes.

No additional GitHub Actions configuration is needed for the website itself.
