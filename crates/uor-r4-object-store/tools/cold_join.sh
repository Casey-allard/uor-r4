#!/usr/bin/env bash
# Cold-join end-to-end benchmark (host only).
#
# Compares two documented ways to get one artifact onto a fresh node:
#   1. cold store pull: objpeer pull over TCP (wire bytes and wall time)
#   2. cold join: gzip download + local decompress + pinned objpeer join, then a warm delta
#
# Sizes and seconds are measured on this host; the output is one block of COLD_JOIN lines.
# No pod, no model compute. Override SIZE_MB (default 2) and DATA (text|random).
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
CRATE=$(cd "$SCRIPT_DIR/.." && pwd)
ROOT=$(cd "$CRATE/../.." && pwd)
OBJPEER_DEFAULT="$ROOT/target/release/objpeer"
if [ -z "$OBJPEER" ]; then OBJPEER="$OBJPEER_DEFAULT"; fi
if [ -z "$SIZE_MB" ]; then SIZE_MB=2; fi
if [ -z "$DATA" ]; then DATA=text; fi
WORK="$ROOT/local/cold-join.$$"
SERVER_PID=
ADDR=

if [ ! -x "$OBJPEER" ]; then
  echo "building objpeer (release) ..." >&2
  (cd "$ROOT" && ~/.cargo/bin/cargo build --release -p uor-r4-object-store --bin objpeer) >&2 || {
    echo "COLD_JOIN_FAIL: cargo build" >&2
    exit 1
  }
fi

cleanup() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null
    wait "$SERVER_PID" 2>/dev/null
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT
mkdir -p "$WORK"

now_seconds() { python3 -c 'import time; print(time.time())'; }
elapsed_seconds() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.3f", b - a }'; }

start_server() {  # STORE OBJECT LOG
  "$OBJPEER" serve --store "$1" --object "$2" --bind 127.0.0.1:0 > "$3" 2>&1 &
  SERVER_PID=$!
  ADDR=
  local i=0
  while [ "$i" -lt 600 ]; do
    ADDR=$(awk '/^READY / { print $2; exit }' "$3")
    [ -n "$ADDR" ] && return 0
    sleep 0.1
    i=$((i + 1))
  done
  echo "server did not become ready" >&2
  cat "$3" >&2
  return 1
}
stop_server() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null
    wait "$SERVER_PID" 2>/dev/null
    SERVER_PID=
  fi
}
pull_field() {  # FIELD LOG
  awk -v key="$1" '{
    for (i = 1; i <= NF; i++) {
      split($i, kv, "=")
      if (kv[1] == key) { print kv[2]; exit }
    }
  }' "$2"
}

# 1. Generate the artifact.
ART="$WORK/artifact.bin"
if [ "$DATA" = "random" ]; then
  dd if=/dev/urandom of="$ART" bs=1048576 count="$SIZE_MB" 2>/dev/null
else
  yes 'cold join benchmark payload: the same line repeated to stay compressible' |
    head -c $((SIZE_MB * 1048576)) > "$ART"
fi
BYTES=$(wc -c < "$ART" | tr -d ' ')
if command -v shasum >/dev/null 2>&1; then
  SHA=$(shasum -a 256 "$ART" | awk '{ print $1 }')
else
  SHA=$(sha256sum "$ART" | awk '{ print $1 }')
fi
echo "COLD_JOIN artifact_bytes=$BYTES data=$DATA sha256=$SHA"

# 2. Cold store pull path.
mkdir -p "$WORK/cold-server" "$WORK/cold-client"
start_server "$WORK/cold-server/store" "$ART" "$WORK/cold-server.log" || exit 1
START=$(now_seconds)
if ! "$OBJPEER" pull --store "$WORK/cold-client/store" --addr "$ADDR" \
    --select-sha256 "$SHA" --out "$WORK/pulled.bin" > "$WORK/cold-pull.log" 2>&1; then
  echo "COLD_JOIN_FAIL: cold pull" >&2
  cat "$WORK/cold-pull.log" >&2
  exit 1
fi
END=$(now_seconds)
stop_server
COLD_SECS=$(elapsed_seconds "$START" "$END")
COLD_WIRE=$(pull_field wire_bytes "$WORK/cold-pull.log")
COLD_NODES=$(pull_field nodes "$WORK/cold-pull.log")
COLD_ROUNDS=$(pull_field rounds "$WORK/cold-pull.log")
cmp -s "$ART" "$WORK/pulled.bin" || { echo "COLD_JOIN_FAIL: cold pull bytes differ" >&2; exit 1; }
echo "COLD_JOIN cold_pull wire_bytes=$COLD_WIRE nodes=$COLD_NODES rounds=$COLD_ROUNDS seconds=$COLD_SECS"

