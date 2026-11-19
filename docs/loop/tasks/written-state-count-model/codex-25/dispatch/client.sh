#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 7 ]]; then echo "usage: client.sh CHAT TOKENIZER PANEL MODEL MODE ROW NEW_OUTPUT"; exit 2; fi
chat=$1 tok=$2 panel=$3 model=$4 mode=$5 row=$6 out=$7
test ! -e "$out"
text=$(jq -er --argjson row "$row" '[.[] | select(.row==$row) | .messages[0].content] | if length==1 then .[0] else error("row count") end' "$panel")
args=(--discrete "$model" --tokenizer "$tok" --protocol 2 --say "$text" --max-tokens 16 --temperature 0)
case "$mode" in
    ref) args+=(--read-mode enabled);;
    disabled) args+=(--read-mode no_read);;
    nl|wl|nc|wc) args+=(--read-mode enabled --contextual-source-trace);;
    *) exit 2;;
esac
"$chat" "${args[@]}" > "$out.partial"
jq -e '.generated_ids|type=="array" and length>0' "$out.partial" > /dev/null
mv "$out.partial" "$out"
