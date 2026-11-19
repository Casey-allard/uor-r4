# Run with jq -e -r -s --argjson seed N --arg cold_sha256 HASH -f results.jq
# after verifying primary and repeat cold.sha256 and their byte equality.
# HASH is the SHA-256 of either identical cold.sha256 file. Use
# panel/range-receipt.json from this source tree for the range receipt.
# Input files, in order:
# held-grades, blind/held-blind, blind-semantic-v2, range-receipt,
# public, no-read, requested, word-start selections,
# paired *-grade and *-independent for public, no-read, requested, word-start,
# forced-gold, forced-rival and wrong-key, in that order,
# forced-gold-cursor, forced-rival-cursor, client-calibration enabled and
# no_read for this seed, and independent-preflight. All are JSON files.
def require($ok; $message): if $ok then . else error($message) end;
def key($which): if $which == "id" then .id else (.index | tostring) end;
def unique_index($which; $count):
  require(type == "array" and length == $count; "wrong index size for " + $which)
  | map({key:key($which), value:.})
  | require(all(.[]; (.key|type) == "string"); "missing index key")
  | require((map(.key)|unique|length) == $count; "duplicate index key")
  | from_entries;

def both($archive; $independent; $count):
  require($archive.schema == "uor-r4.ordered-role.grades/1"
    and $independent.schema == "uor-r4.fresh-validity-observed-grader/1"
    and $archive.selection == $independent.selection_sha256
    and $archive.gpu == $independent.gpu_sha256
    and $archive.grades == $independent.active_grades_sha256
    and $independent.counts.rows == $count; "writer and independent run digests differ")
  | ($archive.rows|unique_index("index";$count)) as $rows
  | ($independent.rows|unique_index("index";$count)) as $checked
  | require(($rows|keys) == ($checked|keys); "writer and independent row sets differ")
  | {digest:{selection:$archive.selection, gpu:$archive.gpu, grades:$archive.grades,
              generated_ids:$independent.generated_id_tape_sha256},
     rows: $rows, checked:$checked};

def first_wrong($observed; $expected):
  [range(0; [($observed|length),($expected|length)]|max)
   | select($observed[.] != $expected[.])][0];

def raw_yes_no($text; $eos; $special):
  if ($eos and ($special|not)) then
    ($text | ascii_downcase
      | if test("^\\s*no([\\s.,!?;:]|$)") then "no"
        elif test("^\\s*yes([\\s.,!?;:]|$)") then "yes" else null end)
  else null end;

def pack_arm($arm; $index; $label; $public; $chosen):
  ($index|tostring) as $key
  | $arm.rows[$key] as $r
  | $arm.checked[$key] as $v
  | require($r != null and $v != null and $r.index == $index and $v.index == $index
      and $r.id == $label.id and $v.id == $label.id
      and $r.generated == $v.generated and $r.exact == $v.active_exact
      and $r.first_wrong == $v.active_first_wrong and $r.expected == $v.active_expected
      and $r.expected_wrong_id == $v.active_expected_wrong_id
      and $r.actual_wrong_id == $v.active_actual_wrong_id
      and $r.terminal_eos == $v.terminal_eos and $r.source == $v.source
      and $r.requested == $v.role_nomination and $r.gold_requested == $label.requested
      and $v.original_expected == $label.expected;
      "archive join diverged for " + $label.id)
  | require(($public|not) or $r.text == $v.decoded_text;
      "public text disagrees with original token bytes")
  | require($chosen == null or (
      $chosen.index == $index and $chosen.prefix_sha256 == $label.prefix_sha256
      and $chosen.paired.choice.source == $r.source
      and ($chosen.paired.role_words as $rw
        | if $rw == null or $rw[0] == 4294967295 then $r.requested == null
          else $r.requested == $rw[0:2] end)
      and ($chosen.paired.role_words as $rw
        | if $rw == null or $rw[2] == 4294967295 then $r.excluded == null
          else $r.excluded == $rw[2:4] end));
      "selection and archive differ")
  | raw_yes_no($v.decoded_text; $v.terminal_eos; $v.body_has_special) as $decision
  | require($v.negative == null or $v.negative.normalized_yes_no == $decision;
      "independent negative decision disagrees with raw text")
  | {
      requested_role:$r.requested, excluded_role:$r.excluded, expected_role:$r.gold_requested,
      role_exact:$r.endpoint_exact, role_words:$chosen.paired.role_words,
      original_role_words:$chosen.original_role_words, source:$r.source,
      onset_exact:(if $label.stratum == "supported" then $r.source == $label.gold_start else null end),
      source_matches_requested_value:$r.source_matches_requested_value,
      supporting_facts:$r.supporting_facts, complete_values:$r.complete_values,
      values_disagree:$r.values_disagree, generated_ids:$v.generated,
      expected_ids:$v.original_expected, active_expected_ids:$v.active_expected,
      original_exact:$v.original_exact, exact:$v.active_exact,
      first_wrong:$v.original_first_wrong, active_first_wrong:$v.active_first_wrong,
      active_expected_wrong_id:$v.active_expected_wrong_id,
      active_actual_wrong_id:$v.active_actual_wrong_id, terminal_eos:$v.terminal_eos,
      body_has_special:$v.body_has_special, raw_text:$v.decoded_text,
      raw_text_bytes:$v.decoded_bytes, first_free:$r.first_free,
      negative:(if $v.negative == null then null
                else $v.negative + {from_raw_text:$decision} end)
    };

