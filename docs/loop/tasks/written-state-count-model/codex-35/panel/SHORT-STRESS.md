# D35 matched short-object writer stress

The primary 246-row panel remains fixed. An independent tokenizer-only census
found that 24 of 216 crossed primary replies fit the seven-token copy limit,
including EOS as a separate eighth token. Replace only object wording with
shorter plain item terms in the nine existing crossed scenario families. Keep
the exact 144 original TRAIN prompts, labels and admitted device records.
The 234-row stress panel has separate IDs 46000–46233, with 216 crossed supported
rows, nine missing facts and nine conflicting facts. The four earlier-context
triples belong to the primary panel. The short-object instrument was designed
without reading any primary score, source choice or held candidate projection.

The table counts gold tapes within the lexical writer cap in each family and
modifier level. Each cell has eight supported rows. The short panel
generator rejects an over-limit supported tape before writing the manifest.
A tape fitting the cap can still lack an eligible frozen original word start.

| Group | Primary bare | Primary only | Primary also | Short bare | Short only | Short also |
|---|---:|---:|---:|---:|---:|---:|
| Bookbinding | 4 | 4 | 4 | 8 | 8 | 8 |
| Cartography | 0 | 0 | 0 | 8 | 8 | 8 |
| Clockmaking | 0 | 0 | 0 | 8 | 8 | 8 |
| Costume | 4 | 4 | 4 | 8 | 8 | 8 |
| Lighting | 0 | 0 | 0 | 8 | 8 | 8 |
| Mycology | 0 | 0 | 0 | 8 | 8 | 8 |
| Sculpture | 0 | 0 | 0 | 8 | 8 | 8 |
| Underwater | 0 | 0 | 0 | 8 | 8 | 8 |
| Waterworks | 0 | 0 | 0 | 8 | 8 | 8 |

The join key across cohorts is scenario family and the four crossing factors.
Report ordinary and supplied separately and show both admitted onset and
complete source-envelope counts in the same grid using the offline key and
verified native trace. Grade first token, full expected tape through EOS,
unsupported fact copies, requested/excluded roles, and assertion recovery on
all applicable rows. Compare frozen D26, fact-prefix true/reversed, and
candidate-prefix true/reversed, with the same two fitted banks and exact writer.
Use two independent native passes and per-candidate CUDA readback as in the
primary panel. When recapturing the supplied arm, use original query/factual head
spans only. Share no answer spans with the serving process.

An improvement confined to the matched short tapes establishes an effect in this
writer-admissible setting when it also exceeds the reversed control. Track
cross-family consistency and check whether the change increases unsupported
copies. If supplied candidate eligibility remains low, classify that failure
before reading rank gains. The matched panel reuses the primary scenarios and
names, so its nine families do not add nine independent scenarios to the
original comparison. This worksheet leaves decisions about run authorization
and any promotion with the top session.

```bash
d35-panel-freeze --short "$TOKENIZER" "$FROZEN_SHORT"
d35-panel project-held "$CHAT" "$D26" "$TOKENIZER" "$FROZEN_SHORT" \
  "$TRAIN_PROJECTION" "$GO_SHORT_PROJECT" "$SHORT_SPLIT"
```

Check each supplemental packet and index manifest. Compare the copied
`train.bin`, `train.index.json` and `train.omitted.json` with the original
TRAIN-only projection for both geometries. Bind the new packet and index
manifests, both FIT receipts and the short held manifest in the phase-specific
receipts. Use the primary run's frozen banks and carriers for the commands below.
Each capture invocation has six workers and the two invocations run serially.

```bash
"$STAGE/run-wick.sh" score "$SHORT_SPLIT" "$SHORT_SCORE_ADMISSION" \
  "$SHORT_SCORE_ADMISSION_SHA" "$SHORT_DEVICE" "$ORIGINAL_FIT_OUTPUT"
d35-panel capture "$CHAT" "$D26" "$FACT_D35" "$CANDIDATE_D35" "$TOKENIZER" \
  "$FROZEN_SHORT/held" "$GO_SHORT_SERVE" "$SHORT_PRIMARY"
d35-panel capture "$CHAT" "$D26" "$FACT_D35" "$CANDIDATE_D35" "$TOKENIZER" \
  "$FROZEN_SHORT/held" "$GO_SHORT_SERVE" "$SHORT_COLD"
```

Verify both capture manifests and compare them byte for byte. Decode the sealed
device outputs and verify common geometry, candidate pairs, orientation and
winner with `d35-parity` for all eight geometry × mode × pass cells.
`ordinary` device readback joins the ordinary captures and `diagnostic` joins
the supplied captures. Release the supplemental key for `d35-value-grade` after
the blind manifests and parity logs are sealed, then derive transition and
envelope counts from the independently graded rows.

The frozen geometry shares candidate membership across rank arms. The
following grid uses the baseline candidate trace to report the onset and
complete-source envelope for each group and modifier, separately by mode.

```bash
jq '
  [ .rows[] | select(.factors != null) ]
  | group_by(.mode, .group, .factors[3])
  | map({
      mode: .[0].mode,
      group: .[0].group,
      modifier: (["bare", "only", "also"][.[0].factors[3]]),
      rows: length,
      eligible_onset: (map(select(.arms.baseline.eligible_gold == true)) | length),
      eligible_complete: (map(select(.arms.baseline.eligible_complete == true)) | length),
      exact_baseline: (map(select(.arms.baseline.exact_eos == true)) | length),
      exact_fact: (map(select(.arms.fact_true.exact_eos == true)) | length),
      exact_candidate: (map(select(.arms.candidate_true.exact_eos == true)) | length)
    })
' "$SHORT_GRADED_ROWS"
```
