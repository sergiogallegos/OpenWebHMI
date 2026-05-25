# MQTT TLS in CI

## The constraint

`rumqttd 0.20.0` — the embedded MQTT broker used by `crates/driver-mqtt`'s integration tests — is **plaintext-only**. It does not support TLS termination. This means CI cannot exercise the TLS path against the embedded broker.

## What CI actually tests

- Plaintext MQTT against `rumqttd 0.20.0` — full integration coverage.
- TLS code paths exercise against unit tests with mocked connect responses.

## What manual smoke tests

The real TLS path is validated against an **external Mosquitto broker** configured with a self-signed CA. The fixture lives outside the repo; setup steps are documented in the designer manual-smoke checklist (step #27 in `apps/designer/README.md`).

This is the maintainer's responsibility before any `v0.x.0` tag that includes MQTT changes.

## Why this matters in reviews

A submission that claims "MQTT TLS tested in CI" is a verification mismatch — the rumqttd broker can't terminate TLS. Reviews should:

- Distinguish "TLS code path compiles and unit-tests pass" (true in CI) from "TLS end-to-end validated" (manual-smoke only).
- Flag any test that pretends to do TLS against rumqttd as misleading — those tests must either move to mocks or be excluded with a `#[ignore]` + comment pointing here.

## The path to lifting this

Either:

1. Upstream `rumqttd` adds TLS termination (no ETA — track upstream).
2. Replace the embedded broker for TLS tests with a containerized Mosquitto fixture in CI. Considered v1.1 scope; would add CI runtime and a Docker dependency, so not yet prioritized.

Until then, the manual-smoke gate is the only real TLS validation.

## See also

- `crates/driver-mqtt/tests/` — integration test layout.
- `apps/designer/README.md` step #27 — manual smoke TLS fixture.
