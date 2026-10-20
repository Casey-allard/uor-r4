#!/usr/bin/env bash
# UOROBJ01 non-Rust conformance smoke (host only).
#
# Runs the independent Python agent (port_agent.py, hashlib/socket/struct only)
# against the Rust objpeer server:
#   1. audit an empty Python store  -> stale, exit 3 (documented contract),
#   2. pull the artifact through the Python client and cmp it byte-for-byte,
#   3. audit the warmed Python store -> verified, exit 0.
#
# No pod, no model compute. Override OBJPEER to use an existing binary.
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
PORT_AGENT="$ROOT/projects/spiralcore/wire/tools/port_agent.py"
OBJPEER_HINT=$OBJPEER
if [ -z "$OBJPEER_HINT" ]; then OBJPEER="$ROOT/target/release/objpeer"; else OBJPEER=$OBJPEER_HINT; fi
WORK="$ROOT/local/uorobj01-conformance.$$"
SERVER_PID=
ADDR=

if [ ! -x "$OBJPEER" ]; then
  echo "building objpeer (release) ..." >&2
  (cd "$ROOT" && ~/.cargo/bin/cargo build --release -p uor-r4-object-store --bin objpeer) >&2 || {
    echo "UOROBJ01_CONFORMANCE_FAIL: cargo build" >&2
    exit 1
  }
fi

cleanup() {
  if [ -n "$SERVER_PID" ]; then kill "$SERVER_PID" 2>/dev/null; wait "$SERVER_PID" 2>/dev/null; fi
  rm -rf "$WORK"
}
trap cleanup EXIT
mkdir -p "$WORK/python" "$WORK/server"
ART="$WORK/artifact.bin"
yes 'UOROBJ01 conformance payload: foreign-runtime peer against the Rust server' |
  head -c 262144 > "$ART"
BYTES=$(wc -c < "$ART" | tr -d ' ')
if command -v shasum >/dev/null 2>&1; then
  SHA=$(shasum -a 256 "$ART" | awk '{ print $1 }')
else
  SHA=$(sha256sum "$ART" | awk '{ print $1 }')
fi
printf '%s\n' "$SHA" > "$WORK/targets.txt"

"$OBJPEER" serve --store "$WORK/server/store" --object "$ART" --bind 127.0.0.1:0 --sessions 3 > "$WORK/server.log" 2>&1 &
SERVER_PID=$!
i=0
while [ "$i" -lt 600 ]; do
  ADDR=$(awk '/^READY / { print $2; exit }' "$WORK/server.log")
  [ -n "$ADDR" ] && break
  sleep 0.1
  i=$((i + 1))
done
if [ -z "$ADDR" ]; then
  echo "UOROBJ01_CONFORMANCE_FAIL: server did not become ready" >&2
  cat "$WORK/server.log" >&2
  exit 1
fi
echo "UOROBJ01_CONFORMANCE artifact_bytes=$BYTES sha256=$SHA addr=$ADDR"

# 1. Cold audit from an empty Python store: stale, exit 3.
AUDIT_RC=0
python3 "$PORT_AGENT" audit "$ADDR" "$WORK/python/store.json" "$WORK/targets.txt" --json > "$WORK/audit-stale.json" 2> "$WORK/audit-stale.err" || AUDIT_RC=$?
if [ "$AUDIT_RC" != 3 ]; then
  echo "UOROBJ01_CONFORMANCE_FAIL: cold audit rc=$AUDIT_RC (want 3 stale)" >&2
  cat "$WORK/audit-stale.err" >&2
  cat "$WORK/audit-stale.json" >&2
  exit 1
fi
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["checks"]==1 and d["stale"]==1 and d["verified"]==0 and d["unknown"]==0, d' "$WORK/audit-stale.json" || {
  echo "UOROBJ01_CONFORMANCE_FAIL: cold audit JSON" >&2; exit 1; }
echo "UOROBJ01_CONFORMANCE cold_audit stale=1 exit=3"

# 2. Pull through the Python client, pinned; the Rust server serves the records.
python3 "$PORT_AGENT" pull "$ADDR" "$SHA" "$WORK/python/store.json" "$WORK/pulled.bin" \
  --expect-sha256 "$SHA" --expect-bytes "$BYTES" > "$WORK/pull.log" 2> "$WORK/pull.err" || {
  echo "UOROBJ01_CONFORMANCE_FAIL: python pull" >&2
  cat "$WORK/pull.err" >&2
  exit 1
}
grep -q "^PORT_PULL_OK" "$WORK/pull.log" || {
  echo "UOROBJ01_CONFORMANCE_FAIL: no PORT_PULL_OK marker" >&2; cat "$WORK/pull.log" >&2; exit 1; }
cmp -s "$ART" "$WORK/pulled.bin" || {
  echo "UOROBJ01_CONFORMANCE_FAIL: pulled bytes differ" >&2; exit 1; }
cat "$WORK/pull.log"
echo "UOROBJ01_CONFORMANCE pull bytes=$BYTES byte_identical=true"

# 3. Warm audit from the Python store: verified, exit 0.
AUDIT_RC=0
python3 "$PORT_AGENT" audit "$ADDR" "$WORK/python/store.json" "$WORK/targets.txt" --json > "$WORK/audit-verified.json" 2> "$WORK/audit-verified.err" || AUDIT_RC=$?
if [ "$AUDIT_RC" != 0 ]; then
  echo "UOROBJ01_CONFORMANCE_FAIL: warm audit rc=$AUDIT_RC (want 0 verified)" >&2
  cat "$WORK/audit-verified.err" >&2
  exit 1
fi
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["checks"]==1 and d["verified"]==1 and d["stale"]==0 and d["unknown"]==0, d' "$WORK/audit-verified.json" || {
  echo "UOROBJ01_CONFORMANCE_FAIL: warm audit JSON" >&2; exit 1; }
cat "$WORK/audit-verified.json"
echo "UOROBJ01_CONFORMANCE warm_audit verified=1 exit=0"
echo "UOROBJ01_CONFORMANCE_PASS checks=1 pull_bytes=$BYTES byte_identical=true stale_then_verified=true"
