#!/usr/bin/env bash
# d41 donor throughput probe (claude-398), docs/harness/prep/d41-donor-throughput-2026-11-15.md.
# Runs inside Claude's wick container after the pilot (claude-390): it reuses the pilot's
# llama.cpp b12000 build, its GGUF and its fixed prompt set. The donor runs only in
# llama-server, outside the repository (CHARTER 3.4). Nothing here is corpus.
#
#   wick-throughput.sh ARMS_FILE OUT_NAME INSTRUMENT_COMMIT
#
# ARMS_FILE holds one arm per line: NAME|SERVER_ARGS|CLIENT_ARGS. Consecutive arms with the
# same SERVER_ARGS share one llama-server. An arm named smoke-* runs on 4 prompts with
# --parallel 1 and must pass the smoke check; every other arm is a timed window.
set -euo pipefail
ARMS=$1
NAME=$2
COMMIT=$3
W=/workspace/d41
IN=$W/in398
OUT=$W/tp/$NAME
SEED=20261115
PROMPTS=$W/prompts.jsonl
PROMPTS_SHA=6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0
WINDOW=${WINDOW:-300}
PORT=8087
LS=$W/engine/llama.cpp/build/bin/llama-server
BASE="-m $W/gpt-oss-20b.gguf -ngl 99 --flash-attn on --jinja --host 127.0.0.1 --port $PORT --metrics"
export PATH=$HOME/.cargo/bin:$PATH
test ! -e "$OUT" || { echo "$OUT exists: one pass per name" >&2; exit 5; }
mkdir -p "$OUT"
cp "$ARMS" "$OUT/arms.txt"

echo "== instrument at $COMMIT"
(cd "$IN/instrument" && CARGO_TARGET_DIR=$W/target398 CARGO_INCREMENTAL=0 cargo build --release -q)
P=$W/target398/release/d41-pilot
test "$(sha256sum "$PROMPTS" | cut -d' ' -f1)" = "$PROMPTS_SHA"
head -4 "$PROMPTS" > "$OUT/smoke-prompts.jsonl"
nvidia-smi --query-gpu=name,uuid,driver_version,memory.used,memory.total --format=csv,noheader | tee "$OUT/gpu-before.txt"

SPID=""
GPID=""
CUR=""
stop_server() {
  if [ -n "$GPID" ]; then kill "$GPID" 2>/dev/null || true; wait "$GPID" 2>/dev/null || true; GPID=""; fi
  if [ -n "$SPID" ]; then kill "$SPID" 2>/dev/null || true; wait "$SPID" 2>/dev/null || true; SPID=""; fi
  CUR=""
}
trap stop_server EXIT

start_server() {  # $1 = server args, $2 = log name
  stop_server
  # shellcheck disable=SC2086
  SLOG=$OUT/server-$2.log
  "$LS" $BASE $1 < /dev/null > "$SLOG" 2>&1 &
  SPID=$!
  for _ in $(seq 1 120); do
    curl -sf "http://127.0.0.1:$PORT/health" >/dev/null && break
    kill -0 "$SPID" 2>/dev/null || { echo "server died: $1" >&2; tail -20 "$OUT/server-$2.log" >&2; return 1; }
    sleep 3
  done
  curl -sf "http://127.0.0.1:$PORT/health" >/dev/null || return 1
  CUR=$1
}

# One line every 5 s: GPU memory used MiB, GPU utilization %, busy slots (llama-server /metrics).
gpu_watch() {  # $1 = file
  ( while true; do
      G=$(nvidia-smi --query-gpu=memory.used,utilization.gpu --format=csv,noheader,nounits)
      B=$(curl -sf "http://127.0.0.1:$PORT/metrics" | awk '/^llamacpp:requests_processing/ {printf "%d", $2}')
      echo "$G, ${B:-0}"
      sleep 5
    done ) > "$1" &
  GPID=$!
}

while IFS='|' read -r -u 3 ARM SARGS CARGS; do
  case $ARM in ''|'#'*) continue;; esac
  echo "== arm $ARM | server: $SARGS | client: $CARGS"
  if [ "$SARGS" != "$CUR" ]; then
    start_server "$SARGS" "$ARM" || { echo "$ARM server-failed" >> "$OUT/failed.txt"; stop_server; continue; }
  fi
  D=$OUT/$ARM
  gpu_watch "$OUT/$ARM.gpu"
  rc=0
  case $ARM in
    smoke-*)
      # shellcheck disable=SC2086
      "$P" generate --server "http://127.0.0.1:$PORT" --prompts "$OUT/smoke-prompts.jsonl" --out "$D" \
        --budget 1000000 --parallel 1 --seed $SEED --max-tokens 1024 --instrument-commit "$COMMIT" $CARGS > /dev/null || rc=$?
      ;;
    *)
      # shellcheck disable=SC2086
      "$P" generate --server "http://127.0.0.1:$PORT" --prompts "$PROMPTS" --out "$D" \
        --budget 1000000000 --seed $SEED --max-tokens 1024 --shard-records 512 --max-seconds "$WINDOW" \
        --instrument-commit "$COMMIT" $CARGS > /dev/null || rc=$?
      ;;
  esac
  kill "$GPID" 2>/dev/null || true; wait "$GPID" 2>/dev/null || true; GPID=""
  if [ $rc -ne 0 ]; then echo "$ARM rc=$rc" >> "$OUT/failed.txt"; continue; fi
  if jq -e '.kept_answers_counted == false' "$D/metrics.json" >/dev/null 2>&1; then
    "$P" count --server "http://127.0.0.1:$PORT" --dir "$D" > /dev/null || echo "$ARM count-failed" >> "$OUT/failed.txt"
  fi
  case $SARGS in
    *--backend-sampling*)
      # The H-sampler activeness check: did the engine set a backend sampler for the slots?
      echo "$ARM $(grep -c 'setting backend sampler' "$SLOG" || true) backend-sampler-lines in $(basename "$SLOG")" >> "$OUT/backend-active.txt"
      ;;
  esac
  case $ARM in
    smoke-*)
      head -c 1500 "$D/shard-0000.jsonl" 2>/dev/null || true; echo
      jq -e '.markup_answers == 0 and .records_kept >= 3 and .errors == 0 and .answers_returned == 16' "$D/metrics.json" \
        || echo "$ARM smoke-check-failed" >> "$OUT/failed.txt"
      ;;
  esac
  jq -c '{wall_ms, window, prompts_dispatched, requests, answers_returned, generated_tokens, kept_answer_tokens,
          truncated_answers, markup_answers, errors, tokenize_errors, request_ms}' "$D/metrics.json" || true
