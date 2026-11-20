# [codex] D29 paired analysis

Use the combined 49-prompt panel and the three retained grade directories. The combined panel's files are named `held.*` and contain both the source and role cohorts. The Rust formatter reads the exact fitted and shuffled tables, checks their availability and registered rotation against the original signed TRAIN packet, and joins every trace with the corresponding native input packet.

```sh
cargo run --locked --offline --manifest-path "$D/Cargo.toml" -- \
  "$Q/panel-all" "$Q/grades" "$Q/inputs/train.packet" \
  "$Q/dispatch/train/primary.raw" "$Q/inputs/shuffled.raw" "$Q/paired-readout"
```

The separate jq audit uses each arm's visit ledger to validate its row-level provenance and rederives role, landing, writer-control and paired outcomes. It does not parse the fit or device input packets. Run it on the same combined panel.

```sh
jq -n \
  --slurpfile gold "$Q/panel-all/held.gold.jsonl" \
  --slurpfile blind "$Q/panel-all/held.blind.jsonl" \
  --slurpfile baseline "$Q/grades/baseline/rows.json" \
  --slurpfile baseline_visits "$Q/grades/baseline/visits.json" \
  --slurpfile calibrated "$Q/grades/calibrated/rows.json" \
  --slurpfile calibrated_visits "$Q/grades/calibrated/visits.json" \
  --slurpfile shuffled "$Q/grades/shuffled/rows.json" \
  --slurpfile shuffled_visits "$Q/grades/shuffled/visits.json" \
  -f "$D/metrics.jq" > "$Q/independent-visit-audit.json"
```

The paired report keeps ordinary, supplied-person, supplied-assertion, forced-gold, forced-competing and disabled lanes. Role outcomes preserve both original span endpoints and cross the requested person, fact order and request/exclusion clause order. The structured signed audit distinguishes unsupported independent-family radii, outside-radius negatives, non-strict class comparisons, both classes missing, and released original candidates. It records genuine early gold starts cleared as negative and keeps them separate from the actual source and output.

The two context pairs use identical original suffix IDs and differing earlier words and gold boundaries. Supplied-head traces are compared after shifting original addresses to offsets from the shared local fact head. The D21 historical panel contributes no rows to the D29 paired outputs.
