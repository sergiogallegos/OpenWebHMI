---
status: active
last-validated: 2026-08-25
---

# `rust-ethernet-ip` integration

## Summary

`driver-rockwell` is OpenWebHMI's first PLC driver. It is a thin adapter that wraps the **`rust-ethernet-ip`** crate (https://github.com/sergiogallegos/rust-ethernet-ip) behind the OpenWebHMI `Driver` trait. The crate is owned and maintained by the same author as this project, but it is consumed as a **versioned dependency from crates.io** — not as a git submodule, not as a vendored copy. Upgrades happen by bumping the version in the workspace `Cargo.toml`.

This page is the durable maintainer reference for that integration: what the crate provides, how we wrap it, when to upgrade, what to validate at each upgrade, and where the boundaries are.

## Current understanding

### What the crate provides (as of 1.2.1)

- **`EipClient`** — main connection/communication interface to a Rockwell EtherNet/IP target.
- **`read_tag()`, `write_tag()`** — single-tag operations.
- **`read_tags_batch()`, `write_tags_batch()`, `execute_batch()`** — batch operations (multi-tag round trip).
- **`RoutePath`** — for routed connections through a ControlLogix backplane (slot/path).
- **`PlcValue`** — enum for PLC data types.
- **`upsert_tag_group()`, `subscribe_tag_group()`** — polling/subscription API. This is what we'll use to back `Driver::subscribe`.
- **`TagGroupEventKind`** — event classification: `Data`, `PartialError`, `ReadFailure`.
- **Async runtime:** Tokio 1.x with the runtime, macros, networking, time, synchronization, I/O, and signal features.
- **Validated targets:** CompactLogix (`5069-L320ERMS3`, firmware 35; `5069-L330ERM`, firmware 38) and ControlLogix (`1756-L81ES`, firmware 37; `1756-L75`, firmware 33).
- **Data types supported:** all 13 common AB types — `BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, STRING, UDT`.

### How we wrap it

`driver-rockwell` implements `openwebhmi_driver_api::Driver`:

| `Driver` method | Backed by `rust-ethernet-ip` API |
|---|---|
| `connect(config)` | `EipClient::new` + connection bring-up; `RoutePath` parsed from `config.route` |
| `disconnect()` | `EipClient::close` (or drop) |
| `read(addr)` | `read_tag()` for single, `read_tags_batch()` if upper layer batches |
| `write(addr, value)` | `read_tag()` to discover the target scalar type, range-checked `TagValue` conversion, then `write_tag()` |
| `subscribe(addrs)` | `upsert_tag_group()` + `subscribe_tag_group()` → translate events to `TagUpdate` stream |
| `browse(path)` | TBD — depends on whether the crate exposes tag introspection at the version we pin |

### Configuration shape

The driver instance config (passed to `Driver::connect`) is:

```json
{
  "host": "192.168.1.10",
  "slot": 0,
  "route": "1,0",
  "poll_rate_ms": 250,
  "connection_timeout_ms": 5000
}
```

Mapped to `EipClient` constructor + `RoutePath` per the crate's docs.

### Quality mapping

| `rust-ethernet-ip` outcome | `Driver` quality |
|---|---|
| Successful read/subscribe data | `Good` |
| `TagGroupEventKind::PartialError` for one tag | That tag → `Bad(read_error)`; others remain `Good` |
| `TagGroupEventKind::ReadFailure` (whole group) | All tags in group → `Bad(read_failure)` |
| Connection lost / timeout | All tags → `Bad(disconnected)` until reconnect succeeds |
| In-flight reconnect | All tags → `Uncertain(reconnecting)` |

## Evidence

All evidence below is sourced from the upstream `rust-ethernet-ip` repo at the **`v1.2.1` release**, which points to commit **`82d9160d84a0d89398c00b6f24c8d64d54d3370c`** (released 2026-08-22, observed by OpenWebHMI maintainers on 2026-08-25). Per `wiki/AGENTS.md` §2 (source authority hierarchy), this commit hash is the load-bearing source identifier — it survives upstream history rewrites, README edits, and tag deletions. URLs below are the channels through which the source was inspected; the commit hash is the artifact itself.

| Claim | Source | URL (channel) |
|---|---|---|
| Crate name `rust-ethernet-ip`, published version `1.2.1` | crates.io listing | https://crates.io/crates/rust-ethernet-ip/1.2.1 |
| Public API surface (`EipClient`, `read_tag`/`write_tag`, batch ops, `RoutePath`, `PlcValue`, `*_tag_group`, `TagGroupEventKind`) | README at `v1.2.1` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v1.2.1/README.md |
| Async runtime feature selection | `Cargo.toml` at `v1.2.1` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v1.2.1/Cargo.toml |
| Exact real-hardware targets and validation scope | README and compatibility program at `v1.2.1` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v1.2.1/docs/HARDWARE_COMPATIBILITY.md |
| Supported data types — `BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, STRING, UDT` | README at `v1.2.1` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v1.2.1/README.md |
| Handle-aware STRING and scalar UDT-array-member writes | `1.2.1` README limitations and `1.2.0` changelog history | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v1.2.1/CHANGELOG.md |
| Release date 2026-08-22 | GitHub release page for `v1.2.1` | https://github.com/sergiogallegos/rust-ethernet-ip/releases/tag/v1.2.1 |

### Independent verification status by OpenWebHMI

Each upstream claim above will be **independently verified** by OpenWebHMI maintainers before it is treated as load-bearing for `driver-rockwell` behavior. Status:

