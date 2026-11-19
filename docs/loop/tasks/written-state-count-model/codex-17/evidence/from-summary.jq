# Compact independent grades for human review. Read only the completed paired summary.
def unique_arm($n): [.arms[]|select(.arm==$n)] | if length==1 then .[0] else error("missing or duplicate arm") end;
def direction($l;$r):
  if $l.exact and $r.exact then "both_exact"
  elif $r.exact then "recovered" elif $l.exact then "regressed"
  elif $l.first_error.index<$r.first_error.index then "later"
  elif $l.first_error.index>$r.first_error.index then "earlier" else "same_index" end;
def compare($b;$a):
  [$a.positive_rows[] as $r | ($b.positive_rows[]|select(.index==$r.index)) as $l |
   if $l.id!=$r.id then error("different prompt") else
      {ordinal:$r.index,matched:($r.matched_triple!=null),movement:direction($l;$r)} end] as $c |
  if ($c|length)!=36 then error("incomplete comparison") else . end |
  {before:$b.arm,after:$a.arm,all_36:($c|group_by(.movement)|map({movement:.[0].movement,ordinals:map(.ordinal)})),
   matched_24:($c|map(select(.matched))|group_by(.movement)|map({movement:.[0].movement,ordinals:map(.ordinal)}))};
. as $study | unique_arm("frozen-normal-public") as $base |
{schema:"uor-r4.held-value-rank-results/1", seed:$seed, summary_sha256:$summary_hash,
 arms:[.arms[]|{arm,exact_36:.all_36.exact,exact_matched_24:.matched_24.exact,
      stopped_at_eos_36:.all_36.terminal_eos,error_index_counts:.all_36.first_error_index_counts,
      matched_error_index_counts:.matched_24.first_error_index_counts,first_failure:.all_36.earliest_stage_counts,
      matched_first_failure:.matched_24.earliest_stage_counts}],
 movements:[.arms[]|select(.arm|startswith("learned-normal-"))|compare($base;.)],
 supported:[$base.positive_rows[] as $row |
      {ordinal:$row.index,id:$row.id,matched_triple:$row.matched_triple,gold_source:$row.gold_source,
       arms:[$study.arms[]|select(.arm=="frozen-normal-public" or (.arm|startswith("learned-normal-")))|
        .arm as $arm | .positive_rows[]|select(.index==$row.index)|
        {arm:$arm,source:.selected_source,requested,exact,eos:.terminal_eos,first_error,earliest_stage}]}],
 negative:[.arms[]|select(.arm=="frozen-normal-public" or (.arm|startswith("learned-normal-")))|
      {arm, rows:[.negative_controls[]|{ordinal:.index,id,semantics,decoded_text,
       terminal_eos,classification:.negative.classification}]}]}
