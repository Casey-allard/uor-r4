#!/usr/bin/env bash
# Step 5 of the 5 October plan (#820): open-domain knowledge corpus and a
# matched-token 29M A/B, then the Arm C chat fine-tune, D19 grounded sessions
# and the open 232-request panel.
#
# Design and frozen decision rule: docs/research/step5/knowledge-ab.md.
# Single GPU (A100 80 GB friendly): tf32=true, data_parallel=1.
#
#   scripts/pod/step5-knowledge.sh STAGE
#
# STAGE (run in this order; `all` runs fetch..replies; each stage skips work whose
# sealed report root already exists and refuses to reuse an unsealed one; a sealed
# base or fine-tune root counts as done only at its full step count (70,609 or
# 4,000): a root capped by max_seconds is moved aside, never deleted, together
# with the roots built from it, and the run continues from its checkpoint):
#   fetch      download Simple English Wikipedia (one parquet file) and check SHA-256
#   build      build the four tools from this checkout and run their focused tests
#   data       verify every uploaded input by SHA-256; rebuild the Arm C corpus
#   knowledge  prepare the decontaminated knowledge stores
#   base       4 base runs: arms A, K x seeds 1, 2 (29M, 433.8M tokens each)
#   ft         4 Arm C fine-tunes from the four bases
#   eval       knowledge held-out NLL (bases and fine-tunes) + D19 sessions (sieve, off)
#   replies    greedy replies of each fine-tune to the 232 open-panel requests
#   grade      qwen2.5:7b grades of those replies (needs Ollama; laptop by default)
#   tabulate   tables and the frozen decision (scripts/pod/step5-tabulate.py)
#
# Inputs to upload from the laptop first (paths are the laptop's; SHA-256 checked
# by `data`):
#   scp tokenizer:     ~/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json  -> $D/tokenizer.json
#   ts-train:          ~/uor-r4-local/ladder/corpora/tinystories-v2-train/   -> $D/corpora/ts-train/
#   ts-valid:          ~/uor-r4-local/ladder/corpora/tinystories-v2-valid/   -> $D/corpora/ts-valid/
#   td-train:          ~/uor-r4-local/ladder/corpora/tinydialogues-train/    -> $D/corpora/td-train/
#   chat-v0-p2 train:  ~/uor-r4-local/chat-v0-p2/train/                      -> $D/chat-v0-p2/train/
#   M-world parar:     ~/uor-r4-local/claude-instr-para/parar-p2-train/train/ -> $D/ft/parar/
#   fine-tune dev:     ~/uor-r4-local/claude-protocol/instr2-p2-dev/train/  -> $D/ft/dev/
#   sieve compiler:    ~/uor-r4-local/claude-log-sieve/compiler-save-op-v25-rawtable/ -> $D/sieve/compiler-save-op-v25-rawtable/
#   sieve trunk:       ~/uor-r4-local/claude-log-sieve/op-model-v25/model/  -> $D/sieve/op-model-v25/model/
#   open panel:        ~/uor-r4-local/ladder/panel/*.json                    -> $D/panel/
# (about 1.6 GB in total; the earlier ladder pod already holds most of it under /root/data.)
#
# Expected wall time on one A100 80 GB (projection, not measured; the 29M rung
# was trained on an RTX 4090): fetch+build+data+knowledge ~0.5 h; base 4 x
# ~1.5-2.5 h; ft 4 x ~0.4 h; eval ~1.5 h (CPU); replies ~0.5 h. About 9-14 h
# end to end, 12 h central. Each base run stops at max_seconds=21600 (6 h) and
# each fine-tune at 10800 s. New storage: ~0.5 GB knowledge stores, ~0.2 GB
# raw parquet, ~1.2 GB runs (4 x 116 MB bases + checkpoints, 4 fine-tunes).
set -euo pipefail

