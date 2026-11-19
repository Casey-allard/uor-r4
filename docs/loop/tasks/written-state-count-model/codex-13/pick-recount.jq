.records as $r | if ($r|length)!=14287 or ($g|length)!=14287 then error("denominator mismatch") end |
[range(14287) as $i | $g[$i] as $k | $r[$i] as $p |
 if $p.ordinal!=$k.ordinal or $p.winner!=$k.count_winner then error("wrong blind row") end |
 {base:($k.target==$k.reference),count:($k.target==$p.winner),pick:($k.target==$p.pick_successor),
 exact:($k.target==$p.exact_neighbor[1]),rotated:($k.target==$p.shuffled_successor),
 possible_two:($k.target==$p.winner or $k.target==$p.pick_successor)}] as $j |
def tally(f): [$j[]|select(f)]|length;
{dose:$dose,rows:($j|length),incumbent:tally(.base),count:tally(.count),pick:tally(.pick),exact:tally(.exact),
 rotated:tally(.rotated),binary_ceiling:tally(.possible_two),gain:tally(.possible_two and (.base|not)),
 harm:tally(.base and (.possible_two|not))}