| Claim | OpenWebHMI verification | When |
|---|---|---|
| API surface compiles against our `Driver` trait wrapper | ✅ 0.7 verified by CODEX-F at `bc2d568`; 1.2.1 verified by the CODEX-BK driver unit and simulator suites, submission ref pending | Phase 1, 2026-04-26; CODEX-BK, 2026-08-25 |
| Validated PLC targets reproduce against our wrapper | ⏳ pending — no hardware yet | Pre-1.0 hardware gate |
| Subscription event semantics (`PartialError`, `ReadFailure`) match our quality mapping in §"Quality mapping" | ✅ verified by mocked `EipClientLike` unit tests and simulator shutdown/restart integration in CODEX-F, merged at `bc2d568`; gateway resubscribe/recovery path verified by CODEX-H e2e, pending merge commit ref | Phase 1, 2026-04-26 |
| Typed scalar write encoding | ✅ wrapper unit tests verify target-aware signed/unsigned integer variants, range rejection, and REAL/LREAL precision; real-PLC wrapper validation remains hardware-gated | CODEX-BK, 2026-08-25; full validation pre-1.0 |
| Direct STRING and UDT-member writes | ◐ upstream 1.2.1 carries real-hardware evidence; OpenWebHMI's wrapper STRING path is simulator-tested, while UDT writes remain out of scope | Upstream 1.2.1; OpenWebHMI full validation pre-1.0 |

Until each row flips from ⏳ to a dated commit/PR reference, downstream pages and `docs/architecture.md` should treat the corresponding behavior as *upstream-claimed, not yet verified by us*.

## Write-type mapping and limitations

OpenWebHMI reads the current tag before each write and uses the returned `PlcValue` variant as the target type. This adds one PLC round trip per operator write but avoids a duplicated type map and prevents CIP data-type-mismatch faults.

| OpenWebHMI value | Target PLC type | Encoding |
|---|---|---|
| `Int` | `SINT`, `INT`, `DINT`, `LINT` | Matching signed variant; narrowed widths use checked conversion |
| `Int` | `USINT`, `UINT`, `UDINT`, `ULINT` | Matching unsigned variant; negative and oversized values are rejected |
| `Real` | `REAL` | Narrow to `f32` |
| `Real` | `LREAL` | Preserve the full `f64` value |
| `Bool` | `BOOL` | Direct |
| `String` | `STRING` | Direct through the upstream handle-aware writer |

When no target metadata is supplied to the conversion helper, it retains the historical `DINT`/`REAL` defaults for compatibility. The production driver does not use that fallback. UDT writes remain unsupported by the OpenWebHMI wrapper even though upstream 1.2.1 supports more UDT operations.

### Migration from 0.7.0

- `EtherNetIpError` removed the specialized string read/write response variants and added an `Unsupported` variant; the wrapper maps the consolidated 1.2.1 surface.
- `EtherNetIpError` and `TagGroupEventKind` are non-exhaustive, so the wrapper preserves forward compatibility with fallback mappings.
- Direct STRING writes no longer use the old discarded-read workaround. The pre-write read now has one purpose for every scalar type: discovering the target type.
- Upstream `PlcValue` scalar variants and the tag-group APIs used by the wrapper remain source-compatible.

## Upgrade workflow

When `rust-ethernet-ip` publishes a new version on crates.io:

1. **Read the upstream changelog.** Note: API breaks, new device support, fixed limitations.
2. **Bump the workspace dep:** `Cargo.toml` → `rust-ethernet-ip = "X.Y"`. Use caret for compatible upgrades.
3. **Run `cargo update -p rust-ethernet-ip`** and inspect the crate plus direct-transitive lockfile changes. A maintainer-requested workspace-wide refresh is the explicit exception.
4. **Run the simulator-based driver test suite** in `crates/driver-rockwell/tests/`.
5. **If real hardware is available**, run the bench validation script (one-off, documented in `wiki/releases/`).
6. **Update the "as of X.Y.Z" markers** on this page and in `docs/feature-matrix.md`.
7. **Append a one-line entry to `wiki/log.md`.**
8. **If the upstream version fixed a limitation we'd documented**, add a `historical` status section linking to the prior behavior.

A new upstream major version (e.g. `0.x → 1.0`) gets its own PR with a checklist; we do not silently bump majors.

## Why a versioned dependency, not a submodule

- A submodule pins to a commit, which silently drifts from upstream releases. Versioned deps have explicit semver intent.
- The crate is published, tested, and documented as a standalone library for a wider audience than just OpenWebHMI. Forking via submodule would split its user base.
- Upgrade discipline (read changelog, run tests) is enforced by the version-bump ritual. Submodule bumps tend to slide through review unnoticed.

## Open questions

- `rust-ethernet-ip` 1.2.1 exposes `TagManager` discovery, but `Driver::browse` still returns the external tag-list import error. Decide whether to wire `TagManager` into the v1 browser or keep browse deferred.
- How does the crate behave under sustained subscription load (e.g. 5,000 tags at 250ms)? Need a Phase 1 perf experiment.
- Reconnect coordination: CODEX-F validates explicit driver reconnect and re-subscribe after simulator restart. OpenWebHMI still treats subscriptions as driver-instance scoped; the supervisor/gateway recreates them after reconnect rather than relying on transparent recovery inside the upstream object.

## Related pages

- [`docs/architecture.md`](../../docs/architecture.md) — §4.4 Driver model.
- [`docs/feature-matrix.md`](../../docs/feature-matrix.md) — §1 Connectivity.
- (future) `wiki/releases/0.1.0-validation-synthesis.md` — first simulator validation results.
- Upstream: https://github.com/sergiogallegos/rust-ethernet-ip
