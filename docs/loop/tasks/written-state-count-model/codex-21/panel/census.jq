# Read the tokenizer-only gold spans and candidate values. This audit has no scorer inputs.
def exact($ids; $s; $e): $ids[$s:$e] + [1];
def overlap($a; $b): ($a[0] < $b[1] and $b[0] < $a[1]);
def supported:
  select(.stratum == "supported");

$grade[0] as $g |
$blind[0] as $b |
[$g[] | supported] as $s |
[$g[] | select(.stratum != "supported")] as $n |
if ($g | length) != 102 or ($b | length) != 102 or ($s | length) != 96
then error("row census differs") else . end |
if ($s | group_by([.family,.distractor_first,.name_order_reversed,.reversed,.object_swap,.connector_swap]) |
    length != 96 or any(.[]; length != 1)) or
   ($s | group_by(.family) | length != 3 or any(.[]; length != 32))
then error("missing or duplicate factorial cell") else . end |
if any($s[]; (.question_mentions | length) != 2 or
        overlap(.requested;.excluded) or
        ((.requested[0] > .excluded[0]) != .distractor_first) or
        ((.direction == 1) != (.name_order_reversed != .distractor_first)) or
        (.gold_assertion_index == .direction) == .reversed)
then error("unsupported role crossing") else . end |
if ($s | group_by([.family,.direction,.distractor_first,.object_swap,.connector_swap]) |
    any(.[]; ((.[0].facts | map([.name,.verb_phrase,.value]) | reverse) !=
              (.[1].facts | map([.name,.verb_phrase,.value])) or
             .[0].answer_text != .[1].answer_text or .[0].expected != .[1].expected)))
then error("assertion swap altered fact content") else . end |
if ($s | group_by([.family,.connector_swap]) | any(.[];
      group_by(.facts[0].name) | any(.[]; ([.[].facts[0].verb_phrase]|unique|length)!=1))) or
   ($s | group_by([.family,.object_swap]) | any(.[];
      group_by(.facts[0].name) | any(.[]; ([.[].facts[0].value]|unique|length)!=1))) or
   ($s | group_by([.family,.direction]) | any(.[];
     ([.[] | .facts[.gold_assertion_index].verb_phrase]|unique|length)!=2 or
     ([.[].answer_text]|unique|length)!=2))
then error("object and connector assignment did not vary independently") else . end |
if any(range(0; $g|length); . as $i | $g[$i] as $r | $b[$i].prefix as $ids |
    if $r.stratum == "supported" then
      [$r.facts[] | select(.name == $r.requested_name)] as $target |
      [$r.facts[] | select(.name == $r.excluded_name)] as $other |
      ($r.expected != exact($ids; $target[0].value_span[0]; $target[0].value_span[1]) or
      any([$target,$other][]; length != 1 or
        (.[0] as $f | all($f.legal[]; .copy_ids_through_eos != exact($ids;$f.value_span[0];$f.value_span[1])))) or
      $target[0].value == $other[0].value)
    else $r.expected != null or $r.gold_start != null end)
then error("intended or opposing original writer tape is unrepresentable") else . end |
if any($s[]; .family == "supplies-transfer" and
    any(.facts[]; [.legal[].offset] != [1,2,3]))
then error("transfer family lost a legal writer start") else . end |
{rows:($g|length), supported:($s|length),
 unsupported:($n|group_by(.stratum)|map({stratum:.[0].stratum, rows:length})),
 factorial_families:($s|group_by(.family)|map({family:.[0].family, cells:length})),
 fixed_positions: {
   first_fact: ([$s[]|select(.gold_assertion_index == 0)]|length),
   second_fact: ([$s[]|select(.gold_assertion_index == 1)]|length),
   first_query_name: ([$s[]|select(.distractor_first|not)]|length),
   second_query_name: ([$s[]|select(.distractor_first)]|length)
 },
 transfer_gold_offsets: ($s | map(select(.family == "supplies-transfer")) |
     group_by(.gold_offset) | map({offset: .[0].gold_offset, rows:length})),
 copyable_gold_rows:($s|length), copyable_opposing_rows:($s|length)}