STAGE=${1:?usage: step5-knowledge.sh fetch|build|data|knowledge|base|ft|eval|replies|grade|tabulate|all}
REPO=${REPO:-/root/uor-r4}                 # checkout of this PR's head commit
D=${D:-/root/data}                         # uploaded inputs
KN=${KN:-$D/knowledge}                     # knowledge corpus root
R=${R:-/root/runs/step5}                   # every run and report of this A/B
GPU=${GPU:-0}
THREADS=${THREADS:-16}                     # CPU threads for preparation, sessions and replies
GRADE_THREADS=${GRADE_THREADS:-2}
OLLAMA_URL=${OLLAMA_URL:-http://127.0.0.1:11434}
CUDA_COMPUTE_CAP=${CUDA_COMPUTE_CAP:-80}   # 80 = A100; 89 = RTX 4090; 90 = H100
SEEDS=(1 2)
ARMS=(A K)

CPU_TARGET=$REPO/target-step5
GPU_TARGET=$REPO/target-step5-cuda
GS=$GPU_TARGET/release/examples/geometric-stack
MWORLD=$CPU_TARGET/release/examples/m-world
CHATGRADE=$CPU_TARGET/release/chat-grade
MIX=$CPU_TARGET/release/mix-chat-corpus
PREP=$CPU_TARGET/release/prepare-knowledge-corpus
T=$D/tokenizer.json

# Simple English Wikipedia, 1 November 2023 dump, as exported by Wikimedia on the
# Hugging Face Hub (wikimedia/wikipedia, config 20231101.simple), pinned to the
# commit that added the file. Licence: CC BY-SA 4.0 (text also GFDL); attribution
# and share-alike apply to the corpus and to redistributed derived text; whether
# they reach trained weights is unsettled (docs/research/step5/knowledge-ab.md §2).
# Keep the prepared stores on the pod/laptop; do not publish them.
WIKI_URL=https://huggingface.co/datasets/wikimedia/wikipedia/resolve/cf26be8eef6bc935da5d2281be5aad4813e49e90/20231101.simple/train-00000-of-00001.parquet
WIKI_FILE=$KN/raw/simplewiki-20231101-train-00000-of-00001.parquet
WIKI_SHA256=31bded16768a47c286becd292079122f5d7d4397a17b87d4250a00ccd581e6f0   # Hub LFS pointer; ~157 MB
KSTORE=$KN/simplewiki-20231101-panel8

PANELS=$D/panel/everyday-32.json,$D/panel/heldout-200-a.json,$D/panel/heldout-200-b.json,$D/panel/heldout-200.json,$D/panel/stretch-32.json
REQUESTS=$D/panel/everyday-32.json,$D/panel/heldout-200-a.json,$D/panel/heldout-200-b.json

log() { echo "$(date -u +%FT%TZ) $*" | tee -a "$R/step5.log"; }
fail() { log "FAILED $1 (see $1.log)"; exit 1; }

check() {  # check FILE SHA256
  local got
  got=$(sha256sum "$1" | cut -d' ' -f1)
  if [ "$got" != "$2" ]; then echo "SHA-256 mismatch: $1 has $got, expected $2" >&2; exit 1; fi
}

# A report root is never reused: skip a sealed one, refuse an unsealed one.
fresh() {  # fresh ROOT -> 0 when the work must run
  if [ -f "$1/manifest.json" ]; then log "skip $1 (sealed)"; return 1; fi
  if [ -e "$1" ]; then
    echo "$1 exists but is not sealed: inspect it, then continue in a NEW root (train/dialogue-train accept resume=OLD/checkpoint)" >&2
    exit 1
  fi
  return 0
}

# A sealed train root counts as done only when it completed its steps. A run
# capped by max_seconds still seals a model and records stopped_early and
# completed_steps; it is not the matched-token arm the decision rule was frozen
# for. Such a root is moved aside (never deleted) and the run continues from its
# checkpoint in a fresh root with the same name.
completed() {  # completed ROOT STEPS STRICT -> 0 when report.json shows the full run
  python3 - "$1/report.json" "$2" "$3" <<'PY'
import json, sys
path, steps, strict = sys.argv[1], int(sys.argv[2]), sys.argv[3] == '1'
try:
    report = json.load(open(path))
except (OSError, ValueError):
    sys.exit(1)
ok = report.get('completed_steps') == steps and (not strict or report.get('stopped_early') is False)
sys.exit(0 if ok else 1)
PY
}

move_aside() {  # move_aside ROOT TAG -> moves ROOT (and ROOT.log) to ROOT.TAG-<UTC>; prints the new path
  local aside
  aside="$1.$2-$(date -u +%Y%m%dT%H%M%SZ)"
  mv "$1" "$aside"
  if [ -f "$1.log" ]; then mv "$1.log" "$aside.log"; fi
  echo "$aside"
}

RESUME_ARG=()
fresh_train() {  # fresh_train ROOT STEPS STRICT [DEPENDENT_ROOT...] -> 0 when the work must run; sets RESUME_ARG
  local root=$1 steps=$2 strict=$3 aside dependent
  shift 3
  RESUME_ARG=()
  if [ -f "$root/manifest.json" ]; then
    if completed "$root" "$steps" "$strict"; then log "skip $root (sealed, $steps steps)"; return 1; fi
    aside=$(move_aside "$root" capped)
    log "capped $root (completed_steps short of $steps or stopped_early): moved to $aside"
    # Roots built from the capped model are stale: move them aside as well.
    for dependent in "$@"; do
      if [ -e "$dependent" ]; then log "stale $dependent: moved to $(move_aside "$dependent" stale)"; fi
    done
    if [ -d "$aside/checkpoint" ]; then
      RESUME_ARG=("resume=$aside/checkpoint")
      log "continue $root from $aside/checkpoint"
    fi
    return 0
  fi
  fresh "$root"
}

stage_fetch() {
  mkdir -p "$KN/raw"
  if [ ! -f "$WIKI_FILE" ]; then
    log "fetch $WIKI_URL"
    curl -fL --retry 5 --retry-delay 10 -o "$WIKI_FILE.part" "$WIKI_URL"
    mv "$WIKI_FILE.part" "$WIKI_FILE"
  fi
  check "$WIKI_FILE" "$WIKI_SHA256"
  { echo "url $WIKI_URL"; echo "bytes $(stat -c %s "$WIKI_FILE")"; sha256sum "$WIKI_FILE"; } > "$KN/raw/FETCH.txt"
  log "fetched $(cat "$KN/raw/FETCH.txt" | tr '\n' ' ')"
}

stage_build() {
  cd "$REPO"
  git rev-parse HEAD > "$R/build-commit.txt"
  CARGO_TARGET_DIR=$CPU_TARGET cargo test --release -p uor-r4-training --features parquet-input \
    --lib knowledge_corpus
  CARGO_TARGET_DIR=$CPU_TARGET cargo test --release -p uor-r4-training --features parquet-input \
    --bin prepare-knowledge-corpus
  CARGO_TARGET_DIR=$CPU_TARGET cargo build --release -p uor-r4-training --features parquet-input \
    --bin prepare-knowledge-corpus --bin mix-chat-corpus --bin chat-grade --example m-world
  CARGO_INCREMENTAL=0 CUDA_COMPUTE_CAP=$CUDA_COMPUTE_CAP CARGO_TARGET_DIR=$GPU_TARGET \
    cargo build --release -p uor-r4-training --features cuda --example geometric-stack
  sha256sum "$GS" "$MWORLD" "$CHATGRADE" "$MIX" "$PREP" > "$R/binaries.sha256"
  log "built $(cat "$R/build-commit.txt")"
}

stage_data() {
  check "$T" d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89
  check "$D/corpora/ts-train/tokens.u16" 2ba5e64fe076bbb5ffd037df4ff90f697e793f96fda01ada921f1db2e533f0fc
  check "$D/corpora/ts-valid/tokens.u16" 8ebf9db3033963aa5a3a34fe91f4715fe275c73f7c4e4ca7b98cf9a2d87af422
  check "$D/corpora/td-train/tokens.u16" b4e24f1f202f2375dc830422c50be5b48d6c1be6a39aeaa7173a2fb84e9f50d3
  check "$D/chat-v0-p2/train/tokens.u16" b5d3bf977a524d9e47c0c75592d0170a183271ddf0cf4d7b560e3ece7a54f184
  check "$D/chat-v0-p2/train/response_mask.u8" 28b7e98bb8a85368596dd7d1b8aa8aa16a277879a8368925dadbd37f20c3b3c1
  check "$D/ft/parar/tokens.u16" 76a409ecd72ca9badfce7e4e032ead236fc9da83e0ab69d830ca99d4e4e3c8fd
  check "$D/ft/parar/response_mask.u8" 199f61565f1f314e74babfb79bb5fb6f797eb8a6b621003d792f7bbfdc95749f
  check "$D/ft/dev/tokens.u16" 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b
  check "$D/ft/dev/response_mask.u8" a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078
  check "$D/ft/dev/manifest.json" 74a9158604336ef8079e13e615508ea4cd56b2253ce020b3335979cb152f847c
  check "$D/sieve/compiler-save-op-v25-rawtable/compiler.json" 34cdf896abf5d823e185edc269454c6ad92d97b083ad7ef8f093944931783f06
  check "$D/sieve/op-model-v25/model/config.json" e1009111a801518160db3b58a1c9981302ad2649fda296f3cf35e6d0af567627
  check "$D/sieve/op-model-v25/model/model.safetensors" 04ce0d7a253cca6aeedbcf8d38837fed9fcf3a6b430e13f10eb098ac055db4ce
  check "$D/panel/everyday-32.json" 945c0c97196d39db430b5821c8c30a888a3c719c6e701c218fc7a914b350b480
  check "$D/panel/heldout-200-a.json" 019cc6f65e8ec815d29328f7dd48f49126beaa29b29d1b4bcc7d4d156dffd2c1
  check "$D/panel/heldout-200-b.json" 5434cfd9767c78a179e0899c9568971466dd36e8dcf4f106e1de1566fb29436c
  check "$D/panel/heldout-200.json" 67d45fde7b84fd25900c81f8183ddd884bc5c5ef55c840a684d9b2ce4efa694f
  check "$D/panel/stretch-32.json" 1d8fcbc4b3399dd51cfa8409612f46fddd0ce59f2b1f69211ed3dce05ad995ff
  # Arm C fine-tune corpus (the 96M chat-100m-C store): M-world parar three times,
  # then chat-v0-p2. Rebuilt here; verified against the bytes Arm C trained on.
  if [ ! -f "$D/ft/mixed-c/tokens.u16" ]; then
    "$MIX" out="$D/ft/mixed-c" tokenizer="$T" \
      inputs="$D/ft/parar,$D/ft/parar,$D/ft/parar,$D/chat-v0-p2/train" \
      labels=mworld-parar-1,mworld-parar-2,mworld-parar-3,chat-v0-p2
  fi
  check "$D/ft/mixed-c/tokens.u16" 7282b7a6ea53dfb79891c2738f8891c146873d7a442d880c435d553be52d6ec7
  check "$D/ft/mixed-c/response_mask.u8" 0a26ee3178181980927c267786fde2ea01c7823bc3aeca40c3e5b8efff9b3188
  log "inputs verified"
}

stage_knowledge() {
  check "$WIKI_FILE" "$WIKI_SHA256"
  if fresh "$KSTORE"; then
    RAYON_NUM_THREADS=$THREADS "$PREP" out="$KSTORE" tokenizer="$T" \
      input="$WIKI_FILE" format=parquet \
      source=wikimedia/wikipedia:20231101.simple@cf26be8eef6bc935da5d2281be5aad4813e49e90 \
      licence=CC-BY-SA-4.0 panel="$PANELS" \
      min_words=20 heldout_every=200 title=prepend \
      2>&1 | tee "$KN/prepare.log"
  fi
  python3 - "$KSTORE/corpus.json" <<'PY'
import json, sys
c = json.load(open(sys.argv[1]))
drawn = 0.25 * 70609 * 16 * 384
t = c['train']['tokens']
print(f"knowledge train tokens {t:,}; drawn per K run {drawn:,.0f}; epochs {drawn / t:.2f}")
print("counts", json.dumps(c['counts']))
if drawn / t > 4:
    sys.exit("more than 4 epochs of the knowledge corpus: stop and revisit the weight")
PY
}

# Arm A: the 29M lr 5e-4 recipe's mix (TinyStories / TinyDialogues / chat-v0-p2 =
# 0.6 / 0.15 / 0.25). Arm K: the same three scaled by 0.75 plus the knowledge
# corpus at 0.25. Same steps x batch x context (433,821,696 tokens) in both.
train_streams() {  # train_streams ARM
  local base="$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16"
  case $1 in
    A) echo "train=$base train_weights=0.6,0.15,0.25" ;;
    K) echo "train=$base,$KSTORE/train/tokens.u16 train_weights=0.45,0.1125,0.1875,0.25" ;;
  esac
}

