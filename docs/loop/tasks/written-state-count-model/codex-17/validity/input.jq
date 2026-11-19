# Independent eligibility audit. Inputs are immutable captures, their original
# blind tapes, and sealed semantic keys. No bank, predictions or parse-time
# guesses are inputs. The audit output is never an input to a selector.
def must($ok; $message): if $ok then . else error($message) end;
def candidate_shape:
  {original_start,first_word_end,connector_slots,word_slots,slot_classes,writer_reachable};
def row($i; $cap; $key; $blind):
  $cap.rows[$i] as $r | $key[$i] as $g | $blind[$i].prefix as $t |
  must($r.ordinal==$i and ($r.prefix_sha256|type)=="string" and
       $g.prefix_sha256==$r.prefix_sha256 and ($t|length)==$r.original_tape_len;
       "wrong original tape or row index at \($i)") |
  ($g.stratum=="supported") as $s |
  ($r.assertions // []) as $sites |
  (if $s then $g.facts[$g.gold_assertion_index] else null end) as $f |
  $sites[0] as $a |
  [$a.legacy_candidates[]? | select(.writer_reachable==true)] as $reachable |
  ($reachable|map(.original_start)) as $starts |
  # Inspect complete *tapes*, including EOS. These are structural controls;
  # the subsequent device controls independently execute the real cursor.
  [$starts[] as $start | ($t[$start:$a.fence]+[1])] as $writes |
  ($s and ($sites|length)==1 and $a.fact_span==$f.name_span and
   $a.fence==$f.delimiter[0] and $r.requested_span==$g.requested) as $site_ok |
  {ordinal:$i,id:$g.id,family:$g.family,stratum:$g.stratum,group:$g.matched_triple,
   source_in_rank_domain:($site_ok and ($starts|index($g.gold_start))!=null),
   requested_site_and_fence:$site_ok,
   original_literal_reachable:($site_ok and ($g.gold_start|type)=="number" and
     $g.gold_start>=$a.fact_span[1] and $g.gold_start<$a.fence and ($a.fence-$g.gold_start)<=7 and
     (($t[$g.gold_start:$a.fence]+[1])==$g.expected)),
   gold_choice:($starts|index($g.gold_start)),
   reachable_starts:($starts|length),different_writes:($writes|unique|length),
   # Exact nonlexical fields the actual ranker can observe, including order.
   shape:{original_tape_len:$r.original_tape_len,requested_span:$r.requested_span,
      fact_span:$a.fact_span,fence:$a.fence,
      candidates:($a.legacy_candidates // []|map(candidate_shape))}};
def inspect($cap; $key; $blind; $expected; $hash):
  must($cap.schema=="uor-r4.legacy-assertion-candidates/1" and
       $cap.arm=="requested-diagnostic" and ($cap.rows|length)==$expected and
       ($key|length)==$expected and ($blind|length)==$expected and
       ($key|map(.id)|unique|length)==$expected and ($hash|test("^[0-9a-f]{64}$"));
       "bad capture, grade roster or split length") |
  [range($expected) as $i|row($i;$cap;$key;$blind)] as $r |
  ($r|map(select(.stratum=="supported"))) as $pos |
  {rows:$expected,capture_sha256:$hash,supported:($pos|length),
   original_literal_reachable:($pos|map(select(.original_literal_reachable))|length),
   source_in_rank_domain:($pos|map(select(.source_in_rank_domain))|length),
   families:($pos|group_by(.family)|map({family:.[0].family,rows:length,
     in_domain:map(select(.source_in_rank_domain))|length})),
   details:$r};
inspect($fc[0];($fg[0]+$extra[0]);$fb[0];94;$fit_sha) as $fit |
inspect($hc[0];$hg[0];$hb[0];42;$held_sha) as $held |
($held.details|map(select(.group!=null))|group_by(.group)|map(
  {group:.[0].group,rows:length,members:map(.ordinal),
   gold_choices:(map(.gold_choice)|sort),distinct_structures:(map(.shape)|unique|length),
   all_in_domain:all(.[];.source_in_rank_domain and .original_literal_reachable),
   three_real_choices:all(.[];.reachable_starts==3 and .different_writes==3)})) as $groups |
($groups|length==8 and all(.[];
  .rows==3 and .all_in_domain and .three_real_choices and .distinct_structures==1 and
  .gold_choices==[0,1,2])) as $matched_ok |
{schema:"uor-r4.value-rank-input-validity/1",fit:$fit,held:$held,
 matched:{rows:($groups|map(.rows)|add // 0),groups:$groups,
   confirmed_structural_ceiling:(if $matched_ok then 8 else null end)},
 eligible_for_deciding_run:($matched_ok and $fit.supported==88 and $fit.source_in_rank_domain==88 and
     $fit.original_literal_reachable==88 and $held.supported==36 and $held.original_literal_reachable==36)}
