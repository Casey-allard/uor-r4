#!/usr/bin/env bash
# Finite count transport with unit KN weights. No training or held text is used.
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
out=$(realpath "$1")
counts=$(realpath "$2")
pack=$(realpath "$3")
container=$(realpath "$4")
chat=$(realpath "$5")
seed=$(realpath "$6")
tokenizer=$(realpath "$7")
cd "$out"
sha256sum "$pack" "$container" "$chat" "$seed" "$tokenizer" "$counts" fit.json weights.bin witnesses.json > input.sha256
"$pack" pack fit.json weights.bin "$(sha256sum weights.bin | cut -d' ' -f1)" "$counts" fixture.p1 > package.json
"$container" pack "$seed" fixture.p1 fixture.d11 > container.json
"$container" readback fixture.d11 > readback.json
mkdir checks
while IFS=$'\t' read -r n prefix; do
    expected=$(jq -c --argjson n "$n" '.[$n]' witnesses.json)
    test -n "$prefix"
    for pass in primary cold; do
        "$chat" --discrete fixture.d11 --prompt-ids "$prefix" --temperature 0 --max-tokens 1 \
            --read-mode enabled --p1-trace > "checks/$n-$pass.json"
        jq -e --argjson w "$expected" '
            .generated_ids==[$w.token] and (.p1_steps|length)==1 and
            (.p1_steps[0]|.token==$w.token and .mass_q48==$w.mass_q48 and
               .mass_sha256==$w.mass_sha256 and .top_ties==$w.top_ties)
        ' "checks/$n-$pass.json" > /dev/null
    done
    cmp "checks/$n-primary.json" "checks/$n-cold.json"
done < <(jq -r 'to_entries[]|[.key,(.value.context|join(","))]|@tsv' witnesses.json)
jq --argjson tokenizer "$(stat -c%s "$tokenizer")" --argjson count "$(stat -c%s "$counts")" \
    --argjson d11 "$(stat -c%s fixture.d11)" --argjson p1 "$(stat -c%s fixture.p1)" \
    '{count_bytes:$count,p1_bytes:$p1,d11_bytes:$d11,tokenizer_bytes:$tokenizer,total_deployment_bytes:($d11+$tokenizer),probes:length}' witnesses.json > physical.json
sha256sum fixture.d11 fixture.p1 readback.json physical.json checks/*.json > results.sha256
