# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 4 — 1.0 release (in progress)

**Driver scope expansion (2026-04-30).** Phase 4 now ships **four new drivers** (in addition to Rockwell from Phase 1): OPC UA, Modbus TCP/RTU, MQTT (incl. Sparkplug B), and Beckhoff ADS. Each lands as a real driver crate + simulator harness + simulator-driven CI integration tests + wiki entry + designer manual smoke step. Real-hardware validation remains the pre-1.0 gate. Drivers are independent — they can run in parallel.

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-BB | Rust 1.96 toolchain bump — 1.95.0 → 1.96.0 + one assert_matches! example conversion | codex | open | 2026-05-29 claude [Opus 4.7] | [`CODEX-BB-rust-1.96-toolchain-bump.md`](tasks/CODEX-BB-rust-1.96-toolchain-bump.md) |
| CODEX-BC | Identifier sanitization at the ProjectStore boundary — close path-traversal / arbitrary file R/W | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BC-identifier-sanitization.md`](tasks/CODEX-BC-identifier-sanitization.md) |
| CODEX-BD | Refuse to boot without an explicit JWT signing secret — remove the hardcoded fallback | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BD-jwt-secret-boot-refusal.md`](tasks/CODEX-BD-jwt-secret-boot-refusal.md) |
| CODEX-BE | Backup HTTP side-channel hardening — pre-auth body cap, header-first auth, TLS parity | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BE-backup-http-hardening.md`](tasks/CODEX-BE-backup-http-hardening.md) |
| CODEX-BF | Server-authoritative tag-write authorization — replace client-steerable view ACL | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BF-server-authoritative-write-acl.md`](tasks/CODEX-BF-server-authoritative-write-acl.md) |
| CODEX-BG | Auth + connection hardening — login rate-limit/lockout, connection caps, message/subscription limits | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BG-auth-rate-limit-conn-caps.md`](tasks/CODEX-BG-auth-rate-limit-conn-caps.md) |
| CODEX-BH | Audit hash-chain tamper-evidence — HMAC keyed digest + external anchor option | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BH-audit-hmac-hash-chain.md`](tasks/CODEX-BH-audit-hmac-hash-chain.md) |
| CODEX-BI | Gateway driver dispatch — route driver_type to the correct driver and adopt DriverSupervisor | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BI-gateway-driver-dispatch.md`](tasks/CODEX-BI-gateway-driver-dispatch.md) |
| CODEX-BJ | DriverUpdate connection-state signal — distinguish one bad tag from a lost connection | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BJ-driverupdate-connection-state.md`](tasks/CODEX-BJ-driverupdate-connection-state.md) |
| CODEX-BK | Rockwell rust-ethernet-ip 0.7 → 1.2 upgrade + target-type-aware writes | codex | submitted | 2026-08-25 codex [gpt-5] | [`CODEX-BK-rockwell-ethernet-ip-1.2-typed-writes.md`](tasks/CODEX-BK-rockwell-ethernet-ip-1.2-typed-writes.md) |
| CODEX-BL | MQTT driver event-loop resilience — survive broker hiccups, propagate Bad on loss | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BL-mqtt-eventloop-resilience.md`](tasks/CODEX-BL-mqtt-eventloop-resilience.md) |
| CODEX-BM | ADS notification fan-out fix — stop MPMC sample theft across subscriptions | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BM-ads-notification-fanout.md`](tasks/CODEX-BM-ads-notification-fanout.md) |
| CODEX-BN | ADS write range-checking — reject out-of-range integers instead of silent truncation | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BN-ads-write-range-checking.md`](tasks/CODEX-BN-ads-write-range-checking.md) |
| CODEX-BO | Modbus robustness — hostname/DNS, per-request timeouts, bounds-checked decode | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BO-modbus-hostname-timeout-bounds.md`](tasks/CODEX-BO-modbus-hostname-timeout-bounds.md) |
| CODEX-BP | OPC UA driver correctness + security — dead-session detect, subscription cleanup, security config, addressing | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BP-opcua-correctness-security.md`](tasks/CODEX-BP-opcua-correctness-security.md) |
| CODEX-BQ | Sparkplug B conformance — signed-int datatype decode, DEATH stale-marking, command writes | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BQ-sparkplug-conformance.md`](tasks/CODEX-BQ-sparkplug-conformance.md) |
| CODEX-BR | Atomic artifact writes — real rename atomicity, fsync, non-colliding temp files | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BR-atomic-artifact-writes.md`](tasks/CODEX-BR-atomic-artifact-writes.md) |
| CODEX-BS | Explicit memory-tag namespace + honest write results for unknown drivers | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BS-memory-tag-namespace-write-honesty.md`](tasks/CODEX-BS-memory-tag-namespace-write-honesty.md) |
| CODEX-BT | Reconnect write-queue safety — don't replay stale operator setpoints after reconnect | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BT-reconnect-write-queue-safety.md`](tasks/CODEX-BT-reconnect-write-queue-safety.md) |
| CODEX-BU | Gateway lifecycle hardening — non-fatal accept, reliable RPC replies, WS keepalive, backup-download TTL sweep | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BU-gateway-lifecycle-hardening.md`](tasks/CODEX-BU-gateway-lifecycle-hardening.md) |
| CODEX-BV | Config-driven data paths — stop hardcoding sqlite files into the working directory | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BV-config-driven-data-paths.md`](tasks/CODEX-BV-config-driven-data-paths.md) |
| CODEX-BW | Tag-engine scaling — sharded locking, slot GC, update coalescing for thousands of sub-second tags | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BW-tag-engine-scaling.md`](tasks/CODEX-BW-tag-engine-scaling.md) |
| CODEX-BX | Gateway service-context refactor — replace process-global OnceLock singletons, split handle_connection | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BX-gateway-service-context-refactor.md`](tasks/CODEX-BX-gateway-service-context-refactor.md) |
| CODEX-BY | Runtime error boundaries — one throwing widget must not blank the operator HMI | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BY-runtime-error-boundaries.md`](tasks/CODEX-BY-runtime-error-boundaries.md) |
| CODEX-BZ | Gateway client robustness — socket teardown, write/ack feedback, request timeouts, error correlation | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-BZ-gateway-client-robustness.md`](tasks/CODEX-BZ-gateway-client-robustness.md) |
| CODEX-CA | Alarm subscription lifecycle — add alarm.unsubscribe, stop re-subscribe churn | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CA-alarm-subscription-lifecycle.md`](tasks/CODEX-CA-alarm-subscription-lifecycle.md) |
| CODEX-CB | Runtime subscription + render architecture — scale to hundreds of sub-second tags | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CB-runtime-subscription-render-scaling.md`](tasks/CODEX-CB-runtime-subscription-render-scaling.md) |
| CODEX-CC | Designer editing integrity — script-source cross-contamination, uneditable JSON props, input races, reconnect | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CC-designer-editing-integrity.md`](tasks/CODEX-CC-designer-editing-integrity.md) |
| CODEX-CD | Component theming + accessibility — CSS-variable adoption, configurable labels, keyboard/switch semantics | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CD-component-theming-accessibility.md`](tasks/CODEX-CD-component-theming-accessibility.md) |
| CODEX-CE | Alarm deadband/hysteresis + on/off delay — stop analog-alarm chattering | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CE-alarm-deadband-delay.md`](tasks/CODEX-CE-alarm-deadband-delay.md) |
| CODEX-CF | Alarm ack state-machine fix — ack re-evaluates, no stuck-active on a stalled tag, no spurious transitions | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CF-alarm-ack-state-machine.md`](tasks/CODEX-CF-alarm-ack-state-machine.md) |
| CODEX-CG | Historian read-path SQL downsampling — bucket server-side instead of loading all raw rows | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CG-historian-sql-downsampling.md`](tasks/CODEX-CG-historian-sql-downsampling.md) |
| CODEX-CH | Audit query SQL pushdown — filter/paginate in SQL using the existing indexes | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CH-audit-query-sql-pushdown.md`](tasks/CODEX-CH-audit-query-sql-pushdown.md) |
| CODEX-CI | Docs-vs-code reconciliation — make architecture.md / README / feature-matrix match the shipped surface | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CI-docs-code-reconciliation.md`](tasks/CODEX-CI-docs-code-reconciliation.md) |
| CODEX-CJ | Gateway observability — /health + /metrics endpoints and optional structured logging | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CJ-gateway-observability.md`](tasks/CODEX-CJ-gateway-observability.md) |
| CODEX-CK | CI hardening — multi-platform matrix, --all-features alignment, supply-chain gate, release workflow | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CK-ci-hardening.md`](tasks/CODEX-CK-ci-hardening.md) |
| CODEX-CL | Scripting system.* library gap — implement or formally defer the documented RPC surface + triggers | codex | open | 2026-07-12 claude [Fable 5] | [`CODEX-CL-scripting-system-library-gap.md`](tasks/CODEX-CL-scripting-system-library-gap.md) |

### Phase 4 dependency graph

```
(v1.0 feature ladder complete — AE/AF/AI/AG/AD/AH/AC merged)
        │
        ├── quality follow-ups (review pass 2 — Rust 1.95 idioms vs tokio/axum/ripgrep)
        │       ├── CODEX-AJ  async hygiene                      ← merged 8022426
        │       ├── CODEX-AK  tokio handle ergonomics            ← merged 3bde658
        │       ├── CODEX-AL  API polish                         ← merged 79ee864
        │       └── CODEX-AM  stdlib + deps modernization        ← merged 823d7a1
        │
        ├── v1.x improvements (competitor feature-parity sweep — opened 2026-05-25)
        │       ├── CODEX-AN  audit-log CFR21 framing + hash chain         ← merged 66d9e76 (top priority — pharma/medical procurement gate)
        │       ├── CODEX-AO  Theme Editor UI (v0.2 Stretch pulled forward) ← merged 428a9cf
        │       ├── CODEX-AP  Recipes (process-control management)          ← REJECTED 2026-05-25 (Claude brief error vs VISION.md L37; planning notes already at roadmap.md L146)
        │       ├── CODEX-AQ  Historian capacity + retention callout        ← merged 411f449
        │       ├── CODEX-AR  Raspberry Pi deployment guide                 ← merged 411f449
        │       ├── CODEX-AS  Widget export/import between projects         ← merged a225aa7
        │       └── CODEX-AT  Material Design widget pack (demo subset)     ← merged 428a9cf (Option C / hand-rolled per Codex log; depends on AO for shared CSS-variable contract)
        │
        ├── CI hygiene (unblock main CI — opened 2026-05-25)
        │       ├── CODEX-AU  ci: install Tauri Linux build deps             ← merged 4e9bc9b
        │       └── CODEX-AV  ci: resolve pnpm version conflict              ← merged 4e9bc9b
        │
        └── CI hygiene follow-ups (downstream failures exposed as earlier blockers cleared)
                ├── CODEX-AW  gateway WS script-event integration test timeout  ← merged cc2e1f7
                ├── CODEX-AX  component-library cannot resolve @openwebhmi/protocol  ← merged 47da4ea
                ├── CODEX-AY  ci: install Tauri Linux build deps in Node job too  ← merged 7505a62
                └── CODEX-AZ  ci: AppImage bundling fails with 'failed to run linuxdeploy'  ← merged bd3d006 (Path B; 🎉 CI fully green for the first time across AU+AV+AW+AX+AY+AZ)

