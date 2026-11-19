#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 1 ]]; then
    echo "usage: $0 NEW_OUTPUT_ROOT" >&2
    exit 2
fi
source=$(cd "$(dirname "$0")" && pwd)
mkdir "$1"
out=$(realpath "$1")
cuda=/usr/local/cuda/bin
"$cuda/nvcc" --version > "$out/nvcc.txt"
rustc --version > "$out/rustc.txt"
"$cuda/nvcc" -O3 -gencode arch=compute_89,code=sm_89 -Xptxas=-v \
    "$source/device.cu" -o "$out/all-starts-sm89" > "$out/nvcc.log" 2>&1
"$cuda/cuobjdump" --list-elf "$out/all-starts-sm89" > "$out/elf.txt"
"$cuda/cuobjdump" --list-ptx "$out/all-starts-sm89" > "$out/ptx.txt"
"$cuda/cuobjdump" --dump-sass "$out/all-starts-sm89" > "$out/sass.txt"
grep -q sm_89 "$out/elf.txt"
test ! -s "$out/ptx.txt"
if grep -Ei '\b(FADD|FMUL|FFMA|HADD2?|HMUL2?|HFMA2?|DADD|DMUL|DFMA|MUFU|[HIFBD]MMA|MMA)\b' "$out/sass.txt"; then
    exit 2
fi
CARGO_TARGET_DIR="$out/target" cargo build --release --locked --offline --manifest-path "$source/Cargo.toml"
probe="$out/target/release/all-starts-aot-finite"
sha256sum "$probe" "$out/all-starts-sm89" > "$out/executables.sha256"
(cd "$source"; sha256sum Cargo.toml Cargo.lock abi.h device.cu src/main.rs kernel_core.h d11_tables.h run-wick.sh ../finite/src/lib.rs SOURCE-REVISION) > "$out/source.sha256"
lock=$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock
for run in cold-1 cold-2; do
    stage="$out/$run"
    mkdir "$stage"
    "$probe" generate "$stage/packet.bin" "$stage/rust.bin"
    (
      flock 9
      nvidia-smi --query-gpu=name,memory.free,driver_version --format=csv,noheader > "$stage/device.txt"
      test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
      /usr/bin/time -f 'wall_s=%e rss_kib=%M' -o "$stage/time.txt" \
          "$out/all-starts-sm89" "$stage/packet.bin" "$stage/cuda.bin"
    ) 9>>"$lock"
    cmp "$stage/rust.bin" "$stage/cuda.bin"
    "$probe" verify "$stage/cuda.bin" > "$stage/verification.txt"
done
for file in packet.bin rust.bin cuda.bin verification.txt; do
    cmp "$out/cold-1/$file" "$out/cold-2/$file"
done
(cd "$out"; sha256sum source.sha256 executables.sha256 elf.txt ptx.txt sass.txt cold-{1,2}/{packet.bin,rust.bin,cuda.bin,verification.txt} > SHA256SUMS && sha256sum -c SHA256SUMS)
