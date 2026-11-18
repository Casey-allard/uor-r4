# One closed-loop writer JSON per invocation. See rollouts/README.md for commands.
def count_where(f): map(select(f))|length;
def dist(f): group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def inside($x;$lo;$hi): ($x!=null and $lo!=null and $hi!=null and $lo<=$x and $x<=$hi);
def expected_error:
  .expected as $expected | .generated as $generated |
  first(range(0;([$expected,$generated]|map(length)|max)) | select($expected[.]!=$generated[.])) // null;
def rank: .first_wrong // ((.expected|length)+1);
def move: if .>0 then "later" elif .<0 then "earlier" else "tie" end;
def error_label: if .first_wrong==null then "exact" else (.first_wrong|tostring) end;
def correct_pair: group_by(.pair)|count_where(length==2 and all(.right_source));
def stats:
  {rows:length,source:count_where(.right_source),abstained:count_where(.selected==null),distractor:count_where(.selected==.other_start),opening:count_where(.generated[0]==.expected[0]),
   value:count_where(.whole_value),exact:count_where(.first_wrong==null),first_wrong:dist(error_label),
   moves_vs_content:dist(.delta|move),
   wrong_expected:dist(.wrong_expected),wrong_emitted:dist(.wrong_emitted)};
def trace(r):
  r as $ref | .gold_positions as $g |
  inside(.choice.query;$g.query_start;$g.query_end) as $query_inside |
  inside(.choice.fact;$g.fact_start;$g.fact_end) as $fact_inside |
  (.choice!=null and .choice.query_start!=null) as $span |
  . + {delta:(rank-($ref|rank)),query_inside:$query_inside,fact_inside:$fact_inside,
       query_exact:($span and [.choice.query_start,.choice.query_end]==[$g.query_start,$g.query_end]),
       fact_exact:($span and [.choice.fact_start,.choice.fact_end]==[$g.fact_start,$g.fact_end]),
       q0:$g.query_start,q1:$g.query_end,f0:$g.fact_start,f1:$g.fact_end}
  | del(.steps);
def stage:
  if .query_inside==false then "query anchor outside requested name"
  elif .fact_inside==false then "fact anchor outside requested name"
  elif .query_exact==false then "query span incorrect"
  elif .fact_exact==false then "fact span incorrect"
  elif .right_source==false then "value start incorrect"
  elif .whole_value==false then "copy lost a target token"
  elif .first_wrong!=null then "complete value followed by wrong continuation"
  else "exact reply" end;
def corresponding_anchor:
  .query_inside and .fact_inside and (.choice.query-.q0)==(.choice.fact-.f0);
def conditional_extraction:
  {rows:length,correct_query_and_fact_spans:count_where(.query_exact and .fact_exact),
   correct_source:count_where(.right_source),whole_value:count_where(.whole_value),
   source_displacement:dist(if .selected==null then "abstain" else (.selected-.asked_start) end),
   reliable:(if length==0 then null else 4*count_where(.right_source)>=3*length end)};
def tally_arms: group_by(.arm)|map({key:.[0].arm,value:stats})|from_entries;
def stratify(f): group_by(f)|map({level:(.[0]|f),arms:tally_arms});
def by_case: group_by(.case)|map(INDEX(.arm));
def joint:
  by_case | {
    rows:length,
    both_correct:count_where(.Span.right_source and .Content.right_source),
    lost:count_where(.Span.right_source==false and .Content.right_source),
    gained:count_where(.Span.right_source and .Content.right_source==false),
    both_wrong:count_where(.Span.right_source==false and .Content.right_source==false),
    error_moves:dist((.Content|error_label)+"->"+(.Span|error_label)),
    full_better_than_endpoint:count_where((.Span|rank)>(.EndpointOnly|rank)),
    full_worse_than_endpoint:count_where((.Span|rank)<(.EndpointOnly|rank)),
    full_better_than_connector:count_where((.Span|rank)>(.ConnectorOnly|rank)),
    full_worse_than_connector:count_where((.Span|rank)<(.ConnectorOnly|rank))
  };
