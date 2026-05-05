---
status: active
last-validated: 2026-05-05
---

# API surface stability

## Summary
CODEX-AL established the v1.0 Rust API pattern for tag-path and driver-id identifiers, future-extensible public surfaces, and cheap-clone runtime handles. The JSON wire protocol remains string-shaped for these identifiers.

## Current understanding
1. Runtime tag paths use `openwebhmi_protocol::TagPath`, a transparent string newtype with `Deref<Target = str>`, `Display`, `Hash`, and serde-transparent JSON. Source: `crates/protocol/src/lib.rs`; regression test: `crates/protocol/tests/identifiers.rs`.
2. Project-local driver instance ids use `openwebhmi_protocol::DriverId`; gateway driver handles are keyed by `DriverId`, while PLC-side `TagAddress` remains in `driver-api` and is not promoted to a runtime path. Source: `crates/protocol/src/lib.rs`, `crates/gateway/src/project.rs`, `crates/driver-api/src/address.rs`.
3. The public WebSocket message enums are `#[non_exhaustive]`; gateway message handling has a wildcard arm that logs unhandled future client message variants. Source: `crates/protocol/src/lib.rs`, `crates/gateway/src/server.rs`.
4. Public growth surfaces that are likely to add fields or variants before or after v1.0 are marked `#[non_exhaustive]`, including driver configs, backup/restore options, audit query/event types, and public error enums. Source: driver config modules, `crates/backup/src/lib.rs`, `crates/audit-log/src`, `crates/auth/src`, `crates/driver-api/src/error.rs`.
5. `ScriptHost` is a cheap-clone value type with its shared task/control state behind an internal `Arc`; gateway no longer exposes `Arc<ScriptHost>`. Source: `crates/scripting/src/host.rs`, `crates/gateway/src/main.rs`, `crates/gateway/src/server.rs`.

## Evidence
- `crates/protocol/tests/identifiers.rs` verifies transparent JSON for `TagPath` and `DriverId`, byte-identical `tag.write` JSON, and required wildcard matching for non-exhaustive message enums.
- `crates/scripting/src/host.rs` has a unit test confirming cloned `ScriptHost` values share the same inner `Arc`.
- `cargo check --workspace --all-targets` passed after the CODEX-AL API migration.

## Open questions
- `ProjectStore` schema types still store tag paths and driver ids as `String` because `openwebhmi_protocol` currently depends on `openwebhmi_project_store` for wire `View`/`ArtifactKind` types. Moving identifiers beneath both crates, or decoupling protocol from project-store view types, would be a v1.1 design change.
- Other identifier newtypes such as `ProjectId`, `ScriptId`, `AlarmId`, `SessionId`, and `UserId` remain v1.1 follow-ups.

## Related pages
- [async-runtime-hygiene.md](async-runtime-hygiene.md)
- [audit-log.md](audit-log.md)
- [backup-restore.md](backup-restore.md)
