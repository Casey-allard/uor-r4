#!/usr/bin/env bash
set -euo pipefail
# Run only after the final FIT and pre-run review for these source and data pins.
[[ $# == 5 ]]
build=$(realpath "$1")
assets=$(realpath "$2")
panel=$(realpath "$3")
panel_manifest=$(realpath "$4")
out=$5
mkdir "$out"
out=$(realpath "$out")
(cd "$build"; sha256sum -c build.sha256)
(cd "$assets"; sha256sum -c SHA256SUMS)
(cd "$panel"; sha256sum -c "$panel_manifest")
cp "$panel_manifest" "$out/panels.sha256"
cp "$assets/SHA256SUMS" "$out/assets.sha256"
cp "$build/"{build,sources}.sha256 "$out/"
nvidia-smi --query-gpu=name,uuid,driver_version --format=csv > "$out/device.txt"

driver="$build/ordered-role-device"
kernel="$build/ordered-role-sm89"
exec 9>>"$HOME/uor-codex/20261118-diagnostics/wick-codex-gpu.lock"
measure() {
    local entry=$1
    shift
    /usr/bin/time -f 'wall_seconds=%e max_host_kib=%M' -o "$entry.time" "$@" \
        > "$entry.stdout" 2> "$entry.stderr"
}
device() {
    flock 9
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    measure "$@"
    flock -u 9
}

for seed in 1 2; do
    model="$assets/seed$seed-paired.d11"
    tokenizer="$assets/tokenizer-seed$seed.json"
    for mode in identity positional; do
        arm="$out/seed$seed-$mode"
        mkdir "$arm"
        device "$arm/signature-full" "$driver" signature "$kernel" "$mode" "$model" "$tokenizer" \
            "$panel/held-blind.json" all "$arm/signature-full.json"
        device "$arm/signature-bounded" "$driver" signature "$kernel" "$mode" "$model" "$tokenizer" \
            "$panel/held-blind.json" "$panel/held-bounds.json" "$arm/signature-bounded.json"
        device "$arm/fit" "$driver" fit "$kernel" "$mode" "$model" "$tokenizer" \
            "$panel/train-blind.json" "$panel/train-grades.json" "$arm/train"
        bank="$arm/train/bank.role"
        train_receipt="$arm/train/train.receipt"
        device "$arm/public" "$driver" public "$kernel" "$model" "$tokenizer" "$bank" \
            "$train_receipt" "$panel/held-blind.json" "$arm/public.json"
        device "$arm/bounded" "$driver" bounded "$kernel" "$model" "$tokenizer" "$bank" \
            "$train_receipt" "$panel/held-blind.json" "$panel/held-bounds.json" "$arm/bounded.json"
        device "$arm/fixed" "$driver" fixed "$kernel" "$arm/bounded.json" "$panel/held-match.json" "$arm/fixed.json"
        measure "$arm/eligibility" "$driver" eligibility "$arm/public.json" "$arm/bounded.json" "$arm/eligibility.json"
    done
done
(cd "$out"; sha256sum seed*/*.json seed*/train/{bank.role,train.json,train.receipt} > deterministic.sha256)
