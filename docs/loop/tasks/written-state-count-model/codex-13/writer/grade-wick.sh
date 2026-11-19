#!/usr/bin/env bash
set -euo pipefail
root=${1:?writer output root}
grade=${2:?grade tape}
split=${3:?held, fresh or nuisance}
script=$(dirname "$0")
jq -cen --slurpfile a "$root/original/chosen.json" --slurpfile grade "$grade" \
    --slurpfile region "$root/region/chosen.json" --slurpfile adverse "$root/adverse/chosen.json" \
    --slurpfile oracle "$root/role-oracle/chosen.json" -f "$script/grade.jq" > "$root/grade.json"
if [[ $split != nuisance ]]; then
    exact=308
    [[ $split != fresh ]] || exact=224
    jq -e --argjson exact "$exact" '.overall.original | .exact==$exact and .answerable==448 and .unsupported==608 and .unsupported_copy==0' \
        "$root/grade.json" >/dev/null
fi
