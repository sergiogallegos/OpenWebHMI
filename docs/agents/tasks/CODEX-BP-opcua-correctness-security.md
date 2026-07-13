---
id: CODEX-BP
title: OPC UA driver correctness + security — dead-session detect, subscription cleanup, security config, addressing
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BP — OPC UA correctness + security

## Brief

> **HIGH — four independent OPC UA defects, bundled because they share the driver file.** (a) `connect()` discards the `bool` from `session.wait_for_connection()` (async-opcua 0.18 returns `false` when the event loop terminated), so connect returns `Ok` on a dead session. (b) Subscriptions and monitored items are never deleted — `Session::delete_subscription` exists but is never called — so every resubscribe leaks a live server-side subscription over a 24h run until `BadTooManySubscriptions`; contrast Rockwell's `impl Drop for SubscriptionState` and ADS's `SubscriptionGuard`. (c) Security is hardcoded off (`SecurityPolicy::None`, `MessageSecurityMode::None`, `trust_server_certs(true)`, `verify_server_certs(false)`), none configurable — most production servers refuse None/None and Username creds transit unencrypted. (d) Addressing bugs: ns=0 subscription updates format without the namespace (NodeId `Display` omits ns 0) so `path_by_address` misses; `OpcUaAddress::parse` requires the `ns=` prefix (asymmetric with Display); and opaque base64 NodeIds with `=` padding are rejected by `id.contains('=')`.

### Goal

Connect fails cleanly when the session is dead. Subscriptions and monitored items are deleted when the subscription stream drops (a guard type), so long runs don't leak server subscriptions. Security policy, message security mode, and cert trust are configurable via `OpcUaConfig` with secure defaults. NodeId round-trips symmetrically between parse and the Display form the subscription path uses, including ns=0 and padded-base64 opaque identifiers.

### Context to read first

- `crates/driver-opcua/src/driver.rs`:
  - `connect()`, lines 55-95 — the hardcoded security at 58-75 (`create_sample_keypair(true)` 61, `trust_server_certs(true)` 62, `verify_server_certs(false)` 63; `SecurityPolicy::None.to_str()` 71, `MessageSecurityMode::None` 72, `UserTokenPolicy::anonymous()` 73); and the discarded bool at 87-89 — `timeout(..., session.wait_for_connection()).await.map_err(|_| NotConnected)?` throws away the inner `bool`.
  - `subscribe()`, lines 167-219 — `create_subscription` (178-189) returns `sub_id`; `create_monitored_items` (206-209); the returned stream (216-218) is a bare `UnboundedReceiverStream` with **no** guard, so nothing deletes `sub_id` or its items on drop.
  - `SubscriptionForwarder`, lines 228-248 — `on_data_value` formats the address as `format!("{node_id}")` at 237 (Display), and `let _ = self.tx.send(...)` at 241 silently drops on a closed receiver.
  - `disconnect()`, lines 97-104 — aborts the event loop; note it does not delete subscriptions either.
- `crates/driver-opcua/src/address.rs`:
  - `FromStr for OpcUaAddress`, lines 63-95 — requires the `ns=` prefix (70-72) and rejects any id containing `=` (78) which kills padded base64 opaque ids; `parses_all_node_id_forms` / `rejects_malformed_addresses` tests (119-148) encode the current (asymmetric) contract.
  - `to_node_id`, lines 39-60 — `Opaque` uses `ByteString::from_base64(&self.id)` (56), which needs the padding the parser currently rejects.
- `crates/driver-opcua/src/connection.rs` — `OpcUaConfig`: the fields available today (endpoint, auth, pki_dir, sampling_interval_ms, browse_depth/breadth). This is where the new security fields go.
- `crates/gateway/src/project.rs`, lines 250-252 — `path_by_address.get(&update.address.raw)`: the subscription update's `address.raw` must exactly match the string the gateway registered from the configured address. If the config address is `ns=0;i=2258` but the update formats as `i=2258` (Display drops ns 0), the lookup misses and the update is silently dropped (`continue`).
- Sibling cleanup guards for the shape: `crates/driver-rockwell/src/…` `impl Drop for SubscriptionState` and `crates/driver-ads/src/driver.rs` `SubscriptionGuard` + `GuardedUpdateStream` (330-383) — copy this "stream owns a guard whose Drop tears down server-side state" pattern.
- async-opcua 0.18 API: `Session::wait_for_connection` (returns `bool`), `Session::delete_subscription` (and whether deleting the subscription also removes its monitored items, or if `delete_monitored_items` is needed separately), `ClientBuilder` security-config methods, `SecurityPolicy`/`MessageSecurityMode` enums, and `NodeId` `Display`/`FromStr` behavior for ns=0. **Confirm** each before coding; the fix hinges on these signatures.

