def hist(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
. as $a | $parent[0] as $p | if length!=($p|length) then error("agreement count mismatch") end |
[range(length) as $i | $a[$i] as $r | $p[$i] as $o |
 if $r.prefix_sha256!=$o.prefix_sha256 or ($r.source==$o.source and $r.generated!=$o.generated) then error("agreement pair differs") end |
 $r+{parent_first:$o.first,parent_source:$o.source,parent_generated:$o.generated,parent_copy:$o.copied}] as $rows |
if $form=="rows" then $rows else
{by_kind:($rows|group_by(.kind)|map({key:.[0].kind,value:{rows:length,
 exact_from_unannotated_address:map(select(.first=="exact" and (.good_address|not)))|length,
 parent_exact:map(select(.parent_first=="exact"))|length,exact:map(select(.first=="exact"))|length,
 parent_false_copy:map(select(.supported==false and .parent_copy))|length,
 false_copy:map(select(.supported==false and .copied))|length,
 movements:(map(select(.expected!=null))|hist((.parent_first|tostring)+"->"+(.first|tostring)))}})|from_entries),
 examples:([$rows[]|select(.first=="exact" and .parent_first!="exact")][0:2] +
 [$rows[]|select(.first!="exact" and .parent_first=="exact")][0:2] +
 [$rows[]|select(.supported==false and .parent_copy==false and .copied)][0:2] |
 map({id,kind,prompt,expected,parent_generated,generated,parent_first,first,parent_source,source,prefix_sha256}))}
end
