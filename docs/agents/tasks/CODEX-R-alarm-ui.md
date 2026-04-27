---
id: CODEX-R
title: AlarmTable component + alarm config UI
owner: codex
phase: 3
status: open
created: 2026-04-27
last-update: 2026-04-27 claude
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

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
