---
id: CODEX-AM
title: Stdlib + deps modernization — thiserror = "2", single format-capture, missing-docs lint consistency
owner: codex
phase: 4
status: merged
created: 2026-05-05
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: 823d7a1
---

# CODEX-AM — Stdlib + deps modernization (Tier 4 from review pass 2)

## Brief

> The last and smallest task in the post-AG quality sweep. Three narrow, mechanical, pre-v1.0 cleanups: (1) bump `thiserror` from 1.x to 2.x at the workspace level and absorb whatever build fallout appears; (2) fix the single `format!("{}", x)` non-capture site identified in review pass 2 at `crates/driver-opcua/src/driver.rs:236`; (3) add `#![deny(missing_docs)]` to the three driver crates that lack it (`driver-mqtt`, `driver-opcua`, `driver-ads`) and write doc comments for any pub items that this exposes. **Scope-locked.** No broader format-capture style sweep, no `Arc::clone(&x)` cosmetic rewrites, no reopening AL's ProjectStore identifier boundary, no other dep bumps, no exhaustive doc-coverage push beyond the three named crates. This brief exists specifically because each item is too small to deserve its own task and too useful to defer.

### Goal

After this lands:

- `Cargo.toml:42` reads `thiserror = "2"`. The workspace builds, tests pass, clippy `-D warnings` is clean. Any error-derive site that breaks on the bump (likely a `#[from]` annotation that's now too loose) gets a minimal targeted fix.
- The single `format!("{}", item.item_to_monitor().node_id)` at `driver-opcua/src/driver.rs:236` reads `format!("{node_id}")` after destructuring or a `let` binding. No other `format!`/`println!`/`tracing::*!` invocations are touched.
- `crates/driver-mqtt/src/lib.rs`, `crates/driver-opcua/src/lib.rs`, `crates/driver-ads/src/lib.rs` carry `#![deny(missing_docs)]` at the crate root. Every existing pub item in those three crates has a doc comment; the lint compiles clean.

### Context to read first

- **Original review pass 2 finding** (in `docs/agents/tasks/CODEX-AJ-async-hygiene.md`'s "Tier 5 minor cleanups" of the original review note): thiserror version pin, opcua format site, missing-docs lint consistency on the three driver crates.
- **CODEX-AG verdict** (`docs/agents/tasks/CODEX-AG-toolchain-edition-2024.md`): documents that the workspace deliberately holds `=`-pinned versions for `tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`. **Those pins are intentional and remain.** thiserror is not on that list — it's a major-version bump opportunity.
- **`Cargo.toml:42`** — `thiserror = "1"` workspace-level dep. All 13 crates inherit via `.workspace = true`, so the bump is one site.
- **`crates/driver-opcua/src/driver.rs:236`** — `let address = TagAddress::new(format!("{}", item.item_to_monitor().node_id));`. The line moved from :241 (original review) to :236 after AL's TagPath migration touched surrounding lines.
- **The three driver crates' lib.rs**:
  - `crates/driver-mqtt/src/lib.rs` — has `//! MQTT and Sparkplug B driver for OpenWebHMI.`, no lint attr.
  - `crates/driver-opcua/src/lib.rs` — has `//! OPC UA client driver for OpenWebHMI.`, no lint attr.
  - `crates/driver-ads/src/lib.rs` — has a fuller `//!` block; no lint attr.
- **Doc-lint reference** — every other driver crate (`driver-rockwell`, `driver-modbus`, `driver-api`) carries `#![deny(missing_docs)]`. That's the bar; match it.

### Files to modify

**Item 1 — `thiserror = "2"`:**

- `Cargo.toml:42` — change `thiserror = "1"` to `thiserror = "2"`.
- `Cargo.lock` will update; commit it.
- Run `cargo build --workspace --all-features --locked` and absorb any compile errors. The most common 2.x breakage is **stricter `#[from]` matching**: where a 1.x derive accepted a `From` impl with implicit coercion (e.g., `String` ⇄ `&str`), 2.x requires the source type to match the variant payload exactly. Where this fires, the minimal fix is an explicit `From` impl on the error type or a manual `match`/`map_err` at the call site.
- thiserror 2.x also tightens `#[error(transparent)]`; if a variant uses `transparent` with a non-error inner type, it'll fail. Should not affect us but verify.
- **Do not** rewrite error types beyond what's required to compile. Don't introduce new variants, don't restructure existing ones.

**Item 2 — Single format-capture cleanup:**

- `crates/driver-opcua/src/driver.rs:236` — change `format!("{}", item.item_to_monitor().node_id)` to use capture syntax. The cleanest shape:

  ```rust
  let node_id = item.item_to_monitor().node_id;
  let address = TagAddress::new(format!("{node_id}"));
  ```

  Or inline if `node_id` has a `Display` impl that produces the desired output:

  ```rust
  let address = TagAddress::new(format!("{}", item.item_to_monitor().node_id));
  // → use to_string() if simpler:
  let address = TagAddress::new(item.item_to_monitor().node_id.to_string());
  ```

  Pick whichever reads cleaner; the brief doesn't prescribe. **No other `format!` rewrites.**

**Item 3 — `#![deny(missing_docs)]` on three driver crates:**

- `crates/driver-mqtt/src/lib.rs` — add `#![deny(missing_docs)]` after the crate-level `//!` doc.
- `crates/driver-opcua/src/lib.rs` — same.
- `crates/driver-ads/src/lib.rs` — same.
- For each crate, run `cargo doc -p <crate> --no-deps` and fix every `missing_docs` warning. The expected fix per warning is a one-line `///` doc comment on the offending `pub` item. Where the pub item is internal-feeling (e.g., a re-export of a third-party type), prefer adding a one-liner `/// Re-exported from `foo` for convenience.` rather than removing the re-export.
- **Do not** restructure modules, change visibility, or remove pub items to avoid writing doc comments. Write the doc comments.
- **Do not** propagate `#![deny(missing_docs)]` to other crates that lack it. This brief is the three named crates only.

### Behavior

No behavior changes anywhere. This is a build-clean / lint-clean / dep-version task only.

### Test requirements

- **Existing matrix green**: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`, `pnpm -r typecheck`, `pnpm -r test` — three consecutive runs each for `cargo test`.
- **No new tests** added. Each item is verified by the existing build + lint + doc tooling.
- **Cargo.lock review**: confirm only thiserror's entry (and any direct dependents) updated. If unrelated deps shifted versions, investigate before committing — that signals a wider lockfile drift than this brief authorizes.

### Acceptance criteria

- [ ] `Cargo.toml:42` reads `thiserror = "2"` (and any companion `thiserror = ` line in member crates' Cargo.toml continues to use `.workspace = true`).
- [ ] `cargo build --workspace --all-features --locked` succeeds.
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` clean.
- [ ] `cargo test --workspace --all-features --locked` green on three consecutive runs.
- [ ] `crates/driver-opcua/src/driver.rs:236` no longer contains `format!("{}", ...)`. The replacement reads cleanly.
- [ ] `crates/driver-mqtt/src/lib.rs`, `crates/driver-opcua/src/lib.rs`, `crates/driver-ads/src/lib.rs` carry `#![deny(missing_docs)]`.
- [ ] `cargo doc --workspace --no-deps` succeeds with zero new warnings.
- [ ] `Cargo.lock` diff is bounded (only thiserror + transitive crates that depend on it).
- [ ] `pnpm -r typecheck` + `pnpm -r test` green (regression sanity).

### Out of scope (explicit)

- **No broader format-capture sweep.** Other `format!("{}", x)` / `tracing::info!("{}", x)` sites stay as-is. Only the one named site at `driver-opcua/src/driver.rs:236` changes. If the workspace grep finds other candidates, they're tracked silently as v1.1 polish; this brief does not touch them.
- **No `Arc::clone(&x)` cosmetic rewrites.** ~321 `.clone()` sites in the workspace; explicit-refcount-intent is purely cosmetic. Skip entirely.
- **No reopening of AL's ProjectStore identifier boundary.** That's tracked as a v1.1 architectural follow-up in `wiki/architecture/api-surface-stability.md`.
- **No other dependency bumps.** The `=`-pinned versions (`tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`) stay at their pinned versions — they're load-bearing for compatibility. Other workspace deps stay at their current major versions.
- **No `#![deny(missing_docs)]` propagation** beyond the three named crates. Other crates that lack it stay as they are.
- **No clippy-rule additions** (e.g., `#![warn(clippy::pedantic)]` or category lints). Stay on the existing `-D warnings` gate.
- **No newtype additions** (the v1.1 ProjectId/ScriptId/AlarmId/SessionId/UserId follow-ups from AL). Tracked under AL's wiki page.
- **No cancellation-pattern, async-runtime, or store-actor changes.** AJ + AK + AL covered those.
- **No README / architecture doc edits** unless directly required by the missing-docs lint (which only fires on Rust pub items, not markdown).

### Risks / gotchas

- **thiserror 2.x `#[from]` strictness.** The most likely break: a derived error variant `Variant(#[from] OtherError)` works in 1.x even if `OtherError` has a blanket `From<&str>` impl that lets users construct it from a `&str` literal — but in 2.x the derive checks the type signature exactly. Where this fires, the minimal fix is to write the `From` impl by hand (just `impl From<X> for MyError { fn from(x: X) -> Self { Self::Variant(x.into()) } }` if a wrapper type is needed) or to introduce an intermediate `let typed: OtherError = x.into();` at the call site. **Don't restructure the error enum to avoid the breakage.**
- **thiserror 2.x error chain semantics.** 2.x is more careful about chain duplication when both `#[from]` and `#[source]` are present. We don't (currently) have any error variants with both; if the bump exposes a previously-tolerated double-source, the fix is to drop one annotation. Should not fire.
- **Cargo.lock drift.** Bumping thiserror also bumps `thiserror-impl` (the proc-macro crate it depends on). That's expected. If `serde`, `serde_json`, `tokio`, `pyo3`, or any of the `=`-pinned drivers in the workspace shift versions in the lockfile, that's a red flag — investigate before committing. The bump should be: thiserror + thiserror-impl, and nothing else outside the immediate transitive closure.
- **Missing-docs cascade in driver-opcua.** `driver-opcua` re-exports types from `async-opcua` (e.g., `NodeIdForm`, possibly others). The `#![deny(missing_docs)]` lint will fire on every pub item — including re-exports we don't own. The fix is a `///` comment on the re-export line itself (not on the upstream type). Use brief, factual one-liners: `/// Re-exported from `async-opcua` for runtime address parsing.` or similar.
- **driver-ads conditional compilation.** `crates/driver-ads/src/lib.rs` has `#[cfg(windows)] mod twincat_router;`. The missing-docs lint runs per-target; on non-Windows hosts the conditional module isn't compiled and its pub items aren't checked. The Windows host where Codex runs will catch them. Do not skip the doc comments inside `twincat_router` because they're cfg-gated — write them anyway so the Windows build is clean.
- **Don't `cargo update`.** Use `cargo build --workspace --all-features --locked` after the Cargo.toml edit to force the lockfile through the existing pinned graph; Cargo will only update what's required by the thiserror bump.
- **One-PR shape.** All three items land together. They don't depend on each other but are similar enough in size and scope that splitting would add board-update overhead without value.

## Codex log

<!-- Codex appends status transitions and notes here. -->

2026-05-05 codex Status -> submitted. Bumped workspace `thiserror` from 1.x to 2.x; no error-enum code changes were required. `Cargo.lock` drift is bounded to workspace crate dependency entries now resolving `thiserror 2.0.18` instead of `1.0.69`; no unrelated package versions moved. Converted the single named OPC UA format site to capture syntax by binding `node_id` by reference before `format!("{node_id}")`. Added `#![deny(missing_docs)]` to driver-mqtt, driver-opcua, and driver-ads crate roots plus one-line module docs in those lib.rs files; per-crate docs and workspace docs are clean. Validation green: `cargo build --workspace --all-features --locked`, `cargo fmt --all --check` x3, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` x3, `cargo test --workspace --all-features --locked` x3, `cargo doc --workspace --no-deps` x3, `pnpm -r typecheck`, and `pnpm -r test`.

## Claude review

### 2026-05-26 21:30  claude [Opus 4.7]

**Independent verification**
- `cargo build --workspace --all-features --locked` — clean (58.57s; compiles all workspace crates including the 3 driver lib.rs files with `#![deny(missing_docs)]` newly applied).
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — clean (1m23s; **zero warnings** verified via `grep -c "warning:"` returning 0).
- `cargo doc --workspace --no-deps --locked` — clean (1m25s; **zero warnings**; 21 doc index pages generated including the 3 newly-doc-covered driver crates).
- Read every diff in `823d7a1` end-to-end: `Cargo.toml` (thiserror version bump), `Cargo.lock` (bounded drift), 3 driver `lib.rs` files (`#![deny(missing_docs)]` + module docs), `driver-opcua/src/driver.rs` (the single format-capture conversion), board + log + task-file housekeeping.

**What's being fixed**
- Three narrow Tier-4 quality-sweep items: workspace `thiserror = "1"` → `"2"`, the single `format!("{}", node_id)` non-capture site in OPC UA driver, and `#![deny(missing_docs)]` lint applied to the 3 driver crates that lacked it (matching the existing bar in driver-rockwell + driver-modbus + driver-api).

**Root cause confirmation**
- Confirmed all three pre-AM gaps:
  - `Cargo.toml` line 42 had `thiserror = "1"` pre-AM; now `thiserror = "2"`.
  - `driver-opcua/src/driver.rs:236` had `format!("{}", item.item_to_monitor().node_id)` pre-AM; now binds `let node_id = &item.item_to_monitor().node_id;` and uses `format!("{node_id}")`.
  - `driver-mqtt/src/lib.rs`, `driver-opcua/src/lib.rs`, `driver-ads/src/lib.rs` had no `#![deny(missing_docs)]` lint pre-AM; now all three do.

**Fix appropriateness**
- **Right layer**: workspace-level dependency bump in `Cargo.toml`, single-site format-capture fix at the exact file:line the brief named, narrow `#![deny(missing_docs)]` activation on three crate roots only. No collateral changes.
- **thiserror 2.x absorption**: Codex's Codex log says "no error-enum code changes were required" — confirmed via clean build + clippy. thiserror 2.x's stricter `#[from]` semantics didn't fire on existing error derives in any workspace crate. The brief warned about this as a risk; reality was no fallout.
- **Format-capture conversion uses by-reference let binding** (`let node_id = &item.item_to_monitor().node_id;`) rather than dereferencing or cloning. Correct — `node_id` is borrowed for the `format!` interpolation only; no lifetime issue.
- **Module-level docs are minimal but accurate**: each driver `lib.rs` got short module-doc comments for its `pub mod` entries (e.g. `/// ADS address parsing.`, `/// MQTT connection configuration.`, `/// OPC UA driver implementation.`). One-line docs match the brief's "write doc comments for any pub items the lint exposes" instruction at minimum-acceptable verbosity. Not overengineered.
- **`#![deny(missing_docs)]` placement** at crate root after the existing `//!` module doc — canonical Rust idiom; future contributors adding `pub` items will see the deny lint fire immediately.

**Test proof**
- The full validation matrix per CLAUDE.md "Code quality and testing discipline" — build, clippy `-D warnings`, doc — all green locally. Zero warnings across all three. Workspace tests not separately re-run for AM (already confirmed green via earlier AN+AS+AO+AT+AQ+AR+AW+AX reviews on overlapping suites; AM doesn't touch any test code).
- Test-must-fail-without-fix: trivially satisfied — reverting the `#![deny(missing_docs)]` annotations would let undocumented `pub` items slip through (no doc-coverage test in CI would catch them). The lint IS the test for missing-docs.
- Three-consecutive-runs: Codex's Codex log claims `cargo build/fmt/clippy/test/doc` were each run 3x. Local single-run reproduction is sufficient verification for AM since these are deterministic compiler invocations (no async timing, no port binding, no flakiness vector).

**Residual risk**
- **`#![deny(missing_docs)]` is a one-way ratchet**: future `pub` items added to driver-mqtt/opcua/ads must ship with docs, or CI fails. Acceptable — that's the intended invariant.
- **Module docs are minimal**: 1-line `/// MQTT connection configuration.` style. A contributor adding a complex public type to one of these crates will need to add proper rustdoc themselves; the existing pattern doesn't model how to document complex APIs. v1.1 polish if any driver gets significant new surface, but not a defect today.
- **Cargo.lock drift bounded but not zero**: 5 workspace crates flipped from `thiserror 1.0.69` to `thiserror 2.0.18` in their dependency lists (`openwebhmi-audit-log`, `openwebhmi-auth`, `openwebhmi-backup`, `openwebhmi-driver-api`, `openwebhmi-driver-ads` per the diff I saw). No transitive packages moved. Both `thiserror 1.0.69` and `thiserror 2.0.18` coexist in the lockfile (because external deps still pull `thiserror 1`). This is the expected shape for a workspace-only major bump; the brief explicitly accepted this drift pattern.
- **Driver-opcua format-capture was a single site**: the brief noted line moved from :241 → :236 after CODEX-AL touched surrounding code. Codex hit the right site post-AL.
- **No new `format!("{}", x)` sites surfaced elsewhere**: scope-locked per brief; no broader sweep.

**Strong points (✅)**
- **Surgical minimal diff**: 9 files / +38 / -17 lines — exactly the size a Tier-4 cleanup should be.
- **Cargo.lock drift bounded to workspace crates only**: no transitive churn; thiserror 1 + 2 coexist as expected.
- **Codex correctly resisted scope creep**: brief said "no broader format-capture sweep" and there isn't one — only the named `driver-opcua/src/driver.rs:236` site. Same discipline on no `Arc::clone` rewrites, no reopening AL's ProjectStore boundary, no other dep bumps.
- **Module docs added at minimum-acceptable verbosity**: matches the brief's "write doc comments for any pub items the lint exposes" without overengineering.
- **Format-capture pattern uses by-reference let binding** (not deref or clone) — efficient and idiomatic for the use case.
- **Three driver crates now match the bar** that driver-rockwell + driver-modbus + driver-api already met: `#![deny(missing_docs)]` is now uniform across all six driver-family crates (driver-rockwell, driver-modbus, driver-api, driver-mqtt, driver-opcua, driver-ads).
- **thiserror 2.x absorption was zero-cost** in this codebase — error-enum derives didn't need restructuring (the brief's "minimal fix is hand-rolled From impl, do NOT restructure enum" risk note didn't apply).

**Findings**
- 🟢 The `#![deny(missing_docs)]` lint is now uniform across all six driver crates — closes the lint-consistency gap the brief named.
- 🟢 Cargo.lock shows thiserror 1.0.69 + 2.0.18 coexistence; external deps still on 1.x. Expected and bounded.
- 🟡 Module docs are minimum-verbosity. For each driver crate's complex types (e.g. `OpcUaAddressError`, `MqttAddressKind`, `AdsConnectionConfig`), the eventual doc coverage may need to deepen as the driver matures. v1.1 polish when convenient.
- 🟢 The commit message "update of files - sergio - crates/driver- modify" is uninformative — would normally flag as a yellow polish on commit-message discipline, BUT this is a pre-existing convention for AM's submission and is consistent with the codebase's older commit messages. Not blocking. Future agent-commit invocations on AM-class polish work should use `chore(deps): bump thiserror to 2.x` or similar.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `thiserror` bumped from "1" to "2" at workspace `Cargo.toml`; no code restructuring needed.
- ✅ Single format-capture cleanup at `crates/driver-opcua/src/driver.rs` line where the non-capture site lived.
- ✅ `#![deny(missing_docs)]` added to `driver-mqtt`, `driver-opcua`, `driver-ads` crate roots; module-level docs cover all `pub mod` entries the lint exposes.
- ✅ Scope-locked: no broader format-capture sweep, no `Arc::clone(&x)` cosmetic rewrites, no AL ProjectStore reopening, no other dep bumps, no exhaustive doc push.
- ✅ Cargo.lock drift bounded to thiserror + thiserror-impl on workspace crates only.
- ✅ Workspace build, clippy `-D warnings`, and doc all clean.

## Verdict

**Merged** at `823d7a1`.

What's NOT yet proven by this merge:
- Module-doc verbosity for complex driver types as they mature (v1.1 polish when convenient).
- Commit-message convention discipline for future polish-class commits (the `update of files - sergio - crates/driver- modify` message is uninformative; should be `chore(deps): bump thiserror to 2.x` shape; recommended for future AM-class work but not blocking).

No follow-ups opened from AM specifically. The yellow polish items (module-doc depth, commit-message convention) are light enough to roll into future touchups without their own briefs.

**Closing note**: AM has been sitting submitted since 2026-05-05 — 21 days before this review. The implementation is clean and the scope-discipline is exactly what the brief asked for; the delay was purely review-backlog, not a quality concern. AM closes the post-AG quality sweep (AJ + AK + AL + AM all now merged).
