# Apply to one canonical content-rollout JSON at a time.
# jq -r --slurpfile original ../codex-1/panel.json --arg format tsv -f summarize.jq ROLLOUT.json > cases.tsv
# jq --slurpfile original ../codex-1/panel.json --arg format summary -f summarize.jq ROLLOUT.json > summary.json

def tally(f): map(select(f)) | length;
def histogram(f): group_by(f) | map({key:(.[0] | f | tostring), value:length}) | from_entries;
def direction: if . < 0 then "earlier" elif . > 0 then "later" else "tie" end;
def first_error:
  .expected as $e | .generated as $g |
  first(range(0; ([$e,$g] | map(length) | max)) | select($e[.] != $g[.])) // null;
def error_rank: .first_wrong // ([.expected,.generated] | map(length) | max | . + 1);
def source_type:
  if .selected == null then "none" elif .selected == .asked_start then "asked"
  elif .selected == .other_start then "distractor" else "other" end;
# Post-hoc reconstruction only on the frozen, single-token Ann/Ben regression.
# Its literal question always names the requested person, then excludes the other.
def original_positions($case):
  (if .role=="Ann" then [2241,546] elif .role=="Ben" then [546,2241]
   else error("unknown original role") end) as $ids |
  if [.expected,.asked_start,.other_start,.role,.pair] !=
     [$case.expected,$case.asked_start,$case.other_start,$case.role,$case.pair]
  then error("original panel mismatch") else . end |
  [$ids[] as $id | [$case.prefix | to_entries[] | select(.value == $id) | .key]] as $at |
  if any($at[]; length!=2) or $at[0][1] >= $at[1][1] then error("unexpected original name occurrences")
  else {fact:$at[0][0],query:$at[0][1],other_fact:$at[1][0],query_other:$at[1][1]} end;
def site_ok($field):
  if .choice == null or .gold_positions[$field] == null then null
  else .choice[$field] == .gold_positions[$field] end;
def site_type($site):
  if .gold_positions.query == null or .choice == null then "unannotated"
  elif $site == null then "none"
  elif $site == .gold_positions.query then "asked_query"
  elif $site == .gold_positions.query_other then "excluded_query"
  elif $site == .gold_positions.fact then "asked_fact"
  elif $site == .gold_positions.other_fact then "other_fact"
  else "other_token" end;
def stage:
  if .query_ok == null then "unannotated"
  elif .query_ok == false then "wrong_query"
  elif .fact_ok == false then "right_query_wrong_fact"
  elif .right_source == false then "right_query_and_fact_wrong_value_start"
  else "right_query_fact_and_value_start" end;
def pair_count:
  group_by(.pair) | tally(length == 2 and all(.right_source));
def counts:
  {
    rows:length, pairs:(map(.pair) | unique | length), both_sources:pair_count,
    correct_query:tally(.query_ok == true), annotated_query:tally(.query_ok != null),
    correct_fact:tally(.fact_ok == true), annotated_fact:tally(.fact_ok != null),
    correct_source:tally(.right_source), correct_opening:tally(.opening),
    whole_value:tally(.whole_value), strict_reply:tally(.first_wrong == null),
    later_than_offset:tally(.rank_delta > 0),
    tied_with_offset:tally(.rank_delta == 0), earlier_than_offset:tally(.rank_delta < 0)
  };
def sample:
  {case,role,asked_start,other_start,choice,gold_positions,selected,
   source_type,query_type,fact_type,stage,expected,generated,first_wrong,wrong_expected,wrong_emitted,
   first_free,first_free_release,offset_first_wrong,offset_generated};
