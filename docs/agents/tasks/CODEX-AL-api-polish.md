---
id: CODEX-AL
title: API polish — TagPath + DriverId newtypes, #[non_exhaustive] sweep, ScriptHost cheap-clone
owner: codex
phase: 4
status: merged
created: 2026-05-05
last-update: 2026-05-05 claude
---

# CODEX-AL — API polish (Tier 3 from review pass 2)

## Brief

> Three SemVer-relevant items that should land **before v1.0 tags** so the public API isn't locked into stringly-typed identifiers and exhaustive-by-default enums. (1) Two newtype wrappers — `TagPath` and `DriverId` — replace the bare `String` shape currently used across protocol message variants, driver-config structs, alarm-engine definitions, historian APIs, scripting RPC types, and gateway routing logic. Wire format unchanged (`#[serde(transparent)]`). The two newtypes establish the *pattern*; future identifier newtypes (`ProjectId`, `ScriptId`, `AlarmId`, `SessionId`, `UserId`) are explicit v1.1 follow-ups, not in this brief. (2) `#[non_exhaustive]` annotation sweep across every public struct/enum that may grow post-1.0 — driver configs, the two top-level message enums, restore/backup options, public error enums, status types, etc. The codebase has zero `#[non_exhaustive]` today; this is a one-time mechanical sweep. (3) `ScriptHost` cheap-clone refactor — replace the three `Arc<ScriptHost>` exposures with a `Clone`-able `ScriptHost` value type that holds the Arc internally, matching the pattern `HistorianStore` / `AuditLog` / `AlarmJournal` already follow. The `TagStore` / `ProjectStore` / `UserStore` audit confirms they're already cheap-clone; only `ScriptHost` is left exposed.
>
> **Scope-locked.** No driver-config Builder + immutable Config split (separate v1.1+ ergonomic improvement; serde-deserialization is the dominant construction path so the ROI is low). No other identifier newtypes (5 future candidates listed in Out of scope). No `thiserror = "2"` bump, no format-capture sweep, no missing-docs lint consistency — those land as CODEX-AM. No store-actor refactor, no cancellation-pattern changes (covered by AJ/AK).

### Goal

After this lands:

- Every public function signature, struct field, and enum variant payload that currently carries a tag path uses `TagPath`. Every one that carries a driver instance id uses `DriverId`. The wire protocol JSON shape is byte-identical to today (transparent serde).
- Every public struct / enum across the workspace that is reasonably expected to grow post-1.0 carries `#[non_exhaustive]`. Internal-only types stay as-is.
- `ScriptHost` is `#[derive(Clone)]` cheap-clone. The three current `Arc<ScriptHost>` sites take or return `ScriptHost` directly. `OnceLock<Arc<ScriptHost>>` becomes `OnceLock<ScriptHost>`.
- Type-checking surfaces the kind of mistake we keep almost making: passing a driver id where a tag path is expected, or vice versa, becomes a compile error rather than a runtime parse failure.

### Context to read first

- **CODEX-AJ + CODEX-AK verdicts** — runtime-health and tokio-handle ergonomics work that AL builds on. Both shipped the discipline that "API polish stays in its own task."
- **Existing newtype shape**: `crates/protocol/src/lib.rs` does not currently define `TagPath` or `DriverId`. Both are introduced here.
- **`tag_path: String` sites in protocol** (initial inventory; do a workspace grep for the rest):
  - `crates/protocol/src/lib.rs:231` — `ClientMessage::TagWrite { tag_path: String, ... }`
  - `crates/protocol/src/lib.rs:398` — `ServerMessage::AlarmEvent { tag_path: String, ... }`
  - `crates/protocol/src/lib.rs:453` — `ServerMessage::HistoryResult { tag_path: String, ... }`
  - Plus deser test fixtures (lines 686+, ts test fixtures) — these stay as JSON strings; only the Rust struct field type changes.
- **Driver instance id sites** (the field is named `driver_id`, sometimes `driver`, occasionally just embedded in a path prefix like `"rockwell-1/Pressure"`):
  - `crates/gateway/src/project.rs` — `DriverConfig.id: String`
  - `crates/protocol` — wherever a driver's identity appears in a wire message
  - `crates/driver-api/src/types.rs` — driver-api carries `TagAddress` (PLC-side address), which is **distinct** from `DriverId` and **does not change**. Don't conflate the two.
