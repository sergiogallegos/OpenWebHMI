---
id: CODEX-CA
title: Alarm subscription lifecycle — add alarm.unsubscribe, stop re-subscribe churn
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CA — Alarm subscription lifecycle (unsubscribe + stop resubscribe churn)

## Brief

> Alarm subscriptions never actually tear down, and they re-subscribe far too often. (a) There is no `alarm.unsubscribe` in the protocol — `ClientMessage` has only `alarm.subscribe` and `alarm.ack` — so the client's `unsubscribeAlarms` only mutates local maps while the gateway keeps streaming alarm events for the whole life of the connection after the operator leaves an alarm view. (b) `AlarmTable` and `AlarmBanner` re-subscribe on **every render** because their effect deps include the `context` object, which `ViewRenderer` rebuilds fresh on every render; combined with fast tag updates this spams `alarm.subscribe` at tag-update rate. Add `alarm.unsubscribe` (Rust + protocol-ts + gateway handler) and call it on teardown; stabilize the context identity so alarm components subscribe once. Also (M15) the alarm components never reconcile on reconnect — an alarm that cleared while disconnected stays rendered active. HIGH.

### Goal

Leaving an alarm view sends `alarm.unsubscribe` and the gateway stops forwarding that project's alarm events to the connection. `AlarmTable`/`AlarmBanner` subscribe exactly once for their lifetime regardless of how many tag updates re-render the tree. On reconnect, alarm components reconcile to current server state rather than displaying a stale active alarm that cleared during the outage. Rust and TypeScript protocol definitions stay in exact sync.

### Context to read first

- `crates/protocol/src/lib.rs` — `ClientMessage` (lines 238–473): `AlarmSubscribe` (lines 251–261) and `AlarmAck` (263–270). There is **no** `AlarmUnsubscribe`. The `ScriptSubscribe`/`ScriptUnsubscribe` pair (271–276 and the matching unsubscribe further down) is the naming/shape precedent to mirror.
- `packages/protocol-ts/src/index.ts` — `ClientMessage` union (lines 106–185): `alarm.subscribe` (110–114), `alarm.ack` (115); `script.subscribe`/`script.unsubscribe` (116–117) are the mirror precedent. This is where the TS `alarm.unsubscribe` variant is added.
- `crates/gateway/src/server.rs`:
  - `alarm_subscriptions: HashMap<String, AbortHandle>` (line 404) — keyed by `project_id`.
  - `AlarmSubscribe` handler (lines 576–618) — dedups by `project_id` (line 597), spawns `spawn_alarm_forwarder` and stores the `AbortHandle` (line 618).
  - `ScriptUnsubscribe` handler (lines 686–698) — the canonical teardown handler: authorize, then `if let Some(handle) = map.remove(&id) { handle.abort(); }`. `AlarmUnsubscribe` mirrors this exactly.
  - Connection-drop cleanup at line 1334 (`for (_, handle) in alarm_subscriptions { handle.abort() }`) — the existing catch-all; `alarm.unsubscribe` makes teardown explicit rather than only-on-disconnect.
- `apps/runtime-web/src/gatewayClient.ts`:
  - `subscribeAlarms` (lines 265–284) and `unsubscribeAlarms` (286–302) — the latter only mutates `alarmCallbacks`; it never sends a message to the gateway. This is where the `alarm.unsubscribe` send goes when the last callback for a key is removed.
  - `alarmKey` / `normalizedAlarmOptions` / `alarmSubscribeMessage` — the option→key→message helpers to mirror for an unsubscribe message.
  - `resubscribeAll` (lines 520–538) — resubscribes all alarm subscriptions on reconnect (line 535–537); the reconcile hook (M15) relates here.
- `packages/component-library/src/components/AlarmTable.tsx` — the subscribe effect (lines 85–95) with deps `[context, projectId, props.events, props.maxEvents]`; `context` is unstable.
- `packages/component-library/src/components/AlarmBanner.tsx` — the subscribe effect (lines 44–52) with deps `[context, projectId, props.events]`; same instability.
- `apps/runtime-web/src/ViewRenderer.tsx` — `renderNode` builds `context={{ mode: "runtime", liveValues: boundValues, ...runtime }}` fresh on **every render** (line 105). This new object identity is what invalidates the alarm effects each render.
- CLAUDE.md frontend discipline: no `any`, remove subscriptions on unmount, deterministic tests; Rust↔TS protocol sync is CI-guarded.

### Files to create / modify

1. **Add `alarm.unsubscribe` to the protocol (Rust + TS, in sync).**
   - **Rust** `crates/protocol/src/lib.rs`: add `ClientMessage::AlarmUnsubscribe { project_id: String }` with `#[serde(rename = "alarm.unsubscribe")]`, mirroring `ScriptUnsubscribe`. Extend the existing alarm round-trip serde test to cover it.
   - **protocol-ts** `packages/protocol-ts/src/index.ts`: add `| { kind: "alarm.unsubscribe"; project_id: string }` to the `ClientMessage` union (and any exhaustive switch over `ClientMessage`, e.g. near lines 350–356 where `alarm.subscribe`/`alarm.ack` are handled) so TS mirrors Rust exactly.
