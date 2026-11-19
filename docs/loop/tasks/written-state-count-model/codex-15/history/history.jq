# One split and seed. Sources must match the historical seal before invoking.
def endpoints($x): [$x.fact_start,$x.fact_end,$x.query_start,$x.query_end];
def known_arms($name): INDEX($writer[0].rows[] | select(.arm==$name);.case);
def verdict($w;$c;$gold):
  if $w==$gold and $c==$gold then "both"
  elif $w==$gold then "w_only"
  elif $c==$gold then "c_only" else "wrong" end;
def completion($a): {source:.choice.source,exact:.exact,generated:.generated};
. as $continuation | $grade[0] as $labels | $value[0] as $values |
known_arms("WordStartEos") as $writer_w | known_arms("ContinuationEos") as $writer_c |
if ($continuation.rows|length)!=($labels|length)
  or ($values.rows|length)!=($labels|length)
  or ($writer_w|length)!=($labels|length)
  or ($writer_c|length)!=($labels|length)
  or ($values.bare_sha256!=null and $continuation.bare_sha256!=$values.bare_sha256)
then error("row census or prefix tape mismatch") else
[range($labels|length) as $i |
 $labels[$i] as $g | $continuation.rows[$i] as $p | $values.rows[$i] as $v |
 $writer_w[$g.id] as $w_write | $writer_c[$g.id] as $c_write |
 $p.word_start as $w | $p.continuation.source as $c |
 $p.continuation.reason as $reason | $g.asked_start as $gold |
 if $p.frozen != $v.frozen or $p.word_start != $v.word_start
     or $p.prefix_sha256 != $v.prefix_sha256
     or $p.prefix_sha256 != $w_write.prefix_sha256 or $p.prefix_sha256 != $c_write.prefix_sha256
     or $g.token_positions != $w_write.gold_positions or $g.token_positions != $c_write.gold_positions
     or $gold != $w_write.asked_start or $gold != $c_write.asked_start
     or $g.expected != $w_write.expected or $g.expected != $c_write.expected
     or $w != $w_write.choice.source or $c != $c_write.choice.source
     or $p.continuation != $c_write.continuation
     or ($g.expected != ($g.prefix[$gold:$g.asked_end]+[1]))
     or ($reason==0) != ($w==$c)
 then error("inconsistent archived row " + $g.id) else
 {
  split:$split,seed:$seed,id:$g.id,prefix_sha256:$p.prefix_sha256,
  fact_and_query_align:(endpoints($p.frozen)==endpoints($g.token_positions)),
  frozen:$p.frozen,reason:$reason, w:$w,c:$c,gold:$gold,equal:($w==$c),
  verdict:verdict($w;$c;$gold),
  expected:$g.expected, answer:$g.answer_text,prompt:$g.prompt,
  gold_connector_ids:$g.prefix[$g.token_positions.fact_end+1:$gold],
  gold_value_ids:$g.prefix[$gold:$g.asked_end],
  gold_connector_surface:$g.factors.asked_connector_surface,
  context: (if $w==null or $c==null then null else
   {offsets:[$w-$p.frozen.fact_end-1,$c-$p.frozen.fact_end-1],
    masks:[$v.eligible_mask,$v.word_start_mask,$v.ordered_mask,
           $p.continuation.connector_mask,$p.continuation.onset_mask],
    skip_ids:$g.prefix[$w:$c], landed_id:$g.prefix[$c],
    native_prefix_ids:$g.prefix[$p.frozen.fact_end+1:$c+1],
    assertion_ids:$g.prefix[$p.frozen.fact_end+1:
      ([$g.prefix|to_entries[]|select(.key>$p.frozen.fact_end and (.value==16 or .value==1 or .value==0 or .value==201))|.key][0]//($g.prefix|length))] }
   end),
  w_writer:($w_write|completion(.)),c_writer:($c_write|completion(.)),
  bound:"legacy query endpoint before whole-assertion fence"
 }
 end]
end
