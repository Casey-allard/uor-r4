#!/bin/bash
# Use the same ordinary protocol-2 CLI as the unchanged read-enabled capture.
set -euo pipefail
if [[ ${1:-} == row ]]; then
    IFS=$'\t' read -r row message < "$2"
    "$CHAT" --discrete "$MODEL" --tokenizer "$TOKENIZER" --protocol 2 --temperature 0 \
        --max-tokens 16 --read-mode no_read --say "$message" > "$DEST/$row.json"
    exit
fi
export CHAT=${1:?uor-chat} MODEL=${2:?artifact} TOKENIZER=${3:?tokenizer} DEST=${4:?new capture root}
mkdir "$DEST"
mkdir "$DEST/jobs"
shift 4
for manifest in "$@"; do
    while IFS=$'\t' read -r row message; do
        [[ "$row" =~ ^[0-9]+$ && -n "$message" && ! -e "$DEST/jobs/$row" ]]
        printf '%s\t%s\n' "$row" "$message" > "$DEST/jobs/$row"
    done < "$manifest"
done
find "$DEST/jobs" -type f -print0 | xargs -0 -P 4 -I '{}' "$0" row '{}'
shasum -a 256 "$MODEL" | cut -d ' ' -f 1 > "$DEST/artifact.sha256"
