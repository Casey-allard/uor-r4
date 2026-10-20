#!/usr/bin/env bash
set -e

LAUNCHER_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$LAUNCHER_DIR/uor-r4-cli" "$@"
