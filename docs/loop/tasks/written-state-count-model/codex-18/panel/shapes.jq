# Anonymous held candidate signatures for the existing TRAIN coverage audit.
[.[] | select(.stratum == "supported") |
  .nonlexical_observation as $observation |
  $observation.facts[.all_requested_facts[0]] as $fact |
  {matched: (.matched_triple != null),
   shapes: [$fact.starts[] |
     [.offset, (.span[1] - .span[0]), $observation.token_classes[$fact.name_span[1]:.span[1]]]]}
]
| group_by([.matched, .shapes])
| map(.[0] + {rows: length})
