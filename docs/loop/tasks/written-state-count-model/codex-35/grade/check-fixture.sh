#!/usr/bin/env bash
# Exercise independent grading on previously sealed native D34 responses.
set -euo pipefail
grade=$(realpath "$1")
out=$2
mkdir "$out"
out=$(realpath "$out")
raw=/mnt/plexpool/uor-r4-corpus/runs/20261120-detached-role-34
key=/home/callard/uor-codex/20261120-context-swap-30/custody-v1/raw/panel/role.gold.json
cd "$out"
jq --slurpfile old "$raw/joined.json" '[.[]|select(.row==30106 or .row==30100)|. as $k |
    {row, group:.family, stratum, original_ids, query:.role_targets.requested,
    fact:($old[0].rows[]|select(.row==$k.row)|.["full-detached"].assertion.expected_fact_span),
    gold_source:(.value_spans[]|select(.[0]==$k.gold_source)),expected_tape}]' "$key" > key.json
jq -n --arg d b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4 \
    '{baseline:$d,fact_true:$d,fact_reversed:$d,candidate_true:$d,candidate_reversed:$d}' > hashes.json
for pass in primary cold; do for mode in ordinary supplied; do for arm in baseline fact_true fact_reversed candidate_true candidate_reversed; do
    dest="$pass/$mode/$arm"
    mkdir -p "$dest"
    for id in 30106 30100; do cp "$raw/ordinary-$pass/full-detached/$id.json" "$dest/$id.json"; done
done; done; done
"$grade" key.json primary cold hashes.json > positive.json
jq -e '[.rows[].arms|to_entries[]|.value] as $all |
    ($all|length)==20 and ([$all[]|select(.exact_eos==true and .correct_source==true)]|length)==10 and
    ([$all[]|select(.first_wrong=={offset:0,expected:262,actual:1155})]|length)==10' positive.json > /dev/null
# Change the active state in both tapes without changing any emitted ID.
for pass in primary cold; do
    target="$pass/ordinary/fact_true/30106.json"
    jq '.copy_step_words[0][0]=0' "$target" > "$target.mutant"
    mv "$target.mutant" "$target"
done
if "$grade" key.json primary cold hashes.json > negative.json 2> negative.log; then exit 4; fi
grep -Fq 'invalid original writer transition at generated offset 0' negative.log
sha256sum positive.json negative.log
