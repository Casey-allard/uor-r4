# Supply a sealed ROLE handoff as input and its matched grade tape via --slurpfile grad.
.rows as $r | $grad[0] as $g |
[range($g|length) as $i | $g[$i].token_positions as $t | $r[$i] as $v |
 (if $v.frozen.query_start==$t.query_start then "requested"
  elif $t.query_other_start!=null and $v.frozen.query_start==$t.query_other_start then "excluded"
  else "outside" end) as $q |
 (if $v.frozen.fact_start==$t.fact_start then "requested"
  elif $v.frozen.fact_start==$t.other_fact_start then "other" else "outside" end) as $f |
 {query:$q,fact:$f,correct_source:($v.continuation.source==$g[$i].asked_start)}] |
group_by(.)|map(.[0]+{count:length})
