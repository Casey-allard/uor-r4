#!/bin/bash
# BVG-K Stage C2 device-level qualification (sheet m1-bvgk-stagec2-block-qualification-2026-10-18.md, FIT).
# Fixture-only synthetic devices; commits only via apply_block; no corpus, no training.
# Kimi PM dispatch #2029 6257564516; owner-approved fixture-only compute.
set -uo pipefail
SRC=$(ls -d /root/build/src-* | head -1)
R=/workspace/uor-r4/runs/claude/bvgk-c2-qualify-20261019
export CARGO_TARGET_DIR=/root/build/target-sm120 UOR_NVCC_ARCH=compute_120 PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
mark() { echo "## $(date -u +%FT%TZ) $*"; }
mkdir -p "$R"
cd "$SRC"
mark "source $(git rev-parse HEAD)"
git rev-parse HEAD > "$R/source-head.txt"
nvidia-smi --query-gpu=name,driver_version,memory.used --format=csv,noheader
set -e
mark "D29 checker"
rustc --edition=2021 scripts/check_native_cuda_vendor.rs -o "$R/d29-checker"
"$R/d29-checker" --self-test > "$R/d29-selftest.txt" 2>&1
"$R/d29-checker" --repo . > "$R/d29.txt" 2>&1
sha256sum "$R/d29-selftest.txt" "$R/d29.txt"
mark "build all uor-r4-discrete CUDA binaries (shared with the injective pre-flight)"
cargo build -q --release --locked -p uor-r4-discrete --features cuda
ls -la "$CARGO_TARGET_DIR/release/bvgk-qualify" "$CARGO_TARGET_DIR/release/discrete-stack" | tee "$R/bins.txt"
sha256sum "$CARGO_TARGET_DIR/release/bvgk-qualify" | tee -a "$R/bins.txt"
touch /workspace/uor-r4/runs/claude/BUILD_DONE
set +e
mark "run bvgk-qualify (fixture-only)"
"$CARGO_TARGET_DIR/release/bvgk-qualify" "$R/report" > "$R/qualify.stdout" 2>&1
echo "QUALIFY_RC=$?"
tail -c 3000 "$R/qualify.stdout"
ls -la "$R/report" 2>&1
mark "done"
echo "C2_DONE"
