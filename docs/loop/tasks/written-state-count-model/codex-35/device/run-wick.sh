#!/usr/bin/env bash
set -euo pipefail
umask 077
phase=${1:?finite, fit, or score}
case "$phase" in
    finite|fit) test "$#" -eq 5 ;;
    score) test "$#" -eq 6 ;;
    *) exit 2 ;;
esac
test "$(hostname -s)" = wick
stage=$(cd -- "$(dirname -- "$0")" && pwd -P)
packet=$(realpath -- "${2:?packet folder}")
admission=$(realpath -- "${3:?reviewed authorization manifest}")
approved_sha=${4:?exact authorized manifest SHA-256}
output=${5:?fresh output directory}
gpu=GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41

printf '%s  %s\n' "$approved_sha" "$admission" | sha256sum --strict -c -
admitted_dir=$(dirname -- "$admission")
(cd -- "$admitted_dir" && sha256sum --strict -c -- "$(basename -- "$admission")")
require_manifest() {
    local manifest=$1 folder name hash
    folder=$(dirname -- "$manifest")
    name=$(realpath --relative-to="$admitted_dir" -- "$manifest")
    hash=$(sha256sum -- "$manifest")
    grep -Fxq -- "${hash%% *}  $name" "$admission"
    (cd -- "$folder" && sha256sum --strict -c -- "$(basename -- "$manifest")")
}
require_payload() {
    local manifest=$1 name=$2 hash
    hash=$(cd -- "$(dirname -- "$manifest")" && sha256sum -- "$name")
    grep -Fxq -- "$hash" "$manifest"
}
require_manifest "$stage/stage.sha256"
require_payload "$stage/stage.sha256" paired-value-rank
geometries=(fact-prefix candidate-prefix)
if [[ "$phase" == finite ]]; then
    require_manifest "$packet/packets.sha256"
    for name in train.bin ordinary.bin bank.expected ordinary.expected                 bank.corrupt ordinary.corrupt ordinary.changed; do
        require_payload "$packet/packets.sha256" "$name"
    done
else
    for geometry in "${geometries[@]}"; do
        manifest=$packet/$geometry/packets.sha256
        require_manifest "$manifest"
        if [[ "$phase" == fit ]]; then
            require_payload "$manifest" train.bin
        else
            require_payload "$manifest" ordinary.bin
            require_payload "$manifest" diagnostic.bin
        fi
    done
fi
if [[ "$phase" == score ]]; then
    bank_root=$(realpath -- "${6:?admitted fitting output directory}")
    require_manifest "$bank_root/results.sha256"
    for geometry in "${geometries[@]}"; do
        for pass in primary cold; do
            require_payload "$bank_root/results.sha256" "$geometry/$pass/bank.bin"
        done
        cmp "$bank_root/$geometry/primary/bank.bin" "$bank_root/$geometry/cold/bank.bin"
    done
    # Both conditions use every TRAIN pair in original order, including duplicates.
    test "$(wc -c < "$bank_root/fact-prefix/primary/bank.bin")" \
        -eq "$(wc -c < "$bank_root/candidate-prefix/primary/bank.bin")"
fi

test -f "$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
exec 8>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
flock -n 8
test "$(nvidia-smi --query-gpu=uuid --format=csv,noheader)" = "$gpu"
test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 8.9
check_idle() {
    local processes
    processes=$(nvidia-smi --query-compute-apps=pid --format=csv,noheader)
    test -z "$processes"
}
check_idle
mkdir -- "$output"
output=$(realpath -- "$output")
nvidia-smi --query-gpu=uuid,name,driver_version,compute_cap,memory.free \
    --format=csv > "$output/device.csv"
cp -- "$admission" "$output/admission.sha256"
export CUDA_VISIBLE_DEVICES="$gpu" CUDA_DISABLE_PTX_JIT=1

device="$stage/paired-value-rank"
launch() {
    local input=$1 result=$2
    shift 2
    check_idle
    { time "$device" "$@" < "$input" > "$result"; } 2> "$result.time"
}
reject() {
    local input=$1 control_output=$2 expected_error=$3 control_status=0
    shift 3
    "$device" "$@" < "$input" > "$control_output.out" 2> "$control_output.error" ||
        control_status=$?
    test "$control_status" -eq 2
    test ! -s "$control_output.out"
    grep -Fxq "$expected_error" "$control_output.error"
}

if [[ "$phase" == finite ]]; then
    for pass in primary cold; do
        destination=$output/$pass
        mkdir -- "$destination"
        launch "$packet/train.bin" "$destination/bank.bin"
        cmp -- "$packet/bank.expected" "$destination/bank.bin"
        launch "$packet/ordinary.bin" "$destination/ordinary.out" "$destination/bank.bin"
        cmp -- "$packet/ordinary.expected" "$destination/ordinary.out"
    done
    cmp -- "$output/primary/bank.bin" "$output/cold/bank.bin"
    cmp -- "$output/primary/ordinary.out" "$output/cold/ordinary.out"
    reject "$packet/ordinary.bin" "$output/corrupt-bank" \
        "prototype outside native coordinate difference ranges" "$packet/bank.corrupt"
    reject "$packet/ordinary.corrupt" "$output/corrupt-score" \
        "invalid frozen singleton choice" "$output/primary/bank.bin"
    if cmp -s -- "$packet/ordinary.changed" "$output/primary/ordinary.out"; then
        echo "Finite comparison accepted a changed winner." >&2
        exit 4
    fi
else
    for geometry in "${geometries[@]}"; do
        for pass in primary cold; do
            destination="$output/$geometry/$pass"
            mkdir -p -- "$destination"
            if [[ "$phase" == fit ]]; then
                launch "$packet/$geometry/train.bin" "$destination/bank.bin"
            else
                for arm in ordinary diagnostic; do
                    launch "$packet/$geometry/$arm.bin" "$destination/$arm.out" \
                        "$bank_root/$geometry/$pass/bank.bin"
                done
            fi
        done
        if [[ "$phase" == fit ]]; then
            cmp -- "$output/$geometry/primary/bank.bin" "$output/$geometry/cold/bank.bin"
        else
            for arm in ordinary diagnostic; do
                cmp -- "$output/$geometry/primary/$arm.out" "$output/$geometry/cold/$arm.out"
            done
        fi
    done
fi
(
    cd -- "$output"
    find . -type f ! -name results.sha256 -printf '%P\0' |
        LC_ALL=C sort -z | xargs -0 sha256sum -- > results.sha256
    sha256sum results.sha256
)
