$g[0] as $g | $old[0].rows as $old | $new[0].rows as $new |
[range($g|length) as $i | $g[$i] as $truth | $old[$i] as $o | $new[$i] as $n |
 if $o.prefix_sha256!=$n.prefix_sha256 or $o.nominee!=$n.nominee or $o.count!=$n.count then error("front-end changed") else
 [$truth.supports[].value_start] as $sources | {id:$truth.id,kind:$truth.kind,complete:$n.complete,count:$n.count,
 agree:($n.disagree|not),query_right:($truth.query_start==$n.nominee),old_source:$o.source,source:$n.source,
 earned:(($truth.expected!=null) and ($n.source!=null) and ($sources|index($n.source)!=null)),
 previous:(($truth.expected!=null) and ($o.source!=null) and ($sources|index($o.source)!=null))} end] |
group_by(.kind)|map({key:.[0].kind,value:{rows:length,query_correct:map(select(.query_right))|length,
 previous:map(select(.previous))|length,correct:map(select(.earned))|length,gained:map(select(.earned and (.previous|not)))|length,
 harmed:map(select(.previous and (.earned|not)))|length,sources:map(select(.source!=null))|length,
 new_claims:map(select(.old_source==null and .source!=null))|length,withdrawn:map(select(.old_source!=null and .source==null))|length,
 incomplete:map(select(.count>.complete))|length,disagree:map(select(.agree|not))|length}})|from_entries
