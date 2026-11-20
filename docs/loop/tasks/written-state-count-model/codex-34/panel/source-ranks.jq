# Read frozen serving traces; no model invocation. Inspect ranks of known sourced answers offline.
. as $rows | [$joined[0].rows[] | select(.stratum=="crossed") as $j |
  $j["full-detached"] as $arm | ($rows | map(select(.id==$j.row)) | if length!=1 then error("row mismatch") else first end) as $trace |
  ($trace.trace | map(select(.fact_span==$arm.assertion.expected_fact_span)) |
    if length!=1 then error("missing bound assertion") else first end) as $bound |
  ($bound.candidates|map(select(.[1].eligible))) as $c |
  ($c|map(select(.[1].source==$j.gold_source))|if length!=1 then error("missing eligible gold") else first end) as $gold |
  {row:$j.row,gold:$j.gold_source,selected:$arm.selected_source,margin:$gold[1].margin,
   eligible:($c|length),strictly_higher:([$c[]|select(.[1].margin>$gold[1].margin)]|length),
   same_margin_rivals:([$c[]|select(.[1].source!=$j.gold_source and .[1].margin==$gold[1].margin)]|length),
   identical_feature_rivals:([$c[]|select(.[1].source!=$j.gold_source and .[0].local==$gold[0].local and .[0].contextual==$gold[0].contextual)]|map(.[1].source)),
   outcome:$arm.assertion.local_pick.outcome,first_error:$arm.first_error}] as $result |
if ($result|length)!=16 then error("incomplete panel") else
{schema:"uor-r4.d34.frozen-value-ranks/1",rows:$result,
 summary:($result|group_by(.outcome)|map({outcome:.[0].outcome,n:length,exact:map(select(.first_error==null))|length})),
 wrong_rows_with_higher:([$result[]|select(.first_error!=null and .strictly_higher>0)]|length),
 wrong_rows_with_top_tie:([$result[]|select(.first_error!=null and .strictly_higher==0 and .same_margin_rivals>0)]|length),
 wrong_rows_with_strictly_best:([$result[]|select(.first_error!=null and .strictly_higher==0 and .same_margin_rivals==0)]|length),
 wrong_rows_with_identical_rival_features:([$result[]|select(.first_error!=null and (.identical_feature_rivals|length)>0)]|length)}
end
