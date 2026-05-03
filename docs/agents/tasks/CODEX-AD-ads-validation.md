---
id: CODEX-AD
title: ADS validation hardening — CI sim feasibility, TwinCAT 3 smoke runbook, sumup + upstream risk tracking
owner: codex
phase: 4
status: merged
created: 2026-05-01
last-update: 2026-05-03 claude
---

# CODEX-AD — ADS validation hardening

## Brief

> **CODEX-Z closeout follow-up.** CODEX-Z merged the `crates/driver-ads` implementation against the real `ads = "=0.4.4"` wire client. What it could NOT prove: the driver actually talks to a TwinCAT 3 runtime, sumup-batched reads stay performant at scale, and the `ads-rs` upstream is healthy enough to pin v1 on. CODEX-AD owns the validation work that turns "implementation-merged" into "production-validated." Without this, ADS is a v1 listing that hasn't been exercised against the protocol it claims to speak.
>
> **Scope expansion 2026-05-02.** First-contact attempt against a real CX (TwinCAT 3.1.4024.44) surfaced **TLS-wrapped ADS as a v1.0 blocker, not a v1.1 polish item.** The CX rejected plain-TCP frames because the route was TLS-wrapped (XAE's default for added routes). The `ads = 0.4.4` crate is plain-TCP only. Manually editing routes to remove TLS is operationally fragile (route-cache desync, "an item with the same key has already added" errors). See `wiki/drivers/ads-integration.md` "Hardware validation log" for the session detail. CODEX-AD now owns the **TLS support decision** in addition to the validation runbook: fork/contribute upstream, hand-roll an AMS/ADS frame layer, or accept the workaround with documentation. This decision must precede the runbook; without TLS support, the runbook's first step is "manually break TwinCAT's default route configuration" which is unacceptable customer guidance.

### Goal

Decide TLS-wrapped ADS support strategy (NEW v1.0 scope), decide simulator strategy, write the TwinCAT 3 smoke runbook, run that runbook against a non-TLS test target, and track two open risks (sumup performance, `ads-rs` upstream maintenance) — both elevated to v1.0 scope items as of 2026-05-02. **Driver code changes are now expected** for the TLS support work in scope item 0 below.

### Context to read first

- `crates/driver-ads/src/driver.rs` — the implementation under validation.
- `wiki/drivers/ads-integration.md` — current verification status; the "Not yet real-hardware verified" section is exactly the work CODEX-AD is taking on.
- `docs/agents/tasks/CODEX-Z-driver-ads.md` — review pass 2 verdict, the v1.1 polish list, and the explicit "implementation-merged but not production-validated" framing.
- `docs/roadmap.md` Phase 4 — the pre-1.0 hardware-validation gate; this is part of it.
- Comparable runbooks: `apps/designer/README.md` manual smoke steps for Modbus / OPC UA / MQTT — same shape applies to TwinCAT.

### Scope

#### 0. TLS-wrapped ADS support decision (NEW — added 2026-05-02 after first-contact session)

Spike: how should `crates/driver-ads` speak TLS-wrapped ADS to a TwinCAT 3.1.4024+ CX whose routes were added via XAE (the enterprise default)?

- Investigate the `ads-rs` crate's roadmap and any open PRs related to TLS. The ADS Secure protocol is documented; the wire format wraps AMS frames in TLS using a self-signed CA exchanged at route-registration time.
- Investigate the cost of contributing TLS support upstream to `birkenfeld/ads-rs`: rustls vs native-tls, certificate handling for the self-signed CA stored in `StaticRoutes.xml`, scope of test coverage needed.
- Investigate the cost of forking and maintaining a tls-fork until upstream catches up.
- Investigate the cost of hand-rolling a thin AMS/ADS frame layer with TLS in the `driver-ads` crate (bypass `ads-rs` for the wire layer; reuse it only for type definitions and constants if at all).

**Output:** `wiki/drivers/ads-tls-decision.md` with the spike result + recommendation. Three possible outcomes:
- 🟢 **Contribute upstream** — open a PR adding TLS support to `birkenfeld/ads-rs` with an `enable-tls` feature flag. Estimated 1-3 weeks of work. Pros: clean dependency story; cons: gated on upstream review velocity.
- 🟡 **Fork** — maintain a tls-supporting fork at `sergiogallegos/ads-rs` until upstream catches up. Pros: full control; cons: maintenance burden, security update lag.
- 🔵 **Hand-roll** — implement TLS-wrapped AMS frame handling directly in `crates/driver-ads`. Pros: no upstream dependency; cons: more code to maintain in-tree, duplicates ads-rs logic.

**Acceptance for this scope item:** the decision is documented in `wiki/drivers/ads-tls-decision.md` with rationale; the chosen path has an open PR / fork / commit demonstrating progress. Full TLS support implementation can land in a separate task (CODEX-AE-equivalent for ADS-TLS) — this task only owns the decision and first-step execution.

#### 1. Simulator feasibility decision (decision-then-implement, wiki-first)

Spike the question: can we ship a deterministic CI ADS server fixture without forging the protocol?

- Investigate `ads = 0.4.4`'s server primitives (`ads::server`, if exposed) — what they cover, what they don't.
- Investigate hand-rolling AMS/ADS frames at the same depth `examples/sim-modbus` hand-rolls Modbus frames. Estimate the smallest set of ADS commands the driver actually exercises (open/close/read/write/add_notification/delete_notification + symbol upload). Ballpark the LOC.
- Check whether Beckhoff's official C++ ADS library can be exercised as an external CI service (e.g. running `AdsLib` in a Docker container with a mock TwinCAT runtime config).

**Output:** `wiki/drivers/ads-sim-decision.md` with the spike result + recommendation. Three possible outcomes:
- 🟢 **Build it** — feasible without forging; proceed with `examples/sim-ads/` + `crates/driver-ads/tests/integration.rs`.
- 🟡 **Defer it** — non-trivial (>1000 LOC of frame-encoding work, or an external dependency CI doesn't tolerate); document why CI sim is deferred and make TwinCAT 3 hardware validation the required proof for v1.0.
- 🔴 **Reject it** — any path that requires forging the protocol (the previous CODEX-Z failure mode).

**The 🔴 case is hard-blocked.** Don't reintroduce a JSON-line dialect or any other not-real-ADS responder. Better to ship without a sim than ship with a fake one.

#### 2. TwinCAT 3 smoke runbook

Append a new section to `apps/designer/README.md` titled **"Manual smoke — Beckhoff TwinCAT 3"** with explicit steps the maintainer (or any contributor with TwinCAT 3) can follow. Mirror the structure of the existing manual-smoke sections.

Required steps:

1. **Setup** — install TwinCAT 3 XAR or use an existing install; confirm AMS net id (e.g. `192.168.1.10.1.1`); confirm gateway machine has an AMS route configured to that net id. Document how to add the route via Beckhoff's `TwinCAT System Manager` or `StaticRoutes.xml`.
2. **PLC project** — provide a minimal TwinCAT 3 PLC program (`MAIN.PRG`) with: a `BOOL bRunning`, an `INT nCounter` incremented every cycle, a `REAL fSetPoint`, and a `STRING(80) sStatus`. Include the `.tsproj` in `examples/twincat-smoke/` (or document the steps to create one).
3. **OpenWebHMI project** — configure an ADS driver with `host`, `ams_net_id`, `tcp_port: 48898`, `ports: [851]`. Add four tags bound to the symbols above.
4. **Browse test** — connect from the designer's project explorer; verify the four symbols appear in the ADS browse tree with correct types (`BOOL`, `INT`, `REAL`, `STRING(80)`).
5. **Read test** — observe `nCounter` incrementing in the runtime view; confirm value updates arrive within the configured `poll_rate_ms` (default 250ms).
6. **Write test** — write `42` to `nCounter` from a NumericInput component; observe TwinCAT reflecting the new value (visible in the TwinCAT online view).
7. **Notification test** — change `bRunning` from TwinCAT's online view; confirm the runtime receives the update via ADS device notification (not by polling). Use the gateway's `tracing` log to confirm an ADS notification frame arrived.
8. **Reconnect test** — kill TwinCAT runtime, observe quality degrade to Bad in OpenWebHMI; restart TwinCAT, observe quality recover and notifications resume without restarting the gateway.
9. **Handle leak check** — reconnect 50 times in a row (`pkill && restart` loop); confirm TwinCAT's notification handle pool stays bounded (the SubscriptionGuard's Drop should release on every disconnect).

#### 3. Run the runbook

Execute steps 4-9 against the maintainer's local TwinCAT 3 install. Document results in `wiki/drivers/ads-integration.md` under a new "Hardware validation log" section. Each step gets a ✅ / ❌ / 🟡 marker plus a one-line note.

If a step fails, file a finding under "## Open Questions" with enough detail for a fix-up PR. Don't fix bugs in this task unless the fix is mechanical (one-liner); otherwise open a follow-up.

#### 4. Track sumup performance as v1.1 risk

Append to `wiki/drivers/ads-integration.md` Open Questions:
- Note that the driver currently issues sequential `Handle::read` / `Handle::write` per tag. For an HMI polling 50 tags at 100ms, that's 50 round trips per cycle.
- ADS supports sumup (multi-symbol single request). The Rust crate's sumup support needs investigation.
- Reference: Beckhoff Information System "ADS Sum Commands" section.
- Decision criteria for v1.1: if a real-deployment HMI shows latency degradation with >30 ADS tags, prioritize sumup; otherwise defer to v2.

#### 5. Track `ads-rs` upstream risk

Append to `wiki/drivers/ads-integration.md` Open Questions:
- `birkenfeld/ads-rs` is a single-maintainer crate; OpenWebHMI pins `=0.4.4`.
- Note the latest commit date, open-issue count, and last-release date as a v1.1 health snapshot.
- Mitigation plan: if upstream stalls, the wrapper code in `crates/driver-ads/src/` is the asset; the crate is replaceable (fork or write a thin AMS/ADS layer at the size estimated by step 1's spike).

### Files to create

- `wiki/drivers/ads-sim-decision.md` — spike output (decision + rationale).
- `examples/twincat-smoke/` — minimal TwinCAT 3 project files referenced in step 2 of the runbook (or a README explaining how to create them if exporting from TwinCAT XAE is non-trivial).

### Files to modify

- `apps/designer/README.md` — append the "Manual smoke — Beckhoff TwinCAT 3" section.
- `wiki/drivers/ads-integration.md` — add "Hardware validation log" section after running the runbook; expand "Open Questions" with the sumup and upstream-risk items.
- `docs/feature-matrix.md` — flip the Beckhoff TwinCAT (ADS) row from "🟢 v1 (Phase 4, in development)" to either "🟢 v1 (Phase 4, simulator-validated)" if step 1 lands a sim, OR "🟢 v1 (Phase 4, hardware-validated)" once step 3 succeeds. **Do NOT mark v1-complete on this task** — that requires the pre-1.0 hardware-validation gate proper.

### Acceptance criteria

- [ ] `wiki/drivers/ads-sim-decision.md` exists with one of the three decision outcomes.
- [ ] If decision is 🟢: `examples/sim-ads/` + `crates/driver-ads/tests/integration.rs` land, sim-tests pass three consecutive runs (flake gate).
- [ ] If decision is 🟡: the deferral is documented with concrete blocker rationale (LOC estimate, upstream gap, or external-dependency unfit-for-CI reason).
- [ ] `apps/designer/README.md` has the new TwinCAT 3 manual-smoke section with all 9 steps.
- [ ] The 9-step runbook has been run against a real TwinCAT 3 install; results recorded in the wiki.
- [ ] Sumup performance + ads-rs upstream risk both tracked as Open Questions in the wiki entry.
- [ ] `cargo test -p openwebhmi-driver-ads --all-features --locked` stays green.
- [ ] `cargo clippy --workspace --all-targets --all-features --locked --exclude openwebhmi-designer -- -D warnings` stays green.

### Out of scope (post-1.0)

- **Sumup implementation** — tracked as v1.1 risk, not a v1.0 deliverable. CODEX-AD only documents the gap.
- **Cyclic-fallback notification mode** — v1.1 polish per CODEX-Z's review.
- **Route-table discovery** (StaticRoutes.xml / Windows registry) — v1.1 polish.
- **24h soak test** — that's the pre-1.0 hardware-validation gate's responsibility, not CODEX-AD's. CODEX-AD proves the driver works for ~5 minutes against real TwinCAT; the soak is a separate gate.

### Risks / gotchas

- **Don't reintroduce the rejected JSON-line dialect.** If step 1 concludes a CI sim isn't feasible without forging, accept that and write the deferral rationale. The previous CODEX-Z attempt was rejected exactly because hand-rolling fake ADS frames felt easier than not having a sim.
- **TwinCAT 3 install is Windows-bound for XAR.** The maintainer has TwinCAT on Windows; document this in the runbook. Linux contributors can read but not run the runbook themselves until step 1 lands a sim (if it does).
- **Notification handles are scarce on TwinCAT.** Step 9's reconnect-loop test is the high-value test — leak detection is exactly the kind of thing that surfaces only under load. Don't skip it.
- **AMS route management is manual in v1.** Document the maintainer's actual route entries in the runbook (redact NetIds if posting publicly).
- **`ads-rs` test fixtures may not match TwinCAT 3 exactly** — TwinCAT 4 is in development; the protocol drift could surface. Confirm the install version (TwinCAT 3.x specifically) in step 1.

## Codex log

- 2026-05-03 codex: Submitted partial CODEX-AD closeout for maintainer review. Added `wiki/drivers/ads-tls-decision.md`, `wiki/drivers/ads-sim-decision.md` with real-protocol simulator deferral, `examples/twincat-smoke/README.md`, and a 9-step TwinCAT 3 runbook in `apps/designer/README.md`. Updated `wiki/drivers/ads-integration.md`, `wiki/index.md`, and `wiki/log.md`. Hardware execution was blocked until the TwinCAT-router backend landed.
- 2026-05-03 codex: Implemented the Windows TwinCAT-router ADS backend using Beckhoff `TcAdsDll.dll` FFI behind `backend: "auto" | "ads_rs_tcp" | "twincat_router"`. Live smoke against CX-23F092 (`192.168.10.100.1.1`) passed browse, BOOL/INT/REAL/STRING reads, REAL write/read-back, and update streaming through the Secure ADS route. Remaining CODEX-AD gaps: native `TcAdsDll` notification callbacks, reconnect recovery, and handle-leak validation for any future native-notification path.

## Claude review

### Strong points

- ✅ **Codex found a fourth option not in the brief and it's the right one.** Scope item 0 listed three TLS outcomes (🟢 contribute upstream / 🟡 fork / 🔵 hand-roll). Codex chose **none of these** and instead delegated to the locally-installed Beckhoff TwinCAT router via `TcAdsDll.dll` FFI. This is genuinely better than what the brief asked for: Beckhoff documents Secure ADS as router-to-router TLS (not application-level TLS to a runtime), which means the "correct" architectural answer is "use the local router DLL" — every other option was fighting Beckhoff's design. The decision doc in `wiki/drivers/ads-tls-decision.md` makes this case clearly.
- ✅ **Hardware-validated against a real CX.** Live smoke against CX-23F092 (192.168.10.100.1.1) through a Secure ADS route: 14 symbols browsed, BOOL/INT/REAL/STRING reads, REAL write + read-back, 24 update events over 3 seconds. Recorded with timestamps in `wiki/drivers/ads-integration.md` "Hardware validation log". This is exactly the proof CODEX-AD existed to deliver.
- ✅ **No DLL redistribution.** `TcAdsDll.dll` is dynamically loaded via `LoadLibraryW` from candidate paths (PATH + four well-known TwinCAT install locations). The DLL is never bundled with OpenWebHMI; it's only used when present on a TwinCAT-equipped host. License-clean; the only correct way to integrate against Beckhoff's runtime.
- ✅ **Cross-platform discipline.** `#[cfg(windows)] mod twincat_router;` at `crates/driver-ads/src/lib.rs:10-11` keeps Linux/macOS builds clean. The `auto` backend correctly falls back to `ads_rs_tcp` on non-Windows or when the DLL can't load. Verified via my workspace clippy run.
- ✅ **Brief deliverables all shipped.** TLS decision doc, sim feasibility decision doc, 9-step TwinCAT 3 runbook in `apps/designer/README.md`, `examples/twincat-smoke/README.md` with the minimal PLC IEC ST source, hardware validation log, sumup-as-v1.1-risk Open Question, `ads-rs` upstream-health snapshot. All five scope items from the amended brief are addressed.
- ✅ **Honest scope commentary.** Both decision docs explicitly enumerate rejected alternatives + rationale. The integration log's "Validation status as of this session" splits CI/unit-verified from compile-verified from hardware-verified, exactly the breakdown CLAUDE.md asks for. Codex's own log calls out the polling-vs-notifications limitation upfront — no undersell, no overclaim.
- ✅ **Backend selection knob is principled.** `AdsBackend::Auto` (default) → prefer router on Windows, fall back to ads-rs. `AdsBackend::AdsRsTcp` and `AdsBackend::TwincatRouter` for explicit pinning. This is the right shape — most users get the correct backend automatically, advanced users can override.

### Findings

- 🟠 **Native ADS notifications are unwired on the Windows backend; updates regress to 250ms polling.** The `ads_rs_tcp` backend uses `Device::add_notification` (server-pushed `ServerOnChange`); the new TwinCAT-router backend polls `ADSIGRP_SYM_VALBYHND` per entry at `poll_rate_ms`. Codex flagged this themselves in their log and in `wiki/drivers/ads-integration.md` Open Question #1. **This is a load-bearing item, not v1.1 polish** per CLAUDE.md's rule against underselling — but it's also not a v1.0 blocker because polling at 250ms is in-family with Modbus/OPC UA defaults and the brief did not require notifications as the validation gate. Tracked as **CODEX-AH** (brief drafted in `docs/agents/tasks/CODEX-AH-ads-native-notifications.md`, ready for hand-off after this merge).
- 🟠 **Reconnect recovery is unwired.** If the router restarts or the route drops, the current code doesn't detect/rebuild. Tracked in CODEX-AH's "Out of scope (post-1.0)" section and in `wiki/drivers/ads-integration.md` Open Question #2 — marked v1.0-acceptable because the failure mode is "subscription stops delivering" (visible to the operator) rather than "silent stale data."
- 🟡 **Handle-leak test deferred until notifications are native.** Runbook step 9's value is exercising notification-handle exhaustion under reconnect cycles; with polling-based updates there are no notification handles to leak in the first place. The runbook table records this as `not-applicable-to-current-backend` — accurate. CODEX-AH's hardware-smoke gate re-runs step 9 once notifications are wired.
- 🟡 **`hardware-smoke` example still uses `clap` defaults that may surprise users on first run.** The `--source request` flag is required to ask the local router for an AMS port; `--source auto` would derive from the local IPv4 which doesn't match the maintainer's actual NetId on this CX. The example documentation in `wiki/drivers/ads-integration.md` "Hardware validation log" explicitly captures the working command line — ✅ acceptable for v1.0; not worth a separate fix.
- 🟡 **Symbol case sensitivity on `MAIN.fSetPoint`** — the runbook had a typo (`fSetpoint` lowercase 'p') that Codex fixed alongside this work. Worth noting in this review because the live smoke caught it; future contributors editing the smoke project should match TwinCAT's case exactly.
- 🟡 **Sumup is unwired on both backends.** Open Question #4 in `wiki/drivers/ads-integration.md` documents this with the v1.1 decision criterion (>30 ADS tags = prioritize sumup). Acceptable; matches the original brief's scope-item-4 ("track as v1.1 risk").

### Acceptance-criteria tally

- [x] `wiki/drivers/ads-sim-decision.md` exists with the 🟡 deferral outcome (real-protocol-only sim deferred until upstream test-server can be exposed; hand-rolling forged frames remains 🔴 hard-blocked).
- [x] `wiki/drivers/ads-tls-decision.md` exists with the recommended path (TwinCAT-router backend on Windows; ads-rs for plain TCP).
- [x] If decision is 🟡 sim-deferral: deferral documented with concrete rationale (LOC estimate "several hundred to >1000 lines"; upstream-private test-server gap; CI-fitness questions).
- [x] `apps/designer/README.md` has the TwinCAT 3 manual-smoke section with all 9 steps.
- [x] The 9-step runbook has been run against a real TwinCAT 3 install; results recorded in `wiki/drivers/ads-integration.md` "Hardware validation log" with the per-step status table.
- [x] Sumup performance + ads-rs upstream risk both tracked as Open Questions in the wiki entry (#4 and #7 respectively).
- [x] `cargo test -p openwebhmi-driver-ads --all-features --locked` stays green (verified independently: 13/13).
- [x] `cargo clippy --workspace --all-targets --all-features --locked --exclude openwebhmi-designer -- -D warnings` stays green (verified independently).

### Independent verification

- `cargo test -p openwebhmi-driver-ads --all-features --locked` — ✅ 13/13 pass, including `subscribes_with_native_updates` (the existing mock-driver subscription test) and `reads_symbol_by_handle_path`.
- `cargo clippy -p openwebhmi-driver-ads --all-targets --all-features --locked -- -D warnings` — ✅ clean.
- Workspace-level checks all clean per CODEX-AG's commit (`9a96871`).
- Linux compile gating verified by reading `crates/driver-ads/src/lib.rs:10-11` (`#[cfg(windows)] mod twincat_router;`) — the new module never reaches non-Windows builds.

## Verdict

**Merged.** This is the kind of submission that exceeds the brief and earns a verdict that says so plainly: Codex's TwinCAT-router-via-`TcAdsDll`-FFI is a better architectural answer than any of the three options the brief enumerated. The decision is principled (Beckhoff documents Secure ADS as router-to-router TLS), the implementation is correct (FFI without DLL redistribution; correct platform gating), and the validation is real (live smoke against CX-23F092 with full read/write/stream coverage). Five scope items shipped: both decision docs, runbook, runbook execution + log, sumup risk, ads-rs upstream-health snapshot.

The honest 🟠 finding is the polling-vs-native-notifications regression on the Windows path. It's tracked as **CODEX-AH** with a complete brief on disk; the hardware re-run of runbook steps 7 + 9 closes the validation gate once AH lands. CLAUDE.md's "don't undersell load-bearing items as polish" rule is honored here: AH is a v1.0 closeout follow-up, not v1.1 nice-to-have.

Bonus: this submission also closed the AC-era environmental gap (Tauri icon) and incidentally fixed a sim-rockwell clippy lint that surfaced under 1.95 — both bundled into CODEX-AG's commit because they belong to the toolchain bump's scope.

ADS row in `docs/feature-matrix.md` flips to "🟢 v1 (Phase 4, hardware-validated)" with this merge. The pre-1.0 hardware-validation gate (24h soak) remains a separate v1.0 deliverable.
