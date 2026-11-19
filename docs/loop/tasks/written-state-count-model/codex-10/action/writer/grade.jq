def first_wrong($a;$b):
  first(range(0;([$a,$b]|map(length)|max)) | select($a[.]!=$b[.])) // "exact";
def frequency(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
. as $a|$grades[0] as $g|$role[0] as $r|$none[0] as $n|$pred[0] as $p|($g|length) as $len |
if $a.arm!="action" or $r.arm!="role" or $n.arm!="no-read" or ($p.rows|length)!=$len
 or any([$a,$r,$n][];.model!=$p.artifact_sha256 or .bare!=$p.bare_sha256 or (.rows|length)!=$len
 or .transitions!=([.rows[].steps|length]|add))
 or ($g|map(.id)|unique|length)!=$len or ($a.rows|map(.prefix_sha256)|unique|length)!=$len
 then error("inconsistent gated output panel") end |
[range($len) as $i|$g[$i] as $k|$a.rows[$i] as $ar|$r.rows[$i] as $rr|$n.rows[$i] as $nr|$p.rows[$i] as $choice|
 ($k.kind//"positive") as $kind|($choice.allowed) as $ok|
 if ($ok|type)!="boolean" or ([$choice.prefix_sha256,$ar.prefix_sha256,$rr.prefix_sha256,$nr.prefix_sha256]|unique|length)!=1 then error("unpaired action") end |
 if $ar!=(if $ok then $rr else $nr end) or $ar.source!=$choice.source or $nr.source!=null or $ar.generated!=[$ar.steps[].emitted] or
    ($ar.generated[-1]!=1 and ($ar.steps|length)!=16) then error("action altered writer state: "+$k.id) end |
 (if ($forced|length)>0 then $forced[0].rows[$i] else null end) as $f |
 if $f!=null and ($f.prefix_sha256!=$ar.prefix_sha256 or $f.source!=($k.forced_fact_source//4)) then error("forced read mismatch") end |
 {id:$k.id,kind:$kind,pair:$k.pair,expected:$k.expected,prompt:$k.prompt,allowed:$ok,
  proposed:$rr.source,issued:$ar.source,copied:any($ar.steps[];.copied==1),role_copy:any($rr.steps[];.copied==1),
  generated:$ar.generated,role_generated:$rr.generated,no_read_generated:$nr.generated,
  role_wrong:(if $k.expected!=null then first_wrong($rr.generated;$k.expected) else null end),
  action_wrong:(if $k.expected!=null then first_wrong($ar.generated;$k.expected) else null end),
  no_read_wrong:(if $k.expected!=null then first_wrong($nr.generated;$k.expected) else null end),
  forced_copy:(if $f!=null then any($f.steps[];.copied==1) else null end),
  forced_wrong:(if $f!=null and $k.expected!=null then first_wrong($f.generated;$k.expected) else null end)}
] as $rows |
if $form=="rows" then $rows else
 {panel:$panel,transitions:$a.transitions,scored:($rows|length),
  classes:($rows|group_by(.kind)|map({key:.[0].kind,value:{
    rows:length,role_candidate:map(select(.proposed!=null))|length,issued:map(select(.issued!=null))|length,
    generated_ordinarily:map(select(.issued==null))|length,role_copied:map(select(.role_copy))|length,
    actual_copied:map(select(.copied))|length,forced_copied:map(select(.forced_copy==true))|length,
    rows_with_reference:map(select(.expected!=null))|length,
    literal_exact_role:map(select(.role_wrong=="exact"))|length,literal_exact_action:map(select(.action_wrong=="exact"))|length,
    literal_exact_no_read:map(select(.no_read_wrong=="exact"))|length,literal_exact_forced:map(select(.forced_wrong=="exact"))|length,
    exact_gained:map(select(.role_wrong!="exact" and .action_wrong=="exact"))|length,
    exact_lost:map(select(.role_wrong=="exact" and .action_wrong!="exact"))|length,
    first_error_transitions:(map(select(.expected!=null))|frequency((.role_wrong|tostring)+"->"+(.action_wrong|tostring)))}})|from_entries),
  ordinary_examples:([$rows[]|select(.id|contains("-ordinary-"))|{id,prompt,expected,generated,issued}])}
end