def cohort:
  .Span as $s |
  if $s.choice.query == $s.q0 and $s.choice.fact == $s.f0 and $s.q0 < $s.q1 and $s.f0 < $s.f1
  then "matched both leading name pieces"
  elif $s.choice.query==$s.q1 and $s.choice.fact==$s.f1 then "matched both final name pieces"
  elif $s.query_inside and $s.fact_inside then "matched other pieces of both requested names"
  else "one or both anchors outside requested names" end;
def cohort_summary:
  {rows:length,
   content_source:count_where(.Content.right_source),full_source:count_where(.Span.right_source),
   full_exact_ends:count_where(.Span.query_exact and .Span.fact_exact),
   connector_only_source:count_where(.ConnectorOnly.right_source),endpoint_only_source:count_where(.EndpointOnly.right_source),
   later:count_where(.Span.delta>0),earlier:count_where(.Span.delta<0),tied:count_where(.Span.delta==0)};
def score:
  . as $rows | map(select(.arm=="Span")) as $full |
  {arms:($rows|group_by(.arm)|map({key:.[0].arm,value:(stats+{both_roles:correct_pair,pairs:(group_by(.pair)|length)})})|from_entries),
   length:($rows|stratify(.factors.asked_connector_len)),
   joint_bridge:($rows|stratify([.factors.asked_connector_len,.factors.other_connector_len])),
   lesion:($rows|joint),
   paired_by_requested_length:($rows|group_by(.factors.asked_connector_len)|map({asked_connector_len:.[0].factors.asked_connector_len} + joint)),
   correct_source_with_imperfect_span:($full|count_where(.right_source and (.query_exact and .fact_exact|not))),
   correct_opening_with_wrong_source:($full|count_where((.right_source|not) and .expected[0]==.generated[0])),
   stages:($full|dist(stage)),
   stages_by_length:($full|group_by(.factors.asked_connector_len)|map({length:.[0].factors.asked_connector_len,counts:dist(stage)})),
   conditional_extraction:($full|group_by(.factors.asked_connector_len)|map(
      .[0].factors.asked_connector_len as $length|map(select(corresponding_anchor))|{asked_connector_len:$length}+conditional_extraction)),

   conditioned:($rows|by_case|group_by(cohort)|map({cohort:(.[0]|cohort)} + cohort_summary)),
   first_free_after_value:($full|map(select(.whole_value))|dist(.first_free[1])),
   examples:($rows|by_case|group_by(.Span.delta|move) | map(.[0]|{case:.Span.case,expected:.Span.expected,current:.Content.generated,span:.Span.generated})),
   primary:(if $split=="new" then {
      span_more_both_roles:(($full|correct_pair)>($rows|map(select(.arm=="Content"))|correct_pair)),
      later_outnumber_earlier:(($full|count_where(.delta>0))>($full|count_where(.delta<0)))} else null end)}
  + (if $split=="new" then {
    shared_leading:($rows|map(select(.factors.lexical_set==1))|joint),
    connector_surface:($rows|stratify(.factors.connector_set)),
    conditional_by_surface_and_length:($full|group_by(.factors.connector_set,.factors.asked_connector_len)|map(
      [.[0].factors.connector_set,.[0].factors.asked_connector_len] as $cell |
      map(select(corresponding_anchor))|{connector_set:$cell[0],asked_connector_len:$cell[1]}+conditional_extraction)),
    both_roles_by_fact_lengths:($rows|group_by(.factors.connector_0_len,.factors.connector_1_len,.arm)|map({
      lengths:[.[0].factors.connector_0_len,.[0].factors.connector_1_len],arm:.[0].arm,pairs:(group_by(.pair)|length),both_roles:correct_pair})),
    connector_triplets:($rows|group_by(.arm)|map(.[0].arm as $arm |
      group_by(.factors|[.lexical_set,.assignment,.fact_order,.requested_position,.exclusion,.tail,.asked_role,.other_connector_len]) |
      if any(.[];(map(.factors.asked_connector_len)|sort)!=[1,2,3]) then error("broken matched connector-only triplet")
      else {arm:$arm,triplets:length,correct_at_all_three:count_where(all(.right_source))} end)),
    by_name_width:($rows|stratify(.q1-.q0+1))
  } elif $split=="original" then {
    previous_nine:($rows|by_case|map(select(.Content.choice.query==.Content.q1 and .Content.choice.fact==.Content.f1 and .Content.selected==.Content.asked_start-1))|cohort_summary)
  } else {} end);
