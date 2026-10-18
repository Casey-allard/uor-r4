#!/bin/bash
# BVG-K Stage C qualification on one RTX 5090 (sheet m1-bvgk-stagec-rowsk-qualification-2026-10-18.md, FIT).
# Finite controls only: no corpus, no training, zero applied updates.
set -uo pipefail
HEAD_SHA=b2d6c7c63cea8715e3d3be9f68a0b820fb78f65f
BASE=/root/build/src-acf3029da680b59d1dff6cfe28b843949b9b13bf
SRC=/root/build/src-stagec-$HEAD_SHA
R=/workspace/uor-r4/runs/claude/bvgk-stagec-20261018
export CARGO_TARGET_DIR=/root/build/target-sm120 UOR_NVCC_ARCH=compute_120 PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
mark() { echo "## $(date -u +%FT%TZ) $*"; }
mkdir -p "$R"
set -e
mark "checkout $HEAD_SHA"
git -C "$BASE" fetch -q https://github.com/UOR-Foundation/uor-r4.git "$HEAD_SHA"
git -C "$BASE" worktree add -q -f "$SRC" "$HEAD_SHA"
cd "$SRC"
test "$(git rev-parse HEAD)" = "$HEAD_SHA"
nvidia-smi --query-gpu=name,driver_version,memory.used --format=csv,noheader
mark "D29 checker (clean)"
rustc --edition=2021 scripts/check_native_cuda_vendor.rs -o "$R/d29-checker"
"$R/d29-checker" --self-test > "$R/d29-selftest.txt" 2>&1
"$R/d29-checker" --repo . > "$R/d29-clean.txt" 2>&1
sha256sum "$R/d29-selftest.txt" "$R/d29-clean.txt"
build() {
  cargo build -q --release --locked --features cuda -p uor-r4-discrete --bin rowsk-qualify
  sha256sum "$CARGO_TARGET_DIR"/release/build/uor-r4-discrete-*/out/rowsk.cu \
    "$CARGO_TARGET_DIR"/release/build/uor-r4-discrete-*/out/rowsk.ptx \
    "$CARGO_TARGET_DIR"/release/build/uor-r4-discrete-*/out/rows.cu 2>/dev/null | sort -u
}
mark "build clean"
build | tee "$R/image-clean.txt"
set +e
mark "run clean"
"$CARGO_TARGET_DIR/release/rowsk-qualify" "$R/clean" > "$R/clean.stdout" 2>&1
echo "CLEAN_RC=$?"
tail -c 4000 "$R/clean.stdout"
set -e
mark "V2-neg: apply sabotage"
git apply docs/labs/bvgk-rowsk-scoring-2026-10-18/sabotage-reducer-offset.patch
"$R/d29-checker" --repo . > "$R/d29-sabotage.txt" 2>&1 || echo "D29 on sabotage rc=$?"
build | tee "$R/image-sabotage.txt"
set +e
mark "run sabotage (V2 must FAIL)"
"$CARGO_TARGET_DIR/release/rowsk-qualify" "$R/sabotage" > "$R/sabotage.stdout" 2>&1
echo "SABOTAGE_RC=$?"
grep -m3 '"v2_scoring_parity"' "$R/sabotage.stdout"
set -e
mark "revert sabotage and re-check image identity"
git apply -R docs/labs/bvgk-rowsk-scoring-2026-10-18/sabotage-reducer-offset.patch
git diff --quiet
build | tee "$R/image-reverted.txt"
if cmp -s "$R/image-clean.txt" "$R/image-reverted.txt"; then echo "IMAGE_REVERTED_EQUALS_CLEAN=yes"; else echo "IMAGE_REVERTED_EQUALS_CLEAN=no"; fi
mark "done"
echo "STAGEC_DONE"