Website refresh (surface AN..AR features on openwebhmi.com — opened 2026-05-27)
        └── CODEX-BA  website refresh — landing + docs + download pages    ← merged a7ba50a (🎉 closes Phase 4 — all 22 in-flight tasks merged + AP rejected for VISION scope)

Tier 5 mechanical cleanups (drift-on-bump — opened 2026-05-29)
        └── CODEX-BB  Rust 1.96 toolchain bump + one assert_matches! example  ← open (mirrors CODEX-AM shape; scope-locked)
```

### v1.0 hardening backlog (whole-repo review — opened 2026-07-12)

A whole-repo review (five parallel review passes: core runtime, drivers, services/security, frontend, architecture) opened **36 tasks (BC..CL)**. The full validation matrix was green at review time — every finding below is latent, not caught by the existing suite. Priority tiers (business/risk order, not dependency order):

```
Tier 1 — Security (v1.0 blockers; small contained fixes; ship first)
        ├── CODEX-BC  ProjectStore identifier sanitization (path traversal → arbitrary file R/W)   ← CRITICAL
        ├── CODEX-BD  refuse to boot without an explicit JWT secret (kill hardcoded fallback)       ← CRITICAL
        ├── CODEX-BE  backup HTTP hardening (pre-auth body cap + header-first auth + TLS parity)     ← CRITICAL
        ├── CODEX-BF  server-authoritative write ACL (replace client-steerable view ACL)             ← HIGH
        ├── CODEX-BG  auth + connection hardening (login rate-limit/lockout, conn/message caps)       ← HIGH
        └── CODEX-BH  audit hash-chain HMAC keyed digest (tamper-evidence vs DB-write attacker)       ← HIGH (CFR21)

