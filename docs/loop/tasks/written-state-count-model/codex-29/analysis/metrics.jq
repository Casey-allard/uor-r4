# Passed variables: $gold (JSONL), $blind (JSONL), and $baseline/$calibrated/$shuffled
# plus $baseline_visits/$calibrated_visits/$shuffled_visits (each a slurped JSON array).
def assert($condition; $error): if $condition then . else error($error) end;
def idx(f):
  reduce .[] as $record ({}; ($record|f) as $key
    | if has($key) then error("duplicate identity: "+$key) else .[$key]=$record end);
def rowkey: .row|tostring;
def lanekey: "\(.row)/\(.lane)";
def signed: if . >= 2147483648 then . - 4294967296 else . end;
def req: if .==[4294967295,4294967295] then null else . end;
def copy($ids;$start):
  if $start==null then null
  else
    ($ids[$start:] | to_entries | map(select(.value as $word | [16,1,0,201]|index($word))) | .[0].key) as $len
  | assert($len!=null and $len>0 and $len<=7; "invalid original copy source")
  | $ids[$start:$start+$len]+[1]
  end;
def mismatch($a;$b):
  if $a==null then null else ([($a|length),($b|length)]|max) as $n
  | [range(0;$n)|select($a[.]!=$b[.])|{offset:.,expected:$a[.],actual:$b[.]}]|.[0] end;
def candidate_reading:
  if .[4]==1 then "copy_cap"
  elif .[4]==4 then "negative_radius_release"
  elif .[4]==3 and (.[8]|signed)>0 then "positive"
  elif .[4]==3 then "nonpositive"
  elif .[4]!=2 then error("unknown candidate status")
  elif .[5]==4294967295 and .[6]==4294967295 then "missing_both"
  elif .[5]==4294967295 then "missing_positive"
  elif .[6]==4294967295 then "missing_negative"
  elif .[7]==65535 then "unfitted_positive_radius"
  else "outside_positive_radius"
  end;
def raw_score($candidates):
  if any($candidates[]; .[4]==2) then {status:2,source:null}
  else [$candidates[]|select(.[4]==3)] as $resolved
    | if ($resolved|length)==0 then {status:1,source:null}
      else (reduce $resolved[] as $next (null;
        if .==null or ($next[8]|signed)*.[9]>(.[8]|signed)*$next[9] then $next else . end)) as $best
      | if ($best[8]|signed)<=0 then {status:4,source:null}
        elif ([$resolved[]|select((.[8]|signed)*$best[9]==($best[8]|signed)*.[9])]|length)>1 then
          {status:3,source:null}
        else {status:0,source:$best[0]} end
      end
  end;
def supported: .expected_tape!=null;
def all_lanes($g):
  ["ordinary","person","assertions","disabled"] +
  (if $g|supported then ["gold_value","opposing"] else [] end);