def cases:
  (["case","pair","role","asked_connector_len","other_connector_len","q_start","q_end","f_start","f_end","gold_value",
    "current_q","current_f","current_value","current_first_wrong","current_wrong_expected","current_wrong_emitted",
    "span_q_start","span_q_end","span_f_start","span_f_end","span_value","span_first_wrong","span_wrong_expected","span_wrong_emitted",
    "span_first_free","span_error_delta","span_stage","endpoint_only_source","endpoint_only_first_wrong","connector_only_source","connector_only_first_wrong",
    "oracle_first_wrong","disabled_first_wrong"]|@tsv),
  (by_case[] | .Span as $s | .Content as $c |
    [$s.case,$s.pair,$s.role,$s.factors.asked_connector_len,$s.factors.other_connector_len,$s.q0,$s.q1,$s.f0,$s.f1,$s.asked_start,
     $c.choice.query,$c.choice.fact,$c.selected,($c|error_label),$c.wrong_expected,$c.wrong_emitted,
     $s.choice.query_start,$s.choice.query_end,$s.choice.fact_start,$s.choice.fact_end,$s.selected,($s|error_label),$s.wrong_expected,$s.wrong_emitted,
     ($s.first_free|tojson),$s.delta,($s|stage),.EndpointOnly.selected,(.EndpointOnly|error_label),.ConnectorOnly.selected,(.ConnectorOnly|error_label),
     (.Oracle|error_label),(.Disabled|error_label)] | map(if .==null then "NA" else . end)|@tsv);
if .schema != "uor-r4.span-writer/1" then error("unexpected rollout schema") else . end |
.differential_steps as $steps | .inputs.artifact as $artifact | .inputs.grade as $grade |
if ($artifact|type)!="string" or ($grade|type)!="string" then error("missing artifact or grade digest") else . end |
.rows | group_by(.case) |
if length!=$n or any(.[];(map(.arm)|sort)!=["ConnectorOnly","Content","Disabled","EndpointOnly","Oracle","Span"] or
  (map([.pair,.role,.asked_start,.other_start,.expected,.gold_positions,.factors])|unique|length)!=1)
then error("incomplete/inconsistent cases") else . end |
map(. as $group | ($group[]|select(.arm=="Content")) as $current | $group[] |
  if (.steps|map(.emitted))!=.generated or .expected[-1]!=1 or .first_wrong!=expected_error
    or .wrong_expected!=(if .first_wrong==null then null else .expected[.first_wrong] end)
    or .wrong_emitted!=(if .first_wrong==null then null else .generated[.first_wrong] end)
    or .right_source!=(.selected==.asked_start) or .whole_value!=(.generated[:((.expected|length)-1)]==.expected[:-1])
    or .first_free!=([.steps|to_entries[]|select(.value.copied==0)|[.key,.value.emitted]][0])
  then error("trajectory disagreement in "+.case) else . end |
  .arm as $arm |
  if .gold_positions.fact_end+1+.factors.asked_connector_len!=.asked_start then error("invalid source/connector grading gap") else . end |
  if (["ConnectorOnly","Span","EndpointOnly"]|index($arm))!=null and
      ([.choice.query,.choice.fact]!=[$current.choice.query,$current.choice.fact])
  then error("q/f changed between lesion arms") else . end | trace($current)) |
if (map(.generated|length)|add)!=$steps or (group_by(.arm,.pair)|any(.[]; length!=2 or .[0].role==.[1].role))
then error("paired or transition coverage differs") else . end |
if $format=="summary" then score | . + {extraction_bar:(if $split=="new" then (.conditional_extraction|length==3 and all(.reliable==true)) else null end),
                 artifact_sha256:$artifact,grade_sha256:$grade,split:$split,differential_steps:$steps}
elif $format=="tsv" then cases else error("format must be summary or tsv") end
