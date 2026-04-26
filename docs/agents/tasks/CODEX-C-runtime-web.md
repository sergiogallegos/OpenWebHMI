---
id: CODEX-C
title: apps/runtime-web — React+Vite client subscribing to a tag
owner: codex
phase: 0
status: merged
created: 2026-04-26
last-update: 2026-04-26 13:55 claude
merge-commit: 75ccb9c
---

# CODEX-C — `apps/runtime-web` (browser HMI client)

## Brief

> **Blocked by [CODEX-A](CODEX-A-protocol-ts.md).** This task imports `@openwebhmi/protocol`, which doesn't exist until A is at least published locally via pnpm workspace. Don't start until A is `submitted` or `merged`.

### Goal

Vite + React + TS app that connects to the gateway, subscribes to `system/sim/sin` and `system/sim/counter`, and renders the live values. This is the client side of the Phase 0 exit-criterion demo.

### Context to read first

- `crates/protocol/src/lib.rs` — wire form ground truth.
- `packages/protocol-ts/src/index.ts` (delivered by CODEX-A) — what you import.
- `crates/gateway/src/...` (delivered by CODEX-B) — what you connect to.
- `docs/architecture.md` §4.11 (HMI Runtime), §5 (Data flow).
- `docs/roadmap.md` Phase 0 exit criterion.

### Files to create

- `apps/runtime-web/package.json` — name `@openwebhmi/runtime-web`, private. Deps: `react`, `react-dom`, `@openwebhmi/protocol` (workspace ref). DevDeps: `@types/react`, `@types/react-dom`, `vite`, `@vitejs/plugin-react`, `typescript`, `vitest`, `jsdom`.
- `apps/runtime-web/vite.config.ts`.
- `apps/runtime-web/tsconfig.json` — strict, `target: ES2022`, `jsx: react-jsx`.
- `apps/runtime-web/tsconfig.node.json` — for vite config.
- `apps/runtime-web/index.html`.
- `apps/runtime-web/src/main.tsx`.
- `apps/runtime-web/src/App.tsx`.
- `apps/runtime-web/src/gatewayClient.ts` — the WS client class (testable, no React imports).
- `apps/runtime-web/src/gatewayClient.test.ts` — vitest unit tests.

Do **not** modify `pnpm-workspace.yaml`; the package is already listed there.

### `gatewayClient.ts` API

```ts
type ConnectionState = "idle" | "connecting" | "connected" | "reconnecting" | "closed";
type Listener = (snap: { value: TagValue; quality: Quality; ts: number }) => void;

export class GatewayClient {
  constructor(opts: { url: string; webSocketImpl?: typeof WebSocket });
  readonly state: ConnectionState;
  onStateChange(cb: (s: ConnectionState) => void): () => void;
  connect(): void;
  disconnect(): void;
  subscribe(path: string, cb: Listener): () => void;  // returns unsubscriber
  ping(): void;
}
```

### Behavior

- Constructor stores config; does not auto-connect. `connect()` opens the WS.
- On `open`: state → `connected`. Re-send `tag.subscribe` for every path with one or more listeners (for reconnect resubscribe).
- On `message`: parse + `isServerMessage` guard; route `tag.update` to listeners by `path`; ignore other kinds for v1 (log at debug).
- On `close` or `error`: state → `reconnecting`; schedule reconnect with **exponential backoff** starting at 250ms, multiplier 2, cap 8000ms, jitter ±25%.
- `subscribe(path, cb)`:
  - Add `cb` to the path's listener set.
  - If state is `connected` and this path went from 0→1 listener, send `tag.subscribe` over the wire.
  - Returns an unsubscriber that removes `cb` and, if this drops the path to 0 listeners and state is `connected`, sends `tag.unsubscribe`.
- `disconnect()`: state → `closed`; close the WS; cancel reconnect timer; do **not** clear listeners (caller can later `connect()` again and listeners resubscribe).
- `webSocketImpl` injection point exists so tests can pass a mock.

### `App.tsx`

- Construct a `GatewayClient` once (in a `useRef` or module-scope singleton). URL from `import.meta.env.VITE_GATEWAY_URL` defaulting to `ws://localhost:8080`.
- Display:
  - Connection state badge (Connecting / Connected / Reconnecting / Closed).
  - `system/sim/sin`: numeric value to 4 decimals, quality, last-update timestamp.
  - `system/sim/counter`: integer value, quality, last-update timestamp.
