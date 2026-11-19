def n(f): map(select(f))|length;
def point($p): split("\t") | map(tonumber) |
  if length==4 and (.[0]>=-1 and .[0]<=2) and all(.[1:][];.>=0) then .[0]
  else error("invalid prediction on row "+($p|tostring)) end;
def requested($key): map(select(.[$key]==1)|.person)|unique;
def classified($key):
  {exact_set:n(.gold==.[$key]), positive_exact:n((.gold|length)>0 and .gold==.[$key]),
   full_classes:n(all(.mentions[];.truth==.[$key])), false_positive_empty:n(.gold==[] and .[$key]!=[]),
   all_abstained:n(all(.mentions[];.[$key]==-1)), sets_by_size:
      [range(3) as $k|map(select((.gold|length)==$k))|
       {size:$k,prompts:length,exact:n(.gold==.[$key])}],
   confusion:([.[].mentions[]] as $m|[range(3) as $g |
      {truth:$g, actual_order:["unspecified","requested","excluded","abstain"],
       counts:([0,1,2,-1]|map(. as $p|$m|n(.truth==$g and .[$key]==$p)))}])};
def counts:
  {prompts:length, mentions:(map(.mentions|length)|add),
   identity:classified("identity"), positional:classified("positional"),
   controls:{empty:n(.gold==[]),first:n(.gold==[.mentions[0].person]),last:n(.gold==[.mentions[-1].person])},
   gained:n(.gold!=.identity and .gold==.positional), lost:n(.gold==.identity and .gold!=.positional)};
def pairs($field):
  map(select(.[$field]!=null)) | group_by(.[$field]) |
  if any(.[];length!=2) then error("unpaired: "+$field) else . end |
  {pairs:length,gold_changed:n(.[0].gold!=.[1].gold),
   identity_changed:n(.[0].identity!=.[1].identity),positional_changed:n(.[0].positional!=.[1].positional),
   identity_both:n(all(.[];.identity==.gold)),positional_both:n(all(.[];.positional==.gold))};
($identity|rtrimstr("\n")|split("\n")) as $i |
($positional|rtrimstr("\n")|split("\n")) as $p |
($truth|rtrimstr("\n")|split("\n")|map(tonumber)) as $g |
if ([$rows,$g,$i,$p]|map(length)|unique|length)!=1 or ($g|length)==0 then error("row mismatch") else . end |
[range($rows|length) as $k|$rows[$k]+{truth:$g[$k],identity:($i[$k]|point($k)),positional:($p[$k]|point($k))}] |
group_by(.prompt_id) | map(sort_by(.span) as $m |
 if (length!=2 or ($m|map(.person)|unique|length)!=2) then error("not two distinct mentions") else . end |
 $m[0] as $a | {prompt_id:$a.prompt_id,family:$a.family,polarity_pair:$a.polarity_pair,clause_order_pair:$a.clause_order_pair,
 fact_order_pair:$a.fact_order_pair,name_variant_pair:$a.name_variant_pair,
 ambiguous:$a.ambiguous,mentions:$m,gold:($m|requested("truth")),identity:($m|requested("identity")),positional:($m|requested("positional"))}) |
. as $all | map(select(.ambiguous!=true)) |
{rows:., ambiguity:($all|map(select(.ambiguous==true)) |
 {prompts:length,positive_count_histograms:(. as $a|["identity","positional"]|map(. as $arm|
 {arm:$arm,counts:([0,1,2]|map(. as $k|$a|n((.[$arm]|length)==$k)))}))}),
 total:counts,by_family:(group_by(.family)|map({family:.[0].family,results:counts})),
 pairs:(. as $r| ["polarity_pair","clause_order_pair","fact_order_pair","name_variant_pair"] |
 map(. as $key|{factor:$key,results:($r|pairs($key))})),
 by_source_support:([.[].mentions[]] | group_by(.assertion_supported) | map({supported:.[0].assertion_supported,
   mentions:length, requested:n(.truth==1), identity_correct_requested:n(.truth==1 and .identity==1),
   positional_correct_requested:n(.truth==1 and .positional==1)}))}