def read_arm($name;$rows;$ledger;$roster;$tapes):
  ($rows|idx(lanekey)) as $r
  | ([$roster[]|. as $key|all_lanes($key)[]|"\($key.row)/\(.)"]|sort) as $wanted
  | assert(($r|keys)==$wanted; $name+" wrong grader roster")
  | reduce $rows[] as $entry ({out:{},seen:{}};
      ($entry.row|tostring) as $id | $roster[$id] as $key | $tapes[$id].ids as $ids
      | ($entry.lane) as $lane
      | assert($entry.stratum==$key.stratum and $entry.expected==$key.expected_tape and $entry.value_spans==$key.selected_value_spans;
        $name+" key differs at "+($entry|lanekey))
      | (if $key|supported then $entry.tape==$key.expected_tape else null end) as $exact
      | ($key|supported and $entry.source!=null and any($key.selected_value_spans[]; .[0]==$entry.source)) as $source_ok
      | assert($entry.exact==$exact and $entry.first==mismatch($key.expected_tape;$entry.tape)
          and $entry.source_correct==$source_ok and $entry.person_correct==($entry.person==$key.requested);
          $name+" contradictory result at "+($entry|lanekey))
      | assert(($entry.tape|length)>0 and ($entry.tape|length)<=16 and ($entry.tape[:-1]|index(1))==null
          and ($entry.source==null or copy($ids;$entry.source)==$entry.tape); $name+" invalid original output "+($entry|lanekey))
      | assert(if $lane=="gold_value" then $exact and $entry.source==$key.selected_value_spans[0][0]
          elif $lane=="opposing" then $exact==false and $entry.source!=null and ($key.wrong_source==null or $entry.source==$key.wrong_source)
          elif $lane=="disabled" then $entry.source==null
          else $entry.decision.choice.source==$entry.source end; $name+" invalid lane "+($entry|lanekey))
      | reduce $entry.stages[] as $stage (.;
          ($stage.visit|tostring) as $ordinal
          | assert(.seen[$ordinal]==null and $ledger[$stage.visit]=={row:$entry.row,lane:$lane,stage:$stage};
            $name+" stage not in unique visit ledger")
          | assert($stage.actual_query==$entry.person and $stage.query==$key.requested; $name+" query trace differs")
          | assert(all($stage.candidate_words[]; length==10
                and .[0]-$stage.head[1]==.[2] and .[1]>.[0] and .[1]<=$stage.fence and .[3]==$stage.fence-.[0]
                and (if .[4]==4 then .[5]>.[6] and .[6]<=.[8] and .[8]!=65535 else true end));
            $name+" invalid candidate geometry/negative-radius trace")
          | assert(raw_score($stage.candidate_words)==$stage.onset; $name+" onset status/source differs from trace ranking")
          | .seen[$ordinal]=true)
      | if $lane=="assertions" then
          assert(($entry.stages|map(.head)|sort)==([$key.selected[] as $i|$key.assertions[$i].person]|sort);
            $name+" supplied assertions differ")
        else . end
      | ($entry.decision.role_words // []) as $role
      | assert(($role|length)==0 or ($role|length)==8; $name+" invalid role word width")
      | assert(if $lane=="ordinary" and ($role|length)==8 then
          ($role[0:2]|req)==$entry.person
        else true end; $name+" ordinary role slot disagrees with query endpoint")
      | assert(if (["ordinary","person","assertions"]|index($lane))!=null and $entry.source!=null then
          any($entry.stages[]; .head[0]==$entry.decision.choice.fact and .onset.source==$entry.source
            and .completion[0]==$entry.source)
        else true end; $name+" released source lacks its completed original assertion")
      | ($entry + {
          actual_requested: ($role[0:2]|if length==2 then req else null end),
          actual_excluded: ($role[2:4]|if length==2 then req else null end)
        }) as $row
      | .out[$entry|lanekey]=$row)
  | assert((.seen|length)==($ledger|length); $name+" unaccounted visits")
  | .out;
def classify_release($key;$head;$source):
  any($key.selected[] as $i|$key.assertions[$i]; .person==$head and .value[0]==$source);
def trace_pairs($before;$after;$key):
  assert(($before.stages|map([.head,.fence,.actual_query]))==($after.stages|map([.head,.fence,.actual_query]));
    "fixed assertion route differs across arms")
  | assert($before.actual_requested==$after.actual_requested and $before.actual_excluded==$after.actual_excluded
      and $before.person==$after.person; "fixed role differs across arms")
  | [$after.stages[] as $right|$before.stages[]|select(.head==$right.head and .fence==$right.fence and .actual_query==$right.actual_query)
   | . as $left
   | assert(($left.candidate_words|map(.[0:4]))==($right.candidate_words|map(.[0:4]));"candidate coordinate set changed")
   | [$left.candidate_words[], $right.candidate_words[]]|group_by(.[0])[]
   | .[0] as $a|.[1] as $b
   | if ($a[4]==$b[4] or ($a[4]==2 and $b[4]==4)) and $a[5:8]==$b[5:8]
          and (if $a[4]==3 then $a[8:10]==$b[8:10] else true end) then
       select($a[4]==2 and $b[4]==4)
       | {head:$right.head,source:$b[0],is_gold:classify_release($key;$right.head;$b[0]),
          previous:($a|candidate_reading),negative_distance:$b[6],negative_radius:$b[8],positive_distance:$b[5]}
     else error("fixed scorer words cannot change outside signed rival admission") end];
def good_late($entry;$key):
  $entry.exact==true and $entry.source_correct
  and $entry.source!=null and any($entry.stages[]; .onset.source==$entry.source
    and $entry.source - .head[1]>3
    and classify_release($key;.head;$entry.source));
def first_problem($row;$key):
  if $row.exact==true then null
  elif $row.exact==null then (if $row.source==null then null else "unsupported_release" end)
  elif (["ordinary","person","assertions"]|index($row.lane))==null then null
  elif $row.person!=$key.requested then "query_endpoint"
  else [$row.stages[]|. as $stage|$key.selected[] as $i
    | $key.assertions[$i]|select(.person==$stage.head)
    | {stage:$stage,gold:.value[0]}] as $selected
  | if ($selected|length)<($key.selected|length) then "assertion_binding"
    elif $row.source!=null and ($row.source_correct|not) then "wrong_released_onset"
    elif any($selected[]; . as $v | all(.stage.candidate_words[]; .[0]!=$v.gold)) then "gold_ineligible"
    elif any($selected[]; . as $v | any(.stage.candidate_words[];
      .[0]==$v.gold and .[4]==4)) then "gold_released_as_negative"
    elif any($selected[]; . as $v | any(.stage.candidate_words[];
      .[0]==$v.gold and (. [4]==1 or .[4]==2))) then "gold_unresolved"
    elif any($selected[];.stage.onset.status==2) then "unseen_rival"
    elif any($selected[];.stage.onset.status==3) then "tie"
    elif any($selected[];.stage.onset.status==4) then "nonpositive"
    elif any($selected[];.stage.onset.source!=.gold) then "rival_selected_before_completion"
    elif $row.source==null then "completion_or_agreement"
    else "writer_output" end
  end;
def status($row;$key):
  {row:$row.row,lane:$row.lane,person_correct:$row.person_correct,
   requested: $row.actual_requested, excluded:$row.actual_excluded,
   excluded_correct:($row.actual_excluded==$key.role_targets.excluded),
   unrequested_in_role:($key.role_targets.unrequested!=null and
     ($row.actual_requested==$key.role_targets.unrequested or $row.actual_excluded==$key.role_targets.unrequested)),
   source:$row.source,source_correct:$row.source_correct,tape:$row.tape,exact:$row.exact,first:$row.first,
   first_problem:first_problem($row;$key),
   matched_onsets:[$row.stages[]|. as $stage|$key.selected[] as $i
     | $key.assertions[$i] | select(.person==$stage.head)
     | {head:.person, gold_offset:(.value[0]-.person[0]),
         selected_offset:(if $stage.onset.source!=null then $stage.onset.source-.person[0] else null end)}],
   query_missing:($row.person==null),assertion_heads:[$row.stages[].head],
   visits:[$row.stages[]|. as $v|{head,fence,onset,completion,
     candidate_readings:[.candidate_words[]|{source:.[0],class:candidate_reading,gold:classify_release($key;$v.head;.[0]),words:.}]}]};
def arm_names: ["baseline","calibrated","shuffled"];
def lane_names: ["ordinary","person","assertions","gold_value","opposing","disabled"];
def paired($rows):
  [lane_names[] as $lane | [$rows[]|. as $row|.lanes[]|select(.lane==$lane)|. + {row:$row.row}] as $observed
   | {lane:$lane,n:($observed|length),
      arms:[arm_names[] as $arm
        | {name:$arm,exact:[$observed[]|select(.[$arm].exact==true)|.row],
           requested_exact:[$observed[]|select(.[$arm].person_correct)|.row],
           excluded_exact:[$observed[]|select(.[$arm].excluded_correct)|.row],
           unrequested_in_role:[$observed[]|select(.[$arm].unrequested_in_role)|.row],
           wrong_source:[$observed[]|select(.[$arm].exact!=null and .[$arm].source!=null and (.[$arm].source_correct|not))|.row],
           unsupported_release:[$observed[]|select(.[$arm].exact==null and .[$arm].source!=null)|.row]}],
      treatment:[["baseline","calibrated"],["baseline","shuffled"],["shuffled","calibrated"]]
      | map(.[0] as $left | .[1] as $right
        | {before:$left,after:$right,
         gains:[$observed[]|select(.[$left].exact==false and .[$right].exact==true)|.row],
         losses:[$observed[]|select(.[$left].exact==true and .[$right].exact==false)|.row]})}];
def context_pairs($details):
  [$details[]|select(.matched_pair!=null)]|group_by(.matched_pair)|map(
    sort_by(.context_arm) as $p
    | assert(($p|length)==2 and ($p|map(.context_arm))==[0,1] and $p[0].expected_tape!=$p[1].expected_tape;
        "incomplete or equivalent contextual pair")
    | {pair:$p[0].matched_pair,rows:($p|map(.row)),lanes:[
       ["ordinary","person","assertions"][] as $lane
       | {lane:$lane,arms:[arm_names[] as $arm
         | [$p[]|.lanes[]|select(.lane==$lane)|.[$arm]] as $read
         | ($read|map(.matched_onsets[0].selected_offset)) as $relative
         | {arm:$arm,exact:($read|map(.exact)),onset_offsets:$relative,
            same_observed_boundary:($relative[0]!=null and $relative[0]==$relative[1]),
            both_exact:all($read[]; .exact==true)}]}]} );
($gold|idx(rowkey)) as $roster
| ($blind|idx(rowkey)) as $tapes
| assert(($roster|keys)==($tapes|keys);"sealed panel identity mismatch")
| assert(all($gold[];. as $g|if supported then all(.selected_value_spans[]; copy($tapes[$g.row|tostring].ids;.[0])==$g.expected_tape) else true end);
    "gold does not replay from exact original IDs")
| read_arm("baseline";$baseline[0];$baseline_visits[0];$roster;$tapes) as $a
| read_arm("calibrated";$calibrated[0];$calibrated_visits[0];$roster;$tapes) as $b
| read_arm("shuffled";$shuffled[0];$shuffled_visits[0];$roster;$tapes) as $c
| [$gold[]|. as $key
  | {row,stratum,family,connector_arm,object_arm,context_arm,matched_pair,role_factors,expected_value,expected_tape,
      role_targets,actual_connector:(if .gold_source!=null then .gold_source-.assertion_head[1] else null end),
      lanes:[all_lanes($key)[] as $lane|"\($key.row)/\($lane)" as $id
        | {lane:$lane,baseline:status($a[$id];$key),calibrated:status($b[$id];$key),shuffled:status($c[$id];$key),
            calibrated_releases:trace_pairs($a[$id];$b[$id];$key),shuffled_releases:trace_pairs($a[$id];$c[$id];$key),
            calibrated_late_exact:good_late($b[$id];$key),shuffled_late_exact:good_late($c[$id];$key),
            baseline_late_exact:good_late($a[$id];$key)}]}] as $details
| {schema:"uor-r4.d29-rival-analysis/1",n:($details|length),rows:$details,
   overall:paired($details),by_stratum:($details|group_by(.stratum)|map({group:.[0].stratum,results:paired(.)})),
   matched_families:([$details[]|select(.stratum=="matched")]|group_by(.family)|map({family:.[0].family,results:paired(.)})),
   matched_connectors:([$details[]|select(.stratum=="matched")]|group_by(.connector_arm)|map({connector:.[0].connector_arm,results:paired(.)})),
   context_pairs:context_pairs($details)}
