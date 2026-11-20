# Reverse requested and excluded labels on just one replicated fixture row.
map(if .row==30100 then .role_targets |=
    (.requested as $r|.requested=.excluded|.excluded=$r) else . end)
