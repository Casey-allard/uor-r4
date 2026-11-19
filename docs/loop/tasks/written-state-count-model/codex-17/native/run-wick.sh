#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 1 ]]; then echo "usage: $0 NEW_SYNTHETIC_RECEIPT_ROOT" >&2; exit 2; fi
source=$(cd "$(dirname "$0")" && pwd)
mkdir "$1"
out=$(realpath "$1")
cuda=/usr/local/cuda/bin
"$cuda/nvcc" --version > "$out/nvcc.txt"
rustc --version > "$out/rustc.txt"
"$cuda/nvcc" -O3 -gencode arch=compute_89,code=sm_89 -Xptxas=-v "$source/device.cu" -o "$out/native-sm89" >"$out/nvcc.log" 2>&1
"$cuda/cuobjdump" --list-elf "$out/native-sm89" > "$out/elf.txt"
"$cuda/cuobjdump" --list-ptx "$out/native-sm89" > "$out/ptx.txt"
"$cuda/cuobjdump" --dump-sass "$out/native-sm89" > "$out/sass.txt"
grep -q sm_89 "$out/elf.txt"
test ! -s "$out/ptx.txt"
if grep -Ei '\b(FADD|FMUL|FFMA|HADD2?|HMUL2?|HFMA2?|DADD|DMUL|DFMA|MUFU|[HIFBD]MMA|MMA)\b' "$out/sass.txt"; then exit 2; fi
CARGO_TARGET_DIR="$out/target" cargo build --release --locked --offline --manifest-path "$source/Cargo.toml"
rust="$out/target/release/all-starts-native"
dev="$out/native-sm89"
(cd "$source"; sha256sum Cargo.toml Cargo.lock abi.h device.cu d11_tables.h src/{lib,main,packet,score}.rs run-wick.sh) > "$out/source.sha256"
sha256sum "$rust" "$dev" > "$out/executables.sha256"
"$rust" fixtures "$out/fixtures"
fixtures="$out/fixtures"
# Only synthetic fixtures above are used by this runner.
lock="$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
for run in cold-1 cold-2; do
  stage="$out/$run"
  mkdir "$stage"
  (
    flock 9
    nvidia-smi --query-gpu=name,memory.free,driver_version --format=csv,noheader > "$stage/device.txt"
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    for arm in ordered neutral position; do
      "$rust" fit "$dev" "$fixtures/right.packet" "$fixtures/right.labels.json" "$arm" "$stage/right-$arm.bank"
      "$rust" score "$dev" "$fixtures/infer.packet" "$stage/right-$arm.bank" "$stage/right-$arm.trace"
      "$rust" verify-finite right "$fixtures/infer.packet" "$stage/right-$arm.trace" "$stage/right-$arm.bank"
    done
    for name in wrong unseen; do
      "$rust" fit "$dev" "$fixtures/$name.packet" "$fixtures/$name.labels.json" ordered "$stage/$name.bank"
      "$rust" score "$dev" "$fixtures/infer.packet" "$stage/$name.bank" "$stage/$name.trace"
      "$rust" verify-finite "$name" "$fixtures/infer.packet" "$stage/$name.trace" "$stage/$name.bank"
    done
    "$rust" score "$dev" "$fixtures/tie-infer.packet" "$stage/right-ordered.bank" "$stage/tie.trace"
    "$rust" verify-finite tie "$fixtures/tie-infer.packet" "$stage/tie.trace" "$stage/right-ordered.bank"
    "$rust" decode-trace "$fixtures/infer.packet" "$stage/right-ordered.bank" "$stage/right-ordered.trace" "$stage/right-sources.json"
    "$rust" decode-trace "$fixtures/infer.packet" "$stage/wrong.bank" "$stage/wrong.trace" "$stage/wrong-sources.json"
    if "$rust" fit "$dev" "$fixtures/overflow.packet" "$fixtures/overflow.labels.json" ordered "$stage/overflow.bank"; then exit 6; fi
    test ! -e "$stage/overflow.bank"
    grep -F 'fit_status=2 used=256' "$stage/overflow.bank.work/device.log"
  ) 9>>"$lock" > "$stage/verification.txt" 2>&1
done
for path in "$out/cold-1/"{*.bank,*.trace,*-sources.json}; do
    cmp "$path" "$out/cold-2/$(basename "$path")"
done
cmp <(wc -c < "$out/cold-1/right-ordered.bank") <(wc -c < "$out/cold-1/right-neutral.bank")
cmp <(wc -c < "$out/cold-1/right-ordered.bank") <(wc -c < "$out/cold-1/right-position.bank")
(cd "$out"; sha256sum source.sha256 executables.sha256 fixtures/* cold-{1,2}/{*.bank,*.trace,*-sources.json,verification.txt} > SHA256SUMS && sha256sum -c SHA256SUMS)