def metrics($rows; $arms):
  {n:($rows|length), arms: (reduce $arms[] as $name ({};
     .[$name] = {
       exact:([$rows[]|select(.arms[$name].original_exact == true)]|length),
       active_exact:([$rows[]|select(.arms[$name].exact == true)]|length),
       onset:([$rows[]|select(.arms[$name].onset_exact == true)]|length),
       source_present:([$rows[]|select(.arms[$name].source != null)]|length),
       role_exact:([$rows[]|select(.arms[$name].role_exact == true)]|length),
       ordinary_eos:([$rows[]|select(.arms[$name].terminal_eos == true)]|length),
       first_wrong:([$rows[].arms[$name].first_wrong] | group_by(.)
         | map({index:.[0], n:length}))
     }))};
def discordance($rows):
  ([$rows[] | [.arms.requested.original_exact,.arms.word_start.original_exact]]) as $paired
  | {n:($paired|length),
     both:([$paired[]|select(. == [true,true])]|length),
     requested_only:([$paired[]|select(. == [true,false])]|length),
     word_start_only:([$paired[]|select(. == [false,true])]|length),
     neither:([$paired[]|select(. == [false,false])]|length)};
def controls($rows; $arm):
  {n:($rows|length), no:([$rows[]|select(.arms[$arm].negative.classification == "no")]|length),
   copy:([$rows[]|select(.arms[$arm].negative.classification == "copy")]|length),
   fallback:([$rows[]|select(.arms[$arm].negative.classification == "fallback")]|length),
   yes:([$rows[]|select(.arms[$arm].negative.from_raw_text == "yes")]|length),
   nominated_role:([$rows[]|select(.arms[$arm].requested_role != null)]|length)};

def pair_side:
  {index,id,form,requested_name,reach_status,expected_address,expected_ids,
   arms:(.arms|map_values({requested_role,excluded_role,source,onset_exact,
                         generated_ids,expected_ids,original_exact,first_wrong,terminal_eos}))};

# Keep the output JSON and make individual rows and arms readable in a diff.
# The caller must set jq -r to write the formatted JSON rather than a string.
def member: (.key|tojson) + ": " + (.value|tojson);
def render_row:
  "    " + (del(.arms)|tojson|rtrimstr("}")) + ",\n      \"arms\": {\n"
  + (.arms|to_entries|map("        " + member)|join(",\n")) + "\n      }\n    }";
def render_array($items):
  if ($items|length) == 0 then "[]" else "[\n" + ($items|join(",\n")) + "\n  ]" end;
def render:
  "{\n" + (to_entries | map(
    "  " + (.key|tojson) + ": " +
    (if .key == "rows" then render_array(.value|map(render_row))
     elif (.value|type) == "array" then render_array(.value|map("    " + tojson))
     elif .key == "quick" or .key == "counterexamples" then
       "{\n" + (.value|to_entries|map("    " + member)|join(",\n")) + "\n  }"
     else (.value|tojson) end)) | join(",\n")) + "\n}";

require(length == 27 and ($seed == 1 or $seed == 2)
  and ($cold_sha256 | test("^[a-f0-9]{64}$")); "bad seed, cold seal or input list")
