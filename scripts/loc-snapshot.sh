#!/usr/bin/env bash
# Reproducible LOC snapshot for OpenWebHMI.
#
# Usage:  bash scripts/loc-snapshot.sh
#
# Output is the canonical "what's the project's LOC" report referenced by
# docs/scale-estimates.md. To capture a new row in the progress history,
# run this, copy the totals, and append a row to the table in that file.
#
# Requires `tokei` (`brew install tokei` or `cargo install tokei`).

set -euo pipefail

if ! command -v tokei >/dev/null 2>&1; then
  echo "tokei is not installed. Install with: brew install tokei  (or  cargo install tokei)" >&2
  exit 1
fi

cd "$(git rev-parse --show-toplevel)"

echo "OpenWebHMI LOC snapshot"
echo "  date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "  commit: $(git rev-parse --short HEAD)"
echo "  branch: $(git rev-parse --abbrev-ref HEAD)"
echo

tokei \
  --exclude target \
  --exclude node_modules \
  --exclude dist \
  --exclude pnpm-lock.yaml \
  --exclude Cargo.lock \
  --exclude .obsidian
