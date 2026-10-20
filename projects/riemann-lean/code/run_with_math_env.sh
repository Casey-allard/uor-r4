#!/usr/bin/env bash
set -euo pipefail

# Keep matplotlib and other cache-writing libs inside workspace-writable paths.
if [[ -z "${MPLCONFIGDIR:-}" ]]; then
  export MPLCONFIGDIR="$(pwd)/research/cache/mplconfig"
fi
mkdir -p "${MPLCONFIGDIR}"

exec "$@"
