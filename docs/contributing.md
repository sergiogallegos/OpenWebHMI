# Contributing to OpenWebHMI

Thank you for considering a contribution. OpenWebHMI is a community-first project: the design (gateway-centric, three-language stack, plugin-by-default) is built for outside contributions, not against them.

This document covers **what** you can contribute, **how** the contribution flows work, and **where** in the repo each kind of contribution lives.

For the architectural context behind these instructions, read [`architecture.md`](architecture.md) and [`roadmap.md`](roadmap.md) first.

## Contributor License Agreement

Every contribution merged on or after 2026-09-01 requires affirmative acceptance
of the [OpenWebHMI Contributor License Agreement](../CONTRIBUTOR_LICENSE_AGREEMENT.md),
version 1.0. Contributors retain their copyright while granting the project rights
needed to maintain and relicense accepted contributions. The contribution remains
available under the OpenWebHMI license governing the relevant material when it was
submitted, even if the project later offers it under additional terms.

The pull-request checklist records acceptance. If an employer, client, or another
entity owns or may own the contribution, an authorized representative must accept
the CLA or provide written authorization linked from the pull request. Mark material
that is not intended as a contribution accordingly and disclose all third-party
code, media, generated content, and license terms before review.

Outbound licensing follows [`LICENSE-POLICY.md`](../LICENSE-POLICY.md): product
code is AGPL-3.0-only, while only `crates/protocol` and `packages/protocol-ts`
are MPL-2.0. Documentation and marks have separate terms. Do not place code in a
protocol package merely to obtain the narrower MPL boundary.

---

## 1. What you can contribute

| You want to add… | You'll write… | Lives in… | Distributed via… |
|---|---|---|---|
| A new PLC / device driver | **Rust** crate implementing the `Driver` trait | Own repo or `crates/driver-*/` (community-maintained drivers can live in-tree if they pass the bar) | crates.io |
| A new HMI component (gauge, special chart, custom widget) | **TS / React** package with a component schema | Own repo or `packages/component-*/` | npm |
| A script / utility library available to project Python scripts | **Python** package | Project-bundled or pinned PyPI dep | PyPI / project bundle |
| A standard-library function (`system.*`) | **Rust** (host-side bridge) + **Python** stub | `crates/scripting/` + `packages/sdk/python-stubs/` | Built into core |
| An IDE / designer enhancement | **TS / React** (or **Rust** for Tauri-native bits) | `apps/designer/` | Built into core |
| A bug fix / test / docs improvement anywhere | Whatever language that area uses | The relevant directory | PR |
| A theme / asset library | Mostly TS + assets | Own repo or `packages/theme-*/` | npm |

---

## 2. The three plugin extension points

### 2.1 Drivers (Rust)

Drivers are the *only* code path between the gateway and a PLC / device. They run in-process inside the gateway under supervisor control.

**To add a driver:**

1. `cargo generate --git https://github.com/sergiogallegos/openwebhmi-driver-template` (template ships in Phase 4).
2. Implement the `openwebhmi_driver_api::Driver` trait.
3. Provide a `metadata()` returning vendor, models supported, capabilities.
4. Implement `connect / disconnect / read / write / subscribe / browse`.
5. Test against either real hardware or a documented simulator.
6. Publish to crates.io as `openwebhmi-driver-<vendor>`.

**Quality bar for inclusion in the official driver list:**

- Reconnect-with-backoff is correct (proven in tests by killing the simulator).
- Quality propagates correctly on disconnect / timeout.
- No Rust panics escape the driver task — they unwind cleanly so the supervisor can restart the driver. (Native faults, FFI aborts, and `unsafe` UB will still terminate the gateway; see [`architecture.md`](architecture.md) §4.4. Drivers must be reviewed for memory safety and FFI hygiene, not just panic discipline.)
- A `wiki/drivers/<your-driver>.md` page with: supported devices, data types, addressing format, known limitations, upgrade history.
- `cargo clippy -- -D warnings` clean.

### 2.2 Components (TypeScript / React)

Components render in both the designer (with adornments — selection handles, drag affordances) and the runtime (live, bound to tags).

**To add a component:**

1. `pnpm create openwebhmi-component` (template ships in Phase 4).
2. Implement the component as a React function.
3. Declare a `propsSchema` (JSON Schema) — this drives the property panel automatically.
4. Mark which props accept tag bindings via `bindableProps`.
5. Provide a `defaultProps` and a thumbnail for the designer palette.
6. Publish to npm as `@<scope>/openwebhmi-component-*`.

**Quality bar for inclusion in `packages/component-library` (the built-in set):**

