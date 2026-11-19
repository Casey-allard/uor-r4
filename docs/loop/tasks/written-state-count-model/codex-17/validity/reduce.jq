# jq -es --slurpfile labels HELD_GRADES -f reduce.jq
# Each input is {arm, report, native_sources?, source_audit?}.
def require($ok; $message): if $ok then . else error($message) end;

def abstention($mode):
    if .status != "unseen" then .status
    elif $mode == "position" then "unseen_position_key"
    elif any(.candidates[]; .status == "unseen"
             and (.positive_evidence == null or .negative_evidence == null)) then
        "unseen_shape_class"
    elif any(.candidates[]; .status == "unseen" and .radius == null) then
        "unseen_family_radius"
    elif any(.candidates[]; .status == "unseen" and .positive_evidence > .radius) then
        "outside_fitted_radius"
    else "unseen"
    end;

def earliest_stage($label; $trace; $audit; $mode):
    if .original_exact then "exact"
    elif .role_nomination == null then "missing_requested_role"
    elif .role_nomination != .gold_role_nomination then "wrong_requested_role"
    elif .source == null then
        if any($trace[]; .assertion_index == null and .status == "no_candidate")
        then "missing_assertion"
        elif any($trace[]; .status != "selected") then
            ($trace | map(select(.status != "selected") | abstention($mode)) | unique |
                if length == 1 then .[0] else "multiple_abstention_reasons" end)
        elif $audit.consensus.values_disagree == true then "assertion_disagreement"
        elif $audit.consensus.supporting_facts == 0 then "missing_assertion"
        elif $audit.consensus.complete_values < $audit.consensus.supporting_facts
        then "incomplete_assertion"
        else "missing_value_source"
        end
    elif .source != $label.gold_start then "wrong_value_source"
    else "correct_source_wrong_emission"
    end;

def counts:
    group_by(.) | map({value: .[0], count: length});

def case_row($label; $trace; $audit; $mode):
    . as $row
    | require((.original_exact | type) == "boolean"
              and (.original_exact == (.original_first_wrong == null))
              and ((.original_exact | not) or .terminal_eos == true);
              "inconsistent independent positive verdict")
    | {
        index, id, family: $label.family, pair: $label.pair, direction: $label.direction,
        matched_triple: $label.matched_triple, exact: .original_exact,
        active_exact, terminal_eos, selected_source: .source, gold_source: $label.gold_start,
        requested: .role_nomination, gold_requested: .gold_role_nomination,
        original_gold_offset: $label.gold_offset,
        earliest_stage: earliest_stage($label; $trace; $audit; $mode),
        native_statuses: [$trace[] | {assertion_index, status, reason: abstention($mode)}],
        first_error: (if .original_first_wrong == null then null else
            .original_first_wrong as $i
            | {index: $i, expected_id: $row.original_expected[$i], emitted_id: $row.generated[$i]}
            end)
      };

def scope:
    . as $cases
    | map(select(.first_error != null)) as $errors
    | {
        n: length,
        exact: (map(select(.exact)) | length),
        active_exact: (map(select(.active_exact)) | length),
        terminal_eos: (map(select(.terminal_eos)) | length),
        exact_indices: [ .[] | select(.exact) | .index ],
        earliest_stage_counts: (map(.earliest_stage) | counts),
        first_error_index_counts: ($errors | map(.first_error.index) | counts),
        earliest_first_errors: ($errors | (map(.first_error.index) | min) as $index |
            map(select(.first_error.index == $index) | {index, id, earliest_stage, first_error})),
        latest_first_errors: ($errors | (map(.first_error.index) | max) as $index |
            map(select(.first_error.index == $index) | {index, id, earliest_stage, first_error})),
        by_family: ($cases | group_by(.family) | map({
            family: .[0].family, n: length, exact: (map(select(.exact)) | length),
            earliest_stage_counts: (map(.earliest_stage) | counts)
        }))
      };

def by_ordinal($rows):
    $rows | group_by(.ordinal) | map({key: (.[0].ordinal | tostring), value: .}) | from_entries;

def arm_summary($labels; $all; $matched; $negative):
    . as $entry
    | .report as $report
    | require($report.schema == "uor-r4.value-rank-observed-grader/1";
              "wrong independent grader schema")
    | require((.arm | type) == "string" and (.arm | length) > 0; "arm name missing")
    | (if (.arm | contains("-normal-")) then ($all + $negative | sort) else $all end) as $expected
    | require(([$report.rows[].index] | sort) == $expected;
              "independent grade rows do not match the entire registered arm cohort")
    | require(all($report.rows[];
                  . as $row | $row.id == $labels[$row.index].id
                  and (($labels[$row.index].stratum == "supported") == ($row.negative == null)));
              "independent grade rows do not bind original strata and ids")
    | by_ordinal($entry.native_sources.rows // []) as $native
    | by_ordinal($entry.source_audit.rows // []) as $audits
    | [$report.rows[] | select(.negative == null) | .index as $i | ($i | tostring) as $key
       | case_row($labels[$i]; ($native[$key] // []); ($audits[$key][0] // {});
                  $entry.native_sources.mode)] | sort_by(.index)
    | . as $positive
    | map(select(.index as $i | $matched | index($i) != null)) as $primary
    | {
        arm: $entry.arm,
        all_36: ($positive | scope),
        matched_24: ($primary | scope),
        matched_triples: ($primary | group_by(.matched_triple) |
            map({triple: .[0].matched_triple, exact: (map(select(.exact)) | length),
                 cases: map({index, id, direction, original_gold_offset, selected_source, gold_source,
                             exact, earliest_stage, first_error})})),
        positive_rows: $positive,
        negative_controls: [$report.rows[] | select(.negative != null) |
            {index, id, semantics, decoded_text, terminal_eos, negative}]
      };

def paired_scope($l; $r):
    require(($l | map(.index)) == ($r | map(.index)); "paired arms have different ordinals")
    | [range(0; ($l | length)) as $i | {
        index: $l[$i].index, left: $l[$i].exact, right: $r[$i].exact
      }]
    | {
        n: length,
        left_only: [ .[] | select(.left and (.right | not)) | .index ],
        right_only: [ .[] | select((.left | not) and .right) | .index ],
        both_exact: (map(select(.left and .right)) | length),
        both_wrong: (map(select((.left | not) and (.right | not))) | length)
      };

require(($labels | length) == 1 and ($labels[0] | length) == 42; "wrong held label tape")
| $labels[0] as $sealed
| [$sealed | to_entries[] | select(.value.stratum == "supported") | .key] as $all
| [$sealed | to_entries[] | select(.value.stratum != "supported") | .key] as $negative
| [$sealed | to_entries[] | select(.value.matched_triple != null) | .key] as $matched
| require(($all | length) == 36 and ($matched | length) == 24
          and all($matched[]; . as $i | $all | index($i) != null);
          "frozen supported or matched cohort changed")
| require((length > 0) and ((map(.arm) | unique | length) == length); "arm absent or repeated")
| map(arm_summary($sealed; $all; $matched; $negative)) as $arms
| {
    schema: "uor-r4.value-rank-paired-summary/1",
    arms: $arms,
    paired_exact: [
        range(0; ($arms | length)) as $i
        | range($i + 1; ($arms | length)) as $j
        | $arms[$i] as $l | $arms[$j] as $r
        | {
            left: $l.arm, right: $r.arm,
            all_36: paired_scope($l.positive_rows; $r.positive_rows),
            matched_24: paired_scope(($l.positive_rows | map(select(.matched_triple != null)));
                                    ($r.positive_rows | map(select(.matched_triple != null))))
          }
    ]
  }
