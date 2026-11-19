def n(f): map(select(f))|length;
def head_word($begin): .[$begin:] |
  if length==0 then [] else .[0:([range(1;length) as $i | select(($classes[0][.[$i]])!=2) | $i][0] // length)] end;
def witness($row): $row.context.assertion_ids as $a | $row.context.offsets as [$w,$c] |
  [$a[:$c], ($a|head_word($w)), ($a|head_word($c))];
def root_steps: map([$classes[0][.],$roots[0][.]]);
def screen: {split,id,prompt,verdict,context,parts:.descriptor};
def collision($rows;key):
 [$rows | group_by(key)[] | select(any(.[];.verdict=="w_only") and any(.[];.verdict=="c_only")) |
  {key:(.[0]|key),retain:n(.verdict=="w_only"),advance:n(.verdict=="c_only"),
   retain_example:(map(select(.verdict=="w_only"))|first|screen),
   advance_example:(map(select(.verdict=="c_only"))|first|screen)}];
add | sort_by([.seed,.split,.id]) | map(select(.reason==2 and .fact_and_query_align and .context!=null and .verdict!="wrong")) |
map(. + {descriptor:witness(.)}) as $r |
{
 rows:($r|length),
 observed_geometry_rows:($r|map(select(.descriptor|all(.[];length>0)))|length),
 offsets_masks_and_skipped_word_shape:collision($r;[.context.offsets,.context.masks,(.context.skip_ids|map($classes[0][.]))]),
 skipped_packed_roots_and_classes:collision($r;(.context.skip_ids|root_steps)),
 skipped_plus_landing_packed_roots_and_classes:collision($r;([.context.skip_ids[],.context.landed_id]|root_steps)),
 paired_witness_original_ids:collision($r;.descriptor),
 paired_witness_ordered_packed_roots_and_classes:collision($r;(.descriptor|map(root_steps))),
 assertion_ordered_packed_roots_and_classes:collision($r;(.context.assertion_ids|root_steps)),
 named_examples: [$r[]|select(.prompt=="Joe took pink. Leo also chose yellow. About Leo, which item was listed?"
    or .prompt=="mary kept scarlet whistle. john kept violet pencil. Name mary's item. john stayed nearby."
    or .prompt=="Ada took silver bell. Noah took silver leaf. What was the item that Noah took?") |
    {split,id,prompt,verdict,descriptor, context,
     skipped_classes:(.context.skip_ids|map($classes[0][.])),
     skipped_packed_roots:(.context.skip_ids|map($roots[0][.])),
     packed_root_descriptor:(.descriptor|map(root_steps)),gold_connector_ids,gold_value_ids,w_writer,c_writer}]
}
