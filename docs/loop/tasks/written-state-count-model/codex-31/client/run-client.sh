#!/usr/bin/env bash
set -euo pipefail
umask 077

# Parent verifies the finite gate, device-bound GO and component identities.
# Each invocation opens the shipped D11 and P1 package in a fresh process.
if [[ $# != 6 ]]; then
    echo "usage: run-client.sh CLIENT TRUNK PACKAGE TOKENIZER PANEL_TSV NEW_OUT" >&2
    exit 2
fi
client=$1 trunk=$2 package=$3 tokenizer=$4 panel=$5 out=$6
test "$(shasum -a 256 "$panel" | cut -d ' ' -f1)" = e50d7812f4a4dcc3016793e07cf2efb55fa7524d880bf1253324ec255b4b8b11
mkdir "$out"
shasum -a 256 "$client" "$trunk" "$package" "$tokenizer" "$panel" > "$out/inputs.sha256"
TIMEFORMAT='%3R'
for pass in primary cold timing; do
    trace=(--p1-trace)
    if [[ $pass == timing ]]; then trace=(); fi
    mkdir "$out/$pass"
    while IFS=$'\t' read -r id gold prompt; do
        for mode in enabled no_read; do
            base="$out/$pass/$id-$mode"
            { time UOR_P1_TIMING=1 "$client" --discrete "$trunk" --p1-state "$package" ${trace[@]+"${trace[@]}"} \
                --tokenizer "$tokenizer" --protocol 2 --temperature 0 --max-tokens 12 \
                --read-mode "$mode" --say "$prompt" > "$base.json" 2> "$base.stderr"; } 2> "$base.seconds"
            if [[ $pass == cold ]]; then cmp "$base.json" "$out/primary/$id-$mode.json"; fi
            if [[ $pass == timing ]]; then
                jq -e --slurpfile primary "$out/primary/$id-$mode.json" \
                    '.generated_ids == $primary[0].generated_ids and (.p1_steps == null)' "$base.json" >/dev/null
            fi
        done
    done < "$panel"
done
