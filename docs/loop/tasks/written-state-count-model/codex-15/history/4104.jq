# Correct-role oracle archive plus separately sealed original held labels.
def classification($w;$c;$gold):
 if $w==$gold and $c==$gold then "both" elif $w==$gold then "w_only"
 elif $c==$gold then "c_only" else "wrong" end;
if length!=3 then error("expected grades, selections, and labels") else . end |
.[0].rows as $grades | .[1].rows as $selections | .[2] as $labels |
if ($grades|length)!=256 or ($selections|length)!=256 then error("wrong oracle denominator") else . end |
[range($grades|length) as $i |
 $grades[$i] as $g | $selections[$i] as $s | $labels[$g.index] as $l |
 if $g.id!=$l.id or $g.index!=$s.index or $s.prefix_sha256!=$l.prefix_sha256
   or $g.expected!=$l.expected or $g.requested!=$l.requested or $g.exact!=($g.generated==$g.expected)
 then error("unbound oracle row") else . end |
 select($g.exact==false) |
 ([$l.facts[]|select(.name==$l.requested_name)] |
  if length!=1 then error("ambiguous labelled fact") else .[0] end) as $f |
 ($s.assertions|if length!=1 then error("unexpected support cardinality") else .[0] end) as $witness |
 $witness.assertion as $a | $a.value.word_start as $w | $a.source as $c |
 $f.value_span[0] as $gold | $f.name_span[1] as $start | $l.prefix_sha256 as $digest |
 if $g.requested!=$l.requested or $f.name_span!=$witness.fact_span or $a.source!=$g.source
   or $a.continuation.source!=$a.source or $g.first_wrong!=0
 then error("exposed oracle mismatch not localized to reported assertion") else . end |
 ($blind[0][$g.index].prefix) as $ids |
 if $g.expected!=($ids[$gold:$f.value_span[1]]+[1]) then error("oracle literal IDs differ") else . end |
 {
  split:"4104-exposed-24",seed:$seed,id:$g.id,prefix_sha256:$digest,
  fact_and_query_align:true,frozen:{fact_end:($start-1),query_start:$g.requested[0]},
  reason:$a.continuation.reason,w:$w,c:$c,gold:$gold,equal:($w==$c),
  verdict:classification($w;$c;$gold),expected:$g.expected,answer:$l.answer_text,
  prompt:$blind[0][$g.index].text,gold_connector_ids:$ids[$start:$gold],
  gold_value_ids:$ids[$gold:$f.value_span[1]],gold_connector_surface:null,
  context:{offsets:[$w-$start,$c-$start],
    masks:[$a.value.eligible_mask,$a.value.word_start_mask,$a.value.ordered_mask,
           $a.continuation.connector_mask,$a.continuation.onset_mask],
    skip_ids:$ids[$w:$c],landed_id:$ids[$c],native_prefix_ids:$ids[$start:$c+1],
    assertion_ids:$ids[$start:$a.fence]},
  w_writer:null,c_writer:{source:$g.source,exact:$g.exact,generated:$g.generated},
  bound:"first frozen assertion delimiter"
 }]
