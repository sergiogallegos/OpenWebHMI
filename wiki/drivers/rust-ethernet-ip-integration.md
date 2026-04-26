---
status: seed
last-validated: 2026-04-26
---

# `rust-ethernet-ip` integration

## Summary

`driver-rockwell` is OpenWebHMI's first PLC driver. It is a thin adapter that wraps the **`rust-ethernet-ip`** crate (https://github.com/sergiogallegos/rust-ethernet-ip) behind the OpenWebHMI `Driver` trait. The crate is owned and maintained by the same author as this project, but it is consumed as a **versioned dependency from crates.io** — not as a git submodule, not as a vendored copy. Upgrades happen by bumping the version in the workspace `Cargo.toml`.

This page is the durable maintainer reference for that integration: what the crate provides, how we wrap it, when to upgrade, what to validate at each upgrade, and where the boundaries are.

## Current understanding

### What the crate provides (as of 0.7.0)

- **`EipClient`** — main connection/communication interface to a Rockwell EtherNet/IP target.
- **`read_tag()`, `write_tag()`** — single-tag operations.
- **`read_tags_batch()`, `write_tags_batch()`, `execute_batch()`** — batch operations (multi-tag round trip).
- **`RoutePath`** — for routed connections through a ControlLogix backplane (slot/path).
- **`PlcValue`** — enum for PLC data types.
- **`upsert_tag_group()`, `subscribe_tag_group()`** — polling/subscription API. This is what we'll use to back `Driver::subscribe`.
- **`TagGroupEventKind`** — event classification: `Data`, `PartialError`, `ReadFailure`.
- **Async runtime:** Tokio (`tokio = { version = "1", features = ["full"] }`).
- **Validated targets:** CompactLogix (`5069-L320ERMS3`, firmware 35) and ControlLogix (`1756-L81ES`, firmware 37).
- **Data types supported:** all 13 common AB types — `BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, STRING, UDT`.

### How we wrap it

`driver-rockwell` implements `openwebhmi_driver_api::Driver`:

| `Driver` method | Backed by `rust-ethernet-ip` API |
|---|---|
| `connect(config)` | `EipClient::new` + connection bring-up; `RoutePath` parsed from `config.route` |
| `disconnect()` | `EipClient::close` (or drop) |
| `read(addr)` | `read_tag()` for single, `read_tags_batch()` if upper layer batches |
| `write(addr, value)` | `write_tag()` (with read-modify-write workaround when needed; see Limitations) |
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

All evidence below is sourced from the upstream `rust-ethernet-ip` repo at the **`v0.7.0` release**, which points to commit **`592bfa716309e3388cf8143c4095622d6302a7f6`** (released 2026-04-08, observed by OpenWebHMI maintainers on 2026-04-26). Per AGENTS.md §2, this commit hash is the load-bearing source identifier — it survives upstream history rewrites, README edits, and tag deletions. URLs below are the channels through which the source was inspected; the commit hash is the artifact itself.

| Claim | Source | URL (channel) |
|---|---|---|
| Crate name `rust-ethernet-ip`, latest published version `0.7.0` | crates.io listing | https://crates.io/crates/rust-ethernet-ip |
| Public API surface (`EipClient`, `read_tag`/`write_tag`, batch ops, `RoutePath`, `PlcValue`, `*_tag_group`, `TagGroupEventKind`) | README at `v0.7.0` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v0.7.0/README.md |
| Async runtime: Tokio with `features = ["full"]` | `Cargo.toml` at `v0.7.0` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v0.7.0/Cargo.toml |
| Validated PLC targets — `5069-L320ERMS3` (CompactLogix, fw 35), `1756-L81ES` (ControlLogix, fw 37) | README "Compatibility" at `v0.7.0` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v0.7.0/README.md |
| Supported data types — `BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, STRING, UDT` | README at `v0.7.0` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v0.7.0/README.md |
| Known limitations — direct STRING / UDT array element writes can fail on some firmware; read-modify-write recommended | README "Known Limitations" at `v0.7.0` | https://github.com/sergiogallegos/rust-ethernet-ip/blob/v0.7.0/README.md |
| Release date 2026-04-08 | GitHub release page for `v0.7.0` | https://github.com/sergiogallegos/rust-ethernet-ip/releases/tag/v0.7.0 |

### Independent verification status by OpenWebHMI

Each upstream claim above will be **independently verified** by OpenWebHMI maintainers before it is treated as load-bearing for `driver-rockwell` behavior. Status:

| Claim | OpenWebHMI verification | When |
|---|---|---|
| API surface compiles against our `Driver` trait wrapper | ✅ verified by `crates/driver-rockwell` in CODEX-F; pending merge commit ref | Phase 1, 2026-04-26 |
| Validated PLC targets reproduce against our wrapper | ⏳ pending — no hardware yet | Pre-1.0 hardware gate |
| Subscription event semantics (`PartialError`, `ReadFailure`) match our quality mapping in §"Quality mapping" | ✅ verified by mocked `EipClientLike` unit tests and simulator shutdown/restart integration in CODEX-F; pending merge commit ref | Phase 1, 2026-04-26 |
| String / UDT write workaround is needed and works | ◐ partially verified: CODEX-F verifies the wrapper's STRING read-before-write path with a mocked `EipClientLike`; firmware need/effectiveness remains hardware-gated | Phase 1 wrapper test, 2026-04-26; full validation pre-1.0 |

Until each row flips from ⏳ to a dated commit/PR reference, downstream pages and `docs/architecture.md` should treat the corresponding behavior as *upstream-claimed, not yet verified by us*.

## Limitations (from the upstream crate)

**Direct writes to standalone `STRING` tags and to UDT array element members can fail on some firmware versions.** The recommended workaround is **read-modify-write** at the parent UDT level. We need to mirror this guidance in `driver-rockwell` either by:

1. Detecting the firmware family on connect and silently doing read-modify-write for affected types, or
2. Returning a structured `WriteUnsupported` error that the gateway / scripts can choose to handle.

Decision deferred to Phase 1 implementation. Track in an issue when the work begins.

## Upgrade workflow

When `rust-ethernet-ip` publishes a new version on crates.io:

1. **Read the upstream changelog.** Note: API breaks, new device support, fixed limitations.
2. **Bump the workspace dep:** `Cargo.toml` → `rust-ethernet-ip = "X.Y"`. Use caret for compatible upgrades.
3. **Run `cargo update -p rust-ethernet-ip`** and verify only that crate moves.
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

- Does `rust-ethernet-ip` 0.7.x expose enough metadata for `Driver::browse`? If not, do we (a) gate browse on a future version, (b) implement our own EIP browse path, or (c) require the user to upload a tag list file?
- How does the crate behave under sustained subscription load (e.g. 5,000 tags at 250ms)? Need a Phase 1 perf experiment.
- Reconnect coordination: CODEX-F validates explicit driver reconnect and re-subscribe after simulator restart. The upstream `rust-ethernet-ip` 0.7.0 subscription object itself does not transparently recover a dead TCP stream in the OpenWebHMI wrapper path; the supervisor/gateway must recreate the driver subscription after reconnect.

## Related pages

- [`docs/architecture.md`](../../docs/architecture.md) — §4.4 Driver model.
- [`docs/feature-matrix.md`](../../docs/feature-matrix.md) — §1 Connectivity.
- (future) `wiki/releases/0.1.0-validation-synthesis.md` — first simulator validation results.
- Upstream: https://github.com/sergiogallegos/rust-ethernet-ip
