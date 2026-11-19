# Human-readable post-scoring view of the UTF-8 ASCII-language holdout.
def surface($span;$ids;$pieces):
  if $span==null then null else
    ($ids[$span[0]:$span[1]] | map($pieces[.]) | join("") | gsub("Ġ";" ") | gsub("Ċ";"\n") | ltrimstr(" ")) as $s
    | if ($s|explode|all(.<128)) then $s else error("unhandled non-ASCII span") end end;
def arm($name;$row;$ids;$pieces):
  $row.outputs[$name] as $r |
  {span:$r.requested,word:surface($r.requested;$ids;$pieces),source:$r.source,
   excluded:$r.excluded,span_correct:($r.role_class=="gold_query_span"),first_wrong:$r.first_wrong,
   expected_wrong_id:$r.expected_wrong_id,actual_wrong_id:$r.actual_wrong_id,exact_eos:$r.exact};
($tokenizer[0].model.vocab|to_entries|sort_by(.value)|map(.key)) as $pieces |
[.rows[] | select(.stratum=="supported") | . as $row | $labels[0][.index] as $g |
  $blind[0][.index] as $b | if .id!=$g.id then error("different cases") else . end |
  {index,id,query:$b.text,requested_name:$g.requested_name,gold_requested,gold_source,
   expected_literal:$g.answer_text,first_fact:$g.facts[0].name,the,about,distractor_first,
   baseline:arm("ordered";$row;$b.prefix;$pieces),treatment:arm("treatment";$row;$b.prefix;$pieces),
   supplied:arm("supplied";$row;$b.prefix;$pieces)}]