def summarize:
  . as $rows | ($rows | map(select(.arm=="Content"))) as $content |
  {
    arms: (group_by(.arm) | map({
      key:.[0].arm, value:(counts + {
        first_wrong:histogram(.first_wrong // "exact"), stop:histogram(.stop),
        source_type:histogram(.source_type), stage:histogram(.stage),
        query_type:histogram(.query_type),fact_type:histogram(.fact_type),
        rank_delta_vs_offset:histogram(.rank_delta),
        first_free_token:histogram(.first_free[1] // "none"),
        first_free_release:histogram(.first_free_release // "none"),
        whole_value_then_wrong:tally(.whole_value and .first_wrong != null),
        right_source_with_wrong_query:tally(.right_source and .query_ok == false)
      })}) | from_entries),
    # Matched role pairs remain intact under each of these groupings.
    marginals: (if $rows[0].split == "held" then
      [ "requested_position","fact_order","exclusion","tail","lexical_set","assignment" ] |
      map(. as $factor | {key:$factor,value:(
        $rows | group_by(.factors[$factor],.arm) | map(
          {level:.[0].factors[$factor],arm:.[0].arm} + counts
        ))}) | from_entries else {} end),
    by_requested_entity: ($content | group_by(.role) | map({
      role:.[0].role,rows:length,correct_query:tally(.query_ok == true),
      annotated_query:tally(.query_ok != null),correct_source:tally(.right_source),
      correct_opening:tally(.opening),whole_value:tally(.whole_value),strict_reply:tally(.first_wrong == null)
    })),
    examples: ($content | group_by(.stage,.source_type,.whole_value,(.rank_delta|direction)) |
      map(sort_by(.case) | .[0] | sample))
  };
def registered:
  .arms.Content as $c |
  {
    both_sources_at_least_108:($c.both_sources >= 108),
    more_pairs_than_offset:($c.both_sources > .arms.Offset.both_sources),
    more_pairs_than_query_ablated:($c.both_sources > .arms.QueryAblated.both_sources),
    all_required_marginals_at_least_half:(
      [.marginals.requested_position[],.marginals.fact_order[],.marginals.exclusion[],.marginals.tail[]]
      | map(select(.arm=="Content")) | all(2 * .both_sources >= .pairs)
    ),
    first_error_later_on_at_least_145:($c.later_than_offset >= 145)
  };

if .schema != "uor-r4.content-writer/1" then error("unrecognized rollout schema") else . end |
.artifact_sha256 as $hash |
(if $hash == "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d" then 1
 elif $hash == "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de" then 2
 else error("unrecognized artifact") end) as $seed |
.grade_sha256 as $grade |
(if $grade == "7c1d5ab8f231dab7660f6d3b811e6dd8190dbbbaccf0dbebed94a772df2ec9e3" then "held"
 elif $grade == "783d3b43a3ab082ae36496626ff350b9613d623d7f696a72f5e6ab2fb1767a3f" then "original"
 else error("unrecognized grade") end) as $split |
(if $split=="held" then 288 else 16 end) as $n |
.differential_steps as $steps |
($original[0] | INDEX(.id)) as $regression |
if ($regression|length) != 16 then error("expected frozen 16-prompt regression panel") else . end |
.rows | group_by(.case) |
if length != $n or any(.[];
  (map(.arm) | sort) != ["Content","Disabled","Offset","Oracle","QueryAblated"] or
  (map([.pair,.role,.factors,.expected,.asked_start,.other_start]) | unique | length) != 1)
then error("incomplete arm/case coverage or inconsistent reference") else . end |
map(. as $arms | ($arms[] | select(.arm == "Offset")) as $offset |
  $arms[] |
  if first_error != .first_wrong or
     (if .first_wrong == null then .wrong_expected != null or .wrong_emitted != null
      else .wrong_expected != .expected[.first_wrong] or .wrong_emitted != .generated[.first_wrong] end) or
     .right_source != (.selected == .asked_start) or .expected[-1] != 1 or
     .whole_value != (.generated[0:(.expected|length)-1] == .expected[:-1]) or
     (.steps | map(.emitted)) != .generated
  then error("trajectory/reference disagreement in " + .case) else . end |
  if $split=="original" then . + {gold_positions:original_positions($regression[.case])} else . end |
  . + {seed:$seed,split:$split,query_ok:site_ok("query"),fact_ok:site_ok("fact"),
       query_type:site_type(.choice.query),fact_type:site_type(.choice.fact),
       opening:(.generated[0] == .expected[0]),source_type:source_type,
       first_free_release:(if .first_free == null then null else .steps[.first_free[0]].release end),
       rank_delta:(error_rank - ($offset | error_rank)),offset_first_wrong:$offset.first_wrong,
       offset_generated:$offset.generated} | . + {stage:stage}
) |
if (group_by(.arm,.pair) | any(.[]; length != 2 or .[0].role == .[1].role)) or
   (map(.generated|length) | add) != $steps
then error("invalid role pairs or transition count") else . end |
if $format == "summary" then
  summarize | . + {seed:$seed,split:$split,artifact_sha256:$hash,grade_sha256:$grade,
                   differential_steps:$steps,anchor_annotations:(if $split=="held" then "registered" else "post_hoc_original_name_occurrences" end),
                   registered:(if $split=="held" then registered else null end)}
elif $format == "tsv" then
  (["seed","split","case","pair","arm","asked_role","position","fact_order","exclusion","tail",
    "gold_query","picked_query","query_ok","query_type","gold_fact","picked_fact","fact_ok","fact_type","gold_source","picked_source","source_type",
    "correct_source","opening","whole_value","strict_reply","first_wrong","wrong_expected","wrong_emitted",
    "expected_ids","generated_ids","offset_first_wrong","error_rank_delta_vs_offset","first_free_offset","first_free_id","stop"] | @tsv),
  (.[] | [.seed,.split,.case,.pair,.arm,.role,.factors.requested_position,.factors.fact_order,.factors.exclusion,.factors.tail,
    .gold_positions.query,.choice.query,.query_ok,.query_type,.gold_positions.fact,.choice.fact,.fact_ok,.fact_type,
    .asked_start,.selected,.source_type,.right_source,.opening,.whole_value,(.first_wrong==null),.first_wrong,.wrong_expected,.wrong_emitted,
    (.expected|tojson),(.generated|tojson),.offset_first_wrong,.rank_delta,.first_free[0],.first_free[1],.stop] |
    map(if . == null then "NA" else . end) | @tsv)
else error("set format to tsv or summary") end
