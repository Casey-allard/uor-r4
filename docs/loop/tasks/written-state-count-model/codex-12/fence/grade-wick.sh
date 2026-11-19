#!/usr/bin/env bash
set -euo pipefail
top=$HOME/uor-codex
cd "$top/20261118-paired-role-fence/fence"
p=$top/20261118-content-support/panels
old=$top/20261118-role-boundary-count
new=$top/20261118-paired-role-fence/role/panels
out=grades
mkdir "$out"
grade() {
    local name=$1 control=$2 candidate=$3 base=$4
    jq -en --slurpfile b "$base-blind.json" --slurpfile g "$base-grade.json" '$b[0]==[$g[0][]|{prefix}]' >/dev/null
    for mode in summary rows; do
        jq -ce --arg mode "$mode" --slurpfile old "$control" --slurpfile grade "$base-grade.json" \
            -f grade.jq "$candidate" > "$out/$name-$mode.json"
    done
}
for n in 1 2; do
    for split in held unseen transfer; do
        case "$split" in
            held) pref=$p/held ;;
            unseen) pref=$old/panels/unseen ;;
            transfer) pref=$top/20261118-continuation-eos/panels/transfer ;;
        esac
        for arm in query both; do
            tag=seed$n-$split-$arm
            grade "$tag" "$p/results/writer/$split/$arm/seed$n-primary.json" \
                "results/primary/historical/$tag-writer.json" "$pref"
        done
    done
    for arm in query-agree agree; do
        tag=seed$n-held-$arm
        grade "$tag" "$p/results/writer/held/$arm/seed$n-primary.json" \
            "results/primary/historical/$tag-writer.json" "$p/held"
    done
    if [[ -s results/primary/fresh-oracle/SHA256SUMS ]]; then
        tag=seed$n-fresh-oracle
        grade "$tag" "results/primary/fresh-oracle/$tag-old-writer.json" \
            "results/primary/fresh-oracle/$tag-new-writer.json" "$new/fresh"
    fi
done
(cd "$out"; sha256sum *.json > SHA256SUMS)
