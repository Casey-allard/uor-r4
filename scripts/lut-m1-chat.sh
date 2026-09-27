#!/usr/bin/env bash
# Integer chat with a converted SmolLM2 checkpoint on an Apple-silicon Mac (owner decision D10).
#
# Usage, from the repository root:
#   scripts/lut-m1-chat.sh MODEL_DIR CALIBRATION_TEXT EVAL_TEXT [OUT_PARENT]
#
# MODEL_DIR holds config.json, model.safetensors and tokenizer.json of a Llama-architecture checkpoint, for
# example SmolLM2-135M-Instruct at the revision registered in models/smollm2-135m-instruct.json:
#   huggingface-cli download HuggingFaceTB/SmolLM2-135M-Instruct \
#     --revision 7e27bd9f95328f0f3b08261d1252705110c806f8 \
#     config.json model.safetensors tokenizer.json --local-dir .uor-models/sources/smollm2-135m-instruct
# CALIBRATION_TEXT and EVAL_TEXT are disjoint plain-text files (a few hundred KB each is enough); the evaluation
# text is never seen by the calibration. OUT_PARENT (default reports/lut-m1-<timestamp>) must not exist; every
# fidelity run claims and seals its own report root inside it.
#
# Stages:
#   1. build lut-tool, kappa-conversion (tokenizer) and lut-chat in release mode
#   2. tokenize both texts with the checkpoint's own byte-level BPE
#   3. export twice: round-to-nearest, and GPTQ calibrated on CALIBRATION_TEXT
#   4. fidelity of each artifact against the float checkpoint on EVAL_TEXT: next-token NLL change, KL, top-1
#   5. decoding throughput at 1 thread and at every performance core
#   6. one chat turn with the GPTQ artifact, sampled with SmolLM2's suggested settings (CHAT_SAMPLING)
#   7. only with CACHE=1: train a learned Lorentz cache memory (lab M4) on CACHE_TEXT over the checkpoint's final
#      states, with a CACHE_WINDOW-token attention window (the cache reads only what the window cannot see),
#      export gptq-cache.lut with that window and the cache, and check that the integer cache keeps the float
#      cache's gain on EVAL_TEXT (cache-fidelity). Chat with it through lut-chat as usual; long chats then slide
#      the window and keep older text in the cache.
#   8. only with ENERGY=1 (needs sudo for powermetrics): joules per token through scripts/energy_per_token.py,
#      idle-subtracted, REPEATS times; with LLAMA_CLI and GGUF also set, the same for llama.cpp on that GGUF
#      (for example SmolLM2-135M-Instruct Q4_0) as the reference point
#
# Knobs: MAX_POSITIONS (2048), WINDOWS (16), TIME (256), CAL_WINDOWS (64), DAMP (0.01), THREADS (performance
# cores), DEVICE (cpu; metal runs the float side on the GPU and needs a --features metal build), CHAT_TOKENS (128),
# CHAT_SAMPLING ("temperature=0.2 top_p=0.9 seed=1"; energy runs stay greedy, like llama.cpp at --temp 0),
# ENERGY (0), REPEATS (3), ENERGY_TOKENS (256), LLAMA_CLI, GGUF, CACHE (0), CACHE_TEXT (a few MB of text, disjoint
# from EVAL_TEXT), CACHE_WINDOW (512), CACHE_SEGMENTS (800), CACHE_STEPS (200; steps x 4 <= segments reads each
# segment once), CACHE_DIM (32). The cache stage keeps about CACHE_SEGMENTS x 2 x CACHE_WINDOW x width x 4 bytes of
# backbone states in memory (1.9 GB for SmolLM2-135M at the defaults).
#
# The serving path is integer-only with multiplier-free weight maps (NEON table reads, shifts and additions);
# the dense backbone is D10's interim chat vehicle and is not D5-sparse. Send OUT_PARENT back for the record.
set -euo pipefail

