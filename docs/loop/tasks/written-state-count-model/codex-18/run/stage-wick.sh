#!/usr/bin/env bash
# Bind the prior fit and writer, prepare new prompt IDs, and identify the packed policies.
# This step uses no held inference.
set -euo pipefail
umask 077
[[ $# == 1 ]] || { echo "usage: stage-wick.sh ABS_STAGE" >&2; exit 2; }
stage=$(realpath "$1")
wick=$(dirname "$stage")
prior="$wick/20261119-value-ranking"
study="$prior/live-rank-d4688456ef77"
runner=$(cd "$(dirname "$0")" && pwd -P)
sum() { sha256sum "$1" | cut -d' ' -f1; }
pin() { printf '%s  %s\n' "$2" "$1" | sha256sum --status --check; }
entry() { jq -nc --arg k "$1" --arg path "$2" --arg sha256 "$(sum "$2")" '{key:$k,value:{path:$path,sha256:$sha256}}'; }
cd "$stage"
sha256sum --status --check build-receipt/executables.sha256
sha256sum --status --check artifacts/identities.sha256
(cd "$prior" && sha256sum --status --check launch.sha256)
(cd "$wick/20261118-ordered-role-transfer" && sha256sum --status --check "$prior/chain.sha256")
test "$(cat "$study/dispatch.status")" = "done 0"
cmp panel/reader-root.json panel/reader-two.json
(cd panel && sha256sum --status --check INPUTS.sha256)
jq -e '.rows==42 and .all_agree and (.disagreements|length)==0' panel/adjudication.json >/dev/null
mkdir specs prepared
# Bind the pristine independent readings and their D17 adapter separately.
pin panel/reader-grader.json 559697e76d2e89dd6021c1441e0184a8f136e69df1729149b774a04977c4ce6b
cmp panel/reader-grader.json <(jq -f panel/grader-reading.jq panel/reader-two.json)
for seed in 1 2; do
    original="$study/spec-$seed.json"
    panel="$stage/panel/seed$seed/held"
    {
        for key in base paired span writer action_writer grader tokenizer binder fit_grades fit_stage supplement; do
            path=$(jq -er --arg key "$key" '.files[$key].path' "$original")
            pin "$path" "$(jq -er --arg key "$key" '.files[$key].sha256' "$original")"
            entry "$key" "$path"
        done
        for entry_spec in "capture=$prior/bin/uor-value-rank-capture" "native=$stage/bin/all-starts-native" \
            "device=$stage/bin/native-sm89" "chat=$stage/bin/uor-chat" "runner=$runner/run-seed.sh" \
            "grades=$panel/grades.json" "readings=$stage/panel/reader-grader.json" "names=$stage/panel/held-names.json" \
            "public=$panel/public-text.json" "bounds=$panel/bounds.json" \
            "fit_capture=$study/primary-seed$seed/fit/requested-diagnostic/candidates.json" \
            "fit_input_gate=$stage/sources/docs/loop/tasks/written-state-count-model/codex-17/validity/check-inputs.sh" \
            "fit_input_gate_jq=$stage/sources/docs/loop/tasks/written-state-count-model/codex-17/validity/input.jq" \
            "reducer=$stage/sources/docs/loop/tasks/written-state-count-model/codex-17/validity/reduce.jq" \
            "prior_spec=$original" "prior_chain=$prior/chain.sha256"; do
            entry "${entry_spec%%=*}" "${entry_spec#*=}"
        done

        tokenizer=$(jq -er '.files.tokenizer.path' "$original")
        fit_stage=$(jq -er '.files.fit_stage.path' "$original")
        fit_blind=$(jq -er '.inputs.blind.path' "$fit_stage")
        pin "$fit_blind" "$(jq -er '.inputs.blind.sha256' "$fit_stage")"
        entry fit_blind "$fit_blind"
        "$prior/bin/uor-value-rank-capture" prepare "$tokenizer" "$(sum "$tokenizer")" \
            "$panel/public-text.json" "$(sum "$panel/public-text.json")" \
            "$stage/panel/held-names.json" "$(sum "$stage/panel/held-names.json")" "$stage/prepared/seed$seed" >&2
        jq -en --slurpfile blind "$panel/blind.json" --slurpfile bounds "$panel/bounds.json" '
            ($blind[0]|length)==42 and ($bounds[0]|length)==42
            and all(range(0; 42); . as $i
                | ($bounds[0][$i]|keys)==["end","start"]
                and $bounds[0][$i].start>=0 and $bounds[0][$i].start<$bounds[0][$i].end
                and $bounds[0][$i].end<=($blind[0][$i].prefix|length))
        ' >/dev/null
        diff -u <(jq -S -c . "$panel/blind.json") <(jq -S -c . "prepared/seed$seed/blind.json") >&2
        entry blind "$stage/prepared/seed$seed/blind.json"
        entry resolved "$stage/prepared/seed$seed/resolved-requested.json"

        for mode in ordered position; do
            fit="$study/primary-seed$seed/modes/$mode"
            repeat="$study/repeat-seed$seed/modes/$mode"
            for name in model.bank profile.json model.bank.work/fit.packet model.bank.work/fit.labels; do
                cmp "$fit/$name" "$repeat/$name"
            done
            test "$(stat -c %s "$fit/model.bank")" = 8416
            jq -e --arg mode "$mode" --arg train "$(sum "$fit/model.bank.work/fit.packet")" \
                --arg labels "$(sum "$fit/model.bank.work/fit.labels")" \
                '.mode==$mode and .train_sha256==$train and .labels_sha256==$labels' "$fit/profile.json" >/dev/null
            entry "bank_$mode" "$fit/model.bank"
            entry "fit_packet_$mode" "$fit/model.bank.work/fit.packet"
            entry "fit_labels_$mode" "$fit/model.bank.work/fit.labels"
            entry "old_profile_$mode" "$fit/profile.json"
        done
        for mode in ordered ranked veto both position; do
            profile="$stage/artifacts/seed$seed/$mode.json"
            artifact="$stage/artifacts/seed$seed/$mode.d11"
            entry "profile_$mode" "$profile"
            entry "artifact_$mode" "$artifact"
        done
    } | jq -es --argjson seed "$seed" --arg wick "$wick" --arg stage "$stage" '
        (map(.key) | length == (unique | length)) as $unique
        | select($unique and all(.[]; .value.sha256 | test("^[0-9a-f]{64}$")))
        | {schema:"uor-r4.value-start-factorial/1",seed:$seed,wick_root:$wick,stage:$stage,files:from_entries}
    ' > "specs/seed$seed.json"
done
{
    sha256sum "$runner"/{build-wick,pack-wick,stage-wick,run-wick,run-seed}.sh build-receipt/{source,executables}.sha256 \
        artifacts/identities.sha256 specs/seed{1,2}.json panel/{INPUTS.sha256,grader-reading.jq} \
        panel/{reader-root,reader-two,reader-grader,adjudication}.json
    jq -r '.files[] | "\(.sha256)  \(.path)"' specs/seed{1,2}.json | LC_ALL=C sort -u
} > inputs.sha256
sha256sum --status --check inputs.sha256
