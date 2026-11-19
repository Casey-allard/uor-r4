def gq($g): $g.query_start // $g.token_positions.query_start // (
 if $g.kind=="positive" or $g.kind=="absent" then
   (if $g.kind=="positive" then $g.role else $g.absent_name end) as $name |
   $tok[0].model.vocab["Ġ"+$name] as $id | [$g.prefix|to_entries[]|select(.value==$id)|.key]|last
 else null end);
def sumclass:
 {rows:length,query_defined:map(select(.q!=null))|length,query_correct:map(select(.q!=null and .qn==.q))|length,
  eligible_support:map(select(.num!=null))|map(.num)|group_by(.)|map({count:.[0],rows:length}),
  emitted:map(select(.src!=null))|length,source_eligible:map(select(.answerable))|length,
  right_source:map(select(.answerable and .oksrc))|length,wrong_source:map(select(.answerable and .src!=null and (.oksrc|not)))|length,
  copied_with_no_answer:map(select((.answerable|not) and .src!=null))|length,
  right_request_wrong_source:map(select(.q!=null and .qn==.q and .answerable and (.oksrc|not)))|length,
  missed_request:map(select(.q!=null and .qn!=.q))|length};
$g[0] as $gs|$a[0].rows as $rs| [range($gs|length) as $i|$gs[$i] as $g|$rs[$i] as $r|
 {kind:($g.kind//"positive"),q:gq($g),qn:$r.nominee,num:$r.count,src:$r.source,answerable:($g.asked_start!=null),
  oksrc:(($r.source!=null) and (if ($g.supports//[]|length)>0 then [$g.supports[].value_start]|index($r.source)!=null else $g.asked_start==$r.source end)),
  prompt:$g.prompt,name_id:($g.prefix[$r.nominee//0]//null),role:$g.role,expected:$g.asked_start, id:$g.id}] as $all|
{total:($all|sumclass),classes:($all|group_by(.kind)|map({key:.[0].kind,value:sumclass})|from_entries),
 "Regarding_name":([$all[] | select(.kind=="positive" and (.prompt|contains("what did the entry name?")))] as $r |
 {rows:($r|length),correct_request:($r|map(select(.qn==.q))|length),name_as_nominee:($r|map(select(.name_id==1412))|length),correct_source:($r|map(select(.oksrc))|length)}),
 examples:($all|group_by([.kind,(.qn==.q),(.src!=null),.oksrc])|map(.[0])|map(del(.answerable,.name_id)))}
