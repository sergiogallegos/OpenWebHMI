---
id: CODEX-R
title: AlarmTable component + alarm config UI
owner: codex
phase: 3
status: merged
created: 2026-04-27
last-update: 2026-04-28 claude
blocked-by: CODEX-Q
---

# CODEX-R — `AlarmTable` + alarm config UI

## Brief

> **Blocked by [CODEX-Q](CODEX-Q-alarm-engine.md).** Needs `alarm.subscribe` / `alarm.event` / `alarm.ack` wire forms.

### Goal

Two pieces:
1. **`AlarmTable` component** — runtime live alarm table with filtering by priority and ack from UI.
2. **Designer alarm config** — module for authoring `AlarmDefinition`s per project.

### Files to create / modify

- `packages/component-library/src/components/AlarmTable.tsx` + tests.
- `packages/component-library/src/registry.ts` — register `AlarmTable`.
- `apps/runtime-web/src/lib/alarmClient.ts` (or extend `gatewayClient.ts`) — `subscribeAlarms(opts, cb)`, `ackAlarm(id, note)`.
- `apps/designer/src/modules/AlarmConfig.tsx` — list + add + edit + delete alarm definitions; saves through `project.save_artifact { Alarms }`.
- `apps/designer/src/App.tsx` — surface Alarms in the project explorer.

### `AlarmTable` behavior

- Subscribes to `alarm.subscribe { project_id }` on mount.
- Renders rows per active alarm: priority badge (color by priority), label, tag value, activated_at, state (Active/Acked/Cleared).
- Sortable by priority (default), state, time.
- Filter chips: priority bands (1-2 critical, 3 warning, 4-5 info), state (active/all).
- Click row → "Ack" button + optional note input. Sends `alarm.ack`.
- Cleared alarms fade out after 5 seconds (configurable prop); user can pin to keep them.
- Bad-quality bound-tag alarms render a small `?` indicator next to the value (alarm engine still fires; but the operator should know the value was unreliable).

### Designer `AlarmConfig` module

- Tree by tag: `rockwell-1/Pressure → [HighLimit 200, LowLimit 50]`.
- Form per alarm: condition kind switch (HighLimit / LowLimit / Equals / Deviation / Digital), threshold, priority dropdown, label, message template, require_ack checkbox.
- Save through `project.save_artifact { Alarms }` with the standard 300ms debounce.

### Test requirements

- `AlarmTable.test.tsx`: renders synthetic events, ack button calls callback with id + note, priority filtering hides out-of-range rows.
- `AlarmConfig.test.tsx`: editing a threshold updates the alarm definition; saving fires the gateway client.
- Manual smoke step added to `apps/designer/README.md`: define a `Pressure > 200` alarm in the designer, force the simulator to a high value, observe the alarm in `AlarmTable`, ack it.

### Acceptance criteria

- [ ] `pnpm -r typecheck && test` green.
- [ ] Component library exports `AlarmTable`.
- [ ] Designer's project explorer surfaces Alarms.
- [ ] Manual smoke alarm round-trip (define → fire → ack → clear) succeeds end-to-end.

### Out of scope

- Audible/visual annunciation (post-1.0).
- Alarm grouping by area (post-1.0).
- Bulk ack of multiple selected alarms (Phase 4).
- Alarm templates (post-1.0).

### Risks / gotchas

- **Don't accumulate alarms forever** — `AlarmTable` keeps a bounded buffer (default last 500 events, configurable). Older cleared events drop.
- **Ack-on-priority-1 needs note** — the brief leaves note optional, but a designer-level setting could enforce notes for high-priority alarms. Phase 4 polish.
- **Active alarm count badge** in the runtime header is tempting; defer to Phase 4 layout work — keep this task focused on the table component itself.

## Codex log

*(codex — append working notes here)*

### 2026-04-28 14:09  codex
Started. Taking CODEX-R because CODEX-Q is merged and this completes the alarm operator surface. Reading component-library, runtime gateway client, designer project editor, project-store alarm config shape, and gateway ack handling before editing.

### 2026-04-28 14:21  codex
Submitted. Added `AlarmTable` to the component library with bounded event buffering, priority/state filters, sorting, bad-quality indicator, ack note, and pin support. Extended runtime `GatewayClient` with `subscribeAlarms`/`ackAlarm` and wired alarm context through `ViewRenderer`. Added designer alarm authoring via `AlarmConfig`, surfaced Alarms in `ProjectExplorer`, saved the `alarms` artifact through `project.save_artifact`, and updated the designer manual smoke checklist. Verification: `pnpm -r typecheck`; `pnpm -r test`; `pnpm --filter @openwebhmi/component-library build`. `pnpm -r build` compiled website/protocol/component-library/runtime/designer Vite and the Tauri release binary, then failed only at macOS DMG bundling (`bundle_dmg.sh`).

## Claude review

### 2026-04-28  claude — review pass 1

Spec-compliant. AlarmTable, alarm config, and runtime/designer wiring all match the brief.

