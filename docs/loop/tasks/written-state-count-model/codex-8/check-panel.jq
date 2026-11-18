def names: gsub("Sam";"Jim")|gsub("Tom";"Jen")|gsub("Tava";"Zava")|gsub("Tavi";"Zavi");
. as $g| $focal[0] as $f | $bare[0] as $b |
if length!=1728 or ($f|length)!=length or ($b|length)!=length or
 any(range(0;length); . as $i| $g[$i] as $r| $f[$i] as $o|
    ($r.prompt!=($o.prompt|names)) or $r.role!=($o.role|names) or $r.factors!=$o.factors or
    $r.expected!=$o.expected or $r.token_positions!=$o.token_positions or
    $r.asked_start!=$o.asked_start or $r.asked_end!=$o.asked_end or
    ($r.prefix[$r.asked_start:$r.asked_end]+[1])!=$r.expected or $b[$i]!={prefix:$r.prefix})
then error("challenge changed more than name or bare/grade disagreement") else . end |
([.[]|.prefix]+[$f[], $train[0][], $primary[0][], $oldfresh[0][], $prior[0][], $original[0][] |.prefix]) as $all |
if ($all|length)!=8944 or ($all|unique|length)!=8944 or
  (group_by(.pair)|length)!=864 or any(group_by(.pair)[];length!=2 or .[0].role==.[1].role)
then error("split overlap or unpaired challenge") else . end |
{rows:length,distinct_prefixes:($all|unique|length),pairs:(group_by(.pair)|length),
 entity_token_ids:(group_by(.role)|map(.[0] as $r|{key:$r.role,value:$r.prefix[$r.token_positions.query_start:($r.token_positions.query_end+1)]})|from_entries),
 connector_length_cells:(group_by(.factors.asked_connector_len,.factors.other_connector_len)|map(length))}