| .[0] as $labels | .[1] as $blind | .[2] as $read
| (.[3]|unique_index("id";24)) as $ranges
| .[4] as $public_selection
| (.[4].rows | unique_index("index";32)) as $public_sel
| (.[5].rows | unique_index("index";32)) as $disabled_sel
| (.[6].rows | unique_index("index";24)) as $requested_sel
| (.[7].rows | unique_index("index";24)) as $word_sel
| both(.[8];.[9];32) as $public
| both(.[10];.[11];32) as $disabled
| both(.[12];.[13];24) as $requested
| both(.[14];.[15];24) as $word_start
| both(.[16];.[17];24) as $gold
| both(.[18];.[19];24) as $rival
| both(.[20];.[21];24) as $wrong_key
| (.[22]|unique_index("index";24)) as $gold_cursor
| (.[23]|unique_index("index";24)) as $rival_cursor
| .[24] as $calibration_on | .[25] as $calibration_off
| (.[26].rows|unique_index("index";32)) as $preflight
| require(($labels|length) == 32 and ($blind|length) == 32 and ($read|length) == 32
      and ([$labels[].id]|unique|length) == 32
      and $gold.digest.generated_ids == $wrong_key.digest.generated_ids
      and $gold.digest.gpu == $wrong_key.digest.gpu
      and $gold.digest.selection == $wrong_key.digest.selection
      and $gold.digest.grades != $wrong_key.digest.grades
      and $calibration_on.selected.source == 7
      and $calibration_on.generated_ids == [3566,421,86,1]
      and $calibration_on.stop == "eos" and $calibration_off.selected.source == null
      and $calibration_off.generated_ids != $calibration_on.generated_ids;
      "panel, root, shuffled-key or planted client identity changed")
