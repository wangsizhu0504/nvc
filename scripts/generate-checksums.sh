#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -lt 1 ]]; then
  echo "usage: bash ./scripts/generate-checksums.sh <artifact> [artifact...]" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$@"
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$@"
else
  echo "no sha256 checksum tool found (expected sha256sum or shasum)" >&2
  exit 1
fi