Tier 2 — Drivers (unblock the Phase-4 driver investment + the pre-1.0 hardware-soak gate)
        ├── CODEX-BI  gateway driver dispatch — the gateway currently only speaks Rockwell; the merged
        │             Modbus/OPC UA/MQTT/ADS drivers are UNREACHABLE and DriverSupervisor is dead code  ← HIGH (headline)
        ├── CODEX-BJ  DriverUpdate connection-state signal (one bad tag ≠ disconnect) — contract; land before BI wires more drivers
        ├── CODEX-BK  Rockwell rust-ethernet-ip 0.7 → 1.2 upgrade + typed writes (DINT/INT/SINT/LINT/REAL/LREAL)  ← HIGH (hardware-gate)
        ├── CODEX-BL  MQTT event-loop resilience (one poll error permanently freezes tags as healthy)   ← CRITICAL
        ├── CODEX-BM  ADS notification fan-out (MPMC channel steals ~50% of concurrent-sub samples)      ← CRITICAL
        ├── CODEX-BN  ADS write range-checking (silent integer truncation on write)                     ← HIGH
        ├── CODEX-BO  Modbus hostname/DNS + per-request timeout + bounds-checked decode                 ← HIGH
        ├── CODEX-BP  OPC UA correctness + security (dead-session, subscription leak, security off)     ← HIGH
        └── CODEX-BQ  Sparkplug B conformance (signed-int decode, DEATH stale-marking, command writes)  ← MEDIUM