stage_base() {
  for seed in "${SEEDS[@]}"; do
    for arm in "${ARMS[@]}"; do
      local key=$arm-s$seed
      local out=$R/base-$key
      fresh_train "$out" 70609 1 "$R/ft-C-$key" "$R/eval-knowledge-base-$key" "$R/eval-knowledge-ft-$key" \
        "$R/session-sieve-$key" "$R/session-off-$key" "$R/replies-$key" "$R/grade-$key" || continue
      log "base $arm seed $seed ${RESUME_ARG[*]+${RESUME_ARG[*]}}"
      # shellcheck disable=SC2046
      CUDA_VISIBLE_DEVICES=$GPU RAYON_NUM_THREADS=8 "$GS" train \
        out="$out" seed="$seed" lr=0.0005 ${RESUME_ARG[@]+"${RESUME_ARG[@]}"} \
        $(train_streams "$arm") \
        valid="$D/corpora/ts-valid/tokens.u16" tokenizer="$T" \
        arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 \
        read=l2 rotation=true key_shift=false \
        steps=70609 batch=16 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
        eval_every=5000 eval_windows=64 final_windows=512 \
        checkpoint_every=10000 max_seconds=21600 \
        device=cuda tf32=true data_parallel=1 \
        > "$out.log" 2>&1 || fail "$out"
      log "base $arm seed $seed done"
    done
  done
}

