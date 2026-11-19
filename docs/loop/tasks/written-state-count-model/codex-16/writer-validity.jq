# Direct address oracles go through the historical CUDA trajectory writer. Bind
# real copy and delimiter release events as well as the final graded IDs.
[ $grade[0].rows[] as $r | $labels[0][$r.index] as $l |
  $l.facts[$l.gold_assertion_index] as $f |
  (if $arm=="forced-gold" then $f.value_span[0] else $l.rival_start end) as $src |
  (if $arm=="forced-gold" then $f.literal_ids_through_eos else $f.rival_ids_through_eos end) as $wanted |
  ($f.value_span[1]-$src) as $len |
  ($src-$f.name_span[1]) as $connector |
  ([$trajectory[0].rows[] | select(.prefix_sha256==$l.prefix_sha256 and .source==$src)]|if length==1 then .[0] else error("ambiguous trajectory") end) as $t |
  if $r.generated!=$t.generated or $r.source!=$src then error("wrong source or unbound emissions")
  elif ($len<=7) then
    if ($t.generated!=$wanted or $r.terminal_eos!=true or ($t.steps|length)!=$len+1 or
        any(range(0;$len); . as $i|$t.steps[$i].copied!=1 or $t.steps[$i].before[2]!=$src+$i or $t.steps[$i].source_id!=$wanted[$i])
        or $t.steps[-1].before[2]!=$f.value_span[1] or $t.steps[-1].release!=1 or $t.steps[-1].emitted!=1)
      then error("wrong copied words or delimiter release")
      else {index:$r.index,matched_reach:($connector>=1 and $connector<=3),cursor_reachable:true,emitted_exact_addressed_text:true} end
  else {index:$r.index,matched_reach:false,cursor_reachable:false,emitted_exact_addressed_text:false,first_release:[$t.steps[]|select(.release!=0)|.release][0]} end
] | if length!=24 or (map(select(.matched_reach))|length)!=(if $arm=="forced-gold" then 24 else 18 end)
  then error("registered reach denominator changed") else . end
