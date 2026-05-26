---
id: CODEX-AW
title: tests: stabilize gateway WS script-event integration test (websocket_gateway_forwards_script_events_by_project timeout)
owner: codex
phase: 4
status: submitted
created: 2026-05-25
last-update: 2026-05-25 codex [gpt-5]
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

## Verdict
