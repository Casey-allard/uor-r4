#!/usr/bin/env bash
set -euo pipefail
top=$HOME/uor-codex
cd "$top/20261118-paired-role-fence/fence"
d=results/final-verification
mkdir "$d"
prior=$top/20261118-content-support
old=$top/20261118-role-boundary-count
role=$top/20261118-paired-role-fence/role
exec 8>"$top/20261118-diagnostics/wick-codex-gpu.lock"
verify() {
    local bare=$1 input=$2 historical=$3 output=$4
    flock 8
    test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 4096
    bin/assertion-fence-final apply bin/fence-device "$model" "$span" "$exact" "$bare" "$input" "$output"
    flock -u 8
    cmp "$historical" "$output"
}
for n in 1 2; do
    model=$top/20261118-value-connectors/inputs/seed$n.d11
    span=$top/20261118-span-connectors/results/span-01/seed$n-state.json
    exact=$top/20261118-value-connectors/results/value-01/seed$n-state.json
    for split in held unseen transfer; do
        case "$split" in
            held) bare=$prior/panels/held-blind.json ;;
            unseen) bare=$old/panels/unseen-blind.json ;;
            transfer) bare=$top/20261118-continuation-eos/panels/transfer-blind.json ;;
        esac
        for arm in both query; do
            tag=seed$n-$split-$arm
            verify "$bare" "$prior/nominee/results/seed$n-$split/$arm-continuation.json" \
                "results/primary/historical/$tag-trace.json" "$d/$tag-trace.json"
        done
    done
    for arm in agree query-agree; do
        bare=$prior/panels/held-blind.json
        tag=seed$n-held-$arm
        for source in "$prior/nominee/agreement/results/$tag"/rank-*-cont.json; do
            name=${source##*/}
            verify "$bare" "$source" "results/primary/historical/$tag/$name" "$d/$tag-$name"
        done
    done
    for split in held fresh; do
        bare=$role/panels/$split-blind.json
        tag=seed$n-$split
        parent=results/primary/prior-nominee/$tag
        for source in "$parent"/old/rank-*-cont.json; do
            name=${source##*/}
            verify "$bare" "$source" "$parent/fenced/$name" "$d/$tag-prior-nominee-$name"
        done
    done
done
(cd "$d"; sha256sum *.json > SHA256SUMS)
