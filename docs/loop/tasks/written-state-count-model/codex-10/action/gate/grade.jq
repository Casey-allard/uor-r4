# Join frozen blind action words to the matched mixed-family grade only after sealing.
.rows as $r | $grade[0] as $g |
if ($r|length)!=($g|length) then error("wrong panel size") else
 [range($r|length) as $i | $r[$i] as $a | $g[$i] as $label |
  if ($a.allowed and ($a.source==null or $a.source!=$a.candidate)) or
     (($a.allowed|not) and $a.source!=null) then error("invalid gated source") else
  {kind:$label.kind,allowed:$a.allowed,candidate:$a.candidate,
   correct_candidate:($label.kind=="positive" and $a.candidate==$label.asked_start),
   correct_address:($label.kind=="positive" and $a.source==$label.asked_start),
   tie:($a.positive_distance==$a.negative_distance and $a.nominee!=4294967295),
   nomination:$a.nominee,matches:$a.earlier_matches,agreement:$a.same_query} end] |
 group_by(.kind) | map({kind:.[0].kind,rows:length,allowed:(map(select(.allowed))|length),
    withheld:(map(select(.allowed|not))|length),correct_candidate:(map(select(.correct_candidate))|length),
    correct_address:(map(select(.correct_address))|length),ties:(map(select(.tie))|length),
    already_null:(map(select(.candidate==null))|length),evidence:
      (group_by([.matches,.agreement,.allowed])|map({matches:.[0].matches,agree:.[0].agreement,copy:.[0].allowed,count:length}))})
end
