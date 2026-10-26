# REPORT: d11-exporter-float-trunk, claude-132

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-residual-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-residual-2026-10-26.md) (FIT in round 2; D67 pre-run review GO). This is the first step of claude-131's §4.4 pivot note ([decision review](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6419387874)).

References #2820.

**Speed.** On one RTX 5090, the job ran in 7 m 7 s, including the build and the GPU tests. Each of the six models takes five DEV panels of 14,287 rows each, plus the head. No tokens/s of the artifact was measured here; that is a PASS-only step of the export-gap sheets.

## What did not work, first

- **The pre-registered classification is not "faithful on all six".** It is 2 faithful, 4 indeterminate and 0 departures. Let d_D be the rows where the D11 panel (D) and the artifact (A) differ:

  | Model | d_D | Share of 14,287 | Class |
  |---|---|---|---|
  | plain-c126, seed 1 | 580 | 4.1% | indeterminate |
  | plain-c126, seed 2 | 484 | 3.4% | indeterminate |
  | d11-c130, seed 1 | 367 | 2.6% | faithful |
  | d11-c130, seed 2 | 697 | 4.9% | indeterminate |
  | dual-c131, seed 1 | 289 | 2.0% | faithful |
  | dual-c131, seed 2 | 497 | 3.5% | indeterminate |

  - Every d_D is below the 10% departure bound. Four are above the 3% faithful bound, three times claude-127's measured integer residual against its f64 reference.
  - **What it decides,** under the sheet's indeterminate row: an instrument investigation of those four models' trunks, led by the drift readings. It is never a mechanism result, and **the item does not park until it is settled.** The exporter is therefore not yet cleared as the cause.
- **The Q12 drift between D's and A's queries is never exact.**
  - On all six models, the faithful two included, no row has D's and A's queries equal (`rows_exact` 0).
  - Only 1% to 53% of rows lie within 2 codes in every channel.
  - The largest per-row drifts are 34,694 to 63,162 codes.
  - The statistic takes the maximum over all of a row's channels, so a few codes of float-against-integer rounding in any one channel moves a row out of "within 2". The very large maxima fit an icosian snap that flips on a few rows. But this run does not show that, so it is the investigation's first question.
- **The FIT's medium note was borne out.** It warned that a float32 D against an integer artifact might land a faithful artifact in "indeterminate". Whether that is what happened here is what the investigation settles; the rule is not changed after the fact.

## What changed

Size against `origin/main`: float-train, in `spiralcore_fbs_sheet.rs`, `hamilton_sheet.rs` and the sheet binary. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **`spiralcore-fbs-sheet --mode d11-residual`** (`run_d11_residual`) scores one saved model on the GPU five ways:
  - F, the plain-float panel;
  - E, the export-mode panel, on the tables as trained;
  - D, the D11 panel, with the tables rounded as exported;
  - H, the writer head at Q12 on the artifact's own queries;
  - A, the artifact's tokens from its stored, verified evaluator run.

  It pairs them, measures the Q12 drift, and classifies the model under `classify_residual`, the sheet's rule. Its options are `--model-sha256` (part of `I-Inputs`) and, where Stage 4b stored the D11 panel, `--stored-d11-preds` (`I-D-Stored`).
- **`hamilton_sheet::dev_panel_q12_queries` and `write_q12_rows`**: the panel's scored Q12 queries under the current gate hook, and the panel's writer head on given queries. `score_dev_panel` is unchanged.
- **Tests:**
  - the host tests `the_q12_drift_is_each_rows_largest_code_distance` and `the_residual_rule_is_the_sheets` (the rule at its 428/429 and 1,428/1,429 boundaries);
  - the host test `the_residual_options_parse_and_only_the_stored_panel_is_optional`, in the binary;
  - the GPU test `gpu_the_head_on_the_d11_queries_is_the_d11_panel`.
- **After the run,** following the code review, the mode's option parsing moved into a pure `residual_args`, which the binary's host test pins. The mode's behaviour is the same as the run built at `9bfbf31c8`.

## Measurements

**No departure on any model.** The departure conditions each failed on all six:
- **The head is identical on every row of every model:** H and A differ on 0 of 14,287 rows. So on A's own queries, the panel's head writes A's tokens.
- **D against A passes the paired test on every model:** the nets are +4, −4, +3, +8, −6 and +6, each against F_I 24.
- **d_D is below 10% on every model** (at most 4.9%).

**Every instrument check was green.**
- **`I-GPU-Tests`:** the float-train CUDA library passed on the RTX 5090, 61 passed and 5 ignored, with the new head test included.
- **`I-Fetch`:** all 26 fetched store files verify against their folders' `SHA256SUMS`, and the 2 DEV files and 6 models against their 8 pins.
- **The per-model checks, on all six:**
  - `I-Save-Load`: F reproduces the stored predictions;
  - `I-Inputs`: the artifact, windows and model pins match;
  - `I-Verify-All`: the stored run's two verify checks are equal at all 82,176 positions (428 windows × 192);
  - `I-DEV-Denominator`: 14,287 rows;
  - `I-D-Determinism`: the D panel repeats exactly;
  - `I-Head`: the head on D's own queries writes D;
  - the writer's i32 oracle shows 0 mismatches.
- **`I-D-Stored`, on plain-c126:** the recomputed D equals Stage 4b's stored D11 panel row for row, on both seeds, so D reproduces across builds.
- **`I-Finite`:** every step exited 0, and all 156 files listed in `resid/SHA256SUMS` verify, on the pod and again on the laptop.

