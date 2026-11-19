$r[0].rows as $r | $g[0] as $g | .rows as $w |
if ($r|length)!=($w|length) or ($r|length)!=($g|length) then error("unbound row counts") end |
[range($r|length) as $i | $g[$i] as $k | $r[$i] as $d | $w[$i] as $out |
 if $d.prefix_sha256!=$out.prefix_sha256 then error("different input digests") end |
 if $k.expected!=null then {
   correct_role:([$d.query_start,$d.query_end]==[$k.query_start,$k.query_end]),
   exact:($out.generated==$k.expected),
   error:([range([($out.generated|length),($k.expected|length)]|max)|select($out.generated[.]!=$k.expected[.])]|first)
 } else empty end] as $rows |
{split:$s,seed:$n,arm:$a,denominator:($rows|length),role_exact_reply_exact:($rows|map(select(.correct_role and .exact))|length),
role_exact_reply_wrong:($rows|map(select(.correct_role and (.exact|not)))|length),
role_wrong_reply_exact:($rows|map(select((.correct_role|not) and .exact))|length),
role_wrong_reply_wrong:($rows|map(select((.correct_role|not) and (.exact|not)))|length),
first_wrong:($rows|map(select(.error!=null)|.error|tostring)|group_by(.)|map({key:.[0],value:length})|from_entries)}
