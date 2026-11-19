# Grade-only. Locate the final exact full tokenizer word spelling the requested name.
($tok[0].model.vocab|to_entries|map({key:(.value|tostring),value:.key})|from_entries) as $v |
map(. as $g | if (.kind=="positive" or .kind=="absent") then
  (if .kind=="positive" then .role else .absent_name end) as $name |
  (.prefix|map($v[tostring])) as $pieces |
  ([range($pieces|length) as $i | select($pieces[$i]|test("^Ġ[A-Za-z]+$")) |
    ([range($i+1; $pieces|length) | select($pieces[.]|test("^[A-Za-z]+$")|not)] | first // ($pieces|length)) as $end |
    select(($pieces[$i:$end]|join("")|ltrimstr("Ġ"))==$name) | $i] | last) as $q |
  if $q==null then error("mixed oracle could not bind exact word") else .+{query_start:$q} end
else .+{query_start:null} end)
