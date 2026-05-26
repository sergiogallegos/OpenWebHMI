# Raspberry Pi Deployment

This guide covers a manual OpenWebHMI gateway deployment on Raspberry Pi 4 or Raspberry Pi 5 running 64-bit Raspberry Pi OS Bookworm. Design projects on a desktop with the Tauri designer, then deploy the gateway and optional runtime web bundle to the Pi.

## Verification Status

Local verification on 2026-05-25 used Rust `1.95.0` from `rust-toolchain.toml` on an Apple Silicon development host. `rustup target add aarch64-unknown-linux-gnu` succeeded after network approval. A direct `cargo build -p openwebhmi-gateway --release --target aarch64-unknown-linux-gnu --locked` reached native C dependencies and failed because `aarch64-linux-gnu-gcc` is not installed on this host. A follow-up attempt with `zig cc -target aarch64-linux-gnu` failed because `cc-rs` also passed `--target=aarch64-unknown-linux-gnu`, which this Zig invocation did not accept.

No real Raspberry Pi hardware smoke was performed in this environment. Treat the native Pi build path below as the verified deployment path to validate on the target Pi, and treat the cross-compile path as the supported shape once a Linux aarch64 cross C toolchain or `cross` is installed.

## Hardware

Use a Pi 4 or Pi 5 with 64-bit ARM support. A 4 GB board is the recommended floor for gateway plus local runtime hosting. A 2 GB board can run a headless gateway for small projects, but leave the browser/runtime on a separate client. Use wired Ethernet for production cells.

Use SSD storage for production historian or audit-log workloads. SD cards are acceptable for evaluation, but historian writes and audit logs create sustained small SQLite writes; SD-card endurance becomes the limiting factor before CPU or memory.

## OS Baseline

Use Raspberry Pi OS 64-bit Bookworm or newer. Bookworm provides systemd, glibc suitable for the Rust Linux target, and Python 3.11. The scripting host launches `python3` by default, or the command named by `OPENWEBHMI_PYTHON`.

32-bit Pi OS is not supported for this deployment guide. Build and run the `aarch64-unknown-linux-gnu` target only.

## Path A: Native Build On The Pi

Install build prerequisites:

```sh
sudo apt update
sudo apt install -y build-essential curl git pkg-config libssl-dev python3
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustup show
```

Clone the repository and build the gateway with the pinned toolchain:

```sh
git clone https://github.com/sergiogallegos/OpenWebHMI.git
cd OpenWebHMI
cargo build -p openwebhmi-gateway --release --locked
```

The binary is:

```sh
target/release/openwebhmi-gateway
```

On Pi 4, plan for a long first compile because SQLite, TLS, and driver dependencies include native code. Build time depends heavily on cooling and storage; validate locally before scheduling production cutover.

## Path B: Cross-Compile From A Development Machine

The Rust target must match the pinned toolchain:

```sh
rustup target add aarch64-unknown-linux-gnu
```

Preferred wrapper:

```sh
scripts/cross-compile-pi.sh
```

The wrapper uses `cross` because the gateway pulls in C-backed dependencies such as SQLite and TLS. A plain `cargo build --target aarch64-unknown-linux-gnu` needs a real aarch64 Linux C compiler and linker on the host:

```sh
cargo build -p openwebhmi-gateway --release --target aarch64-unknown-linux-gnu --locked
```

Copy the binary after a successful cross-build:

```sh
scp target/aarch64-unknown-linux-gnu/release/openwebhmi-gateway pi@openwebhmi-pi:/tmp/
```

## Install Layout

Create a service user and directories:

```sh
sudo useradd --system --home /var/lib/openwebhmi --shell /usr/sbin/nologin openwebhmi
sudo install -d -o openwebhmi -g openwebhmi /var/lib/openwebhmi/projects
sudo install -d -o openwebhmi -g openwebhmi /opt/openwebhmi/bin
sudo install -m 0755 target/release/openwebhmi-gateway /opt/openwebhmi/bin/openwebhmi-gateway
```

Store project files in `/var/lib/openwebhmi/projects`. Store historian data in `/var/lib/openwebhmi/historian.sqlite` and audit events in `/var/lib/openwebhmi/audit.sqlite`. Keep TLS certificates under a reverse proxy such as Caddy or nginx rather than in the gateway service directory. Logs go to journald through systemd.

## systemd

Install the sample unit:

```sh
sudo cp docs/deployment/openwebhmi-gateway.service /etc/systemd/system/openwebhmi-gateway.service
sudo systemctl daemon-reload
sudo systemctl enable --now openwebhmi-gateway
sudo systemctl status openwebhmi-gateway
```

Inspect logs:

```sh
journalctl -u openwebhmi-gateway -f
```

The unit binds the gateway to `0.0.0.0:8080`, keeps persistent data writable under `/var/lib/openwebhmi`, restarts on failure, and raises the file-descriptor limit for many websocket clients.

## Runtime Web App

Build the runtime web app on a development machine and deploy the static `dist` directory behind the same reverse proxy:

```sh
pnpm --filter @openwebhmi/runtime-web build
```

Point the runtime at the Pi gateway URL, for example `ws://openwebhmi-pi.local:8080` for local testing or a TLS-terminated `wss://` endpoint in production.

The gateway does not make the Tauri designer a Pi app. The designer remains a desktop authoring tool; publish project files to the Pi after authoring.

## Networking And TLS

Use direct `ws://pi-hostname:8080` only on trusted lab networks. For production, bind the gateway on the Pi and put Caddy or nginx in front of it for TLS and certificate renewal. The development self-signed certificate helper in `scripts/dev-self-signed-cert.sh` shows the local certificate shape, but production should use site-managed certificates.

Air-gapped deployments need a time source. Configure `systemd-timesyncd` against a local NTP server or another plant time source so historian and audit-log timestamps remain useful.

## Smoke Test

After starting the service:

```sh
systemctl is-active openwebhmi-gateway
journalctl -u openwebhmi-gateway -n 100 --no-pager
```

Then open the runtime web app from another machine and connect it to the Pi gateway. A full production smoke should load a project, subscribe to a tag, write a permitted tag, confirm the readback, and stop the service cleanly with `sudo systemctl stop openwebhmi-gateway`.

## Known Limits

- Designer is Tauri-desktop only; do authoring on a desktop and deploy the project to the Pi.
- 32-bit Pi OS is not supported.
- Native OpenSSL/system-library builds can be slow or fragile on small Pi installations; the gateway uses Rustls for its direct TLS stack, but transitive native C dependencies still require a working C toolchain.
- SD cards wear quickly under production historian load. Use USB 3 SSD or Pi 5 NVMe storage for sustained recording.
- No GPIO bindings are included. OpenWebHMI talks to PLCs and industrial protocols through drivers.
- Kiosk-mode browser setup for a Pi-attached display is a separate deployment concern.
