---
id: CODEX-BS
title: Explicit memory-tag namespace + honest write results for unknown drivers
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BS — Explicit memory-tag namespace + honest write results

## Brief

> A script write to an unknown or misspelled driver id silently succeeds. In `crates/gateway/src/script_writes.rs` (lines 62-72), `GatewayTagWriteSink::enqueue` treats any path with no known driver prefix — including a typo like `rockwel-1/Setpoint` — as a memory tag: it publishes to the `TagStore` with `Quality::Good` and audits `success: true`. The operator's script believes the PLC write landed while the gateway quietly minted a phantom memory tag. This is asymmetric with the WebSocket write path (`crates/gateway/src/server.rs:1387-1405`), which correctly rejects an unknown driver with `tag.write.unknown_driver`. Require an explicit memory-tag namespace (a `mem/` prefix) for intentional in-memory tags, and treat every other unknown-driver write as an error, matching the WS path. Separately, tighten audit honesty on the WS path: `tag.write` audits `success: true` the moment the command is *enqueued* (`server.rs:1407-1418`) even though the driver write is fire-and-forget and may never reach the PLC. Record the enqueue and the confirmed outcome distinctly.

### Goal

A script or WS write to a driver id that does not exist fails loudly (matching `tag.write.unknown_driver` semantics) instead of creating a phantom memory tag. Intentional in-process tags use an explicit `mem/` namespace and continue to publish to the `TagStore`. The tamper-evident audit log distinguishes "operator command accepted into the driver queue" from "driver confirmed the write" — it never records a write that never reached the PLC as an unqualified `success: true`.

### Context to read first

- `crates/gateway/src/script_writes.rs:60-97` — `GatewayTagWriteSink::enqueue` and `split_tag_path`. The two silent-success branches are lines 63-64 (no driver prefix → publish to store) and 68-72 (prefix present but driver unknown → publish to store). The audit helper is at lines 39-57.
- `crates/gateway/src/server.rs:1387-1405` — the WS unknown-driver rejection (`tag.write.unknown_driver`, audits `success: false`). This is the honest reference behavior the script path must mirror.
- `crates/gateway/src/server.rs:1407-1418` — the WS success audit that fires on *enqueue*, not on driver confirmation. `try_write` returns `Ok(())` when the command is accepted into the mpsc queue; the actual wire write happens later in `run_subscription_until_disconnect` (`crates/gateway/src/project.rs:260-278`) and is fire-and-forget with no ack back to the audit path.
- **`docs/agents/notes/python-tag-write-routing.md`** — required reading. `system.tag.write` routes through `GatewayTagWriteSink`; driver-prefixed paths dispatch to the per-driver write mpsc, memory-tag paths publish to the `TagStore` directly. Do **not** describe or introduce a "publish directly" bypass for driver-backed tags.
- `crates/gateway/src/script_writes.rs:99-143` — the existing test module (`memory_paths_publish_to_store`, `script_writes_append_audit_events`). Extend these; note both already write `mem/derived`, so the `mem/` prefix convention aligns with existing test intent.
- `crates/scripting` (`TagWriteError`) — the error surface `enqueue` returns; add an unknown-driver variant here if none fits.

### Files to create / modify

