---
id: CODEX-BM
title: ADS notification fan-out fix — stop MPMC sample theft across subscriptions
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BM — ADS notification fan-out fix

## Brief

> **CRITICAL.** `ads::Client::get_notification_channel()` (ads 0.4.4) hands back a **clone of one crossbeam MPMC receiver** — every clone competes for the *same* stream, and each notification is delivered to exactly one receiver. The ADS driver spawns a fresh receiver thread per `subscribe_symbols` call, each with its own `entry_map`, each `continue`-ing on handles it doesn't recognize. With two live subscriptions — or the normal resubscribe overlap, where the old thread lingers until one more failed `send` — notifications are handed out round-robin and roughly half are silently eaten by the wrong thread. Replace the N-racing-threads design with a **single demultiplexing thread** that owns the one notification channel and routes each sample to the correct per-handle sink via a shared registry. Register/deregister handles cleanly on subscribe/unsubscribe and across resubscribe, with no window where a sample can be dropped.

### Goal

All notification samples for every live ADS subscription reach their subscriber. A second concurrent subscription does not steal the first's samples; a resubscribe does not drop samples during the handover. Exactly one thread ever calls `receiver.recv()` on the crate's notification channel; that thread demultiplexes by `sample.handle` through a shared `handle → sink` registry.

### Context to read first

- `crates/driver-ads/src/driver.rs`:
  - `RealAdsClient::subscribe_symbols`, lines 110-167 — `get_notification_channel()` at 125, per-call `entry_map`/`handles` build (119-140), and the per-call `std::thread::spawn` recv loop at 143-156 that `continue`s on an unknown handle (146-148) and `return`s only after a `send` fails (151-153).
  - `SubscriptionGuard`, lines 330-367 — holds `handles` and, on `Drop` (349-367), spawns a thread to `delete_notification` each handle. Note how deregistration currently races the recv thread's lifetime.
  - `AdsClientLike::subscribe_symbols` trait method, lines 31-37, and the `MockAdsClient` test double, lines 533-553 — the mock sends updates directly and returns a `SubscriptionGuard` with no client; the new design must keep this test seam working.
  - `update_from_result`, lines 470-489 — how a decoded sample becomes a `DriverUpdate`.
- The `ads` 0.4.4 notification API: `Client::get_notification_channel`, `Device::add_notification` / `delete_notification`, `notif::Handle`, `notif::Sample { handle, data }`, and `Notification::samples()`. **Confirm** that `get_notification_channel` returns a clone of a shared receiver (MPMC) and that there is exactly one underlying channel per `Client` — this is the crux; the fix is wrong if the crate actually gives an independent channel per call.
- Sibling drivers for the "one demux owner + per-subscription sink" shape: `crates/driver-opcua/src/driver.rs` `SubscriptionForwarder` (one callback fans to a per-subscription `mpsc`) and the ADS `GuardedUpdateStream` (369-383) that already ties stream lifetime to a guard.

### Files to create / modify

