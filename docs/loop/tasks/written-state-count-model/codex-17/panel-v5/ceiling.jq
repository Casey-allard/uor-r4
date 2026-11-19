# Pre-inference bounds. No prediction or runtime output is consumed.
def count: length;
def supporting: .facts[.all_requested_facts[0]];
def candidate_by_words($words):
  supporting | .legal[] |
  select(([.entire_copy | scan("[^ ]+")]|length)==$words);
def selected_groups:
  group_by(.nonlexical_observation) |
  map({
    total:length,
    gold_counts:(group_by(.gold_offset)|map({offset:.[0].gold_offset,count:length})),
    best:(group_by(.gold_offset)|map(length)|max)
  });
def sum_best: selected_groups | map(.best) | add;
def legal_count: supporting | .legal | length;
def group_by_size: group_by(legal_count)|map({legal_starts:(.[0]|legal_count),rows:length});
def by_word_and_rank:
  group_by([.gold_offset,.written_value_words]) |
  map({gold_offset:.[0].gold_offset,words:.[0].written_value_words,rows:length});
def lexical_key:
  [.nonlexical_observation,
    (.facts|map(.legal|map({offset,first_word_ids,copy_ids_through_eos})))];
map(select(.stratum=="supported")) as $all |
($all|map(select(.matched_triple!=null))) as $matched |
($all|map(select(.matched_triple==null))) as $off_diagonal |
if any($all[];.gold_candidate_present!=true) then error("gold candidate missing") else . end |
($matched|group_by(.matched_triple)) as $triples |
if ($triples|length)!=8 or any($triples[];
   length!=3 or (map(.gold_offset)|sort)!=[1,2,3] or
   (map(.written_value_words)|sort)!=[1,2,3] or
   (map(.nonlexical_observation)|unique|length)!=1)
then error("broken or vacuous matched triple") else . end |
{
 supported:($all|length),
 reachable_oracle_count:($all|map(select(.gold_candidate_present))|length),
 fixed_last_k:([range(1;4) as $k|
   {k:$k,all:([$all[] as $r|($r|candidate_by_words($k))|select(.start==$r.gold_start)]|length),
    matched:([$matched[] as $r|($r|candidate_by_words($k))|select(.start==$r.gold_start)]|length)}]),
 fixed_offset:([range(1;4) as $o | {offset:$o,all:([$all[]|select(.gold_offset==$o)]|length),
    matched:([$matched[]|select(.gold_offset==$o)]|length)}]),
 any_target_blind_observation_function:{
    all:($all|sum_best),matched:($matched|sum_best),
    groups:($all|selected_groups|length),matched_groups:($matched|selected_groups|length)
 },
 observations_with_exact_candidate_words_and_ids:{
    contradictory_groups:($all|group_by(lexical_key)|map(select((map(.gold_offset)|unique|length)>1))|length)
 },
 matched_triples:($triples | map({
    id:.[0].matched_triple, common_observation:.[0].nonlexical_observation,
    distinct_labels:map({id,gold_offset,written_value_words,gold_start,
      supporting:(supporting|{name,verb_phrase,value,legal})})
 })),
 legal_candidate_counts: {all:($all|group_by_size),matched:($matched|group_by_size),
     coverage_controls:($off_diagonal|group_by_size)},
 written_words_cross_gold_rank:($all|by_word_and_rank),
 negative_rows: (map(select(.stratum!="supported")) | map({
    id,stratum,all_requested_facts,per_fact_legal_starts:(.facts|map(.legal|length))
 })),
 not_run: ["native writer","learned scores","end-to-end public conversation"]
}
