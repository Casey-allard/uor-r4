#!/usr/bin/env bash
set -euo pipefail
stage=${1:?sealed stage}
receipt=${2:?new build receipt}
tasks="$stage/sources/docs/loop/tasks/written-state-count-model"
bash "$tasks/codex-19/dispatch/build.sh" "$stage" "$receipt"
cd "$stage"
export PATH="$HOME/.cargo/bin:$PATH"
cuda=/usr/local/cuda/bin
"$cuda/nvcc" -O3 -gencode arch=compute_89,code=sm_89 -Xptxas=-v "$tasks/codex-17/native/device.cu" -o bin/earlier-veto-sm89 > "$receipt/veto-nvcc.log" 2>&1
"$cuda/cuobjdump" --list-elf bin/earlier-veto-sm89 > "$receipt/veto-elf.txt"
grep -q sm_89 "$receipt/veto-elf.txt"
"$cuda/cuobjdump" --dump-sass bin/earlier-veto-sm89 > "$receipt/veto-sass.txt"
if grep -Eiq '\b(FADD|FMUL|FFMA|FSEL|FMNMX|FSET(P)?|FCHK|FCMP|HADD2?|HMUL2?|HFMA2?|HSET(P)?2?|DADD|DMUL|DFMA|DSET(P)?|DMNMX|I2F|F2I|F2F|FRND|MUFU|[HIFBD]MMA|MMA)\b' "$receipt/veto-sass.txt"; then exit 1; fi
if "$cuda/cuobjdump" --list-ptx bin/earlier-veto-sm89 > "$receipt/veto-ptx.txt" 2> "$receipt/veto-ptx.stderr"; then
    test ! -s "$receipt/veto-ptx.txt"
else
    grep -Fxq 'cuobjdump info    : File contains no PTX data' "$receipt/veto-ptx.stderr"
fi
target="$stage/target/$(basename "$receipt")/veto"
CARGO_TARGET_DIR="$target" cargo build --release --offline --locked --manifest-path "$tasks/codex-17/native/Cargo.toml" > "$receipt/veto-rust.log" 2>&1
install -m 500 "$target/release/all-starts-native" bin/all-starts-native
sha256sum bin/{all-starts-native,earlier-veto-sm89} >> "$receipt/executables.sha256"
sha256sum --status --check "$receipt/source.sha256"
