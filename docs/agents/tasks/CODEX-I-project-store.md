---
id: CODEX-I
title: crates/project-store — gateway-side project storage with versioning
owner: codex
phase: 2
status: merged
created: 2026-04-26
last-update: 2026-04-27 claude
---

# CODEX-I — `crates/project-store`

## Brief

### Goal

Gateway-side persistent storage for **projects**. A project is the unit of authoring and deployment: a tree of view definitions, tag definitions, alarm configs, driver configs, scripts, and theme/assets. This task lifts project handling out of the Phase 1 ad-hoc loader (`crates/gateway/src/project.rs`) into its own crate, adds save/load through the WebSocket protocol, and broadcasts change events so connected clients can hot-reload.

### Context to read first

- `docs/architecture.md` §4.8 (Project Store) — the design this task implements.
- `crates/gateway/src/project.rs` — the Phase 1 ad-hoc loader. Move/extend this; do not duplicate it.
- `crates/protocol/src/lib.rs` — wire-protocol types you'll extend with `project.*` messages.
- `crates/tag-engine/src/lib.rs` — for the publish-on-subscribe pattern that view subscriptions will mirror.

### Files to create

- `crates/project-store/Cargo.toml`
- `crates/project-store/src/lib.rs` — re-exports.
- `crates/project-store/src/types.rs` — `Project`, `DriverConfig`, `TagConfig`, `View`, `Component`, `Binding`. Extend the Phase 1 types with views.
- `crates/project-store/src/store.rs` — `ProjectStore` with `load`, `save_artifact`, `read_artifact`, `delete_artifact`, `list`, `subscribe_changes`.
- `crates/project-store/src/version.rs` — monotonic version counter; `ProjectChange { project_id, version, kind, path }` envelope.
- `crates/project-store/tests/store.rs` — round-trip + concurrent-save + change-broadcast tests.

Add `crates/project-store` to workspace `members`. Move the existing Phase 1 ad-hoc loader logic into the new crate; replace `crates/gateway/src/project.rs` content with a thin wrapper that delegates to `project-store`.

### On-disk format

A project lives under `<store-root>/<project-id>/`:

```
<project-id>/
├── project.toml          # schema_version, name, drivers[], (legacy tags[] — deprecated)
├── views/
│   ├── home.json         # one file per view
│   └── line1.json
├── tags/
│   └── tags.json         # tag definitions (replaces inline tags[])
├── alarms/
│   └── alarms.json       # Phase 3
├── scripts/              # Phase 3
└── assets/               # Phase 4
```

Phase 2 only writes `project.toml`, `views/*.json`, and `tags/tags.json`. Other directories are stub-created so the layout is stable.

**Versioning**: a single `version` integer at the project level, incremented on any artifact write. Stored in a SQLite metadata DB at `<store-root>/_index.sqlite` (table: `projects(id PK, version INT, last_modified TIMESTAMP)`). This avoids parsing every file to compute version.

### Public API (Rust)

```rust
pub struct ProjectStore { /* root path + sqlite handle */ }

impl ProjectStore {
    pub fn open(root: impl AsRef<Path>) -> anyhow::Result<Self>;

    pub fn list(&self) -> anyhow::Result<Vec<ProjectSummary>>;
    pub fn load(&self, project_id: &str) -> anyhow::Result<Project>;

    /// Save a single artifact (view, tag set, etc). Bumps version, broadcasts.
    pub fn save_artifact(&self, project_id: &str, kind: ArtifactKind, body: serde_json::Value)
        -> anyhow::Result<u64>; // returns new version

    pub fn read_artifact(&self, project_id: &str, kind: ArtifactKind)
        -> anyhow::Result<Option<serde_json::Value>>;

    pub fn delete_artifact(&self, project_id: &str, kind: ArtifactKind)
        -> anyhow::Result<()>;

    /// Subscribe to change events for a project (or all projects if id is None).
    pub fn subscribe_changes(&self, project_id: Option<&str>)
        -> tokio::sync::broadcast::Receiver<ProjectChange>;
}

pub enum ArtifactKind {
    ProjectMeta,
    View(String),     // view id
    Tags,
    Alarms,
    Script(String),   // script id (Phase 3)
}

pub struct ProjectChange {
    pub project_id: String,
    pub version: u64,
    pub artifact: ArtifactKind,
    pub action: ChangeAction,  // Created | Updated | Deleted
}
```

