def row($obj;$arm): $obj.scores[]|select(.arm==$arm);
def difference($higher;$lower):
  if $higher.bytes != $lower.bytes or $higher.targets != $lower.targets then error("denominators differ") else
  ($higher.bits_q12 - $lower.bits_q12) as $d |
  {bits_q12:$d, bpb:($d / 4096 / $higher.bytes)}
  end;
.[0] as $original | .[1].score as $crossed | .[2].score as $restored |
row($original;"full-fit") as $full | row($original;"pruned-fit") as $pruned |
if [$full.weight,$crossed.weight,$restored.weight,$pruned.weight]!=[242,102,102,102] then error("unexpected weight") else . end |
["overall","novel"] | map(. as $key |
  difference($crossed[$key]; $full[$key]) as $w |
  difference($restored[$key]; $crossed[$key]) as $rows |
  difference($pruned[$key]; $restored[$key]) as $rounding |
  difference($pruned[$key]; $full[$key]) as $whole |
  if $w.bits_q12+$rows.bits_q12+$rounding.bits_q12!=$whole.bits_q12 then error("changes do not telescope") else
    {split:$key,targets:$full[$key].targets,bytes:$full[$key].bytes,weight:$w,drop_rows:$rows,round_counts:$rounding,total:$whole}
  end)
