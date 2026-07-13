---
id: CODEX-BN
title: ADS write range-checking — reject out-of-range integers instead of silent truncation
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BN — ADS write range-checking

## Brief

> **HIGH.** `encode_ads_value` writes integers with truncating `as` casts — `(value as i8) as u8`, `value as i16`, `value as u16`, `value as i32`, `value as u32`. Writing `70000` to an `INT` symbol silently stores `4464` and returns `Ok(())`: operator setpoint corruption with no error. Modbus already range-checks every write (`crates/driver-modbus/src/address.rs`) and Rockwell uses `try_from`; ADS is the outlier. Also the `ULINT` **decode** path does `u64 … as i64`, silently wrapping any value above `i64::MAX` into a negative, whereas Rockwell maps the same case to `UnsupportedType`. Use `try_from` with a `DriverError` on out-of-range for every integer width on the write path, and make `ULINT` overflow behavior consistent with Rockwell. This is the ADS slice of the cross-driver "one shape for numeric bounds" principle.

### Goal

Every out-of-range integer write to an ADS symbol is rejected with a descriptive `DriverError` instead of being silently truncated; every in-range value still round-trips byte-identically. `ULINT` values above `i64::MAX` are handled consistently with the Rockwell driver (rejected as unsupported/out-of-range) rather than silently wrapping to a negative `TagValue::Int`.

### Context to read first

- `crates/driver-ads/src/symbols.rs`:
  - `encode_ads_value`, lines 180-217 — the truncating write casts at lines 187-194: `Sint` `(value as i8) as u8` (187), `Usint` `value as u8` (188), `Int` `(value as i16)` (189), `Uint` `(value as u16)` (190), `Dint` `(value as i32)` (191), `Udint` `(value as u32)` (192), `Lint` `value.to_le_bytes()` (193, i64 — no range issue), `Ulint` `(value as u64)` (194). Note the STRING branch (197-209) already returns a proper `DriverError` on overflow — mirror that error style.
  - `decode_ads_value`, lines 156-159 — `Lint` decodes to `i64` (fine); `Ulint` does `u64::from_le_bytes(...) as i64` (159), which wraps values above `i64::MAX` into negatives.
- `crates/driver-modbus/src/address.rs`, `encode_registers`, lines 252-289 — the reference pattern: `if (0..=u16::MAX as i64).contains(&value)` guards, else fall through to an `UnsupportedType` error. Match this shape.
- `crates/driver-rockwell/src/types.rs`, lines 8-32 — `plc_to_tag_value` maps `PlcValue::Ulint` above `i64::MAX` to `DriverError::UnsupportedType { device_type: "ULINT value exceeds OpenWebHMI signed integer range" }` (lines 18-24); `tag_to_plc_value` uses `i32::try_from` on the write side (lines 38-40). Align ADS's `ULINT` decode message/behavior with this.
- `openwebhmi_protocol::TagValue` — `Int` is `i64`; that is the source range every write narrows from.

### Files to create / modify