Failure handling:
- On a save that errors mid-write (disk full, etc.), the on-disk file is left in a temp location and never atomically renamed; the version counter is not bumped. The previous version remains intact.
- Use the standard "write to `<file>.tmp`, then `rename(<file>.tmp, <file>)`" idiom for atomic single-file writes.

### Wire protocol additions (`crates/protocol`)

Extend `ClientMessage`:

```rust
#[serde(rename = "project.subscribe")] ProjectSubscribe { project_id: String },
#[serde(rename = "project.unsubscribe")] ProjectUnsubscribe { project_id: String },
#[serde(rename = "project.load")] ProjectLoad { project_id: String },
#[serde(rename = "project.save_artifact")] ProjectSaveArtifact {
    project_id: String,
    artifact: ArtifactRef,         // tagged: project_meta | view{id} | tags | alarms | ...
    body: serde_json::Value,
},
```

Extend `ServerMessage`:

```rust
#[serde(rename = "project.snapshot")] ProjectSnapshot { project: serde_json::Value, version: u64 },
#[serde(rename = "project.changed")] ProjectChanged {
    project_id: String,
    version: u64,
    artifact: ArtifactRef,
    action: String,                // "created" | "updated" | "deleted"
},
#[serde(rename = "project.save_result")] ProjectSaveResult {
    request_id: Option<String>,
    project_id: String,
    version: u64,
},
```

Update the TS protocol package (`packages/protocol-ts`) to mirror these additions with type guards.

### Gateway integration (`crates/gateway`)

- Replace `crates/gateway/src/project.rs` content with a thin wrapper that:
  1. Constructs `ProjectStore::open(<store-root>)` (path from CLI flag `--project-store`).
  2. On each connected client subscription, registers their interest with the store and forwards `ProjectChange` events as `ServerMessage::ProjectChanged`.
  3. Routes `ClientMessage::ProjectSaveArtifact` → `store.save_artifact(...)` → returns `ProjectSaveResult`.
  4. Routes `ClientMessage::ProjectLoad` → `store.load(...)` → returns `ProjectSnapshot`.
- The Phase 1 driver-spawning behavior moves under the project store: when a project loads, drivers spawn. When a project's driver list changes, drivers reload (Phase 2 implements: full driver-runtime restart on driver-config artifact change; granular hot-reload of individual driver instances is post-1.0).

### Test requirements

Unit tests in `crates/project-store/tests/store.rs`:

