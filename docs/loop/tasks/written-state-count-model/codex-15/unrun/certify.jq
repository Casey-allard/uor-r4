# Bind every device candidate and every ordered-or-identity lattice coordinate
# to its original input positions in the reloaded, independently executed client.
$device[0] as $d | $served[0] as $s | $packed[0] as $p | $blind[0] as $b |
if ($d.bare_sha256 != $s.blind or $d.bounds_sha256 != $s.bounds
   or $s.artifact != $p.artifact or $d.bank_sha256 != $p.bank
   or $d.bindings[4] != $p.train_receipt or $p.bank_bytes != 400
   or ($b | length) != ($s.rows | length) or ($s.rows | length) != ($d.traces | length))
then error("unbound device capture or serving artifact") else . end |
[range(0; $b | length) as $i |
  $d.traces[$i] as $g | $s.rows[$i] as $h | $b[$i].prefix as $ids |
  if ($g.row != $i or $h.index != $i or $g.region != $h.region
      or $g.prefix_sha256 != $h.prefix_sha256 or $g.decision.words != $h.paired.role_words
      or ($g.candidates|length) != ($h.candidates|length)) then error("role row parity at \($i)") else . end |
  [range(0; $g.candidates|length) as $j | $g.candidates[$j] as $a | $h.candidates[$j] as $z |
    (if $z.positions==null then null else [$z.positions[] | if .==null then null else $ids[.] end] end) as $slot_ids |
    if ($a.span!=$z.span or $a.admitted!=$z.admitted or $a.ids!=$slot_ids or
        $a.feature!=$z.feature or $a.score!=$z.score) then error("candidate parity row \($i) candidate \($j)") else 1 end] | add // 0
] | {rows:length,candidates:add,bank:$p.bank,artifact:$p.artifact,operation:$d.operation}
