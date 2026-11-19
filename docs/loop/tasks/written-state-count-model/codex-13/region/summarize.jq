def count(f): map(select(f)) | length;
def stats:
  {rows:length, requested:count(.expected_request[0]!=null), excluded:count(.expected_excluded[0]!=null),
   baseline_requested:count(.baseline_class=="exact-request"), masked_requested:count(.masked_class=="exact-request"),
   baseline_excluded:count(.expected_excluded[0]!=null and .expected_excluded==.baseline_excluded),
   masked_excluded:count(.expected_excluded[0]!=null and .expected_excluded==.masked_excluded),
   no_request:count(.expected_request[0]==null),
   baseline_abstain:count(.baseline_class=="exact-abstention"), masked_abstain:count(.masked_class=="exact-abstention"),
   recovered:count(.baseline_class!="exact-request" and .masked_class=="exact-request"),
   lost:count(.baseline_class=="exact-request" and .masked_class!="exact-request"),
   retained_excluded_lost:count(.retained_excluded and .expected_excluded==.baseline_excluded and .expected_excluded!=.masked_excluded),
   baseline_inversion:count(.baseline_inversion), masked_inversion:count(.masked_inversion),
   named_intrusions:count(.named_intrusion), named_intrusions_recovered:count(.named_intrusion and .masked_class=="exact-request")};
def grouped(f): group_by(f) | map({group: (.[0] | f), stats:stats});
{
  total:stats,
  within_case:grouped(.case),
  within_construction:grouped(.construction),
  within_direction:grouped(.direction),
  within_evidence:grouped(.evidence),
  within_nuisance:grouped(.nuisance),
  baseline_classes:(group_by(.baseline_class)|map({class:.[0].baseline_class,n:length})),
  masked_classes:(group_by(.masked_class)|map({class:.[0].masked_class,n:length})),
  paired_classes:(group_by([.baseline_class,.masked_class])|map({from:.[0].baseline_class,to:.[0].masked_class,n:length})),
  both_directions:(group_by(.pair) | map(select(length == 2 and all(.[]; .expected_request[0] != null))) |
    {pairs:length, original:count(all(.[]; .baseline_class=="exact-request")),
     region:count(all(.[]; .masked_class=="exact-request"))})
}
