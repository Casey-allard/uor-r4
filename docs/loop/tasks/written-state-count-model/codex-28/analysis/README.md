# D28 paired result reducer

Build and run the isolated Rust package with the checked lockfile.

```sh
cargo build --locked --offline --release --manifest-path "$ANALYSIS/Cargo.toml"
d28-onset-analysis "$PANEL" "$GRADE_ROOT" "$NEW_REPORT_JSON"
```

The panel directory supplies the sealed raw prompts, original IDs, requested
spans and gold key. The grader root supplies `rows.json` and `visits.json`
in `frozen-legacy`, `frozen-whole`, `fitted-legacy`, `fitted-whole`,
`augmented-legacy` and `augmented-whole`. Run the reducer after verifying
the enclosing panel and run manifests. All six arms must contain the same
row and lane keys. The reducer checks forced-gold and opposing lanes against
the original source IDs and EOS and verifies each reported first mismatch
independently. It checks the raw candidate coordinates, signed score ratios,
exact winner or stop status, and the one-to-one assertion visit ledger.

The report includes paired gain and loss rows by arm for ordinary, supplied
person and supplied assertion runs. Within each arm it also compares those
lanes, forced gold and the disabled writer. Groups preserve supported and
unsupported strata, lexical family, early versus late connector, and family
by connector. Unsupported rows have no literal gold and are tracked for
source release. Per-row records include the original prompt, exact tapes,
first mismatch, and the candidate evidence for gold ineligibility, absent
classes, radius rejection, unknown-rival veto, nonpositive scores, and
premature or later onset. Repeated cells within a lexical family are paired
diagnostics and do not count as independent transfer trials.

The planted test checks a paired gain across the six-arm roster. It also
requires refusal after changing a first mismatch and after misindexing a
native assertion visit.
