#!/usr/bin/env bash
# Ahead-of-time builds on Wick or the leased SM 8.9 research host.
set -euo pipefail
umask 077
stage=${1:?absolute stage required}
receipt=${2:?new build receipt directory required}
test "${stage:0:1}" = /
test "${receipt:0:1}" = /
cd "$stage"
mkdir "$receipt"
export PATH="$HOME/.cargo/bin:$PATH"
target="$stage/target/$(basename "$receipt")"
mkdir "$target"
src="$stage/sources"
tasks="$src/docs/loop/tasks/written-state-count-model"
cuda=/usr/local/cuda/bin
find sources -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > "$receipt/source.sha256"
rustc -V > "$receipt/rustc.version"
"$cuda/nvcc" --version > "$receipt/nvcc.version"
"$cuda/nvcc" -O3 -std=c++17 -gencode arch=compute_89,code=sm_89 -Xptxas=-v "$tasks/codex-19/operator/device.cu" -o "$stage/bin/requested-role-sm89" > "$receipt/nvcc.log" 2>&1
"$cuda/cuobjdump" --list-elf bin/requested-role-sm89 > "$receipt/elf.txt"
"$cuda/cuobjdump" --dump-sass bin/requested-role-sm89 > "$receipt/sass.txt"
grep -q sm_89 "$receipt/elf.txt"
if grep -Eiq '\b(FADD|FMUL|FFMA|FSEL|FMNMX|FSET(P)?|FCHK|FCMP|HADD2?|HMUL2?|HFMA2?|HSET(P)?2?|DADD|DMUL|DFMA|DSET(P)?|DMNMX|I2F|F2I|F2F|FRND|MUFU|[HIFBD]MMA|MMA)\b' "$receipt/sass.txt"; then exit 1; fi
if "$cuda/cuobjdump" --list-ptx bin/requested-role-sm89 > "$receipt/ptx.txt" 2> "$receipt/ptx.stderr"; then
    test ! -s "$receipt/ptx.txt"
else
    grep -Fxq 'cuobjdump info    : File contains no PTX data' "$receipt/ptx.stderr"
fi
CARGO_TARGET_DIR="$target/client" cargo build --offline --locked --release --manifest-path "$src/Cargo.toml" -p uor-r4-integer --features serving-wasm --bin uor-chat --bin pack-all-starts > "$receipt/client-build.log" 2>&1
for tool in uor-chat pack-all-starts; do
    install -m 500 "$target/client/release/$tool" "bin/$tool"
done
for package in codex-19/run codex-19/operator codex-17/regrade; do
    case "$package" in
        codex-19/run) tool=project-role ;;
        codex-19/operator) tool=requested-role-device ;;
        codex-17/regrade) tool=fresh-validity-check ;;
    esac
    CARGO_TARGET_DIR="$target/$tool" cargo build --offline --locked --release --manifest-path "$tasks/$package/Cargo.toml" >> "$receipt/tools-build.log" 2>&1
    install -m 500 "$target/$tool/release/$tool" "bin/$tool"
done
sha256sum --status --check "$receipt/source.sha256"
sha256sum bin/{uor-chat,pack-all-starts,requested-role-device,requested-role-sm89,project-role,fresh-validity-check} > "$receipt/executables.sha256"
