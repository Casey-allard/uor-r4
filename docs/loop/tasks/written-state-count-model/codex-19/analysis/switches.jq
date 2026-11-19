# Run after scoring. Paired directions share facts in their original order.
def measure($arm; $labels):
  [ .rows[] | select(.stratum=="supported") | . as $r |
    $labels[.index] as $g |
    if .id != $g.id or ($g.question_mentions|length)!=2 then error("switch grade misaligned") else . end |
    ($g.question_mentions|sort_by(.[1][0])) as $names |
    ($r.outputs[$arm].requested) as $span |
    {index:.index, key:[.family,.reversed,.the,.about,.distractor_first],
     direction:.direction, gold:$g.requested_name,
     selected:([$names[]|select(.[1]==$span)|.[0]][0]),
     first:($span==$names[0][1]),last:($span==$names[1][1]),exact:$r.outputs[$arm].exact}
  ] as $rows |
  ($rows|group_by(.key)|map(sort_by(.direction) |
    if length!=2 or map(.direction)!=[0,1] or .[0].gold==.[1].gold then error("direction pair invalid") else . end |
    {indices:map(.index),actual:map(.selected),wanted:map(.gold),
     type:(if any(.[]; .selected==null) then "one_or_both_outside_query_names"
           elif all(.[]; .selected==.gold) then "both_correct"
           elif .[0].selected==.[1].selected then "same_named_person"
           else "both_distractors" end),both_replies_exact:all(.[];.exact)})) as $pairs |
  if ($rows|length)!=64 or ($pairs|length)!=32 then error("missing direction contrasts") else
    {first_query_name:([$rows[]|select(.first)]|length),
     last_query_name:([$rows[]|select(.last)]|length),
     neither_query_name:([$rows[]|select(.first==false and .last==false)]|length),
     pairs_by_result:($pairs|group_by(.type)|map({key:.[0].type,value:length})|from_entries),
     pairs_both_replies_exact:([$pairs[]|select(.both_replies_exact)]|length),pairs:$pairs}
  end;
{schema:"uor-r4.requested-occurrence-switches/1",seed:.seed,
 ordered:measure("ordered";$labels[0]),treatment:measure("treatment";$labels[0])}
