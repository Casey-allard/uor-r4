# claude-342: classify a served-reply-panel score by how its replies end.
#
# usage: jq -f loop-class.jq score.json
#
# A reply "loops" when its last 24 token ids repeat with a period of 1 to 8, the smallest such
# period being its period. Its onset is the first reply position from which that period holds to
# the end. A reply shorter than 24 ids does not count as looping.

def period($ids):
  ($ids[-24:]) as $t
  | if ($t | length) < 24 then null
    else first(
      range(1; 9) as $p
      | select([range(0; 24 - $p)] | all(. as $i | $t[$i] == $t[$i + $p]))
      | $p
    ) // null
    end;

def onset($ids; $p):
  ($ids | length) as $n
  | first(
      range(0; $n - $p) as $k
      | select([range($k; $n - $p)] | all(. as $i | $ids[$i] == $ids[$i + $p]))
      | $k
    ) // ($n - $p);

[.rows[] | .reply_ids as $ids | period($ids) as $p
  | {stop, p: $p, onset: (if $p then onset($ids; $p) else null end)}] as $r
| ($r | map(select(.p != null))) as $l
| {
    rows: ($r | length),
    eos: ($r | map(select(.stop == "eos")) | length),
    looping: ($l | length),
    period_1: ($l | map(select(.p == 1)) | length),
    period_2: ($l | map(select(.p == 2)) | length),
    period_3_to_8: ($l | map(select(.p >= 3)) | length),
    median_onset: (if ($l | length) > 0 then ($l | map(.onset) | sort | .[length / 2 | floor]) else null end),
    onset_at_most_5: ($l | map(select(.onset <= 5)) | length),
    onset_max: ($l | map(.onset) | max),
    no_stop: ($r | map(select(.stop != "eos")) | length),
    looping_that_stopped: ($l | map(select(.stop == "eos")) | length)
  }
