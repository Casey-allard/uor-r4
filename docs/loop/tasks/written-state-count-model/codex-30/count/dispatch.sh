#!/usr/bin/env bash
set -euo pipefail
umask 077

cd /workspace/count30
case "${1:?build or run}" in
build)
  sha256sum -c stage.sha256
  mkdir source
  tar -xf build-base-v2.tar.gz -C source
  tar -xf overlay-v2.tar -C source
  rustc --version > rustc.txt
  nvcc --version > nvcc.txt
  nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap --format=csv > device.csv
  test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 12.0
  UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR="$PWD/target" cargo build --locked --release \
    --manifest-path source/docs/labs/kn-reference/instrument/Cargo.toml \
    --features cuda --bin compact-count
  ldd target/release/compact-count > linked.txt
  if grep -E 'libnvrtc|libcublas' linked.txt; then exit 1; fi
  find target/release/build -path '*/out/ngram-count.cubin' -print -quit > cubin.path
  cubin=$(cat cubin.path)
  test -n "$cubin"
  cuobjdump -lelf "$cubin" > cubin-elf.txt
  cuobjdump -lptx "$cubin" > cubin-ptx.txt
  if grep -E '\.ptx([[:space:]]|$)' cubin-ptx.txt; then exit 1; fi
  sha256sum target/release/compact-count "$cubin" > build.sha256
  ;;
run)
  sha256sum -c stage.sha256
  sha256sum -c build.sha256
  sha256sum -c GO.sha256
  grep -q '^TEST FITNESS: FIT' FIT.md
  mkdir runs
  for trial in 1 2; do
    target/release/compact-count c128.uort seed1.d11 "runs/r$trial" > "runs/run-$trial.log"
  done
  for result in receipt.json baseline.counts compact.counts; do
    cmp "runs/r1/$result" "runs/r2/$result"
  done
  sha256sum runs/r{1,2}/* > results.sha256
  ;;
*) exit 2 ;;
esac
