def pairs: map(map(tostring) | join(":")) | join(" ");
select(.model_schema == "uor-r4.lut-discrete-trunk-s2/2")
| select((.s2_steps | length) > 0 and
         (.s2_steps | length) == (.generated_ids | length) and
         (.s2_steps | length) == (.generated_mass_q32 | length))
| ([.context] + .prompt_ids | map(tostring) | join(" ")),
  (. as $trace | .s2_steps | to_entries[] |
    .key as $i | .value |
    [.order, (.key_hex // "-"), $trace.generated_ids[$i], $trace.generated_mass_q32[$i],
     (.reads | pairs), (.top8 | pairs)] | @tsv)
