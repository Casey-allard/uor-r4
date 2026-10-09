#!/usr/bin/env bash
set -euo pipefail
ROOT=/workspace/uor-r4/codex/sol-bounded-pair-20261009
SRC=/root/codex/bounded-pair-src
export PATH=/root/.cargo/bin:/usr/local/cuda-12.8/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda-12.8/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export CARGO_TARGET_DIR=/root/codex/bounded-pair-target
export UOR_BUILD_SOURCE_COMMIT=a66ecef50848cc11160fc0c5a1f188355731ff98 CARGO_INCREMENTAL=0
export CUDA_COMPUTE_CAP=120 CARGO_BUILD_JOBS=2 RAYON_NUM_THREADS=2 OMP_NUM_THREADS=2
export MKL_NUM_THREADS=2 OPENBLAS_NUM_THREADS=2
mkdir "$ROOT/evidence/source-bound-build"
EVIDENCE="$ROOT/evidence/source-bound-build"
cd "$SRC"
test "$(git rev-parse HEAD)" = a66ecef50848cc11160fc0c5a1f188355731ff98
test -z "$(git status --porcelain)"
/usr/bin/time -v cargo build --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit > "$EVIDENCE/build.log" 2>&1
cp "$CARGO_TARGET_DIR/release/examples/geometric-frozen-map-fit" "$ROOT/runtime/geometric-frozen-map-fit"
sha256sum "$ROOT/runtime/geometric-frozen-map-fit" > "$EVIDENCE/binary.sha256"
/usr/bin/time -v cargo test --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit generate_episode_learning::tests -- --test-threads=2 > "$EVIDENCE/generate-tests.log" 2>&1
/usr/bin/time -v cargo test --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit prefix_fragment_learning::tests -- --test-threads=2 > "$EVIDENCE/prefix-tests.log" 2>&1
git rev-parse HEAD > "$EVIDENCE/source.commit"
sha256sum crates/uor-r4-training/examples/geometric_frozen_map_fit/{generate_episode_learning,prefix_fragment_learning}.rs > "$EVIDENCE/source-files.sha256"
printf 'BUILD_AND_FOCUSED_TESTS_PASS\n'