MODEL=${1:?usage: scripts/lut-m1-chat.sh MODEL_DIR CALIBRATION_TEXT EVAL_TEXT [OUT_PARENT]}
CAL_TEXT=${2:?calibration text required}
EVAL_TEXT=${3:?evaluation text required}
OUT=${4:-reports/lut-m1-$(date +%Y%m%d-%H%M%S)}
MAX_POSITIONS=${MAX_POSITIONS:-2048}
WINDOWS=${WINDOWS:-16}
TIME=${TIME:-256}
CAL_WINDOWS=${CAL_WINDOWS:-64}
DAMP=${DAMP:-0.01}
THREADS=${THREADS:-$(sysctl -n hw.perflevel0.physicalcpu 2>/dev/null || getconf _NPROCESSORS_ONLN)}
DEVICE=${DEVICE:-cpu}
CHAT_TOKENS=${CHAT_TOKENS:-128}
ENERGY=${ENERGY:-0}
REPEATS=${REPEATS:-3}
ENERGY_TOKENS=${ENERGY_TOKENS:-256}
PROMPT=${PROMPT:-"Explain in two sentences why the sky is blue."}
CACHE=${CACHE:-0}
CACHE_WINDOW=${CACHE_WINDOW:-512}
CACHE_SEGMENTS=${CACHE_SEGMENTS:-800}
CACHE_STEPS=${CACHE_STEPS:-200}
CACHE_DIM=${CACHE_DIM:-32}
CHAT_SAMPLING=${CHAT_SAMPLING:-"temperature=0.2 top_p=0.9 seed=1"}
TARGET=${CARGO_TARGET_DIR:-target}
TOOL=$TARGET/release/examples/lut-tool
TOKENIZE=$TARGET/release/examples/kappa-conversion
CHAT=$TARGET/release/lut-chat
CACHE_TOOL=$TARGET/release/examples/cache-memory

for file in config.json model.safetensors tokenizer.json; do
  [ -f "$MODEL/$file" ] || { echo "missing $MODEL/$file (download the checkpoint first)" >&2; exit 1; }
done
for text in "$CAL_TEXT" "$EVAL_TEXT"; do
  [ -f "$text" ] || { echo "missing text file $text" >&2; exit 1; }
done
[ "$(cd "$(dirname "$CAL_TEXT")" && pwd)/$(basename "$CAL_TEXT")" != \
  "$(cd "$(dirname "$EVAL_TEXT")" && pwd)/$(basename "$EVAL_TEXT")" ] ||
  { echo "calibration and evaluation texts must differ" >&2; exit 1; }
[ -e "$OUT" ] && { echo "$OUT exists; choose a new output parent" >&2; exit 1; }
mkdir -p "$OUT"
git rev-parse HEAD > "$OUT/source-revision.txt"
uname -a > "$OUT/machine.txt"
sysctl -n machdep.cpu.brand_string >> "$OUT/machine.txt" 2>/dev/null || true

features=()
[ "$DEVICE" = metal ] && features=(--features metal)
cargo build --release ${features[@]+"${features[@]}"} -p uor-r4-training --example lut-tool --example kappa-conversion \
  --example cache-memory
cargo build --release -p uor-r4-lut --bin lut-chat

"$TOKENIZE" mode=tokenize model="$MODEL" text="$CAL_TEXT" out="$OUT/calibration.u16"
"$TOKENIZE" mode=tokenize model="$MODEL" text="$EVAL_TEXT" out="$OUT/eval.u16"

"$TOOL" mode=export model="$MODEL" out="$OUT/nearest.lut" max_positions="$MAX_POSITIONS" 2>&1 |
  tee "$OUT/export-nearest.log"
"$TOOL" mode=export model="$MODEL" out="$OUT/gptq.lut" max_positions="$MAX_POSITIONS" \
  calibration="$OUT/calibration.u16" calibration_windows="$CAL_WINDOWS" calibration_time="$TIME" damp="$DAMP" 2>&1 |
  tee "$OUT/export-gptq.log"

for artifact in nearest gptq; do
  "$TOOL" mode=fidelity model="$MODEL" lut="$OUT/$artifact.lut" tokens="$OUT/eval.u16" out="$OUT/fidelity-$artifact" \
    windows="$WINDOWS" time="$TIME" threads="$THREADS" device="$DEVICE"
