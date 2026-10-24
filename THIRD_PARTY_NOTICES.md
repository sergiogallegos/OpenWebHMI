# Third-party notices

OpenWebHMI depends on third-party Rust crates, npm packages, Python, operating
system libraries, fonts, icons, and protocol implementations. Those materials
remain governed by their original licenses; OpenWebHMI's AGPL/MPL declarations
do not replace them.

The exact dependency set for a build is recorded by:

- `Cargo.lock` for Rust crates;
- `pnpm-lock.yaml` for npm packages;
- the operating system/package manifest used to build a release; and
- any asset-specific notice or manifest shipped beside bundled media.

Important direct dependencies include the independently published MIT
`rust-ethernet-ip` library, `async-opcua`, `tokio-modbus`, `rumqttc`, `rumqttd`,
`prost`, `ads`, React, Vite, Tauri, and Monaco Editor. Each retains its upstream
copyright and license.

Release builders must generate a dependency-license report from the locked
dependency graph and ship it beside this notice. This repository-level notice is
not a substitute for that release-specific report.

No Eclipse Tahu source is bundled. Sparkplug wire structures in
`crates/driver-mqtt/src/sparkplug.rs` are first-party Rust definitions written
against the public protocol specification.

## Website social marks

The SVG files in `apps/website/public/brands/` are third-party marks, used to
identify links to their respective services. They are not covered by the product
AGPL/MPL licenses or the documentation CC BY license.

- `github-black.svg` and `github-white.svg`: unmodified Invertocat SVGs from
  `GitHub Logos/SVG/GitHub_Invertocat_Black.svg` and
  `GitHub Logos/SVG/GitHub_Invertocat_White.svg` in the official
  [GitHub logo pack](https://brand.github.com/GitHub_Logos.zip).
  Copyright and trademarks belong to GitHub, Inc.; use is governed by the
  [GitHub brand guidelines](https://brand.github.com/foundations/logo).
- `x-white.svg`: unmodified `logo.svg` from the official
  [X logo pack](https://about.x.com/content/dam/about-twitter/x/brand-toolkit/x-logo.zip).
  Copyright and trademarks belong to X Corp.; use is governed by the
  [X brand toolkit](https://about.x.com/en/who-we-are/brand-toolkit).

Assets were retrieved on 2026-10-24 and are served locally. Heart and globe icons
in the website Icon component are first-party interface artwork, not service logos.