1. **Modify** `crates/gateway/src/script_writes.rs`:
   - In `enqueue`, replace the two silent-success branches:
     - A path with the `mem/` prefix (and a non-empty remainder) publishes to the `TagStore` with `Quality::Good` and audits `success: true` — this is the intentional memory-tag path.
     - A path with a driver prefix whose driver is **unknown** returns an error (a new/appropriate `TagWriteError` variant, e.g. `UnknownDriver`) and audits `success: false` with the reason, mirroring `server.rs:1387-1405`. It must **not** publish to the store.
     - A path with **no** `/` and no `mem/` prefix is likewise an error, not a phantom memory tag. (Decide and document the one edge: a bare path with no prefix — treat as error requiring explicit `mem/`, per the brief's "explicit namespace" contract.)
   - Keep driver-prefixed writes to *known* drivers routing through `driver.try_write` exactly as today.
   - No `unwrap`/`expect`/`panic!` on the production path.

2. **Modify** `crates/scripting/src/…` (wherever `TagWriteError` is defined) — add an `UnknownDriver(String)` variant (or equivalent) with a `Display` impl matching the WS `unknown driver '{id}'` wording. `#![deny(missing_docs)]` bar applies — document the variant.

3. **Modify** `crates/gateway/src/server.rs` — audit honesty for `tag.write`:
   - Record the enqueue outcome and the confirmed driver outcome **distinctly**. Preferred: on `try_write` success, audit the *acceptance* with an explicit qualifier (e.g. `success: false` is wrong here, so introduce a distinct signal — an "enqueued/pending" state, or defer the `success: true` audit until a driver ack), rather than an unqualified `success: true` that implies the PLC confirmed. Choose the minimal change that stops the audit from claiming a confirmed write when only the queue accepted it. If a confirmed-ack channel does not exist, audit the enqueue as a distinct event kind (pending) and leave the confirmed-success audit for when the driver ack path is wired — document the boundary in the Codex log and, if the ack path is genuinely absent, own it as a brief-scoped limitation rather than fabricating a confirmation.
   - Keep the existing `Busy`/`Closed` failure audits (`server.rs:1419+`).

4. **Modify** `crates/gateway/src/script_writes.rs` test module — extend the existing tests:
   - `mem/`-prefixed write publishes to the store and audits `success: true` (adapt `memory_paths_publish_to_store` / `script_writes_append_audit_events`, which already use `mem/derived`).
   - An unknown-driver write (e.g. `rockwel-1/Setpoint`) returns `Err(TagWriteError::UnknownDriver(..))`, does **not** publish to the store, and audits `success: false`.
   - A bare no-prefix path (no `mem/`, no `/`) errors and does not create a phantom tag.

### Behavior

- `mem/foo` → publishes to `TagStore`, `Quality::Good`, audit `success: true`.
- `rockwell/Setpoint` (known driver) → routes to the driver write queue (unchanged).
- `rockwel-1/Setpoint` (unknown driver) → `Err`, no store publish, audit `success: false`.
- `Setpoint` (no prefix) → `Err`, no phantom memory tag.
- WS `tag.write` audit no longer records an unqualified confirmed success on mere enqueue; enqueue-acceptance and driver confirmation are distinguishable in the audit log.

### Test requirements

- The unknown-driver regression test must **fail against the pre-fix code** (which currently returns `Ok(())` and publishes). Confirm fail-before / pass-after; note it in the Codex log.
- Tests observe the actual outcome (store state **and** audit event), per `python-tag-write-routing.md`: a driver-prefixed write test that asserts only the tag value is incomplete.
- No `sleep()` / wall-clock waits; no hardcoded ports.
- Full validation matrix clean (`cargo build`, `cargo clippy … -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`).

### Acceptance criteria

- [ ] `mem/`-prefixed script writes publish to the `TagStore` and audit `success: true`.
- [ ] Unknown-driver script writes return an error, do not publish, and audit `success: false` — symmetric with `server.rs:1387-1405`.
- [ ] Bare no-prefix paths error rather than minting a phantom memory tag.
- [ ] Known-driver script writes still route through `driver.try_write` (per `python-tag-write-routing.md`; no "publish directly" bypass).
- [ ] `TagWriteError::UnknownDriver` (or equivalent) added, documented, with WS-matching `Display`.
- [ ] WS `tag.write` audit distinguishes enqueue-acceptance from confirmed driver outcome; no unqualified `success: true` on mere enqueue. Any residual gap (missing driver-ack path) owned explicitly in the Codex log.
- [ ] Regression test fails pre-fix, passes post-fix.
- [ ] No `unwrap`/`expect`/`panic!` on production paths; full matrix clean.

### Out of scope

- Redesigning the driver write-ack protocol end-to-end. If no driver-confirmation channel exists, the audit-honesty fix is limited to *not* claiming confirmation on enqueue; a full ack round-trip is a separate brief. Own the boundary; don't fabricate a confirmation.
- Changing the memory-tag storage model or the `TagStore` API.
- Migrating existing memory-tag call sites in project configs to `mem/` (data migration is out of scope; this is the write-path contract).
- Touching the reconnect write-queue replay hazard — that is CODEX-BT's territory. Coordinate but don't overlap.

### Risks / gotchas

- **Asymmetry is the bug.** The WS path already does this right (`unknown_driver` error). The script path must reach parity, not invent a third behavior. Copy the WS neighbor's shape.
- **`python-tag-write-routing.md` is authoritative** on routing. A brief error once claimed `system.tag.write` "publishes directly" (CODEX-T, owned in CODEX-V's verdict). Do not reintroduce that framing. Driver-backed tags route through the per-driver mpsc; only `mem/` tags publish to the store.
- **Existing tests use `mem/derived`.** They already assume the memory namespace shape — adapt them rather than fighting them, but confirm they were previously passing *because* the prefix has no known driver, not because they exercised the intended `mem/` contract. The old code would have published `mem/derived` via the no-driver branch regardless; the new code publishes it via the explicit-`mem/` branch. Same observable result, correct reason.
- **Audit honesty vs. audit spam.** Don't emit two audit events per write if one event with an accurate state field suffices. Prefer the minimal, tamper-evidence-preserving change; the audit log's hash chain (`crates/audit-log`) must stay well-formed.
- **`#![deny(missing_docs)]`** on `crates/scripting` — the new error variant needs a doc comment.

## Codex log

## Claude review

## Verdict
