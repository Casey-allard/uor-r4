. as $g | ($bare[0]|map(.prefix)) as $prefix |
([$train[0],$primary[0],$fresh[0],$transfer[0],$challenge[0]]|add) as $seen |
($seen|map(.prefix|tojson)|INDEX(.)) as $used |
([$g[]|.token_positions as $s|select((.prefix[.asked_start:.asked_end]+[1]) != .expected or (.asked_start==.other_start)
  or .factors.asked_connector_len != (.asked_start-$s.fact_end-1) or $s.query_start <= $s.fact_end)]|length) as $bad |
if ($g|map(.prefix))!=$prefix or ($prefix|unique|length)!=1728 or ($g|length)!=1728
 or any($bare[0][];keys!=["prefix"]) or any($prefix[];$used[tojson]!=null)
 or $bad!=0 or any($g|group_by(.pair)[];length!=2 or .[0].expected==.[1].expected or .[0].role==.[1].role)
 or ($negative[0]|length)!=48 or ($neg_bare[0]|map(.prefix))!=($negative[0]|map(.prefix))
 or any($neg_bare[0][];keys!=["prefix"]) or any($negative[0][];.valid_unique_source!=null or $used[(.prefix|tojson)]!=null)
 or ([$prefix[],($negative[0][].prefix)]|unique|length)!=1776
then error("inconsistent held family corpus, ambiguous role, overlap or grade leakage") else . end |
{positive_rows:length, opposite_role_pairs:(group_by(.pair)|length),historical_and_train_rows_checked:($seen|length),
  positive_overlap:0,duplicate_or_conflicting_visible_prefixes:0,
  requested_positions:(group_by(.factors.requested_position)|map([.[0].factors.requested_position,length])),
  exclusion_positions:(group_by(.factors.exclusion)|map([.[0].factors.exclusion,length])),
  tails:(group_by(.factors.tail)|map([.[0].factors.tail,length])),
  source_lengths:(group_by(.factors.asked_connector_len)|map([.[0].factors.asked_connector_len,length])),
  pairs_with_shared_answer_prefix:(group_by(.pair)|map(select(.[0].expected[0]==.[1].expected[0]))|length),
  unanswerable:($negative[0]|group_by(.kind)|map([.[0].kind,length]))}
