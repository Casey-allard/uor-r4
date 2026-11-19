#!/usr/bin/env bash
# Under the unit's count root, after staging reviewed source in review-run/src.
set -euo pipefail
cd review-run
export PATH="$HOME/.cargo/bin:/usr/local/cuda/bin:$PATH"
export CUDA_PATH=/usr/local/cuda CUDA_HOME=/usr/local/cuda UOR_NVCC_ARCH=compute_89 CARGO_BUILD_JOBS=3 CARGO_TARGET_DIR="$PWD/target" LD_LIBRARY_PATH=/usr/local/cuda/lib64
nvcc -O3 -arch=sm_89 -std=c++17 -o count-score-device src/docs/labs/kn-reference/instrument/src/bin/count-score.cu
cargo build --release --locked --manifest-path src/docs/labs/kn-reference/instrument/Cargo.toml --features count-replay --bin count-score > build.log 2>&1
bash ../run-wick.sh
for part in run finite; do
  for image in full.d11 pruned.d11 full-em.u32 pruned-em.u32 fit.u16 query.u16; do
    cmp "../$part/$image" "$part/$image"
    echo "PASS $part/$image"
  done
  for artifact in "$part"/*.gpu "$part"/*.words; do
    cmp "../$artifact" "$artifact"
    echo "PASS $artifact"
  done
done > receipt/identical-tapes.txt
for scientific in result finite; do
  jq 'del(.scores[].wall_including_host_parity_ms)' "../receipt/$scientific.json" > "$scientific-reference.json"
  jq 'del(.scores[].wall_including_host_parity_ms)' "receipt/$scientific.json" > "$scientific-repeat.json"
  cmp "$scientific-reference.json" "$scientific-repeat.json"
  sha256sum "$scientific-reference.json" "$scientific-repeat.json" >> receipt/scientific.sha256
done
