# Intersect matched outcomes by prompt identity. Do not pool seeds into a larger denominator.
def ids($rows): $rows | map([.ordinal,.id,.prompt,.stratum]);
def cells($left; $right; $compare; $endpoint):
  [$left[$compare][] | . as $one |
   $right[$compare][] | select(.ordinal==$one.ordinal and .id==$one.id) |
   {ordinal,id,family,
    seed1:$one[$endpoint],seed2:.[$endpoint]}]
  | group_by([.seed1,.seed2]) | map({seed1:.[0].seed1,seed2:.[0].seed2,rows:length,
    ordinals:map(.ordinal)});
if length!=2 or .[0].seed!=1 or .[1].seed!=2 or ids(.[0].rows)!=ids(.[1].rows)
  then error("compared seed readouts must bind the same prompt identities in order") else . end |
.[0] as $one | .[1] as $two |
{schema:"uor-r4.clause-entity-veto-cross-seed/1",
 denominators_per_seed:$one.totals,
 seed_summaries:map({seed,supported,unsupported,verification,
   role_margins:(.role_margins | group_by([.family,.exclusion_first,.reference,.clause]) |
     map((.[0]|del(.ordinal,.clause_bounds)) + {rows:length}))}),
 matched_prompt_outcomes:{
   role_span:cells($one;$two;"role_pairs";"span"),
   role_exact:cells($one;$two;"role_pairs";"exact"),
   value_source:cells($one;$two;"value_pairs";"source"),
   value_exact:cells($one;$two;"value_pairs";"exact"),
   value_wrong_source:cells($one;$two;"value_pairs";"new_wrong_source")}}
