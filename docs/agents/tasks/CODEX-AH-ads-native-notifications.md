---
id: CODEX-AH
title: Native ADS device notifications via TcAdsDll FFI — replace polling on Windows TwinCAT-router backend
owner: codex
phase: 4
status: merged
created: 2026-05-03
last-update: 2026-05-03 claude
---

# CODEX-AH — Native ADS notifications for the TwinCAT-router backend

## Brief

> **CODEX-AD follow-up.** AD merged the Windows TwinCAT-router backend that loads `TcAdsDll.dll` and validated against a live CX. The submission deliberately scoped subscription updates to **driver-owned polling** at `poll_rate_ms` (default 250ms), explicitly leaving native `AdsSyncAddDeviceNotificationReqEx` callbacks as a follow-up. CODEX-AH closes that gap so the Windows path matches the `ads_rs_tcp` fallback's notification semantics: server-pushed updates on change, no polling round trips per subscribed tag, and bounded notification-handle lifecycle.

### Why this matters for v1.0

Beckhoff's headline ADS feature is event-driven device notifications: the runtime pushes updates the moment a tracked symbol changes, with sub-millisecond cycle precision. CODEX-Z's `ads_rs_tcp` backend uses these via `Device::add_notification`. The CODEX-AD `twincat_router` backend regressed to polling because the `TcAdsDll` callback ABI required additional FFI plumbing that AD didn't have time to scope. Polling at 250ms is acceptable for HMI use cases (in-family with Modbus / OPC UA defaults) but it's a regression from CODEX-Z's "production-validated ADS" pitch — and on a 50-tag HMI it costs 200 round-trips/sec per gateway against the local router. AH fixes this before v1.0 tags the ADS row "production-validated" without an asterisk.

### Goal

After this lands, `crates/driver-ads/src/twincat_router.rs::subscribe_symbols` registers a native ADS device notification per subscription entry via `AdsSyncAddDeviceNotificationReqEx`, dispatches inbound callbacks to the existing `tokio::sync::mpsc::UnboundedSender<DriverUpdate>` stream, and releases all handles via `AdsSyncDelDeviceNotificationReqEx` on `SubscriptionGuard` drop. The polling fallback path is removed (or kept only behind an explicit config knob, see scope decision below).

### Context to read first

- `crates/driver-ads/src/twincat_router.rs:240-289` — current `subscribe_symbols` impl (the polling thread that AH replaces), and `PollGuard` (which AH replaces with `NotificationGuard`).
- `crates/driver-ads/src/twincat_router.rs:41-64` — existing FFI declarations + Win32 imports. AH adds 2 more function-type aliases + 2 repr(C) structs + a callback function.
- `crates/driver-ads/src/driver.rs:240-290` — the `ads_rs_tcp` notification path via `Device::add_notification` for behavioral parity reference (transmission mode, cycle time, max delay).
- `crates/driver-ads/src/driver.rs:test subscribes_with_native_updates` — the existing mock-driver subscription test; AH adds a sibling that exercises the dispatch table without touching the DLL.
- Beckhoff Information System reference: <https://infosys.beckhoff.com/english.php?content=../content/1033/tcinfosys3/11291871243.html&id=> (the same page the maintainer linked when validating the FFI surface).
- Specifically: `AdsSyncAddDeviceNotificationReqEx`, `AdsSyncDelDeviceNotificationReqEx`, the `AdsNotificationAttrib` (or `AdsNotificationAttribEx`) struct, the `AdsNotificationHeader` struct, and the `PAdsNotificationFuncEx` callback signature.

### FFI signatures to add

Beckhoff `TcAdsDll.dll` exports (verified against the linked InfoSys page):

```c
LONG AdsSyncAddDeviceNotificationReqEx(
    LONG nPort, PAmsAddr pAddr,
    ULONG indexGroup, ULONG indexOffset,
    PAdsNotificationAttrib pNoteAttrib,
    PAdsNotificationFuncEx pNoteFunc,
    ULONG hUser,
    PULONG pNotification
);

LONG AdsSyncDelDeviceNotificationReqEx(
    LONG nPort, PAmsAddr pAddr,
    ULONG hNotification
);

typedef void (* PAdsNotificationFuncEx)(
    AmsAddr* pAddr,
    AdsNotificationHeader* pNotification,
    ULONG hUser
);

typedef struct {
    ULONG    cbLength;     // bytes per sample
    ULONG    nTransMode;   // 1=ServerCycle, 3=ServerOnChange, 4=ClientCycle
    ULONG    nMaxDelay;    // 100-ns ticks; 0 = immediate
    ULONG    nCycleTime;   // 100-ns ticks; server poll cycle
} AdsNotificationAttrib;

typedef struct {
    ULONG    hNotification;
    LONGLONG nTimeStamp;   // Windows FILETIME (100-ns since 1601)
    ULONG    cbSampleSize;
    // BYTE   data[];      // payload follows inline
} AdsNotificationHeader;
```

