---
id: CODEX-BD
title: Refuse to boot without an explicit JWT signing secret — remove the hardcoded fallback
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BD — Refuse to boot without an explicit JWT signing secret (Tier-1 SECURITY)

## Brief

> **v1.0 blocker.** When no JWT secret is supplied, the gateway falls back to a source-visible literal (`"openwebhmi-dev-secret-rotate-immediately"`), so anyone who can read the public repository can forge an Administrator HS256 token and issue tag writes to live PLCs. Remove the hardcoded fallback: the gateway must **refuse to start** without an explicit secret, unless an explicit `--dev-insecure` opt-in flag is passed (which logs a loud warning and generates a random per-boot ephemeral secret). This is startup validation — a hard, clear error-and-exit is the correct shape here, not a panic on a reachable path.

### Goal

Booting the gateway with neither `--jwt-secret` nor `OPENWEBHMI_JWT_SECRET` set, and without `--dev-insecure`, exits with a clear error naming the flag/env var to set — no server ever binds. Booting with `--dev-insecure` starts with a randomly generated ephemeral secret (unique per boot, never the source literal) and logs a prominent warning that tokens are dev-only and won't survive a restart. Booting with an explicit secret behaves exactly as today. The literal `"openwebhmi-dev-secret-rotate-immediately"` no longer exists anywhere in the source tree.

### Context to read first

- `crates/gateway/src/main.rs:559-562` — `init_auth`: `let secret = args.jwt_secret.clone().unwrap_or_else(|| { warn!(...); "openwebhmi-dev-secret-rotate-immediately".to_string() });`. This is the exact fallback to delete.
- `crates/auth/src/sessions.rs:75-85` — `SessionManager::verify` decodes HS256 with `DecodingKey::from_secret(&self.secret)`. Any party knowing the secret bytes can mint a token that passes this check; the sign side is at 67-71. The secret is the only thing standing between the repo reader and an Administrator session.
- `crates/gateway/src/main.rs` `Args` struct — where `jwt_secret` is declared (`--jwt-secret` / `OPENWEBHMI_JWT_SECRET`). Add the `--dev-insecure` flag alongside it, matching the existing clap derive style (`tls_cert` / `tls_key` are nearby examples of the option shape).
- `crates/gateway/src/main.rs:547-567` — the full `init_auth` fn. Note `init_auth` returns `anyhow::Result<AuthContext>`; propagating an `Err` here already fails startup cleanly. Prefer that over `std::process::exit` so the error message formats through the existing `main` error path.
- `VISION.md:45` — "**No bundled accounts or default credentials.** Auth is local-first, configured by the operator." The hardcoded fallback is a bundled default credential in all but name; this brief brings the gateway in line with the stated vision.
- CLAUDE.md "Rust code quality" — `fn main` / startup config validation is explicitly exempt from the no-panic rule. This IS startup validation: a returned `Err` (preferred) or a hard exit with a clear message is correct. Do **not** reach for `panic!`/`expect` inside a request path — nothing here is on a request path.

### Files to create / modify

1. **Modify** `crates/gateway/src/main.rs`:
   - Add a `--dev-insecure` boolean flag to `Args` (clap derive; default `false`; a doc comment explaining it generates an ephemeral per-boot secret and is never for production). Give it an env fallback only if it matches the existing option conventions — otherwise flag-only is safer (an env var that silently enables insecure mode is a footgun).
   - Rewrite the secret resolution in `init_auth` (559-562):
     - If `args.jwt_secret` is `Some`, use it (unchanged).
     - Else if `args.dev_insecure`, generate a cryptographically random secret (e.g. 32 bytes from a CSPRNG — check whether a suitable RNG is already a workspace dependency before adding one; `rand` or `getrandom` transitive availability is likely) and `warn!` loudly that this is a dev-only ephemeral secret that invalidates all tokens on restart.
     - Else return `anyhow::bail!(...)` with a message naming `--jwt-secret` / `OPENWEBHMI_JWT_SECRET` (and mentioning `--dev-insecure` for local development). No server binds.
   - Delete the literal `"openwebhmi-dev-secret-rotate-immediately"`.
2. **Do NOT**:
   - Change `SessionManager` / `sessions.rs` — the signing/verifying logic is fine; only the secret *provisioning* is wrong.
   - Add a config-file secret source, a secrets-manager integration, or key rotation — those are post-1.0. Env var + flag is the v1.0 surface.
   - Touch the first-run admin-password bootstrap (550-557) — that is a separate, already-correct generate-and-warn flow.

### Behavior

