#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 4 ]]; then exit 2; fi
base=$(realpath "$1")
key=$(realpath "$2")
program=$(realpath "$3")
out=$4
mkdir "$out"
out=$(realpath "$out")
cd "$base"
printf '%s  %s\n' 220ec3c5fd0c839bed864c051458b30f95668cc1b7a26c725d646d189fdc8096 "$key" | sha256sum --strict -c -
sha256sum --strict -c captures.sha256 > "$out/checked.log"
for id in $(jq -r '.[]|select(.stratum=="crossed")|.row' "$key"); do
    jq -ce --argjson id "$id" '{row:$id,result:{prompt_ids,role_replay}}' "full-repeated/$id.json"
done > "$out/rows.jsonl"
jq -se --slurpfile key "$key" -f "$program" "$out/rows.jsonl" > "$out/result.json"