| [range(0;32) as $i | ($i|tostring) as $k | $labels[$i] as $l
   | ($l.stratum == "supported") as $ok
   | $ranges[$l.id] as $range
   | $l.facts[$l.gold_assertion_index // 0] as $fact
   | require($read[$i].index == $i and $read[$i].requested_name == $l.requested_name
     and $read[$i].answer == $l.answer_text and $preflight[$k].semantics == $read[$i].status
     and (if $ok then
       $range != null and $range.model_outcome_used == false
       and $range.literal == $l.answer_text and $range.gold_start == $l.gold_start
       and $range.rival_start == $l.rival_start
       and $range.gold_in_original_window and $range.gold_can_reach_delimiter
       and $preflight[$k].accepted_planted and ($preflight[$k].rejected|length)==7
       and $gold_cursor[$k].matched_reach
       and $rival_cursor[$k].matched_reach == $range.both_possible_in_original_window
       else $range == null end);
     "misaligned independent panel or reach receipt")
   | {index:$i,id:$l.id,family:$l.family,pair:$l.pair,direction:$l.direction,
       form:$l.form,panel_stratum:$l.panel_stratum,semantics:$read[$i].status,
       requested_name:$l.requested_name, prompt:$blind[$i].text,
       prefix_original_ids:$blind[$i].prefix, prefix_sha256:$l.prefix_sha256,
       expected_role:$l.requested, question_bounds:$l.question_bounds,
       semantic_reading:$read[$i], expected_text:$l.answer_text, expected_ids:$l.expected,
       expected_address:(if $ok then {start:$l.gold_start,end:$fact.value_span[1],
           value_span:$fact.value_span,factual_name_span:$fact.name_span,
           rival_start:$l.rival_start,rival_span:$l.rival_span}
         else null end),
       recorded_facts:($l.facts|map({name,name_span,value,value_span,literal_ids_through_eos})),
       reach_status:(if $ok|not then null
           elif $range.both_possible_in_original_window then "matched"
           elif $range.rival_in_original_window|not then "rival_outside_window"
           else "rival_beyond_copy_cap" end),
       range_receipt:$range, controls:(if $ok then
         {forced_gold_cursor:$gold_cursor[$k],forced_rival_cursor:$rival_cursor[$k]}
         else null end),
       arms:(
         {public:pack_arm($public;$i;$l;true;$public_sel[$k]),
          no_read:pack_arm($disabled;$i;$l;true;$disabled_sel[$k])}
         + (if $ok then
             {requested:pack_arm($requested;$i;$l;false;$requested_sel[$k]),
              word_start:pack_arm($word_start;$i;$l;false;$word_sel[$k]),
              forced_gold:pack_arm($gold;$i;$l;false;null),
              forced_rival:(pack_arm($rival;$i;$l;false;null)
                | . + {intervention_expected_ids:$fact.rival_ids_through_eos,
                       intervention_first_wrong:first_wrong(.generated_ids;$fact.rival_ids_through_eos)}),
              wrong_key:pack_arm($wrong_key;$i;$l;false;null)} else {} end))
     }
   | require(.arms.public.original_role_words == null
       or .arms.public.original_role_words == .arms.public.role_words;
       "public endpoint intervention")
   | require(($ok|not) or (.arms.forced_gold.source == $l.gold_start
       and .arms.forced_rival.source == $l.rival_start
       and .arms.forced_gold.original_exact and .arms.forced_gold.terminal_eos
       and (.arms.forced_rival.original_exact|not)
       and .arms.wrong_key.generated_ids == .arms.forced_gold.generated_ids
       and .arms.wrong_key.source == .arms.forced_gold.source
       and .arms.wrong_key.original_exact and (.arms.wrong_key.exact|not)
       and .arms.requested.original_role_words == .arms.public.role_words);
       "source or shuffled-key writer intervention failed")
  ] as $rows
| [$rows[]|select(.semantics=="supported")] as $positive
| [$positive[]|select(.reach_status=="matched")] as $matched
| [$positive[]|select(.reach_status!="matched")] as $tracked
| [$rows[]|select(.semantics=="absent")] as $absent
| [$rows[]|select(.semantics=="conflict")] as $conflict
| require(($positive|length)==24 and ($matched|length)==18 and ($tracked|length)==6
    and ($absent|length)==4 and ($conflict|length)==4; "wrong registered denominator")
| ["public","no_read","requested","word_start","forced_gold","forced_rival","wrong_key"] as $arms
| {
    schema:"uor-r4.fresh-validity-results/1", seed:$seed, verified_cold_manifest_sha256:$cold_sha256,
    artifact:$public_selection.artifact, blind:$public_selection.blind,
    arm_digests:{public:$public.digest,no_read:$disabled.digest,
      requested:$requested.digest,word_start:$word_start.digest,forced_gold:$gold.digest,
      forced_rival:$rival.digest,wrong_key:$wrong_key.digest},
    quick:{
      all_24:metrics($positive;$arms), matched_18:metrics($matched;$arms),
      tracked_6:metrics($tracked;$arms),
      requested_vs_word_start:{all_24:discordance($positive), matched_18:discordance($matched),
                               tracked_6:discordance($tracked)},
      absent:{public:controls($absent;"public"),no_read:controls($absent;"no_read")},
      conflict:{public:controls($conflict;"public"),no_read:controls($conflict;"no_read")},
      calibration:{same_original_ids_on_alternate_grade:true, independent_original_exact:24,
                   alternate_label_exact:0, forced_gold_exact:24, forced_rival_exact:0,
                   forced_rival_with_selector_and_copy_reach:18}
    },
    family_counts:($positive | group_by(.family)
      | map(. as $group | {family:.[0].family,
          all:metrics($group;$arms),
          matched:metrics([$group[]|select(.reach_status=="matched")];$arms),
          tracked:metrics([$group[]|select(.reach_status!="matched")];$arms),
          by_form:($group|group_by(.form)|map(. as $form|{form:.[0].form,counts:metrics($form;$arms)}))})),
    calibration:{enabled:($calibration_on|{generated_ids,text,stop,selected,artifact_sha256}),
      disabled:($calibration_off|{generated_ids,text,stop,selected,artifact_sha256}),
      wrong_entity_same_emitted_tape_sha256:$wrong_key.digest.generated_ids,
      preflight_negative_and_positive_rows:($preflight|length)},
    matched_reachable_ids:[$matched[].id],
    tracked_only:[$tracked[]|{index,id,reach_status,expected_address,range_receipt}],
    role_reversal_pairs:($positive|group_by(.pair)|map(. as $pair
      | require(length==2 and (map(.direction)|sort)==[0,1];"role-reversal pair malformed")
      | {pair:$pair[0].pair,family:$pair[0].family,
         rows:($pair|sort_by(.direction)|map(pair_side))})),
    factorial_short_long_pairs:([$positive[]|select(.panel_stratum=="factorial")
      | . + {form_pair:[.family,(.pair|split("-")|last),.direction]}] | group_by(.form_pair)
      | map(. as $pair | require(length==2 and (map(.form)|sort)==["long","short"]; "short-long pair malformed")
        | {pair:$pair[0].form_pair, both_match:all($pair[];.reach_status=="matched"),
           short:($pair[]|select(.form=="short")|pair_side),
           long:($pair[]|select(.form=="long")|pair_side)})),
    counterexamples:{
      requested_only:[$positive[]|select(.arms.requested.original_exact and (.arms.word_start.original_exact|not))|pair_side],
      word_start_only:[$positive[]|select(.arms.word_start.original_exact and (.arms.requested.original_exact|not))|pair_side],
      same_source_both_wrong:[$positive[]|select(.arms.requested.source != null and
        .arms.requested.source == .arms.word_start.source and
        (.arms.requested.original_exact|not) and (.arms.word_start.original_exact|not))|pair_side],
      public_wrong_role_or_source:[$positive[]|select((.arms.public.role_exact|not)
        or (.arms.public.onset_exact|not))|.id],
      unsupported_public_copy:[$rows[]|select(.arms.public.negative.classification=="copy")|.id]
    },
    rows:$rows
  } | render
