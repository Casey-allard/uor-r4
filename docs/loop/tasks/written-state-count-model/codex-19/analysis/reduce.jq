def by_index:
    .rows | map({key: (.index|tostring), value: .}) | from_entries;
def overlap($a; $b):
    $a!=null and $b!=null and $a[0]<$b[1] and $b[0]<$a[1];
def source_kind($r; $g):
    if $r.source==null then "abstained"
    elif $r.source_matches_requested_value then "gold_original_start"
    elif any($g.facts[]; .name == $g.requested_name
        and .value_span[0] <= $r.source and $r.source < .value_span[1]) then "requested_fact_suffix"
    elif any($g.facts[]; .value_span[0] <= $r.source and $r.source < .value_span[1])
        then "other_fact"
    else "outside_authored_values" end;
def role_kind($r; $g):
    if $r.requested==null then "abstained"
    elif $r.requested==$g.requested then "gold_query_span"
    elif overlap($r.requested; $g.excluded) then "distractor_query_span"
    elif any($g.facts[]; .name==$g.requested_name and overlap($r.requested; .name_span))
        then "requested_name_factual_occurrence"
    elif any($g.facts[]; overlap($r.requested; .name_span)) then "other_factual_occurrence"
    else "other_span" end;
def output($row; $g; $role):
    if $row==null then null
    elif $row.id!=$g.id then error("writer row misaligned")
    elif $role!=null and (
        ($role.role_words[:2] | if .==[4294967295,4294967295] then null else . end)!=$row.requested
        or ($role.role_words[2:4] | if .==[4294967295,4294967295] then null else . end)!=$row.excluded
    ) then error("CUDA role and graded writer role misaligned")
    else $row | {requested, excluded, endpoint_exact, source, supporting_facts, complete_values,
        values_disagree, generated, text, terminal_eos, exact, first_wrong, expected_wrong_id,
        actual_wrong_id, role_words:$role.role_words,
        requested_no_eligible_evidence:(if $role==null then null
            else $role.role_words[4]==2147483648 and $role.role_words[6]==2147483648 end),
        requested_evidence_tied:(if $role==null then null
            else $role.role_words[4]!=2147483648 and $role.role_words[4]==$role.role_words[6] end),
        requested_overlaps_selected_excluded:overlap(.requested; .excluded),
        requested_overlaps_gold_distractor:overlap(.requested; $g.excluded),
        role_class:role_kind($row; $g), source_class:source_kind($row; $g)
    } end;
def arm_counts($arm):
    map(.outputs[$arm] | select(.!=null)) | {
        rows:length,
        exact_through_eos:map(select(.exact==true))|length,
        correct_query_occurrence:map(select(.role_class=="gold_query_span"))|length,
        requested_excluded_overlap:map(select(.requested_overlaps_selected_excluded))|length,
        correct_query_with_excluded_overlap:
            map(select(.role_class=="gold_query_span" and .requested_overlaps_selected_excluded))|length,
        role_abstentions:map(select(.role_class=="abstained"))|length,
        no_eligible_role_evidence:map(select(.requested_no_eligible_evidence==true))|length,
        equal_role_score_words:map(select(.requested_evidence_tied==true))|length,
        correct_source:map(select(.source_class=="gold_original_start"))|length,
        no_source:map(select(.source==null))|length,
        ended_at_eos:map(select(.terminal_eos))|length,
        first_wrong_histogram:(map(select(.first_wrong!=null)) | group_by(.first_wrong)
            | map({key:(.[0].first_wrong|tostring),value:length}) | from_entries)
    };
def paired:
    {
      rows:length,
      role_up:[.[]|select(.outputs.ordered.role_class!="gold_query_span" and
                          .outputs.treatment.role_class=="gold_query_span")|.index],
      role_down:[.[]|select(.outputs.ordered.role_class=="gold_query_span" and
                            .outputs.treatment.role_class!="gold_query_span")|.index],
      source_up:[.[]|select(.outputs.ordered.source_class!="gold_original_start" and
                            .outputs.treatment.source_class=="gold_original_start")|.index],
      source_down:[.[]|select(.outputs.ordered.source_class=="gold_original_start" and
                              .outputs.treatment.source_class!="gold_original_start")|.index],
      exact_up:[.[]|select(.outputs.ordered.exact==false and .outputs.treatment.exact==true)|.index],
      exact_down:[.[]|select(.outputs.ordered.exact==true and .outputs.treatment.exact==false)|.index],
      emitted_tape_changed:[.[]|select(.outputs.ordered.generated!=.outputs.treatment.generated)|.index]
    };
def summary:
    . as $rows | {
        arms:(["ordered","treatment","supplied","wrong-role","no-read"]
            | map(. as $arm|{key:$arm,value:($rows|arm_counts($arm))})|from_entries),
        paired:paired
    };

($ordered[0]|by_index) as $baseline | ($treatment[0]|by_index) as $new |
($supplied[0]|by_index) as $oracle | ($wrong_role[0]|by_index) as $wrong |
($no_read[0]|by_index) as $disabled |
$labels[0] | to_entries | map(.key as $i | (.key|tostring) as $k | .value as $g |
  if $baseline[$k]==null or $new[$k]==null or $disabled[$k]==null or
      ($g.stratum=="supported" and ($oracle[$k]==null or $wrong[$k]==null))
  then error("missing grade row")
  else {index:$i,id:$g.id, stratum:$g.stratum, family:$g.family, direction:$g.direction,
    reversed:$g.reversed, distractor_first:$g.distractor_first, the:$g.the, about:$g.about,
    gold_requested:$g.requested, gold_excluded:$g.excluded, gold_source:$g.gold_start,
    expected:$g.expected, prior_training_literal:($g.answer_text=="brushes"),
    outputs:{
      ordered:output($baseline[$k];$g;$ordered_candidates[0].rows[$i]),
      treatment:output($new[$k];$g;$treatment_candidates[0].rows[$i]),
      supplied:output($oracle[$k];$g;null),
      "wrong-role":output($wrong[$k];$g;null),
      "no-read":output($disabled[$k];$g;null)
    }} end) as $rows |
($rows|map(select(.stratum=="supported"))) as $supported |
{
    schema:"uor-r4.requested-occurrence-summary/1", seed:$seed, rows:$rows,
    supported:($supported|summary),
    held_transfer_unseen:($supported|map(select(.prior_training_literal|not))|summary),
    control_overlap_seen:($supported|map(select(.prior_training_literal))|summary),
    negatives:($rows|map(select(.stratum!="supported"))|group_by(.stratum)|
        map({key:.[0].stratum,value:summary})|from_entries),
    factor_cells:($supported|group_by([.direction,.reversed,.distractor_first,.the,.about])|
        map({direction:.[0].direction,reversed:.[0].reversed,distractor_first:.[0].distractor_first,
            the:.[0].the,about:.[0].about,summary:summary}))
}
