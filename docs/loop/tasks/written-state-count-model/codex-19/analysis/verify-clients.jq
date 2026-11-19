# Input: the 70 ordinary client JSON records sorted by their zero-padded ordinal filenames.
def first_error($expected; $emitted; $stop):
  [range(0;([$expected,$emitted]|map(length)|max)) as $i
    | select($expected[$i]!=$emitted[$i]) | $i][0] as $mismatch |
  if $mismatch!=null then $mismatch elif $stop!="eos" then ($expected|length) else null end;
if length!=70 or ($labels[0]|length)!=70 or ($blind[0]|length)!=70 then error("row count differs") else . end |
to_entries|map(.key as $i | .value as $c | $labels[0][$i] as $g | $summary[0].rows[$i] as $s |
  if $g.id!=$s.id or $blind[0][$i].prefix!=$c.prompt_ids or $s.outputs[$arm].generated!=$c.generated_ids
    then error("client, reference or graded tape misaligned at \($i)") else . end |
  (if $c.paired.role_words[0]==4294967295 then null else $c.paired.role_words[:2] end) as $req |
  if $req!=$s.outputs[$arm].requested or $c.selected.source!=$s.outputs[$arm].source
    then error("client and recorded source or requested occurrence differ at \($i)") else . end |
  (if $g.stratum=="supported" then first_error($g.expected;$c.generated_ids;$c.stop) else null end) as $wrong |
  if $g.stratum=="supported" and ($wrong!=$s.outputs[$arm].first_wrong or (($wrong==null)!=$s.outputs[$arm].exact))
    then error("independent first error or EOS grade disagrees at \($i)") else . end |
  {index:$i,gold_occurrence:($g.stratum=="supported" and $g.requested==$req),supported:($g.stratum=="supported"),
   source:$c.selected.source,first_wrong:$wrong,exact:($g.stratum=="supported" and $wrong==null),stop:$c.stop}) |
{rows:length,supported:map(select(.supported))|length,role_exact:map(select(.gold_occurrence))|length,
 exact_replies:map(select(.exact))|length, first_wrong:([.[]|select(.supported and .exact==false)]|group_by(.first_wrong)|
   map({index:.[0].first_wrong,count:length})), unsupported_writes:map(select((.supported|not) and .source!=null))|length}
