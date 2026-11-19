# Counterfactuals share the same carrier and wording; changing which person is requested swaps the value.
def signatures($x): $x|map({id,kind,role,source,expected,emitted:.generated,old_emitted:.old_generated,
  first,old_first,role_first,prompt,old_wrong_id:(if (.old_first|type)=="number" then .old_generated[.old_first] else null end),
  wrong_id:(if (.first|type)=="number" then .generated[.first] else null end),prefix_sha256});
. as $all|[.[]|select(.kind=="unique")]|group_by(.contrast)|map(select(length==2)) as $pairs |
{duplicate_harm:signatures([$all[]|select(.kind=="duplicate_agree" and .old_first=="exact" and .first!="exact")][0:1]),
 gain:signatures(first($pairs[]|select(any(.[];.old_first!="exact" and .first=="exact"))) // []),
 harm:signatures(first($pairs[]|select(any(.[];.old_first=="exact" and .first!="exact"))) // []),
 changed_wrong_id_same_position:signatures(first($pairs[]|select(any(.[];(.first|type)=="number"
  and .first==.old_first and .generated[.first]!=.old_generated[.first]))) // []),
 unchanged_error:signatures(first($pairs[]|select(any(.[];(.first|type)=="number" and .first==.old_first
  and .generated[.first]==.old_generated[.first]))) // [])}