1. **Modify** `crates/driver-ads/src/symbols.rs`:
   - Replace the truncating write casts (187-194) with `try_from`-based narrowing that returns a `DriverError` (`UnsupportedType` or an out-of-range variant matching the existing STRING-branch style) when the `i64` source value doesn't fit the target width:
     - `Sint` → `i8::try_from(value)`, then encode as one byte.
     - `Usint` → `u8::try_from(value)`.
     - `Int` → `i16::try_from(value)`.
     - `Uint` → `u16::try_from(value)`.
     - `Dint` → `i32::try_from(value)`.
     - `Udint` → `u32::try_from(value)`.
     - `Lint` → unchanged (`i64` source fits `i64`).
     - `Ulint` → `u64::try_from(value)` (rejects negative `i64` values that can't be a `ULINT`).
   - Error messages should name the value and the target ADS type (e.g. `"value 70000 out of range for ADS INT"`), consistent with the STRING branch's format.
2. **Modify** `decode_ads_value` `Ulint` branch (158-159): reject `u64` values above `i64::MAX` with a `DriverError` matching Rockwell's "ULINT value exceeds OpenWebHMI signed integer range" rather than the current `as i64` wrap. Keep `Lint` as-is.
3. **Add tests** to the existing `#[cfg(test)] mod tests` in `crates/driver-ads/src/symbols.rs` (do not create a new test file).

### Behavior

- In-range writes for every width encode to the exact same bytes as today.
- Out-of-range writes (e.g. `70000` → `INT`, `-1` → `USINT`, `300` → `SINT`/`USINT`, `> u32::MAX` → `UDINT`, negative → `ULINT`) return a `DriverError`, never `Ok(())`.
- `ULINT` decode of a value above `i64::MAX` returns a `DriverError` (matching Rockwell), not a silently-negative `TagValue::Int`.
- No change to `Bool`, `Real`, `Lreal`, `String`, or the `Lint` path.

### Test requirements

- **Per-width rejection tests**: one out-of-range write per integer width asserts a `DriverError` (not `Ok`). Each must fail against the pre-fix truncating code (which returns `Ok` with a wrapped value) — verify the pre-fix path returns `Ok` so the test is proven to catch the bug.
- **In-range round-trip tests**: representative in-range values per width still encode to the expected little-endian bytes (extend the existing `encodes_primitives` test rather than fragmenting).
- **ULINT decode overflow test**: bytes for a `u64` above `i64::MAX` decode to a `DriverError`; a value at/below `i64::MAX` still decodes to the correct `TagValue::Int`.
- No `sleep()`, no ports (pure encode/decode unit tests).
- Full matrix clean: `cargo test -p openwebhmi-driver-ads`, workspace `clippy -D warnings`, `fmt --check`.

### Acceptance criteria

- [ ] Every integer write width uses `try_from` and returns a `DriverError` on out-of-range; no truncating `as` casts remain on the write path (187-194).
- [ ] In-range writes for every width encode byte-identically to the pre-fix output.
- [ ] `ULINT` decode above `i64::MAX` returns a `DriverError` consistent with `crates/driver-rockwell/src/types.rs` (18-24), not a wrapped negative.
- [ ] Per-width rejection tests fail against pre-fix code and pass after; round-trip tests cover in-range values.
- [ ] No `unwrap`/`expect`/`panic` introduced; error style matches the existing STRING branch. New suppressions (if any) use `#[expect(...)]`.
- [ ] Codex log confirms the exact `DriverError` variant chosen and why it matches the Modbus/Rockwell precedent.

### Out of scope

- The ADS notification fan-out fix (CODEX-BM) — separate brief; don't touch the driver's subscription plumbing.
- The `expect("len")` calls in `decode_ads_value` (lines 157, 159, 163) — those are guarded by the preceding `require(bytes, 8)` and are a separate cleanup; leave them unless the ULINT change already touches line 159, in which case removing that one `expect` via `try_into().map_err(...)` is acceptable and should be noted.
- `Real`/`Lreal` precision-loss on `f64 → f32` narrowing (the `Real` write casts `value as f32`) — float clamping is a distinct decision; not in this brief.
- Any change to `AdsDataType::from_ads` type inference.

### Risks / gotchas

- **`i8::try_from(value)` then `as u8`.** For `SINT`, the wire format is one byte holding a signed value; `i8::try_from(i64)` gives the range check, then `to_le_bytes()` / `as u8` produces the byte. Confirm the byte pattern matches the pre-fix `(value as i8) as u8` for in-range inputs (it does for in-range; the point is rejecting out-of-range).
- **Negative into unsigned.** `u16::try_from(-1_i64)` correctly errors — that is the desired behavior (a negative can't be a `UINT`). Make sure the round-trip tests don't assume the old silent-wrap.
- **Error variant consistency.** The STRING branch uses `DriverError::UnsupportedType { device_type: … }`. Reuse that variant (or a more specific out-of-range one if the enum has it) so the gateway's `into_quality()` mapping stays consistent across all width errors.
- **Don't widen the fix into decode range-checks that already work.** `Usint`/`Uint`/`Udint` decode paths widen into `i64` losslessly; leave them. Only `Ulint` decode is lossy.
- **Copy the neighbor:** Modbus's `(range).contains(&value)` guard and Rockwell's `try_from` are the two agreed shapes; pick `try_from` (cleaner for narrowing) and state that choice rather than inventing a third.

## Codex log

## Claude review

## Verdict
