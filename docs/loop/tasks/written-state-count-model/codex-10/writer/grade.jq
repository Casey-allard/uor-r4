# Grade one blind actual-feedback output against its matched baseline.
def counts(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def first_wrong($a;$b):
  (first(range(0; ([$a,$b]|map(length)|max))|select($a[.]!=$b[.]))) // "exact";
def bucket($p;$r):
  if $r.source == null then "null"
  elif $r.source == $p.asked_start then "requested"
  elif $r.source == $p.other_start then "distractor"
  else "other" end;
def verify_steps($p):
  . as $r |
  if (.steps|length)==0 or (.steps|length)>16 or .generated!=[.steps[].emitted]
     or (.generated[-1]!=1 and (.generated|length)!=16)
     or any(.generated[:-1][];.==1) then error("invalid issued tokens: "+$p.id) end |
  reduce .steps[] as $s (
    {state:[(if $r.source==null then 0 else 1 end),($r.source//4294967295),($r.source//4294967295),($p.prefix|length)],
     pos:0,free:null};
    .state as [$on,$source,$cursor,$end] |
    (if $on==0 then 0 elif $cursor-$source>=8 then 3 elif $cursor>=$end then 2
     elif ([16,1,0,201]|index($p.prefix[$cursor]))!=null then 1 else 0 end) as $release |
    ($on==1 and $release==0) as $copy | ($release==1 and $cursor>$source) as $eos |
    (if $copy then $p.prefix[$cursor] elif $eos then 1 else $s.native_id end) as $id |
    [(if $copy then 1 else 0 end),$source,$cursor+(if $copy then 1 else 0 end),$end] as $after |
    if $s.before!=.state or $s.after!=$after or $s.release!=$release or
       $s.source_id!=(if $copy then $id else null end) or $s.copied!=(if $copy then 1 else 0 end)
       or $s.emitted!=$id then error("cursor or committed ID drift: "+$p.id) end |
    if .free==null and ($copy|not) and ($eos|not) then .free=[.pos,$id] end |
    .pos+=1 | .state=$after
  ) | if .free!=$r.first_free then error("native release drift: "+$p.id) else $r end;
def paired:
  {rows:length,exact_baseline:map(select(.baseline=="exact"))|length,exact:map(select(.wrong=="exact"))|length,
   gained:map(select(.baseline!="exact" and .wrong=="exact"))|length,lost:map(select(.baseline=="exact" and .wrong!="exact"))|length,
   later:map(select(.baseline!="exact" and (.wrong=="exact" or .wrong>.baseline)))|length,
   earlier:map(select(.wrong!="exact" and (.baseline=="exact" or .wrong<.baseline)))|length,
   same:map(select(.baseline==.wrong))|length};
. as $raw|$base[0] as $b|$grades[0] as $g|($bare[0]|map(.prefix)) as $prefix|($g|length) as $n |
if $raw.arm!=$arm or $b.arm!="baseline" or $raw.schema!="uor-r4.entity-writer/1"
 or $raw.model!=$model_sha or $raw.bare!=$bare_sha or $raw.picks!=$pick_sha or $b.bare!=$bare_sha or $b.model!=$model_sha
 or $raw.writer_cubin!=$b.writer_cubin or $raw.completion_cubin!=$b.completion_cubin or $n==0
 or $prefix!=($g|map(.prefix)) or ($g|map(.id)|unique|length)!=$n
 or ($raw.rows|length)!=$n or ($b.rows|length)!=$n or $raw.transitions!=([$raw.rows[].steps|length]|add)
 or ($raw.rows|map(.prefix_sha256)|unique|length)!=$n
 then error("wrong writer identity or panel coverage") end |
[range($n) as $i | $g[$i] as $p |
 ($raw.rows[$i]|verify_steps($p)) as $r | ($b.rows[$i]|verify_steps($p)) as $ref |
 if $r.prefix_sha256!=$ref.prefix_sha256 or ($r.source==$ref.source and
  [$r.generated,$r.steps,$r.first_free]!=[$ref.generated,$ref.steps,$ref.first_free]) or
  ($arm=="no-read" and $r.source!=null) or ($arm=="value" and $r.source!=$p.asked_start) or
  ($arm=="wrong" and $r.source!=$p.other_start) then error("unmatched trajectory or wrong control: "+$p.id) end |
 {id:$p.id,hash:$r.prefix_sha256,source:$r.source,base_source:$ref.source,
  copied:any($r.steps[];.copied==1),generated:$r.generated,expected:$p.expected,
  pair:$p.pair,role:$p.role,kind:($p.kind//"positive"),factors:$p.factors,
  source_class:bucket($p;$r),wrong:first_wrong($r.generated;$p.expected//[]),baseline:first_wrong($ref.generated;$p.expected//[])}
] as $rows |
if $format=="tsv" then
 (["case","prefix_sha256","pair","role","kind","requested_position","exclusion","tail","connector_length","source","base_source","source_class","baseline_first_wrong","first_wrong","expected","generated"]|@tsv),
 ($rows[]|[.id,.hash,.pair,.role,.kind,.factors.requested_position,.factors.exclusion,.factors.tail,.factors.asked_connector_len,.source,.base_source,.source_class,
 (if .kind=="positive" then .baseline else null end),(if .kind=="positive" then .wrong else null end),(.expected|tojson),(.generated|tojson)]|@tsv)
elif $panel=="unanswerable" then
 {panel:$panel,arm:$arm,transitions:$raw.transitions,classes:($rows|group_by(.kind)|map({key:.[0].kind,value:{rows:length,copied:map(select(.copied))|length,
 source_abstain:map(select(.source==null))|length,baseline_source_abstain:map(select(.base_source==null))|length}})|from_entries)}
else
 if any($g[];.expected!=(.prefix[.asked_start:.asked_end]+[1]) or .other_start==.asked_start)
 or any($rows|group_by(.pair)[];length!=2 or .[0].role==.[1].role or .[0].expected==.[1].expected) then error("invalid reference pair") end |
 {panel:$panel,arm:$arm,model:$model_sha,bare:$bare_sha,picks:$pick_sha,transitions:$raw.transitions,
  results:($rows|paired),both_roles_exact:($rows|group_by(.pair)|map(select(all(.[];.wrong=="exact")))|length),
  sources:($rows|counts(.source_class)),first_wrong:($rows|counts(.wrong)),
  movement:($rows|counts((.baseline|tostring)+"->"+(.wrong|tostring))),
  strata:(["requested_position","exclusion","tail","asked_connector_len"]|map(. as $f|{key:$f,value:
  ($rows|group_by(.factors[$f])|map({key:(.[0].factors[$f]|tostring),value:paired})|from_entries)})|from_entries)}
end
