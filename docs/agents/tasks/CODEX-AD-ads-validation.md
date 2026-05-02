---
id: CODEX-AD
title: ADS validation hardening — CI sim feasibility, TwinCAT 3 smoke runbook, sumup + upstream risk tracking
owner: codex
phase: 4
status: open
created: 2026-05-01
last-update: 2026-05-01 claude
---

# CODEX-AD — ADS validation hardening

## Brief

> **CODEX-Z closeout follow-up.** CODEX-Z merged the `crates/driver-ads` implementation against the real `ads = "=0.4.4"` wire client. What it could NOT prove: the driver actually talks to a TwinCAT 3 runtime, sumup-batched reads stay performant at scale, and the `ads-rs` upstream is healthy enough to pin v1 on. CODEX-AD owns the validation work that turns "implementation-merged" into "production-validated." Without this, ADS is a v1 listing that hasn't been exercised against the protocol it claims to speak.

### Goal

Decide simulator strategy, write the TwinCAT 3 smoke runbook, run that runbook against the maintainer's local TwinCAT 3 install, and track two open risks (sumup performance, `ads-rs` upstream maintenance) as v1.1 scope items. **No driver code changes are expected** unless the validation surfaces a real bug — in which case fix it.

### Context to read first

- `crates/driver-ads/src/driver.rs` — the implementation under validation.
- `wiki/drivers/ads-integration.md` — current verification status; the "Not yet real-hardware verified" section is exactly the work CODEX-AD is taking on.
- `docs/agents/tasks/CODEX-Z-driver-ads.md` — review pass 2 verdict, the v1.1 polish list, and the explicit "implementation-merged but not production-validated" framing.
- `docs/roadmap.md` Phase 4 — the pre-1.0 hardware-validation gate; this is part of it.
- Comparable runbooks: `apps/designer/README.md` manual smoke steps for Modbus / OPC UA / MQTT — same shape applies to TwinCAT.

### Scope

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
2. **PLC project** — provide a minimal TwinCAT 3 PLC program (`MAIN.PRG`) with: a `BOOL bRunning`, an `INT nCounter` incremented every cycle, a `REAL fSetpoint`, and a `STRING(80) sStatus`. Include the `.tsproj` in `examples/twincat-smoke/` (or document the steps to create one).
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

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
