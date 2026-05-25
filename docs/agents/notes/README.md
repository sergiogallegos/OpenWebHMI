# Agent notes

Surface-specific durable lore. Each file is a one-page reference for a single project surface, intentionally narrow so it can be loaded on demand.

Notes live here instead of inline in `CLAUDE.md` so:

- `CLAUDE.md` stays a short operating manual, not a kitchen sink.
- Lore is discoverable by topic — agents grep `docs/agents/notes/` when touching a surface.
- Each note has its own commit history; updates show on the surface, not buried in a CLAUDE.md edit.
- Briefs can link directly to the relevant note instead of restating the gotcha.

## When to add a note

Add a note when:

- A non-obvious constraint will trip the next agent on the same surface (firmware quirks, environment requirements, plumbing that doesn't match the obvious shape).
- A "why this and not the alternative" answer is load-bearing for future changes.
- The fact would otherwise belong inline in `CLAUDE.md` as a gotcha bullet.

Don't add a note for:

- Code conventions that read naturally from the code itself.
- One-off bugs already fixed and covered by tests.
- Ephemeral decisions about a specific task — those belong in the task file.

## When to retire a note

Notes can rot. Retire one when:

- The underlying surface is removed (e.g. driver replaced, dependency dropped).
- The constraint is gone (e.g. upstream library fixed the bug the note worked around).
- The fact is now obvious from current code (e.g. an `#[expect]` comment with the same rationale lands at the relevant site).

Retirement is a delete + a one-liner in `log.md` explaining why the constraint no longer applies.

## Current notes

- [`binding-write-asymmetry.md`](binding-write-asymmetry.md) — component bindings expose read paths but not write paths; the `tagPath?: string` prop pattern is the v1.0 workaround.
- [`mqtt-tls-in-ci.md`](mqtt-tls-in-ci.md) — `rumqttd 0.20.0` is plaintext-only; real TLS testing uses an external Mosquitto-with-CA fixture.
- [`python-tag-write-routing.md`](python-tag-write-routing.md) — `system.tag.write` routing through `GatewayTagWriteSink` (CODEX-V plumbing); don't say "publish directly" in briefs.
- [`toolchain-drift.md`](toolchain-drift.md) — Rust pinned via `rust-toolchain.toml`, Node pinned in CI; include both when reporting environment-specific verification failures.
