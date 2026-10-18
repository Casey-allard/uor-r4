#!/usr/bin/env bash
# Hub-mode smoke: one gateway, two named routes to different services, per-route stats.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
REPO_ROOT=$(cd "$PROJ/../.." && pwd)
if [ -z "$(printenv OBJPEER)" ]; then
  OBJPEER="$REPO_ROOT/target/release/objpeer"
else
  OBJPEER=$(printenv OBJPEER)
fi
[ -x "$OBJPEER" ] || { echo "missing objpeer: $OBJPEER (build: cargo build --release -p uor-r4-object-store --bin objpeer)" >&2; exit 2; }
mkdir -p "$PROJ/local"
WORK=$(mktemp -d "$PROJ/local/wire-hub.XXXXXX")
ALL_PIDS=""
cleanup() { for pid in $ALL_PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done; rm -rf "$WORK"; }
trap cleanup EXIT
cd "$WORK"
fail() { echo "WIRE_ADAPTER_HUB_FAIL: $1" >&2; exit 1; }
track() { ALL_PIDS="$ALL_PIDS $1"; }
poll() {
  local log=$1 value=''
  for _ in $(seq 1 600); do
    value=$(grep -m1 '^READY ' "$log" 2>/dev/null | awk '{print $2}')
    [ -n "$value" ] && break
    sleep 0.1
  done
  [ -n "$value" ] || fail "no READY in $log"
  echo "$value"
}
python3 - "$WORK" <<'PYMAKE'
import pathlib, sys
root = pathlib.Path(sys.argv[1])
letters = b'abcdefghijklmnopqrstuvwxyz'
root.joinpath('a.bin').write_bytes(bytes(letters[i % 26] for i in range(2 * 1024 * 1024)))
root.joinpath('b.bin').write_bytes(bytes(letters[(i * 3 + 7) % 26] for i in range(2 * 1024 * 1024)))
PYMAKE
SHA_A=$(shasum -a 256 a.bin | awk '{print $1}')
SHA_B=$(shasum -a 256 b.bin | awk '{print $1}')
"$OBJPEER" serve --store a.store --object a.bin --sessions 2 --bind 127.0.0.1:0 > a.log 2>&1 &
A_PID=$!
track "$A_PID"
ADDR_A=$(poll a.log)
"$OBJPEER" serve --store b.store --object b.bin --sessions 1 --bind 127.0.0.1:0 > b.log 2>&1 &
B_PID=$!
track "$B_PID"
ADDR_B=$(poll b.log)
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 \
  --route "model=$ADDR_A" --route "store=$ADDR_B" --stats-file "$WORK/hub.json" > hub.log 2>&1 &
HUB_PID=$!
track "$HUB_PID"
HUB=$(poll hub.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$HUB" --route model --stats-file "$WORK/model.json" > model.log 2>&1 &
MODEL_PID=$!
track "$MODEL_PID"
MODEL=$(poll model.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$HUB" --route store --stats-file "$WORK/store.json" > store.log 2>&1 &
STORE_PID=$!
track "$STORE_PID"
STORE=$(poll store.log)
"$OBJPEER" pull --store m.store --addr "$MODEL" --select-sha256 "$SHA_A" --out m.bin > m1.log 2>&1 || fail 'model route pull A'
cmp -s m.bin a.bin || fail 'model route payload'
"$OBJPEER" pull --store s.store --addr "$STORE" --select-sha256 "$SHA_B" --out s.bin > s1.log 2>&1 || fail 'store route pull B'
cmp -s s.bin b.bin || fail 'store route payload'
if "$OBJPEER" pull --store w.store --addr "$MODEL" --select-sha256 "$SHA_B" --out w.bin > w1.log 2>&1; then
  fail 'wrong route unexpectedly served B'
fi
kill -TERM "$MODEL_PID" 2>/dev/null || true
kill -TERM "$STORE_PID" 2>/dev/null || true
kill -TERM "$HUB_PID" 2>/dev/null || true
sleep 0.5
python3 - "$WORK/hub.json" <<'PYSTATS'
import json, sys
d = json.load(open(sys.argv[1]))
routes = d.get('routes', {})
assert routes.get('model', {}).get('connections') == 2, routes
assert routes.get('store', {}).get('connections') == 1, routes
print('hub_routes=' + json.dumps(routes, sort_keys=True))
PYSTATS
echo "WIRE_ADAPTER_HUB_PASS model_ratio=$(python3 -c "import json; print(json.load(open('$WORK/model.json'))['ratio'])") store_ratio=$(python3 -c "import json; print(json.load(open('$WORK/store.json'))['ratio'])")"
