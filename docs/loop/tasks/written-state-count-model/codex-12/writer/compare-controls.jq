# Compare final paired-role + fence against the independently rolled, frozen writer arms.
def first($got;$want): if $got==$want then "exact" else [range([($got|length),($want|length)]|max) | select($got[.] != $want[.])][0] end;
def hist: group_by(.) | map({key:.[0],value:length})|from_entries;
. as $arm | $joint[0] as $joint | $grades[0] as $g |
if $arm.model!=$joint.model or $arm.bare!=$joint.bare or ($g|length)!=($joint.rows|length) or ($arm.rows|length)!=($g|length)
then error("tape length or carrier mismatch") end |
[range($g|length) as $i|$g[$i] as $k|$arm.rows[$i] as $a|$joint.rows[$i] as $j|
 if $a.prefix_sha256!=$j.prefix_sha256 then error("prompt mismatch") end|
 {pair:$k.pair,expected:$k.expected,oldcopy:($a.steps|any(.copied==1)),newcopy:($j.steps|any(.copied==1)),
 old:(if $k.expected!=null then first($a.generated;$k.expected) end), new:(if $k.expected!=null then first($j.generated;$k.expected) end)}] as $r |
{split:$split,seed:$n,control:$name,rows:($r|length),scorable:($r|map(select(.expected!=null))|length),
 control_exact:($r|map(select(.old=="exact"))|length),joint_exact:($r|map(select(.new=="exact"))|length),
 recovered:($r|map(select(.old!="exact" and .new=="exact"))|length),lost:($r|map(select(.old=="exact" and .new!="exact"))|length),
 control_unsupported_copy:($r|map(select(.expected==null and .oldcopy))|length),
 joint_unsupported_copy:($r|map(select(.expected==null and .newcopy))|length),
 first_wrong_transitions:($r|map(select(.expected!=null)|"\(.old)->\(.new)")|hist),
 both_directions:($r|map(select(.expected!=null))|group_by(.pair)|map(select(length==2))|
 {pairs:length,control_both:(map(select(all(.[];.old=="exact")))|length),joint_both:(map(select(all(.[];.new=="exact")))|length)})}
