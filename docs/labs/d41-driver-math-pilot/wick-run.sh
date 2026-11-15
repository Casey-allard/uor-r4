#!/usr/bin/env bash
# d41 donor pilot (claude-390), wick steps 2-6 of the sheet
# (docs/harness/prep/d41-driver-math-pilot-2026-11-15.md): convert, prompt set, smoke, run, seal, verify.
# Runs inside Claude's wick container after wick-setup.sh, with the instrument source,
# data/panels and the laptop's prompts.jsonl put under /workspace/d41/in.
# The donor runs only in llama-server, outside the repository (CHARTER 3.4).
set -euo pipefail
W=/workspace/d41
IN=$W/in
OUT=$W/out
META=$W/meta
SEED=20261115
PROMPTS_SHA=6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0
LLAMA_TAG=b12000
LLAMA_SHA=68d20b884c3878217de79eade0a249efde96ab38
DONOR=openai/gpt-oss-20b
REV=6cee5e81ee83917806bbde320786a8fb61efebee
PORT=8086
SERVER_ARGS="-ngl 99 --flash-attn on -np 32 -c 131072 --jinja --host 127.0.0.1 --port $PORT"
export PATH=$HOME/.cargo/bin:$PATH
test ! -e "$OUT" || { echo "$OUT exists: one pass only" >&2; exit 5; }
mkdir -p "$META"
command -v jq >/dev/null || { apt-get install -y -qq jq >/dev/null; }

echo "== instrument"
(cd "$IN/instrument" && CARGO_TARGET_DIR=$W/target CARGO_INCREMENTAL=0 cargo build --release -q)
P=$W/target/release/d41-pilot

echo "== prompt set (must reproduce the laptop's)"
"$P" prompts --seed $SEED --count 6000 --panels "$IN/panels" --out "$W/prompts.jsonl" | tee "$META/prompts-build.json"
test "$(sha256sum "$W/prompts.jsonl" | cut -d' ' -f1)" = "$PROMPTS_SHA"
cmp "$W/prompts.jsonl" "$IN/prompts.jsonl"

echo "== convert (MXFP4 experts repacked as MXFP4, the rest bf16 as released)"
[ -s "$W/gpt-oss-20b.gguf" ] || "$W/venv/bin/python" "$W/engine/llama.cpp/convert_hf_to_gguf.py" "$W/donor-hf" \
  --outtype bf16 --outfile "$W/gpt-oss-20b.gguf" 2>&1 | grep -E "MXFP4|not in MXFP4|Model successfully|error" | sort | uniq -c | tail -8 || true
test -s "$W/gpt-oss-20b.gguf"
GGUF_SHA=$(sha256sum "$W/gpt-oss-20b.gguf" | cut -d' ' -f1)
GGUF_BYTES=$(stat -c %s "$W/gpt-oss-20b.gguf")
LS=$W/engine/llama.cpp/build/bin/llama-server

echo "== identity"
jq -n --arg repo $DONOR --arg rev $REV --arg sha "$GGUF_SHA" --argjson bytes "$GGUF_BYTES" \
  --rawfile src "$W/donor-hf.sha256" --arg conv "convert_hf_to_gguf.py at $LLAMA_SHA, --outtype bf16" \
  '{repo:$repo, revision:$rev, license:"apache-2.0",
    quantization:"MXFP4 expert tensors as released, repacked to ggml MXFP4 by the converter without requantization; every other tensor bf16 as released",
    source_files_sha256:($src|split("\n")|map(select(length>0))), converter:$conv,
    gguf:{file:"gpt-oss-20b.gguf", sha256:$sha, bytes:$bytes}}' > "$META/model.json"
jq -n --arg tag $LLAMA_TAG --arg sha $LLAMA_SHA --arg ver "$("$LS" --version 2>&1 | tail -2 | tr '\n' ' ')" \
  --arg nvcc "$(nvcc --version | tail -1)" --arg args "$SERVER_ARGS" \
  '{engine:"llama.cpp llama-server (external, outside the repository)", tag:$tag, commit:$sha, version:$ver,
    build:"cmake -DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES=89 -DLLAMA_CURL=OFF -DCMAKE_BUILD_TYPE=Release", nvcc:$nvcc,
    server_args:$args}' > "$META/engine.json"
jq -n --arg gpu "$(nvidia-smi --query-gpu=name,uuid,driver_version,memory.total --format=csv,noheader)" \
  --arg image "nvidia/cuda:12.8.1-devel-ubuntu24.04" \
  '{host:"wick (the owner'"'"'s own server)", container:"uor-r4-claude", image:$image, gpu:$gpu}' > "$META/machine.json"
cat "$META"/*.json

echo "== server"
nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader
# shellcheck disable=SC2086
"$LS" -m "$W/gpt-oss-20b.gguf" $SERVER_ARGS > "$W/server.log" 2>&1 &
SPID=$!
( while kill -0 $SPID 2>/dev/null; do echo "$(date -u +%FT%TZ) $(nvidia-smi --query-gpu=memory.used,memory.total,utilization.gpu --format=csv,noheader)"; sleep 30; done ) > "$W/gpu.log" &
GPID=$!
trap 'kill $SPID $GPID 2>/dev/null || true' EXIT
for _ in $(seq 1 120); do curl -sf "http://127.0.0.1:$PORT/health" >/dev/null && break; sleep 5; done
curl -sf "http://127.0.0.1:$PORT/health"; echo

echo "== smoke (instrument check, not corpus)"
head -4 "$W/prompts.jsonl" > "$W/smoke-prompts.jsonl"
"$P" generate --server "http://127.0.0.1:$PORT" --prompts "$W/smoke-prompts.jsonl" --out "$W/smoke" \
  --budget 1000000 --parallel 1 --seed $SEED --max-tokens 1024 > /dev/null
cat "$W/smoke/shard-0000.jsonl"
jq -e '.markup_answers == 0 and .records_kept >= 3 and .errors == 0' "$W/smoke/metrics.json"

echo "== run"
"$P" generate --server "http://127.0.0.1:$PORT" --prompts "$W/prompts.jsonl" --out "$OUT" \
  --budget 1000000 --parallel 32 --seed $SEED --max-tokens 1024 --shard-records 512 --max-seconds 10800 > /dev/null
cat "$OUT/metrics.json"

echo "== seal and verify"
cp "$W/prompts.jsonl" "$OUT/prompts.jsonl"
"$P" seal --dir "$OUT" --model "$META/model.json" --engine "$META/engine.json" --machine "$META/machine.json"
"$P" verify --dir "$OUT" | tee "$META/verify-wick.json"
cp "$W/gpu.log" "$W/server.log" "$META/" 2>/dev/null || true
sha256sum "$OUT"/* | tee "$META/out.sha256"
echo run-done
