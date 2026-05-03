---
id: CODEX-AH
title: Native ADS device notifications via TcAdsDll FFI — replace polling on Windows TwinCAT-router backend
owner: codex
phase: 4
status: open
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

## Claude review

<!-- Claude reviews after submission. -->

## Verdict

<!-- Final disposition. -->