Tier 3 — Gateway data integrity & robustness
        ├── CODEX-BR  atomic artifact writes (pre-remove defeats rename atomicity; no fsync)
        ├── CODEX-BS  memory-tag namespace + honest write results (unknown-driver writes phantom-succeed)
        ├── CODEX-BT  reconnect write-queue safety (stale operator setpoints replayed to PLC on reconnect — SCADA safety)
        ├── CODEX-BU  gateway lifecycle hardening (fatal accept, dropped RPC replies, no keepalive, download leak)
        └── CODEX-BV  config-driven data paths (historian/alarm sqlite hardcoded into cwd)

Tier 4 — Scalability (before the performance-baseline milestone)
        ├── CODEX-BW  tag-engine scaling (single global lock, immortal slots, task-per-(conn×tag))
        └── CODEX-BX  gateway service-context refactor (process-global singletons, 900-line dispatch, unordered audit spawn)

Tier 5 — Frontend
        ├── CODEX-BY  runtime error boundaries (one throwing widget blanks the whole HMI)               ← HIGH
        ├── CODEX-BZ  gateway client robustness (zombie sockets, silent write drops, dangling requests)  ← HIGH
        ├── CODEX-CA  alarm subscription lifecycle (no alarm.unsubscribe; re-subscribe churn)            ← HIGH
        ├── CODEX-CB  runtime subscription + render architecture (whole-tree re-render per tag update)   ← HIGH (refactor)
        ├── CODEX-CC  designer editing integrity (script-source cross-contamination + uneditable JSON props)  ← HIGH
        └── CODEX-CD  component theming + accessibility (default pack ignores theme vars; generic aria-labels)

Tier 6 — SCADA feature parity (integrator blockers vs Ignition / FactoryTalk)
        ├── CODEX-CE  alarm deadband/hysteresis + on/off delay (analog-alarm chattering)
        ├── CODEX-CF  alarm ack state-machine fix (ack leaves a normalized alarm stuck Active)
        ├── CODEX-CG  historian read-path SQL downsampling (loads all raw rows before downsampling)
        └── CODEX-CH  audit query SQL pushdown (full-table load + Rust-side filter, indexes unused)

Tier 7 — Docs / ops / CI (honesty + operability)
        ├── CODEX-CI  docs-vs-code reconciliation (docs claim PyO3 + system.alarm/db/http + sandbox + UDTs the code lacks)
        ├── CODEX-CJ  gateway observability (/health + /metrics + optional structured logging)
        ├── CODEX-CK  CI hardening (single-platform, no --all-features, no cargo-deny, no release workflow)
        └── CODEX-CL  scripting system.* library gap (implement-or-defer the documented RPC surface + triggers; pairs with CI)
