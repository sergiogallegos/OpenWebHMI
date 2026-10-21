# Dependency upgrade evidence — 2026-10-21

Codex [GPT-6] execution evidence for EG/EH/DU. Versions were verified against official registry metadata on 2026-10-21. This report covers version upgrades; it does not close engine extraction, feature-budget, transport-migration or performance-baseline tasks.

## Toolchains

| Tool | Previous project pin | Selected | Decision |
|---|---|---|---|
| Rust / Cargo | 1.98.1 | 1.99.1 | Latest stable; edition 2024 retained |
| Product TypeScript | 7.0.2 | 7.0.2 | Already latest stable |
| Website TypeScript | 5.9.3 | 6.0.2 | Latest supported compiler API; latest @astrojs/check declares `^5.0.0 || ^6.0.0` |
| Node | 24.21.0 | 24.22.1 | Latest patch of the existing Node 24 LTS line; Node 26 Current is not adopted |
| pnpm | 9.15.9 | 12.14.0 | Latest stable; strict peers, explicit lifecycle-build policy and frozen-install verification |

Exact validation tools: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`, `rustfmt 1.10.0-stable`, `clippy 0.1.99`; Node `v24.22.1`, pnpm `12.14.0`. Rust was installed with rustup. Node/pnpm verification executables were downloaded with checksum/integrity verification into `/tmp/openwebhmi-tools`; the machine-wide Node/pnpm installations were not replaced.

Sources: [Rust releases](https://blog.rust-lang.org/releases/), [stable distribution manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml), [Node releases](https://nodejs.org/dist/index.json), [TypeScript metadata](https://registry.npmjs.org/typescript/latest), [Astro checker metadata](https://registry.npmjs.org/@astrojs/check/0.9.12), [pnpm metadata](https://registry.npmjs.org/pnpm/12.14.0).

## Every used direct external Cargo dependency

The table includes normal, build and dev consumers and target-specific declarations. “Before” lists versions present in the old workspace lockfile, including versions required by other dependency families. Features are those requested by workspace consumers; shared feature-unification limits are described in the footprint section. Each selected version equals the verified latest non-yanked stable candidate. Existing exact driver pins remain exact.

| Dependency | Declared before → after | Before locked | Selected / verified latest | License | Requested features | Consumers / dependency kinds |
|---|---|---|---|---|---|---|
| [ads](https://crates.io/crates/ads) | `=0.4.4` → `=0.4.4` | 0.4.4 | 0.4.4 | MIT/Apache-2.0 | defaults only | driver-ads; normal |
| [anyhow](https://crates.io/crates/anyhow) | `1` → `1.0.104` | 1.0.104 | 1.0.104 | MIT OR Apache-2.0 | defaults only | alarm-engine, audit-log, auth, backup, driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, gateway, historian, project-store, scripting, sim-modbus, sim-mqtt, sim-opcua, sim-rockwell; normal |
| [async-opcua](https://crates.io/crates/async-opcua) | `=0.18.0` → `=0.19.0` | 0.18.0 | 0.19.0 | MPL-2.0 | client, server | driver-opcua, sim-opcua; normal |
| [async-trait](https://crates.io/crates/async-trait) | `0.1` → `0.1.92` | 0.1.92 | 0.1.92 | MIT OR Apache-2.0 | defaults only | driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell; normal |
| [base64](https://crates.io/crates/base64) | `0.22` → `0.23.1` | 0.21.7, 0.22.1 | 0.23.1 | MIT OR Apache-2.0 | defaults only | auth; normal |
| [bcrypt](https://crates.io/crates/bcrypt) | `0.15` → `0.19.3` | 0.15.1 | 0.19.3 | MIT | defaults only | auth; normal |
| [clap](https://crates.io/crates/clap) | `4` → `4.6.7` | 4.6.6 | 4.6.7 | MIT OR Apache-2.0 | derive, env | gateway, sim-modbus, sim-mqtt, sim-opcua, sim-rockwell; normal |
| [flate2](https://crates.io/crates/flate2) | `1` → `1.1.10` | 1.1.9 | 1.1.10 | MIT OR Apache-2.0 | defaults only | backup; normal |
| [futures-util](https://crates.io/crates/futures-util) | `0.3` → `0.3.34` | 0.3.34 | 0.3.34 | MIT OR Apache-2.0 | defaults only | driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, gateway; dev, normal |
| [jsonpath-rust](https://crates.io/crates/jsonpath-rust) | `=1.0.4` → `=1.0.12` | 1.0.4 | 1.0.12 | MIT | defaults only | driver-mqtt; normal |
| [jsonwebtoken](https://crates.io/crates/jsonwebtoken) | `9` → `11.1.0` | 9.3.1 | 11.1.0 | MIT | aws_lc_rs | auth; normal |
| [prost](https://crates.io/crates/prost) | `=0.13.5` → `=0.14.4` | 0.13.5 | 0.14.4 | Apache-2.0 | defaults only | driver-mqtt, sim-mqtt; normal |
| [rand](https://crates.io/crates/rand) | `0.8` → `0.10.3` | 0.10.2, 0.8.8, 0.9.5 | 0.10.3 | MIT OR Apache-2.0 | defaults only | auth, driver-api, scripting; normal |
| [rumqttc](https://crates.io/crates/rumqttc) | `=0.25.1` → `=0.25.1` | 0.25.1 | 0.25.1 | Apache-2.0 | websocket | driver-mqtt, sim-mqtt; normal |
| [rumqttd](https://crates.io/crates/rumqttd) | `=0.20.0` → `=0.20.0` | 0.20.0 | 0.20.0 | Apache-2.0 | defaults only | sim-mqtt; normal |
| [rusqlite](https://crates.io/crates/rusqlite) | `0.32` → `0.40.2` | 0.32.1 | 0.40.2 | MIT | backup, bundled | alarm-engine, audit-log, auth, historian, project-store; normal |
| [rust-ethernet-ip](https://crates.io/crates/rust-ethernet-ip) | `1.2.1` → `1.2.1` | 1.2.1 | 1.2.1 | MIT | defaults only | driver-rockwell, sim-rockwell; dev, normal |
| [rustls-pemfile](https://crates.io/crates/rustls-pemfile) | `2` → `2.2.0` | 2.2.0 | 2.2.0 | Apache-2.0 OR ISC OR MIT | defaults only | gateway; normal |
| [serde](https://crates.io/crates/serde) | `1` → `1.0.229` | 1.0.229 | 1.0.229 | MIT OR Apache-2.0 | derive | alarm-engine, audit-log, auth, backup, designer, driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, gateway, historian, project-store, protocol, scripting, sim-rockwell; normal |
| [serde_json](https://crates.io/crates/serde_json) | `1` → `1.0.154` | 1.0.151 | 1.0.154 | MIT OR Apache-2.0 | defaults only | alarm-engine, audit-log, auth, backup, designer, driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, gateway, historian, project-store, protocol, scripting; normal |
| [sha2](https://crates.io/crates/sha2) | `0.10` → `0.11.1` | 0.10.9 | 0.11.1 | MIT OR Apache-2.0 | defaults only | audit-log; normal |
| [tauri](https://crates.io/crates/tauri) | `2` → `2.12.5` | 2.11.5 | 2.12.5 | Apache-2.0 OR MIT | defaults only | designer; normal |
| [tauri-build](https://crates.io/crates/tauri-build) | `2` → `2.7.4` | 2.6.3 | 2.7.4 | Apache-2.0 OR MIT | defaults only | designer; build |
| [tempfile](https://crates.io/crates/tempfile) | `3` → `3.27.0` | 3.27.0 | 3.27.0 | MIT OR Apache-2.0 | defaults only | alarm-engine, audit-log, auth, backup, gateway, historian, project-store, scripting; dev |
| [thiserror](https://crates.io/crates/thiserror) | `2` → `2.0.21` | 1.0.69, 2.0.20 | 2.0.21 | MIT OR Apache-2.0 | defaults only | alarm-engine, audit-log, auth, driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, historian, project-store, scripting; normal |
| [tokio](https://crates.io/crates/tokio) | `1` → `1.54.1` | 1.53.1 | 1.54.1 | MIT | full, sync | alarm-engine, audit-log, driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, gateway, historian, project-store, scripting, sim-modbus, sim-mqtt, sim-opcua, sim-rockwell, tag-engine; normal |
| [tokio-modbus](https://crates.io/crates/tokio-modbus) | `=0.16.1` → `=0.17.0` | 0.16.1 | 0.17.0 | MIT OR Apache-2.0 | defaults only | driver-modbus; normal |
| [tokio-rustls](https://crates.io/crates/tokio-rustls) | `0.26` → `0.26.6` | 0.25.0, 0.26.4 | 0.26.6 | MIT OR Apache-2.0 | defaults only | gateway; normal |
| [tokio-serial](https://crates.io/crates/tokio-serial) | `5` → `5.5.0` | 5.5.0 | 5.5.0 | MIT | defaults only | driver-modbus; normal |
| [tokio-stream](https://crates.io/crates/tokio-stream) | `0.1` → `0.1.19` | 0.1.19 | 0.1.19 | MIT | sync | driver-ads, driver-mqtt, driver-opcua, scripting; normal |
| [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite) | `0.24` → `0.30.0` | 0.24.0 | 0.30.0 | MIT | defaults only | gateway; normal |
| [tokio-util](https://crates.io/crates/tokio-util) | `0.7` → `0.7.20` | 0.7.19 | 0.7.20 | MIT | rt | gateway; normal |
| [toml](https://crates.io/crates/toml) | `0.8` → `1.1.8` | 0.8.23, 0.9.12+spec-1.1.0, 1.1.4+spec-1.1.0 | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | defaults only | gateway, project-store, sim-rockwell; normal |
| [tracing](https://crates.io/crates/tracing) | `0.1` → `0.1.44` | 0.1.44 | 0.1.44 | MIT | defaults only | alarm-engine, backup, driver-ads, driver-api, driver-modbus, driver-mqtt, driver-opcua, driver-rockwell, gateway, historian, scripting, sim-modbus, sim-mqtt, sim-opcua, sim-rockwell; normal |
| [tracing-subscriber](https://crates.io/crates/tracing-subscriber) | `0.3` → `0.3.23` | 0.3.23 | 0.3.23 | MIT | env-filter | driver-ads, gateway, scripting, sim-modbus, sim-mqtt, sim-opcua, sim-rockwell; dev, normal |
| [uuid](https://crates.io/crates/uuid) | `1` → `1.28.0` | 1.25.0 | 1.28.0 | Apache-2.0 OR MIT | serde, v4 | auth, gateway; normal |

PyO3 `0.22` was an unused workspace declaration: no workspace consumer or locked package existed. It was removed rather than updated to the verified stable `0.30.1`.

Axum is not a direct gateway dependency. `rumqttd 0.20.0` (already latest stable) requires Axum `0.7`; the lockfile retains `0.7.9`. The verified latest Axum is `0.8.9`. Forcing that incompatible major into the broker is not a version-only update. DP still owns the future direct gateway Axum/Hyper migration.

## Frontend families

| Package | Previous declarations | Selected / verified latest | License |
|---|---|---|---|
| [@astrojs/check](https://www.npmjs.com/package/@astrojs/check) | ^0.9.0 | ^0.9.12 | MIT |
| [@monaco-editor/react](https://www.npmjs.com/package/@monaco-editor/react) | ^4.7.0 | ^4.7.0 | MIT |
| [@tauri-apps/cli](https://www.npmjs.com/package/@tauri-apps/cli) | ^2.11.4 | ^2.12.4 | Apache-2.0 OR MIT |
| [@testing-library/dom](https://www.npmjs.com/package/@testing-library/dom) | absent | ^10.4.2 | MIT |
| [@testing-library/react](https://www.npmjs.com/package/@testing-library/react) | ^15.0.0 | ^16.3.3 | MIT |
| [@testing-library/user-event](https://www.npmjs.com/package/@testing-library/user-event) | ^14.5.0 | ^14.6.8 | MIT |
| [@types/node](https://www.npmjs.com/package/@types/node) | ^22.10.0 | ^26.6.5 | MIT |
| [@types/react](https://www.npmjs.com/package/@types/react) | ^18.2.0 | ^19.3.0 | MIT |
| [@types/react-dom](https://www.npmjs.com/package/@types/react-dom) | ^18.2.0 | ^19.3.0 | MIT |
| [@vitejs/plugin-react](https://www.npmjs.com/package/@vitejs/plugin-react) | ^4.2.0 | ^6.1.2 | MIT |
| [astro](https://www.npmjs.com/package/astro) | ^5.0.0 | ^7.4.2 | MIT |
| [jsdom](https://www.npmjs.com/package/jsdom) | ^24.1.0 | ^30.1.3 | MIT |
| [monaco-editor](https://www.npmjs.com/package/monaco-editor) | ^0.55.1 | ^0.57.0 | MIT |
| [react](https://www.npmjs.com/package/react) | ^18.2.0 | ^19.3.1 | MIT |
| [react-dom](https://www.npmjs.com/package/react-dom) | ^18.2.0 | ^19.3.1 | MIT |
| [typescript](https://www.npmjs.com/package/typescript) | 5.9.3, 7.0.2 | 6.0.2, 7.0.2 | Apache-2.0 |
| [vite](https://www.npmjs.com/package/vite) | ^5.2.0 | ^8.3.5 | MIT |
| [vite-plugin-monaco-editor](https://www.npmjs.com/package/vite-plugin-monaco-editor) | ^1.1.0 | ^1.1.0 | MIT |
| [vitest](https://www.npmjs.com/package/vitest) | ^1.6.0 | ^5.0.5 | MIT |

`@testing-library/dom` is now explicit to satisfy the React testing adapter’s peer dependency. The Monaco plugin is already at its latest `1.1.0`, but its published package imports `esbuild` without declaring it. A version-specific pnpm package extension declares `esbuild 0.28.2`; the clean pnpm 12 build failed before that correction and passed afterward.

Security overrides select patched DOMPurify `3.4.17`, esbuild `0.28.2`, fast-uri `3.1.9` and source-map-js `1.2.2`. fast-uri stays on the compatible major 3 line (latest major 4 is not forced into its parent). Overrides are restricted to affected version ranges.

pnpm 12 enforces release-age verification. The explicit `minimumReleaseAgeExclude` entries identify the exact current upstream releases verified during this upgrade, including matching platform binaries. The default age protection is retained for other versions. Only esbuild has an allowed dependency lifecycle build; no blanket build-script permission was added.

The website uses static SVG assets and no `astro:assets` transforms. Astro’s documented passthrough image service is selected and its unused optional Sharp package is excluded; this prevents installing its LGPL libvips binaries. [Astro documentation](https://docs.astro.build/en/reference/errors/missing-sharp/) describes this service.

## Compatibility migrations and test proof

- Rand 0.10 uses `rng`, `RngExt::random_range` and `Rng::fill_bytes`; jitter ranges and password entropy inputs are unchanged.
- rusqlite 0.40 uses `MAIN_DB` for backup/restore. Existing online-backup and round-trip coverage passes.
- async-opcua 0.19 service calls return the library error type. The existing status-code mapping is reached through `map_opcua_error`; real-protocol simulator read/write/browse/subscription tests pass.
- tungstenite 0.30 owns text/binary payloads through `Utf8Bytes`/`Bytes`; send sites convert owned buffers and the shutdown test compares the close reason as a string. Wire JSON remains unchanged.
- TOML 1 changed `Value::from_str` to parse a value rather than a document. Existing backup tests failed (7 failures with `name = "Demo"` rejected) after the dependency update. Switching project artifact parsing to `toml::from_str` restores document parsing; those same tests pass in all three final suites.
- jsonwebtoken 11 requires an explicit crypto backend. AWS-LC is selected through its supported feature; it is already used by the TLS stack. The gateway’s HMAC JWT issue/verify/tamper/expiry tests pass. The OPC UA dependency still brings RSA, as listed in residual advisories below.
- jsdom 30 retains the correct border color while serializing a mixed shorthand/longhand border differently. The existing bad-quality assertion now checks `style.borderColor`, preserving the expected red-color check without depending on shorthand serialization.

## Dependency and feature footprint

| Locked inventory | Before | After |
|---|---:|---:|
| External package/version entries | 662 | 671 |
| Names with multiple locked versions | 56 | 62 |

| Normal/build dependency closure: all-target metadata inventory | Before external entries / duplicate names | After external entries / duplicate names |
|---|---:|---:|
| openwebhmi-gateway | 201 / 9 | 207 / 15 |
| openwebhmi-tag-engine | 66 / 2 | 79 / 2 |
| openwebhmi-designer | 445 / 33 | 438 / 29 |
| sim-mqtt | 309 / 23 | 311 / 24 |
| sim-opcua | 230 / 10 | 229 / 9 |

These closures exclude dev edges but include target-specific possibilities from metadata; they are not loaded runtime dependencies or native-only build measurements. The workspace still enables Tokio `full` and OPC UA client/server together. Feature isolation and Edge/Standard/Medium profiles are not implemented here and remain DO work. No claim of a smaller gateway or faster build is made. A clean before/after Rust build-time baseline was not captured.

| Production browser entry JS (build output, decimal kB) | Before | After | Before gzip | After gzip |
|---|---:|---:|---:|---:|
| Runtime | 447.73 | 527.67 | 124.48 | 153.80 |
| Designer | 523.28 | 586.03 | 139.19 | 166.96 |

These are entry chunks, not total assets or Monaco worker payloads. Both entry bundles grew; Vite’s existing chunk-size warning remains visible. No optimization or performance gain is claimed. The unsigned debug macOS app bundle was produced successfully (27.52 MiB); no prior native size baseline exists.

## Advisory and license results

The final frontend `pnpm audit --json` reports zero info/low/moderate/high/critical vulnerabilities. Before targeted transitive patches, the upgraded frontend lockfile reported 13 advisories: 3 low, 1 moderate, 9 high.

The final OSV query checked all 671 external Cargo package/version entries. Patched rustls `0.23.46` and rustls-webpki `0.103.16` were selected with precise updates. Remaining results are not suppressed or represented as fixed:

| Package | Advisory results | Constraint / ownership |
|---|---|---|
| atomic-polyfill 1.0.3 | [RUSTSEC-2023-0089](https://osv.dev/vulnerability/RUSTSEC-2023-0089) | Unmaintained transitive embedded-platform fallback. |
| glib 0.18.5 | [GHSA-wrw7-89jp-8q8g](https://osv.dev/vulnerability/GHSA-wrw7-89jp-8q8g), [RUSTSEC-2024-0429](https://osv.dev/vulnerability/RUSTSEC-2024-0429) | Linux GTK/Tauri dependency; Linux validation remains required. |
| proc-macro-error 1.0.4 | [RUSTSEC-2024-0370](https://osv.dev/vulnerability/RUSTSEC-2024-0370) | Unmaintained transitive procedural-macro tool. |
| rsa 0.9.10 | [RUSTSEC-2023-0071](https://osv.dev/vulnerability/RUSTSEC-2023-0071) | OPC UA 0.19 uses RSA 0.9; no fixed stable release is available in that family. |
| rustls 0.22.4 | [RUSTSEC-2026-0346](https://osv.dev/vulnerability/RUSTSEC-2026-0346), [RUSTSEC-2026-0349](https://osv.dev/vulnerability/RUSTSEC-2026-0349) | Old 0.22 line through rumqttd 0.20 → tokio-rustls 0.25; simulator/dev closure. |
| rustls-pemfile 2.2.0 | [RUSTSEC-2025-0134](https://osv.dev/vulnerability/RUSTSEC-2025-0134) | Latest 2.2.0 is marked unmaintained; gateway and MQTT consumers remain. |
| rustls-webpki 0.102.8 | [GHSA-82j2-j2ch-gfr8](https://osv.dev/vulnerability/GHSA-82j2-j2ch-gfr8), [GHSA-965h-392x-2mh5](https://osv.dev/vulnerability/GHSA-965h-392x-2mh5), [GHSA-pwjx-qhcg-rvj4](https://osv.dev/vulnerability/GHSA-pwjx-qhcg-rvj4), [GHSA-xgp8-3hg3-c2mh](https://osv.dev/vulnerability/GHSA-xgp8-3hg3-c2mh), [RUSTSEC-2026-0049](https://osv.dev/vulnerability/RUSTSEC-2026-0049), [RUSTSEC-2026-0098](https://osv.dev/vulnerability/RUSTSEC-2026-0098), [RUSTSEC-2026-0099](https://osv.dev/vulnerability/RUSTSEC-2026-0099), [RUSTSEC-2026-0104](https://osv.dev/vulnerability/RUSTSEC-2026-0104), [RUSTSEC-2026-0382](https://osv.dev/vulnerability/RUSTSEC-2026-0382) | Old 0.102 line required by latest rumqttc/rumqttd; includes the production MQTT driver. |
| unsafe-libyaml 0.2.11 | [RUSTSEC-2024-0449](https://osv.dev/vulnerability/RUSTSEC-2024-0449) | Unmaintained transitive YAML parser. |

This is an advisory-presence scan of exact versions, not a reachability analysis. Informational unmaintained notices are distinct from security findings. The updated dependency set is not claimed vulnerability-free. Resolving the upstream-bound findings requires separate library integration work, not false compatible-major overrides.

License metadata was inspected for changed Rust packages and installed frontend packages. New/updated Rust expressions are MIT/Apache/BSD/ISC/Zlib alternatives plus the existing MPL-2.0 OPC UA family. Frontend exceptions reviewed for this build: Lightning CSS is an unmodified MPL-2.0 build tool; no Lightning CSS source is copied into product source, and source/license notices must accompany tool redistribution. Existing BlueOak-1.0.0 dependencies have permissive terms; CC0-1.0 packages supply data. DOMPurify is consumed under its Apache-2.0 alternative. The unused optional LGPL Sharp/libvips chain is excluded. No GPL/AGPL/SSPL external dependency was intentionally introduced. This review is not a release-specific binary/source notice audit; release builders must still generate the notices required by THIRD_PARTY_NOTICES.md. [Mozilla MPL FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/) and [Lightning CSS license](https://github.com/parcel-bundler/lightningcss/blob/master/LICENSE) are the reference terms.

## Validation and limits

- Rust: locked workspace all-features build; all-targets/all-features Clippy with `-D warnings`; locked docs; formatting check. All passed.
- Rust: three consecutive final locked workspace all-features suites passed, each with 165 tests and zero failures. Simulator coverage includes Rockwell, Modbus, OPC UA, MQTT/WebSocket; ADS uses its existing harness, not real TwinCAT hardware.
- Frontend: Node 24.22.1 / pnpm 12.14.0 frozen force reinstall with strict peers; recursive strict typechecking, tests and builds passed. Three consecutive suites each passed 152 tests (48 protocol, 67 components, 16 runtime, 21 designer). The final website configuration is also built three times.
- Native: `pnpm --filter @openwebhmi/designer exec tauri build --debug --bundles app --no-sign --ci` produced an unsigned macOS arm64 `.app`; signing, notarization and Linux/Windows packaging were not run.
- Browser: Chrome against a disposable gateway with no PLC drivers: designer login, project load, component tree and theme navigation; runtime login, connected project rendering and modal Cancel interaction passed. The runtime used a temporary build configured for the ephemeral gateway. Designer embedded preview was not validated: its existing fixed localhost:5173 URL differs from the ephemeral smoke server. Monaco editing/worker interactions and the full manual designer checklist were not exercised.
- Repository validators: `scripts/validate-agent-files`, `scripts/validate-licenses --self-test`, and `git diff --check` passed.
- Real PLC hardware, the 24-hour CompactLogix/ControlLogix soak, MQTT TLS hardware smoke, Linux and Windows execution, and performance workloads were not run. Verification completed before the subsequent maintainer-directed commit and push.