2. **Gateway handler** `crates/gateway/src/server.rs`: add an `AlarmUnsubscribe { project_id }` arm mirroring `ScriptUnsubscribe` (686–698): authorize with `Permission::ReadTags` (matching `AlarmSubscribe`), then `if let Some(handle) = alarm_subscriptions.remove(&project_id) { handle.abort(); }`. After this, no alarm events for that project reach the connection until it re-subscribes.
3. **Client** `apps/runtime-web/src/gatewayClient.ts`: in `unsubscribeAlarms` (286–302), when the last callback for a key is removed (`subscription.callbacks.size === 0`, line 299), send an `alarm.unsubscribe` message for that subscription's `project_id` (add an `alarmUnsubscribeMessage(options)` helper mirroring `alarmSubscribeMessage`). Note the gateway keys by `project_id` only while the client keys by full options (project + priority range) — if two client subscriptions share a `project_id`, only send `alarm.unsubscribe` when the last one for that `project_id` is gone. Handle that de-dup so unsubscribing one priority band doesn't kill another band's stream. State the chosen de-dup approach in the log.
4. **Stabilize the runtime context identity** so alarm components subscribe once:
   - `apps/runtime-web/src/ViewRenderer.tsx`: memoize the `runtime`/`context` object so it keeps a stable identity across renders that don't change its inputs. `useMemo` the `runtime` bundle in the `ViewRenderer` component keyed on its actual dependencies (`projectId`, `packId`, and the callback identities), and build `context` from it stably — so a `boundValues` change (from a tag update) does **not** mint a new `context` identity for the alarm effect's purposes. Note: `liveValues: boundValues` currently rides inside `context`; if alarm components don't need live values in the subscribe effect, keep `boundValues` out of the memoized identity that the effects depend on, or split it so the subscribe effect depends only on stable fields.
   - **And/or narrow the effect deps** in `AlarmTable.tsx` (85–95) and `AlarmBanner.tsx` (44–52): depend on the specific stable fields the effect actually uses (`context.mode`, `context.onSubscribeAlarms`, `projectId`, `props.events`, `props.maxEvents`) rather than the whole `context` object. `onSubscribeAlarms` must itself be stable (memoized in `App.tsx`/`ViewRenderer`) for this to hold. Do both if needed; the acceptance test is "subscribe fires once across many tag updates".
