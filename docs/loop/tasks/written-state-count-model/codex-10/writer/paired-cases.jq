# Use complete first-seed TSVs; compare a fixed query within the same fact pair.
def rows($blob):
  ($blob|split("\n")|map(select(length>0)|split("\t"))) as $data |
  $data[0] as $header | $data[1:] |
  map(. as $r | if ($r|length)!=($header|length) then error("short TSV row") end |
      [$header,$r]|transpose|map({key:.[0],value:.[1]})|from_entries);
def both_exact(f): group_by(.pair)|map(select(all(.[];f=="exact")))|length;
def pair_change(f):
  group_by(.pair)|map({before:all(.[];.old=="exact"),after:all(.[];f=="exact")}) |
  {gained:map(select((.before|not) and .after))|length,lost:map(select(.before and (.after|not)))|length};
(rows($b)) as $baseline | (rows($r)) as $role | (rows($v)) as $boundary | (rows($rv)) as $joint |
if ([$baseline,$role,$boundary,$joint]|map(map([.case,.prefix_sha256,.pair,.expected]))|unique|length)!=1 then error("case mismatch") end |
[range($baseline|length) as $i | $baseline[$i] as $row |
 {case:$row.case,prefix_sha256:$row.prefix_sha256,pair:$row.pair,expected:($row.expected|fromjson),old:$row.first_wrong,role:$role[$i].first_wrong,
  boundary:$boundary[$i].first_wrong,both:$joint[$i].first_wrong,got:($joint[$i].generated|fromjson),base:($row.generated|fromjson),
  sources:[$row.source,$role[$i].source,$boundary[$i].source,$joint[$i].source],
  factors:[$row.requested_position,$row.exclusion,$row.tail,$row.connector_length]}
] as $rows |
if ($rows|group_by(.pair)|any(.[];length!=2)) then error("role pair mismatch") end |
{rows:($rows|length),paired_exact:($rows|{baseline:both_exact(.old),role:both_exact(.role),boundary:both_exact(.boundary),both:both_exact(.both)}),
 paired_discordance:($rows|{role:pair_change(.role),boundary:pair_change(.boundary),both:pair_change(.both)}),
 interaction:($rows|group_by([.old,.role,.boundary,.both]|map(.=="exact")|@csv)|map({key:(.[0]|[.old,.role,.boundary,.both]|map(.=="exact")|@csv),value:length})|from_entries),
 both_repaired_example:([$rows[]|select(.old!="exact" and .both=="exact")][0]//null),
 both_regressed_example:([$rows[]|select(.old=="exact" and .both!="exact")][0]//null),
 both_persistent_example:([$rows[]|select(.old!="exact" and .both!="exact")][0]//null)}
