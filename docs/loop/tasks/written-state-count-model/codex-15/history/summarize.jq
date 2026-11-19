def n(f): map(select(f))|length;
def tab: {rows:length,w_equals_c:n(.equal),w_ne_c:n(.equal|not),
          both:n(.verdict=="both"),w_only:n(.verdict=="w_only"),
          c_only:n(.verdict=="c_only"),wrong:n(.verdict=="wrong"),
          w_writer_exact:n(.w_writer.exact==true), c_writer_exact:n(.c_writer.exact==true),
          w_writer_recorded:n(.w_writer!=null)};
def matrix: {total:tab,by_reason:[group_by(.reason)[]|{reason:.[0].reason}+tab]};
def with_refs: {split,id,answer,prompt,prefix_sha256,gold_connector_ids,gold_value_ids,reason,w,c,gold,verdict,context,w_writer,c_writer};
def collision($rows; key):
  [$rows|group_by(key)[] |
   select(any(.[];.verdict=="w_only") and any(.[];.verdict=="c_only")) |
   {key:(.[0]|key),retain:n(.verdict=="w_only"),advance:n(.verdict=="c_only"),
    retain_example:(map(select(.verdict=="w_only"))|first|with_refs),
    advance_example:(map(select(.verdict=="c_only"))|first|with_refs)}
  ];
add | sort_by([.seed,.split,.id]) as $all |
{
 archived_source_only:true,
 matrix: [$all|group_by([.seed,.split])[] |
  {seed:.[0].seed,split:.[0].split, all:matrix,
   correctly_anchored:(map(select(.fact_and_query_align))|matrix)}],
 reason2_collisions_by_seed: [$all|group_by(.seed)[] | .[0].seed as $seed |
    map(select(.reason==2 and .fact_and_query_align and .context!=null and .verdict!="wrong")) as $r |
    {seed:$seed,number_of_competing_rows:($r|length),
     offsets_and_raw_masks:collision($r;[.context.offsets,.context.masks]),
     skipped_ids_only:collision($r;.context.skip_ids),
     skipped_and_landing_ids:collision($r;[.context.skip_ids,.context.landed_id]),
     observed_prefix_of_assertion_ids:collision($r;.context.native_prefix_ids),
     complete_assertion_ids:collision($r;.context.assertion_ids),
     full_observed_prefix_digest:collision($r;.prefix_sha256)}
    ],
 reason2_examples_by_seed: [$all|group_by(.seed)[] |
   {seed:.[0].seed,multipart_true_advance:
      (map(select(.fact_and_query_align and .reason==2 and .verdict=="c_only"
                and (.gold_connector_ids|length)>1)) |
       group_by(.gold_connector_ids)|map(first|with_refs)),
    false_prefix_drop:
      (map(select(.fact_and_query_align and .reason==2 and .verdict=="w_only")) |
       group_by(.context.skip_ids)|map(first|with_refs))}]
}
