# Inputs: one complete sealed predictor output and the corresponding grade.
def n(f): map(select(f))|length;
def tally:
  {total:length, word_start:n(.w),continuation:n(.c),gained:n(.c and (.w|not)),
   lost:n(.w and (.c|not)), exact_witness:n(.reason==1), geometric_witness:n(.reason==2)};
($grades[0]) as $g |
if (.rows|length)!=($g|length) then error("invalid source census size") else . end |
[range(0;$g|length) as $i | .rows[$i] as $r | $g[$i] as $q |
  $r.continuation as $c | $r.word_start as $w | $q.token_positions as $p |
  if (($w==null)!=($c.source==null)) or ($w!=null and ($c.source<$w or $c.source>$w+2))
     or (($c.reason==0)!=($c.source==$w)) or ($c.reason<0 or $c.reason>2)
  then error("forward-only continuation or witness invariant broken") else . end |
  {family:($q.factors.asked_connector_surface // ("IDs "+($q.prefix[$p.fact_end+1:$q.asked_start]|map(tostring)|join(" ")))),
   w:($w==$q.asked_start), c:($c.source==$q.asked_start),reason:$c.reason,
   aligned:([$r.frozen.fact_start,$r.frozen.fact_end,$r.frozen.query_start,$r.frozen.query_end]
         == [$p.fact_start,$p.fact_end,$p.query_start,$p.query_end])}] |
{all:tally, per_witness:(group_by(.reason)|map({reason:.[0].reason}+tally)),
 aligned_per_surface:(map(select(.aligned))|group_by(.family)|
 map({surface:.[0].family,total:tally,witnesses:(group_by(.reason)|map({reason:.[0].reason}+tally))}))}
