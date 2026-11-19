# A post-gate intervention on the actual failed oracle edges only.
$grades[0].rows as $g | $labels[0] as $l | $original[0] as $s |
if ($s.arm!="oracle" or ($g|length)!=256 or ($g|map(select(.exact==false))|length)!=24)
then error("wrong frozen cohort") else . end |
$s | .arm=$arm | .origin="oracle-source-intervention" |
.rows = [ .rows[] as $r | ($g[]|select(.index==$r.index and .exact==false)) as $loss |
    $l[$r.index] as $gold | $r.assertions as $sites |
    if (($sites|length)!=1 or $r.prefix_sha256!=$gold.prefix_sha256 or $loss.source!=$r.paired.choice.source)
    then error("unbound failed assertion") else . end |
    $sites[0].assertion as $a |
    (if $arm=="literal-start" then ([$gold.facts[]|select(.name==$gold.requested_name)]|if length!=1 then error("ambiguous literal") else .[0].value_span[0] end)
     elif $arm=="current" then $a.value.current
     elif $arm=="word-start" then $a.value.word_start
     else error("unknown intervention") end) as $src |
    ($r | .paired.choice.source=(if $src==null or $src>=$a.fence or ($a.fence-$src)>7 then null else $src end)
        | .downstream_intervention={arm:$arm,previous_source:$r.paired.choice.source,candidate:$src,fence:$a.fence})
]
