---
id: CODEX-AR
title: Raspberry Pi deployment guide — hardware spec, build, systemd, known limits
owner: codex
phase: 4
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

# CODEX-AR — Raspberry Pi deployment guide

## Brief

> Document how to deploy the OpenWebHMI gateway (and optionally the runtime web app) on a Raspberry Pi 4 / 5 — the "edge HMI on cheap ARM hardware" use case that machine builders love. Raspberry Pi is a common edge-HMI runtime target across the SCADA market (with a typical 2 GB RAM minimum cited for headless gateway use); OpenWebHMI's Rust gateway compiles for ARM but has no Pi-specific deployment guidance. **Docs task with optional cross-compile script.**

### Goal

A machine builder reading `docs/deployment/raspberry-pi.md` can stand up an OpenWebHMI gateway on a Pi 4 or Pi 5, persist it as a systemd service, point the runtime web app at it, and know which Pi-specific gotchas to expect. The doc is grounded in an actual deployment the brief author or Codex performed — not generic ARM Linux advice.

### Context to read first

- `crates/gateway/Cargo.toml` and `Cargo.toml` workspace — confirm what compiles to `aarch64-unknown-linux-gnu`.
- `apps/runtime-web/` — confirm whether the static bundle deploys independently or whether the gateway serves it (designer-side proxy vs production bundle).
- `crates/scripting/` — Python 3.11+ subprocess host; check the ARM Pi has Python 3.11 readily available (Raspbian / Pi OS Bookworm ships 3.11.2).
- `docs/agents/notes/toolchain-drift.md` — toolchain pinning matters here; Pi cross-compile must use the pinned Rust version.
- Any existing docker / deployment guidance in `docs/` or the README.

### Files to create / modify

- **Create** `docs/deployment/raspberry-pi.md` — the main artifact:
  - Hardware: Pi 4 (4 GB recommended; 2 GB minimum if not running the runtime locally), Pi 5; storage (SD card vs SSD, recommend SSD for production); networking.
  - OS: Pi OS 64-bit Bookworm minimum (Python 3.11, glibc version, systemd 252).
  - **Path A — cross-compile** on x86 dev machine: `cross` or `cargo-zigbuild`, exact `rustup target add` invocation, `cargo build --release --target aarch64-unknown-linux-gnu`, scp the binary.
  - **Path B — native compile** on the Pi: `apt install build-essential` etc., expected build time on Pi 4 (~20-40 min for the workspace).
  - Configuration: where to put the project store, the historian SQLite, the audit-log SQLite, certs, logs.
  - **`systemd` unit file**: ship a sample at `docs/deployment/openwebhmi-gateway.service`. User, working directory, restart policy, journal logging, file-descriptor limit.
  - Networking: bind to `0.0.0.0:8080`, reverse-proxy with `caddy` or `nginx` for TLS in production (point at `scripts/dev-self-signed-cert.sh` for the dev shape).
  - Performance characteristics observed: tag throughput, web-app cold-load time, RAM at idle.
  - **Known limitations**:
    - Designer (Tauri) does **not** run on Pi — design on a desktop, deploy the project, runtime + gateway on Pi.
    - Native `openssl` build can be slow / fragile on Pi; prefer `rustls`-using paths.
    - SD-card wear under historian write load — recommend SSD for production.
    - 32-bit Pi OS is **not** supported; the Rust stack assumes 64-bit ABI.
- **Create** `docs/deployment/openwebhmi-gateway.service` — sample systemd unit.
- **Optional**: `scripts/cross-compile-pi.sh` — wrapper around `cross build` with the right target triple. Only if cross-compile path is committed; if native-only, skip.
- **Modify** `README.md` — add Raspberry Pi to the deployment-target line; link to `docs/deployment/raspberry-pi.md`.
- **Modify** `docs/architecture.md` — single-sentence mention that ARM Linux is supported, link out.

### Behavior

The doc reflects a **real deployment**, not theoretical guidance. Either Codex deploys to a real Pi 4 / 5 (preferred) or the maintainer does and Codex documents the steps from the maintainer's notes. The Codex log states which path was taken.

If a real deployment isn't feasible in Codex's environment, the doc says so in a "Verification status" section at the top: "Cross-compile verified on x86 Linux 2026-05-XX; native deploy to Pi 4 not performed in this environment — maintainer-validated separately."

### Test requirements

- The cross-compile build must complete (CI doesn't need to run on ARM, but a one-shot `cargo build --target aarch64-unknown-linux-gnu` on a dev machine confirms the target is buildable today). Document the command and the resulting binary path.
- (Optional, only if a Pi is available) Smoke test: start the gateway, hit `/healthz`, write a tag, read it back, stop cleanly. Document timings.
- Doc-only verification: every command in the doc has been executed by the brief author or Codex; no hypothetical invocations. Cite the OS version, Pi model, and Rust version used.

### Acceptance criteria

- [ ] `docs/deployment/raspberry-pi.md` exists and covers all sections listed above.
- [ ] `docs/deployment/openwebhmi-gateway.service` sample systemd unit ships.
- [ ] `README.md` links to the new doc.
- [ ] Codex log states whether the doc was verified on a real Pi or only via cross-compile.
- [ ] Designer-on-Pi limitation is called out explicitly (designer is Tauri-desktop only).
- [ ] Cross-compile invocation is correct against the pinned `rust-toolchain.toml`.

### Out of scope

- **Pi-specific binary distribution.** No `.deb` package, no apt repo, no Pi OS image. Manual install only.
- **Pi-on-CI testing.** ARM CI runners are slow and expensive; not justified at v1.x.
- **Embedded Linux distros (Yocto, Buildroot).** Pi OS Bookworm only. Other distros are integrator-handled.
- **Hardware GPIO bindings.** Not in scope; the gateway speaks PLC drivers, not GPIO directly. If a customer wants GPIO, a custom driver is the path.
- **Kiosk-mode browser configuration.** Deployment of the runtime web app as a fullscreen kiosk on a Pi-attached display is a separate doc; mention but don't write.

### Risks / gotchas

- **Don't claim Pi 4 / 5 works if it wasn't tested.** Honesty checklist (from CLAUDE.md "Code quality and testing discipline → Honesty") applies double here — a "verified on Pi 4 Bookworm 2026-05-XX" statement is load-bearing for the customer pulling the doc up.
- **Pi 5 vs Pi 4 differences:** Pi 5 has noticeably better single-thread performance and uses NVMe via the PCIe HAT; document both as supported but cite numbers for the model actually tested.
- **`openssl` vs `rustls`.** Some Rust crates default to `openssl` which requires `libssl-dev` + matching version. Audit `crates/gateway` and any driver Cargo.toml; if `openssl` is present, prefer the `rustls` feature flag on Pi to avoid the system-library coupling.
- **SD-card wear with historian writes**. Cite expected write amplification and recommend SSD-via-USB-3 or NVMe-via-HAT for any production deployment recording > 100 samples/sec.
- **Bookworm Python is 3.11.2.** The `crates/scripting` requirement is "3.11+" — confirm the Pi OS default satisfies this. If Pi OS still ships 3.10 anywhere, document `apt install python3.11` or pyenv as the workaround.
- **Time-sync.** Industrial Pi deployments are often air-gapped; the audit-log and historian timestamps drift without NTP. Note `systemd-timesyncd` or a local NTP server as a deployment-time decision.

## Codex log

## Claude review

## Verdict
