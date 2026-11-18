# Input: one #4094 rank array. --rawfile writer binds its matching rollout TSV.
def counts(f):
  group_by(f) | map({key: (.[0] | f | tostring), value: length}) | from_entries;

def writer_rows:
  split("\n") | map(select(length > 0) | split("\t")) |
  .[0] as $header |
  .[1:] | map(
    if length != ($header | length) then error("truncated writer row") end |
    [$header, .] | transpose | map({key: .[0], value: .[1]}) | from_entries
  );

def mismatch($got; $want):
  first(range(0; ([$got, $want] | map(length) | max)) | select($got[.] != $want[.]))
  // "exact" | tostring;

def joined($r; $w):
  ($w.reference | fromjson) as $want |
  mismatch(($w.repair_emissions | fromjson); $want) as $repair |
  mismatch(($w.oracle_emissions | fromjson); $want) as $oracle |
  if $w.case != $r.id or $w.prefix_sha256 != $r.prefix_hash
     or ($w.requested_address | tonumber) != $r.source
     or ($w.repair_address | tonumber) != $r.new.source
     or ($w.frozen_null | fromjson) != ($r.old.source == null)
     or $repair != $w.repair_first_wrong or $oracle != $w.oracle_first_wrong
     or ($repair == "exact") != $r.new.hit or $want[-1] != 1
  then error("unmatched source or emission on " + $r.id) end |
  $r + {repair: $repair, oracle: $oracle};

def outcomes:
  {rows: length, first_wrong: counts(.repair),
   wrong_anchor_miss: (map(select(.new.first_loss == "joint_anchor_ranking" and (.new.hit | not))) | length),
   wrong_anchor_exact: (map(select(.new.first_loss == "joint_anchor_ranking" and .new.hit)) | length),
   correct_anchor_wrong_value: (map(select(.new.first_loss == "value_address")) | length)};

. as $ranks | ($writer | writer_rows) as $rows |
if length != 1728 or length != ($rows | length) or (map(.id) | unique | length) != length
then error("wrong panel size or duplicate case ID") end |
[range(length) as $i | joined($ranks[$i]; $rows[$i])] |
{
  all: outcomes,
  baseline_null: (map(select(.old.source == null)) | outcomes),
  baseline_nonnull: (map(select(.old.source != null)) | outcomes),
  wrong_anchor: (map(select(.new.first_loss == "joint_anchor_ranking")) | {
    selections: counts([.new.q_occurrence, .new.f_occurrence, .repair] | join("/")),
    energy_vs_requested: counts([(if .repair_pick.cost == .asked_pick.cost then "tie"
                                  elif .repair_pick.cost < .asked_pick.cost then "strictly_lower"
                                  else error("winner cost exceeds requested") end), .repair] | join("/")),
    requested_eligible_rank: counts(.asked_compatible_rank)
  }),
  wrong_value_offsets: (map(select(.new.first_loss == "value_address"))
                        | counts([.connector, (.new.source - .source)] | join("/"))),
  repair_to_anchor_oracle: counts([.repair, .oracle] | join("->"))
}