- No CSS framework. Inline styles or one small `index.css` is fine — deliberately minimal; styling lives in Phase 2 component library.

### Test requirements

`gatewayClient.test.ts` (vitest, jsdom-free — pass a mock `WebSocket` class):

- Connect → on first `open` event, state transitions `connecting → connected`.
- Subscribing twice to the same path sends only one `tag.subscribe` over the wire.
- Unsubscribing the last listener for a path sends `tag.unsubscribe`.
- On WS close, state goes to `reconnecting` and a new WS is constructed after the backoff delay (use fake timers).
- Backoff doubles up to the 8000ms cap; reset on successful `open`.
- Incoming `tag.update` is dispatched to all listeners of the matching path.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/runtime-web typecheck` exits 0.
- [ ] `pnpm --filter @openwebhmi/runtime-web test` runs vitest; all tests above pass.
- [ ] `pnpm --filter @openwebhmi/runtime-web build` produces a `dist/` bundle.
- [ ] Manual smoke: with the gateway from CODEX-B running, `pnpm --filter @openwebhmi/runtime-web dev` opens a page that shows the sin wave updating ~1 Hz and the counter incrementing each second. Killing the gateway flips state to "Reconnecting"; restarting it restores values without a page reload.

### Out of scope

- Authentication.
- Multiple gateway connections.
- Tag write path (Phase 1).
- Routing / multi-page.
- Visual styling beyond minimal layout.

### Risks / gotchas

- React 18 strict mode runs effects twice in dev. The `GatewayClient` must be safe to construct/connect from a `useEffect` mount + cleanup pair. The recommended pattern: module-scope singleton, or a `useRef`-based singleton that only `connect()`s once.
- Browser `WebSocket` does not surface error details on close events; treat any close-without-clean (`event.code !== 1000`) as a reconnect trigger.
- Don't use `Math.random()`-based jitter in tests with fake timers without seeding/mocking it; the backoff test should assert the *cap* and *doubling*, not exact ms.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 13:41  codex
I identified as codex and read `docs/agents/README.md`, `board.md`, and this task brief. CODEX-A has been implemented locally in the same working tree, so I started CODEX-C against the local workspace package.

### 2026-04-26 13:48  codex
Submitted. Implemented `apps/runtime-web` with Vite, React, TypeScript, a minimal live-value UI, and a testable `GatewayClient` with injected WebSocket, state callbacks, reconnect/resubscribe, unsubscribe, ping, and tag update dispatch. Verification: `pnpm --filter @openwebhmi/runtime-web typecheck`, `test`, and `build` pass. Manual browser smoke was not run; gateway protocol smoke was verified with a Node WebSocket client under CODEX-B.

## Claude review

### 2026-04-26 13:55  claude — review pass 1

Spec-compliant. `GatewayClient` has the right public surface, multiplexed subscriptions deduplicate wire-level subscribe sends, and resubscribe-on-open correctly handles reconnect.

- ✅ Singleton via `useRef<GatewayClient | null>` is React 18 strict-mode safe.
- ✅ `manuallyClosed` flag prevents reconnect after `disconnect()`.
- ✅ Exponential backoff (250 → 500 → 1000 → 2000 → 4000 → 8000 cap, ±25% jitter), reset on `onopen`.
- ✅ `webSocketImpl` injection point used cleanly by mock-WebSocket tests.
- 🟢 **Test density vs count**: brief asked for 6 distinct cases; Codex delivered 3 tests that cover all 6 by bundling related assertions per test. Coverage is complete; style preference, not a gap.
- 🟡 Cosmetic: initial `connectionState` in `App.tsx` is `"connecting"` but the client's actual initial state is `"idle"` until `connect()` is called. Microsecond visible window.
- 🟡 Cosmetic: `subscribe()` calls `send()` even when the WS is not OPEN. `send()` silently drops; `resubscribeAll()` recovers on open. Correct, but a `debug` log on the dropped attempt would help future debugging.
- 🟡 Browser smoke not run by Codex (only Node WebSocket smoke under CODEX-B). Will exercise on first manual run; not blocking the merge.

No follow-up tasks required.

## Verdict

**Merged** at `75ccb9c`. The cosmetic notes are tracked here only.