done

for threads in 1 "$THREADS"; do
  "$TOOL" mode=bench lut="$OUT/gptq.lut" tokens=256 threads="$threads" | tee "$OUT/bench-threads-$threads.json"
done

# shellcheck disable=SC2086 # CHAT_SAMPLING is a list of key=value arguments
"$CHAT" lut="$OUT/gptq.lut" tokenizer="$MODEL/tokenizer.json" threads="$THREADS" tokens="$CHAT_TOKENS" \
  $CHAT_SAMPLING prompt="$PROMPT" > "$OUT/chat-sample.txt" 2> "$OUT/chat-sample.log"
cat "$OUT/chat-sample.txt"

if [ "$CACHE" = 1 ]; then
  [ -f "${CACHE_TEXT:-}" ] || { echo "CACHE=1 needs CACHE_TEXT, a text file disjoint from EVAL_TEXT" >&2; exit 1; }
  "$TOKENIZE" mode=tokenize model="$MODEL" text="$CACHE_TEXT" out="$OUT/cache-train.u16"
  "$CACHE_TOOL" model="$MODEL" train="$OUT/cache-train.u16" valid="$OUT/eval.u16" out="$OUT/cache" \
    geometries=lorentz seeds=1 dim="$CACHE_DIM" gap="$CACHE_WINDOW" window="$CACHE_WINDOW" \
    segment=$((2 * CACHE_WINDOW)) train_segments="$CACHE_SEGMENTS" valid_segments=16 steps="$CACHE_STEPS" \
    batch=4 save=true device="$DEVICE"
  "$TOOL" mode=export model="$MODEL" out="$OUT/gptq-cache.lut" max_positions="$CACHE_WINDOW" \
    calibration="$OUT/calibration.u16" calibration_windows="$CAL_WINDOWS" calibration_time="$TIME" damp="$DAMP" \
    cache="$OUT/cache/models/lorentz-seed1.safetensors" 2>&1 | tee "$OUT/export-gptq-cache.log"
  "$TOOL" mode=cache-fidelity model="$MODEL" lut="$OUT/gptq-cache.lut" \
    cache="$OUT/cache/models/lorentz-seed1.safetensors" tokens="$OUT/eval.u16" out="$OUT/cache-fidelity" \
    segments=16 segment=$((2 * CACHE_WINDOW)) window="$CACHE_WINDOW" threads="$THREADS"
fi

if [ "$ENERGY" = 1 ]; then
  for repeat in $(seq 1 "$REPEATS"); do
    sudo python3 scripts/energy_per_token.py --label "lut-gptq-$repeat" --idle-seconds 8 -- \
      "$CHAT" lut="$OUT/gptq.lut" tokenizer="$MODEL/tokenizer.json" threads="$THREADS" \
      tokens="$ENERGY_TOKENS" prompt="$PROMPT" 2>&1 | tee "$OUT/energy-lut-$repeat.txt"
    if [ -n "${LLAMA_CLI:-}" ] && [ -n "${GGUF:-}" ]; then
      # llama.cpp prints no count the energy script recognizes; the prompt is short, so the generated count
      # is used (a slight overstatement of llama.cpp's J/token; its prompt runs as one batched step). Flag
      # names follow recent llama.cpp builds; adjust them for the installed version.
      sudo python3 scripts/energy_per_token.py --label "llama-cpp-$repeat" --idle-seconds 8 \
        --tokens "$ENERGY_TOKENS" -- "$LLAMA_CLI" -m "$GGUF" -p "$PROMPT" -n "$ENERGY_TOKENS" -t "$THREADS" \
        --temp 0 -no-cnv 2>&1 | tee "$OUT/energy-llama-cpp-$repeat.txt"
    fi
  done
fi

echo "done: fidelity-*/fidelity.json, bench-threads-*.json, chat-sample.txt, cache-fidelity/ and energy-*.txt are in $OUT"
