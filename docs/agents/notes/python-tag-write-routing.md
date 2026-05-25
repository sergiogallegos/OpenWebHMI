# Python `system.tag.write` routing

## The plumbing (CODEX-V outcome)

When Python scripts call `system.tag.write(path, value)` from a scripting subprocess, the write does **not** call `TagStore::publish` directly. It routes through `GatewayTagWriteSink`, which:

1. Inspects the tag path prefix.
2. **Driver-prefixed paths** (e.g. `rockwell/MyTag`, `modbus/Coils[5]`) — dispatch to the corresponding driver's write mpsc queue. The driver owns the actual wire write; backpressure and ordering are per-driver.
3. **Memory-tag paths** (no driver prefix) — call `TagStore::publish` directly. Memory tags are in-process state and don't need driver-side write coordination.

## Why this matters

The v1.0 brief for the scripting host (CODEX-T) said "`system.tag.write` calls `TagStore::publish` directly." That was wrong — it would have written driver-backed tags to memory and let the driver's next read overwrite them, silently dropping operator-initiated commands. CODEX-V was the closeout follow-up that fixed it by introducing the per-driver write queue.

This is a Claude-authored brief error owned in the CODEX-V verdict.

## What this means for new tasks

When briefing or reviewing anything that touches Python-script writes:

- **Don't say "publish directly"** in a brief. Say "route through `GatewayTagWriteSink`."
- **Don't propose bypassing the write queue** for performance — the queue exists because direct publish is wrong for driver-backed tags.
- **New drivers** must provide a write mpsc that `GatewayTagWriteSink` can dispatch to. The driver API spec in `crates/driver-api` formalizes this contract.

## Reviewing test coverage

A scripting-host test that writes a driver-prefixed tag and asserts the tag updated **without** asserting the driver write queue received the message is incomplete — it could pass against the old broken plumbing. Tests must observe the driver-side write to be valid.

## See also

- `crates/gateway/src/scripting.rs` — `GatewayTagWriteSink` implementation.
- `crates/driver-api` — driver write-mpsc contract.
- CODEX-V verdict in `docs/agents/tasks/CODEX-V-system-tag-write.md`.
