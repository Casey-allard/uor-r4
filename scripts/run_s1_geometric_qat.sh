#!/bin/bash
set -euo pipefail

# Reproduction script for Result (B): S1 Geometric Quantization-Aware Training (QAT)
# References #973 under #820.
# Laboratory: Anti-Gravity (Lab 3)

MODEL_DIR="${MODEL_DIR:-/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model}"
C3_DATA="${C3_DATA:-/Volumes/UOR-Workspace/uor-r4-models/investigations/cloud-data-20260928/c3/data/code/train.u16}"
C4_DATA="${C4_DATA:-/Volumes/UOR-Workspace/uor-r4-models/investigations/cloud-data-20260928/c4/data/registry.u16}"
VALID_DATA="${VALID_DATA:-/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/valid.u16}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
OUT_ROOT="${OUT_ROOT:-/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4-reproduction}"
RUN_DIR="$OUT_ROOT/s1_reproduction_$TIMESTAMP"

echo "=== S1 Geometric QAT Replication ==="
echo "Model Dir:  $MODEL_DIR"
echo "Out Root:   $RUN_DIR"
echo "STATUS: REPRODUCTION NOT_RUN UNDER HOST PRODUCTION HOLD (#1536 / PR #1537) — PENDING FORMAL RUNNER ADMISSION"

# Arm 1: QAT with straight-through estimator and interim 4-bit codec
echo "Running Arm 1 (QAT)..."
cargo run --release -p uor-r4-training --example geometric-stack -- \
  train \
  init="$MODEL_DIR" \
  train="$C3_DATA,$C4_DATA" \
  train_weights="1.0,1.0" \
  valid="$VALID_DATA" \
  out="$RUN_DIR/geometric_s1_qat_d11_1000" \
  steps=1000 \
  batch=16 \
  lr=0.0005 \
  warmup=100 \
  min_lr=0.1 \
  weight_decay=0.1 \
  clip=1.0 \
  seed=1 \
  eval_every=250 \
  eval_windows=64 \
  final_windows=512 \
  checkpoint_every=250 \
  qat=true

# Arm 1 Export to LUT
echo "Exporting Arm 1 to LUT..."
cargo run --release -p uor-r4-training --example geometric-stack -- \
  export \
  model="$RUN_DIR/geometric_s1_qat_d11_1000/model" \
  out="$RUN_DIR/geometric_s1_qat_d11_1000_export"

# Arm 1 LUT Integer Evaluation
echo "Evaluating Arm 1 LUT Integer Engine..."
cargo run --release -p uor-r4-training --example geometric-stack -- \
  lut-evaluate \
  artifact="$RUN_DIR/geometric_s1_qat_d11_1000_export/model.lut" \
  valid="$VALID_DATA" \
  out="$RUN_DIR/geometric_s1_qat_d11_1000_lut_eval" \
  windows=512

# Arm 1 D11 / D10 Integer Evaluation & Parity
echo "Evaluating Arm 1 D11/D10 Integer Parity..."
cargo run --release -p uor-r4-training --example geometric-stack -- \
  d11-evaluate \
  artifact="$RUN_DIR/geometric_s1_qat_d11_1000_export/model.lut" \
  valid="$VALID_DATA" \
  out="$RUN_DIR/geometric_s1_qat_d11_1000_d11_eval" \
  windows=512

# Arm 2: Float Continuation Control
echo "Running Arm 2 (Float Continuation Control)..."
cargo run --release -p uor-r4-training --example geometric-stack -- \
  train \
  init="$MODEL_DIR" \
  train="$C3_DATA,$C4_DATA" \
  train_weights="1.0,1.0" \
  valid="$VALID_DATA" \
  out="$RUN_DIR/geometric_s1_float_ctrl_1000" \
  steps=1000 \
  batch=16 \
  lr=0.0005 \
  warmup=100 \
  min_lr=0.1 \
  weight_decay=0.1 \
  clip=1.0 \
  seed=1 \
  eval_every=250 \
  eval_windows=64 \
  final_windows=512 \
  checkpoint_every=250 \
  qat=false

echo "Replication recipe prepared (execution deferred under host production hold)."
