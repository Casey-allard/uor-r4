def check($condition; $message):
    if $condition then . else error($message) end;

def expected_fact($g; $target):
    [$g.facts[] | select(.name==$target)] |
    check(length==1; "control target lacks one recorded fact") | .[0];

def expected_start($fact):
    [$fact.legal[] | select(.start==$fact.value_span[0])] |
    check(length==1; "control fact lacks exactly one original beginning") | .[0];

def other($g): expected_fact($g; $g.excluded_name);

if $mode == "wrong-key" then
    map(if .stratum=="supported" then
        other(.) as $fact | expected_start($fact) as $start
        | check(.expected!=$start.copy_ids_through_eos; "wrong key equals gold")
        | .answer_text=$fact.value | .expected=$start.copy_ids_through_eos
    else . end)
elif $mode == "force-gold" or $mode == "force-wrong" then
    $labels[0] as $g | $capture[0].rows as $c
    | check(.blind==$capture[0].blind_sha256 and (.rows|length)==($c|length)
            and ($c|length)==($g|length); "control tape cannot bind")
    | check($capture[0].selection_sha256==$selection_sha; "control selection digest disagrees")
    | .arm=$mode
    | .rows |= map(.index as $i | $g[$i] as $grade | $c[$i] as $captured
      | check(.prefix_sha256==$grade.prefix_sha256 and .prefix_sha256==$captured.prefix_sha256;
              "control row cannot bind")
      | select($grade.stratum=="supported")
      | (if $mode=="force-gold" then expected_fact($grade; $grade.requested_name)
         else other($grade) end) as $fact
      | expected_start($fact) as $start
      | (if $mode=="force-gold" then $grade.requested else $grade.excluded end) as $endpoint
      | check($captured.requested_span==$endpoint and ($captured.assertions|length)==1
          and $captured.assertions[0].fact_span==$fact.name_span
          and $captured.assertions[0].fence==$fact.value_span[1]
          and any($captured.assertions[0].legacy_candidates[];
              .original_start==$start.start and .writer_reachable);
          "CUDA capture did not make the intended original literal reachable at row \($i)")
      | check((if $mode=="force-gold" then $grade.expected==$start.copy_ids_through_eos
               else $grade.expected!=$start.copy_ids_through_eos end)
          and ($start.copy_ids_through_eos[-1]==1); "control literal contradicts sealed EOS")
      | .paired.choice.source=$start.start)
else error("unrecognized control mode") end
