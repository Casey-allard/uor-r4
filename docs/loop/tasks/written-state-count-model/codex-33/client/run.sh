#!/usr/bin/env bash
# Run ordinary one-shot text serving on the merged component bytes.
set -euo pipefail
umask 077
if [[ $# != 7 ]]; then
    echo "usage: run.sh CLIENT D11 TOKENIZER PROMPTS OUTER_SHA CLIENT_SHA NEW_OUT" >&2
    exit 2
fi
client=$(realpath "$1"); model=$(realpath "$2"); tokenizer=$(realpath "$3")
panel=$(realpath "$4"); outer=$5; exe=$6; out=$7
[[ $outer =~ ^[[:xdigit:]]{64}$ && $exe =~ ^[[:xdigit:]]{64}$ ]]
printf '%s  %s\n' "$outer" "$model" "$exe" "$client" \
    a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 "$tokenizer" \
    e50d7812f4a4dcc3016793e07cf2efb55fa7524d880bf1253324ec255b4b8b11 "$panel" | sha256sum --strict -c -
mkdir "$out"
cd "$out"
sha256sum "$client" "$model" "$tokenizer" "$panel" > inputs.sha256
for pass in primary cold timing; do
    mkdir "$pass"
    trace=(--p1-trace)
    if [[ $pass == timing ]]; then trace=(); fi
    while IFS=$'\t' read -r id unused_key prompt; do
        for mode in enabled no_read; do
            name="$id-$mode"
            /usr/bin/time -f '%e %M' -o "$pass/$name.resources" \
                env UOR_P1_TIMING=1 "$client" --discrete "$model" --tokenizer "$tokenizer" \
                --protocol 2 --temperature 0 --max-tokens 12 --read-mode "$mode" \
                "${trace[@]}" --say "$prompt" > "$pass/$name.json" 2> "$pass/$name.stderr"
            if [[ $pass == cold ]]; then cmp "primary/$name.json" "$pass/$name.json"; fi
            if [[ $pass == timing ]]; then
                jq -e --slurpfile traced "primary/$name.json" \
                    '(.p1_steps==null) and .generated_ids==$traced[0].generated_ids and .stop==$traced[0].stop' \
                    "$pass/$name.json" >/dev/null
            fi
        done
    done < "$panel"
done
sha256sum primary/* cold/* timing/* > captures.sha256
