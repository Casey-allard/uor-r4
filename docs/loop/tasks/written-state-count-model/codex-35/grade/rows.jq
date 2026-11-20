# All token offsets are zero-based. Span ends are exclusive. Arm columns are compact JSON.
def cell($a): {source:$a.selected_source,ids:$a.generated,through_eos:$a.exact_eos,first_wrong:$a.first_wrong,release_eos:$a.copy_released_at_eos}|tojson;
(["row","mode","group","stratum","factors","query_key","excluded_key","fact_key","value_key","expected_ids","requested_observed","excluded_observed","gold_fact_visited","assertion_fence","eligible_gold_onset","eligible_full_span","baseline","fact_true","fact_reversed","candidate_true","candidate_reversed"]|@tsv),
(.rows[]|.arms as $a|
 [.row,.mode,.group,.stratum,(.factors|tojson),(.query|tojson),(.excluded|tojson),(.fact|tojson),(.gold_source|tojson),(.expected|tojson),
  ($a.baseline.requested|tojson),($a.baseline.excluded|tojson),$a.baseline.assertion_bound,$a.baseline.fence,$a.baseline.eligible_gold,$a.baseline.eligible_complete,
  cell($a.baseline),cell($a.fact_true),cell($a.fact_reversed),cell($a.candidate_true),cell($a.candidate_reversed)]|@tsv)
