def wrong($got;$expected): if $expected==null then null else
  first(range(0;([$got,$expected]|map(length)|max)) | select($got[.]!=$expected[.])) // "exact" end;
def histogram(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def summary: {rows:length,scored:(map(select(.expected!=null))|length),
  exact_eos:(map(select(.first=="exact"))|length),issued:map(select(.source!=null))|length,
  unexpected_read:map(select(.supported==false and .source!=null))|length,
  copied:map(select(.copied))|length,unexpected_copy:map(select(.supported==false and .copied))|length,
  requested_value_address:map(select(.supported and .good_address))|length,
  role_to_arm:(map(select(.expected!=null))|histogram((.role_first|tostring)+"->"+(.first|tostring))),
  old_to_arm:(map(select(.expected!=null))|histogram((.old_first|tostring)+"->"+(.first|tostring)))};
. as $a | $grades[0] as $g | $role[0] as $r | $old[0] as $o | $none[0] as $no |
if any([$a,$r,$o,$no][];.model!=$a.model or .bare!=$a.bare or (.rows|length)!=($g|length)
 or .writer_cubin!=$a.writer_cubin or .completion_cubin!=$a.completion_cubin
 or .transitions!=([.rows[].steps|length]|add)) then error("model, tape or writer mismatch") end |
[range($g|length) as $i | $g[$i] as $k | $a.rows[$i] as $w |
 $r.rows[$i] as $base | $o.rows[$i] as $prev | $no.rows[$i] as $n |
 if ([$base,$prev,$w,$n]|map(.prefix_sha256)|unique|length)!=1 or $n.source!=null
  or $w.generated!=[$w.steps[].emitted] or ($w.generated[-1]!=1 and ($w.steps|length)!=16)
  or ($w.source==null and $w!=$n) or ($w.source==$base.source and $w!=$base) then error("unpaired trajectory") end |
 ($k.kind//"positive") as $kind | ($k.asked_start!=null) as $supported |
 (if $k.supports!=null then [$k.supports[].value_start] else [$k.asked_start] end) as $valid |
 {id:$k.id,pair:$k.pair,contrast:$k.contrast,role:$k.role,form:$k.query_form,kind:$kind,supported:$supported,source:$w.source,
  good_address:($supported and ($w.source as $s|$valid|index($s)!=null)),copied:any($w.steps[];.copied==1),
  first:wrong($w.generated;$k.expected),role_first:wrong($base.generated;$k.expected),old_first:wrong($prev.generated;$k.expected),
  generated:$w.generated,expected:$k.expected,old_generated:$prev.generated,role_generated:$base.generated,
  first_free:$w.first_free,prompt:$k.prompt,prefix_sha256:$w.prefix_sha256}] as $rows |
if ([$rows[].prefix_sha256]|unique|length)!=($rows|length) then error("duplicate prefix") end |
if $mode=="rows" then $rows else
{panel:$panel,arm:$arm,model:$a.model,bare:$a.bare,transitions:$a.transitions,overall:($rows|summary),
 by_kind:($rows|group_by(.kind)|map({key:.[0].kind,value:summary})|from_entries),
 by_kind_form:($rows|map(select(.form!=null))|group_by([.kind,.form])|
    map({key:(.[0].kind+"/"+(.[0].form|tostring)),value:summary})|from_entries),
 fixed_fact_role_pairs:($rows|map(select(.supported and .contrast!=null))|group_by(.contrast)|map(select(length==2 and ([.[].role]|unique|length)==2))|{pairs:length,exact:map(select(all(.[];.first=="exact")))|length}),
 both_roles_exact:($rows|map(select(.supported and .pair!=null))|group_by(.pair)|
   map(select(length==2))|{pairs:length,exact:map(select(all(.[];.first=="exact")))|length})}
end
