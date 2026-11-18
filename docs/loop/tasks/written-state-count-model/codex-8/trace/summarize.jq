# Aggregate independently reconstructed anchor ranks and the raw emitted IDs.
def n(f): map(select(f))|length;
def counts(f):group_by(f)|map({key:(.[0]|f|tostring),value:length})|from_entries;
def wrong($g;$p):first(range(0;([$g,$p]|map(length)|max))|select($g[.]!=$p[.]))//null;
def augment($g;$w):
  . as $a |
  if any($w[];.prefix_sha256!=$a.prefix_hash or .generated!=[.steps[].emitted]) or
     [$w[0].source,$w[1].source]!=[.old.source,.new.source] then error("writer source or feedback mismatch: "+.id) else . end |
  [$w[]|{gen:.generated,wrong:wrong($g.expected;.generated),hit:(.source==$g.asked_start)}] as [$b,$n,$o] |
  .+{prompt:$g.prompt,target:$g.expected,baseline_writer:$b,repair_writer:$n,oracle_writer:$o,
     move:(if $b.wrong==$n.wrong then "tied" elif $n.wrong==null or ($b.wrong!=null and $n.wrong>$b.wrong) then "later"
           else "earlier" end),candidate_delta:(.asked_pick.cost-.baseline_pick.cost),
     same_selected_pair:(.baseline_pick.q==.repair_pick.q and .baseline_pick.f==.repair_pick.f)};
def example_row: {id,prompt,asked_query,asked_fact,source,old,new,baseline_pick,repair_pick,asked_pick,asked_rank,asked_compatible_rank,target,baseline_writer,repair_writer,oracle_writer};

def score:
 {n:length,movement:counts(.move),old_exact:n(.baseline_writer.wrong==null),new_exact:n(.repair_writer.wrong==null),
  oracle_exact:n(.oracle_writer.wrong==null),new_source:n(.repair_writer.hit),new_wrong_source_exact:n(.repair_writer.wrong==null and (.repair_writer.hit|not)),
  old_loss:counts(.old.first_loss),new_loss:counts(.new.first_loss),old_gate:counts(.old.gate),new_gate:counts(.new.gate),
  anchor_comparison:counts(.anchor_reason),baseline_asked_rank:counts(.asked_rank),repair_asked_rank:counts(.asked_compatible_rank),
  old_occurrences:counts(.old.q_occurrence+"/"+.old.f_occurrence),q_position:counts(.query_position),
  n_asked_pair_available:n(.asked_survived),old_compatible_pair_winner:n(.old.gate!="unequal_whole_words" and .same_selected_pair),
  examples:(group_by(.old.first_loss,.new.first_loss,.new.gate,.move)|map(.[0]|example_row))};
. as $a | $grade[0] as $g | $base[0].rows as $b | $fix[0].rows as $f | $oracle[0].rows as $o |
if any(([$g,$b,$f,$o])[];length!=($a|length)) then error("row coverage mismatch") else . end |
[range(0;length) as $i|.[$i]|augment($g[$i];[$b[$i],$f[$i],$o[$i]])] as $j |
if $format=="all" then $j elif $format=="tsv" then
 (["prefix_sha256","case","old_q","old_f","repair_q","repair_f","asked_q","asked_f","winner_cost","asked_cost","asked_rank","asked_compatible_rank","old_gate","repair_first_loss","repair_gate","baseline_first_wrong","repair_first_wrong","oracle_first_wrong","target","old_generated","repair_generated","oracle_generated"]|@tsv),
 ($j[]|select(.old.source==null)|[.prefix_hash,.id,.baseline_pick.q,.baseline_pick.f,.repair_pick.q,.repair_pick.f,.asked_pick.q,.asked_pick.f,.baseline_pick.cost,.asked_pick.cost,.asked_rank,.asked_compatible_rank,.old.gate,.new.first_loss,.new.gate,(.baseline_writer.wrong//"exact"),(.repair_writer.wrong//"exact"),(.oracle_writer.wrong//"exact"),(.target|tojson),(.baseline_writer.gen|tojson),(.repair_writer.gen|tojson),(.oracle_writer.gen|tojson)]|@tsv)
elif $format=="summary" then
 {n:($j|length),all:($j|score),nulls:($j|map(select(.old.source==null))|score),
 others:($j|map(select(.old.source!=null))|score),
 by_role:($j|group_by(.asked_role)|map({role:.[0].asked_role,rows:length,original_nulls:n(.old.source==null),repair_exact:n(.repair_writer.wrong==null)})),
 by_lexical_set:($j|group_by(.lexical_set)|map({lexical_set:.[0].lexical_set,rows:length,original_nulls:n(.old.source==null),repair_exact:n(.repair_writer.wrong==null)})),
 pairs:($j|group_by(.pair)|if any(.[];length!=2) then error("incomplete pair") else {n:length,baseline:n(all(.baseline_writer.wrong==null)),repair:n(all(.repair_writer.wrong==null)),oracle:n(all(.oracle_writer.wrong==null))} end)}
else error("unknown format") end
