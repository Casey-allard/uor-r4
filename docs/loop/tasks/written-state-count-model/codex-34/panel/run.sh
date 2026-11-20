#!/bin/bash
# One capture root per independent set of ordinary client invocations.
set -euo pipefail
chat=${1:?uor-chat}; artifact=${2:?D26 carrier}; tokenizer=${3:?tokenizer}
blind=${4:?sealed blind TSV}; go=${5:?top-level GO JSON}; dest=${6:?new capture directory}
[[ $(hostname -s) == wick ]]
native_sha=$(jq -er 'select(.decision == "GO" and .phase == "exposed" and .host == "wick")
    | .native_sha256 | select(test("^[a-f0-9]{64}$"))' "$go")
printf '%s  %s\n' \
    "$native_sha" "$chat" \
    b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4 "$artifact" \
    a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$tokenizer" \
    176082f66f2e73ba4080567fb2b5e77cc7c4478c14da939b47fad72fa2660a30 "$blind" \
    | sha256sum --check --strict
umask 077
mkdir "$dest"
sha256sum "$chat" "$artifact" "$tokenizer" "$blind" "$go" > "$dest/inputs.sha256"
hostname > "$dest/host.txt"
for arm in full-repeated full-detached; do
    mkdir "$dest/$arm"
    while IFS=$'\t' read -r row prompt; do
        [[ "$row" =~ ^[0-9]+$ && -n "$prompt" && "$prompt" != *$'\t'* ]]
        "$chat" --discrete "$artifact" --tokenizer "$tokenizer" --protocol 2 \
            --temperature 0 --max-tokens 16 --read-mode enabled --role-replay "$arm" \
            --say "$prompt" > "$dest/$arm/$row.json"
    done < "$blind"
done
(
    cd "$dest"
    sha256sum -- */*.json > captures.sha256
)
