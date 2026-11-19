#!/usr/bin/env bash
set -euo pipefail
[[ $# == 2 ]]
source_root=$(realpath "$1")
out=$2
here="$source_root/docs/loop/tasks/written-state-count-model/codex-15/device"
mkdir "$out"
out=$(realpath "$out")
cuda=/usr/local/cuda/bin
"$cuda/nvcc" --version > "$out/nvcc.txt"
"$cuda/nvcc" -O3 -gencode arch=compute_89,code=sm_89 -Xptxas=-v     "$here/device.cu" -o "$out/ordered-role-sm89" 2> "$out/ptxas.txt"
"$cuda/cuobjdump" --list-elf "$out/ordered-role-sm89" > "$out/device-elf.txt"
"$cuda/cuobjdump" --list-ptx "$out/ordered-role-sm89" > "$out/device-ptx.txt"
"$cuda/cuobjdump" --dump-sass "$out/ordered-role-sm89" > "$out/device-sass.txt"
grep -q 'sm_89' "$out/device-elf.txt"
test ! -s "$out/device-ptx.txt"
if grep -Ei '\b(FADD|FMUL|FFMA|HADD2?|HMUL2?|HFMA2?|DADD|DMUL|DFMA|MUFU|[HIFBD]MMA|MMA)\b' "$out/device-sass.txt"; then
    exit 2
fi
CARGO_TARGET_DIR="$out/target" cargo build --locked --release --manifest-path "$here/Cargo.toml"
cp "$out/target/release/ordered-role-device" "$out/ordered-role-device"
(cd "$out"; sha256sum ordered-role-device ordered-role-sm89 device-{elf,ptx,sass}.txt > build.sha256)
(cd "$source_root"; sha256sum     docs/loop/tasks/written-state-count-model/codex-15/device/{device.cu,driver.rs,Cargo.toml,Cargo.lock,build-wick.sh,finite-wick.sh,capture-wick.sh}     crates/uor-r4-integer/src/{ordered_role.rs,paired_role.rs,paired_copy.rs,discrete_d11.rs}     > "$out/sources.sha256")
