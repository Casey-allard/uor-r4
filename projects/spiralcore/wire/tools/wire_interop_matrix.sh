#!/usr/bin/env bash
# Interop/regression matrix: run every wire smoke and aggregate PASS markers.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
WORK="$PROJ/local/wire-matrix"
mkdir -p "$WORK"
PASS=0
FAIL=0
ROOT=$(cd "$PROJ/../.." && pwd)
OBJPEER=${OBJPEER:-$ROOT/target/release/objpeer}
export OBJPEER
if [ "${1:-}" = "--list" ]; then
  for n in socket-layer socket-routing socket-tls mux mux-routing interposer-macos interposer-linux-mvm hub-cli rust-interop rust-hub rust-adapter rust-mux adapter-pair adapter-hub adapter-policy adapter-metrics adapter-idle adapter-protocol; do echo "$n"; done
  echo "LEGS_COUNT=18"
  exit 0
fi
if [ ! -x "$OBJPEER" ]; then
  echo "MATRIX PRECHECK FAIL: no objpeer at $OBJPEER" >&2
  echo "  build: (cd \"$ROOT\" && cargo build --release -p uor-r4-object-store --bin objpeer)" >&2
  echo "  or set OBJPEER to an existing binary" >&2
  exit 1
fi
run() {
  local name=$1
  shift
  local log="$WORK/$name.log"
  if timeout 420 "$@" > "$log" 2>&1; then
    local marker
    marker=$(grep -m1 -E 'WIRE_[A-Z_]+_PASS|VERIFY_ALL_PASS' "$log")
    echo "MATRIX $name PASS $marker"
    PASS=$((PASS + 1))
  else
    echo "MATRIX $name FAIL"
    tail -8 "$log"
    FAIL=$((FAIL + 1))
  fi
}
run socket-layer python3 "$PROJ/tools/wire_socket_smoke.py"
run socket-routing python3 "$PROJ/tools/wire_socket_routing_smoke.py"
run socket-tls python3 "$PROJ/tools/wire_socket_tls_smoke.py"
run mux python3 "$PROJ/tools/wire_mux_smoke.py"
run mux-routing python3 "$PROJ/tools/wire_mux_routing_smoke.py"
run interposer-macos bash "$PROJ/tools/wire_interposer_smoke.sh"
run interposer-linux-mvm bash "$PROJ/tools/wire_interposer_linux_smoke_via_mvm.sh"
run hub-cli bash "$PROJ/tools/wire_hub_cli_smoke.sh"
run rust-interop bash "$PROJ/tools/wire_rust_interop_smoke.sh"
run rust-hub bash "$PROJ/tools/wire_rust_hub_smoke.sh"
run rust-adapter bash "$PROJ/tools/wire_rust_adapter_smoke.sh"
run rust-mux bash "$PROJ/tools/wire_rust_mux_gateway_smoke.sh"
run adapter-pair bash "$PROJ/tools/wire_adapter_smoke.sh"
run adapter-hub bash "$PROJ/tools/wire_adapter_hub_smoke.sh"
run adapter-policy bash "$PROJ/tools/wire_adapter_policy_smoke.sh"
run adapter-metrics bash "$PROJ/tools/wire_adapter_metrics_smoke.sh"
run adapter-idle bash "$PROJ/tools/wire_adapter_idle_smoke.sh"
run adapter-protocol bash "$PROJ/tools/wire_adapter_protocol_smoke.sh"
echo "WIRE_MATRIX_RESULT pass=$PASS fail=$FAIL"
[ "$FAIL" = "0" ] && echo "WIRE_INTEROP_MATRIX_PASS tests=$PASS" || exit 1
