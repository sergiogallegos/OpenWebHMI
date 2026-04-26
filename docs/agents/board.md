# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Phase 0 — Foundations

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-A | `packages/protocol-ts` — TS protocol types | codex | in-progress | 2026-04-26 13:41 codex | [tasks/CODEX-A-protocol-ts.md](tasks/CODEX-A-protocol-ts.md) |
| CODEX-B | `crates/gateway` — WS gateway binary with sim provider | codex | in-progress | 2026-04-26 13:41 codex | [tasks/CODEX-B-gateway.md](tasks/CODEX-B-gateway.md) |
| CODEX-C | `apps/runtime-web` — React+Vite client | codex | in-progress | 2026-04-26 13:41 codex | [tasks/CODEX-C-runtime-web.md](tasks/CODEX-C-runtime-web.md) |
| CODEX-D | `.github/workflows/ci.yml` — Phase 0 CI | codex | in-progress | 2026-04-26 13:41 codex | [tasks/CODEX-D-ci.md](tasks/CODEX-D-ci.md) |

## Done

*(empty — first tasks just opened)*

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** stay in their phase section but get crossed out (`~~CODEX-A~~`) until the phase ships, then get archived to `## Done` with the merge commit ref.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
