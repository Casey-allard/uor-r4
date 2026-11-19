# Ordered role paired comparison

Build with `cargo build --release --locked --manifest-path Cargo.toml` from this directory. Run the resulting binary once for each seed.

```sh
ordered-role-comparison SEED ARMS_JSON OUT
```

The JSON config paths resolve relative to the config file. Absolute paths also work. The output must be a new file.

```json
{
  "blind": "panel/held-blind.json",
  "labels": "panel/held-grades.json",
  "matching": "panel/held-match.json",
  "public_positional": "grades/public-positional.json",
  "public_identity": "grades/public-identity.json",
  "public_frozen": "grades/public-frozen.json",
  "bounded_positional": "grades/bounded-positional.json",
  "bounded_identity": "grades/bounded-identity.json",
  "bounded_frozen": "grades/bounded-frozen.json",
  "fixed_positional": "fixed/positional.json",
  "fixed_identity": "fixed/identity.json",
  "correct_oracle": "grades/correct-oracle.json",
  "diagnostics": {
    "lexical": "grades/lexical.json",
    "no_read": "grades/no-read.json",
    "wrong": "grades/wrong.json"
  }
}
```

The oracle must cover all 256 supported rows. Below 248 exact EOS replies, the command writes its oracle miss receipt and leaves selector scores unread. Successful file creation indicates the comparison ran to its registered stopping point. Read `disposition` for the numeric decision. Runtime, executable and grades-off parity receipts are preconditions supplied by the collection runner.

The public and bounded arms use 128 full pairs. The nested diagnostic uses the separately sealed matched pairs and verifies its global roles against the graded bounded arms. The full-panel contrasts share one SplitMix64 draw schedule. The matched subset resets the stream and resamples within its own family denominators. Intervals are in whole net pair counts. Divide their endpoints by that comparison's denominator for fractions.

An unsupported copy means a selected source on a row without a supported literal, or a selected source different from the requested person's registered value. The safety gate compares the presence of unsupported copying at the same original row. An exact reply produced without an admitted copy source is identified separately. Public endpoint consistency is measured only among pairs newly exact relative to that comparator. The incidental nomination, counterpart and no-increase checks appear for public and bounded modes. Prose output and its independent relevance criteria remain available for human grading.

`cargo test --locked --manifest-path Cargo.toml` checks the SplitMix64 stream, shared bootstrap draws, the exact 75% endpoint threshold, the row-level unsupported-copy veto and independent direction accounting.
