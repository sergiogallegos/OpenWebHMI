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
