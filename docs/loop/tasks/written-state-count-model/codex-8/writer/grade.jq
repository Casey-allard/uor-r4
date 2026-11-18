# Join sealed grade-free rollouts to the external key only after the GPU run.
def mismatch($got;$want): first(range(0;([$got,$want]|map(length)|max))|select($got[.]!=$want[.]))//null;
def idx: if .==null then "exact" else tostring end;
def direction($a;$b): if $a==$b then "same" elif $b==null then "later" elif $a==null then "earlier"
  elif $a<$b then "later" else "earlier" end;
def counts(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def n(f): map(select(f))|length;
def check($g):
  . as $r | $g.prefix as $p | $g.expected as $want |
  if $r.generated!=[$r.steps[].emitted] or ($r.steps|length)>16 or ($r.steps|length)==0
     or $want!=($p[$g.asked_start:$g.asked_end]+[1]) or $g.other_start==$g.asked_start
     or ([$r.generated[:-1][]|select(.==1)]|length)>0
     or ($r.generated[-1]!=1 and ($r.generated|length)!=16)
  then error("inconsistent grade/emission on "+$g.id) else . end |
  (reduce $r.steps[] as $s (
    {state:[(if $r.source==null then 0 else 1 end),($r.source//4294967295),($r.source//4294967295),($p|length)],pos:0,first_free:null};
    .state as [$active,$source,$cursor,$end] |
    (if $active==0 then 0 elif $cursor-$source>=8 then 3 elif $cursor>=$end then 2
     elif ([16,1,0,201]|index($p[$cursor]))!=null then 1 else 0 end) as $release |
    ($active==1 and $release==0) as $copy | ($release==1 and $cursor>$source) as $finish |
    (if $copy then $p[$cursor] elif $finish then 1 else $s.native_id end) as $emit |
    [(if $copy then 1 else 0 end),$source,($cursor+(if $copy then 1 else 0 end)),$end] as $next |
    if $s.before!=.state or $s.after!=$next or $s.release!=$release or
       $s.source_id!=(if $copy then $p[$cursor] else null end) or $s.copied!=(if $copy then 1 else 0 end) or
       $s.emitted!=$emit then error("CE trace divergence for "+$g.id) else . end |
    if .first_free==null and ($copy|not) and ($finish|not) then .first_free=[.pos,$emit] else . end |
    .state=$next | .pos+=1)) as $trace |
  if $r.first_free!=$trace.first_free then error("first native emission divergence for "+$g.id) else . end |
  mismatch($r.generated;$want) as $wrong |
  {source,generated,first_wrong:$wrong,exact:($wrong==null),
   kind:(if .source==$g.asked_start then "requested" elif .source==$g.other_start then "distractor"
         elif .source==null then "abstain" else "other" end),
   expected_wrong:(if $wrong!=null then $want[$wrong] else null end),
   emitted_wrong:(if $wrong!=null then .generated[$wrong] else null end)};
def compare($a;$b):
  {moves:counts(direction(.[$a].first_wrong;.[$b].first_wrong)),
   matrix:counts((.[$a].first_wrong|idx)+"->"+(.[$b].first_wrong|idx)),
   source_recovered:n(.[$a].kind!="requested" and .[$b].kind=="requested"),
   source_lost:n(.[$a].kind=="requested" and .[$b].kind!="requested"),
   source_changed:n(.[$a].source!=.[$b].source),
   later_source:(map(select(direction(.[$a].first_wrong;.[$b].first_wrong)=="later"))|counts(.[$b].kind)),
   exact_gained:n((.[$a].exact|not) and .[$b].exact),exact_lost:n(.[$a].exact and (.[$b].exact|not))};
def stats: {cases:length, arms:(. as $cases | ["baseline","repair","oracle"]|map(. as $a|
  {key:$a,value:($cases|{exact:n(.[$a].exact),source:counts(.[$a].kind),wrong_index:counts(.[$a].first_wrong|idx),
   exact_origin:(map(select(.[$a].exact))|counts(.[$a].kind))})})|from_entries),
  paired:{repair:compare("baseline";"repair"),oracle:compare("baseline";"oracle")},
  repair_vs_oracle:compare("repair";"oracle")};
def tsv:
  (["case","prefix_sha256","pair","role","frozen_null","requested_address","distractor_address","reference"]+
   (["baseline","repair","oracle"]|map(. as $a|["address","kind","first_wrong","expected_wrong","emitted_wrong","emissions"]|map($a+"_"+.))|add)|@tsv),
  (.[]|[.id,.hash,.pair,.role,.null_baseline,.asked,.other,(.expected|tojson)]+
   [(.baseline,.repair,.oracle)|.source,.kind,(.first_wrong|idx),.expected_wrong,.emitted_wrong,(.generated|tojson)]|
   map(if .==null then "NA" else . end)|@tsv);
. as $base | $repaired[0] as $fix | $oracle[0] as $upper | $grades[0] as $g |
([$base,$fix,$upper]) as $runs | ($g|length) as $n |
if $n==0 or ($g|map(.prefix))!=($bare_rows[0]|map(.prefix)) or $base.bare!=$bare_hash or
  [$base.arm,$fix.arm,$upper.arm]!=["baseline","repair","oracle"] or ([$runs[]|.model]|unique|length)!=1 or ([$runs[]|.bare]|unique|length)!=1 or
  any($runs[]; .schema!="uor-r4.entity-writer/1" or (.rows|length)!=$n or ([.rows[].prefix_sha256]|unique|length)!=$n or .transitions!=([.rows[].steps|length]|add))
  then error("coverage or bound model/prefix mismatch") else . end |
(if ($history|length)==0 then null else ($history[0] |
  if .inputs.artifact!=$base.model or .inputs.bare!=$base.bare or .inputs.grade!=$grade_hash then error("wrong historical controls") else . end |
  [.rows[]|select(.arm=="ContinuationEos")]|if length!=$n then error("wrong historical CE coverage") else INDEX(.prefix_sha256) end) end) as $old |
[range(0;$n) as $i | ($g[$i]) as $key | $base.rows[$i] as $r | $fix.rows[$i] as $f | $upper.rows[$i] as $u |
 if $r.source==$f.source and [$r.steps,$r.generated,$r.first_free]!=[$f.steps,$f.generated,$f.first_free]
    then error("downstream CE drift at identical source: "+$key.id) else . end |
 if $r.source==$u.source and [$r.steps,$r.generated,$r.first_free]!=[$u.steps,$u.generated,$u.first_free]
    then error("diagnostic CE drift at identical source: "+$key.id) else . end |
 if $r.prefix_sha256!=$f.prefix_sha256 or $r.prefix_sha256!=$u.prefix_sha256 then error("prefix order mismatch") else . end |
 if $old!=null then ($old[$r.prefix_sha256]) as $h |
    if $h==null or $h.case!=$key.id or $h.expected!=$key.expected or
       [$h.selected,$h.generated,$h.first_free,$h.steps]!=[$r.source,$r.generated,$r.first_free,$r.steps]
      then error("historical CE trace drift on "+$key.id) else . end else . end |
 {id:$key.id,hash:$r.prefix_sha256,pair:$key.pair,role:$key.role,asked:$key.asked_start,other:$key.other_start,
  expected:$key.expected,null_baseline:($r.source==null),
  baseline:($r|check($key)),repair:($f|check($key)),oracle:($u|check($key))}] as $rows |
if $format=="tsv" then $rows|tsv else
 {grade_sha256:$grade_hash,artifact:$base.model,bare:$base.bare,
  transitions:($runs|map(.transitions)|add),historical_transitions:(if $old!=null then $base.transitions else 0 end),
  all:($rows|stats),null_cohort:($rows|map(select(.null_baseline))|stats),
  complement:($rows|map(select(.null_baseline|not))|stats),
  both_role_exact:($rows|group_by(.pair)|. as $p |
    if any(.[];length!=2 or .[0].role==.[1].role) then error("broken role pairs") else . end |
    ["baseline","repair","oracle"]|map(. as $a|{key:$a,value:($p|n(all(.[];.[$a].exact)))})|from_entries)} end
