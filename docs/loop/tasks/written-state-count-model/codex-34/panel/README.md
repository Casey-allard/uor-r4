# D34 detached boundary panel

The deciding comparison replays D32's 22 exposed D30 user prompts. The panel
includes 16 crossed requests, two direct requests, two unspecified controls and
two incidental-only controls. D32 diagnosed the detached-blank failure on these
same examples. D34 uses the frozen D26 state and weight tables with no fit in
this unit. The earlier model training boundary is not established by this
panel. The preserved D30 held panel is outside this experiment and its rows
are never loaded. The synthetic finite rows have separately planted expected
words and are mechanism tests.

## Input custody

On Wick the existing sealed D30 input root is
`/home/callard/uor-codex/20261120-context-swap-30/custody-v1/raw`.
Inputs to the ordinary and device runners consist of `panel/role.blind.tsv`
and the frozen D26 model and tokenizer. The TSV contains only numeric row ID
and original user text. The gold, `panel/role.gold.json`, remains on Wick and
enters the scoring command only after the primary and cold capture roots are
sealed. The D32 receipts under
`/home/callard/uor-codex/20261120-d32-role-grade/ordinary-primary/full-repeated`
provide the baseline tape, role audit, selected source and writer through EOS.

| Bound hash | SHA-256 |
|---|---|
| Blind text TSV | `176082f66f2e73ba4080567fb2b5e77cc7c4478c14da939b47fad72fa2660a30` |
| Offline original-ID gold | `220ec3c5fd0c839bed864c051458b30f95668cc1b7a26c725d646d189fdc8096` |
| D26 recurrence artifact | `b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4` |
| D26 tokenizer | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |

## Finite expectations before natural replay

- A tokenizer-bound singleton blank followed by a complete lexical run must
  retain both original half-open endpoints, including the blank. A matching
  prior factual head with the same IDs must survive as the witness.
- A longer same-prefix lexical run must not bind the shorter span, regardless
  of which run occurs first. A matching suffix within a longer lexical run
  must not create a new start or match a complete earlier head.
- Punctuation bounds a complete lexical run. The punctuation ID stays outside
  the selected entity span. An incidental mention can remain visible in the raw
  candidate trace and may not manufacture an assertion witness. A repeated
  unsupported mention needs no requested admission.
- A boundary in the user frame cannot be escaped by consuming a lexical
  continuation, a blank, a delimiter or a witness from beyond that frame.
- Consecutive singleton blanks retain only the blank adjacent to the lexical
  run. A double-space token is a different original ID and is not the
  single-space class.
- For duplicate complete lexical tapes at different mention positions, retain
  independently indexed occurrences, score the requested and excluded contexts
  independently, and retain the earliest exact prior structural witness. A tie
  cannot be interpreted as proof of a semantic role.

Use the independent Rust regression fixture and CUDA fixture on the same
boundary cases. Intermediate endpoints and the four score words matter even
when another stage blocks the final reply.

## Replay and join

After top-level GO, use the staged exact Wick executable and the two-arm
ordinary runner. Protocol 2, temperature 0, read enabled and 16 maximum output
IDs are frozen. The runner checks the carrier, tokenizer, blind text and
authorized executable digests. Use fresh directories for primary and cold.

```sh
bash run.sh "$chat" "$model" "$tokenizer" "$blind" "$go" "$ordinary_primary"
bash run.sh "$chat" "$model" "$tokenizer" "$blind" "$go" "$ordinary_cold"
cargo build --release --locked --offline --manifest-path Cargo.toml
target/release/d34-role-panel "$gold" "$archive_full_repeated" \
  "$ordinary_primary" "$ordinary_cold" "$packets" "$cuda_primary" "$cuda_cold" \
  > "$join"
```

The join requires the original frame bounds and prompt IDs, the D32 mode 7
baseline role audit, selection, policy, connector and cursor words, and two
byte-identical ordinary captures. Both GPU launches must agree with the
ordinary selector on all eight decision words and all position records for
detached candidates and prior factual witnesses. Endpoint presence, admission
and assignment are scored independently. The offline result reports the
expected assertion, source selection and actual writer steps. Supported
replies require exact original IDs through EOS. Unsupported controls are scored
for factual source selection. First errors retain the original expected and
actual IDs and zero-based output position. The summary presents both arms over
the same strata and the total panel.