1. **Modify** `crates/driver-ads/src/driver.rs`:
   - Introduce a single long-lived demultiplexer that owns the one `get_notification_channel()` receiver and a shared registry mapping `notif::Handle → mpsc::UnboundedSender<DriverUpdate>` (plus the per-handle `SubscriptionEntry` needed to decode). The demux thread is started once per connected `RealAdsClient` (lazily on first subscribe, or at connect), not once per subscription.
   - `subscribe_symbols` registers its handles in the shared registry (handle → this subscription's sink + entry) and returns a `SubscriptionGuard` whose `Drop` both `delete_notification`s the handles **and** removes them from the registry — closing the overlap window so no sample is routed to a dead subscription.
   - The demux loop routes each `sample` to the sink registered for `sample.handle`; unknown/removed handles are dropped with a debug log (expected briefly during teardown), never by a thread that "isn't mine".
   - Guarantee no drop window on resubscribe: new handles register before old ones deregister, or the registry swap is atomic per handle, so a sample in flight is never orphaned.
2. **Keep** the `AdsClientLike` trait contract and `MockAdsClient` behavior intact — the mock still delivers updates through the returned guard/sink without a real channel. Adjust the trait/guard types only as far as the demux registry requires, and document any signature change in the Codex log.
3. **Add tests** to the existing `#[cfg(test)] mod tests` in `crates/driver-ads/src/driver.rs` (do not create a new test file).

### Behavior

- One recv thread per connected client; N subscriptions share it via the registry.
- Two concurrent subscriptions each receive **all** of their own samples — no round-robin theft.
- Resubscribe (drop old guard, create new) loses no samples in the handover.
- Dropping a `SubscriptionGuard` deregisters its handles from the registry and deletes the ADS notifications; a later sample for a stale handle is a no-op debug log, not a mis-route.
- No `unwrap`/`expect`/`panic` added on the production recv/route path (the existing `expect("len")` in `symbols.rs` is out of scope here; see CODEX-BN).

### Test requirements

- **Fan-out test (the load-bearing one)**: exercise the demux with two registered handle→sink pairs and a sequence of samples addressed to both handles; assert each sink receives exactly its own samples and none are lost. This must fail against the current per-thread `entry_map`+`continue` design — i.e. reproduce the round-robin theft with the pre-fix code path (via the FFI-router/mock seam or a directly-driven demux unit) and show the fix delivers 100%.
- **Resubscribe-no-drop test**: register handles, hand a sample mid-handover while swapping guards, assert delivery.
- **Deregistration test**: after a guard drops, a sample for its handle routes nowhere (no panic, no delivery to a surviving subscription).
- Tests must be deterministic — drive the demux with an injected sample source (a channel the test feeds), not a live PLC and not `sleep()`. Use channels/`JoinHandle::join` for synchronization. No hardcoded ports (n/a here, but no wall-clock waits either).
- Full matrix clean: `cargo test -p openwebhmi-driver-ads`, workspace `clippy -D warnings`, `fmt --check`.

### Acceptance criteria

- [ ] Exactly one thread calls `receiver.recv()` on the ADS notification channel per connected client.
- [ ] Two concurrent subscriptions each receive all their samples; the fan-out test fails pre-fix and passes post-fix.
- [ ] Resubscribe handover drops no samples; guard `Drop` deregisters from the registry and deletes notifications.
- [ ] `MockAdsClient` / `AdsClientLike` test seam still works; any trait/guard signature change is documented in the Codex log with rationale.
- [ ] No `sleep()`/wall-clock waits in tests; deterministic sample injection.
- [ ] No new `unwrap`/`expect`/`panic` on production paths; new suppressions use `#[expect(...)]` with a reason. Convert the incidental `#[allow(dead_code)]` on `SubscriptionGuard::backend` (line 338) to `#[expect(dead_code, reason = "…")]` if it is still touched.
- [ ] Codex log confirms, from the ads 0.4.4 source/docs, that `get_notification_channel` clones one MPMC receiver (the premise of the bug).

### Out of scope

- ADS write range-checking and ULINT overflow — that is CODEX-BN.
- The Windows `twincat_router` backend's notification path (`connect_twin_cat_router`) beyond keeping it compiling; if it shares the same channel model note it, but the FFI backend gets its own brief if it needs the same fix.
- Reconnect-on-connection-loss for ADS (the driver has no reconnect today) — separate concern; this brief is strictly about sample routing across live subscriptions.
- Changing `poll_rate_ms` / notification attribute semantics.

### Risks / gotchas

- **Verify the MPMC premise first.** The entire bug rests on `get_notification_channel` returning a clone of one shared receiver. Read the ads 0.4.4 source; if per-call channels are actually independent, stop and re-scope (append a question to the Codex log — this changes the fix).
- **`notif::Handle` as a map key.** Confirm `Handle` is `Ord`/`Hash` (the current code uses it in a `BTreeMap`). The registry needs the same key type; reuse it.
- **Teardown ordering.** The demux thread must outlive individual subscriptions but end when the client disconnects. Decide the demux thread's lifetime explicitly (tie it to `RealAdsClient` drop / a shutdown signal) and make sure `recv()` unblocks on shutdown rather than leaking a thread.
- **Don't reintroduce a per-subscription recv thread as a "small optimization".** One owner of the channel is the invariant; a second `recv()` caller re-creates the theft. If you deviate, state why.
- **Copy the neighbor:** the OPC UA `SubscriptionForwarder` already models "one notification source → per-subscription mpsc sink". Match that shape rather than inventing a new synchronization scheme.

## Codex log

## Claude review

## Verdict
