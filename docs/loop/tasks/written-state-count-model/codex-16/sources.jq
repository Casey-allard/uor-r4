# The labels file may be passed only to explicit source oracles.
$original[0] as $sel |
(if $arm=="word-start" then
    ([.[]|.rows[]]|group_by(.prefix_sha256)|map({key:.[0].prefix_sha256,value:.})|from_entries)
 else {} end) as $index |
$sel | .arm=$arm | .rows=[.rows[] as $r |
    (if $arm=="word-start" then
        ($index[$r.prefix_sha256] // [] |map(select(.frozen.query_start==$r.paired.role_words[0] and .frozen.query_end+1==$r.paired.role_words[1]))) as $c |
        if ($r.assertions|length)==0 then null
        elif ($r.assertions|length)!=1 or ($c|length)!=1 or $c[0].bounded!=$r.assertions[0].assertion then error("frozen assertion parity")
        else $c[0].bounded as $w | $w.value.word_start as $s | if $s==null or $s>=$w.fence or ($w.fence-$s)>7 then null else $s end end
    elif $arm=="forced-gold" then $labels[0][$r.index].gold_start
    elif $arm=="forced-rival" then $labels[0][$r.index].rival_start
    else error("bad intervention") end) as $source |
    $r | .paired.choice.source=$source | .source_intervention=$arm]
