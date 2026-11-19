map({panel,arm,rows:.overall.rows,references:.overall.scored,exact:.overall.exact_eos,
   by_kind:(.by_kind|map_values({rows,exact:.exact_eos,issued,unexpected_copy})),fixed_fact_role_pairs,
   movement_from_ungated:.overall.role_to_arm,movement_from_gated:.overall.old_to_arm,
   transitions}) | sort_by(.panel,.arm)
