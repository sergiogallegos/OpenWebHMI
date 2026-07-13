---
id: CODEX-BK
title: Rockwell rust-ethernet-ip 0.7 → 1.2 upgrade + target-type-aware writes
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BK — Rockwell `rust-ethernet-ip` 0.7 → 1.2 upgrade + typed writes

## Brief

> Two coupled goals. **(A) Version bump:** the maintainer released `rust-ethernet-ip 1.2.0`. Move the workspace pin (`Cargo.toml:71`, currently `rust-ethernet-ip = "0.7"`) to 1.2 — a **major-version jump** (0.7 → 1.2), so the crate's API will have changed; read the new crate's API (docs.rs / released source) and adapt `crates/driver-rockwell` accordingly, then update `wiki/drivers/rust-ethernet-ip-integration.md` for the new version + migration notes. **(B) Typed writes (hardware-gate blocker):** `crates/driver-rockwell/src/types.rs::tag_to_plc_value` (35-48) hardcodes `TagValue::Int → PlcValue::Dint` and `TagValue::Real → f32 REAL` with no target-type awareness. Writing a setpoint to an `INT`/`SINT`/`LINT`/`LREAL` tag on a real CompactLogix returns a CIP data-type-mismatch fault, and `LREAL` writes lose precision through the `as f32` cast. The **read** path (types.rs:8-32) already handles all these types, so the asymmetry is silent until a write. Make the write path aware of the target tag's PLC data type and encode accordingly, extending the existing `i32::try_from` range-check discipline (types.rs:38-44) to `INT`/`SINT`/`LINT`/`LREAL`. This is the first thing the pre-1.0 CompactLogix soak hits. The two goals are paired because 1.2's API likely changes how writes/types are expressed — if Codex finds them cleanly separable, note that in the Codex log.

### Goal

`driver-rockwell` builds against `rust-ethernet-ip 1.2.0`, all existing behavior preserved. Writing an OpenWebHMI value to a Rockwell tag encodes to the **target tag's actual PLC data type** — an `INT` tag receives an `INT`, a `LREAL` tag receives a full-precision `LREAL`, a `SINT` tag receives a range-checked `SINT` — instead of always `DINT`/`REAL`. Out-of-range integer writes are **rejected** with a clear error, not silently truncated. The wiki integration page reflects 1.2 and documents the write-type mapping.

### Context to read first

