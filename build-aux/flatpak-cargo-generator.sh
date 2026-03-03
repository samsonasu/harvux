#!/usr/bin/env bash
# Generate cargo-sources.json for Flatpak offline builds.
#
# Prerequisites:
#   pip install flatpak-cargo-generator
#   (or clone https://github.com/niclas-aspect/flatpak-cargo-generator)
#
# Run from the project root:
#   ./build-aux/flatpak-cargo-generator.sh
#
# Re-run whenever Cargo.lock changes.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

python3 -m flatpak_cargo_generator \
    "${PROJECT_ROOT}/Cargo.lock" \
    -o "${PROJECT_ROOT}/cargo-sources.json"

echo "Generated cargo-sources.json"
