def route($name):
  if $name == "reference" then .reference
  elif $name == "disabled" then .disabled.writer
  elif $name == "parent_supplied" then .parent_supplied.writer
  elif $name == "local" then .local.writer
  elif $name | endswith("_donor") then .views.donor[$name | sub("_donor$"; "")].writer
  else .views[.split][$name].writer
  end;

def identifier($row):
  $row | if .row < 300 then "prior_D25" elif .row < 400 then "prior_D24" else .stratum end;

def tally($name):
  map(. as $r | {row, reading, writer: route($name)})
  | {
    rows: length,
    supported: map(select(.reading == "early" or .reading == "late")) | length,
    exact_source: map(select(.writer.exact_gold_source == true) | .row),
    exact_eos: map(select(.writer.exact_through_eos == true) | .row),
    no_source: map(select(.writer.source_absent == true) | .row),
    unsupported_false_source: map(select(.writer.false_copy == true) | .row),
    verified_literal_refusal: map(select(.writer.literal_refusal == true) | .row),
    first_wrong_token: map(select(.writer.first_mismatch.offset == 0) | .row),
    later_first_error: map(select(.writer.first_mismatch.offset > 0) | {row, mismatch: .writer.first_mismatch})
  };

def pairs($name):
  map(select(.donor_alignment != null))
  | map(select(.row < .donor_alignment.donor) | [.row, .donor_alignment.donor]) as $partners
  | map({key: (.row | tostring), value: {writer: route($name), donor: route($name + "_donor"), aligned: .donor_alignment.relative_local_identical}})
  | from_entries as $rows
  | [$partners[] | map(. as $id | {id: $id} + $rows[tostring]) | {
      rows: map(.id),
      aligned_local: all(.aligned),
      native_exact_sources: map(select(.writer.exact_gold_source) | .id),
      donor_exact_sources: map(select(.donor.exact_gold_source) | .id),
      selected_addresses_changed: map(select(.writer.source != .donor.source) | .id)
    }];

.rows as $rows
| .bindings as $bindings
| {
  bindings: $bindings,
  groups: [
    $rows | group_by(identifier(.))[] | . as $group | identifier(.[0]) as $group_name
    | {
      group: $group_name,
      arms: [("reference", "disabled", "parent_supplied", "local", "post", "relation", "recurrence",
               if $group_name | startswith("prior_") then empty
               else ("post_donor", "relation_donor", "recurrence_donor")
               end) as $arm | {arm: $arm} + ($group | tally($arm))],
      paired_controls: (if $group_name == "primary" or $group_name == "secondary" then
        [("post", "relation", "recurrence") as $view | {view: $view, pairs: ($group | pairs($view))}]
      else [] end)
    }
  ],
  changes_against_post: [
    $rows[] | . as $row | ("relation", "recurrence") as $view
    | .views[.split].post as $post | .views[.split][$view] as $compared
    | select($compared.writer.source != $post.writer.source or $compared.reason != $post.reason)
    | {
      row, group: identifier($row), view: $view, post: $post.writer.source, selected: $compared.writer.source,
      decision: $compared.reason, expected: $compared.writer.gold_source, lost_at: $post.writer.first_mismatch,
      now_wrong_at: $compared.writer.first_mismatch,
      tied_best: (if $compared.reason == "tie" then
        [$compared.margins[] | select(.eligible)] | sort_by(.margin) | group_by(.margin) | last
      else [] end)
    }
  ]
}
