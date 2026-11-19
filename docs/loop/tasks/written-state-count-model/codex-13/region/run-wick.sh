#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 && "$1" =~ ^[A-Za-z0-9-]+$ ]]
out=$1
sha256sum -c panels/collection.sha256
(cd baseline; sha256sum -c retained.sha256)
sha256sum -c frozen-binaries-inputs.sha256
mkdir "$out"

base=$HOME/uor-codex
input=$base/20261118-value-connectors/inputs
exec 9>>"$base/20261118-diagnostics/wick-codex-gpu.lock"
device() {
    flock 9
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    "$@"
    flock -u 9
}
nvidia-smi --query-gpu=name,uuid,driver_version --format=csv > "$out/device.txt"
date -u -Iseconds > "$out/start.txt"
device bin/query-region-device finite bin/region-kernel > "$out/finite.txt"
for seed in 1 2; do
    model=$input/seed$seed.d11
    bank=baseline/seed$seed.role
    tokenizer=$input/tokenizer.json
    [[ $seed == 1 ]] || tokenizer=$input/tokenizer-seed-2.json
    for split in held fresh nuisance; do
        file="$out/seed$seed-$split"
        tape="panels/$split-blind.json"
        device bin/paired-role-device blind bin/device "$model" "$tokenizer" "$bank" "$tape" \
            "$file-original.json" > "$file-original.txt"
        if [[ $split != nuisance ]]; then
            cmp "$file-original.json" "baseline/seed$seed-$split.json"
        fi
        for arm in identity region adverse; do
            case $arm in
                identity) mask=all ;;
                region) mask="panels/$split-regions.json" ;;
                adverse) mask="panels/$split-adverse.json" ;;
            esac
            device bin/query-region-device bin/region-kernel "$model" "$tokenizer" "$bank" "$tape" "$mask" \
                "$file-$arm.json" > "$file-$arm.txt"
        done
        cmp "$file-original.json" "$file-identity.json"
        jq -ce --slurpfile grade "panels/$split-grade.json" '
            .role_intervention="oracle-original-requested-endpoints" |
            .rows |= (to_entries | map(.key as $i | .value |
                .query_start=$grade[0][$i].query_start |
                .query_end=$grade[0][$i].query_end |
                .words[0]=(.query_start // 4294967295) |
                .words[1]=(.query_end // 4294967295)))' \
            "$file-original.json" > "$file-role-oracle.json"
    done
done
if bin/query-region-device bin/region-kernel "$input/seed2.d11" "$input/tokenizer-seed-2.json" \
    baseline/seed1.role panels/held-blind.json all "$out/rejected.json" > "$out/rejection.txt" 2>&1; then
    echo 'cross-seed bank was accepted' >&2
    exit 2
fi
grep -q 'role bank model/tokenizer binding differs' "$out/rejection.txt"
(cd "$out"; sha256sum seed*.json > decisions.sha256)
date -u -Iseconds > "$out/end.txt"
