#!/usr/bin/env bash
# Run the Linux LD_PRELOAD interposer smoke inside the mvm microVM (macOS host only).
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
MVMCTL=$(printenv MVMCTL)
[ -n "$MVMCTL" ] || MVMCTL="$HOME/.local/bin/mvmctl"
[ -x "$MVMCTL" ] || { echo "missing mvmctl" >&2; exit 2; }
WIRE_SMOKE_WORK=/tmp/wire-interposer-linux "$MVMCTL" machine run --image python:3.12 --mount "$PROJ/tools:/work/tools" -- env WIRE_SMOKE_WORK=/tmp/wire-interposer-linux sh /work/tools/wire_interposer_linux_smoke.sh