```

Cross-task coordination flagged in the briefs: **BW + BX** both edit `crates/gateway/src/server.rs`; **BZ + CA** both extend the `crates/protocol` + `packages/protocol-ts` pair (non-overlapping additions, same-commit sync); **BC + BE** both touch the backup restore path (BC owns manifest identifier validation, BE owns transport/resource hardening); **BH** reuses **BD**'s secret-provisioning shape; **CI + CL** are paired (CL decides implement-vs-defer per item, CI keeps the docs honest to that decision). Incidental finding for a future cleanup: `Cargo.toml` declares `pyo3 = "0.22"` as a workspace dep that no crate consumes (scripting is a subprocess, not PyO3).

Feature-parity sweep tasks are **independent** of each other (except for AN→AP audit coverage and AO→AT CSS-variable sharing noted in the briefs); they can run in any order Codex prefers. Priority order in the table reflects business value, not dependency order.

CI hygiene tasks AU + AV merged at `4e9bc9b`. Each unblocked the job's setup gate but exposed a downstream failure (AW = gateway WS test timeout; AX = component-library module resolution). Neither downstream failure is an AU/AV regression — both were masked by the earlier setup-step breakage. AW + AX are the actual "CI fully green" gate.

**🎉 Phase 4 v1.0 feature ladder complete.** AE (audit log), AF (backup/restore), AI (historian import closeout) all merged. Driver slice was AG (toolchain) → AD (ADS validation) → AH (ADS native notifications). Component slice closed at AC (25 components). Remaining v1.0 closeout items live outside the agent-task ladder: plugin SDK, performance baseline, pre-1.0 hardware-validation 24h soak gate, plus the v1.1 polish list flagged across AE/AF/AI verdicts (SessionExpired hook, wiki/protocol/* pages, SQL-side audit-log filter pushdown, alarm-journal merge dedupe, designer session disconnect on replace).

**Quality sweep (review pass 2 — Rust 1.95 + edition 2024 idioms vs tokio/axum/ripgrep).** CODEX-AG was the mechanical edition migration; this is the deferred idiom modernization, sliced four ways. CODEX-AJ (Tier 1, runtime-health gaps) leads; AK/AL/AM open after AJ merges to avoid blurring correctness fixes with surface cleanup.

## Phase 5 — Manufacturing platform foundation and first modules

**Gate:** CN is a v1.0 cross-platform contract task. CM is documentation-only.
CO..CY do not begin until the relevant v1.0 hardening, plugin SDK, performance,
and hardware-validation gates are complete. `open` means the brief exists; it
does not pull post-1.0 implementation into the current release.

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-CM | Evidence-based manufacturing platform roadmap and task program | codex | submitted | 2026-09-01 codex [gpt-5] | [`CODEX-CM-manufacturing-platform-plan.md`](tasks/CODEX-CM-manufacturing-platform-plan.md) |
| CODEX-CN | Linux Designer v1 parity and three-platform release proof | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CN-linux-designer-v1-parity.md`](tasks/CODEX-CN-linux-designer-v1-parity.md) |
| CODEX-CO | Module SDK and module-owned storage contract | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CO-module-sdk-storage-contract.md`](tasks/CODEX-CO-module-sdk-storage-contract.md) |
| CODEX-CP | Versioned project templates and deterministic scenario engine | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CP-project-templates-scenario-engine.md`](tasks/CODEX-CP-project-templates-scenario-engine.md) |
| CODEX-CQ | Optional equipment model and manufacturing fact/projection layer | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CQ-equipment-manufacturing-facts.md`](tasks/CODEX-CQ-equipment-manufacturing-facts.md) |
| CODEX-CR | Server-authoritative machine command gateway and action ACLs | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CR-command-gateway-action-acl.md`](tasks/CODEX-CR-command-gateway-action-acl.md) |
| CODEX-CS | Manufacturing component pack and interactive line layout | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CS-manufacturing-components-line-layout.md`](tasks/CODEX-CS-manufacturing-components-line-layout.md) |
| CODEX-CT | Production monitoring, targets, machine state, and timeline module | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CT-production-state-timeline.md`](tasks/CODEX-CT-production-state-timeline.md) |
| CODEX-CU | Alarm maturity and equipment-aware fault analytics | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CU-alarm-fault-analytics.md`](tasks/CODEX-CU-alarm-fault-analytics.md) |
| CODEX-CV | Reproducible report definitions, rendering, and export framework | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CV-reporting-framework.md`](tasks/CODEX-CV-reporting-framework.md) |
| CODEX-CW | Versioned downtime policy, corrections, and recalculation module | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CW-downtime-policy-recalculation.md`](tasks/CODEX-CW-downtime-policy-recalculation.md) |
| CODEX-CX | Quality-aware hierarchical OEE module | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CX-oee-module.md`](tasks/CODEX-CX-oee-module.md) |
| CODEX-CY | Default-bundled fictional manufacturing demo and product-ready release gate | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CY-manufacturing-demo-integration.md`](tasks/CODEX-CY-manufacturing-demo-integration.md) |
| CODEX-CZ | Demo marketing capture pipeline and public content kit | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-CZ-demo-marketing-content-kit.md`](tasks/CODEX-CZ-demo-marketing-content-kit.md) |
| CODEX-DA | Generic Rockwell demo PLC profile, controller package, and hardware proof | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-DA-rockwell-demo-plc-profile.md`](tasks/CODEX-DA-rockwell-demo-plc-profile.md) |
| CODEX-DB | Omarchy Linux first public demo deployment and video proof | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-DB-omarchy-first-demo-video.md`](tasks/CODEX-DB-omarchy-first-demo-video.md) |
| CODEX-DE | Website demo hub and shipped-product visual story | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-DE-website-demo-hub-visual-story.md`](tasks/CODEX-DE-website-demo-hub-visual-story.md) |
| CODEX-DF | Website discoverability, social previews, and launch metadata | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-DF-website-discovery-social-metadata.md`](tasks/CODEX-DF-website-discovery-social-metadata.md) |
| CODEX-DH | Atomic AGPL core and MPL protocol repository transition | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-DH-agpl-mpl-repository-transition.md`](tasks/CODEX-DH-agpl-mpl-repository-transition.md) |
| CODEX-DI | AGPL/MPL public documentation, website, and transition communication | codex | open | 2026-09-01 codex [gpt-5] | [`CODEX-DI-license-docs-website-transition.md`](tasks/CODEX-DI-license-docs-website-transition.md) |

### Phase 5 dependency graph

```text
v1.0 hardening + plugin SDK + performance + hardware gate
  ├── CN  three-platform Designer proof (v1.0)
  └── CO  module/storage contract
        ├── CP  project templates + deterministic scenarios
        ├── CQ  equipment + semantic facts/projections
        │     ├── CR  machine command gateway (also depends on BF/audit)
        │     ├── CS  manufacturing component pack + line layout
        │     └── CT  production + state timeline
        │           ├── CU  alarm/fault analytics
        │           ├── CV  reporting framework
        │           └── CW  downtime policy/recalculation
        │                 └── CX  OEE
        └─────────────────────────────── CY demo integration (incremental)
                                          ├── CZ marketing capture/content kit
                                          ├── DA Rockwell demo PLC profile + hardware proof
                                          └── CN + CZ + DA -> DB Omarchy first-video proof

