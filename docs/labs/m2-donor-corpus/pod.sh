#!/usr/bin/env bash
# pod.sh STEP: the on-pod recipe for claude-400 (D87), run inside the vLLM image on the
# leased Vast pod (4xH200, 8xH200 or 4xB200; sheet amendments 2-4). Steps: setup | serve | unserve | wait-ready | smoke | run | status.
# The engine (vLLM) runs here, outside every compiled module (CHARTER 3.4); the client
# is the repository's Rust instrument, built here from the copied source.
set -euo pipefail
W=${M2_POD_W:-/workspace/m2}
MODEL_REPO=deepseek-ai/DeepSeek-V4.1-Flash-1017
MODEL_REV=08c8657f02291e11b6f6b63e0e16014f8e748518
MODEL_DIR=$W/donor
PORT=8000
SERVER=http://127.0.0.1:$PORT
BIN=$W/instrument/target/release/donor-corpus
mkdir -p "$W"

step=${1:?usage: pod.sh setup|serve|unserve|wait-ready|smoke|run|status}
shift
case $step in
setup)
  # 1. Client: Rust toolchain and the instrument copied to $W/instrument by the laptop.
  if ! [ -x "$HOME/.cargo/bin/cargo" ]; then
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal >/dev/null
  fi
  (cd "$W/instrument" && CARGO_INCREMENTAL=0 "$HOME/.cargo/bin/cargo" build --release 2>&1 | tail -2)
  # 2. Donor weights at the pinned revision, then every LFS file against HF's sha256.
  # The vLLM image ships no jq (16 Nov, pod 3: setup stopped at the LFS list).
  command -v jq > /dev/null || { apt-get update -qq > /dev/null && apt-get install -y -qq jq > /dev/null; }
  pip install -q 'huggingface_hub[hf_xet]' >/dev/null 2>&1 || true
  HF_XET_HIGH_PERFORMANCE=1 hf download "$MODEL_REPO" --revision "$MODEL_REV" --local-dir "$MODEL_DIR" >/dev/null
  curl -sf "https://huggingface.co/api/models/$MODEL_REPO/tree/$MODEL_REV?recursive=true" |
    jq -r '.[] | select(.lfs) | "\(.lfs.oid)  \(.path)"' > "$W/donor-lfs.sha256"
  # 16 parallel checkers; every line that is not "OK" is printed, and any one fails setup.
  rm -f "$W"/lfs-part.*
  split -n l/16 "$W/donor-lfs.sha256" "$W/lfs-part."
  (cd "$MODEL_DIR" && for p in "$W"/lfs-part.??; do sha256sum -c "$p" > "$p.out" 2>&1 & done; wait) || true
  if grep -h -v ': OK$' "$W"/lfs-part.*.out; then echo "donor LFS sha256 check FAILED (lines above)"; exit 1; fi
  # An empty or short list must never read as a pass: at least the 48 weight shards, every line OK.
  n_list=$(wc -l < "$W/donor-lfs.sha256"); n_ok=$(cat "$W"/lfs-part.*.out | grep -c ': OK$' || true)
  [ "$n_list" -ge 48 ] && [ "$n_ok" -eq "$n_list" ] || { echo "donor LFS sha256 check FAILED: $n_ok OK of $n_list listed"; exit 1; }
  echo "donor LFS sha256: all $(cat "$W"/lfs-part.*.out | grep -c ': OK$') of $(wc -l < "$W/donor-lfs.sha256") OK"
  nvidia-smi --query-gpu=index,name,uuid,driver_version,memory.total --format=csv,noheader > "$W/gpus.csv"
  nvidia-smi topo -m > "$W/gpu-topo.txt" 2>&1 || true   # NV# between all four GPUs = the NVLink bridge TP4 needs
  free -g | head -2 > "$W/host-mem.txt"; df -h /workspace | tail -1 >> "$W/host-mem.txt"
  vllm --version > "$W/vllm-version.txt" 2>&1
  echo "setup done"
  ;;
