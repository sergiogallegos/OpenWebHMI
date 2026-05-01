---
status: active
last-validated: 2026-05-01
---

# OPC UA integration

## Summary

`driver-opcua` wraps `async-opcua` as OpenWebHMI's OPC UA client driver. v1 is client-only: the gateway connects to an OPC UA endpoint, reads/writes configured NodeIds, browses bounded address-space slices, and subscribes with OPC UA monitored items. `examples/sim-opcua` provides the deterministic simulator used by CI and manual smoke.

The original task brief named the older `opcua = "0.13"` package. At implementation time, the maintained crate on crates.io is `async-opcua` `0.18.0`, which re-exports the crate name `opcua` and provides async client and server APIs. OpenWebHMI pins that package exactly in the workspace.

## Current understanding

- Dependency: `async-opcua` is pinned at `0.18.0` in the workspace and resolved in `Cargo.lock` under package name `async-opcua`.
- Source identifier: the crate archive contains `.cargo_vcs_info.json` with upstream commit `5b854e3e2ff225038367d57807de3b36b5dd7c6c`.
- OpenWebHMI address shape is `ns=<namespace-index>;<id-form>=<id>`, for example `ns=2;s=Pressure` or `ns=4;i=1234`.
- Supported NodeId forms are string (`s`), numeric (`i`), GUID (`g`), and opaque base64 (`b`).
- Supported scalar value types are bool, signed integer, real/double, and string. Other OPC UA variants currently return an unsupported-type error.
- v1 auth supports anonymous and username/password sessions. Certificate-based auth is deferred.
- Default sampling and publishing interval is 250ms. v1 intentionally uses one interval for both.
- Browse defaults to `ObjectsFolder` and is capped by depth and breadth to avoid accidentally traversing very large namespaces.
- The simulator exposes mutable `Pressure` and `Counter` nodes and validates browse/read/write/subscribe through the real `async-opcua` client/server stack.

### Address syntax

| Shape | Meaning |
|---|---|
| `ns=2;s=Pressure` | String NodeId in namespace 2 |
| `ns=4;i=1234` | Numeric NodeId in namespace 4 |
| `ns=1;g=00000000-0000-0000-0000-000000000001` | GUID NodeId |
| `ns=1;b=AQID` | Opaque byte-string NodeId encoded as base64 |

Namespace indexes are server-specific. The same node may be `ns=2;s=Pressure` on one server and `ns=4;s=Pressure` on another, depending on that server's namespace array.

## Evidence

| Claim | Source | URL / path |
|---|---|---|
| `async-opcua` selected under the brief's allowed escape hatch | CODEX-W task brief and implementation | `docs/agents/tasks/CODEX-W-driver-opcua.md`, `Cargo.toml` |
| Resolved version is `0.18.0` | Cargo lockfile | `Cargo.lock` |
| Upstream commit for the crate archive is `5b854e3e2ff225038367d57807de3b36b5dd7c6c` | Cargo VCS metadata | `~/.cargo/registry/src/.../async-opcua-0.18.0/.cargo_vcs_info.json` |
| `async-opcua` provides async client and server APIs through the re-exported `opcua` crate name | Upstream crate manifest/source | `~/.cargo/registry/src/.../async-opcua-0.18.0/Cargo.toml.orig`, `src/lib.rs` |
| Address parser supports all four OPC UA NodeId forms | OpenWebHMI code | `crates/driver-opcua/src/address.rs` |
| Driver maps OPC UA reads/writes/subscriptions into the `Driver` trait | OpenWebHMI code | `crates/driver-opcua/src/driver.rs` |
| Simulator-backed integration validates read/write/browse/subscribe | OpenWebHMI test | `crates/driver-opcua/tests/integration.rs` |

### Independent verification status by OpenWebHMI

| Claim | OpenWebHMI verification | When |
|---|---|---|
| Address syntax accepts string/numeric/GUID/opaque NodeIds and rejects malformed forms | ✅ verified by `address.rs` unit tests | CODEX-W, 2026-05-01 |
| Driver metadata advertises OPC UA native subscribe, browse, batch read, and batch write | ✅ verified by `driver.rs` unit test | CODEX-W, 2026-05-01 |
| OPC UA status codes map into OpenWebHMI quality values | ✅ verified by `driver.rs` unit test | CODEX-W, 2026-05-01 |
| Client can read and write simulator values | ✅ verified by `cargo test -p openwebhmi-driver-opcua --features sim-tests` | CODEX-W, 2026-05-01 |
| Client can browse simulator nodes and receive monitored-item updates | ✅ verified by `cargo test -p openwebhmi-driver-opcua --features sim-tests` | CODEX-W, 2026-05-01 |
| Certificate-based production auth works against a real PLC/server | ⏳ pending — deferred to v1.1 / real-hardware gate | Pre-1.0 hardware gate |

## Limitations

- OpenWebHMI does not ship OPC UA server mode in v1.
- Certificate-based authentication and strict production certificate validation are not exposed through project config yet.
- Historical Access, method calls, events, and Alarms & Conditions are out of scope for v1.
- Structured values, arrays, extension objects, and vendor-specific complex types are not decoded yet.
- Large namespace browsing is intentionally capped by `browse_depth` and `browse_breadth`.
- The simulator has a namespace-index padding workaround because the upstream diagnostics node manager claims namespace 1. Tests use the namespace returned by `SimHandle::namespace_index()` rather than assuming a fixed `ns=`.

## Open questions

- Should v1 expose namespace URI resolution so users can write `uri=<namespace-uri>;s=Pressure` instead of hard-coding namespace indexes?
- Which certificate enrollment UX belongs in v1.1: trust-on-first-use, explicit import, or both?
- How should complex OPC UA variants surface in the tag model: reject, JSON-encode, or introduce structured `TagValue` variants?

## Related pages

- [`docs/feature-matrix.md`](../../docs/feature-matrix.md) — §1 Connectivity.
- [`docs/roadmap.md`](../../docs/roadmap.md) — Phase 4 driver expansion.
- [`docs/agents/tasks/CODEX-W-driver-opcua.md`](../../docs/agents/tasks/CODEX-W-driver-opcua.md) — implementation brief and review lifecycle.
