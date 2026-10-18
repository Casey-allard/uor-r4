#!/usr/bin/env bash
# Fleet maintenance demo (host only): audit -> failover sync -> repair -> audit.
#
# Two real objpeer replicas on loopback hold overlapping artifact sets. The demo
#   1. audits a fresh client against both replicas with the real verify command,
#   2. kills one replica and shows sync failing over to the surviving replica,
#   3. brings the first replica back and repairs the one missing artifact,
#   4. audits again and checks every target is verified on at least one replica.
#
# No pod, no model compute. Override OBJPEER to use an existing binary.
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
CRATE=$(cd "$SCRIPT_DIR/.." && pwd)
ROOT=$(cd "$CRATE/../.." && pwd)
OBJPEER=$OBJPEER
if [ -z "$OBJPEER" ]; then OBJPEER="$ROOT/target/release/objpeer"; fi
WORK="$ROOT/local/fleet-demo.$$"
PID_A=
PID_B=
ADDR_A=
ADDR_B=

if [ ! -x "$OBJPEER" ]; then
  echo "building objpeer (release) ..." >&2
  (cd "$ROOT" && ~/.cargo/bin/cargo build --release -p uor-r4-object-store --bin objpeer) >&2 || {
    echo "FLEET_DEMO_FAIL: cargo build" >&2
    exit 1
  }
fi

cleanup() {
  if [ -n "$PID_A" ]; then kill "$PID_A" 2>/dev/null; wait "$PID_A" 2>/dev/null; fi
  if [ -n "$PID_B" ]; then kill "$PID_B" 2>/dev/null; wait "$PID_B" 2>/dev/null; fi
  rm -rf "$WORK"
}
trap cleanup EXIT
mkdir -p "$WORK"

start_replica() {  # STORE LOG OBJ...
  local store=$1 log=$2 pidvar=$3
  shift 3
  local args=
  for obj in "$@"; do args="$args --object $obj"; done
  # shellcheck disable=SC2086
  "$OBJPEER" serve --store "$store" $args --bind 127.0.0.1:0 --sessions 8 > "$log" 2>&1 &
  local pid=$!
  local i=0
  local addr=
  while [ "$i" -lt 600 ]; do
    addr=$(awk '/^READY / { print $2; exit }' "$log")
    [ -n "$addr" ] && { eval "$pidvar=$pid"; ADDR_REPLY=$addr; return 0; }
    sleep 0.1
    i=$((i + 1))
  done
  echo "replica did not become ready ($log)" >&2
  cat "$log" >&2
  return 1
}
stop_replica() {  # PIDVAR
  eval "local pid=\$$1"
  if [ -n "$pid" ]; then kill "$pid" 2>/dev/null; wait "$pid" 2>/dev/null; fi
  eval "$1="
}
sum_sha() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    sha256sum "$1" | awk '{ print $1 }'
  fi
}
json_assert() {  # FILE PYTHON-EXPR
  python3 -c "import json,sys; d=json.load(open(sys.argv[1])); assert $2, d" "$1" || {
    echo "FLEET_DEMO_FAIL: assertion '$2' on $1" >&2
    cat "$1" >&2
    exit 1
  }
}

# 1. Real local artifacts: two per replica, one overlapping.
for n in 1 2 3 4; do
  yes "fleet maintenance artifact $n: stable local payload" | head -c 65536 > "$WORK/object-$n.bin"
done
SHA1=$(sum_sha "$WORK/object-1.bin")
SHA2=$(sum_sha "$WORK/object-2.bin")
SHA3=$(sum_sha "$WORK/object-3.bin")
SHA4=$(sum_sha "$WORK/object-4.bin")
printf '%s\n%s\n%s\n%s\n' "$SHA1" "$SHA2" "$SHA3" "$SHA4" > "$WORK/targets.txt"
printf '%s\n%s\n%s\n' "$SHA2" "$SHA3" "$SHA4" > "$WORK/targets-b.txt"
echo "FLEET_DEMO artifacts=4 storage=loopback"

# 2. Two replicas: A has 1,2; B has 2,3,4.
mkdir -p "$WORK/replica-a" "$WORK/replica-b"
start_replica "$WORK/replica-a/store" "$WORK/replica-a.log" PID_A "$WORK/object-1.bin" "$WORK/object-2.bin" || exit 1
ADDR_A=$ADDR_REPLY
start_replica "$WORK/replica-b/store" "$WORK/replica-b.log" PID_B "$WORK/object-2.bin" "$WORK/object-3.bin" "$WORK/object-4.bin" || exit 1
ADDR_B=$ADDR_REPLY
echo "FLEET_DEMO replicas=2 a_has=1,2 b_has=2,3,4"

