#!/bin/bash
set -euo pipefail

# Reproduction script for S2 Dialogue Quantization-Aware Training (QAT) & Behavior Analysis
# References #973 under #820.
# Laboratory: Anti-Gravity (Lab 3)
#
# EXECUTION STATUS: REPRODUCTION NOT_RUN UNDER HOST PRODUCTION HOLD (#1536 / PR #1537) — PENDING FORMAL RUNNER ADMISSION

MODE="${1:-train-and-eval}"

MODEL_DIR="${MODEL_DIR:-/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model}"
BASELINE_LUT="${BASELINE_LUT:-/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/export-1/model.lut}"
TRAIN_DIR="${TRAIN_DIR:-/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/train}"
HELDOUT="${HELDOUT:-/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/heldout}"
REQUESTS="${REQUESTS:-tests/fixtures/development-requests.json}"
TOKENIZER="${TOKENIZER:-tests/fixtures/tokenizer.json}"
OUT_ROOT="${OUT_ROOT:-/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4-reproduction-$(date +%Y%m%d%H%M%S)}"

echo "=== S2 Dialogue QAT & Behavior Replication ==="
echo "Mode:         $MODE"
echo "Model:        $MODEL_DIR"
echo "Baseline LUT: $BASELINE_LUT"
echo "Train:        $TRAIN_DIR"
echo "Heldout:      $HELDOUT"
echo "Requests:     $REQUESTS"
echo "Tokenizer:    $TOKENIZER"
echo "Output Root:  $OUT_ROOT"

if [ "$MODE" = "train-and-eval" ]; then
  # --------------------------------------------------------------------------
  # Result (C) S2 Dialogue QAT (Arm 1)
  # --------------------------------------------------------------------------
  echo "--- Running Arm 1: S2 Dialogue QAT (qat=true, 1024 steps) ---"
  cargo run --release --example geometric-stack -- \
    dialogue-train \
    init="$MODEL_DIR" \
    train_tokens="$TRAIN_DIR/tokens.u16" \
    train_mask="$TRAIN_DIR/response_mask.u8" \
    train_manifest="$TRAIN_DIR/manifest.json" \
    dev_tokens="$HELDOUT/tokens.u16" \
    dev_mask="$HELDOUT/response_mask.u8" \
    dev_manifest="$HELDOUT/manifest.json" \
    requests="$REQUESTS" \
    tokenizer="$TOKENIZER" \
    out="$OUT_ROOT/s2_dialogue_qat_1/train" \
    steps=1024 \
    batch=16 \
    lr=0.0002 \
    warmup=50 \
    min_lr=0.1 \
    weight_decay=0.1 \
    clip=1.0 \
    eval_every=128 \
    checkpoint_every=512 \
    qat=true

  echo "--- Exporting Arm 1 QAT checkpoint to LUT ---"
  cargo run --release --example geometric-stack -- \
    export \
    model="$OUT_ROOT/s2_dialogue_qat_1/train/model" \
    out="$OUT_ROOT/s2_dialogue_qat_1/export"

  echo "--- Evaluating Arm 1 QAT greedy dialogue panel (58 turns) ---"
  cargo run --release --example geometric-stack -- \
    lut-chat \
    artifact="$OUT_ROOT/s2_dialogue_qat_1/export/model.lut" \
    requests="$REQUESTS" \
    tokenizer="$TOKENIZER" \
    out="$OUT_ROOT/s2_dialogue_qat_1/chat"

  echo "--- Evaluating Arm 1 QAT D11 integer serving parity ---"
  cargo run --release --example geometric-stack -- \
    d11-evaluate \
    artifact="$OUT_ROOT/s2_dialogue_qat_1/export/model.lut" \
    valid="$HELDOUT/tokens.u16" \
    out="$OUT_ROOT/s2_dialogue_qat_1/d11_eval" \
    windows=64

  # --------------------------------------------------------------------------
  # Result (C) S2 Dialogue Float Continuation Control (Arm 2)
  # --------------------------------------------------------------------------
  echo "--- Running Arm 2: Float Continuation Control (qat=false, 1024 steps) ---"
  cargo run --release --example geometric-stack -- \
    dialogue-train \
    init="$MODEL_DIR" \
    train_tokens="$TRAIN_DIR/tokens.u16" \
    train_mask="$TRAIN_DIR/response_mask.u8" \
    train_manifest="$TRAIN_DIR/manifest.json" \
    dev_tokens="$HELDOUT/tokens.u16" \
    dev_mask="$HELDOUT/response_mask.u8" \
    dev_manifest="$HELDOUT/manifest.json" \
    requests="$REQUESTS" \
    tokenizer="$TOKENIZER" \
    out="$OUT_ROOT/s2_dialogue_float_ctrl_1/train" \
    steps=1024 \
    batch=16 \
    lr=0.0002 \
    warmup=50 \
    min_lr=0.1 \
    weight_decay=0.1 \
    clip=1.0 \
    eval_every=128 \
    checkpoint_every=512 \
    qat=false

  echo "--- Exporting Arm 2 checkpoint to LUT ---"
  cargo run --release --example geometric-stack -- \
    export \
    model="$OUT_ROOT/s2_dialogue_float_ctrl_1/train/model" \
    out="$OUT_ROOT/s2_dialogue_float_ctrl_1/export"

  echo "--- Evaluating Arm 2 greedy dialogue panel (58 turns) ---"
  cargo run --release --example geometric-stack -- \
    lut-chat \
    artifact="$OUT_ROOT/s2_dialogue_float_ctrl_1/export/model.lut" \
    requests="$REQUESTS" \
    tokenizer="$TOKENIZER" \
    out="$OUT_ROOT/s2_dialogue_float_ctrl_1/chat"

  echo "--- Evaluating Arm 2 D11 integer serving parity ---"
  cargo run --release --example geometric-stack -- \
    d11-evaluate \
    artifact="$OUT_ROOT/s2_dialogue_float_ctrl_1/export/model.lut" \
    valid="$HELDOUT/tokens.u16" \
    out="$OUT_ROOT/s2_dialogue_float_ctrl_1/d11_eval" \
    windows=64

elif [ "$MODE" = "diagnostic" ]; then
  # --------------------------------------------------------------------------
  # Diagnostic Tools: s2-behavior-codec and s2-behavior-distortion
  # --------------------------------------------------------------------------
  OUT_DIR="${OUT_DIR:-$OUT_ROOT/s2-turn-optimization-2}"
  echo "--- Running s2-behavior-codec diagnostic ---"
  cargo run --release --bin s2-behavior-codec -- \
    model="$MODEL_DIR" \
    baseline_lut="$BASELINE_LUT" \
    heldout="$HELDOUT" \
    requests="$REQUESTS" \
    tokenizer="$TOKENIZER" \
    out="$OUT_DIR"

  if [ "${RUN_DISTORTION:-0}" = "1" ]; then
    DIST_OUT="${DIST_OUT:-$OUT_ROOT/s2-behavior-distortion-1}"
    echo "--- Running s2-behavior-distortion diagnostic ---"
    cargo run --release --bin s2-behavior-distortion -- \
      model="$MODEL_DIR" \
      artifact="$BASELINE_LUT" \
      heldout="$HELDOUT/tokens.u16" \
      requests="$REQUESTS" \
      tokenizer="$TOKENIZER" \
      windows=32 \
      out="$DIST_OUT"
  fi
fi

echo "S2 dialogue execution complete."
