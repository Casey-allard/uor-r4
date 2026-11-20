#!/usr/bin/env bash
set -euo pipefail
umask 077
phase=${1:?finite or exposed}
case "$phase" in
    finite) rows=16 ;;
    exposed) rows=22 ;;
    *) exit 2 ;;
esac
test "$(hostname -s)" = wick
stage=$(cd -- "$(dirname -- "$0")" && pwd -P)
packet=$(realpath -- "${2:?packet folder}")
admission=$(realpath -- "${3:?reviewed execution authorization manifest}")
approved_sha=${4:?exact authorized manifest SHA-256}
output=${5:?fresh output directory}
gpu=GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41

printf '%s  %s\n' "$approved_sha" "$admission" | sha256sum --strict -c -
admitted_dir=$(dirname -- "$admission")
(
    cd -- "$admitted_dir"
    sha256sum --strict -c -- "$(basename -- "$admission")"
)
for required in "$stage/stage.sha256" "$packet/packets.sha256"; do
    name=$(realpath --relative-to="$admitted_dir" -- "$required")
    hash=$(sha256sum -- "$required")
    grep -Fxq -- "${hash%% *}  $name" "$admission"
done
(cd -- "$stage" && sha256sum --strict -c stage.sha256)
(cd -- "$packet" && sha256sum --strict -c packets.sha256)

test -f "$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -n 8
test "$(nvidia-smi --query-gpu=uuid --format=csv,noheader)" = "$gpu"
test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 8.9
test -z "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)"

mkdir -- "$output"
output=$(realpath -- "$output")
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap,memory.free \
    --format=csv > "$output/device.csv"
cp -- "$admission" "$output/admission.sha256"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1
for pass in primary cold; do
    mkdir -- "$output/$pass"
    for mode in 7 8; do
        test -z "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)"
        {
            time "$stage/detached-endpoint" < "$packet/mode-$mode.bin" \
                > "$output/$pass/mode-$mode.out"
        } 2> "$output/$pass/mode-$mode.time"
        test "$(wc -c < "$output/$pass/mode-$mode.out")" -eq "$((rows * 3872))"
    done
done
for mode in 7 8; do
    cmp -- "$output/primary/mode-$mode.out" "$output/cold/mode-$mode.out"
    if [[ "$phase" == finite ]]; then
        cmp -- "$packet/mode-$mode.expected" "$output/primary/mode-$mode.out"
    fi
done
if [[ "$phase" == finite ]]; then
    # Change the first byte of the known-positive mode-8 query start, 7 -> 0.
    cp -- "$packet/mode-8.expected" "$output/changed-endpoint.bin"
    printf '\000' | dd of="$output/changed-endpoint.bin" bs=1 count=1 conv=notrunc status=none
    if cmp -s -- "$output/changed-endpoint.bin" "$output/primary/mode-8.out"; then
        echo "Finite comparator accepted a deliberately changed endpoint." >&2
        exit 4
    fi
fi
(
    cd -- "$output"
    sha256sum -- admission.sha256 device.csv primary/* cold/* \
        $(if [[ "$phase" == finite ]]; then echo changed-endpoint.bin; fi) > results.sha256
    sha256sum results.sha256
)
