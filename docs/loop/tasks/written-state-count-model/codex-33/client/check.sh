#!/usr/bin/env bash
# Validate the new container identity before projecting the record to the prior
# component-indexed CUDA grading schema. Keep the original record untouched.
set -euo pipefail
if [[ $# != 6 ]]; then
    echo "usage: check.sh RUN PREVIOUS_ORDINARY PRIVATE_PANEL CUDA D31_JQ NEW_OUT" >&2
    exit 2
fi
run=$1; old=$2; panel=$3; gpu=$4; grade=$5; out=$6
outer=$(awk 'NR==2 {print $1}' "$run/inputs.sha256")
trunk=04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d
package=075f0432ddca78ed48292104167f5056b88143ca3211954da98f04bf0dce64a9
mkdir "$out"
printf '%s  %s\n' \
    95d556eaf457b2f517123f01b5b3b5a39784e947ce106033119fe97a195238b5 "$panel/key.json" \
    a4fb823cdd5031cc7ffdb1c402f7d7df2c149002b1335a102a8a012371f081dc "$panel/blind.json" \
    ffb4302a3677d94e156f6ad18f7639f2a8966010b0bed69cbf0f18afaff68ed9 "$gpu/steps.jsonl" \
    71c36b01b0063279fc7c4da8da93aa9dbc4d998f65aa2b7d22e56a24279eef56 "$gpu/receipt.json" | sha256sum --strict -c -
for file in "$run"/primary/*.json; do
    name=${file##*/}; id=${name%%-*}
    cmp "$file" "$run/cold/$name"
    jq -ce --arg outer "$outer" --arg trunk "$trunk" --arg package "$package" --argjson id "$id" \
        --slurpfile previous "$old/primary/$name" '
        if .artifact_sha256!=$outer or .p1_trunk_sha256!=$trunk or .p1_sha256!=$package or .p1_state_sha256!=$package
        then error("unbound deployed components")
        elif ([.p1_steps,.prompt_ids,.generated_ids,.stop,.text,.p1_components,.tokenizer_sha256,.read_enabled]
            != ( $previous[0] | [.p1_steps,.prompt_ids,.generated_ids,.stop,.text,.p1_components,.tokenizer_sha256,.read_enabled]))
        then error("old and embedded client decisions differ")
        else {id:$id,result:(.artifact_sha256=.p1_trunk_sha256)} end' "$file"
done > "$out/component-view.jsonl"
jq -se --slurpfile cpu "$out/component-view.jsonl" --slurpfile keys "$panel/key.json" \
    --slurpfile blind "$panel/blind.json" --slurpfile receipt "$gpu/receipt.json" \
    --arg pkg_sha "$package" --arg trunk_sha "$trunk" -f "$grade" "$gpu/steps.jsonl" > "$out/grade.json"
sha256sum "$out/grade.json"