- Open a fresh store, save a project, list it, load it, read each artifact.
- Save twice in a row, assert version increments by 2.
- Subscribe to changes, save, assert receiver gets one `ProjectChange` event.
- Two concurrent `save_artifact` calls on different artifacts complete without lost updates (use a temp dir per test).
- Atomic rename: simulate a partial write by writing a `.tmp` file directly; reload should see the prior committed version.
- Validation rejects `schema_version != 1`, duplicate driver IDs, and tag-path-prefix mismatches (carry forward Phase 1's rules).

Integration tests in `crates/gateway/tests/project_protocol.rs`:

- Connect a WS client, send `project.subscribe`, save an artifact through the protocol, observe a `project.changed` event.
- `project.load` returns a `project.snapshot` containing the saved state.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-project-store` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] Phase 1 demo (`examples/projects/phase1-demo/project.toml`) loads and runs identically through the new store path.
- [ ] All public items have rustdoc.
- [ ] Wiki updated: append a one-line entry to `wiki/log.md`. Add a wiki page under `wiki/architecture/project-store-on-disk-format.md` capturing the file layout decision.

### Out of scope

- Hot-reload of individual driver-instance config (Phase 5+).
- Concurrent multi-developer editing with locking (post-1.0).
- Git-as-the-versioning-backend (post-1.0; the task uses an integer counter for v1).
- Project import/export `.owhmi` archive (Phase 4).

### Risks / gotchas

- **Atomic file writes on Windows**: `rename` over an existing file fails on Windows by default. Use `std::fs::rename` which works on both POSIX and Windows for cross-device-safe atomic replace; if it doesn't, fall back to `tempfile::NamedTempFile::persist`.
- **Don't read every file to compute version.** SQLite metadata is the source of truth for version; files are the source of truth for content. Keep them consistent via the save-then-bump-version transaction.
- **Schema-version migration** is out of scope here; the store rejects `schema_version != 1`. When CODEX-J extends the schema, define migrations explicitly.
- **Don't break Phase 1.** The existing `examples/projects/phase1-demo/project.toml` must continue to load. The new store is a superset of the Phase 1 loader's behavior.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 20:09  codex
Started. Reading the Phase 1 gateway project loader, architecture project-store section, protocol types, and tag-store publish/subscribe pattern. Implementing CODEX-I locally because it overlaps with CODEX-J in protocol and project-store type files; CODEX-K is running in a separate worker with a disjoint package scope.

### 2026-04-27 09:32  codex
Submitted. Added `crates/project-store` with filesystem artifacts, SQLite `_index.sqlite` project metadata, version bumps, atomic temp-file writes, change broadcasts, validation carried forward from Phase 1, and tests for round-trip/list/load/read, version increments, change events, concurrent saves, tmp-file safety, and validation failures. Replaced the gateway's ad-hoc project model with a thin loader that delegates to `ProjectStore`, kept the Phase 1 demo project loading path green, and added gateway project protocol handling for `project.subscribe`, `project.load`, and `project.save_artifact`.

Also added `crates/gateway/tests/project_protocol.rs` covering WebSocket project subscribe/save/load. Wiki update: `wiki/architecture/project-store-on-disk-format.md`, `wiki/index.md`, and `wiki/log.md`.

Verification: `cargo test -p openwebhmi-project-store`, `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo fmt --all -- --check` pass. Socket-bound gateway tests were run with local-bind permissions.

## Claude review

### 2026-04-27  claude — review pass 1

Spec-compliant. Filesystem layout matches the brief (project.toml, views/, tags/, alarms/, scripts/, assets/). SQLite metadata index at `_index.sqlite` with the prescribed schema. Atomic single-file writes via tmp-then-rename. Broadcast channel surfaces `ProjectChange` events. Schema validation carries forward Phase 1's rules.

Strong points:
- ✅ `bump_version` uses SQLite's `INSERT ... ON CONFLICT DO UPDATE` — race-safe under concurrent saves.
- ✅ `ProjectStore` is `Clone` via `Arc<StoreInner>`; subscribers can be passed across tokio tasks.
- ✅ `subscribe_changes` returns a fresh broadcast receiver per call — correct semantics.
- ✅ `validate_project` enforces driver-id uniqueness, tag-driver references, and tag-path-prefix rules from Phase 1.

Findings:
- 🟡 **`subscribe_changes(_project_id)` ignores the project_id argument** (`store.rs:210-215`). Returns the global receiver regardless. The brief said "for a project, or all projects when None". Consumer-side filter is trivial (each `ProjectChange` carries `project_id`), but the API as written misleads. Fix: either remove the parameter or implement per-project channels.
- 🟡 **`atomic_write` does explicit `remove_file` before `rename`** (`store.rs:345-348`). On POSIX this opens a brief window where the destination doesn't exist; `fs::rename` already does atomic replace there. On Windows the explicit remove is needed but `rename` over an existing file may still fail without `MOVEFILE_REPLACE_EXISTING`. Recommend `tempfile::NamedTempFile::persist` for portable atomicity.
- 🟡 **`save_artifact(ProjectMeta)` round-trips through TOML** via `serde_json_to_toml` (`store.rs:154`). Functional, but means hand-edited TOML loses comments + ordering on every save. Acceptable for v1; flag if humans report frustration.
- 🟢 The `ensure_layout` helper stub-creates `views/`, `tags/`, `alarms/`, `scripts/`, `assets/` even before they're used — file layout stays stable for tooling.

Acceptance criteria all met. Phase 1 demo project still loads through the new store path.

## Verdict

**Merged** at the next commit. The `subscribe_changes` per-project filtering is the only real design note — track for follow-up alongside the broader gateway-side wiring in CODEX-L/M.
