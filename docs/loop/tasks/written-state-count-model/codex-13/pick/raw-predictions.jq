# Join the already captured and graded immutable rows by ordinal.
# Usage: jq -n --slurpfile blind ARM.blind.json --slurpfile grades ARM.grades.jsonl -f this_file
def score:
  {rows:length, eligible:([.[] | select(.candidate != null)] | length),
   correct:([.[] | select(.target == .candidate)] | length)};
def count_or_pick: .target == .count or .target == .pick;
def binary_switch_bound:
  {correct: (map(select(count_or_pick)) | length),
   baseline_correct: (map(select(.target == .baseline)) | length),
   gains: (map(select(count_or_pick and .target != .baseline)) | length),
   harms: (map(select((count_or_pick | not) and .target == .baseline)) | length)}
  | (4 * (.gains + .harms)) as $discordance
  | . + {net:(.gains - .harms), paired_band:(0 | until(. * . >= $discordance; . + 1))};
def paired:
  {rows:length,
   pick: (map({target,candidate:.pick}) | score),
   exact: (map({target,candidate:.exact}) | score),
   rotated: (map({target,candidate:.rotated}) | score),
   count: (map({target,candidate:.count}) | score),
   oracle_binary_switch: binary_switch_bound,
   pick_exact_same_id: ([.[] | select(.exact != null and .pick == .exact)] | length),
   pick_exact_both_correct: ([.[] | select(.target == .pick and .target == .exact)] | length),
   pick_correct_exact_wrong: ([.[] | select(.target == .pick and .target != .exact)] | length),
   exact_correct_pick_wrong: ([.[] | select(.target == .exact and .target != .pick)] | length),
   pick_correct_s2_absent: ([.[] | select(.target == .pick and (.s2 | not))] | length),
   pick_correct_at_positive_distance: ([.[] | select(.target == .pick and .distance > 0)] | length),
   pick_correct_and_count_correct: ([.[] | select(.target == .pick and .target == .count)] | length)};

($blind[0].records) as $records
| if ($records|length) != ($grades|length) then error("different row counts") else . end
| [
    range(0; $records|length) as $i
    | $records[$i] as $r
    | $grades[$i] as $g
    | if $r.ordinal != $i or $g.ordinal != $i or $r.winner != $g.count_winner
      then error("row identity mismatch")
      else {target:$g.target, pick:$r.pick_successor, exact:($r.exact_neighbor[1] // null),
            rotated:$r.shuffled_successor, count:$r.winner, baseline:$g.reference,
            distance:$r.pick_distance,
            response_pos:$g.response_pos, s2:$g.support_arms[1]}
      end
  ]
| {all:paired, early:(map(select(.response_pos <= 2)) | paired),
   eos:(map(select(.target == 1)) | paired),
   distance_zero:(map(select(.distance == 0)) | paired),
   distance_positive:(map(select(.distance > 0)) | paired)}
