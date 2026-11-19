#!/usr/bin/env bash
set -euo pipefail
umask 077
[[ $# == 1 ]] || { echo "usage: build-wick.sh ABS_STAGE" >&2; exit 2; }
stage=$(realpath "$1")
src="$stage/sources"
prior="$src/docs/loop/tasks/written-state-count-model/codex-17"
cuda=/usr/local/cuda/bin
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR="$stage/target"
cd "$stage"
mkdir build-receipt
find sources -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > build-receipt/source.sha256
"$cuda/nvcc" --version > build-receipt/nvcc.txt
rustc --version > build-receipt/rustc.txt
"$cuda/nvcc" -O3 -gencode arch=compute_89,code=sm_89 -Xptxas=-v \
    -I "$prior/device" "$prior/native/device.cu" -o bin/native-sm89 > build-receipt/nvcc.log 2>&1 &
nvcc_pid=$!
cargo build --locked --offline --release --manifest-path "$src/Cargo.toml" \
    -p uor-r4-integer --features serving-wasm --bin uor-chat --bin pack-all-starts > build-receipt/client-build.log 2>&1
cargo build --locked --offline --release --manifest-path "$prior/native/Cargo.toml" > build-receipt/native-build.log 2>&1
wait "$nvcc_pid"
for tool in uor-chat pack-all-starts all-starts-native; do
    install -m 500 "$CARGO_TARGET_DIR/release/$tool" "bin/$tool"
done
"$cuda/cuobjdump" --list-elf bin/native-sm89 > build-receipt/elf.txt
if "$cuda/cuobjdump" --list-ptx bin/native-sm89 > build-receipt/ptx.txt 2> build-receipt/ptx.stderr; then
    test ! -s build-receipt/ptx.txt
else
    grep -Fxq 'cuobjdump info    : File contains no PTX data' build-receipt/ptx.stderr
fi
"$cuda/cuobjdump" --dump-sass bin/native-sm89 > build-receipt/sass.txt
grep -q sm_89 build-receipt/elf.txt
if grep -Ei '\b(FADD|FMUL|FFMA|HADD2?|HMUL2?|HFMA2?|DADD|DMUL|DFMA|MUFU|[HIFBD]MMA|MMA)\b' build-receipt/sass.txt; then
    exit 2
fi
sha256sum --status --check build-receipt/source.sha256
sha256sum bin/{uor-chat,pack-all-starts,all-starts-native,native-sm89} > build-receipt/executables.sha256
