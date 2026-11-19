def verify($b): if $b then . else error("unbound row, native trace or grade") end;
[.arms[]|select(.arm=="learned-normal-ordered")]|verify(length==1)|.[0].positive_rows as $rows |
[$rows[] as $g | [$s[0].rows[]|select(.ordinal==$g.index)] as $sites |
  [$sites[]?.candidates[]?|select(.source==$g.gold_source)] as $cand |
  {index:$g.index,id:$g.id,matched:($g.matched_triple!=null),baseline:$g.earliest_stage,
   gold:$g.gold_source,sites:($sites|length),observations:($cand|length),gold_rank_status:
   (if ($cand|length)!=1 then "unresolved" else $cand[0]|
    if .status!="scored" then .status elif .margin.numerator>0 then "positive"
    elif .margin.numerator==0 then "zero" else "negative" end end)}] as $r |
{rows:$r,counts:($r|map(select(.matched))|group_by([.baseline,.gold_rank_status])|
  map({stage:.[0].baseline,gold:.[0].gold_rank_status,count:length,ordinals:map(.index)}))}