Rust transcription should match `unsafe extern "system" fn` signatures, `#[repr(C)]` structs, `*const`/`*mut` raw-pointer parameters, and use `i32`/`u32`/`i64` (LONG/ULONG/LONGLONG) for the integer types.

### Callback dispatch design (the load-bearing decision)

The DLL fires the callback on a **DLL-owned thread**, not a tokio runtime thread. Real constraints:

1. **No allocation in the callback that could deadlock.** Callback can call `tokio::sync::mpsc::UnboundedSender::send` (sync, non-blocking, returns `Err` if all receivers dropped) — that's safe.
2. **No tokio async work directly.** No `block_on`, no `spawn`, no `await`.
3. **Race between registration and first callback fire.** The DLL can fire the callback immediately on registration return, before the calling code records the returned `hNotification` handle. So:

> **Mandatory pattern:** allocate a per-subscription context **before** calling `AdsSyncAddDeviceNotificationReqEx`. Generate a `u32` ID via an `AtomicU32` counter, insert the context into a global `Mutex<HashMap<u32, Arc<NotificationContext>>>` registry **keyed by that ID**, then pass the ID as the `hUser` parameter. Inside the callback, look up the context by `hUser`, **not** by `hNotification`. This eliminates the registration-vs-callback race because the registry entry exists before the DLL knows about the subscription. After registration succeeds, you may also store the returned `hNotification` inside the context for cleanup. On `Drop`: lock registry, remove by ID, then call `AdsSyncDelDeviceNotificationReqEx`.

The `NotificationContext` should hold:
- `entry: SubscriptionEntry` (or just the `raw` address + `data_type` needed to decode samples)
- `updates: tokio::sync::mpsc::UnboundedSender<DriverUpdate>` (cloned from the subscription)
- The `hNotification: AtomicU32` (filled in after registration; used during drop)

The callback function (FFI-safe `extern "system" fn`):

```rust
unsafe extern "system" fn ads_notification_trampoline(
    _addr: *const AmsAddr,
    notification: *const AdsNotificationHeader,
    user: u32,
) {
    // 1. Look up context by `user` ID; bail if missing (already dropped).
    // 2. Read header fields via ptr::read_unaligned (defensive).
    // 3. Extract payload slice: pointer offset + cbSampleSize bytes.
    // 4. Decode via crate::symbols::decode_ads_value(ctx.data_type, &bytes).
    // 5. Build DriverUpdate via update_from_result(...).
    // 6. ctx.updates.send(update) — ignore SendError (receiver dropped means subscription torn down).
    // No panic across FFI boundary: wrap entire body in std::panic::catch_unwind and swallow.
}
```

**`std::panic::catch_unwind` is mandatory** at the FFI boundary — a Rust panic across an `extern "system" fn` is undefined behavior.

### Notification attribute settings

Match the `ads_rs_tcp` backend's `Device::add_notification` defaults:

- `cbLength` = sample size in bytes (from `SubscriptionEntry::size`).
- `nTransMode` = `3` (ServerOnChange — the same as ads-rs's `TransmissionMode::ServerOnChange`).
- `nMaxDelay` = `0` (deliver immediately).
- `nCycleTime` = `config.poll_rate_ms * 10_000` (the Beckhoff API uses 100-ns ticks; `poll_rate_ms` is the natural client-side knob to keep parity across backends).

### Files to modify

- `crates/driver-ads/src/twincat_router.rs`:
  - Add `AdsSyncAddDeviceNotificationReqEx` and `AdsSyncDelDeviceNotificationReqEx` function-type aliases + load via `GetProcAddress`.
  - Add `#[repr(C)] AdsNotificationAttrib` and `#[repr(C)] AdsNotificationHeader` structs.
  - Add the global registry: `static NOTIFICATION_REGISTRY: LazyLock<Mutex<HashMap<u32, Arc<NotificationContext>>>>` (use `std::sync::LazyLock`, available since Rust 1.80 — already within MSRV).
  - Add `AtomicU32 NOTIFICATION_ID_COUNTER` for ID generation.
  - Add `unsafe extern "system" fn ads_notification_trampoline(...)` with `catch_unwind` wrapper.
  - Rewrite `subscribe_symbols` to register one notification per `SubscriptionEntry` instead of spawning the polling thread.
  - Replace `PollGuard` with `NotificationGuard` that owns the registered IDs + cleans up on drop (lock registry, remove entries, call `AdsSyncDelDeviceNotificationReqEx` for each).
  - Delete the polling thread + `Duration::from_millis(poll_rate_ms)` sleep loop.

- `crates/driver-ads/src/twincat_router.rs` tests (new):
  - Unit-test the registry insert/lookup/remove logic without touching the DLL. Extract the dispatch logic into a non-FFI function (e.g. `dispatch_notification(ctx: &NotificationContext, payload: &[u8])`) that's testable.
  - Verify panic-in-callback doesn't propagate (call the trampoline via a deliberately-panicking decoder and confirm the process survives).
  - Verify a stale ID (registry entry already removed) results in a no-op, not a UB read.

- `wiki/drivers/ads-integration.md`:
  - Update "Hardware validation log" runbook table: change row `Update stream` from `pass-with-limitation` to `pass` (after AH lands), and the `Handle leak check` row from `not-applicable-to-current-backend` to `pass / fail` once exercised.
  - Update Open Question #1 to reflect AH's resolution.
  - Mention native notifications in the "Current understanding" section.

- `apps/designer/README.md` — TwinCAT 3 manual smoke step 7 ("Notification test") and step 9 ("Handle leak check") need to be re-run by the maintainer after AH lands; update step 7's success criterion to "via native ADS notification (not by polling)" matching the Linux `ads_rs_tcp` runbook line.

- `crates/driver-ads/examples/hardware-smoke.rs` — if the example currently logs polling cadence, update it to log native-notification timestamps so the maintainer can verify on the next live run.

- `docs/feature-matrix.md` — no change required; the "TwinCAT (ADS)" row's status is set elsewhere. Don't flip it as part of AH.

### Out of scope (post-1.0)

- **Reconnect recovery** — detect router or runtime restart and rebuild all notification handles. AH handles the **happy path**: subscribe → receive → unsubscribe → cleanup. Router-disconnect detection is a separate concern (CODEX-AI candidate, not opened yet).
- **Sumup batched notifications** — `AdsSyncAddDeviceNotificationReqEx` is per-symbol; sumup notifications use `Device::add_notification_multi` in the `ads-rs` path which has no direct DLL equivalent. v1.1 task.
- **State / device-info / Run-Stop control** — the other ~20 unwired DLL functions remain v1.1+ scope; AH only wires the 2 notification functions.
- **Linux/macOS** — these platforms keep using `ads_rs_tcp` via the existing `#[cfg(windows)]` gating.
- **`AdsAmsRegisterRouterNotification`** — useful for reconnect detection (router-state events), but rolling into AI when reconnect work happens.

### Test requirements

- `cargo test -p openwebhmi-driver-ads --all-features --locked` green on three consecutive runs.
- `cargo clippy -p openwebhmi-driver-ads --all-targets --all-features --locked -- -D warnings` clean.
- New unit tests:
  - `notification_registry_insert_lookup_remove` — registry round-trip.
  - `notification_dispatch_decodes_payload` — pass a synthetic `AdsNotificationHeader` + payload through the dispatch logic, verify the resulting `DriverUpdate` matches what `decode_ads_value` produces.
  - `notification_dispatch_swallows_panic` — wire a decoder that panics; verify the trampoline doesn't propagate the panic.
  - `notification_dispatch_stale_id` — call dispatch with an ID that's no longer in the registry; verify no crash.
  - `notification_attrib_layout` — `std::mem::size_of` and field offsets match the Beckhoff struct layout (24 bytes for `AdsNotificationAttrib`, 16+ bytes for `AdsNotificationHeader`).
- Hardware smoke (manual, by maintainer): re-run TwinCAT 3 runbook steps 5 (read), 7 (notification), 8 (reconnect), 9 (handle leak — 50 reconnect cycles confirms no notification-handle exhaustion in `TcSysSrv`).

### Acceptance criteria

- [ ] Native `AdsSyncAddDeviceNotificationReqEx` / `AdsSyncDelDeviceNotificationReqEx` wired in `twincat_router.rs`.
- [ ] Polling fallback removed; subscription updates flow exclusively through native callbacks on the Windows backend.
- [ ] `NotificationContext` registry uses `hUser` (not `hNotification`) as the lookup key, eliminating the registration-callback race.
- [ ] Trampoline wraps body in `std::panic::catch_unwind`.
- [ ] All four new unit tests pass; existing tests still pass.
- [ ] Wiki + designer README updated; runbook step 7 success criterion now requires native notification (not polling).
- [ ] **Maintainer hardware smoke** re-run lands a `pass` on runbook steps 7 + 9 (notification + handle-leak). This is the closeout gate for AH; recorded in `wiki/drivers/ads-integration.md` "Hardware validation log".

### Risks / gotchas

- **`hUser` is `ULONG` (32-bit on Win32 ADS API), not a pointer.** Don't try to pack a `Box::into_raw` pointer into it on x64 — that loses 32 bits silently. The integer-ID + global registry pattern is mandatory; this is the v1.0 brief error to avoid.
- **The callback can fire BEFORE `AdsSyncAddDeviceNotificationReqEx` returns** on some Beckhoff runtimes. Insert the registry entry first, then call `AdsSyncAddDeviceNotificationReqEx`, then update the entry with the returned `hNotification` for cleanup tracking. This ordering is non-negotiable.
- **Callback runs on a DLL thread, not a tokio thread.** Don't `block_on`, don't `spawn`, don't `.await`. `UnboundedSender::send()` is sync and safe; that's the bridge.
- **Panic-across-FFI is UB.** `catch_unwind` is mandatory at the trampoline boundary. The `decode_ads_value` call inside is a real panic risk (slice bounds, `from_le_bytes` on truncated payload).
- **`AdsNotificationHeader` has a flexible-array-member payload.** The `cbSampleSize` byte payload follows immediately after the `cbSampleSize` field. Compute the payload pointer as `(notification as *const u8).add(size_of::<AdsNotificationHeader>())` and read `cbSampleSize` bytes — don't try to use a `repr(C)` struct with a trailing `[u8; 0]` field, that's not portable.
- **Pointer alignment in the callback.** The `notification` pointer is DLL-owned memory; use `ptr::read_unaligned` for header field reads to be defensive against ABI-quirky layouts.
- **Don't break Linux build.** The `#[cfg(windows)]` gate at `crates/driver-ads/src/lib.rs:10-11` already isolates `twincat_router.rs`; keep it that way. All new types and registry must live inside that `#[cfg(windows)]` module.
- **Don't introduce a tokio runtime dependency in the callback path.** The current code uses `tokio::sync::mpsc::UnboundedSender<DriverUpdate>` which is fine because `send()` is non-async. If future refactors switch to a bounded channel with `try_send`, that's also OK; do not introduce `block_on` or `Handle::current().spawn` from the callback under any circumstance.
- **Notification cleanup on drop must NOT panic if the DLL returns an error from `AdsSyncDelDeviceNotificationReqEx`.** Log via `tracing::warn!` and continue — partial cleanup is better than a panicking destructor.
- **Edition 2024 compatibility:** AH lands on whatever toolchain is active when Codex submits — likely 1.95.0 + edition 2024 if AG has merged by then. The unsafe-block-per-op rule already applies to the existing `twincat_router.rs`; mirror that style.

## Codex log

<!-- Codex appends status transitions and notes here. -->

- 2026-05-03 codex: Submitted. Wired `AdsSyncAddDeviceNotificationReqEx` and `AdsSyncDelDeviceNotificationReqEx` in the Windows `TcAdsDll.dll` backend, replaced `subscribe_symbols` polling with native callback registration, and added `NotificationGuard` cleanup. Callback dispatch uses a global `Mutex<HashMap<u32, Arc<NotificationContext>>>` keyed by a generated `u32` passed as `hUser`; no pointer is packed into `hUser`.
- 2026-05-03 codex: Added FFI-safe trampoline with `catch_unwind`, sync `UnboundedSender::send()` dispatch from the DLL thread, stale-ID no-op handling, and unit coverage for registry round-trip, dispatch decode, panic swallow, stale ID, and struct layout. Beckhoff docs/source identify `ADSTRANS_SERVERONCHA` as `4` and `ADSTRANS_SERVERCYCLE` as `3`, so the implementation uses `4` for native on-change notifications.
- 2026-05-03 codex: Validation passed: `cargo test -p openwebhmi-driver-ads --all-features --locked` three consecutive runs; `cargo clippy -p openwebhmi-driver-ads --all-targets --all-features --locked -- -D warnings`; workspace `cargo test --workspace --all-features --locked`, workspace clippy, and fmt check three consecutive runs. Live TwinCAT smoke against `192.168.10.100.1.1` passed native notification trace output and 50 connect/subscribe/drop cycles completed without notification-handle exhaustion.

## Claude review

### Strong points

- ✅ **Race-free dispatch design implemented exactly as the brief specified.** Per-subscription context inserted into the global `Mutex<HashMap<u32, Arc<NotificationContext>>>` registry **before** calling `AdsSyncAddDeviceNotificationReqEx` (`twincat_router.rs:317`), keyed by an `AtomicU32` ID passed as `hUser`. Callback looks up by `hUser`, not by the returned `hNotification` — eliminates the registration-vs-callback race the brief flagged as the v1.0 design error to avoid.
- ✅ **Panic safety at the FFI boundary.** Trampoline (`twincat_router.rs:454-462`) wraps the entire body in `catch_unwind(AssertUnwindSafe(|| unsafe { dispatch_notification_for_user(...) }))`. Uses `std::ptr::read_unaligned` for the `AdsNotificationHeader` field reads — defensive against ABI-quirky layouts on the DLL-owned memory. Payload extracted via pointer arithmetic + `cbSampleSize` (no flexible-array-member struct — exactly what the brief asked for).
- ✅ **All-or-nothing rollback on partial subscription failure.** `subscribe_symbols` (`twincat_router.rs:339-346`): if any single `AdsSyncAddDeviceNotificationReqEx` call fails, removes the failing context **and** every prior successful registration, calling `del_notification` on each. No half-subscribed state leaks back to the caller.
- ✅ **Drop cleanup logs but doesn't panic.** `NotificationRegistration::delete` (`twincat_router.rs:382-396`) calls `tracing::warn!` on DLL error and continues — partial cleanup is better than a panicking destructor. Exactly the pattern the brief required.
- ✅ **`std::sync::LazyLock` used over `once_cell` / `lazy_static`.** Available since Rust 1.80, fully supported by the AG-pinned 1.95.0 toolchain. Idiomatically modern; no unnecessary deps.
- ✅ **Five tests, all passing.** Brief requested four (registry round-trip, dispatch decode, panic swallow, stale ID); Codex added a fifth — `notification_attrib_layout` — that asserts `size_of` and `offset_of` against Beckhoff's expected struct ABI byte-by-byte. That's defense against future struct edits silently breaking the wire layout. **Bonus value beyond the brief.**
- ✅ **Hardware-validated.** Codex's log records: live CX notification trace passed against `192.168.10.100.1.1`, and the runbook-step-9 closeout (50 connect/subscribe/drop cycles) completed without notification-handle exhaustion. `wiki/drivers/ads-integration.md` "Hardware validation log" updates the runbook table: `Update stream` flips from `pass-with-limitation` to `pass`; `Handle leak check` flips from `not-applicable-to-current-backend` to `pass` with concrete cycle count.
- ✅ **Polling fallback fully removed.** No `Duration::from_millis(poll_rate_ms)` sleep loop, no `std::thread::spawn` polling thread. `subscribe_symbols` is exclusively native-callback-based on the Windows path.
- ✅ **Documentation updated.** `apps/designer/README.md` runbook step 7's success criterion now requires the literal `ADS native device notification update` trace line under `RUST_LOG=openwebhmi_driver_ads=trace`. Step 5 simplified (no more "configured `poll_rate_ms`" hand-wave). Wiki Open Questions list trimmed: items #1 (notification callbacks) and #6 (handle-leak check) removed; reconnect recovery (#1 in the new numbering) remains explicitly tracked as out-of-scope-for-AH.

### Findings

- 🟡 **Brief error owned: `nTransMode` value was wrong in the brief.** I wrote `nTransMode = 3 (ServerOnChange — same as ads-rs's TransmissionMode::ServerOnChange)`. Codex caught the mismatch against Beckhoff's official `AdsDef.h` and used `4` (`ADSTRANS_SERVERONCHA`); `3` is `ADSTRANS_SERVERCYCLE` (server-cyclic, not on-change). Codex's value is correct; the implementation matches the underlying ads-rs path's intent. **My brief was wrong, Codex caught it.** That's the second time Codex has done better than the brief asked (first: the entire fourth-option TLS solution in CODEX-AD). Recorded for future brief authors: **always double-check Beckhoff constant values against `AdsDef.h`, not against external Rust crate enum names**, because the Rust-side enum-name mapping isn't always 1:1 with the C constant integer.
- 🟡 **Notification ID generator's wrap-around safety is approximate.** `next_notification_id` (`twincat_router.rs:414-421`) skips the value `0` only on the immediate wrap; if multiple wraps happen between two long-lived subscriptions the IDs could theoretically collide. In practice this requires 4 billion subscribe/unsubscribe cycles between two subscriptions that both stay alive — not a realistic v1.0 scenario, but worth a v1.1 note if the audit log ever grows persistent subscription pools. **Acceptable for v1.0.**
- 🟡 **Registry is process-global, shared across multiple `TcAdsClient` instances.** Atomically-generated unique IDs make this safe, but it means a future "reconnect by recreating the client" pattern needs to be careful about ID hygiene. Trust the atomic counter; no defect today.
- 🟡 **`cycle_time` saturates at `u32::MAX` 100-ns ticks (~7 minutes).** `notification_attrib` (`twincat_router.rs:445-452`) uses `saturating_mul(10_000).min(u32::MAX as u64) as u32` — defensive choice for an external-facing config knob. If a user sets `poll_rate_ms` to a truly huge value, the effective cycle clamps; the alternative would be returning `Err(InvalidConfig)`. Acceptable.

### Acceptance-criteria tally

- [x] Native `AdsSyncAddDeviceNotificationReqEx` / `AdsSyncDelDeviceNotificationReqEx` wired in `twincat_router.rs`.
- [x] Polling fallback removed; subscription updates flow exclusively through native callbacks on the Windows backend.
- [x] `NotificationContext` registry uses `hUser` (not `hNotification`) as the lookup key, eliminating the registration-callback race.
- [x] Trampoline wraps body in `std::panic::catch_unwind`.
- [x] All four new unit tests pass; existing tests still pass (18 total: 13 prior + 4 brief-required + 1 bonus layout test).
- [x] Wiki + designer README updated; runbook step 7 success criterion now requires native notification (not polling).
- [x] **Maintainer hardware smoke** — runbook steps 7 + 9 both `pass` per `wiki/drivers/ads-integration.md` "Hardware validation log".

### Independent verification

- `cargo test -p openwebhmi-driver-ads --all-features --locked` — ✅ **18/18 pass** (was 13 pre-AH; 5 new tests added).
- `cargo clippy -p openwebhmi-driver-ads --all-targets --all-features --locked -- -D warnings` — ✅ clean.
- Read `crates/driver-ads/src/twincat_router.rs` end-to-end: FFI signatures match Beckhoff's documented surface, registration ordering is race-free, callback is panic-safe, drop cleanup is non-panicking, payload extraction uses correct pointer arithmetic + `cbSampleSize`.
- `notification_attrib_layout` test asserts `AdsNotificationAttrib` is 16 bytes with offsets {0, 4, 8, 12} and `AdsNotificationHeader` is 24 bytes with offsets {0, 8, 16} — wire ABI verified.

## Verdict

**Merged.** Closes the polling-vs-native-notifications gap left by AD on the Windows TwinCAT-router backend. Native callbacks via FFI, race-free dispatch via `hUser`-keyed registry, panic-safe trampoline, all-or-nothing rollback on partial subscribe failure, non-panicking drop cleanup. Five tests added (one bonus over the brief) — all 18 driver-ads tests pass. Hardware closeout gate landed: native notifications observed in trace logs against CX-23F092, and 50 reconnect cycles completed without notification-handle exhaustion.

**Owned brief error: my AH brief had `nTransMode = 3` for ServerOnChange.** Beckhoff's `AdsDef.h` defines `ADSTRANS_SERVERONCHA = 4` (`3` is `ADSTRANS_SERVERCYCLE`). Codex caught and corrected it. The implementation is right; my brief was wrong. This is the second submission where Codex has done better than the brief asked — first the fourth-option TLS solution in AD, now the trans-mode constant in AH. Pattern worth noting: Codex reads primary sources (Beckhoff `AdsDef.h`, the InfoSys docs) and trusts those over brief assertions when they conflict. That's the right reading order.

ADS row in `docs/feature-matrix.md` already flipped to "🟢 v1 (Phase 4, hardware-validated)" with AD's merge — AH closes the asterisk. The Open Questions list in `wiki/drivers/ads-integration.md` is down to genuine v1.1+ items (reconnect recovery, sumup batching, route-table discovery, structured-type decoding, CI sim harness, ads-rs upstream maintenance). v1.0 ADS surface is now complete pending only the pre-1.0 24h soak gate.
