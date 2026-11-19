#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
fixture=$(jq -n -f reduce-fixture.jq)

reduce() {
    jq -c "$1" <<< "$fixture" |
        jq -es --slurpfile labels <(jq '.labels' <<< "$fixture") -f reduce.jq
}

reduce '.inputs[]' | jq -e '
    .schema == "uor-r4.value-rank-paired-summary/1"
    and .arms[0].all_36.n == 36
    and .arms[0].all_36.exact == 4
    and .arms[0].matched_24.n == 24
    and .arms[0].matched_24.exact == 3
    and (.arms[0].matched_triples | length) == 8
    and .arms[0].all_36.latest_first_errors[0].first_error
        == {index:1, expected_id:43, emitted_id:1}
    and .arms[0].positive_rows[3].earliest_stage == "missing_requested_role"
    and .arms[0].positive_rows[4].earliest_stage == "wrong_value_source"
    and .arms[0].positive_rows[5].earliest_stage == "correct_source_wrong_emission"
    and .arms[0].positive_rows[6].earliest_stage == "missing_assertion"
    and .arms[0].positive_rows[7].earliest_stage == "unseen_shape_class"
    and .arms[0].positive_rows[8].earliest_stage == "unseen_family_radius"
    and .arms[0].positive_rows[9].earliest_stage == "outside_fitted_radius"
    and (.arms[2].negative_controls | length) == 6
    and .paired_exact[0].all_36
        == {n:36, left_only:[0,12,25], right_only:[2,26], both_exact:1, both_wrong:30}
    and .paired_exact[0].matched_24
        == {n:24, left_only:[0,12], right_only:[2], both_exact:1, both_wrong:20}
' >/dev/null

check_refusal() {
    local filter="$1" message="$2" report
    if report=$(reduce "$filter" 2>&1); then
        printf '%s\n' "incorrectly accepted $message" >&2
        exit 1
    fi
    [[ "$report" == *"$message"* ]]
}

check_refusal '.inputs[0] | del(.report.rows[35])' 'entire registered arm cohort'
check_refusal '.inputs[2] | del(.report.rows[41])' 'entire registered arm cohort'
check_refusal '.inputs[0] | .report.rows[2].original_exact = true' 'inconsistent independent positive verdict'
check_refusal '.inputs[0], .inputs[0]' 'arm absent or repeated'
printf '%s\n' 'paired reducer fixture and four refusals passed'
