# Only FIT labels and anonymous held shapes. This tool does not access roots or a scorer.
def fit_records($c;$labels):
  [$c|to_entries[]|.key as $i|.value as $r|$labels[$i] as $g |
   if $r.id!=$g.id or $r.stratum!=$g.stratum then error("misbound FIT row") else . end |
   select($g.stratum=="supported") |
   if ($r.all_requested_facts|length)!=1 then error("FIT row lacks one supporting fact") else . end |
   $r.nonlexical_observation as $o |
   $o.facts[$r.all_requested_facts[0]] as $fact |
   $fact.starts[] |
   {shape:[.offset,(.span[1]-.span[0]),$o.token_classes[$fact.name_span[1]:.span[1]]],
    family:$g.family,is_onset:(.start==$g.gold_start)}];
def grouped:
  group_by(.shape)|map({shape:.[0].shape,records:length,
    onset_families:[.[]|select(.is_onset)|.family]|unique,
    rival_families:[.[]|select(.is_onset|not)|.family]|unique,
    onset_rows:[.[]|select(.is_onset)]|length,
    rival_rows:[.[]|select(.is_onset|not)]|length});
def support($records;$shape):
  any($records[];.shape==$shape and (.onset_families|length)>=2 and .rival_rows>0);
def check_groups($groups;$records):
  [$groups[]|. as $g |
   {matched,rows,shapes,all_supported:all($g.shapes[];support($records;.)),
    unsupported:[.shapes[]|select(support($records;.)|not)]}];
def summary($groups):
 {all:([$groups[].rows]|add),matched:([$groups[]|select(.matched)|.rows]|add),
  all_testable:(([$groups[]|select(.all_supported)|.rows]|add)//0),
  matched_testable:(([$groups[]|select(.matched and .all_supported)|.rows]|add)//0)};
fit_records($base_c[0];$base_g[0]) as $base |
fit_records($supp_c[0];$supp_g[0]) as $supp |
($base|grouped) as $before |
($supp|grouped) as $added |
($base+$supp|grouped) as $after |
($held[0]|check_groups(.;$before)) as $old |
($held[0]|check_groups(.;$after)) as $new |
{
 capacity:256,
 raw_fit_slots:{base:($base|length),supplement:($supp|length),total:(($base+$supp)|length),
   reserved_padding:(256-($base|length)-($supp|length))},
 generic_shapes_supplement:$added,
 combined_shapes:$after,
 before:summary($old),after:summary($new),
 before_groups:$old,after_groups:$new,
 unknown_gate:"no geometry distance, radius membership, exact geometry-collision, nomination, fit, or writer check"
}