serve)
  # One endpoint, one replica per TP GPUs (the preset below),
  # no speculative decoding (the recipe marks it for small batches).
  # serve [h200|b200|b200-tp4]: the recipe preset, named on the command line so nothing needs
  # quoting through ssh (sheet amendment 3). h200: TP4, 256 seqs per replica (the H200 recipe).
  # b200: vLLM's Blackwell TP recipe, TP2 with its attention config and 128 seqs per replica.
  # b200-tp4: amendment 3's one fallback restart, TP4 with the same attention config and 256 seqs
  # (the H200 value, chosen deliberately for one replica). DP is the GPU count over TP.
  B200_ATTN='{"backend":"FLASHINFER_MLA_SPARSE_DSV41","indexer_kv_dtype":"mxfp4","indexer_sparse_logits":true}'
  case ${1:-h200} in
    h200) TP=4; MAX_SEQS=${MAX_SEQS:-256}; ATTN=() ;;
    b200) TP=2; MAX_SEQS=128; ATTN=(--attention-config "$B200_ATTN") ;;
    b200-tp4) TP=4; MAX_SEQS=256; ATTN=(--attention-config "$B200_ATTN") ;;
    *) echo "serve: unknown preset $1"; exit 1 ;;
  esac
  # serve PRESET SEQS (amendment 4): the sequences per replica for a tuning segment, a plain
  # positive integer, so that nothing is quoted through ssh.
  if [ -n "${2:-}" ]; then
    case $2 in *[!0-9]*|0*) echo "serve: sequences per replica must be a positive integer, got $2"; exit 1 ;; esac
    MAX_SEQS=$2
  fi
  NG=$(nvidia-smi -L | wc -l); DP=$((NG / TP))
  [ "$DP" -ge 1 ] && [ $((DP * TP)) -eq "$NG" ] || { echo "serve: $NG GPUs is not a multiple of $TP"; exit 1; }
  echo "serve: $NG GPUs, data-parallel $DP x tensor-parallel $TP, max-num-seqs $MAX_SEQS${ATTN[1]:+, attention ${ATTN[1]}}"
  nohup env VLLM_ENGINE_READY_TIMEOUT_S=7200 vllm serve "$MODEL_DIR" \
    --served-model-name donor --port $PORT \
    --tokenizer-mode deepseek_v41 \
    --data-parallel-size "$DP" --tensor-parallel-size "$TP" \
    ${ATTN[@]+"${ATTN[@]}"} \
    --kv-cache-dtype fp8 \
    --engram-config '{"cpu_offload":true}' \
    --reasoning-parser deepseek_v41 \
    --language-model-only \
    --max-num-seqs "$MAX_SEQS" --max-num-batched-tokens 8192 \
    --gpu-memory-utilization 0.92 \
    > "$W/vllm.log" 2>&1 < /dev/null &
  echo $! > "$W/vllm.pid"
  echo "vllm pid $(cat "$W/vllm.pid"), log $W/vllm.log"
  ;;
unserve)
  # Stop this pod's vLLM for a serve restart (amendment 4 tuning). serve starts vLLM from a fresh
  # ssh session, so the server and all of its workers share one process group, which holds nothing
  # else: signal that group, never a pid read from nvidia-smi (in a container it can be a host pid).
  pid=$(cat "$W/vllm.pid")
  pgid=$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ' || true)
  if [ -n "$pgid" ]; then
    [ "$pgid" != "$(ps -o pgid= -p $$ | tr -d ' ')" ] || { echo "unserve: vllm shares this shell's process group: refusing"; exit 1; }
    for sig in TERM KILL; do
      echo "unserve: SIG$sig to vllm's process group $pgid ($(ps -e -o pgid= | awk -v g="$pgid" '$1 == g' | wc -l | tr -d ' ') processes)"
      kill -"$sig" -- "-$pgid" 2>/dev/null || true
      for _ in $(seq 1 36); do [ -z "$(ps -e -o pgid= | awk -v g="$pgid" '$1 == g')" ] && break; sleep 5; done
      [ -z "$(ps -e -o pgid= | awk -v g="$pgid" '$1 == g')" ] && break
    done
  fi
  sleep 10
  used=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | sort -n | tail -1)
  [ "$used" -lt 2048 ] || { echo "unserve: a GPU still holds $used MiB"; exit 1; }
  echo "unserve: vllm stopped, every GPU under 2 GiB"
  ;;
wait-ready)
  for _ in $(seq 1 720); do
    if curl -sf "$SERVER/v1/models" >/dev/null; then echo ready; exit 0; fi
    kill -0 "$(cat "$W/vllm.pid")" 2>/dev/null || { echo "vllm exited"; tail -30 "$W/vllm.log"; exit 1; }
    sleep 10
  done
  echo "not ready after 2 h"; exit 1
  ;;