DC website positioning/claim reconciliation (may proceed before the demo ships)
  └── CY + CZ + CN -> DE website demo hub/visual story
                        └── CZ -> DF discovery/social/launch metadata
                              (DB required only for published-video metadata)

DG ownership/provenance + licensing policy gate
  └── DJ remove unused EPL-derived Sparkplug schema
        └── DH atomic AGPL core/MPL protocol transition
              └── DI repository docs + website communication
              (DH + DI publish in one coordinated transition window)
```

Recipes/maintenance remain planned candidates after CR/CO. Part/process data,
SPC, energy, equipment health, traceability, Andon, and MES remain
exploratory/deferred and intentionally have no executable board task yet.

## Phase 3 — Core SCADA features

**🎉 Phase 3 code-complete.** All seven tasks merged (O, Q, S, R, P, T, U) plus the V closeout follow-up. The demo HMI now has the full SCADA stack: alarms + history + auth + alarm UI + trends + scripting + Monaco script editor with live error surfacing. Awaiting **manual-smoke validation** of the full 19-step checklist in [`apps/designer/README.md`](../../apps/designer/README.md) (covers Phase 2 and Phase 3 together) before tagging `v0.3.0`.

*(no open Phase 3 tasks)*

## Phase 2 — Designer MVP

**🎉 Phase 2 code-complete.** All Required-slice tasks (I/J/K/L/M + the CODEX-N follow-up) merged. Awaiting **manual-smoke validation** of the 10-step checklist in [`apps/designer/README.md`](../../apps/designer/README.md) before tagging `v0.2.0`. The Stretch slice (visual canvas, drag/drop, snap-to-grid, undo/redo, theme editor UI, four more components) is deferred to Phase 4 per the de-risked plan.

*(no open Phase 2 tasks)*

## Phase 1 — Vertical slice (PLC tag in browser, simulator-backed)

**🎉 Phase 1 complete.** Released as [`v0.1.0`](https://github.com/sergiogallegos/OpenWebHMI/releases/tag/v0.1.0). Exit criterion met: a PLC tag from the Rockwell driver, sourced from `examples/sim-rockwell`, updates live in the browser through the gateway, with quality propagation on simulator restart. Hardware validation remains gated to pre-1.0 (per `docs/roadmap.md`).

*(no open Phase 1 tasks)*

## Done

| Id | Title | Owner | Merge commit | Phase |
|---|---|---|---|---|
| CODEX-A | `packages/protocol-ts` — TS protocol types | codex | `75ccb9c` | 0 |
| CODEX-B | `crates/gateway` — WS gateway binary with sim provider | codex | `75ccb9c` | 0 |
| CODEX-C | `apps/runtime-web` — React+Vite client | codex | `75ccb9c` | 0 |
| CODEX-D | `.github/workflows/ci.yml` — Phase 0 CI | codex | `75ccb9c` | 0 |
| CODEX-E | `crates/driver-api` — Driver trait + types + supervisor | codex | `e19a3c2` | 1 |
| CODEX-G | `examples/sim-rockwell` — EtherNet/IP simulator harness | codex | `e19a3c2` | 1 |
| CODEX-F | `crates/driver-rockwell` — wrap `rust-ethernet-ip` 0.7.x | codex | `bc2d568` | 1 |
| CODEX-H | Phase 1 wire-up — gateway loads driver-rockwell, runtime-web shows PLC tag | codex | `ca481d4` | 1 |
| CODEX-I | `crates/project-store` — gateway-side project storage with versioning | codex | `24c1ac7` | 2 |
| CODEX-J | View schema + protocol additions for view-tree authoring | codex | `24c1ac7` | 2 |
| CODEX-K | `packages/component-library` — 6 essential components | codex | `24c1ac7` | 2 |
| CODEX-L | `apps/runtime-web` — load views from gateway, render via component library | codex | `921e3d9` | 2 |
| CODEX-N | `tag.write` end-to-end — protocol message + gateway routing to driver | codex | `f40b780` | 2 |
| CODEX-M | `apps/designer` — Tauri shell, project explorer, form-based view editor | codex | `bc5bd38` | 2 |
| CODEX-O | `crates/historian` — tag time-series storage + read API | codex | `9db711e` | 3 |
| CODEX-S | `crates/auth` — local users + roles + JWT sessions + per-view ACLs | codex | `7eff30a` | 3 |
| CODEX-Q | `crates/alarm-engine` — definitions + state machine + journal | codex | `ff56780` | 3 |
| CODEX-R | `AlarmTable` component + designer alarm config | codex | `29be0e9` | 3 |
| CODEX-P | `Trend` component — multi-pen historical + live chart | codex | `151afdb` | 3 |
| CODEX-T | `crates/scripting` — CPython 3.11+ host + worker subprocesses + system.* RPC | codex | `f8b74a9` | 3 |
| CODEX-V | Route `system.tag.write` through per-driver write queue | codex | `9db307e` | 3 |
| CODEX-U | Designer script editor — Monaco + Python syntax + system.* stubs | codex | `cfa1cc0` | 3 |
| CODEX-X | `crates/driver-modbus` — Modbus TCP + RTU client driver | codex | `16f11bb` | 4 |
| CODEX-W | `crates/driver-opcua` — OPC UA client driver | codex | `6a8c2e2` | 4 |
| CODEX-Y | `crates/driver-mqtt` — MQTT (generic + Sparkplug B) driver | codex | `6a8c2e2` | 4 |
| CODEX-AA | Close v1 scope gaps in `driver-mqtt` (TLS + WS + json_path + binary BE) | codex | `560227e` | 4 |
| CODEX-AB | Component library batch 2 — 8 new components (Gauge, ProgressBar, Slider, Dropdown, ToggleSwitch, Button, MultiState, AlarmBanner) | codex | `3e88046` | 4 |
| CODEX-AC | Component library batch 3 — 9 new components (Tabs, Modal, DataGrid, BarChart, PieChart, Card, Spinner, Divider, Stepper); v1 25-component target met | codex | `6bff505` | 4 |
| CODEX-Z | `crates/driver-ads` — Beckhoff TwinCAT (ADS) client driver; implementation-merged but not production-validated (see CODEX-AD) | codex | `f34ebd6` | 4 |
| CODEX-AG | Workspace toolchain modernization — pin Rust 1.95.0 + edition 2024 | codex | `9a96871` | 4 |
| CODEX-AD | ADS validation hardening — TwinCAT-router FFI backend + hardware-validated against CX-23F092 | codex | `54ec808` | 4 |
| CODEX-AH | Native ADS device notifications via TcAdsDll FFI — closes AD's polling regression | codex | `2be075e` | 4 |
| CODEX-AE | `crates/audit-log` — SQLite security event journal + query/subscribe wire protocol + gateway hooks | codex | `83aac91` | 4 |
| CODEX-AF | `crates/backup` — project export/import + gateway HTTP side-channel; historian/alarm import deferred to CODEX-AI | codex | `83aac91` | 4 |
| CODEX-AI | Historian + alarm-journal import path in `crates/backup` — closes AF's v1.0 gap | codex | `79e0ec4` | 4 |
| CODEX-AJ | Async hygiene — blocking SQLite spawn_blocking, coordinated shutdown, scripting fan-in | codex | `8022426` | 4 |
| CODEX-AK | Tokio handle ergonomics — AbortHandle map sweep, cancel-safety docs, per-connection drain | codex | `3bde658` | 4 |
| CODEX-AL | API polish — TagPath + DriverId newtypes, #[non_exhaustive] sweep, ScriptHost cheap-clone | codex | `79ee864` | 4 |
| CODEX-AU | ci: install Tauri Linux build deps so the Rust job's clippy step succeeds | codex | `4e9bc9b` | 4 |
| CODEX-AV | ci: resolve pnpm version conflict (workflow vs package.json packageManager) | codex | `4e9bc9b` | 4 |
| CODEX-AQ | Historian capacity + retention callout — docs, plus optional retention-policy MVP | codex | `411f449` | 4 |
| CODEX-AR | Raspberry Pi deployment guide — hardware spec, build, systemd, known limits | codex | `411f449` | 4 |
| CODEX-AY | ci: install Tauri Linux build deps in the Node job too (pnpm build step runs tauri build) | codex | `7505a62` | 4 |
| CODEX-AW | tests: stabilize gateway WS script-event integration test (websocket_gateway_forwards_script_events_by_project timeout) | codex | `cc2e1f7` | 4 |
| CODEX-AX | packages: fix component-library typecheck — cannot resolve @openwebhmi/protocol | codex | `47da4ea` | 4 |
| CODEX-AS | Widget export/import between projects — single-widget JSON round-trip | codex | `a225aa7` | 4 |
| CODEX-AO | Designer Theme Editor UI — pulled deferred v0.2 Stretch item forward | codex | `428a9cf` | 4 |
| CODEX-AT | Material Design widget pack — demo subset proving theme-pack architecture | codex | `428a9cf` | 4 |
| CODEX-AN | audit-log CFR21 Part 11 framing + tamper-evident hash chain | codex | `66d9e76` | 4 |
| CODEX-AM | Stdlib + deps modernization — thiserror = "2", single format-capture, missing-docs lint consistency | codex | `823d7a1` | 4 |
| CODEX-AZ | ci: AppImage bundling fails with 'failed to run linuxdeploy' (apps/designer tauri build) | codex | `bd3d006` | 4 |
| CODEX-BA | website refresh — surface AN/AO/AS/AT/AQ/AR features on openwebhmi.com landing + docs + download pages | codex | `a7ba50a` | 4 |
| CODEX-DC | Website positioning and shipped-versus-planned claim reconciliation; interactive browser smoke deferred | codex | `f635f4a` | 4 |
| CODEX-DG | AGPL/MPL transition specification, ownership audit, CLA, and policy gates; attorney review gates DH | codex | `f635f4a` | 4 |
| CODEX-DJ | Remove unused EPL-derived Sparkplug schema and reconcile provenance docs | codex | `f635f4a` | 4 |

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
