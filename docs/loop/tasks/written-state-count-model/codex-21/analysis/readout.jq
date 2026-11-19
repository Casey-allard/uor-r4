# Post-run only. Reads sealed original tapes and writer records, never changes a grade.
def require($condition; $message): if $condition then . else error($message) end;
def indexed($file; $key):
  $file.rows | map({key:(.[$key]|tostring),value:.}) | from_entries
  | require(length == ($file.rows|length); "duplicate row ordinal");
def source_trace:
  .rows | group_by(.ordinal)
  | map({key:(.[0].ordinal|tostring),value:.}) | from_entries;
def mismatch($expected; $emitted):
  if $expected == null then null
  else first(range([($expected|length), ($emitted|length)]|max)
           | select($expected[.] != $emitted[.])) // null end;
def piece($id): if $id == null then null else $pieces[0][$id] end;
def query_tokens($tape; $span):
  if $span == null then null else $tape[$span[0]:$span[1]] | map(piece(.)) end;
def role_margin($row; $span):
  if $span == null then null else
    (first($row.requested_occurrence.candidates[] | select(.span==$span)
       | {contrast, scores, clause_evidence}) // null) end;
def device_roles($rows; $grades; $device; $arm):
  $device.rows as $device_rows |
  require(($device_rows|length)==($rows|length); "role trace census differs") |
  [$rows[] | . as $r | $device_rows[$r.ordinal] as $row |
   (if $row.role_words[0]==4294967295 then null else $row.role_words[0:2] end) as $chosen |
   require($row.index==$r.ordinal and $row.prefix_sha256==$grades[$r.ordinal].prefix_sha256 and
           $chosen==$r.arms[$arm].query; "role words disagree with writer endpoint") |
   select($r.stratum=="supported") |
   {ordinal:$r.ordinal,family:$r.family,exclusion_first:$r.exclusion_first,
    requested:role_margin($row;$r.gold_query),
    excluded:role_margin($row;$r.excluded_query)}];
def source($g; $start):
  if $start == null then {classification:"no_source"}
  else (first($g.facts[] | select(.name_span[0] <= $start and $start < .delimiter[0])) // null) as $f
  | if $f == null then {classification:"outside_fact"}
    else
      $f.value_span[0] as $gold |
      ($f.name == $g.requested_name and $g.stratum == "supported") as $requested |
      {person:$f.name, offset:($start - $f.name_span[1]), value_span:$f.value_span,
       classification:
        (if $start == $gold then
           if $requested then "correct_value" else "other_value" end
         elif $start > $gold then
           if $requested then "requested_suffix" else "other_suffix" end
         elif $start >= $f.connector_span[0] then
           if $requested then "requested_connector" else "other_connector" end
         else "name" end)}
    end
  end;
def endpoint($g; $span):
  if $span == null then "none"
  elif $span == $g.requested then "requested"
  elif $span == $g.excluded then "excluded"
  elif any($g.facts[]; .name_span == $span and .name == $g.requested_name) then "requested_fact"
  else "other" end;
def normalize($g; $row; $source_rows; $audit; $observed; $tape; $expect):
  if $row == null then null else
  $row.generated as $got | mismatch($expect; $got) as $first |
  ($expect != null and $got == $expect and $got[-1] == 1) as $exact |
  require($row.id == $g.id and ($got|type) == "array" and
    $row.expected == $expect and $row.first_wrong == $first and
    $row.terminal_eos == ($got[-1] == 1) and
    $row.exact == (if $expect == null then null else $exact end) and
    $observed.generated == $got and $observed.source == $row.source and
    $observed.role_nomination == $row.requested and
    $observed.original_exact == $row.exact and $observed.original_first_wrong == $first and
    ($first == null or ($row.expected_wrong_id == $expect[$first] and $row.actual_wrong_id == $got[$first])) and
    all($got[]; piece(.) != null) and
    ($audit == null or ($audit.consensus.source == $row.source and
      ($row.source == null or any($source_rows[]; .source == $row.source))));
    "writer, raw source trace or independent witness disagrees at \($g.id)") |
  {query:$row.requested, query_tokens:query_tokens($tape; $row.requested),
   endpoint:endpoint($g; $row.requested), excluded:$row.excluded,
   source:$row.source, source_kind:source($g; $row.source),
   source_token:(if $row.source == null then null else piece($tape[$row.source]) end),
   value_status:(if $source_rows==null then null
     elif ($source_rows|length)==1 then $source_rows[0].status
     else "multiple_assertions" end),
   candidates:[$source_rows[]? | .assertion_index as $assertion |
               .candidates[] | . + {assertion_index:$assertion}],
   emitted:$got, text:$observed.decoded_text, terminal_eos:($got[-1] == 1),
   exact:(if $expect == null then null else $exact end),
   mismatch:(if $first == null then null else
      {index:$first,expected:piece($expect[$first]),actual:piece($got[$first])} end),
   negative:$observed.negative}
  end;
def compare($old; $new):
  if $old and $new then "retained"
  elif $new then "recovered"
  elif $old then "regressed"
  else "both_wrong" end;
def pair($a; $b):
  map(. as $r | .arms[$a] as $old | .arms[$b] as $new |
    {ordinal,id, family, exclusion_first, name_b_first, reverse_facts, connector_swap, object_swap,
     span: compare(($old.endpoint=="requested");($new.endpoint=="requested")),
     source: compare(($old.source_kind.classification=="correct_value");
                     ($new.source_kind.classification=="correct_value")),
     exact: compare($old.exact; $new.exact),
     old_source: $old.source_kind.classification, new_source: $new.source_kind.classification,
     old_source_offset:$old.source,new_source_offset:$new.source,
     old_source_token:$old.source_token,new_source_token:$new.source_token,
     old_query:$old.query,new_query:$new.query,
     old_status:$old.value_status,new_status:$new.value_status,
     old_mismatch:$old.mismatch,new_mismatch:$new.mismatch,
     changed_ids:($old.emitted != $new.emitted),
     changed_query:($old.query != $new.query),
     new_wrong_source:($new.source != null and $new.source != $old.source and
                       $new.source_kind.classification != "correct_value"),
     new_wrong_but_both_wrong:($old.exact == false and $new.exact == false and $old.emitted != $new.emitted),
     earlier_unseen_gold: any($old.candidates[]?;
        . as $c | $c.source==$r.gold_start and $c.status=="unseen" and
        any($old.candidates[]?; .assertion_index == $c.assertion_index and .source > $c.source)),
     bypassed_unseen: [($old.candidates // [])[] as $c |
         select($c.status=="unseen" and any($new.candidates[]?;
            .assertion_index==$c.assertion_index and .source==$c.source and .status=="outside_radius")) |
         $c.source]});
def counts($key):
  group_by(.[$key]) | map({key:(.[0][$key]|tostring),value:length}) | from_entries;
def arm_counts($arm):
  map(.arms[$arm]) | {span:(map(select(.endpoint=="requested"))|length),
    endpoints:counts("endpoint"),sources:(map(.source_kind)|counts("classification")),
    exact:(map(select(.exact))|length), eos:(map(select(.terminal_eos))|length),
    cap_without_eos:(map(select((.terminal_eos|not) and (.emitted|length)==16))|length),
    first_mismatches:(map(.mismatch.index)|map({index:.})|counts("index"))};
def summary($rows; $arms; $old; $new):
  ($rows | pair($old; $new)) as $paired |
  {rows:($rows|length),
   arms:($arms|map(. as $arm|{key:$arm,value:($rows|arm_counts($arm))})|from_entries),
   span_changes:($paired|counts("span")),
   source_changes:($paired|counts("source")),
   exact_changes:($paired|counts("exact")),
   wrong_to_different_wrong:([$paired[]|select(.new_wrong_but_both_wrong)]|length),
   old_new_source:([$paired[]|{class:(.old_source+" → "+.new_source)}]|counts("class")),
   new_wrong_sources:([$paired[]|select(.new_wrong_source)]|length),
   early_unseen_gold:([$paired[]|select(.earlier_unseen_gold)]|length),
   retained_unseen:([$paired[]|select(.old_status=="unseen" and .new_status=="unseen")]|length),
   retained_ties:([$paired[]|select(.old_status=="tie" and .new_status=="tie")]|length),
   early_unseen_gold_wrong_releases:([$paired[]|select(.earlier_unseen_gold and .new_wrong_source and
      (.bypassed_unseen|length)>0)]|length)};
def dataset($g; $blind; $input; $observed; $sources; $audits; $arms):
  $g | to_entries | map(.key as $i | (.key|tostring) as $k | .value as $row |
    $blind[$i].prefix as $tape |
    ($row.stratum == "supported") as $is_supported |
    (if $is_supported then
      $row.facts[$row.gold_assertion_index].value_span as [$start,$end] |
      $tape[$start:$end] + [1]
    else null end) as $expect |
    require(($tape|type)=="array" and $row.expected == $expect and
      ($is_supported | not or ($row.gold_start == $row.facts[$row.gold_assertion_index].value_span[0]));
      "independently sliced answer differs from seal") |
    {ordinal:$i,id:$row.id,family:$row.family, stratum:$row.stratum,
     name_b_first:$row.name_order_reversed, exclusion_first:$row.distractor_first,
     reverse_facts:$row.reversed, object_swap:$row.object_swap,connector_swap:$row.connector_swap,
     person:$row.requested_name,prompt:$blind[$i].text,gold_query:$row.requested,
     gold_query_tokens:query_tokens($tape;$row.requested),
     excluded_query:$row.excluded, gold_start:$row.gold_start,gold_offset:$row.gold_offset,
     gold_answer:$row.answer_text,expected:$expect,
     arms:($arms|map(. as $arm |
       ($is_supported or (["reference","clause","no_read"]|index($arm))!=null) as $runs |
       require(($input[$arm][$k]!=null)==$runs and ($observed[$arm][$k]!=null)==$runs;
          "writer and observations do not cover the required ordinals") |
       {key:$arm,value:normalize($row;$input[$arm][$k];$sources[$arm][$k];$audits[$arm][$k];
                   $observed[$arm][$k];$tape;$expect)})|from_entries)});
def transitions($rows; $a; $b):
  $rows | group_by([.family,.exclusion_first,.name_b_first])
  | map({family:.[0].family,exclusion_first:.[0].exclusion_first, name_b_first:.[0].name_b_first}
      + summary(.; [$a,$b];$a;$b));
def negatives($rows):
  ($rows|map(select(.stratum!="supported"))) as $unsupported |
  {count:($unsupported|length),
   arms:(["reference","clause","no_read"]|map(. as $arm |
     {key:$arm,value:{source:([$unsupported[].arms[$arm]|select(.source!=null)]|length),
       eos:([$unsupported[].arms[$arm]|select(.terminal_eos)]|length),
       cap_without_eos:([$unsupported[].arms[$arm]|select((.terminal_eos|not) and (.emitted|length)==16)]|length),
       exact_literal:([$unsupported[].arms[$arm]|select(.negative.matches_recorded_literal==true)]|length),
       no_without_copy:([$unsupported[].arms[$arm]|select(.negative.no_without_copy==true)]|length),
       classifications:([$unsupported[].arms[$arm].negative]|counts("classification"))}})|from_entries),
   newly_written_sources:([$unsupported[]|select(.arms.clause.source!=null and .arms.reference.source!=.arms.clause.source)|.ordinal]),
   newly_written_literals:([$unsupported[]|select(.arms.reference.negative.matches_recorded_literal!=true and
       .arms.clause.negative.matches_recorded_literal==true)|.ordinal])};

["reference","clause","supplied","veto","wrong_role","no_read"] as $arms |
($writers[0] | map_values(indexed(.;"index"))) as $writer_rows |
($observations[0] | map_values(indexed(.;"index"))) as $observed_rows |
($traces[0] | map_values(source_trace)) as $source_rows |
($audits[0] | map_values(indexed(.;"ordinal"))) as $audit_rows |
dataset($labels[0];$blind[0];$writer_rows;$observed_rows;$source_rows;$audit_rows;$arms) as $rows |
($rows|map(select(.stratum=="supported"))) as $sup |
device_roles($rows;$labels[0];$reference_roles[0];"reference") as $ref_margin |
device_roles($rows;$labels[0];$clause_roles[0];"clause") as $cls_margin |
require(($rows|length)==102 and ($sup|length)==96 and
   all($arms[]; . as $a | ($writer_rows[$a]|length)==
      (if (["reference","clause","no_read"]|index($a))!=null then 102 else 96 end)) and
   all(["reference","clause","supplied","veto","wrong_role"][]; . as $a |
     ($source_rows[$a]|length)==102 and ($audit_rows[$a]|length)==102 and
     all(range(102); $source_rows[$a][tostring]!=null and $audit_rows[$a][tostring]!=null));
    "unexpected arm or candidate-trace census") |
{schema:"uor-r4.clause-entity-veto-paired-readout/1",seed:$seed,
 totals:{rows:($rows|length),supported:($sup|length),unsupported:(($rows|length)-($sup|length))},
 supported:{role:summary($sup; ["reference","clause"]; "reference"; "clause"),
   value:summary($sup; ["supplied","veto"]; "supplied"; "veto"),
   other_controls:{wrong_role:($sup|arm_counts("wrong_role")),no_read:($sup|arm_counts("no_read"))}},
 roles_by_cell:transitions($sup;"reference";"clause"),
 values_by_cell:transitions($sup;"supplied";"veto"),
 role_pairs:($sup|pair("reference";"clause")),value_pairs:($sup|pair("supplied";"veto")),
 role_margins:([range($ref_margin|length) as $i | $ref_margin[$i] as $ref | $cls_margin[$i] as $cls |
   {ordinal:$ref.ordinal, family:$ref.family, exclusion_first:$ref.exclusion_first,
    reference:[$ref.requested.contrast,$ref.excluded.contrast],
    clause:[$cls.requested.contrast,$cls.excluded.contrast],
    clause_bounds:[$cls.requested.clause_evidence.span,$cls.excluded.clause_evidence.span]}]),
 unsupported:negatives($rows),
 verification:$verification[0],rows:$rows}
