# This diagnostic sees the keys only after the model observations were frozen.
def require($p;$why): if $p then . else error($why) end;
def summarize(key):
  group_by(key)| map(. as $g | (group_by(.role)|map({role:.[0].role,n:length})) as $h |
    {rows:length,labels:$h,examples:[$g[]|{row,role,span}],best:($h|map(.n)|max)}) as $groups |
  {unique_observations:($groups|length),present:($groups|map(.rows)|add//0),
   upper_bound_correct:($groups|map(.best)|add//0),
   conflicting_groups:[$groups[]|select((.labels|length)>1)]};
. as $c | [$key[0][]|select(.stratum=="crossed")] as $rows |
require(($rows|length)==16 and ($c|length)==16;"need every crossed row once") |
[$rows[] as $k | ($c|map(select(.row==$k.row))|require(length==1;"duplicate or missing capture")|first) as $record |
  require($record.result.prompt_ids==$k.original_ids;"prompt mismatch") |
  ($k.role_targets|to_entries[]) as $t | ($record.result.role_replay.candidates|map(select(.span==$t.value))) as $m |
  require(($m|length)<=1;"duplicate candidate span") |
  if ($m|length)==0 then {row:$k.row,family:$k.family,role:$t.key,span:$t.value,missing:true}
  else $m[0] as $o | require(($o.feature_keys|length)==6 and all(range(0;6); . as $i | ($o.feature_keys[$i]/65537|floor)==$i);"unexpected role feature encoding") |
    {row:$k.row,family:$k.family,role:$t.key,span:$t.value,missing:false,scores:$o.scores,
     ordered:$o.feature_keys,neutral:($o.feature_keys|map(.%65537)|sort),supporting_head:$o.supporting_head} end] as $mentions |
($mentions|map(select(.missing==false))) as $seen |
{schema:"uor-r4.d34.exposed-role-collisions/1",mentions:$mentions,
 denominator:($mentions|length),present_by_role:($mentions|group_by(.role)|map({role:.[0].role,present:(map(select(.missing==false))|length),total:length})),
 ordered:($seen|summarize(.ordered)),offset_neutral:($seen|summarize(.neutral)),frozen_scores:($seen|summarize(.scores))}
