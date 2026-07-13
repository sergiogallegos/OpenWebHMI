---
id: CODEX-BF
title: Server-authoritative tag-write authorization — replace client-steerable view ACL
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BF — Server-authoritative tag-write authorization (Tier-1 SECURITY)

## Brief

> **v1.0 blocker.** Tag-write authorization is steered by the client. The gateway sets one per-connection `current_view_allowed_roles` from whatever view the client last opened; `ViewClose` is a no-op; and `authorize_write` skips the ACL check entirely when that field is `None` — so a client that never opens a view (or opens a permissive one) can write any tag. Writes are also not restricted to the tags actually bound in the opened view. Derive write authorization **server-side** from the project's view/tag relationship, deny by default when a tag's write isn't authorized for the session's roles, and correct the `docs/architecture.md` claims that overstate the current guarantee. **Scope-locked to write authorization.**

### Goal

A tag write is authorized against a server-computed policy — which views expose which writable tags, and each of those views' `allowed_roles` — evaluated against the session's roles, not against client-declared current-view state. A session that never opened a view cannot write. A session that opened view A cannot write a tag that only view B exposes. Deny-by-default: a tag with no authorizing view/role for the session is rejected. `docs/architecture.md:298` and `:364` are updated to state the actual guarantee, and the open question at `:386` is resolved (or narrowed) to reflect the chosen model.

### Context to read first

- `crates/gateway/src/server.rs:1088-1124` — `ClientMessage::ViewOpen`. Line 1108 sets `current_view_allowed_roles = view.allowed_roles.clone()` from whatever view the client asked to open. This single per-connection field is the entire "ACL" and it is fully client-steerable.
- `crates/gateway/src/server.rs:1125` — `ClientMessage::ViewClose { .. } => {}` — a no-op. Opening a view latches its roles for the rest of the connection; closing never clears them. A client opens a permissive view once and keeps the grant forever.
- `crates/gateway/src/server.rs:1817-1846` — `authorize_write`. After the base `Permission::WriteTags` check, if `allowed_roles` is `None` it returns `true` (1829-1831) — i.e. **no view opened ⇒ writes allowed**. When `Some`, it checks role membership against the latched view roles, but nothing ties the write to the *tag* being in that view. A session can write any tag path once any (or no) view is open.
- The tag-write handler that calls `authorize_write` (search for `authorize_write(` call sites and the `TagWrite`/`system.tag.write` routing) — to see what context (tag path, session, project) is available at the write point. The write must be checked against the *tag*, not just a latched role set.
- `crates/project-store/src/store.rs` — the `View` shape: `view.allowed_roles`, and the components/bindings that reference writable tags. This is the source of the server-side policy: which views bind which writable tags, and each view's `allowed_roles`.
- [`docs/agents/notes/binding-write-asymmetry.md`](../notes/binding-write-asymmetry.md) — component bindings expose read paths but not write paths; the `tagPath?: string` prop is the write-target workaround. Relevant to computing "which tags are writable from a view."
- [`docs/agents/notes/python-tag-write-routing.md`](../notes/python-tag-write-routing.md) — `system.tag.write` routes through `GatewayTagWriteSink`; confirm whether script-originated writes go through the same `authorize_write` path or a different one, so the policy covers (or intentionally exempts) script writes. Scripts are semi-trusted per architecture §7 — decide and document.
- `docs/architecture.md:298` — "gateway authorizes against role + view ACL"; `:364` — "every write checked against role + view ACL"; `:386` — open question "Tag-write authorization: per-tag ACLs vs view-only ACLs." The code does not deliver what 298/364 claim; 386 is the decision this brief makes.

### Files to create / modify

1. **Modify** `crates/gateway/src/server.rs`:
   - Replace the single latched `current_view_allowed_roles` with a server-authoritative check. When a write for tag `T` by session `S` arrives, authorize it iff there exists a view `V` in the project such that `V` exposes `T` as a writable tag **and** `S.roles ∩ V.allowed_roles ≠ ∅`. Deny by default (no such view ⇒ reject).
   - Remove the "`None` ⇒ allow" escape at 1829-1831. Absence of an opened view is not a grant.
   - Decide the membership model (state it in the Codex log and in the architecture update):
     - **Option A (view-membership ACL, recommended for v1.0):** a write is allowed if the tag is writable from *any* view whose `allowed_roles` admit the session. Compute the tag→(authorizing roles) map server-side from the project on load/change; no dependence on which view the client currently claims to have open.
     - **Option B (per-tag ACL):** tags carry their own `allowed_roles` independent of views. More precise but a bigger schema change — likely a v1.1 follow-up; if chosen, keep it minimal and additive.
   - Cache the derived policy per project and refresh it on `ProjectChange` (the store already broadcasts changes — see `subscribe_changes`). Don't recompute from disk on every write.
   - `ViewOpen`/`ViewClose` may still track UI subscription state, but they must **not** be the source of write authorization.