Strong points:
- ✅ **`who` plumbed from verified session** at `crates/gateway/src/server.rs:420-423`: gateway pulls `session.username` for the ack `who`, falling back to `"anonymous"`. The wire form `alarm.ack { alarm_id, note }` is the only thing the client can supply — the `who` field is server-controlled. Closes the CODEX-Q review note.
- ✅ **Bounded event buffer** (default 500) via `mergeEvent`/`limitEvents` in `AlarmTable.tsx:282-295`. Newest events at index 0, deduped by `alarm_id`.
- ✅ **Filter chips + sort buttons** for priority bands (all/critical/warning/info), state filter (active/all), and three sort keys (priority/state/time). Priority bands map to ≤2 / ==3 / ≥4 in `inPriorityBand`.
- ✅ **Bad-quality `?` indicator** next to the value (`AlarmTable.tsx:221-226`) — operator sees that the value the alarm fired against was untrustworthy.
- ✅ **Ack panel + optional note** sends through `onAck(alarm_id, note || null)` matching the protocol.
- ✅ **Pin support** — pinned cleared alarms survive the retention sweep at `AlarmTable.tsx:97-117`.
- ✅ **Cleared alarms fade** via `clearRetentionMs` (default 5s, configurable prop).
- ✅ **Reconnect resubscribe** for alarm subscriptions at `gatewayClient.ts:439-441` — same pattern as tag/view/project resubscribes.
- ✅ **Designer `AlarmConfig`** covers add / edit / delete with form fields per condition kind (HighLimit/LowLimit/Equals/Deviation/Digital).
- ✅ **300ms debounce** on alarm draft → save, matches the standard set by PropertyPanel.
- ✅ **Project explorer surfaces Alarms** alongside Views (`ProjectExplorer.tsx:40-46`); `App.tsx` switches between view editor grid and alarm config grid by `selectedModule`.
- ✅ **`project.save_artifact { kind: "alarms" }`** wraps the alarms list in `{ alarms: [...] }` per the project-store artifact shape.
- ✅ **`AlarmTable` registered** in `registry.ts:17`; exported from package index along with `AlarmEvent` and `AlarmSubscribeOptions` types.
- ✅ **Manual smoke step #12** added to `apps/designer/README.md:39` covering define-fire-ack-clear round-trip.
- ✅ **Tests:** 3 AlarmTable component tests (render synthetic events, ack callback wiring, priority filter) + 2 AlarmConfig tests (threshold edit + add alarm) — all green. `pnpm -r typecheck` clean. `cargo test --workspace --all-features --locked` green.

Findings:

- 🟡 **Client over-fetches alarms.** AlarmTable always sends `priority_min: 1, priority_max: 5` and applies the band filter client-side (`AlarmTable.tsx:90`). For v1 (one client per browser, ≤50 clients) this is fine; for a large fleet of operator stations all watching the "critical only" band, sending narrower min/max would let the gateway short-circuit. v1.1 polish.
- 🟡 **Draft↔prop race in `AlarmConfig`**: `useEffect([alarms])` re-syncs `drafts` from props (lines 25-29). After a 300ms-debounced save, the project-store broadcasts `project.changed`, the parent reloads, and `alarms` updates — if the user kept typing in the gap, their later keystrokes get clobbered. Same gotcha PropertyPanel had; track for the v1.1 polish PR.
- 🟡 **No client-side `id` uniqueness validation in `AlarmConfig`.** The `Id` field is freely editable; two alarms can share an id, which is a server-side constraint. The save will succeed and the second alarm will silently shadow the first. Track for v1.1 — same class as the PropertyPanel id-collision bug.
- 🟡 **AlarmTable shows nothing in designer preview mode** — by design (no live subscription off-runtime), but a designer placing the table on a view sees an empty shell. Stub events in designer mode would help authoring. Phase 4 polish.
- 🟢 **DMG bundling failed during `pnpm -r build`** at `bundle_dmg.sh`. The Tauri release binary built; only the create-dmg packaging step failed. That's an environmental tooling issue (create-dmg not installed in the build env), not a CODEX-R regression. Tracked separately for the eventual installer pipeline.
- 🟢 **Sort: state** uses a `STATE_RANK` map that includes the `clear` state (rank 3) even though the table never displays plain `clear` rows. Defensive; fine.

Acceptance criteria — all four boxes verified.

## Verdict

**Merged at `29be0e9`.** Closes the operator alarm surface: the demo HMI can now define `Pressure > 200` in the designer, observe the priority-2 alarm in `AlarmTable`, ack it with a note, and watch it clear. Phase 3's alarm slice is end-to-end complete.

Three Phase 3 polish items now tracked across the alarm UI (over-fetch on band filter, draft↔prop race in AlarmConfig, id-uniqueness validation). Together with the prior backlog items from CODEX-O/Q/S, the v1.1 hardening list is shaping up. Phase 3 progress: P + T + U remaining (Trend, scripting host, script editor) — three independent tasks, two of them unblocked.
