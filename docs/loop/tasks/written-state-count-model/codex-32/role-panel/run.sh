#!/bin/bash
# Run independent client invocations with only the sealed user-text projection.
set -euo pipefail
chat=${1:?uor-chat}; artifact=${2:?D26 carrier}; tokenizer=${3:?tokenizer}
blind=${4:?sealed blind TSV}; dest=${5:?new capture directory}
printf '%s  %s\n' \
    b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4 "$artifact" \
    a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$tokenizer" \
    176082f66f2e73ba4080567fb2b5e77cc7c4478c14da939b47fad72fa2660a30 "$blind" \
    | sha256sum --check --strict
mkdir "$dest"
sha256sum "$chat" "$artifact" "$tokenizer" "$blind" > "$dest/inputs.sha256"
printf '%s\n' "$(hostname)" > "$dest/host.txt"
for arm in historical-inherited historical-repeated full-inherited full-repeated; do
    mkdir "$dest/$arm"
    while IFS=$'\t' read -r row text; do
        [[ "$row" =~ ^[0-9]+$ && -n "$text" ]]
        "$chat" --discrete "$artifact" --tokenizer "$tokenizer" --protocol 2 \
            --temperature 0 --max-tokens 16 --read-mode enabled --role-replay "$arm" \
            --say "$text" > "$dest/$arm/$row.json"
    done < "$blind"
done
(
    cd "$dest"
    sha256sum -- */*.json > captures.sha256
)
