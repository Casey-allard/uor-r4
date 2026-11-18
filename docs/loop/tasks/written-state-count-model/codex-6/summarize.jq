# Pair sealed, closed-loop trajectories. See rollouts/README.md for the data and invocation.
def interventions: ["WordStart","Ordered","Both"];
def selectors: ["Current"] + interventions;
def arms: selectors + ["Oracle","Disabled"];
def count_if(f): map(select(f))|length;
def counts(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def ranks: .first_wrong // ((.expected|length)+1);
def index_label: if .first_wrong==null then "exact" else (.first_wrong|tostring) end;
def wrong_index:
  .expected as $e|.generated as $g|
  first(range(0;([$e,$g]|map(length)|max))|select($e[.]!=$g[.]))//null;
def direction($old): if ranks>($old|ranks) then "later" elif ranks<($old|ranks) then "earlier" else "tied" end;
def endpoints: [.query,.fact,.query_start,.query_end,.fact_start,.fact_end];
def corresponding($g):
  .query!=null and .fact!=null and $g.query_start<=.query and .query<=$g.query_end and
  $g.fact_start<=.fact and .fact<=$g.fact_end and (.query-$g.query_start)==(.fact-$g.fact_start);
def belongs($dict;$seq): any($dict[]; .==$seq);
def mask: map(pow(2;.-1))|add//0;
def class_of($src;$ids;$classes): if $src==null then "abstain" else ["other","start","continuation"][$classes[$ids[$src]]] end;
def source_status:
  if .right_source then "requested"
  elif .selected==null then "abstained" elif .selected==.other_start then "distractor"
  else "other" end;
def first_free:
  [.steps|to_entries[]|select(.value.copied==0)|[.key,.value.emitted]][0];
def check_row($g):
  (if .first_wrong==null then [null,null] else [.expected[.first_wrong],.generated[.first_wrong]] end) as $w |
  if .generated!=(.steps|map(.emitted)) or (.generated|length)<1 or .expected!=$g.expected or
     .gold_positions!=$g.token_positions or .factors!=$g.factors or .asked_start!=$g.asked_start or .other_start!=$g.other_start or
     .pair!=$g.pair or .role!=$g.role or .expected[-1]!=1 or .first_wrong!=wrong_index or
     [.wrong_expected,.wrong_emitted]!=$w or .right_source!=(.asked_start==.selected) or
     .whole_value!=(.generated[:((.expected|length)-1)]==.expected[:-1]) or .first_free!=first_free
  then error("raw trajectory disagrees with grade/steps: "+.case+" "+.arm) else . end;
def grade_case($grades;$dict;$classes):
  . as $group | if (map(.arm)|sort)!=(arms|sort) then error("missing treatment") else . end |
  .[0].case as $id | $grades[$id] as $g |
  if $g==null then error("unregistered case "+$id) else . end |
  map(check_row($g))| INDEX(.arm) as $rows |
  $rows.Current.choice as $c | $g.prefix as $ids | $g.token_positions as $gold |
  ($ids[($gold.fact_end+1):$g.asked_start]) as $truth |
  if ($truth|length)!=$g.factors.asked_connector_len then error("gold gap mismatch "+$id) else . end |
  ([$dict[] | select(.==$truth[:length]) | length]|max//0) as $registered |
  (if $registered==($truth|length) then "full" elif $registered>0 then "proper_prefix" else "none" end) as $coverage |
  (if $c.source==null then [] else [range(1;4) | . as $n | ($c.fact_end+1+$n) as $src |
     select($src<$c.query_start) | {n:$n,source:$src,word:($classes[$ids[$src]]==1),
       ordered:belongs($dict; $ids[($c.fact_end+1):$src])}] end) as $candidates |
  {eligible:($candidates|map(.n)|mask),word_start:($candidates|map(select(.word)|.n)|mask),
   ordered:($candidates|map(select(.ordered)|.n)|mask)} as $masks |
  ($candidates|map(select(.ordered))|last.source) as $ordered |
  ($candidates|map(select(.ordered and .word))|last.source) as $both |
  if any(selectors[]; . as $a | ($rows[$a].choice|endpoints)!=($c|endpoints) or
                         $rows[$a].choice.source!=$rows[$a].selected)
     or $rows.Ordered.selected!=$ordered or $rows.Both.selected!=$both
     or ([$candidates[]|select(.word)|.source] as $word | ($word|length)==0 and $rows.WordStart.selected!=null or
         ($word|length)>0 and ($word|index($rows.WordStart.selected))==null)
     or ($rows.Oracle|(.right_source and .whole_value)|not)
     or any(arms[]; . as $a|($rows[$a].selected==null and $rows[$a].steps!=$rows.Disabled.steps))
  then error("arm endpoint/eligibility/abstention mismatch: "+$id) else . end |
  if any(selectors[]; . as $a | $rows[$a].masks!=$masks) then error("mask mismatch: "+$id+" computed "+($masks|tojson)) else . end |
  {id:$id,pair:$g.pair,role:$g.role,source:$g.asked_start,truth_length:($truth|length),registered:$registered,
   triplet:($ids[:($gold.fact_end+1)]+$ids[$g.asked_start:]),
   coverage:$coverage,family:($g.factors.connector_set//"historical"),gold:$gold,masks:$masks,
   corresponding:($c|corresponding($gold)),candidates:$candidates,
   base_class:class_of($c.source;$ids;$classes),
   cohort_ose:(($c|corresponding($gold)) and $c.source!=null and $c.source==$g.asked_start-1 and $ids[$c.source]==1935),
   arms:($rows|with_entries(.value|=del(.steps,.factors,.gold_positions)))};
def score_arm($a):
  map(.arms[$a])|
  {starts:count_if(.right_source),opening:count_if(.expected[0]==.generated[0]),whole_value:count_if(.whole_value),
   replies:count_if(.first_wrong==null),both_roles:(group_by(.pair)|count_if(length==2 and all(.right_source))),
   abstained:count_if(.selected==null),status:counts(source_status),first_wrong:counts(index_label),
   first_free_after_right_value:(map(select(.right_source and .whole_value))|counts(.first_free[1]))};
def pair($base;$a):
  map(.arms[$base] as $b | .arms[$a] as $r |
      {move:($r|direction($b)),transition:(($b|index_label)+"->"+($r|index_label)),old:$b.right_source,new:$r.right_source,
       old_source:$b.selected,new_source:$r.selected,old_status:($b|source_status),new_status:($r|source_status),
       changed_wrong_id:(($b|index_label)==($r|index_label) and ($b.wrong_emitted!=$r.wrong_emitted)),
       start_class:.base_class}) |
  {rows:length,moves:counts(.move),index_transitions:counts(.transition),right:count_if(.new),gained:count_if(.new and (.old|not)),
   lost:count_if(.old and (.new|not)),abstained:count_if(.new_source==null),wrong_copy:count_if((.new|not) and .new_source!=null),
   later_source_status:(map(select(.move=="later"))|counts(.new_status)),earlier_source_status:(map(select(.move=="earlier"))|counts(.new_status)),
   tied_index_changed_wrong_id:count_if(.changed_wrong_id),changed_source:count_if(.old_source!=.new_source)};
def compare: . as $cases | interventions|map(. as $a|{key:$a,value:($cases|pair("Current";$a))})|from_entries;
def conditional: map(select(.corresponding)) | . as $c |
  {rows:length,arms:(selectors|map(. as $a|{key:$a,value:($c|map(.arms[$a])|{correct:count_if(.right_source),
    abstained:count_if(.selected==null),displacement:counts(if .selected==null then "abstain" else .selected-.asked_start end),
    reliable:(if length==0 then null else 4*count_if(.right_source)>=3*length end)})})|from_entries)};
def stratum:
  {rows:length,moves:compare,corresponding:conditional};
def by_length: group_by(.truth_length)|map({length:.[0].truth_length}+stratum);
def examples:
  . as $cases |
  [interventions[] as $a | ["later","earlier"][] as $m |
    first($cases[] | .arms.Current as $b | select((.arms[$a]|direction($b))==$m) |
      {arm:$a,move:$m,case:.id,expected:$b.expected,asked:.source,current_source:$b.selected,treatment_source:.arms[$a].selected,
       current:$b.generated,treatment:.arms[$a].generated})];
def triplets:
  group_by(.triplet) as $g |
  if any($g[]; (map(.truth_length)|sort)!=[1,2,3]) then error("broken connector-only triplet") else
    {triplets:($g|length),correct_at_all_three:(arms|map(. as $a|{key:$a,value:($g|count_if(all(.arms[$a].right_source)))})|from_entries)}
  end;
def summary:
  . as $c |
  {cases:length,arms:(arms | map(. as $a|{key:$a,value:($c|score_arm($a))})|from_entries),
   triplets:(if $split=="primary" or $split=="fresh" then triplets else null end),
   comparisons:compare,joint_vs_word:pair("WordStart";"Both"),joint_vs_order:pair("Ordered";"Both"),
   length:by_length,coverage:(group_by(.coverage)|map({coverage:.[0].coverage}+stratum)),
   family_by_length:(group_by(.family,.truth_length)|map({family:.[0].family,length:.[0].truth_length}+stratum)),
   current_source_class:(group_by(.base_class)|map({class:.[0].base_class}+stratum)),
   historical_ose:(map(select(.cohort_ose))|by_length),
   endpoint_groups:(group_by(.corresponding)|map({corresponding:.[0].corresponding,rows:length,moves:compare})),
   examples:examples} |
  . as $s | . + {bars:(interventions|map(. as $a|$s.comparisons[$a] as $delta|
     {key:$a,value:{directional:((($delta.moves.later//0)>($delta.moves.earlier//0)) and ($delta.gained>$delta.lost)),
                    pair_nondecrease:($s.arms[$a].both_roles >= $s.arms.Current.both_roles),
                    extraction:([$s.length[].corresponding.arms[$a].reliable]|length==3 and all(.==true))}})|from_entries)};
def cell($c;$a):
  $c.arms[$a] | [.selected,index_label,.wrong_expected,.wrong_emitted,.first_free[0],.first_free[1]];
def tsv:
  (["case","pair","role","connector_ids","longest_registered_prefix","gold_coverage","gold_value","corresponding_anchors","old_value_class",
    "q_start","q_end","f_start","f_end","eligible_mask","start_mask","ordered_mask"] +
   [arms[] as $a|["source","first_wrong","expected","emitted","first_free_index","first_free_id"][]|$a+"_"+.]|@tsv),
  (.[] as $c | $c.arms.Current.choice as $v |
    ([$c.id,$c.pair,$c.role,$c.truth_length,$c.registered,$c.coverage,$c.source,$c.corresponding,$c.base_class,
      $v.query_start,$v.query_end,$v.fact_start,$v.fact_end,$c.masks.eligible,$c.masks.word_start,$c.masks.ordered] +
     [arms[] as $a | cell($c;$a)[]])|map(if .==null then "NA" else . end)|@tsv);
if .schema!="uor-r4.value-writer/1" then error("unrecognized writer schema") else . end |
($training[0]|map(. as $r|.prefix[($r.fact_end+1):$r.source])|unique) as $dict |
($grading[0]|INDEX(.id)) as $grades | $span_state[0].bank.classes as $classes |
if ($classes|length)!=4096 or ($dict|length)!=6 or any($dict[];length<1 or length>3) or ($grades|length)!=$n then error("invalid inputs") else . end |
.differential_steps as $steps | .inputs as $inputs | .rows |
if (map(.steps|length)|add)!=$steps or length!=6*$n then error("transition/arm count mismatch") else . end |
group_by(.case)|map(grade_case($grades;$dict;$classes))|
if length!=$n or any(group_by(.pair)[];length!=2 or .[0].role==.[1].role) then error("case/role coverage mismatch") else . end |
if $format=="tsv" then tsv elif $format=="summary" then summary + {schema:"uor-r4.value-boundary-comparison/1",split:$split,
     differential_steps:$steps,artifact_sha256:$inputs.artifact,grade_sha256:$inputs.grade}
else error("unknown format") end