smoke)
  COMMIT=${1:?smoke needs the instrument commit}
  for f in model machine engine; do [ -s "$W/$f.json" ] || { echo "missing $W/$f.json"; exit 1; }; done
  head -32 "$W/bank.jsonl" > "$W/smoke-bank.jsonl"
  rm -rf "$W/smoke"
  "$BIN" generate --server "$SERVER" --prompts "$W/smoke-bank.jsonl" --out "$W/smoke" \
    --budget 100000000 --parallel 8 --limit 32 --instrument-commit "$COMMIT" ${EXTRA:+--extra-body "$EXTRA"}
  "$BIN" seal --dir "$W/smoke" --prompts "$W/smoke-bank.jsonl" --model "$W/model.json" --machine "$W/machine.json" --engine "$W/engine.json"
  "$BIN" verify --dir "$W/smoke" --prompts "$W/smoke-bank.jsonl"
  # The sheet's sampling, as recorded (temperature 1 and top_p 1 are fixed in request_body).
  jq -e 'select(.effort == 50 and .max_tokens == 16384 and .base_seed == 20261116 and .model_name == "donor")' \
    "$W/smoke/segments.jsonl" > /dev/null || { echo "smoke: recorded sampling differs from the sheet"; exit 1; }
  jq -c '{records: .records, kept_tokens, empty_reasoning_turns, dropped_by_reason}' "$W/smoke/metrics.json"
  echo "smoke OK"
  ;;
run)
  # run COMMIT PARALLEL MAX_SECONDS: one segment of the corpus run, detached; resumes.
  # run COMMIT PARALLEL MAX_SECONDS PART (amendment 5): PART 0-2 is a contiguous range of bank lines,
  # FIRST..LAST below, so no prompt is generated on two pods. Ranges are in proportion to the parts'
  # budgets (29% / 47% / 24% of 1B: pod 3's 4 GPUs plus its head start, the 8xB200, a 4xB200). The
  # base seed is 20261116 + 2*(FIRST-1), so turn_seed(base, i, t) = base + 2i + t gives each prompt
  # the seed of its index in the full bank. Without PART: the full bank and 1B.
  COMMIT=${1:?}; PAR=${2:?}; SECS=${3:?}; PART=${4:-}
  BANK=$W/bank.jsonl; SEED=20261116; BUDGET=1000000000
  if [ -n "$PART" ]; then
    case $PART in
      0) FIRST=1; LAST=200786; BUDGET=290000000 ;;
      1) FIRST=200787; LAST=526197; BUDGET=470000000 ;;
      2) FIRST=526198; LAST=692364; BUDGET=240000000 ;;
      *) echo "run: PART must be 0, 1 or 2, got $PART"; exit 1 ;;
    esac
    [ "$(wc -l < "$W/bank.jsonl")" -eq 692364 ] || { echo "run: the bank is not 692364 lines"; exit 1; }
    BANK=$W/bank-part-$PART.jsonl; SEED=$((20261116 + 2 * (FIRST - 1)))
    sed -n "${FIRST},${LAST}p" "$W/bank.jsonl" > "$BANK.tmp" && mv "$BANK.tmp" "$BANK"
    [ "$(wc -l < "$BANK")" -eq $((LAST - FIRST + 1)) ] || { echo "run: $BANK is not $((LAST - FIRST + 1)) lines"; exit 1; }
  fi
  mkdir -p "$W/out"
  # A restore from the laptop is incomplete: the guard does not sync during one, so refuse to run.
  [ ! -e "$W/out/.restoring" ] || { echo "run: $W/out/.restoring exists (restore incomplete): run drive.sh put again"; exit 1; }
  nohup "$BIN" generate --server "$SERVER" --prompts "$BANK" --out "$W/out" --seed "$SEED" \
    --budget "$BUDGET" --parallel "$PAR" --max-seconds "$SECS" --instrument-commit "$COMMIT" \
    ${EXTRA:+--extra-body "$EXTRA"} > "$W/run-$(date -u +%Y%m%dT%H%M%SZ).log" 2>&1 < /dev/null &
  echo $! > "$W/run.pid"
  echo "run pid $(cat "$W/run.pid")"
  ;;
status)
  echo "run: $(kill -0 "$(cat "$W/run.pid" 2>/dev/null)" 2>/dev/null && echo alive || echo stopped)"
  echo "vllm: $(kill -0 "$(cat "$W/vllm.pid" 2>/dev/null)" 2>/dev/null && echo alive || echo stopped)"
  tail -3 "$W/out/rate.jsonl" 2>/dev/null || true
  echo "finished shards: $(find "$W/out" -maxdepth 1 -name 'shard-*.jsonl' 2>/dev/null | wc -l)"
  nvidia-smi --query-gpu=index,utilization.gpu,memory.used --format=csv,noheader
  ;;
*) echo "unknown step $step" >&2; exit 1;;
esac
