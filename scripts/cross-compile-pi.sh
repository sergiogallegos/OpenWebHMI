#!/usr/bin/env bash
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
PACKAGE="openwebhmi-gateway"

if ! command -v cross >/dev/null 2>&1; then
  echo "cross is required for this wrapper. Install it with: cargo install cross --locked" >&2
  exit 2
fi

rustup target add "${TARGET}"
cross build -p "${PACKAGE}" --release --target "${TARGET}" --locked
echo "Built target/${TARGET}/release/openwebhmi-gateway"
