# jq -n --slurpfile fit train.exemplars.jsonl --slurpfile held held.gold.jsonl -f census.jq
def profile: [.descriptor_depth, .word_width, .descriptor_classes];
($fit | group_by(.candidate|profile) | map({
  key: (.[0].candidate|profile|tojson),
  value: {
    positive: map(select(.candidate.positive)) | length,
    negative: map(select(.candidate.positive|not)) | length
  }
}) | from_entries) as $support
| [$held[] | select(.stratum == "matched") | {
    row, family, connector_arm, object_arm,
    candidates: [.candidates[] | . as $candidate
      | ($support[$candidate|profile|tojson] // {positive:0, negative:0}) as $counts
      | {source, offset, legacy, positive, counts:$counts}]
  }] as $matched
| {
  held_rows: ($held|length), matched: ($matched|length), train_candidates: ($fit|length),
  positive_train_candidates: ($fit|map(select(.candidate.positive))|length),
  matched_by_connector: ($matched|group_by(.connector_arm)|map({
    connector_arm: .[0].connector_arm, rows:length,
    gold_with_both_classes: (map(select(any(.candidates[]; .positive and .counts.positive > 0 and .counts.negative > 0)))|length),
    every_candidate_with_both_classes: (map(select(all(.candidates[]; .counts.positive > 0 and .counts.negative > 0)))|length)
  })),
  cases: $matched
}