- **Top-level message enums for `#[non_exhaustive]`** — `crates/protocol/src/lib.rs:133` `pub enum ClientMessage` and `:373` `pub enum ServerMessage`. Both have many variants and are guaranteed to grow.
- **`Arc<ScriptHost>` exposures**:
  - `crates/gateway/src/server.rs:43` — `static DEFAULT_SCRIPT_HOST: OnceLock<Arc<ScriptHost>>`
  - `crates/gateway/src/server.rs:84` — `pub fn set_default_script_host(host: Arc<ScriptHost>)`
  - `crates/gateway/src/main.rs:238` — `fn spawn_script_runtime(...) -> Option<Arc<ScriptHost>>`
- **Already-cheap-clone stores** (do **not** modify; they're the pattern target):
  - `HistorianStore`, `AuditLog`, `AlarmJournal`, `TagStore`, `ProjectStore`, `UserStore` — all `#[derive(Clone)]` with Arc inside.

### Files to modify

**Item 1 — `TagPath` + `DriverId` newtypes:**

- `crates/protocol/src/lib.rs` — add two newtype definitions near the top of the file (or in a new submodule `crates/protocol/src/identifiers.rs` if the file is getting long; pick whichever shape fits the existing layout). Each newtype is:

  ```rust
  /// Fully qualified path to a tag in the runtime (e.g. "rockwell-1/Pressure").
  /// Wire format is transparent string.
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  #[serde(transparent)]
  pub struct TagPath(String);

  impl TagPath {
      /// Construct from any string-like input.
      pub fn new(value: impl Into<String>) -> Self {
          Self(value.into())
      }
      /// Borrow the underlying string slice.
      pub fn as_str(&self) -> &str {
          &self.0
      }
      /// Consume and return the underlying String.
      pub fn into_string(self) -> String {
          self.0
      }
  }

  impl From<&str> for TagPath { fn from(s: &str) -> Self { Self(s.to_owned()) } }
  impl From<String> for TagPath { fn from(s: String) -> Self { Self(s) } }
  impl AsRef<str> for TagPath { fn as_ref(&self) -> &str { &self.0 } }
  impl std::ops::Deref for TagPath { type Target = str; fn deref(&self) -> &str { &self.0 } }
  impl std::fmt::Display for TagPath { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) } }

  // ts-rs annotation: emit as TS `string` directly so the existing protocol-ts
  // wire types stay unchanged.
  #[cfg(feature = "ts-rs")]
  impl ts_rs::TS for TagPath { /* delegates to String */ }
  ```

  Same shape for `DriverId`.

  **Notes:** include the ts-rs delegation if the protocol crate has the ts-rs feature; check existing patterns. The existing protocol-ts package (`packages/protocol-ts`) must continue to type these fields as `string` — verify after the change with `pnpm -r typecheck`.

- **Replace `String` with `TagPath` at every relevant site** in:
  - `crates/protocol/src/lib.rs` — message variants. Run a diligent grep for `tag_path:` and `path:` (the latter is more ambiguous; check the type context). Also `addresses: Vec<TagPath>` if any list-of-paths field exists.
  - `crates/tag-engine/src/lib.rs` — `TagSnapshot.path` field, `TagStore::publish/subscribe/get` parameters.
  - `crates/historian/src/store.rs` and `recorder.rs` — `HistorianStore::write_sample/read` parameters, `HistoryTagConfig.path`.
  - `crates/alarm-engine/src/types.rs` — `AlarmDefinition.tag_path`, `ActiveAlarm.tag_path`, `AlarmTransition.tag_path`.
  - `crates/audit-log/src/event.rs` — wherever AuditEvent payloads carry tag paths (search the file).
  - `crates/scripting/src/rpc.rs` — `TagChangeArgs.tag_path`, RPC method args that take paths.
  - `crates/scripting/src/sink.rs` — `TagWriteSink::enqueue` parameter.
  - `crates/gateway/src/script_writes.rs`, `crates/gateway/src/server.rs`, `crates/gateway/src/project.rs` — every site that handles a tag path.

- **Replace `String` with `DriverId` at every driver-instance-id site:**
  - `crates/gateway/src/project.rs` — `DriverConfig.id`, `DriverHandles` map keys (decide: keep `String` map keys for hashing simplicity or use `DriverId`; either works since `DriverId` impls `Hash` + `Eq`).
  - `crates/driver-api/src/supervisor.rs` — supervisor instance identification.
  - Any wire message that names a driver instance.

- **Inside drivers themselves**: `TagAddress` (PLC-side address, e.g. an EtherNet/IP tag name + datatype) is **NOT** a TagPath — it's the layer below. Do not change `TagAddress`. The `addresses: Vec<TagAddress>` parameter to `Driver::read/subscribe` stays `TagAddress`. The PLC-side path prefix that the gateway uses to *route* writes (e.g. `"rockwell-1/Pressure"` where `"rockwell-1"` is the driver id and `"Pressure"` is the tag) splits into `(DriverId, TagPath-suffix)` — but the splitting logic is gateway-internal; the *suffix* is still a `String` at the driver-API boundary.

**Item 2 — `#[non_exhaustive]` sweep:**

Add `#[non_exhaustive]` to:

- `crates/protocol/src/lib.rs` — `ClientMessage` (line 133), `ServerMessage` (line 373), and any public config struct that may grow (e.g. `BackupOptions`, `RestoreOptions`, `Aggregation` enum if it may add variants, etc.). **Check every `pub enum` and `pub struct` in the file.**
- `crates/driver-rockwell/src/...`, `crates/driver-modbus/src/...`, `crates/driver-opcua/src/...`, `crates/driver-mqtt/src/...`, `crates/driver-ads/src/...` — driver-config structs. Each driver has a `Config` or similar; annotate.
- `crates/audit-log/src/event.rs` — `AuditEvent` enum (will grow as new event kinds are added; e.g. the brief flagged `SessionExpired` as a v1.1 follow-up).
- `crates/audit-log/src/query.rs` — `AuditQuery` struct (filters will grow).
- `crates/backup/src/...` — `RestoreOptions`, `BackupManifest`, etc.
- `crates/auth/src/...` — error enums that may grow.
- `crates/driver-api/src/types.rs` — public error / capability / metadata types that aren't intended to be matched exhaustively by callers.

**Do not** add `#[non_exhaustive]` to:
- `TagValue` (the wire-protocol value enum — adding variants is a real breaking change anyway because it changes JSON schema; keep exhaustive so callers know they need new arms).
- `Quality` (small, stable, three variants, complete coverage of OPC UA quality semantics).
- The new `TagPath` / `DriverId` newtypes (single-tuple struct; no growth direction).
- Any `enum` with three or fewer variants where adding a fourth would be a real wire-protocol change.

When `#[non_exhaustive]` lands on `ClientMessage` / `ServerMessage`, every match against them must add `_ =>`. Do the rewrite.

**Item 3 — `ScriptHost` cheap-clone:**

- `crates/scripting/src/host.rs` — change `pub struct ScriptHost { ... }` to hold an internal `Arc<ScriptHostInner>`. Add `#[derive(Clone)]`. Move the existing fields into `ScriptHostInner`. Public API stays the same.
- `crates/gateway/src/server.rs:43` — `static DEFAULT_SCRIPT_HOST: OnceLock<Arc<ScriptHost>>` becomes `OnceLock<ScriptHost>`.
- `crates/gateway/src/server.rs:84` — `pub fn set_default_script_host(host: Arc<ScriptHost>)` becomes `pub fn set_default_script_host(host: ScriptHost)`.
- `crates/gateway/src/main.rs:238` — `fn spawn_script_runtime(...) -> Option<Arc<ScriptHost>>` becomes `Option<ScriptHost>`.
- All call sites that previously did `host.clone()` (cloning the Arc) keep working — `ScriptHost::clone` is now the Arc-internal cheap-clone.

### Behavior

**Wire protocol unchanged.** Every JSON message that currently has `"tag_path": "rockwell-1/Pressure"` continues to have exactly that. The Rust-side type changes from `String` to `TagPath` but `#[serde(transparent)]` makes the JSON shape identical.

**TS protocol-ts crate unchanged.** Verify with `pnpm -r typecheck` after the change. If `TagPath` ends up as `{ "0": "..." }` instead of `"..."` in the ts-rs output, the `#[serde(transparent)]` annotation isn't being picked up by ts-rs — fix the ts-rs annotation (likely a `#[ts(type = "string")]` attribute).

**Pattern-matching against `ClientMessage` / `ServerMessage` requires `_ =>`.** Add it everywhere these enums are matched. The arm should `warn!("unhandled message kind: {:?}")` rather than silently doing nothing — this preserves observability when new variants are added.

**`ScriptHost` value semantics.** `ScriptHost` becomes `Clone` via internal Arc. Call sites that previously did `Arc::clone(&host)` or `host.clone()` work identically (and are now both spelled `host.clone()`). Lifetime is unchanged: as long as one `ScriptHost` exists, the inner state is alive.

### Test requirements

- **Existing matrix green**: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`, `pnpm -r typecheck`, `pnpm -r test` — three consecutive runs each for `cargo test`.
- **Wire-format round-trip test** in `crates/protocol/tests/identifiers.rs` (new):
  - A struct containing `TagPath` and `DriverId` round-trips via serde to/from JSON without changing string form.
  - The serialized JSON for `TagWrite { tag_path: TagPath::new("a/b") }` is byte-identical to today's `TagWrite { tag_path: "a/b".to_string() }` form (dump both, compare).
- **Type-safety test** (compile-fail or runtime assertion):
  - A function taking `TagPath` cannot be called with a `DriverId` and vice-versa. A compile-fail test using `compile_fail` doc tests is the canonical shape, but a unit test that just exercises `From` / `Display` / `Deref` is sufficient if `compile_fail` is heavy.
- **`#[non_exhaustive]` regression check**:
  - Add a small unit test that exhaustive-matches `ClientMessage` / `ServerMessage` *with* a `_ =>` arm and confirms no warning fires (proves the annotation is in place AND the `_ =>` is mandatory).
- **`ScriptHost::clone` cheap-clone check**:
  - Unit test in `crates/scripting/tests/...` confirming that two clones of a `ScriptHost` share the same internal `Arc` (identity check via `Arc::ptr_eq` if exposed, or behavioral check via state mutation).
- **`pnpm -r typecheck` + `pnpm -r test`** — must stay green. The protocol-ts package is auto-derived from the Rust types via ts-rs; the newtype change must not perturb the TS shape.
- **Manual smoke (record findings in the Codex log):** load an existing project (any of the simulator examples), connect a runtime-web client, write a tag, ack an alarm, query history. Confirm no JSON-shape regression. (This is short; doesn't require hardware.)

### Acceptance criteria

- [ ] `pub struct TagPath(String)` and `pub struct DriverId(String)` defined in `crates/protocol`. Both are `Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`, with `From<&str>`, `From<String>`, `AsRef<str>`, `Deref<Target=str>`, `Display` impls.
- [ ] Every site in the workspace that currently carries a tag path as `String` uses `TagPath`; every site that carries a driver instance id uses `DriverId`. Workspace `cargo check --workspace --all-targets` clean.
- [ ] `#[non_exhaustive]` added to every public struct/enum in the workspace that's reasonably expected to grow post-1.0 (driver configs, message enums, restore/backup options, audit event/query, public error enums). Enums that are intentionally exhaustive (`TagValue`, `Quality`) explicitly noted in their doc comments as such.
- [ ] Every match against `ClientMessage` / `ServerMessage` now has a `_ =>` arm with a `warn!("unhandled ...")` log.
- [ ] `ScriptHost` is `#[derive(Clone)]` cheap-clone. The three `Arc<ScriptHost>` exposures (`server.rs:43`, `server.rs:84`, `main.rs:238`) take or return `ScriptHost` directly.
- [ ] Wire-format round-trip test confirms JSON byte-identical for `TagPath` / `DriverId` fields.
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` clean.
- [ ] `pnpm -r typecheck` green (protocol-ts unchanged at the TS layer).
- [ ] Manual smoke: existing project loads, tag write succeeds, alarm acks, history queries — all without JSON-shape regression. Findings recorded in the Codex log.

### Out of scope (explicit)

- **No other identifier newtypes.** `ProjectId`, `ScriptId`, `AlarmId`, `SessionId`, `UserId` stay as `String` for now. Two newtypes (`TagPath`, `DriverId`) are the highest-confused-pair; the brief establishes the pattern. Adding the others is a v1.1 follow-up that can land mechanically with the same shape.
- **No driver-config Builder + immutable Config split.** Configs are deserialized from project JSON via serde; programmatic construction is rare. Builder pattern lower ROI; deferred indefinitely.
- **No cheap-clone refactor for stores other than `ScriptHost`.** `TagStore`, `HistorianStore`, `AuditLog`, `AlarmJournal`, `ProjectStore`, `UserStore` are already `#[derive(Clone)]` cheap-clone — no change needed.
- **No restructuring of stores into actor-style services.** AJ explicitly deferred this.
- **No `thiserror = "2"` bump.** CODEX-AM.
- **No `format!` capture-syntax sweep.** CODEX-AM.
- **No missing-docs lint consistency** across driver-mqtt/opcua/ads. CODEX-AM.
- **No newtype validation logic.** `TagPath::new(s)` accepts any string. We do **not** validate that paths are non-empty, well-formed, or lack illegal characters. Validation is a separate concern; if added, it goes in a `TagPath::parse() -> Result<...>` constructor alongside the unchecked `new`.
- **No `#[non_exhaustive]` on internal-only types.** Only public surface.
- **No expansion to additional patterns from the design-system roadmap.** That work is post-1.0.

### Risks / gotchas

- **ts-rs and `#[serde(transparent)]`.** The ts-rs derive macro may not honor `#[serde(transparent)]` automatically. If `pnpm -r typecheck` after the change shows TS field types changing from `string` to `{ "0": string }`, fix with `#[ts(type = "string")]` on the newtype's ts-rs annotation. The protocol-ts package consumers (component-library, runtime-web, designer) all assume `string` shape.
- **`DriverHandles` map keys.** Currently `HashMap<String, DriverHandle>`. Decide: keep `String` keys (cheap; `DriverId` derefs to `str` so lookup with a `&DriverId` works via the `Borrow` trait) or change to `HashMap<DriverId, DriverHandle>`. Either is fine; the brief mildly prefers `HashMap<DriverId, ...>` for type consistency, but if it bloats the diff, `String` keys are acceptable. Document the choice in code.
- **`split_once` and path arithmetic.** Gateway code that splits a fully qualified path like `"rockwell-1/Pressure"` into `(driver_id, suffix)` currently does `path.split_once('/')`. After the change, this still works on `&str` (via Deref), but the suffix that gets passed to the driver API stays `String` / `&str`, NOT `TagPath`. Don't accidentally promote it.
- **`#[non_exhaustive]` is a one-way SemVer ratchet.** Once shipped as part of v1.0, removing it is technically backwards-compatible but practically suspicious. Be conservative — don't annotate enums that are genuinely fixed-set (state machines with bounded states, e.g. `AlarmState`). Brief errs on the side of *more* `#[non_exhaustive]`; flag any enum where you think the brief is wrong rather than silently skipping the annotation.
- **`Arc<ScriptHost>` ↔ `ScriptHost` migration.** When the public API changes from `Arc<ScriptHost>` to `ScriptHost`, every internal call site that does `host.clone()` continues to compile (now cloning the cheap value type instead of cloning the Arc). But sites that explicitly construct `Arc::new(ScriptHost { ... })` change to just `ScriptHost { ... }` (then `set_default_script_host(host)` instead of `Arc::clone(&Arc::new(host))` etc.). Watch for double-Arc patterns.
- **`OnceLock<Arc<ScriptHost>>` semantics.** Today `DEFAULT_SCRIPT_HOST.get().cloned()` returns `Option<Arc<ScriptHost>>` — a clone of the Arc. After the change, `OnceLock<ScriptHost>::get().cloned()` returns `Option<ScriptHost>` — a cheap-clone of the value. Same memory shape, simpler type.
- **Don't migrate `TagAddress`.** It's the PLC-side address (EtherNet/IP tag, Modbus register, OPC UA NodeId — varies per driver), structurally different from a tag path. Conflating them is the brief's biggest correctness risk.
- **Big diff.** Plan for ~1500-2500 lines of changes across 15+ files. Submit as a single PR; reviewable but dense. The mechanical nature (every `tag_path: String` → `tag_path: TagPath`) means the diff is large but each individual change is trivial.

## Codex log

<!-- Codex appends status transitions and notes here. -->

2026-05-05 codex Status -> submitted. Added `TagPath` and `DriverId` transparent newtypes in `openwebhmi-protocol` and migrated runtime tag-path surfaces through protocol messages, tag-engine snapshots/store APIs, historian configs/store APIs, alarm definitions/events, audit tag-write payloads, scripting RPC/sink APIs, and gateway write/subscription routing. Gateway `DriverHandles` is keyed by `DriverId`; `TagAddress` remains driver-native. Added `#[non_exhaustive]` to `ClientMessage`, `ServerMessage`, driver configs, backup/restore options, audit event/query types, public driver/auth error enums, and related growable public metadata; fixed the gateway `ClientMessage` match with a wildcard warn arm. Refactored `ScriptHost` into a cheap-clone value type with an internal `Arc`, replacing the three `Arc<ScriptHost>` gateway exposures. Tests added: `crates/protocol/tests/identifiers.rs` for JSON byte identity, type separation, and wildcard matching; `ScriptHost` clone identity test. Validation green: `cargo fmt --all --check` x3, `cargo clippy --workspace --all-targets --all-features -- -D warnings` x3, `cargo test --workspace --all-features --locked` x3, `cargo doc --workspace --no-deps`, `pnpm -r typecheck`, and `pnpm -r test`. Manual browser/project smoke was not run in this environment. Note: `ProjectStore` schema structs still carry tag paths/driver ids as `String` because `openwebhmi-protocol` depends on project-store for wire `View`/`ArtifactKind`; this boundary is documented in `wiki/architecture/api-surface-stability.md` as a v1.1 design follow-up.

## Claude review

### Strong points

- ✅ **`string_newtype!` macro** (`crates/protocol/src/lib.rs:16-77`) DRYs up the boilerplate. Both `TagPath` and `DriverId` come from one declaration each (lines 79-93). Future identifier newtypes (`ProjectId`, `ScriptId`, `AlarmId`, `SessionId`, `UserId` — the v1.1 follow-ups) drop in as one-line additions. Right structural call.
- ✅ **`Borrow<str>` impl included.** Brief listed `AsRef<str>` and `Deref` but not `Borrow<str>`. Codex correctly identified that without `Borrow<str>`, every HashMap lookup via `&str` against `HashMap<TagPath, _>` would force conversion at the call site. The `Borrow<str>` impl is load-bearing for the gateway's `DriverHandles: HashMap<DriverId, _>` lookups elsewhere. Bonus that's actually critical.
- ✅ **`PartialOrd, Ord` derives, `From<&String>`, `From<&Self>`** — minor ergonomic adds beyond brief. Make the newtypes feel like first-class string types in sorted collections and at trait boundaries.
- ✅ **`TagValue` and `Quality` doc comments** explicitly note "intentionally exhaustive" with rationale (lines 191-194, 209-211). Per brief discipline; future contributors don't accidentally add `#[non_exhaustive]` here.
- ✅ **`#[non_exhaustive]` sweep correctly applied**: `ClientMessage`, `ServerMessage`, `AuditQuery`, plus all 5 driver configs (`config.rs` / `connection.rs` files in each driver crate got 1-line additions), `RestoreOptions`, `BackupOptions`, the public error enums in `auth`, `audit-log`, `driver-api`. Exhaustive coverage of the growth surfaces.
- ✅ **Wildcard match arm** at `gateway/src/server.rs` has the exact pattern brief specified: `_ => { warn!(message = ?client_message, "unhandled client message kind"); }`. Preserves observability when new variants land.
- ✅ **`ScriptHost` cheap-clone refactor structurally clean**: new `ScriptHostInner` struct holds the previously-direct fields; `ScriptHost { inner: Arc<ScriptHostInner> }` with `#[derive(Clone)]`. `task: Mutex<Option<JoinHandle<()>>>` — `Mutex` for interior mutability through `Arc`, `Option` for `take()` on shutdown. Both required, both correct.
- ✅ **Comprehensive identifier test** at `crates/protocol/tests/identifiers.rs`:
  - `identifier_newtypes_are_transparent_json_strings` — wire-format byte-identical round-trip.
  - `tag_write_json_shape_matches_pre_newtype_wire_form` — TagWrite specifically (the most load-bearing message), asserts byte-identical pre/post-newtype JSON.
  - `tag_path_and_driver_id_are_not_interchangeable_types` — TypeId comparison + function-arg overload proves the type-safety win.
  - `non_exhaustive_messages_are_matched_with_wildcard_arms` — verifies the annotation enforces `_ =>`.
- ✅ **`script_host_clone_shares_inner_state` test** uses `Arc::ptr_eq(&host.inner, &clone.inner)` — the canonical cheap-clone identity check.
- ✅ **`DriverHandles` keys by `DriverId`** (the brief's mildly-preferred option). Type-consistency at the gateway routing layer.
- ✅ **`TagAddress` correctly NOT migrated** (per brief discipline). Wiki page explicitly calls out the distinction so future contributors don't conflate the PLC-side address with the runtime tag path.
- ✅ **Wiki page (`wiki/architecture/api-surface-stability.md`)** is honest and complete. Lists what AL accomplished, evidence (with the new test names), and the **two** legitimate "Open questions": (1) ProjectStore schema caveat (see findings); (2) other identifier newtypes as v1.1 follow-ups.
- ✅ **Diff size much smaller than estimated** — 48 files / +399 / -266 (vs the brief's 1500-2500 estimate). The macro DRYs up the boilerplate; most call sites are 1-character type changes. Same correctness, less to review.
- ✅ **Internal scripting refactors tracked correctly**: `tag_paths: &'a [String]` → `&'a [TagPath]` in `ReadyWorkerRuntime`, `StreamMap<String, BroadcastStream<TagSnapshot>>` → `StreamMap<TagPath, BroadcastStream<TagSnapshot>>` in `run_ready_worker`. Internal types follow the public newtype, no boundary leak.

### Bonuses beyond brief

- ✅ **Honest scope-boundary discipline** on the `ProjectStore` schema. `crates/protocol/src/lib.rs:13` re-exports `View`/`ArtifactKind` from `openwebhmi_project_store`, which means promoting `View.tag_path` to `TagPath` would create a circular crate dependency (project-store imports protocol's TagPath; protocol re-exports project-store's View). Codex correctly stopped at the boundary, documented the caveat in the wiki, and explicitly flagged it as a v1.1 design follow-up. Not trying to force the migration through is the correct call.
- ✅ **Macro design choice** — using `string_newtype!` as a declarative macro (rather than copy-pasting the impl block twice) means the v1.1 identifier-newtype additions cost one line of code each. Anticipates the follow-up work.

### Findings

- 🟡 **`ProjectStore` schema caveat owned but not closed.** `View.tag_path` and `View.driver_id` (project-store schema types reachable from project JSON) still carry these as `String`. Closing this requires either (a) moving `View`/`ArtifactKind` from `project-store` to `protocol`, (b) defining the identifier newtypes in a new lower-level crate that both depend on, or (c) inverting the dependency so `protocol` is consumed by `project-store`. Each is a real architectural decision. **Tracked** as v1.1 design follow-up in the wiki page's "Open questions". Not a merge blocker.
- 🟡 **Manual project-load smoke deferred** (same as AJ/AK). Codex correctly didn't fake it. The wire-format round-trip tests provide JSON shape assurance; a real project load + tag write + alarm ack + history query would catch any deserialization-edge-case regression that the unit tests miss. Maintainer-action item.
- 🟡 **No verification of the v1.1 design follow-up plan.** The wiki "Open questions" #1 lists three candidate approaches for the ProjectStore schema migration but doesn't recommend one. When AL2 (or whatever picks this up) is briefed, that decision needs to be made first.

### Independent verification

- `cargo fmt --all --check` — ✅ clean.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — ✅ clean.
- `cargo test --workspace --all-features --locked` — ✅ green (single workspace run on my side; Codex documents three consecutive runs).
- `cargo doc --workspace --no-deps` — ✅ green; newtype doc comments render cleanly.
- `pnpm -r typecheck` — ✅ green. **Critical AL gate**: TS shape preserved, `tag_path: string` and `driver_id: string` in protocol-ts unchanged. The `#[serde(transparent)]` annotation honored by the ts-rs derive without manual `#[ts(type = "string")]` override.
- `pnpm -r test` — ✅ green.
- Read the load-bearing files (protocol/src/lib.rs newtype + non_exhaustive sweep, scripting/src/host.rs cheap-clone, gateway/src/server.rs wildcard arm, the new tests, the new wiki page) plus spot-check of driver crates and the `DriverHandles: HashMap<DriverId, _>` migration.

### Acceptance-criteria tally

- [x] `pub struct TagPath(String)` and `pub struct DriverId(String)` defined in `crates/protocol`. Both `Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize` (last two via `#[serde(transparent)]`), with `From<&str>`, `From<String>`, `From<&String>`, `From<&Self>`, `AsRef<str>`, `Borrow<str>`, `Deref<Target=str>`, `Display` impls.
- [x] Every site in the workspace that carries a tag path uses `TagPath`; driver instance ids use `DriverId`. ProjectStore schema types deferred per documented v1.1 architectural caveat.
- [x] `#[non_exhaustive]` added to the brief's growth surfaces (driver configs, message enums, restore/backup options, audit event/query, public error enums). `TagValue`/`Quality` doc-commented as intentionally exhaustive.
- [x] Every match against `ClientMessage` / `ServerMessage` has a `_ =>` arm with a `warn!` log.
- [x] `ScriptHost` is `#[derive(Clone)]` cheap-clone. The three `Arc<ScriptHost>` exposures (`server.rs:43, 84`, `main.rs:238`) take or return `ScriptHost` directly.
- [x] Wire-format round-trip test confirms JSON byte-identical (`identifier_newtypes_are_transparent_json_strings`, `tag_write_json_shape_matches_pre_newtype_wire_form`).
- [x] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` clean.
- [x] `pnpm -r typecheck` + `pnpm -r test` green.
- [~] Manual smoke deferred to maintainer hardware/runtime session (same as AJ/AK).

## Verdict

**Merged.** TagPath + DriverId newtypes via a `string_newtype!` macro that makes future identifier newtypes one-liners; transparent JSON shape preserved (verified by `pnpm -r typecheck` keeping protocol-ts unchanged + byte-identical round-trip tests); `#[non_exhaustive]` sweep across the brief's growth surfaces with `TagValue`/`Quality` documented as intentionally exhaustive; `ClientMessage` wildcard arm with `warn!`-level observability; `ScriptHost` cheap-clone refactor with `Arc<ScriptHostInner>` and a `Mutex<Option<JoinHandle>>` to make `shutdown(self)` work cleanly under cheap-clone semantics; comprehensive new test coverage including TypeId-based proof that `TagPath ≠ DriverId` at the type level.

The diff turned out smaller than briefed (~400 net lines vs the 1500-2500 estimate) because the macro DRYs the newtype boilerplate and most call-site changes are 1-character type renames. The `Borrow<str>` impl beyond brief is load-bearing — without it, every `&str` lookup against a `HashMap<TagPath, _>` would force conversion at the call site.

Codex held scope-discipline on the **ProjectStore schema caveat**: `protocol::lib.rs:13` re-exports `View`/`ArtifactKind` from `project-store`, so promoting `View.tag_path` to `TagPath` would create a circular crate dependency. Codex correctly stopped, documented three architectural-decision options in the wiki, and tracked it as v1.1 design work rather than forcing the migration through. **This is the fourth time this sprint Codex has caught an in-scope boundary the brief missed** (after AJ's alarm-engine async refactor, AK's SupervisorHandle::Drop, and now this); pattern noted across the verdicts.

Three v1.1 follow-ups surface and are tracked in the wiki:
1. ProjectStore schema migration (architectural decision required first).
2. Other identifier newtypes — `ProjectId`, `ScriptId`, `AlarmId`, `SessionId`, `UserId` — drop in as one-line `string_newtype!` additions once the pattern is settled.
3. The deferred manual project-load smoke (same as AJ/AK; maintainer hardware/runtime session).

Phase 4 quality sweep status: AJ + AK + AL merged. **CODEX-AM** (`thiserror = "2"` / format-capture / missing-docs lint consistency) is the last queued Tier 4 task. None of the AJ/AK/AL items blocked v1.0; all four reviews were tightly scoped, three caught in-scope brief gaps, all four merged on first review.