done 3< "$ARMS"
stop_server

# The trimmed chain on the CPU and on the GPU must give different sampled texts if the GPU
# sampler ran (different random generators), and identical ones if it silently fell back.
if [ -d "$OUT/smoke-temp" ] && [ -d "$OUT/smoke-temp-bs" ]; then
  if cmp -s "$OUT/smoke-temp/shard-0000.jsonl" "$OUT/smoke-temp-bs/shard-0000.jsonl"; then
    echo "smoke-temp vs smoke-temp-bs: identical (backend sampler not active)" >> "$OUT/backend-active.txt"
  else
    echo "smoke-temp vs smoke-temp-bs: differ" >> "$OUT/backend-active.txt"
  fi
fi
# Evidence (not proof) that the trimmed CPU chain samples as the default chain does: the same
# prompts, seeds and values through the default chain in the earlier smoke job.
if [ -d "$OUT/smoke-temp" ] && [ -s "$W/tp/smoke/smoke-seq/shard-0000.jsonl" ]; then
  if cmp -s "$OUT/smoke-temp/shard-0000.jsonl" "$W/tp/smoke/smoke-seq/shard-0000.jsonl"; then
    echo "smoke-temp vs smoke/smoke-seq (default chain): identical" >> "$OUT/backend-active.txt"
  else
    echo "smoke-temp vs smoke/smoke-seq (default chain): differ" >> "$OUT/backend-active.txt"
  fi
fi

# The bar's rates use the window: requests and prompts that finished within WINDOW seconds.
# Rates in thousandths of a token per second; gpu = mean utilization % and peak MiB; slots = mean busy slots.
echo "== summary"
{
  printf 'arm\twall_ms\ttail_ms\tprompts_win\tgen_win\tkept_win\tgen_win_mtps\tkept_win_mtps\tkept_per_1000_gen\tanswers\ttruncated\tmarkup\terrors\ttok_err\tgpu_util\tgpu_mib\tslots\n'
  for D in "$OUT"/*/; do
    A=$(basename "$D")
    [ -s "$D/metrics.json" ] || continue
    KW=$(jq '.window.kept_answer_tokens' "$D/metrics.json" || echo 0)
    TE=$(jq '.tokenize_errors' "$D/metrics.json" || echo -1)
    if [ -s "$D/count.json" ]; then
      KW=$(jq '.window_kept_answer_tokens // 0' "$D/count.json" || echo 0)
      TE=$(jq '.tokenize_errors' "$D/count.json" || echo -1)
    fi
    # Utilization and busy slots average only the samples inside the dispatch window (one per 5 s),
    # so an arm's drain after the window does not lower them; peak memory reads every sample.
    GU=$(awk -F", " -v k=$((WINDOW / 5)) 'NR<=k {u+=$2; n++} END {if (n) printf "%d", u/n; else printf "-"}' "$OUT/$A.gpu")
    GM=$(awk -F", " '{if ($1>m) m=$1} END {printf "%d", m}' "$OUT/$A.gpu")
    BS=$(awk -F", " -v k=$((WINDOW / 5)) 'NR<=k {b+=$3; n++} END {if (n) printf "%d", b/n; else printf "-"}' "$OUT/$A.gpu")
    jq -r --argjson kw "$KW" --argjson te "$TE" --arg gu "$GU" --arg gm "$GM" --arg bs "$BS" --arg a "$A" \
      '[$a, .wall_ms, .window.tail_ms, .window.prompts_finished, .window.generated_tokens, $kw,
        (.window.generated_tokens * 1000000 / .window.ms | floor), ($kw * 1000000 / .window.ms | floor),
        ($kw * 1000 / ([.window.generated_tokens, 1] | max) | floor), .answers_returned,
        .truncated_answers, .markup_answers, .errors, $te, $gu, $gm, $bs] | @tsv' "$D/metrics.json" || true
  done
} | tee "$OUT/summary.tsv"
test ! -s "$OUT/backend-active.txt" || { echo "== backend sampler:"; cat "$OUT/backend-active.txt"; }
test ! -s "$OUT/failed.txt" || { echo "== failed:"; cat "$OUT/failed.txt"; }
echo probe-done
