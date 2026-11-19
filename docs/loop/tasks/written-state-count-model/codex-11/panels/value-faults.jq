. as $r|[range(length) as $i|$r[$i] as $w|$grades[0][$i] as $g|$chain[0].rows[$i] as $c|
 select($w.kind=="unique" and $w.first!="exact")|
 if $c.prefix_sha256!=$w.prefix_sha256 then error("unaligned query oracle") end |
 {id:$w.id,prompt:$w.prompt,expected:$w.expected,generated:$w.generated,first:$w.first,
  gold_query:[$g.query_start,$g.query_end],gold_fact:[$g.supports[0].fact_start,$g.supports[0].fact_end],
  gold_value:[$g.asked_start,$g.asked_end],chosen:$c.frozen,
  value:$c.value,word_start:$c.word_start,continuation:$c.continuation,writer_source:$w.source,
  expected_token:$w.expected[$w.first],emitted_token:$w.generated[$w.first]}]
