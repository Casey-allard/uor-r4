#!/usr/bin/env bash
set -euo pipefail
[[ $# == 2 && "$1" =~ ^[A-Za-z0-9-]+$ && "$2" =~ ^[A-Za-z0-9-]+$ ]]
decisions=$1
out=$2
p=docs/loop/tasks/written-state-count-model/codex-13/region
(cd "$decisions"; sha256sum -c decisions.sha256)
sha256sum -c panels/collection.sha256
mkdir "$out"
printf '[]\n' > "$out/empty.json"
for seed in 1 2; do
    for split in held fresh nuisance; do
        file="seed$seed-$split"
        cohort="panels/$file-intrusions.json"
        [[ $split != nuisance ]] || cohort="$out/empty.json"
        for arm in region adverse role-oracle; do
            raw="$out/$file-$arm-rows.json"
            jq -en --slurpfile gold "panels/$split-grade.json" \
                --slurpfile region "panels/$split-regions.json" --slurpfile cohort "$cohort" \
                --slurpfile old "$decisions/$file-original.json" --slurpfile test "$decisions/$file-$arm.json" \
                -f "$p/grade.jq" > "$raw"
            jq -e -f "$p/summarize.jq" "$raw" > "$out/$file-$arm-summary.json"
        done
        jq -e '.total.lost==0 and .total.retained_excluded_lost==0' \
            "$out/$file-region-summary.json" >/dev/null
    done
done
(cd "$out"; sha256sum seed*.json > grades.sha256)
