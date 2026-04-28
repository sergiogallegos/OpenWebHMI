import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
  // Domain registered via Cloudflare. The trailing slash is intentional.
  site: "https://openwebhmi.com/",
  // Static output is fine for v1; switch to "server" only if a feature
  // genuinely requires SSR (we deliberately avoid that for now).
  output: "static",
  build: {
    format: "directory",
  },
  trailingSlash: "ignore",
  compressHTML: true,
});
