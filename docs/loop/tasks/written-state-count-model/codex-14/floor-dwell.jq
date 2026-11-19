# Count a lane only in a bucket with fitted targets and a positive starting weight.
def dwell($name):
  .[$name] as $fit
  | [$fit.buckets[] | select(.targets > 0) as $b
     | range(1; ($b.start_q32 | length)) as $e
     | select($b.start_q32[$e] == 1)
     | {bucket: $b.bucket, lane: $e, targets: $b.targets, dwell: $b.floor_dwell[$e],
        ended_above_floor: ($b.weights_q32[$e] > 1)}] as $lanes
  | {
      endpoint: $fit.endpoint,
      observed_bucket_lanes: ($lanes | length),
      ever_rose: ([$lanes[] | select(.dwell < $fit.endpoint)] | length),
      ended_above_floor: ([$lanes[] | select(.ended_above_floor)] | length),
      always_at_one: ([$lanes[] | select(.dwell == $fit.endpoint)] | length),
      targets_in_buckets_with_a_rise:
        ([$lanes[] | select(.dwell < $fit.endpoint)] | unique_by(.bucket) | map(.targets) | add // 0)
    };
{all_experts: dwell("u_count"), restricted: dwell("kn_u")}
