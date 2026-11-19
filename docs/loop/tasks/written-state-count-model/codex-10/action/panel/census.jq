. as $all |
($all|map(.prefix)|unique|length) as $unique |
if length!=388 or $unique!=length or any(.[];.kind=="positive" and
 (.asked_start==null or .expected!=(.prefix[.asked_start:.asked_end]+[1])) or .kind!="positive" and .asked_start!=null)
or any([$train_bare[0][], $held_bare[0][]][];keys!=["prefix"])
or ([$train_bare[0][], $held_bare[0][]]|map(.prefix))!=map(.prefix)
then error("action blind/target mismatch") end |
{unique_prefixes:$unique, held_train_prefix_overlap:0,cells:(group_by([(.id|split("-")[0]),.kind])|map({split:(.[0].id|split("-")[0]),kind:.[0].kind,rows:length})),
 source_bearing_positive:map(select(.kind=="positive" and .asked_start!=null))|length}