stage_ft() {
  for seed in "${SEEDS[@]}"; do
    for arm in "${ARMS[@]}"; do
      local key=$arm-s$seed
      local out=$R/ft-C-$key
      if ! completed "$R/base-$key" 70609 1; then
        echo "base-$key has not completed 70609 steps: run the base stage first" >&2
        exit 1
      fi
      # A fine-tune is complete at 4,000 steps (the tabulator's criterion).
      fresh_train "$out" 4000 0 "$R/eval-knowledge-ft-$key" "$R/session-sieve-$key" "$R/session-off-$key" \
        "$R/replies-$key" "$R/grade-$key" || continue
      log "ft Arm C $arm seed $seed ${RESUME_ARG[*]+${RESUME_ARG[*]}}"
      CUDA_VISIBLE_DEVICES=$GPU RAYON_NUM_THREADS=8 "$GS" dialogue-train \
        out="$out" tokenizer="$T" ${RESUME_ARG[@]+"${RESUME_ARG[@]}"} \
        train_tokens="$D/ft/mixed-c/tokens.u16" train_mask="$D/ft/mixed-c/response_mask.u8" \
        train_manifest="$D/ft/mixed-c/manifest.json" \
        dev_tokens="$D/ft/dev/tokens.u16" dev_mask="$D/ft/dev/response_mask.u8" \
        dev_manifest="$D/ft/dev/manifest.json" \
        init="$R/base-$arm-s$seed/model" \
        pointer=32 protocol=2 context=384 policy=full_prefix data_seed="$seed" \
        steps=4000 batch=16 lr=0.0003 warmup=100 min_lr=0.1 weight_decay=0.1 clip=1.0 \
        eval_every=500 checkpoint_every=4000 dev_seed=20260930 dev_per_source=32 \
        max_seconds=10800 device=cuda tf32=true \
        > "$out.log" 2>&1 || fail "$out"
      log "ft $arm seed $seed done"
    done
  done
}