5. **Reconnect reconciliation (M15).** On reconnect, `AlarmTable`/`AlarmBanner` must not keep showing an alarm that cleared during the outage. First **verify whether the gateway replays a current-alarm snapshot on `alarm.subscribe`** (read `spawn_alarm_forwarder` and the alarm-engine `subscribe_events` contract):
   - If it **does** replay a snapshot, ensure the components reset/reconcile their local `events` state on resubscribe so the snapshot becomes the source of truth (don't merge stale events on top of a fresh snapshot).
   - If it **does not** replay, implement the client-side reconcile you can (e.g. clear local alarm state on reconnect and let the fresh subscription repopulate) **and** record a gateway follow-up note (a snapshot-on-subscribe is the proper fix and belongs to the gateway). Don't fake a reconcile that can't actually reflect server truth.

### Behavior

- Leaving an alarm view (last alarm callback removed) sends `alarm.unsubscribe`; the gateway aborts that project's forwarder and stops streaming its alarm events.
- `AlarmTable`/`AlarmBanner` call `onSubscribeAlarms` exactly once for their mounted lifetime, regardless of tag-update-driven re-renders.
- Unsubscribing one priority band for a project does not tear down another band still subscribed for the same project.
- After a reconnect, the alarm components reflect current server alarm state (a cleared-during-outage alarm is no longer shown active), either via replayed snapshot or client-side reconcile.
- Rust and TS `ClientMessage` definitions stay in exact sync.

### Test requirements

- **Client sends `alarm.unsubscribe` on teardown** (extend the existing `gatewayClient` test file, using the `WebSocketLike` fake): subscribe alarms, then remove the last callback and assert an `alarm.unsubscribe` with the right `project_id` was sent. Assert that removing one of two callbacks for the same `project_id` does **not** send it, but removing the last does. Must fail against pre-fix `unsubscribeAlarms` (which sends nothing).
- **Subscribe fires once across many tag updates** (extend the component-library test suite for `AlarmTable`/`AlarmBanner`, Vitest + RTL): mount the component in runtime mode with a spy `onSubscribeAlarms`, push many bound-value/tag updates that re-render the tree, and assert `onSubscribeAlarms` was called exactly once (and its cleanup once on unmount). Must fail against pre-fix code (unstable `context` dep re-subscribes each render). Deterministic — drive re-renders explicitly, no timers.
- **Reconnect reconcile**: assert that on resubscribe the component's active-alarm set reflects the fresh subscription rather than retaining a stale cleared alarm. Shape the test to whichever reconcile path the gateway supports (snapshot vs client clear); state which in the log.
- **Rust serde**: extend the alarm round-trip test to cover `AlarmUnsubscribe` wire form (`{"kind":"alarm.unsubscribe","project_id":"..."}`).
- **Gateway handler**: a test (bind `127.0.0.1:0`) that subscribes then unsubscribes and asserts no further alarm events arrive for that project. Deterministic sync, no `sleep`, no hardcoded ports.
- Full matrix clean: `cargo test`/`clippy -D warnings`/`fmt --check`, `pnpm -r typecheck`, `pnpm -r test`; component-library `build`.

### Acceptance criteria

- [ ] `ClientMessage::AlarmUnsubscribe { project_id }` added in `crates/protocol` and mirrored in `packages/protocol-ts` with no drift; both exhaustive switches/matches updated; Rust serde test covers it.
- [ ] Gateway `AlarmUnsubscribe` handler mirrors `ScriptUnsubscribe`: authorize + `remove(&project_id).abort()`; no alarm events after unsubscribe.
- [ ] Client `unsubscribeAlarms` sends `alarm.unsubscribe` when the last callback for a `project_id` is removed, with correct de-dup across priority bands (approach recorded in the log).
- [ ] Runtime `context`/`onSubscribeAlarms` identity stabilized (memoized) and/or `AlarmTable`/`AlarmBanner` effect deps narrowed so subscribe fires exactly once across many tag updates.
- [ ] Reconnect reconciliation implemented against verified gateway behavior (snapshot reset or client clear), with a gateway follow-up note if snapshot-on-subscribe is missing.
- [ ] All listed tests pass; the "sends unsubscribe" and "subscribe once" tests demonstrably fail against pre-fix code; all deterministic (no timers, `127.0.0.1:0` for gateway tests).
- [ ] Full matrix clean including component-library `build`.

### Out of scope

- **Adding a current-alarm-snapshot message to `alarm.subscribe` on the gateway** if it doesn't already replay one — that is a gateway feature and gets its own follow-up brief (noted here, not built here). This task does the client-side reconcile it can and records the gap.
- **Redesigning the alarm priority-band subscription model** (client keys by options, gateway by project) — this task de-dups within the existing model, it doesn't unify the keying.
- **`ServerMessage::Error` correlation** and socket-teardown reliability — that is CODEX-BZ.
- **Alarm-engine internals / ack semantics** — untouched; this task is subscription lifecycle only.
- **Broadly memoizing all of `ViewRenderer`** for performance — stabilize the context/callback identities enough to fix the alarm churn; a full render-perf pass is separate.

### Risks / gotchas

- **Client-vs-gateway subscription keying mismatch.** The client keys `alarmCallbacks` by full options (project + priority range) via `alarmKey`; the gateway keys `alarm_subscriptions` by `project_id` alone. Two client subscriptions to the same project with different priority bands map to **one** gateway forwarder. Sending `alarm.unsubscribe` when only one band is gone would silently kill the other band's stream. De-dup on the client: only send `alarm.unsubscribe` when no remaining `alarmCallbacks` entry shares that `project_id`.
- **Memoizing `context` can break other consumers.** `context` also carries `liveValues: boundValues`, which non-alarm components legitimately read live. Don't freeze `boundValues` out of the context that live components see — stabilize only the identity the *subscribe effects* depend on (narrow the deps, or split a stable subscribe-context from the live-values context). Verify no component that needs live values regresses.
- **`onSubscribeAlarms` must be stable for narrowed deps to help.** If the callback passed from `App.tsx` is a fresh closure each render, narrowing the effect to depend on it still re-fires. Memoize the callback at its source (`useCallback` in `App.tsx`) — check the source before assuming narrowing alone fixes it.
- **Cleanup must run on unmount, not just on dep change.** The subscribe effect returns the unsubscribe function; ensure stabilizing deps doesn't accidentally drop the cleanup (a subscription that's created once must still be torn down once on unmount → which now also sends `alarm.unsubscribe`).
- **Reconcile honesty.** If the gateway doesn't replay a snapshot, a client-side "clear on reconnect" only reflects truth once fresh events arrive — a currently-active alarm won't reappear until it next transitions. Don't claim full reconciliation; state the limitation and file the gateway follow-up. Faking it hides a real SCADA correctness gap.
- **Rust↔TS drift is a CI failure.** Add the variant to both the Rust enum and the TS union (and both exhaustive switches) in the same commit; run `pnpm -r typecheck` and the Rust serde test together.
- **Honesty (CLAUDE.md).** State the de-dup approach, whether the gateway was confirmed to replay a snapshot (read the code, don't guess), whether the "subscribe once" and "sends unsubscribe" tests were run against pre-fix code to confirm they fail, and which reconcile path was taken.

## Codex log

## Claude review

## Verdict
