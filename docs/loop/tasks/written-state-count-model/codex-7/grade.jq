# Independently grade real per-step writer traces against externally bound prompts.
def selectors: ["WordStart","Continuation","WordStartEos","ContinuationEos"];
def arms: selectors + ["Oracle","OracleEos","Disabled"];
def contrasts: [["WordStart","Continuation"],["WordStart","WordStartEos"],
  ["WordStart","ContinuationEos"],["Continuation","ContinuationEos"],["WordStartEos","ContinuationEos"]];
def counts(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def n(f): map(select(f))|length;
def fail($why): error($why+" case="+(.case//.id//"unknown")+" arm="+(.arm//"unknown"));
def first_wrong($want;$got):
  first(range(0;([$want,$got]|map(length)|max))|select($want[.]!=$got[.]))//null;
def index_label: if .first_wrong==null then "exact" else (.first_wrong|tostring) end;
def direction($a;$b):
  if $a==$b then "same" elif $b==null then "later" elif $a==null then "earlier"
  elif $a<$b then "later" else "earlier" end;
def eos_arm: .=="WordStartEos" or .=="ContinuationEos" or .=="OracleEos";
def frozen: del(.source);
def corresponding($row;$g):
  $g.token_positions as $p | $row.choice as $c |
  $c.query!=null and $c.fact!=null and $c.query>=$p.query_start and $c.query<=$p.query_end and
  $c.fact>=$p.fact_start and $c.fact<=$p.fact_end and ($c.query-$p.query_start)==($c.fact-$p.fact_start);
def source_kind:
  if .right_source then "requested" elif .selected==null then "abstain"
  elif .selected==.other_start then "distractor" else "other" end;
def exact_origin($g):
  if .selected==null then "unaddressed"
  elif .selected==$g.asked_start then "requested_address"
  elif .selected>$g.asked_start and .selected<$g.asked_end then "requested_suffix"
  elif .selected==$g.other_start then "distractor_address"
  elif .selected>$g.other_start and .selected<$g.other_end then "distractor_suffix"
  else "outside_both_values" end;
def original_fields:
  {case,pair,role,arm,choice,expected,asked_start,other_start,selected,right_source,
   first_free,first_wrong,wrong_expected,wrong_emitted,generated,steps,stop,whole_value};
def check_trace($g):
  . as $r | $g.prefix as $p | (.arm|eos_arm) as $enabled |
  (.selected//4294967295) as $s |
  (if $r.selected==null then [0,$s,$s,($p|length)] else [1,$s,$s,($p|length)] end) as $init |
  (reduce range(0;(.steps|length)) as $i ({state:$init,action:null};
    $r.steps[$i] as $step | .state as $b | $b[2] as $cursor |
    (if $b[0]==0 then 0 elif $cursor-$b[1]==8 then 3 elif $cursor==$b[3] then 2
     elif ([16,1,0,201]|index($p[$cursor]))!=null then 1 else 0 end) as $release |
    ($b[0]==1 and $release==0) as $copy |
    ($enabled and $release==1 and $cursor>$b[1]) as $finish |
    (if $copy then [$b[0],$b[1],$cursor+1,$b[3]] elif $release>0 then [0,$b[1],$cursor,$b[3]] else $b end) as $after |
    (if $copy then $p[$cursor] elif $finish then 1 else $step.native_id end) as $emit |
    if $step.before!=$b or $step.after!=$after or $step.release!=$release or $step.copied!=(if $copy then 1 else 0 end) or
       $step.source_id!=(if $copy then $p[$cursor] else null end) or $step.emitted!=$emit or
       $step.native_id!=$step.native_top8[0] or ($step.emitted==1 and $i+1!=($r.steps|length))
    then $r|fail("raw copy cursor, release, action or emitted token disagrees at "+($i|tostring)) else
      {state:$after,action:(if $finish then $i else .action end)} end
  )) as $trace |
  ($r.steps|map(.emitted)) as $out | $g.expected as $e | first_wrong($e;$out) as $wrong |
  if ($e[-1]!=1 or $e[:-1]!=$p[$g.asked_start:$g.asked_end] or $r.generated!=$out or ($out|length)<1 or
      ($out|length)>16 or ($out[-1]!=1 and ($out|length)!=16) or $r.stop!=(if $out[-1]==1 then "eos" else "cap" end) or
      $r.first_wrong!=$wrong or $r.exact!=($out==$e) or $r.eos_action_step!=$trace.action or
      $r.expected!=$e or $r.pair!=$g.pair or $r.role!=$g.role or $r.asked_start!=$g.asked_start or $r.other_start!=$g.other_start or
      $r.factors!=$g.factors or $r.gold_positions!=$g.token_positions or
      $r.first_free!=([$r.steps|to_entries[]|select(.value.copied==0 and .key!=$trace.action)|[.key,.value.emitted]][0]) or
      $r.right_source!=($r.selected==$g.asked_start) or $r.whole_value!=($out[:(($e|length)-1)]==$e[:-1]) or
      [$r.wrong_expected,$r.wrong_emitted]!=(if $wrong==null then [null,null] else [$e[$wrong],$out[$wrong]] end))
  then $r|fail("recorded grade or EOS witness disagrees with canonical grade and emitted IDs") else $r end;
def paired_eos($base;$end):
  if $base.selected!=$end.selected or $base.choice!=$end.choice then false
  elif $end.eos_action_step==null then $base.steps==$end.steps
  else $end.eos_action_step as $i |
    $base.steps[:$i]==$end.steps[:$i] and
    ($base.steps[$i]|del(.emitted))==($end.steps[$i]|del(.emitted)) and ($end.steps|length)==$i+1 end;
def grade_group($g;$old):
  if (map(.arm)|sort)!=(arms|sort) then error("missing or repeated arms") else . end |
  map(check_trace($g))|INDEX(.arm) as $r |
  if any(selectors[]; . as $a | ($r[$a].choice|frozen)!=($r.WordStart.choice|frozen) or $r[$a].choice.source!=$r[$a].selected)
     or (paired_eos($r.WordStart;$r.WordStartEos)|not)
     or (paired_eos($r.Continuation;$r.ContinuationEos)|not)
     or (paired_eos($r.Oracle;$r.OracleEos)|not)
     or ($r.Oracle.selected!=$g.asked_start or ($r.Oracle.whole_value|not) or ($r.OracleEos.exact|not) or $r.Disabled.selected!=null)
     or any(arms[]; . as $a|$r[$a].selected==null and $r[$a].steps!=$r.Disabled.steps)
  then $g|fail("factor isolation, frozen endpoints, oracle or abstention mismatch") else . end |
  ["WordStart","Oracle","Disabled"] as $controls |
  if $old!=null and any($controls[]; . as $a | ($r[$a]|original_fields)!=($old[$a]|original_fields))
  then $g|fail("historical control trajectory changed") else . end |
  {id:$g.id,pair:$g.pair,role:$g.role,prompt:$g.prompt,source:$g.asked_start,
   corresponding:corresponding($r.WordStart;$g),family:($g.factors.connector_set//"historical"),
   length:$g.factors.asked_connector_len,surface:($g.factors.asked_connector_surface//null),
   controls:(if $old==null then 0 else [$controls[] as $a|($r[$a].steps|length)]|add end),
   arms:($r|with_entries(.value|={selected,right_source,kind:source_kind,origin:exact_origin($g),exact,whole_value,
      first_wrong,wrong_emitted,wrong_expected,expected,generated,eos_action_step,
      release:([.steps[]|select(.release!=0)|.release][0]//0)}))};
def compact($arm):
  map(.arms[$arm])|{requested:n(.right_source),exact:n(.exact),whole_value:n(.whole_value),
    exact_by_source:(map(select(.exact))|counts(.origin)),wrong_index:counts(index_label),
    source:counts(.kind),terminal_action:n(.eos_action_step!=null),releases:counts(.release)};
def pair($a;$b):
  map(.arms[$a] as $old|.arms[$b] as $new|
    {move:direction($old.first_wrong;$new.first_wrong),transition:(($old|index_label)+"->"+($new|index_label)),
     exact_gain:($new.exact and ($old.exact|not)),exact_loss:($old.exact and ($new.exact|not)),
     old_source:$old.kind,new_source:$new.kind,changed_source:($old.selected!=$new.selected),
     changed_wrong:(($old.first_wrong==$new.first_wrong) and ($old.wrong_emitted!=$new.wrong_emitted))}) |
  {moves:counts(.move),transitions:counts(.transition),exact_gain:n(.exact_gain),exact_loss:n(.exact_loss),
   address_gain:n(.old_source!="requested" and .new_source=="requested"),address_loss:n(.old_source=="requested" and .new_source!="requested"),
   source_changed:n(.changed_source),tied_index_changed_id:n(.changed_wrong),
   later_by_source:(map(select(.move=="later"))|counts(.new_source))};
def breakdown:
  . as $rows|{cases:length,arms:(selectors|map(. as $a|{key:$a,value:($rows|compact($a))})|from_entries)};
def summary:
  . as $rows | {cases:length,identical_historical_steps:(map(.controls)|add),
    family_deletion:(map(.family)|unique|map(. as $omit |
      ($rows|map(select(.family!=$omit))) as $keep | select(($keep|length)>0) |
      {omitted:$omit,cases:($keep|length),pairs:($keep|map(.pair)|unique|length),
       contrasts:(contrasts|map(. as [$a,$b]|
         {key:($a+"->"+$b),value:($keep|
           {net_later:(n(direction(.arms[$a].first_wrong;.arms[$b].first_wrong)=="later")-
                       n(direction(.arms[$a].first_wrong;.arms[$b].first_wrong)=="earlier")),
            net_exact:(n(.arms[$b].exact)-n(.arms[$a].exact))})})|from_entries)})),
    exact_count_interaction:(n(.arms.ContinuationEos.exact)-n(.arms.Continuation.exact)-n(.arms.WordStartEos.exact)+n(.arms.WordStart.exact)),
    per_case_exact_interaction:counts(
      (.arms.ContinuationEos.exact|if . then 1 else 0 end)-(.arms.Continuation.exact|if . then 1 else 0 end)-
      (.arms.WordStartEos.exact|if . then 1 else 0 end)+(.arms.WordStart.exact|if . then 1 else 0 end)),
    joint_vs_eos_exact_overlap:{both:n(.arms.ContinuationEos.exact and .arms.WordStartEos.exact),
      joint_only:n(.arms.ContinuationEos.exact and (.arms.WordStartEos.exact|not)),
      eos_only:n(.arms.WordStartEos.exact and (.arms.ContinuationEos.exact|not)),
      neither:n((.arms.ContinuationEos.exact or .arms.WordStartEos.exact)|not)},
    arms:(arms|map(. as $a|{key:$a,value:(($rows|compact($a))+
      {both_role_addresses:($rows|group_by(.pair)|n(all(.arms[$a].right_source))),
       both_role_exact:($rows|group_by(.pair)|n(all(.arms[$a].exact)))})})|from_entries),
    paired:(contrasts|map(. as [$a,$b]|{key:($a+"->"+$b),value:($rows|pair($a;$b))})|from_entries),
    by_connector:(group_by(.family,.length)|map({family:.[0].family,length:.[0].length}+breakdown)),
    anchor_location:(group_by(.corresponding)|map({correct_entity_offsets:.[0].corresponding}+breakdown))};
def tsv:
  (["case","pair","role","connector_family","connector_ids","connector_surface","correct_entity_offsets","asked_source","expected"]+
    [arms[] as $a | ["source","first_wrong","wrong_expected","wrong_emitted","eos_action_at","generated"][]|$a+"_"+.]|@tsv),
  (.[] as $case|([$case.id,$case.pair,$case.role,$case.family,$case.length,$case.surface,$case.corresponding,$case.source,($case.arms.WordStart.expected|tojson)]+
    [arms[] as $a|$case.arms[$a]|.selected,index_label,.wrong_expected,.wrong_emitted,.eos_action_step,(.generated|tojson)])|
    map(if .==null then "NA" else . end)|@tsv);
. as $raw | ($grade[0]|INDEX(.id)) as $g |
if $raw.schema!="uor-r4.completion-writer/1" or ($raw.rows|length)!=7*$n or ($g|length)!=$n or ($raw.rows|map(.steps|length)|add)!=$raw.differential_steps
then error("writer population, grade or step total mismatch") else . end |
(if ($previous|length)>0 then
   $previous[0] as $p |
   if $raw.inputs.artifact!=$p.inputs.artifact or $raw.inputs.grade!=$p.inputs.grade or $p.schema!="uor-r4.value-writer/1"
   then error("historical artifact/grade changed") else
     $p.rows|map(select(.arm=="WordStart" or .arm=="Oracle" or .arm=="Disabled"))|group_by(.case)|
     map({key:.[0].case,value:INDEX(.arm)})|from_entries end
 else null end) as $old |
if $old!=null and ($old|length)!=$n then error("historical coverage mismatch") else . end |
$raw.rows|group_by(.case)|map(.[0].case as $id|
 if $g[$id]==null or ($old!=null and $old[$id]==null) then error("unregistered case: "+$id) else grade_group($g[$id];$old[$id]) end) |
if length!=$n or any(group_by(.pair)[];length!=2 or .[0].role==.[1].role) then error("missing case or role") else . end |
if $format=="summary" or $format=="both" then
  (summary+{split:$split,differential_steps:$raw.differential_steps,inputs:$raw.inputs}) as $s |
  if $format=="both" then ($s|tojson),tsv else $s end
elif $format=="tsv" then tsv else error("unknown output format") end
