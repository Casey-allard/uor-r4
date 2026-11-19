#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261119-value-ranking"
root=$HOME/uor-codex
old=$root/20261118-ordered-role-transfer
mkdir specs
for seed in 1 2; do
    for split in fit held; do
        paths=(
            "base=$root/20261118-value-connectors/inputs/seed$seed.d11"
            "paired=$old/assets/seed$seed-paired.d11"
            "span=$root/20261118-span-connectors/results/span-01/seed$seed-state.json"
            "writer=$old/bins/writer-c" "capture=$PWD/bin/uor-value-rank-capture"
            "blind=$PWD/endpoints/$split-seed$seed/blind.json"
            "resolved=$PWD/endpoints/$split-seed$seed/resolved-requested.json"
        )
        for pair in "${paths[@]}"; do
            path=${pair#*=}
            jq -nc --arg k "${pair%%=*}" --arg path "$path" --arg hash "$(sha256sum "$path" | cut -d' ' -f1)" '{key:$k,value:{path:$path,sha256:$hash}}'
        done | jq -s --arg root "$root" --argjson seed "$seed" '{schema:"uor-r4.requested-capture-stage/1",seed:$seed,wick_root:$root,inputs:from_entries}' > "specs/$split-seed$seed.json"
    done
done
