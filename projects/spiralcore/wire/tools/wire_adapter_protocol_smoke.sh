#!/usr/bin/env bash
# Full protocol through the adapter pair: pull, verify, pinned verify, audit, repair, sync.
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
WORK=$(mktemp -d "$PROJ/local/wire-protocol.XXXXXX")
ALL_PIDS=""
cleanup() { for pid in $ALL_PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done; rm -rf "$WORK"; }
trap cleanup EXIT
cd "$WORK"
fail() { echo "WIRE_ADAPTER_PROTOCOL_FAIL: $1" >&2; exit 1; }
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
root.joinpath('a.bin').write_bytes(bytes(letters[i % 26] for i in range(512 * 1024)))
root.joinpath('b.bin').write_bytes(bytes(letters[(i * 3 + 7) % 26] for i in range(512 * 1024)))
PYMAKE
SHA_A=$(shasum -a 256 a.bin | awk '{print $1}')
SHA_B=$(shasum -a 256 b.bin | awk '{print $1}')
printf '%s\n%s\n' "$SHA_A" "$SHA_B" > targets.txt
"$OBJPEER" serve --store server.store --object a.bin --object b.bin --sessions 20 --bind 127.0.0.1:0 > server.log 2>&1 &
SERVER_PID=$!
track "$SERVER_PID"
SERVER=$(poll server.log)
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$SERVER" --stats-file "$WORK/gw.json" > gw.log 2>&1 &
GW_PID=$!
track "$GW_PID"
GATEWAY=$(poll gw.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$GATEWAY" --stats-file "$WORK/edge.json" > edge.log 2>&1 &
EDGE_PID=$!
track "$EDGE_PID"
EDGE=$(poll edge.log)
"$OBJPEER" pull --store client.store --addr "$EDGE" --select-sha256 "$SHA_A" --out a.out > pull-a.log 2>&1 || fail 'pull A'
cmp -s a.out a.bin || fail 'pull A payload'
"$OBJPEER" pull --store client.store --addr "$EDGE" --select-sha256 "$SHA_B" --out b.out > pull-b.log 2>&1 || fail 'pull B'
cmp -s b.out b.bin || fail 'pull B payload'
"$OBJPEER" verify --store client.store --addr "$EDGE" --select-sha256 "$SHA_A" --select-sha256 "$SHA_B" --json > verify.json 2>&1 || fail 'verify'
grep -q '"verified":2' verify.json || fail 'verify count'
"$OBJPEER" verify --store client.store --addr "$EDGE" --select-sha256 "$SHA_A" --expect-sha256 "$SHA_A" --expect-bytes 524288 --json > pinned.json 2>&1 || fail 'pinned verify'
"$OBJPEER" verify --store client.store --addr "$EDGE" --targets targets.txt --json > audit.json 2>&1 || fail 'audit'
python3 - audit.json <<'PYAUDIT'
import json, sys
d = json.load(open(sys.argv[1]))
assert d['checks'] == 2 and d['verified'] == 2 and d['stale'] == 0, d
PYAUDIT
"$OBJPEER" repair --store repaired.store --addr "$EDGE" --targets targets.txt --json > repair.json 2>&1 || fail 'repair'
python3 - repair.json <<'PYREPAIR'
import json, sys
d = json.load(open(sys.argv[1]))
assert d['repaired'] == 2 and d['unavailable'] == 0, d
PYREPAIR
"$OBJPEER" pull --store repaired.store --addr "$EDGE" --select-sha256 "$SHA_B" --out warm.out > warm.log 2>&1 || fail 'post-repair pull'
grep -q 'nodes=0' warm.log || fail 'repaired store not warm'
mkdir -p sync-out
"$OBJPEER" sync --store sync.store --addr "$EDGE" --targets targets.txt --out-dir sync-out > sync.log 2>&1 || fail 'sync'
grep -q 'SYNC_SERIES artifacts=2' sync.log || fail 'sync count'
cmp -s "sync-out/$SHA_A.bin" a.bin || fail 'sync artifact A'
cmp -s "sync-out/$SHA_B.bin" b.bin || fail 'sync artifact B'
kill -TERM "$EDGE_PID" 2>/dev/null || true
sleep 0.5
python3 - "$WORK/edge.json" <<'PYSTATS'
import json, sys
d = json.load(open(sys.argv[1]))
assert d['connections'] >= 7, d['connections']
print('edge_connections=%d ratio=%.4f' % (d['connections'], d['ratio']))
PYSTATS
echo "WIRE_ADAPTER_PROTOCOL_PASS pull=2 verify=2 pinned=1 audit=2 repair=2 sync=2 warm=0"