- `Cargo.toml:70-71` — the pin. The comment on line 70 (`# Driver dep — upgraded by version bump (see wiki/...)`) already anticipates this bump. This IS an explicit version-bump task, so the pin moves; all other `=`-pinned deps (`tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads` — lines 62-69) stay pinned per CLAUDE.md.
- `crates/driver-rockwell/src/types.rs` — the whole file. Read path (8-32) handles `Bool/Sint/Int/Dint/Lint/Usint/Uint/Udint/Ulint/Real/Lreal/String/Udt` with `try_from` for `Ulint` (18-24). Write path (35-48) collapses everything to `Bool/Dint/Real(as f32)/String`. `i32::try_from` at 38-44 is the range-check precedent to extend.
- `crates/driver-rockwell/src/driver.rs:93-106` — `write`. Note the **uncommented read-modify-write workaround** at 98-100: `if matches!(plc_value, PlcValue::String(_)) { let _current = guard.read_tag(...).await?; }` — a discarded read before a string write, with no WHY comment. Address this: add a comment explaining the invariant (why a string write needs a preceding read) or remove it if 1.2 makes it unnecessary.
- `crates/driver-rockwell/src/driver.rs:14-16` — current `rust_ethernet_ip` imports (`EtherNetIpError, PlcValue, TagGroupEvent, TagGroupEventKind, TagGroupValueResult`). The 1.2 API may rename/restructure these; `map_eip_error` (235-280) enumerates every `EtherNetIpError` variant and will break if variants changed — that exhaustive match is your compile-time migration checklist.
- `crates/driver-rockwell/src/eip_client.rs` — the `EipClientLike` trait boundary (`read_tag`/`write_tag`/`upsert_tag_group`/`subscribe_tag_group`) that wraps the real client behind a mockable seam. Type metadata for the write path likely comes through here — check whether 1.2 exposes a tag-type query (`read_tag` returning a typed `PlcValue` already tells you the target type on a prior read).
- `crates/driver-rockwell/src/config.rs` — `RockwellConfig`. If target tag types come from config rather than a live browse/read, this is where a type map would live. Prefer live type discovery (read the tag's current `PlcValue` → learn its type) over hand-maintained config if 1.2 supports it.
- `wiki/drivers/rust-ethernet-ip-integration.md` — the durable integration reference. "as of 0.7.0" sections (crate API, `PlcValue`, supported types, the wrap table) need updating to 1.2. Follow `wiki/AGENTS.md` for the wiki edit.
- [`docs/agents/notes/toolchain-drift.md`](../notes/toolchain-drift.md) — for reconciling any "builds for me, not in CI" version-drift surprises.

### Files to create / modify

1. **Modify** `Cargo.toml:71`: `rust-ethernet-ip = "1.2"`. Keep the line-70 comment (or update it to point at the new migration notes). Do not touch any other dependency version.
2. **Modify** `crates/driver-rockwell/src/types.rs::tag_to_plc_value`: make encoding target-type-aware. Signature likely gains a target-type parameter (e.g. `tag_to_plc_value(value: TagValue, target: PlcType) -> DriverResult<PlcValue>`, where `PlcType` is derived from a prior read of the tag or from 1.2 metadata). Map:
   - `TagValue::Int` → `SINT` / `INT` / `DINT` / `LINT` per target, each with a `try_from` range check (`i8::try_from`, `i16::try_from`, `i32::try_from`, direct `i64`). Out-of-range → `DriverError::UnsupportedType`/`RemoteFault` with a message naming the target type and the offending value. **Reject, never truncate.**
   - `TagValue::Real` → `REAL` (`as f32`) only when the target is `REAL`; `LREAL` target → full-precision `f64` `PlcValue::Lreal` (no `as f32` narrowing).
   - `TagValue::Bool` → `BOOL`; `TagValue::String` → `STRING`, as today.
   - When the target type is unknown (no prior read / 1.2 gives no metadata), fall back to the current `DINT`/`REAL` default **but** document the fallback in a WHY comment — it's a known-lossy path, not the intended one.
3. **Modify** `crates/driver-rockwell/src/driver.rs::write` (93-106): thread the target type into `tag_to_plc_value` (a read-to-learn-type, or 1.2 metadata). Add a WHY comment to — or remove — the string-write read-modify-write workaround at 98-100. Adapt any imports/`map_eip_error` variants that 1.2 changed.
4. **Modify** `crates/driver-rockwell/src/eip_client.rs` and any mock (`with_client` seam) as needed for 1.2's API and to expose target-type discovery for tests.
5. **Modify** `wiki/drivers/rust-ethernet-ip-integration.md`: version `0.7.0` → `1.2.0`, API changes, the write-type mapping table, and any migration notes (what broke, what moved). Update `last-validated`.
6. **Do NOT** touch the read path's semantics (types.rs:8-32) beyond what 1.2 forces; it already handles all 13 types correctly.

### Behavior

- `driver-rockwell` compiles and all existing tests pass against `rust-ethernet-ip 1.2.0`.
- Writing `TagValue::Int(200)` to a `SINT` tag (range -128..127) is **rejected** with an error naming SINT and 200 — not truncated to a wrapped byte.
- Writing `TagValue::Int(30000)` to an `INT` tag round-trips as `INT`; to a `DINT` tag as `DINT`; to a `LINT` tag as `LINT`.
- Writing `TagValue::Real(3.14159265358979)` to a `LREAL` tag preserves `f64` precision (no `as f32`); to a `REAL` tag narrows to `f32` as before.
- A write to a tag whose type couldn't be discovered uses the documented `DINT`/`REAL` fallback and the WHY comment explains the limitation.
- The string-write path's discarded read either has a WHY comment justifying it or is gone.
- `Cargo.lock` diff is bounded to `rust-ethernet-ip` + its own transitives that changed 0.7→1.2 — not a workspace-wide churn (CLAUDE.md "Cargo.lock diff is bounded"). If it churns wider, stop and investigate.

### Test requirements

- **Add to `crates/driver-rockwell/src/types.rs`'s existing `#[cfg(test)] mod tests`** (50-89) — do not fragment. For each target type (`SINT`/`INT`/`DINT`/`LINT`/`REAL`/`LREAL`), assert `tag_to_plc_value` encodes to the matching `PlcValue` variant:
  - In-range integer round-trips to the correct variant per target.
  - **Out-of-range integer is rejected** (e.g. `Int(200)` → `SINT` errors; `Int(70000)` → `INT` errors; `Int(i64::MAX)` → `DINT` errors). Assert the `Err` — this is the "reject not truncate" guarantee and must **fail against the current unconditional-`DINT` code** (today `Int(200)` silently becomes `Dint(200)` and would be a data-type mismatch only at the PLC). Run against pre-fix once to confirm the test catches it.
  - `Real(f64)` to `LREAL` preserves precision (assert the `f64` survives, not a `f32`-narrowed value); to `REAL` narrows as before.
- **Write-path test** (extend the existing driver.rs tests using the `with_client`/`EipClientLike` mock seam): a mocked client whose tag reports a given type receives the correctly-typed `PlcValue` on write. If 1.2's API changes how the mock reports type, adapt the mock.
- **Version-bump proof:** full validation matrix clean against 1.2:
  - `cargo build --workspace --all-features --locked`
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - `cargo test --workspace --all-features --locked`
  - `cargo fmt --check`
  - `cargo doc --workspace --no-deps` (no missing-docs regression; `driver-rockwell` is `#![deny(missing_docs)]`).
- No `sleep()`/wall-clock waits; bind any test listener to `127.0.0.1:0` if network testing is added (unlikely — the mock seam avoids real sockets).

### Acceptance criteria

- [ ] `Cargo.toml` pins `rust-ethernet-ip = "1.2"`; no other dep version moved; `Cargo.lock` diff bounded to that crate + its changed transitives.
- [ ] `driver-rockwell` builds and all existing tests pass against 1.2, with the API migration absorbed (imports + `map_eip_error` variants reconciled).
- [ ] `tag_to_plc_value` encodes `SINT`/`INT`/`DINT`/`LINT`/`REAL`/`LREAL` per the target tag's type; `LREAL` writes are full-precision.
- [ ] Out-of-range integer writes are rejected with a type-and-value error, never truncated — proven by a test that fails against pre-fix code.
- [ ] The unknown-target-type fallback (if retained) has a WHY comment marking it as lossy.
- [ ] The string-write read-modify-write workaround (driver.rs:98-100) has a WHY comment or is removed.
- [ ] `wiki/drivers/rust-ethernet-ip-integration.md` updated to 1.2 with the write-type mapping and migration notes; `last-validated` bumped.
- [ ] Full validation matrix clean; no `unwrap`/`expect`/`panic!` on new production paths; `#[expect]` over `#[allow]` for any new lint.
- [ ] Codex log states whether the version bump and the typed-write fix turned out separable, and how target-type discovery is done (prior read vs 1.2 metadata vs config).

### Out of scope

- **Driver-type dispatch / connection-state signal** — CODEX-BI / CODEX-BJ. This task stays inside `driver-rockwell` + the pin + the wiki.
- **`browse` implementation** for Rockwell (still returns the "use external tag-list import" error) unless 1.2 makes tag introspection trivial and it's needed for type discovery — in which case implement the minimum needed and note it, don't build a full browse tree.
- **UDT write support** — `Udt` stays `UnsupportedType` on both read and write.
- **Batch write optimization** (`write_tags_batch`) — single-tag typed writes are the v1 scope.
- **Bumping any other `=`-pinned dependency.**
- **Real-hardware validation** — the CompactLogix soak is the maintainer's manual gate; this task proves the encoding against the mock seam.

### Risks / gotchas

- **Major-version API churn (0.7 → 1.2).** `PlcValue`, `EtherNetIpError` variants, `TagGroupEvent*`, and the client methods may all have moved. `map_eip_error`'s exhaustive match (driver.rs:235-280) is the compile-time checklist — if it stops compiling, that's the list of what changed. Read the released 1.2 source/docs before writing; don't guess the API from the 0.7 shape.
- **Target-type discovery mechanism is the key design choice.** Options: (a) read the tag first to learn its `PlcValue` variant, then encode to match — costs an extra round trip per write; (b) 1.2 exposes tag-type metadata directly — preferred if available; (c) type comes from `RockwellConfig` — brittle, hand-maintained. Ask "why this and not the alternative?" and state the pick. The existing string-write path already does a pre-write read (98-100), so precedent for (a) exists.
- **`LREAL` precision.** The current `value as f32` (types.rs:45) silently narrows every `Real`. For a `LREAL` target this is a data-loss bug, not a type-mismatch fault (the write succeeds but the value is wrong). The read path already returns `Lreal` as full `f64` (types.rs:26) — the write asymmetry is the silent half.
- **Range-check discipline.** Extend the `i32::try_from` pattern (38-44), don't invent a new one. `SINT` = `i8::try_from`, `INT` = `i16::try_from`, `DINT` = `i32::try_from`, `LINT` = direct `i64`. Each `Err` maps to a `DriverError` naming the type and value.
- **Pin discipline.** This is the one task allowed to move the `rust-ethernet-ip` pin. Every other `=`-pin stays. Do NOT `cargo update` broadly — use the bounded update and inspect the lockfile diff.
- **Separability.** If the 1.2 API makes typed writes trivial (metadata built in) or, conversely, if the write fix is cleanly doable on 0.7, the two goals might separate. The brief pairs them because 1.2 likely changes write/type expression; if Codex finds otherwise, note it in the log so a future split is on record — but land both here unless there's a blocking reason not to.
- **Honesty.** State plainly that hardware validation (real CompactLogix INT/SINT/LINT/LREAL writes) is deferred to the maintainer soak — the mock proves encoding, not the on-wire CIP exchange.

## Codex log

## Claude review

## Verdict