### Files to create / modify

1. **Modify** `connect()` (driver.rs 87-89): capture the `bool` from `wait_for_connection()`; `false` → a connect `DriverError` (session's event loop terminated), not `Ok`. Keep the outer timeout → `NotConnected` for the timeout case.
2. **Modify** `subscribe()` to return a guarded stream: introduce an `OpcUaSubscriptionGuard` holding the `Session` handle + `sub_id` (+ monitored item ids if `delete_subscription` doesn't cascade) whose `Drop` calls `delete_subscription` (spawning if the delete must be async, mirroring ADS's `SubscriptionGuard::drop` thread-spawn or an equivalent async teardown). Wrap the `UnboundedReceiverStream` so the guard lives as long as the stream (mirror ADS `GuardedUpdateStream`).
3. **Modify** `OpcUaConfig` (connection.rs) to add configurable `security_policy`, `message_security_mode`, and cert-trust settings (e.g. `trust_server_certs`, `verify_server_certs`) with **secure defaults** (a signed+encrypted policy such as `Basic256Sha256` / `SignAndEncrypt`, verification on). Thread these into `connect()` (58-75) instead of the hardcoded None/None/trust-all. Document that None/None remains selectable for lab use but is not the default.
4. **Fix addressing symmetry** in `address.rs`:
   - Allow parsing an address without the `ns=` prefix as ns=0 (so the Display form `i=2258` round-trips), OR normalize the subscription update's address to always include the namespace — pick the direction that makes `path_by_address` lookups consistent, and make parse ⇄ format symmetric either way. Prefer a single canonical string form used on both sides.
   - Stop rejecting `=` inside the identifier for the `Opaque` (`b=`) form so padded base64 parses; keep rejecting genuinely malformed shapes. (The `split_once('=')` split already separates the first `=`; the remaining `contains('=')` guard at 78 is what over-rejects — scope it to the non-opaque forms or drop it in favor of form-specific validation in `to_node_id`.)
5. **Handle** the silent `let _ = self.tx.send(...)` at 241 with at least a debug log on send failure (the receiver being gone means the stream was dropped — expected during teardown, so debug not warn).
6. **Add tests** to the existing `#[cfg(test)] mod tests` in `driver.rs` and `address.rs` (do not create new test files).

### Behavior

- A dead session (event loop terminated) makes `connect()` return an error, not `Ok`.
- Dropping a subscription stream deletes the server-side subscription (and its monitored items); a 24h resubscribe loop does not accumulate server subscriptions.
- `connect()` uses the configured security policy/mode/cert-trust; defaults are secure (signed+encrypted, verification on); None/None is opt-in.
- `ns=0;i=2258`, plain `i=2258`, and padded-base64 opaque addresses all parse and match their subscription-update address form; `path_by_address` no longer misses ns=0.

### Test requirements

- **Dead-session test**: simulate `wait_for_connection()` returning `false` (extract the check into a testable helper if the real session can't be faked) and assert `connect()` yields an error, not `Ok`. Prove it fails pre-fix (pre-fix returns `Ok`).
- **Guard-deletes-subscription test**: assert that dropping the guarded stream invokes `delete_subscription` with the right `sub_id`. Use a session test double / spy that records delete calls (the driver has no session mock today — add a minimal seam, or unit-test the guard's `Drop` against a recording fake). Prove pre-fix leaks (no delete call).
- **Security-config test**: `OpcUaConfig` with explicit policy/mode deserializes and the values flow into the endpoint/builder; the default config yields a secure (non-None) policy. (Can be a config-plumbing unit test; a live secured server is a maintainer manual-smoke gate.)
- **Addressing tests** (in `address.rs`): `ns=0;i=2258` and (per the chosen direction) bare `i=2258` parse and round-trip to the same canonical string the subscription path emits; a padded-base64 `b=` identifier parses and `to_node_id()` succeeds. Update the existing `rejects_malformed_addresses` test so it no longer asserts that padded-base64 / ns-omitted forms are errors, and add the positive cases. Prove the base64 case fails pre-fix.
- No `sleep()`, no live network in unit tests. Full matrix clean: `cargo test -p openwebhmi-driver-opcua`, workspace `clippy -D warnings`, `fmt --check`.

### Acceptance criteria

- [ ] `connect()` honors `wait_for_connection()`'s bool; `false` → connect error.
- [ ] Subscription stream owns a guard whose `Drop` calls `delete_subscription` (+ monitored-item cleanup if not cascaded); no server-side leak across resubscribes.
- [ ] `OpcUaConfig` exposes `security_policy`, `message_security_mode`, and cert-trust with secure defaults; `connect()` uses them instead of hardcoded None/None/trust-all.
- [ ] `ns=0` and padded-base64 opaque addresses parse and match the subscription-update address form; parse ⇄ format is symmetric; `path_by_address` no longer drops ns=0 updates.
- [ ] `SubscriptionForwarder` send failure is logged (debug), not silently swallowed.
- [ ] Dead-session, guard-delete, and base64-address tests fail pre-fix and pass after; existing address tests updated for the new symmetric contract.
- [ ] No `unwrap`/`expect`/`panic` on production paths introduced; new suppressions use `#[expect(...)]` with a reason.
- [ ] Codex log confirms the async-opcua 0.18 signatures used (`wait_for_connection` bool, `delete_subscription` cascade behavior, security-config builder methods) and whether monitored items need explicit deletion.

### Out of scope

- Certificate provisioning / PKI trust-store management workflow (generating and trusting certs is operational; this brief makes the knobs configurable, not the cert lifecycle).
- Reconnect-and-resubscribe on session loss (the driver has no auto-resubscribe today) — surfacing the dead session as an error is this brief; automatic recovery is a follow-up.
- Browse-tree address handling beyond the ns=0 fix (browse already null-checks `namespace_uri`/`server_index` at 346-351).
- `Variant` type coverage expansion in `variant_to_tag_value`.

### Risks / gotchas

- **`delete_subscription` may be async.** If it can't run in a synchronous `Drop`, mirror ADS's approach (spawn a thread/task holding the `Session` clone to perform the delete), and make sure the `Session` handle outlives the spawned teardown. Confirm whether deleting the subscription cascades to monitored items or if `delete_monitored_items` is also required.
- **Secure defaults can break lab setups.** Flipping the default to signed+encrypted will make previously-working None/None lab servers fail to connect until configured. That's the correct posture, but call it out in the Codex log and keep None/None selectable. This is a behavior change integrators will notice.
- **Address symmetry — pick ONE canonical form.** The bug is that parse and Display disagree. Decide the single canonical string (e.g. always include `ns=`, normalizing the update side) and make both sides use it; don't patch only one side or you'll trade one asymmetry for another. Verify against the gateway `path_by_address` registration source so the keys match.
- **The `=` guard is load-bearing for the non-opaque forms.** `split_once('=')` splits on the first `=`; the `contains('=')` re-check rejects `s=a=b`. Base64 padding (`b=SGVsbG8=`) legitimately contains a trailing `=`. Scope the rejection to string/numeric/guid forms, or move validation into `to_node_id` per form, so opaque isn't collateral damage. Don't just delete the guard blindly — keep `s=a=b` rejected if the current tests intend that (re-evaluate whether that's even correct for string NodeIds, which can contain `=`).
- **Testing the guard needs a seam.** The driver holds a concrete `Arc<Session>` with no trait. Adding a minimal recording fake or extracting the delete call behind a small trait is acceptable; document the seam. Don't claim the leak is fixed if only the happy path compiles — the test must prove `delete_subscription` is actually called on drop.
- **Honesty:** live secured-server validation (real cert exchange, real `BadTooManySubscriptions` avoidance over 24h) is a maintainer manual-smoke gate — say so by name; unit tests prove the plumbing, not the server behavior.

## Codex log

## Claude review

## Verdict
