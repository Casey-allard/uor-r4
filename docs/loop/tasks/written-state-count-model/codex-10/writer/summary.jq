# Input: all one-seed per-arm summaries in filename order. Fail on missing cells.
group_by(.panel)|map(. as $p |
  if ($p|map(.arm)|unique|length)!= (if $p[0].panel=="unanswerable" then 5 else 10 end)
  then error("wrong arm coverage") end |
  if $p[0].panel=="unanswerable" then
   {panel:$p[0].panel,arms:($p|map({key:.arm,value:.classes})|from_entries)}
  else
   {panel:$p[0].panel,arms:($p|map({
     key:.arm,value:{exact:.results.exact,pairs:.both_roles_exact,first_wrong,
       source_right:(.sources.requested//0),source_null:(.sources.null//0),source_other:((.sources.other//0)+(.sources.distractor//0)),
       gained:.results.gained,lost:.results.lost,later:.results.later,earlier:.results.earlier,same:.results.same,movement}})|from_entries)}
  end)
