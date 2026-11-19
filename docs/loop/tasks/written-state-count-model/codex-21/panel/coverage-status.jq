# Join the anonymous D18 shape audit to held families for reporting only.
[$candidates[0] | to_entries[] | .key as $i | .value |
 select(.stratum == "supported") |
 .nonlexical_observation as $obs |
 $obs.facts[.all_requested_facts[0]] as $fact |
 [$fact.starts[] |
  [.offset, (.span[1] - .span[0]), $obs.token_classes[$fact.name_span[1]:.span[1]]]] as $shapes |
 {family: $grades[0][$i].family, gold_offset, shapes: $shapes,
  supported: any($audit[0].after_groups[]; .shapes == $shapes and .all_supported)}]
| group_by([.family, .gold_offset, .supported])
| map(.[0] + {rows: length})
