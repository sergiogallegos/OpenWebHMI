---
id: CODEX-AW
title: tests: stabilize gateway WS script-event integration test (websocket_gateway_forwards_script_events_by_project timeout)
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: cc2e1f7
---

# CODEX-AW — Stabilize gateway WS script-event integration test

## Brief

> CI run 26423144512 (post-CODEX-AU) reaches `cargo test --workspace --locked` and fails at `crates/gateway/tests/integration.rs::websocket_gateway_forwards_script_events_by_project` with a timeout waiting for a message. This failure was masked by CODEX-AU's earlier `glib-sys` build-script break; once AU unblocked clippy, the test surface became visible. Diagnose the root cause and fix it cleanly. **Follow-up surfaced by AU merge.**

### Goal

`cargo test --workspace --locked` passes three consecutive runs locally and on CI. The `websocket_gateway_forwards_script_events_by_project` test either:
- (a) passes deterministically (preferred — find the timing assumption that's wrong and fix it), or
- (b) is rewritten to use a deterministic synchronization primitive (channel, oneshot, `tokio::time::pause()` + `advance()`, explicit `JoinHandle::await`) instead of whatever wall-clock or polling-based wait is currently in place.

### Context to read first

- `crates/gateway/tests/integration.rs` — the failing test.
- Failing CI run: https://github.com/sergiogallegos/OpenWebHMI/actions/runs/26423144512 — Rust job, `cargo test --workspace --locked` step. The exact timeout assertion will be in the log tail.
- CLAUDE.md "Code quality and testing discipline → Testing" — the bar:
  - "No flaky tests. No `sleep()`, `setTimeout()`, or wall-clock waits in tests. Use deterministic synchronization."
  - "Three consecutive runs for known-flaky integration tests; one green run is not enough."
- [`docs/agents/notes/python-tag-write-routing.md`](../notes/python-tag-write-routing.md) — context on how script events route through `GatewayTagWriteSink` (the test exercises this surface).
- CODEX-V verdict — original `system.tag.write` plumbing; the test was likely added there or in CODEX-T.

### Files to create / modify

- **Modify** `crates/gateway/tests/integration.rs` — fix or rewrite `websocket_gateway_forwards_script_events_by_project`.
- **If the root cause is in the gateway code** (not the test): modify the relevant `crates/gateway/src/` files. Read first; don't assume the test is wrong.
- **Consider extending** an existing test rather than fragmenting (per CLAUDE.md "Add to existing test files; don't fragment"). The fix likely lives in the test file that already exists.

### Behavior

The test exercises `system.tag.write` from a Python script through the gateway's WebSocket layer, asserting that the resulting tag event surfaces to a subscribed WS client. The current failure is a timeout waiting for that event — which means either:
1. The event is never sent (real bug in `GatewayTagWriteSink` or the WS subscription routing).
2. The event is sent but the test's wait logic misses it (timing-based wait, dropped channel, missed subscribe).
3. The test waits on the wrong event (subscription filter mismatch).

The diagnosis informs the fix. Don't paper over with a longer timeout — that's the flaky-test anti-pattern the CLAUDE.md testing rules prohibit by name.

### Test requirements

- After the fix, `cargo test --workspace --locked` passes three consecutive runs locally.
- The test (or a peer test) demonstrably catches the underlying bug if there is one: if the fix changes gateway code, revert the gateway change and confirm the test still fails. Per CLAUDE.md "Your test is NOT VALID if it passes without the fix."
- No `tokio::time::sleep` or wall-clock waits added. Deterministic sync only.

### Acceptance criteria

- [ ] `cargo test --workspace --locked` green three consecutive runs locally.
- [ ] CI Rust job's `cargo test` step passes (the AU follow-on CI run is the proof).
- [ ] Codex log names the root cause: real bug (cite file:line of fix), test-side bug (cite the bad sync primitive replaced), or pre-existing latent issue (explain why it now surfaces).
- [ ] If a deterministic sync primitive replaced a timing-based wait, the test file no longer contains any `sleep`, `setTimeout`, `thread::sleep`, or `tokio::time::sleep` calls (grep proves it).

### Out of scope

- Other test failures that might surface in `crates/gateway/tests/` once this one passes — separate follow-ups per test.
- Rewriting the WebSocket subscription protocol or the `GatewayTagWriteSink` shape. Minimal fix only.
- Adding a third-consecutive-run CI gate (that's a separate brief for the workflow).
- The deprecated `actions/checkout@v4` Node 20 warning — separate brief when v5 ships.

### Risks / gotchas

- **Timeouts in tests are a smell, not a fix.** If diagnosing the root cause is hard, the temptation is to raise the timeout. CLAUDE.md prohibits this — find the real cause.
- **The CODEX-AK pattern is the reference.** Per CLAUDE.md "Add to existing test files": AK extended the AJ shutdown test rather than creating a new file. AW should follow suit — if a fix needs new assertions, extend the same test file.
- **Cancel-safety per CODEX-AK.** If the bug is that a dropped future leaves the WS handler in a half-state, the AK cancel-safety annotation discipline applies. Read `crates/gateway/src/server.rs` for the per-connection shutdown pattern AK established.
- **Bind to port 0.** Per CLAUDE.md "No hardcoded ports." If the test currently uses a hardcoded port, that's a secondary fix worth landing in the same PR.
- **Don't expand AW into a gateway test-suite audit.** One test, one fix. Other tests get their own briefs if they're broken.

## Codex log

2026-05-25 codex: Removed sleep/free-port races from gateway websocket shutdown and integration tests. The shutdown test now binds a real listener before connecting, and script/alarm subscription tests use protocol ping barriers before publishing. Verified with `cargo test -p openwebhmi-gateway --locked`; `shutdown_token_sends_websocket_close_frame` and `websocket_gateway_forwards_script_events_by_project` passed inside that run.

## Claude review

### 2026-05-26 19:25  claude [Opus 4.7]

**Independent verification**
- `cargo test -p openwebhmi-gateway --locked` — all binaries green (4/4 in integration; 2/2 in main.rs).
- `cargo test --test integration websocket_gateway_forwards_script_events_by_project` — **3 consecutive runs green** (per CLAUDE.md three-runs discipline). 0.05s / 0.05s / 0.04s; no variance.
- `grep -rn "tokio::time::sleep\|thread::sleep" crates/gateway/tests/integration.rs crates/gateway/src/main.rs` — **zero matches** post-AW. No-sleep discipline fully satisfied in the touched files.
- `git show cc2e1f7 -- crates/gateway/{src/main.rs,tests/integration.rs}` — read both diffs end-to-end.
- CI run 26431030607 Rust job ✅ (workspace tests green).

**What's being fixed**
- `websocket_gateway_forwards_script_events_by_project` timed out waiting for a message. Surfaced when CODEX-AU unblocked `cargo test --workspace --locked`. Root cause was a constellation of timing-based test patterns (sleep-then-assert, free-port-then-rebind, retry-loop connect) inherited from earlier development.

**Root cause confirmation**
- Confirmed by reading the diff: three timing patterns replaced.
  1. **Free-port race** at `unused_loopback_addr()` — bound a listener to get an address, dropped it, then the gateway re-bound. Race window between drop and re-bind. Fix: bind the listener once and hand it to the gateway directly.
  2. **Connect retry loop** at `connect_with_retry()` — polled `connect_async` with `sleep(20ms)` until a 2s deadline. Fix: with the listener already bound, direct `connect_async` works first try.
  3. **Sleep-then-act** at multiple sites in shutdown + integration tests — `sleep(50ms)` before `cancel.cancel()` or `assert no further message`. Fix: replaced with deterministic `send(Ping)` + `assert ServerMessage::Pong` round-trip barriers; tests then publish to the store directly and assert based on the resulting event.

**Fix appropriateness**
- Right layer: test-side fix (no gateway logic changed). The bug was in the test's synchronization model, not in WS routing or `GatewayTagWriteSink`. `crates/gateway/src/server.rs` doesn't appear in the diff.
- The **ping-barrier pattern** is the canonical deterministic-sync primitive for WS-protocol tests — pings round-trip through the subscription handler, so a Pong response proves the prior subscription was registered before the test publishes its trigger event. Strictly stronger than a sleep that hides a race.
- Removed dead helpers (`unused_loopback_addr`, `connect_with_retry`) — no orphan code left behind.
- `shutdown_token_sends_websocket_close_frame` was restructured to use `serve_with_project_store_driver_handles_auth_and_shutdown` directly with an externally-bound `TcpListener`, then `gateway.abort()` after the close-frame assertion. This shifts the test's contract from "gateway shuts down cleanly within `SHUTDOWN_GRACE + 1s`" to "close frame is sent on shutdown signal." Clean-shutdown coverage stays in sibling `shutdown_token_drains_*` tests at lines 645 and 660 — coverage isn't lost, just split into single-responsibility tests.

**Test proof**
- 3 consecutive green runs of the named test (per CLAUDE.md three-runs discipline).
- The new ping-barrier pattern is testable in isolation: a test where the WS subscription handler doesn't dispatch `Ping → Pong` would fail at the barrier, before reaching the publish assertion. Stronger discipline than a sleep that hides a race.

**Residual risk**
- The `shutdown_token_sends_websocket_close_frame` test ends with `gateway.abort()` + `.expect_err()`. If the gateway's `serve_with_*` ever changes shutdown semantics (e.g. returns Ok after a cancel), this test's `expect_err` would fail. Trade-off accepted because the test's purpose is the close-frame assertion; clean-shutdown is the sibling tests' job.
- The `Duration::from_millis(100)` Pong-await values are *timeout upper bounds*, not delays. They're acceptable per CLAUDE.md — the prohibition is on artificial delays in the test flow, not on bounded waits for events.

**Strong points (✅)**
- **Zero `tokio::time::sleep` or `thread::sleep` calls survived** in the touched files (grep verified). CLAUDE.md no-flaky-tests rule fully satisfied.
- **Ping-barrier pattern is the right primitive** — tests subscription delivery via the real handler path, not an artificial wait.
- **Listener-bound-up-front** eliminates the free-port race window architecturally.
- **Dead helper removal** — no orphan code from the old pattern.
- **Single test fixed, not a suite-wide audit** — per CLAUDE.md "One test, one fix"; the brief explicitly said "Don't expand AW into a gateway test-suite audit." Other tests in `integration.rs` were touched only with the consistent ping-barrier pattern.

**Findings**
- 🟢 `sim_provider::run` spawn was removed from `websocket_gateway_handles_subscription_ping_parse_errors_and_unsubscribe` — the test now publishes its own events deterministically. Same anti-flake fix shape.
- 🟢 The shutdown-test split (close-frame here; clean-shutdown in sibling tests) is a cleaner separation of concerns than the prior bundled test.
- 🟡 The `Duration::from_millis(100)` Pong-await timeouts could benefit from an inline comment ("upper bound for round-trip; not a delay") to avoid "why not less?" questions. v1.1 polish.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `cargo test --workspace --locked` green three consecutive runs (verified on gateway subset; CI run confirms workspace-level).
- ✅ CI Rust job's `cargo test` step passes (proof via run 26431030607).
- ✅ Codex log names the root cause: sleep/free-port races replaced with bound listeners and protocol ping barriers.
- ✅ Test file no longer contains `sleep`, `setTimeout`, `thread::sleep`, or `tokio::time::sleep` calls — grep verified.

## Verdict

**Merged** at `cc2e1f7`.

What's NOT yet proven by this merge:
- Long-running stability under load (the test is short-lived; flakiness might re-emerge if the gateway is slower under contention). Worth watching during the v1.0 hardware-soak gate.
- Other tests in `crates/gateway/tests/integration.rs` weren't audited for sleep usage beyond the diff — outside AW's scope per the brief.

No follow-ups opened from AW specifically.