**Readings, with no bar:**
- **Where the earlier FAILs' top-1 gaps lie.** A paired net is hits(X) − hits(Y), so net(F, A) = net(F, D) + net(D, A) exactly:

  | Model | F − A, the FAIL's net | F − D | D − A |
  |---|---|---|---|
  | plain-c126, s1 (claude-129) | +81 | +77 | +4 |
  | plain-c126, s2 (claude-129) | +349 | +353 | −4 |
  | d11-c130, s1 | +278 | +275 | +3 |
  | d11-c130, s2 | −332 | −340 | +8 |
  | dual-c131, s1 | +353 | +359 | −6 |
  | dual-c131, s2 | +231 | +225 | +6 |

  - The F − A column reproduces the earlier gap receipts exactly.
  - In top-1 counts, each FAIL's gap lies between the plain-float forward and the D11 forward. The artifact moves the net by at most 8.
  - This is a hit-count reading. The FIT required identity, not hit counts, for the classification, so it does not clear the exporter by itself.
- **The other forwards:**
  - E against D: 4,989 to 9,501 rows differ, so the export-mode panel is far from the D11 forward.
  - F against D: 7,516 to 11,828 rows differ.
- **Top-1 counts** (F / E / D / H / A):
  - plain-c126: 1,988 / 2,027 / 1,911 / 1,907 / 1,907 and 2,643 / 2,444 / 2,290 / 2,294 / 2,294;
  - d11-c130: 3,941 / 3,638 / 3,666 / 3,663 / 3,663 and 917 / 1,237 / 1,257 / 1,249 / 1,249;
  - dual-c131: 3,845 / 3,511 / 3,486 / 3,492 / 3,492 and 3,128 / 3,023 / 2,903 / 2,897 / 2,897.
- **The drift on the rows where D and A differ:** 0 to 24 of those rows lie within 2 codes.

**Evidence:** the HF store at `results/claude/d11-fbs-residual-2026-10-26/`, each folder with a `SHA256SUMS`:
- `resid/` holds every step's stdout, stderr and exit code, the six residual receipts, the input pins and `job.log`;
- `resid-bin/` holds the binary run, `spiralcore-fbs-sheet`, sha256 `5df27885…`.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** The rule was fixed before the run, and FIT round 1 made it stricter. It was applied as written, including the four indeterminate results, although the hit-count reading points the other way.
- **writing-meaningful-tests.** The rule's boundaries are pinned by a host test. The head's identity with the D11 panel is pinned by a GPU test, which also ran as an instrument check on every model.

## Still open

- **The instrument investigation, the sheet's indeterminate row, next on this item.** It should settle where the 2% to 5% of D-against-A token differences come from. Two candidates:
  - **float32 rounding in D.** D is a float32 candle forward. The 3% bound came from the integer path against an **f64** quantized reference.
  - **An exporter departure.**

  claude-127's own residual matters here. The integer session differs from its f64 specification on about 1% of single tokens at 6 layers, and that residual is itself unexplained. One candidate is the artifact's Q16 residual stream, though claude-127 notes that its 1-layer seed-2 differences do not fit accumulation alone. The control that would test it, the f64 reference with its residual rounded to Q16 at the same points, was not built ([claude-127 REPORT](../claude-127/REPORT.md)).

  So the investigation has two parts.
  - **Three host checks, needing no new GPU spend:**
    1. build that Q16 control;
    2. run the integer session against the f64 reference and against the Q16 control, on all six stored snapshots;
    3. record per-channel drift and the layer and position of every icosian-snap difference.
  - **The drift readings the code review asked for.** This run's per-row maximum cannot separate a scale or exponent error, one saturating channel, and a real forward departure. So the readings needed are:
    - the column holding each row's maximum;
    - the median absolute difference;
    - the saturation counts (±32,767 and −32,768) on each side;
    - a per-row least-squares scale.

    They need D's queries, which this run did not store, so they are a short GPU evaluation with its own sheet. No departure is attributed to the exporter before these readings exist.

  How it reads:
  - **A tracks the Q16 control:** it computes the D11 forward with a Q16 residual, as designed. The extra D-against-A difference is D's float32 rounding together with Q16. Whether that is faithful is then a calibration question for the rule, settled by a decision review, not after the fact.
  - **A does not track it:** that points to a departure. With the drift readings below, it is attributed, and a repair item opens.
- **The code review's low findings, for that unit:**
  - `I-D-Stored` records `null` when its flag is absent, which reads like "passed". Make it "not run".
  - The new options are accepted, and ignored, by every gap-family mode, as `--train-forward` already was.
  - The GPU head test does not exercise the query clamp. Nothing yet compares D's queries with an exported artifact's on a synthetic trunk.
- **After that, the pivot note's order stands:** the owner's reading of §3.7 (#2028), then one decisive run. The item parks only once the investigation is settled.

## Plain summary

This checked whether the shipped integer files compute what the model's integer-like mode computes.
- **The file's final step is exactly right on every model.** It picks the same answers as the model's own head, on all 14,287 rows of all six models.
- **The files get as many answers right as the model's integer-like mode,** to within 8 of 14,287 per model. But the answers they pick still differ from that mode's on 2% to 5% of the rows.
- **Counted as right answers, the earlier failed comparisons sit almost entirely between the model's ordinary float mode and its integer-like mode.** That is a reading, not a verdict. It does not clear the export.
- **Four of the six models differ by more than the pre-agreed limit.** So the export is not yet cleared. Next come checks on the laptop and one short GPU measurement of how the internal numbers differ.
