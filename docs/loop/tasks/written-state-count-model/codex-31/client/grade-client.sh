#!/usr/bin/env bash
set -euo pipefail
umask 077
if [[ $# != 4 ]]; then echo "usage: grade-client.sh CLIENT_RUN PRIVATE_PANEL_DIR CUDA_PANEL NEW_GRADE" >&2; exit 2; fi
root=$1 panel=$2 device=$3 out=$4
here=$(dirname "$0")
key_sha=$(shasum -a 256 "$panel/key.json" | cut -d ' ' -f1)
test "$key_sha" = 95d556eaf457b2f517123f01b5b3b5a39784e947ce106033119fe97a195238b5
test "$(shasum -a 256 "$panel/blind.json" | cut -d ' ' -f1)" = a4fb823cdd5031cc7ffdb1c402f7d7df2c149002b1335a102a8a012371f081dc
test "$(shasum -a 256 "$device/steps.jsonl" | cut -d ' ' -f1)" = "$(jq -er '.steps_sha256' "$device/receipt.json")"
pkg_sha=$(awk 'NR==3 {print $1}' "$root/inputs.sha256")
trunk_sha=04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d
mkdir "$out"
for file in "$root"/primary/*.json; do
    name=${file##*/}
    cmp "$file" "$root/cold/$name"
    jq -c --argjson id "${name%%-*}" '{id:$id,result:.}' "$file"
done > "$out/client.jsonl"
jq -se --slurpfile cpu "$out/client.jsonl" --slurpfile keys "$panel/key.json" --slurpfile blind "$panel/blind.json" \
    --slurpfile receipt "$device/receipt.json" --arg pkg_sha "$pkg_sha" --arg trunk_sha "$trunk_sha" \
    -f "$here/client-grade.jq" "$device/steps.jsonl" > "$out/grade.json"