- Works correctly when its bound tag is in `Bad` quality (visible bad-quality state, doesn't crash).
- Renders within 16ms for a typical prop set.
- Storybook story covering: default, bound to live tag, bound to bad-quality tag, edge values.
- Accessible (keyboard navigation, ARIA labels for interactive components).

### 2.3 Script libraries (Python)

Anything you'd reach for in a Python project script. These are user-authored and project-bundled by default — most won't go upstream. But generally useful libraries (e.g. `openwebhmi-recipe-helpers`) belong on PyPI and can be pinned by projects.

There is **no separate plugin format for Python**: it's just Python. The only OpenWebHMI-specific surface is the `system.*` module, which is auto-injected at script execution time.

---

## 3. The contribution flow

### 3.1 Discussing first

For anything bigger than a typo or a clearly-scoped bug fix, **open an issue first**. Describe:

- What problem it solves (real plant scenario beats abstract concern).
- Which phase of the roadmap it fits.
- Roughly what the design looks like.

We'd rather say "yes, but make the design X instead" before you've spent a weekend on it.

### 3.2 Branching

- `main` is the stable trunk.
- Feature branches: `feat/<short-name>`.
- Bugfix branches: `fix/<short-name>`.
- Driver / component contributions in their own repos: open a PR against the index of recommended community packages once published.

### 3.3 Commit messages

We don't enforce conventional commits. We do enforce **commits that explain the why**:

- Bad: `fix bug in tag engine`
- Good: `fix tag engine: drop subscriber refcount on client disconnect (was leaking driver polls)`

### 3.4 PR checklist

A PR is ready when:

- It addresses one thing. If you find another thing on the way, file an issue or a follow-up PR.
- Tests cover the new behavior. Driver work has integration tests against a simulator. Tag engine work has unit tests.
- `cargo test`, `cargo clippy -- -D warnings`, `pnpm lint`, `pnpm typecheck` are all green locally.
- The PR description has a **Test plan** section with the steps you ran.
- Docs touched if behavior changed: `architecture.md` for design changes, `feature-matrix.md` for scope changes, `wiki/log.md` for anything durable.

### 3.5 Reviews

- At least one maintainer approval before merge.
- Driver and component PRs must include a review by someone who has run the change against the simulator (or hardware) — not just code-read it.

### 3.6 Releases

- Core releases follow semver: breaking changes bump major.
- Plugin packages release independently. Their compatibility range is declared in the package metadata.

---

## 4. Local development

### 4.1 Prerequisites

- **Rust** stable (pinned in `rust-toolchain.toml`)
- **Node** 20+ and **pnpm** 9+
- **Python** 3.11+ (for scripting host work; not needed for gateway core)
- **Tauri prerequisites** (see https://tauri.app/start/prerequisites/) for designer work

### 4.2 First-time setup

> **Pre-Phase-0 caveat.** While the workspace is empty (before Phase 0 lands its first crates and packages), `cargo build --workspace` reports "virtual workspace has no members" and exits — that's expected, not a misconfiguration. `cargo metadata` and `pnpm install` succeed today; the rest of this section becomes runnable when the first Phase 0 crate is committed. See [`docs/roadmap.md`](roadmap.md#phase-0--foundations-target-1-month).

```bash
git clone https://github.com/sergiogallegos/OpenWebHMI.git
cd OpenWebHMI
pnpm install
cargo build --workspace   # available once Phase 0 crates land
```

### 4.3 Running the gateway with the simulator

(Available from Phase 1 onward.)

```bash
# Terminal 1: simulated EtherNet/IP target
pnpm --filter @openwebhmi/sim-rockwell start

# Terminal 2: gateway
cargo run -p openwebhmi-gateway -- --config dev/gateway.toml

# Terminal 3: web runtime
pnpm --filter @openwebhmi/runtime-web dev

# Terminal 4 (optional): designer
pnpm --filter @openwebhmi/designer tauri dev
```

### 4.4 Tests

```bash
# Rust workspace
cargo test --workspace

# TS workspaces
pnpm test
```

---

## 5. Code style

- **Rust:** rustfmt (default), clippy clean. Public APIs documented with `///` comments.
- **TypeScript:** project-standard ESLint + Prettier config. Strict mode on.
- **Python:** `ruff` for lint + format.
- **Comments:** explain *why*, not *what*. The code already says what.

---

## 6. Code of conduct

Be kind. Be specific. Assume good faith. The full Contributor Covenant text will land in `CODE_OF_CONDUCT.md` before 1.0.

---

## 7. Getting help

- Issues for bugs, feature requests, design discussions.
- Discussions for open-ended questions, "how would I…", show-and-tell.
- Tag a maintainer in a PR if you've been waiting more than a week for review.

---

## 8. Maintainers

- Sergio Gallegos — repo owner — sergiogallegos.net

To become a maintainer: sustained, high-quality contributions over a few months, plus willingness to review others' work. Maintainers are nominated by existing maintainers, no application form.
