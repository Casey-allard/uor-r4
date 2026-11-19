# Both readers use {index,person,answer,status}; retain any mismatch separately.
def authored($rows):
  $rows | to_entries | map(
    {index: .key, person: .value.requested_name, answer: .value.answer_text,
     status: .value.stratum});

$raw[0] as $prompts |
$reader_a[0] as $a |
$reader_b[0] as $b |
authored($seed1[0]) as $gold1 |
authored($seed2[0]) as $gold2 |
if any([$prompts, $a, $b, $gold1, $gold2][]; length != 42)
then error("expected 42 rows in each input") else . end |
[range(0; 42) as $i |
  select($prompts[$i].index != $i or ($prompts[$i].text | type) != "string" or
         $a[$i] != $b[$i] or $a[$i] != $gold1[$i] or $a[$i] != $gold2[$i]) |
  {index: $i, reader_a: $a[$i], reader_b: $b[$i],
   author_seed1: $gold1[$i], author_seed2: $gold2[$i]}] as $disagreements |
{rows: ($prompts | length),
 statuses: ($gold1 | group_by(.status) | map({status: .[0].status, rows: length})),
 all_agree: ($disagreements | length == 0), disagreements: $disagreements}
