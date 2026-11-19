#!/usr/bin/env bash
# Keep the pinned public client oracle; shard only independent client processes.
set -euo pipefail
umask 077
[[ $# == 7 ]]
writer=$1; chat=$2; artifact=$3; tokenizer=$4; blind=$5; arm=$6; target=$7
[[ "$arm" == public || "$arm" == no-read ]]
source "$(dirname "$0")/../../codex-19/run/common.sh"
count=$(jq -er 'length | select(. > 0 and . < 10000)' "$blind")
batch=$(((count+3)/4))
mkdir "$target" "$target/shards"
pids=()
for ((start=0; start<count; start+=batch)); do
    shard="$target/shards/$start"
    jq -c --argjson start "$start" --argjson batch "$batch" '.[$start:($start+$batch)]' "$blind" > "$shard.json"
    RAYON_NUM_THREADS=1 "$writer" public "$chat" "$artifact" "$tokenizer" "$shard.json" "$arm" "$shard" &
    pids+=("$!")
done
failed=0
for pid in "${pids[@]}"; do
    wait "$pid" || failed=1
done
[[ "$failed" == 0 ]]
seals=()
for ((start=0; start<count; start+=batch)); do
    shard="$target/shards/$start"
    seals+=("$shard/seal.json")
    end=$((start+batch))
    if ((end>count)); then end=$count; fi
    jq -e --arg blind "$(sum "$shard.json")" --argjson count "$((end-start))" \
        '.blind==$blind and (.row_sha256|length)==$count' "$shard/seal.json" >/dev/null
    for ((row=start; row<end; row++)); do
        printf -v local_row 'row-%04d.json' "$((row-start))"
        printf -v global_row 'row-%04d.json' "$row"
        ln "$shard/$local_row" "$target/$global_row"
    done
done
# Match serde_json's sorted compact object, which the original oracle writes without a newline.
jq -csj --arg blind "$(sum "$blind")" '
    .[0] as $first
    | if all(.[]; del(.blind,.row_sha256)==($first|del(.blind,.row_sha256)))
      then $first + {blind:$blind,row_sha256:map(.row_sha256)|add}
      else error("ordinary shards disagree on the executable, model, tokenizer or arm") end
' "${seals[@]}" > "$target/seal.json"