# 3. Compressed download + local decompress + pinned join.
GZ_BYTES=0
if command -v gzip >/dev/null 2>&1; then
  START=$(now_seconds)
  gzip -9 -c "$ART" > "$WORK/artifact.bin.gz"
  END=$(now_seconds)
  GZ_SECS=$(elapsed_seconds "$START" "$END")
  GZ_BYTES=$(wc -c < "$WORK/artifact.bin.gz" | tr -d ' ')
  START=$(now_seconds)
  gzip -dc "$WORK/artifact.bin.gz" > "$WORK/decompressed.bin"
  END=$(now_seconds)
  DECOMP_SECS=$(elapsed_seconds "$START" "$END")
  echo "COLD_JOIN gzip bytes=$GZ_BYTES ratio=$(awk -v g="$GZ_BYTES" -v b="$BYTES" 'BEGIN { printf "%.5f", g / b }') gzip_seconds=$GZ_SECS decompress_seconds=$DECOMP_SECS"
else
  echo "COLD_JOIN gzip missing; skipping compressed leg" >&2
fi

mkdir -p "$WORK/joined"
START=$(now_seconds)
if ! "$OBJPEER" join --store "$WORK/joined/store" --object "$WORK/decompressed.bin" \
    --expect-sha256 "$SHA" --expect-bytes "$BYTES" > "$WORK/join.log" 2>&1; then
  echo "COLD_JOIN_FAIL: join" >&2
  cat "$WORK/join.log" >&2
  exit 1
fi
END=$(now_seconds)
JOIN_SECS=$(elapsed_seconds "$START" "$END")
JOIN_MILLIS=$(pull_field millis "$WORK/join.log")
JOIN_RECORDS=$(pull_field records "$WORK/join.log")
JOIN_STORE_KB=$(du -sk "$WORK/joined/store" | awk '{ print $1 }')
echo "COLD_JOIN join records=$JOIN_RECORDS millis=$JOIN_MILLIS seconds=$JOIN_SECS store_bytes=$((JOIN_STORE_KB * 1024))"

# 4. Warm delta from the joined store: a client that already has the artifact must fetch nothing.
mkdir -p "$WORK/warm-client"
cp "$WORK/joined/store" "$WORK/warm-client/store"
start_server "$WORK/joined/store" "$WORK/decompressed.bin" "$WORK/warm-server.log" || exit 1
START=$(now_seconds)
if ! "$OBJPEER" pull --store "$WORK/warm-client/store" --addr "$ADDR" \
    --select-sha256 "$SHA" --out "$WORK/warm-pulled.bin" > "$WORK/warm-pull.log" 2>&1; then
  echo "COLD_JOIN_FAIL: warm pull" >&2
  cat "$WORK/warm-pull.log" >&2
  exit 1
fi
END=$(now_seconds)
stop_server
WARM_SECS=$(elapsed_seconds "$START" "$END")
WARM_WIRE=$(pull_field wire_bytes "$WORK/warm-pull.log")
WARM_NODES=$(pull_field nodes "$WORK/warm-pull.log")
echo "COLD_JOIN warm_delta wire_bytes=$WARM_WIRE nodes=$WARM_NODES seconds=$WARM_SECS"

# 5. Checks and summary.
[ "$COLD_NODES" != "0" ] || { echo "COLD_JOIN_FAIL: cold pull fetched no nodes" >&2; exit 1; }
[ "$WARM_NODES" = "0" ] || { echo "COLD_JOIN_FAIL: warm delta was not zero (nodes=$WARM_NODES)" >&2; exit 1; }
[ "$JOIN_RECORDS" != "0" ] || { echo "COLD_JOIN_FAIL: join stored no records" >&2; exit 1; }
if [ "$GZ_BYTES" != "0" ]; then
  [ "$GZ_BYTES" -lt "$COLD_WIRE" ] || { echo "COLD_JOIN_FAIL: compressed download is not smaller than cold pull wire" >&2; exit 1; }
fi
cmp -s "$ART" "$WORK/warm-pulled.bin" || { echo "COLD_JOIN_FAIL: warm pull bytes differ" >&2; exit 1; }
echo "COLD_JOIN_RESULT cold_pull_wire_bytes=$COLD_WIRE compressed_bytes=$GZ_BYTES join_records=$JOIN_RECORDS warm_wire_bytes=$WARM_WIRE"
