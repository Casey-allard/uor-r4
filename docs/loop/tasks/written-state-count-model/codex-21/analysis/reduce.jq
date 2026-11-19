def index_rows: .rows | map({key:(.index|tostring),value:.}) | from_entries;
def table($raw): $raw[0] | index_rows;
def output($g; $x; $device):
  if $x==null then null
  elif $g.id!=$x.id then error("grade order mismatch")
  else {query:$x.requested,excluded:$x.excluded,source:$x.source,generated:$x.generated,text:$x.text,
        eos:$x.terminal_eos,exact:$x.exact,wrong:$x.first_wrong,expected_id:$x.expected_wrong_id,actual_id:$x.actual_wrong_id,
        span_ok:($g.requested!=null and $g.requested==$x.requested),
        source_ok:($x.source!=null and $g.gold_start==$x.source),evidence:$device.requested_occurrence} end;
def counts($arm):
  map(.arms[$arm]|select(.!=null)) |
  {rows:length,spans:map(select(.span_ok))|length,source:map(select(.source_ok))|length,
   no_source:map(select(.source==null))|length,wrong_source:map(select(.source!=null and (.source_ok|not)))|length,
   exact:map(select(.exact))|length,eos:map(select(.eos))|length,
   first_wrong:(map(select(.wrong!=null))|group_by(.wrong)|map({key:(.[0].wrong|tostring),value:length})|from_entries)};
table($reference) as $r | table($treatment) as $t | table($supplied) as $s |
table($veto) as $v | table($wrong_role) as $w | table($no_read) as $n |
[$labels[0]|to_entries[] | .key as $i | ($i|tostring) as $k | .value as $g |
  if $r[$k]==null or $t[$k]==null or $n[$k]==null or ($g.stratum=="supported" and ($s[$k]==null or $v[$k]==null or $w[$k]==null))
  then error("ungraded row") else
  {i:$i,id:$g.id,stratum:$g.stratum,family:$g.family,clause_exclusion_first:$g.distractor_first,name_b_first:$g.name_order_reversed,
   reverse_facts:$g.reversed,object_swap:$g.object_swap,connector_swap:$g.connector_swap,
   person:$g.requested_name,prompt:$blind[0][$i].text,gold_query:$g.requested,rival_query:$g.excluded,gold_start:$g.gold_start,
   expected:$g.expected,facts:$g.facts,
   arms:{reference:output($g;$r[$k];$reference_candidates[0].rows[$i]),clause:output($g;$t[$k];$treatment_candidates[0].rows[$i]),
         supplied:output($g;$s[$k];null),veto:output($g;$v[$k];null),wrong_role:output($g;$w[$k];null),no_read:output($g;$n[$k];null)}}
  end] as $rows |
($rows | map(select(.stratum=="supported"))) as $sup |
{schema:"uor-r4.clause-entity-veto-summary/1",seed:$seed,rows:$rows,
 supported:(["reference","clause","supplied","veto","wrong_role","no_read"]|map(. as $a|{key:$a,value:($sup|counts($a))})|from_entries),
 positional:($sup|{rows:length,first:map(select(.gold_query[0]<.rival_query[0]))|length,last:map(select(.gold_query[0]>.rival_query[0]))|length}),
 cells:($sup|group_by([.family,.clause_exclusion_first,.name_b_first])|map({key:(.[0]|[.family,.clause_exclusion_first,.name_b_first]),reference:counts("reference"),clause:counts("clause"),supplied:counts("supplied"),veto:counts("veto")}))}
