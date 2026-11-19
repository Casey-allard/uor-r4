def exact($row; $gold): $gold.query_start != null
  and [$row.query_start,$row.query_end] == [$gold.query_start,$gold.query_end];

def classify($row; $gold; $r):
  if exact($row; $gold) then "exact-request"
  elif $gold.query_start == null then
    if $row.query_start == null then "exact-abstention"
    elif $row.query_end <= $r.start then "unsupported-assertion-candidate"
    elif $row.query_start >= $r.end then "unsupported-trailing-candidate"
    else "unsupported-query-candidate" end
  elif $row.query_start == null then
    if [$row.excluded_start,$row.excluded_end] == [$gold.query_start,$gold.query_end] then "requested-word-routed-to-excluded"
    elif $row.requested_score == -2147483648 then "no-request-candidate"
    elif $row.requested_score < 12 then "requested-score-below-threshold"
    else "tied-requested-maxima" end
  elif $row.query_end <= $r.start then "assertion-intrusion"
  elif $row.query_start >= $r.end then "trailing-intrusion"
  elif [$row.query_start,$row.query_end] == [$gold.excluded_start,$gold.excluded_end] then "different-named-entity"
  else "in-query-different-word" end;

$gold[0] as $g | $region[0] as $r | $cohort[0] as $c |
$old[0].rows as $original | $test[0].rows as $masked |
if (($r|length) != ($g|length)) then error("grade/region length mismatch") else . end |
if ([$original,$masked] | any(length != ($g|length))) then error("decision length mismatch") else . end |
[$g | to_entries[] | .key as $i | .value as $row |
  $original[$i] as $o | $masked[$i] as $m | $r[$i] as $region |
  if $row.prefix != $region.prefix or $o.prefix_sha256 != $m.prefix_sha256 then error("row alignment mismatch") else . end |
  {id:$row.id, pair:$row.pair, construction:$row.construction, case:$row.case,
   direction:(if $row.kind=="ordinary" then null else ($row.nuisance_pair // $row.id | split("-")[-1] | tonumber) end),
   kind:$row.kind, evidence:$row.evidence, nuisance:$row.nuisance, suffix_pair:$row.nuisance_pair,
   expected_request:[$row.query_start,$row.query_end], expected_excluded:[$row.excluded_start,$row.excluded_end],
   region:[$region.start,$region.end], prefix_sha256:$o.prefix_sha256,
   baseline_words:$o.words, masked_words:$m.words,
   baseline_request:[$o.query_start,$o.query_end], masked_request:[$m.query_start,$m.query_end],
   baseline_excluded:[$o.excluded_start,$o.excluded_end], masked_excluded:[$m.excluded_start,$m.excluded_end],
   baseline_class:classify($o; $row; $region), masked_class:classify($m; $row; $region),
   baseline_inversion:([$o.excluded_start,$o.excluded_end] == [$row.query_start,$row.query_end] and $row.query_start != null),
   masked_inversion:([$m.excluded_start,$m.excluded_end] == [$row.query_start,$row.query_end] and $row.query_start != null),
   named_intrusion:any($c[]; .row == $i and .id == $row.id and .prefix_sha256 == $o.prefix_sha256),
   retained_request:($row.query_start != null and $row.query_start >= $region.start and $row.query_end <= $region.end),
   retained_excluded:($row.excluded_start != null and $row.excluded_start >= $region.start and $row.excluded_end <= $region.end)}]
