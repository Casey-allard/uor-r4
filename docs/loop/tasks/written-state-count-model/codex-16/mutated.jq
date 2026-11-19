map(if .stratum=="supported" then . as $r |
       ([.facts[]|select(.name!=$r.requested_name)]|if length!=1 then error("wrong-entity mutant not unique") else .[0] end) as $other |
       if $other.value==.answer_text then error("mutant literal is unchanged") else .expected=$other.literal_ids_through_eos|.answer_text=$other.value end
     else . end)