# 3. Audit a fresh client against both replicas.
VERIFY_RC=0
"$OBJPEER" verify --store "$WORK/client/store" --targets "$WORK/targets.txt" \
  --addr "$ADDR_A" --addr "$ADDR_B" --json --retries 1 > "$WORK/audit-before.json" 2> "$WORK/audit-before.err" || VERIFY_RC=$?
if [ "$VERIFY_RC" != 0 ] && [ "$VERIFY_RC" != 2 ] && [ "$VERIFY_RC" != 3 ]; then
  echo "FLEET_DEMO_FAIL: initial audit rc=$VERIFY_RC" >&2; cat "$WORK/audit-before.err" >&2; exit 1;
fi
cat "$WORK/audit-before.json"
json_assert "$WORK/audit-before.json" 'd["checks"] == 4 and d["verified_all"] == 0 and d["stale"] == 4'
echo "FLEET_DEMO audit_before checks=4 verified=0 stale=4"

# 4. Failover: replica A goes down; sync must use B for the three targets it serves.
stop_replica PID_A
"$OBJPEER" sync --store "$WORK/client/store" --targets "$WORK/targets-b.txt" \
  --addr 127.0.0.1:1 --addr "$ADDR_B" --out-dir "$WORK/sync-out" --json --retries 1 \
  > "$WORK/sync.json" 2> "$WORK/sync.err" || {
  echo "FLEET_DEMO_FAIL: failover sync" >&2; cat "$WORK/sync.err" >&2; exit 1;
}
grep -q "unavailable for sync" "$WORK/sync.err" || {
  echo "FLEET_DEMO_FAIL: sync did not report the dead replica" >&2; cat "$WORK/sync.err" >&2; exit 1;
}
cat "$WORK/sync.json"
json_assert "$WORK/sync.json" 'd["artifacts"] == 3'
for n in 2 3 4; do
  [ -f "$WORK/sync-out/$(eval "echo \$SHA$n").bin" ] || {
    echo "FLEET_DEMO_FAIL: object $n not synced from the surviving replica" >&2; exit 1; }
done
echo "FLEET_DEMO failover_sync artifacts=3 skipped_object=1"

# 5. Replica A returns; repair the one missing target from both replicas.
start_replica "$WORK/replica-a/store" "$WORK/replica-a.log" PID_A "$WORK/object-1.bin" "$WORK/object-2.bin" || exit 1
ADDR_A=$ADDR_REPLY
"$OBJPEER" repair --store "$WORK/client/store" --targets "$WORK/targets.txt" \
  --addr "$ADDR_A" --addr "$ADDR_B" --json > "$WORK/repair.json" 2> "$WORK/repair.err" || {
  echo "FLEET_DEMO_FAIL: repair" >&2; cat "$WORK/repair.err" >&2; exit 1;
}
cat "$WORK/repair.json"
json_assert "$WORK/repair.json" 'd["checks"] == 4 and d["current"] == 3 and d["repaired"] == 1 and d["unavailable"] == 0 and d["failed"] == 0'
echo "FLEET_DEMO repair current=3 repaired=1 unavailable=0 failed=0"

# 6. Final audit: every target is verified on at least one live replica.
VERIFY_RC=0
"$OBJPEER" verify --store "$WORK/client/store" --targets "$WORK/targets.txt" \
  --addr "$ADDR_A" --addr "$ADDR_B" --json --retries 1 > "$WORK/audit-after.json" 2> "$WORK/audit-after.err" || VERIFY_RC=$?
if [ "$VERIFY_RC" != 0 ] && [ "$VERIFY_RC" != 2 ] && [ "$VERIFY_RC" != 3 ]; then
  echo "FLEET_DEMO_FAIL: final audit rc=$VERIFY_RC" >&2; cat "$WORK/audit-after.err" >&2; exit 1;
fi
cat "$WORK/audit-after.json"
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); covered=all(any(r["status"]=="verified" for r in s["replicas"]) for s in d["results"]); assert d["checks"]==4 and covered and d["mismatch"]==0 and d["stale"]==0, d' "$WORK/audit-after.json"
echo "FLEET_DEMO audit_after checks=4 verified_on_some_replica=4 mismatch=0"
echo "FLEET_DEMO_RESULT audit_before=stale*4 failover_artifacts=3 repair_repaired=1 final_verified=4"
