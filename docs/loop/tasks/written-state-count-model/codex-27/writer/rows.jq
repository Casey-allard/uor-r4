def headtext: map(map(tostring)|join(".."))|join(",");
def error_text:
  if .exact==null then "unscored"
  elif .exact then "exact through EOS"
  else .first_wrong|[.offset,.expected,.actual]|map(if .==null then "end" else tostring end)|join("/") end;
["parent","row","stratum","tail_instruction","requested_person","gold_fact_heads","unfiltered_heads","selected_heads","unfiltered_source","selected_source","unfiltered_first_error_offset/expected/actual","exact_first_error_offset/expected/actual","geometry_first_error_offset/expected/actual"],
(group_by(.row)[]|map(select(.lane=="ordinary"))|INDEX(.arm)| . as $x |
  select($x.unfiltered and $x.exact and $x.geometry) |
  [$parent,.exact.row,.exact.stratum,.exact.instructed,(.exact.person|map(tostring)|join("..")),
   (.exact.expected_heads|headtext),(.unfiltered.heads|headtext),(.exact.heads|headtext),(.unfiltered.source//"none"),(.exact.source//"none"),
   (.unfiltered|error_text),(.exact|error_text),(.geometry|error_text)])
|@tsv
