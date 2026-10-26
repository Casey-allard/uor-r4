# REPORT: d11-exporter-float-trunk, claude-133

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-investigation-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-investigation-2026-10-26.md) (FIT in round 2; D67 GO). This is the instrument investigation that [claude-132's residual sheet](../../../../harness/prep/d11-exporter-float-trunk-residual-2026-10-26.md) pre-registered for its indeterminate row.

References #2820.

**Speed and cost.** On one RTX 5090, the job ran in 6 m 24 s, from 16:33:05Z to 16:39:29Z. That covers the build, the GPU tests, the fetch, and six models of six DEV panels each. The pod was up for about 15 minutes at $0.975/h, about $0.25. No tokens/s of the artifact was measured; that is a PASS-only step of the export-gap sheets.

## What did not work, first

- **The outcome is "other arithmetic, still indeterminate",** the result the sheet expected before the run.
  - **D16 does not explain the residual.** `classification_q16` is still indeterminate on all four models that claude-132 classified indeterminate. By the sheet's definition (fewer rows differ, and a lower median and a lower mean drift), D16 is closer to A than D is on none of them. It moves toward A on one, d11-c130 seed 2, with fewer rows and a lower mean but the same median.
  - **No model shows a gross signature** beyond the faithful models' own (table below).
  - **What it decides,** under the sheet's third row: the remaining residual is fine-grained, and in its gross readings no different from the faithful models'. The next control is snap-flip localisation, which needs the evaluator to emit its snap choices. It is never a mechanism result, and **the item does not park.**
- **The Q16 control has almost no effect, as predicted.** On five of six models, D16 and D differ on only 8 to 18 rows.
  - **The exception is d11-c130 seed 2.** There D16 and D differ on 275 rows, and D16 is closer to A:
    - 441 rows differ, against 697 for D;
    - the mean drift is 65.0 codes, against 105.2;
    - but the median is 1 code on both, so it is not lower, and the sheet's "closer" needs both the median and the mean lower.

    At 441 rows (3.09%) it is still just over the 3% bound.
- **So the exporter is still not cleared.** But the readings show none of the signatures of a gross defect, on any model.

## What changed

Size against `origin/main`: float-train, in `hamilton_sheet.rs`, `geometric_stack_native.rs` and `spiralcore_fbs_sheet.rs`. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **The Q16 control.**
  - `hamilton_sheet::with_q16_residual` rounds the trunk's residual stream to 2^-16 after the embedding and after each sublayer add (`StackModel::hidden`), where the integer session holds it.
  - It has its own thread-local flag, because `with_d11_mode` saves and restores the gate hook.
  - It runs in evaluation only. With the flag off, `q16_residual` returns the tensor unchanged.
- **`d11-residual` gains:**
  - the D16 panel and its head check (`I-Head-Q16`);
  - D16's pairings with A and with D;
  - `classification_q16`, claude-132's rule with D16 as the reference;
  - `drift_readings` for A against D, A against D16, and D16 against D.
- **`drift_readings`** gives, in Q12 codes:
  - the median, mean, 99th-percentile and largest absolute difference;
  - the columns holding the most per-row maxima;
  - the largest per-channel mean signed difference;
  - the saturated entries on each side;
  - the sign flips (both sides at least 64 codes, of opposite sign);
  - the per-row L1 scale Σ|a| / Σ|r|. It has no products (§3.3).
- **Tests:**
  - the host tests `the_q16_control_sets_and_restores_its_flag` and `the_drift_readings_count_codes_columns_saturation_and_scale`, the latter including the 64-code sign-flip boundary;
  - the GPU test `gpu_the_q16_control_holds_the_residual_on_its_grid_and_is_off_by_default`.
- **After the run, from the code review:**
  - **`drift_readings` no longer credits a row with no difference to column 0.** It counts such rows as `rows_no_gap`. The test fails with this fix set aside (`rows_no_gap` 0 against 1) and passes with it restored.
    - In the stored receipts, made before the fix, `top_max_columns` is inflated at column 0 by the exactly equal rows. That affects only the comparisons that have such rows, D16 against D.
    - The A-against-D readings this REPORT cites are unaffected, because every one of their rows differs (`rows_exact` 0).
  - **The Q16 head check now gates only `classification_q16`** (`instruments_pass_q16`). It was true on every model in the run.
  - **`with_q16_residual` restores its flag on a panic too,** and the flag test checks that.

## Measurements

**The gross-signature check (sheet §3), A against D.**
- **The envelope** is the larger of claude-132's two faithful models (d11-c130 s1 and dual-c131 s1).
- **A signature** is a reading above twice the envelope plus the floor.

| Reading | Envelope | Threshold | Indeterminate models: plain-c126 s1 / s2, d11-c130 s2, dual-c131 s2 | Signature? |
|---|---|---|---|---|
| \|median row L1 scale − 1\| | 3.6e-6 | 0.0100 | 1.3e-6, 2.5e-6, 0, 2.6e-6 | none |
| \|largest channel mean signed difference\| (codes) | 118.8 | 239.7 | 142.0, 28.5, 189.2, 24.7 | none |
| \|saturation excess\| (entries) | 195 | 4,047 | 7, 2, 330, 3 | none |
| Sign flips (entries) | 10,100 | 23,857 | 18,558, 3,055, 17,311, 3,419 | none |

No model has `classification_q16` = departure.

**D16 against A, and against D, per model:**

| Model | D ≠ A rows | D16 ≠ A rows | D ≠ D16 rows | D16 − A net | `classification_q16` |
|---|---|---|---|---|---|
| plain-c126 s1 | 580 | 586 | 9 | +3 | indeterminate |
| plain-c126 s2 | 484 | 496 | 18 | −4 | indeterminate |
| d11-c130 s1 | 367 | 365 | 8 | +3 | faithful |
| d11-c130 s2 | 697 | 441 | 275 | 0 | indeterminate |
| dual-c131 s1 | 289 | 282 | 13 | −6 | faithful |
| dual-c131 s2 | 497 | 505 | 16 | +5 | indeterminate |

**Every instrument check was green.**
- **`I-GPU-Tests`:** the float-train CUDA library passed, 64 passed and 5 ignored, with the Q16 GPU test included.
- **`I-Fetch`:** every fetched store file verifies against its folder's `SHA256SUMS`, claude-132's six receipts included. The DEV data and the six models verify against their pins.
- **claude-132's per-model checks,** on all six: `I-Save-Load`, `I-Inputs`, `I-Verify-All`, `I-DEV-Denominator`, `I-D-Determinism` and `I-Head`. Also `I-Head-Q16`, and the writers' i32 oracle with 0 mismatches.
- **`I-D-Stored`, on plain-c126:** D equals Stage 4b's stored D11 panel.
- **`I-D-Reproduces`, on all six:** the D top-1, the D-against-A rows-differ count and the classification equal claude-132's receipts, each value non-null.
- **`I-Finite`:** every step exited 0, and all 201 files listed in `inv/SHA256SUMS` verify, on the pod and again on the laptop.

**Readings, with no bar** (A against D, in Q12 codes):
- **The median difference** is 1 to 3 codes on every model. The mean is 14.6 to 105.2, pulled by tails. The largest single difference is 34,694 to 63,162.
- **0.1% to 1.5% of entries saturate at the i16 ends** (5,228 to 53,016 per side, of 3,657,472), equally on both sides to within 330. The per-row maxima cluster in a few channels: for example, 1,241 rows in channel 56 on plain-c126 s1, and 1,321 rows in channel 127 on d11-c130 s1.
  - That fits a saturating channel whose value sits near ±8 and flips sign: one code of difference at the clamp becomes a jump of up to 65,535.
  - It also fits the sign-flip counts (0.08% to 0.5% of entries).
  - This is a hypothesis about the tails, not a measurement of them.
- **The faithful models show the same pattern:** channel means of 74 to 119 codes, about 1% saturated, and 8,257 to 10,100 sign flips. So it is not what separates the four indeterminate models.
- **The Q12 drift between D16 and A is never exact either** (`rows_exact` 0 on every model). Within 2 codes it ranges from 160 to 7,586 rows, against 161 to 7,602 for D.

**Evidence:** the HF store at `results/claude/d11-fbs-investigation-2026-10-26/`, each folder with a `SHA256SUMS`:
- `inv/` holds every step's stdout, stderr and exit code, the six receipts, the input pins and `job.log`;
- `inv-bin/` holds the binary run, `spiralcore-fbs-sheet`, sha256 `5a2943ab…`.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** The expected outcome was written down before the run, and the rule was applied as fixed. That includes the near miss on d11-c130 seed 2, which stays indeterminate at 3.09%.
- **writing-meaningful-tests.** The Q16 control's off state is pinned bit for bit on the GPU. The drift readings are pinned at their boundaries, including the 64-code sign-flip edge.

## Still open

- **Snap-flip localisation, the next control.** It records the layer and position of every icosian-snap disagreement between D and A. It needs the GPU evaluator to emit its snap choices, which are not in the evaluator's output today.
  - It would test the leading hypothesis: that a snap flipping on a near-tie, together with saturating channels, carries the residual through the recurrence.
  - After it come `stack_quantize16` and the other integer roundings.
- **Two readings for the localisation:**
  - the channels that hold the per-row maxima on each model;
  - whether the rows where D and A differ coincide with saturated channels.
- **The pivot note's order is unchanged:** first this investigation settles, then the owner's reading of §3.7 (#2028), then one decisive run. The item parks only once the investigation settles.

## Plain summary

This looked for signs that the shipped integer files compute something different from what they should.
- **None of the obvious signs of a defect showed up** on any of the six models: no scale error, no sign errors beyond the normal ones, no lopsided clipping. In those respects the four uncertain models fall within the agreed tolerance of the two that passed, although some raw readings are larger, up to about twice the sign flips (18,558 against 10,100), and others are smaller.
- **A small adjustment to make the reference match the integer arithmetic more closely changed almost nothing,** as expected. So the leftover differences are not explained yet.
- **An untested hypothesis** is a rounding decision that flips at a tipping point and then carries forward. Checking that needs the integer evaluator to report those decisions, which is the next step.