- No secret, no `--dev-insecure`: process exits non-zero before binding, with an error naming the env var and flag. The error text is actionable ("set OPENWEBHMI_JWT_SECRET or pass --jwt-secret; for local dev use --dev-insecure").
- `--dev-insecure`, no secret: starts with a random 32-byte secret; a `warn!` states tokens are ephemeral and dev-only. Two consecutive boots produce different secrets (a token from boot 1 fails verification on boot 2).
- Explicit secret via flag or env: unchanged from today.
- The string `openwebhmi-dev-secret-rotate-immediately` returns zero hits in a repo-wide grep after the change.

### Test requirements

- Add a unit/integration test for the secret-resolution logic. Factor the resolution into a small testable fn (e.g. `resolve_jwt_secret(jwt_secret: Option<&str>, dev_insecure: bool) -> anyhow::Result<Vec<u8>>`) so the three branches are testable without booting a full server:
  - `None, false` → `Err`.
  - `None, true` → `Ok`, and two calls yield **different** secrets (proves ephemeral randomness, not a constant).
  - `Some("x"), _` → `Ok` with exactly those bytes.
- A guard test (or a grep-based CI note in the Codex log) confirming the literal is gone. A cheap `include_str!("main.rs").contains(...)` style test is acceptable, or simply verify via the resolution test that the `None,false` path errors rather than returning a constant.
- **The `None, false → Err` test must fail against the pre-fix code** (which returned the literal). Confirm it does before claiming it guards the regression.
- No `sleep`/wall-clock. Full Rust matrix clean (build + clippy + test + doc).

### Acceptance criteria

- [ ] The literal `"openwebhmi-dev-secret-rotate-immediately"` is deleted; repo-wide grep returns zero hits.
- [ ] Boot with no secret and no `--dev-insecure` exits non-zero before binding, with a message naming `--jwt-secret` / `OPENWEBHMI_JWT_SECRET` and mentioning `--dev-insecure`.
- [ ] `--dev-insecure` generates a random per-boot secret (different across boots) and logs a loud dev-only warning.
- [ ] Explicit secret via flag/env is unchanged.
- [ ] Secret resolution is factored into a testable fn with three branch tests; the `None,false → Err` test fails without the fix.
- [ ] No `panic!`/`expect`/`unwrap` added on any request path (startup `bail!`/`Err` propagation only).
- [ ] Full Rust matrix clean; no new `#[allow]` (use `#[expect]` + reason if needed).

### Out of scope

- **Config-file or secrets-manager secret sources**, key rotation, multiple active keys, or JWK sets — all post-1.0.
- **Changing the JWT algorithm** (HS256 → asymmetric). A real improvement, but a separate brief; this one only fixes provisioning.
- **The first-run admin-password bootstrap** (main.rs 550-557) — separate, already-correct flow.
- **Coordinating the secret store with the audit HMAC key** — CODEX-BH will provision its HMAC key in the same *shape* as this flag/env pair; establishing that shape cleanly here is the contribution, but wiring BH's key is BH's job.

### Risks / gotchas

- **`Err` over `exit`.** `init_auth` already returns `anyhow::Result`; a `bail!` propagates to `main`'s error handler and prints cleanly. `std::process::exit` bypasses destructors and the existing error formatting — prefer the `Result` path. Both are acceptable "startup validation" per CLAUDE.md, but the `Result` path is the neighbor pattern here.
- **CSPRNG, not `rand::random` from a seeded/weak source.** The ephemeral secret must be from an OS CSPRNG (`getrandom` / `OsRng`). Check the existing lockfile before adding a dependency — `rand` or `getrandom` is very likely already transitively present (rustls, jsonwebtoken, uuid pull it). Per CLAUDE.md, don't `cargo update`; if a new direct dep is truly needed, add just that one with a pinned version and note it in the Codex log.
- **Don't let an env var silently enable insecure mode.** `--dev-insecure` as a flag is a deliberate, visible act. If an `OPENWEBHMI_DEV_INSECURE=1` env fallback is added, weigh that a compromised environment could flip it — flag-only is the safer default; document the choice either way.
- **The warning must be loud and honest.** State that tokens are ephemeral (invalidated on restart) and MUST NOT be used in production. Per CLAUDE.md honesty discipline, don't undersell it as "dev note" — it's a security-relevant mode.
- **`--dev-insecure` interaction with `--jwt-secret`.** If both are supplied, the explicit secret wins (dev-insecure is a fallback, not an override). Decide and document; don't leave it ambiguous.

## Codex log

## Claude review

## Verdict
