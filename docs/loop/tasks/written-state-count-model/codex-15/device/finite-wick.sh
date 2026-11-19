#!/usr/bin/env bash
set -euo pipefail
[[ $# == 3 ]]
build=$(realpath "$1")
assets=$(realpath "$2")
out=$3
mkdir "$out"
(cd "$build"; sha256sum -c build.sha256)
(cd "$assets"; sha256sum -c SHA256SUMS)
nvidia-smi --query-gpu=name,uuid,driver_version --format=csv > "$out/device.txt"
for seed in 1 2; do
    /usr/bin/time -f 'wall_seconds=%e max_host_kib=%M' -o "$out/model-seed$seed.time" \
        "$build/ordered-role-device" model-check "$assets/seed$seed-paired.d11" \
        "$assets/tokenizer-seed$seed.json" 2> "$out/model-seed$seed.log"
done
exec 9>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock 9
nvidia-smi --query-gpu=memory.total,memory.free --format=csv,noheader,nounits > "$out/memory-before.txt"
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
/usr/bin/time -f 'wall_seconds=%e max_host_kib=%M' -o "$out/finite.time" \
    "$build/ordered-role-device" finite "$build/ordered-role-sm89" "$out/finite.json" \
    > "$out/finite.log" 2>&1
for n in 1 32; do
    /usr/bin/time -f 'wall_seconds=%e max_host_kib=%M' -o "$out/timing-$n.time" \
        "$build/ordered-role-device" timing "$build/ordered-role-sm89" "$n" \
        > "$out/timing-$n.log" 2>&1
done
nvidia-smi --query-gpu=memory.total,memory.free --format=csv,noheader,nounits > "$out/memory-after.txt"
flock -u 9
(cd "$out"; sha256sum *.txt *.time *.log *.json > finite.sha256)