2. **Modify** `docs/architecture.md`:
   - Update `:298` and `:364` to state the real guarantee (writes authorized server-side against the project's view/tag→role policy; deny by default).
   - Resolve or narrow the `:386` open question to record the chosen model (A or B) and note the other as a possible v1.1 refinement.
3. **Do NOT**:
   - Change the wire protocol's `ViewOpen`/`ViewClose`/`TagWrite` message shapes unless strictly required; the fix is server-side policy, not a new client contract.
   - Redesign the role system or `Permission` enum.

### Behavior

- Session with `WriteTags` permission but roles disjoint from every view exposing tag `T` → write to `T` denied (`auth.forbidden`).
- Session that never opened any view → cannot write (deny-by-default), reversing the current "None ⇒ allow."
- Session opened view A (roles admit A) but writes a tag only view B exposes → denied.
- Session whose roles admit a view exposing `T` → write to `T` allowed, regardless of which view the client currently claims open.
- Editing the project (changing a view's `allowed_roles` or bindings) updates the policy without a gateway restart (via the change broadcast).

### Test requirements

- Add tests to the gateway crate's existing server/auth test module. Bind any test listener to `127.0.0.1:0`.
  - **No-view-write denied:** a session that never opened a view attempts a write → denied. This **must fail against the pre-fix code** (which allowed it via the `None ⇒ true` path).
  - **Cross-view write denied:** session authorized for view A writes a tag only exposed by view B → denied.
  - **Role-mismatch denied:** session whose roles don't intersect the authorizing view's `allowed_roles` → denied.
  - **Authorized write allowed:** session whose roles admit a view exposing the tag → allowed.
  - **Policy refresh:** after a `ProjectChange` that grants a role, a previously-denied write becomes allowed without restart (drive the change deterministically via the store's change channel — no `sleep`).
- If script-originated writes share the path, add a case pinning the decided behavior (covered vs intentionally exempt).
- Full Rust matrix clean.

### Acceptance criteria

- [ ] Write authorization is computed server-side from the project's view/tag→role relationship, not from client-declared current-view state.
- [ ] The "no view opened ⇒ allow" path (server.rs 1829-1831) is removed; deny-by-default holds.
- [ ] Writes are constrained to tags actually exposed as writable by an authorizing view (not any arbitrary tag path).
- [ ] Policy is cached per project and refreshed on `ProjectChange` without restart.
- [ ] `docs/architecture.md:298` and `:364` updated to the real guarantee; `:386` open question resolved/narrowed to the chosen model.
- [ ] Regression tests (no-view, cross-view, role-mismatch, authorized, policy-refresh) present; the no-view and cross-view tests fail against pre-fix code.
- [ ] Script-write behavior decided and pinned by a test (covered or documented exempt).
- [ ] No `panic!`/`expect`/`unwrap` on the write path; full Rust matrix clean; no new `#[allow]` (use `#[expect]` + reason if needed).

### Out of scope

- **Per-tag ACL schema (Option B) as the mandated model** — view-membership (Option A) is the v1.0 target; per-tag can be a v1.1 refinement unless Option B proves simpler in practice.
- **Role system / `Permission` enum redesign.**
- **Client protocol changes** to `ViewOpen`/`ViewClose`/`TagWrite` shapes (unless strictly required and then minimal).
- **Auditing every existing architecture.md security claim** — update only the three cited lines (298/364/386) that this change makes true/false.
- **Rate-limiting or connection caps** — CODEX-BG.

### Risks / gotchas

- **"Which tags are writable from a view" is the crux.** Per `binding-write-asymmetry.md`, bindings expose read paths; write targets come via the `tagPath?: string` prop pattern. The policy must enumerate the *write* targets a view exposes, which may not equal its read bindings. Confirm how a view declares writable tags before computing the map; if the data model doesn't cleanly express it, surface that as an open question rather than guessing.
- **Deny-by-default is a behavior change that could break the demo HMI.** Removing the `None ⇒ allow` path means any flow currently relying on writing without an authorizing view will start failing. That is the *correct* security posture, but per CLAUDE.md "don't undersell load-bearing items" — verify the demo HMI's headline write flows still work under the new policy, and if they don't, that's a policy/data-model gap to fix (grant the demo's operator role on the relevant view), not a reason to keep the escape hatch.
- **Script writes.** `system.tag.write` routes through `GatewayTagWriteSink`. Determine whether that path hits `authorize_write` at all. Scripts are semi-trusted (project-owner authored) per architecture §7, so exempting them may be correct — but decide explicitly and pin it with a test; don't leave it accidentally unauthorized-and-allowed or accidentally-denied.
- **Recompute cost.** Don't load the project from disk on every write. Cache the derived tag→roles map and invalidate on the store's change broadcast (`subscribe_changes`). Confirm the broadcast fires on the edits that matter (view `allowed_roles` change, binding change).
- **`ViewClose` no-op.** Even after this change, if `ViewOpen`/`ViewClose` still track UI subscription state, keep that separate from authorization. The bug is conflating "what the client says it's viewing" with "what the server allows it to write."
- **Ask "why this and not the alternative?"** If Option A (view-membership) can't express a needed policy (e.g. a tag writable from no view but still legitimately writable by an admin script), note it — that's the signal the model needs Option B or an explicit exemption, and it's better surfaced in review than worked around silently.

## Codex log

## Claude review

## Verdict
