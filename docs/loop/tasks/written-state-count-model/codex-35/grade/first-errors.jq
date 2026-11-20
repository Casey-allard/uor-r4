def stage($r; $a):
    if $r.expected==null then (if $a.unsupported_copy then "unsupported copy" else "unsupported, no copy" end)
    elif $a.exact_eos then "exact through EOS"
    elif $a.requested_correct!=true then "requested role"
    elif $a.assertion_bound!=true then "factual head"
    elif $a.eligible_gold!=true then "gold onset outside candidate pool"
    elif $a.eligible_complete!=true then "gold completion outside candidate pool"
    elif $a.correct_source!=true then (if $a.withheld then "value selection abstained" else "rival source selected" end)
    else "after correct onset" end;
def movement($a;$b):
    if $a.exact_eos==null then "unkeyed"
    elif $a.exact_eos and $b.exact_eos then "both exact"
    elif $b.exact_eos then "error to exact"
    elif $a.exact_eos then "exact to error"
    elif $a.first_wrong.offset < $b.first_wrong.offset then "later error"
    elif $a.first_wrong.offset > $b.first_wrong.offset then "earlier error"
    elif $a.first_wrong.actual != $b.first_wrong.actual then "different token, same offset"
    else "same first error" end;
def freq: group_by(.) | map({key:.[0],value:length}) | from_entries;
[.rows | group_by(.mode)[] | . as $rows |
  ["fact_true","fact_reversed","candidate_true","candidate_reversed"][] as $arm |
  {mode:$rows[0].mode,arm:$arm,
   stage:([$rows[] | stage(.;.arms[$arm])] | freq),
   first_error_movement_vs_baseline:([$rows[] | movement(.arms.baseline;.arms[$arm])] | freq),
   changed:[$rows[]|. as $r | .arms.baseline as $a | .arms[$arm] as $b |
      select(($a.first_wrong != $b.first_wrong) or ($a.correct_source != $b.correct_source) or ($a.unsupported_copy!=$b.unsupported_copy)) |
      {row,group,stratum,requested_expected:.query,requested_seen:$b.requested,fact_expected:.fact,gold_source,
       baseline:{onset:$a.selected_source,first_wrong:$a.first_wrong,exact:$a.exact_eos},
       arm_result:{onset:$b.selected_source,first_wrong:$b.first_wrong,exact:$b.exact_eos},expected,emitted:$b.generated}]}]
