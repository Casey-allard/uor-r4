#!/bin/bash
# Capture independent one-shot generations from the ordinary text entry point.
set -euo pipefail
if [[ ${1:-} == row ]]; then
    IFS=$'\t' read -r row text < "$2"
    "$CHAT" --discrete "$MODEL" --tokenizer "$TOKENIZER" --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say "$text" > "$DEST/$row.json"
    exit
fi
export CHAT=${1:?uor-chat} MODEL=${2:?artifact} TOKENIZER=${3:?tokenizer} DEST=${5:?new output root}
mkdir "$DEST"
mkdir "$DEST/jobs"
while IFS=$'\t' read -r row text; do
    [[ "$row" =~ ^[0-9]+$ && -n "$text" ]]
    printf '%s\t%s\n' "$row" "$text" > "$DEST/jobs/$row"
done < "${4:?blind prompts}"
find "$DEST/jobs" -type f -print0 | xargs -0 -P 4 -I '{}' "$0" row '{}'
shasum -a 256 "$MODEL" | awk '{print $1}' > "$DEST/artifact.sha256"
