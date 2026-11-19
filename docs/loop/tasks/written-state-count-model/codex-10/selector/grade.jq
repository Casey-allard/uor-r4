# All device selections and their complete blind overrides must be sealed before joining gold.
def occurrence($r;$g):
  [$r.frozen.query_start, $r.frozen.query_end, $r.frozen.fact_start, $r.frozen.fact_end]
  == [$g.token_positions.query_start, $g.token_positions.query_end,
      $g.token_positions.fact_start, $g.token_positions.fact_end];
def row($r;$g):
  {role:occurrence($r;$g),value:($r.continuation.source==$g.asked_start),
   start:$r.word_start,final:$r.continuation.source,overrode:($r.word_start!=$r.continuation.source)};
def add_count(f): map(select(f))|length;
def summary:
 {n:length,correct_pair:add_count(.arm.role),correct_source:add_count(.arm.value),
  correct_pair_and_source:add_count(.arm.role and .arm.value),
  pair_gained:add_count(.arm.role and (.base.role|not)),pair_lost:add_count((.arm.role|not) and .base.role),
  source_gained:add_count(.arm.value and (.base.value|not)),source_lost:add_count((.arm.value|not) and .base.value),
  precontinuation_start_changes:add_count(.arm.start!=.base.start),
  postcontinuation_source_changes:add_count(.arm.final!=.base.final),
  continuation_advances:add_count(.arm.overrode)};
$baseline[0].rows as $b | $grad[0] as $g | .rows as $r |
if ($b|length)!=($g|length) or ($r|length)!=($g|length) then error("incomplete joined rows") else
  [range(0;($g|length)) as $i |
    if $b[$i].prefix_sha256 != $r[$i].prefix_sha256 then error("unequal prefixes") else
      {base:row($b[$i];$g[$i]),arm:row($r[$i];$g[$i])} end] | summary
end