stage_eval() {
  for seed in "${SEEDS[@]}"; do
    for arm in "${ARMS[@]}"; do
      local key=$arm-s$seed
      # Knowledge held-out NLL: the manipulation check (bases) and retention after the fine-tune.
      for pair in "base:$R/base-$key/model" "ft:$R/ft-C-$key/model"; do
        local stage=${pair%%:*} model=${pair#*:}
        local out=$R/eval-knowledge-$stage-$key
        fresh "$out" || continue
        RAYON_NUM_THREADS=$THREADS "$GS" evaluate model="$model" \
          tokens="$KSTORE/heldout/tokens.u16" out="$out" tune_blocks=64 > "$out.log" 2>&1 || fail "$out"
        log "eval knowledge $stage $key done"
      done
      # D19 grounded sessions: 300 conversations, 1,075 scored turns; sieve on and off.
      for recall in sieve off; do
        local out=$R/session-$recall-$key
        fresh "$out" || continue
        RAYON_NUM_THREADS=$THREADS VECLIB_MAXIMUM_THREADS=$THREADS "$MWORLD" session \
          out="$out" world=v2 model_root="$R/ft-C-$key" tokenizer="$T" \
          compiler="$D/sieve/compiler-save-op-v25-rawtable/compiler.json" \
          trunk="$D/sieve/op-model-v25/model" op_policy=unless_query \
          log_recall=$recall max_new_tokens=64 arms=default reload=0 \
          > "$out.log" 2>&1 || fail "$out"
        log "session $recall $key done"
      done
    done
  done
}

stage_replies() {
  for seed in "${SEEDS[@]}"; do
    for arm in "${ARMS[@]}"; do
      local out=$R/replies-$arm-s$seed
      fresh "$out" || continue
      RAYON_NUM_THREADS=$THREADS "$CHATGRADE" reply out="$out" \
        model="$R/ft-C-$arm-s$seed/model" tokenizer="$T" requests="$REQUESTS" \
        protocol=2 max_new_tokens=64 > "$out.log" 2>&1 || fail "$out"
      log "replies $arm seed $seed done"
    done
  done
}

# Grading needs a local Ollama with qwen2.5:7b (the grader of every ladder panel).
# On the laptop: copy $R/replies-* to the same layout under R=~/uor-r4-local/step5/runs,
# register the job (2 threads), then run this stage with REPO pointing at a
# checkout whose target-step5 holds chat-grade.
stage_grade() {
  for seed in "${SEEDS[@]}"; do
    for arm in "${ARMS[@]}"; do
      local out=$R/grade-$arm-s$seed
      fresh "$out" || continue
      RAYON_NUM_THREADS=$GRADE_THREADS "$CHATGRADE" grade-replies out="$out" \
        replies="$R/replies-$arm-s$seed/replies.json" grader=qwen2.5:7b \
        ollama_url="$OLLAMA_URL" > "$out.log" 2>&1 || fail "$out"
      log "grade $arm seed $seed done"
    done
  done
}

stage_tabulate() {
  python3 "$REPO/scripts/pod/step5-tabulate.py" "$R" \
    --clean "$REPO/docs/research/step5/panel-clean" --json "$R/step5-result-$(date -u +%Y%m%dT%H%M%SZ).json"
}

mkdir -p "$R"
case $STAGE in
  fetch|build|data|knowledge|base|ft|eval|replies|grade|tabulate) "stage_$STAGE" ;;
  all) for s in fetch build data knowledge base ft eval replies; do "stage_$s"; done ;;
  *) echo "unknown stage $STAGE" >&2; exit 2 ;;
esac
