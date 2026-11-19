[ to_entries[] | select(.value.answer_text!=null) | . as $r |
  ($blind[0][$r.key].text | split(".")[$r.value.gold_assertion_index] |
    [scan("[^[:space:]]+")] | .[-2:] | join(" ")) as $out |
  {id:$r.value.id, expected:$r.value.answer_text, last_two_words:$out, same:($out==$r.value.answer_text)}
] | {given:"supporting sentence index", rows:length, exact_strings:(map(select(.same))|length), details:.}
