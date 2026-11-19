[ .[] | select(.answer_text!=null) | . as $l | .facts[.gold_assertion_index].legal |
  map(select((.entire_copy | [scan("[^[:space:]]+")] | length)==$n)) |
  {id:$l.id,starts:map(.start),expected:$l.gold_start,correct:(length==1 and .[0].start==$l.gold_start)}
] | {words:$n, rows:length, unique_correct:map(select(.correct))|length, details:.}
