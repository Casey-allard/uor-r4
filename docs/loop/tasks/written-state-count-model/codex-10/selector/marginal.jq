# Compare the addition of the boundary arm when role treatment is already enabled.
$role[0].rows as $r | .rows as $b | $grad[0] as $g |
[range($r|length) as $i |
 if $r[$i].prefix_sha256!=$b[$i].prefix_sha256 then error("row identity mismatch") else
 {changed_before:($r[$i].word_start!=$b[$i].word_start),
  changed_after:($r[$i].continuation.source!=$b[$i].continuation.source),
  role_before_correct:($r[$i].word_start==$g[$i].asked_start),
  both_before_correct:($b[$i].word_start==$g[$i].asked_start),
  role_after_correct:($r[$i].continuation.source==$g[$i].asked_start),
  both_after_correct:($b[$i].continuation.source==$g[$i].asked_start)}
 end] | group_by(.) | map(.[0]+{count:length})
