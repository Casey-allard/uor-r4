# Static membership only. No root distance, prediction, fitted bank or held answer is read.
def starts($c):
  $c.nonlexical_observation as $o |
  select(($o.same_name_fact_indices|length)==1) |
  $o.facts[$o.same_name_fact_indices[0]] as $f |
  $f.starts[] as $a |
  {start:$a.start, shape:([$a.offset,($a.span[1]-$a.span[0]),$o.token_classes[$f.name_span[1]:$a.span[1]]]|tojson)};
. as $held | [$fitc[0]|to_entries[] | .key as $i | .value as $r | $fitg[0][$i] as $g |
  select($g.stratum=="supported" and $r.id==$g.id) | starts($r) |
  . + {family:$g.family,onset:(.start==$g.gold_start)}]
| group_by(.shape) | map({key:.[0].shape,value:{onset_families:([.[]|select(.onset)|.family]|unique),
   non_onsets:([.[]|select(.onset|not)]|length),rows:length}}) | from_entries as $s |
[$held[] | select(.stratum=="supported") | . as $r | [starts($r)|.shape as $k|{shape:$k,
 eligible:(($s[$k].onset_families|length)>=2 and ($s[$k].non_onsets//0)>0)}] as $a |
 {id:$r.id,candidates:($a|length),all_shapes_represented:(($a|length)>0 and all($a[];.eligible))}] as $rows |
 {training_shapes:$s, supported:($rows|length),all_shapes_represented:([$rows[]|select(.all_shapes_represented)]|length),rows:$rows}
