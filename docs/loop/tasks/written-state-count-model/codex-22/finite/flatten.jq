{
  schema: .schema,
  assets: [.fixtures[] | del(.rows)],
  seed_tapes_equal: ([.fixtures[0].rows[].prefix] == [.fixtures[1].rows[].prefix]),
  rows: [
    .fixtures[] | .seed as $seed | .rows | to_entries[] |
    .key as $row | .value |
    if (.candidates[0].supporting_head[1] > .candidates[1].supporting_head[0]) then
      error("fixture assertion support overlaps")
    else
      . + {id: "seed\($seed)-\(.family)-\($row)", seed: $seed, prompt: .text, ids: .prefix} |
      del(.text, .prefix) |
      .candidates |= map(. + {name: .person} | del(.person))
    end
  ]
}
