# Inputs are the immutable primary selections, independent key and raw original IDs.
($tokens[0].model.vocab | to_entries | map({key:(.value|tostring),value:.key}) | from_entries) as $v |
[.rows[] as $r | $labels[0][$r.index] as $l |
 select($l.answer_text!=null and $r.paired.choice.query==null) |
 ($r.candidates|map(select(.admitted))) as $admitted |
 ($admitted|map(.scores[0])|max) as $best |
 {id:$l.id,requested_span:$l.requested,requested_name:$l.requested_name,selected_query:$r.paired.choice.query,
  best:$best, tied:[$admitted[]|select(.scores[0]==$best)|. as $c |
  ($blind[0][$r.index].prefix[$c.span[0]:$c.span[1]]) as $ids |
  {span:$c.span,ids:$ids,pieces:[$ids[]|$v[tostring]],scores:$c.scores,keys:$c.feature_keys}]}]
