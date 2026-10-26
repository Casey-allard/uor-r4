# REPORT: d11-exporter-float-trunk, claude-135

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-forced-snaps-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-forced-snaps-2026-10-26.md) (FIT in round 2, D67 GO). This is the forced-snap control set by the decision review after [claude-134](../claude-134/REPORT.md).

References #2820.

**Speed and cost.**
- **The tokens/s are the instrument's, not the served product's.**
  - Each full-panel trace scored 428 sequences × 192 positions in the evaluator, then held the GPU's trace against the host twin's.
  - Each trace step took about 6 minutes, nearly all of it the host twin. Each forced comparison (five D11 panels over 14,287 rows) took about 54 s.
- **Times:**
  - The job ran in 45 minutes, from 19:35:44Z to 20:20:44Z.
  - The sheet's estimate of 30 minutes left out the host twin's share of each trace.
- **Cost:** the pod was up from 19:33:21Z to 20:24:38Z at $1.054/h, about $0.90.

## What did not work, first

- **The four indeterminate models split, so there is no item reading.** The sheet sends a split to a decision review, and the item does not park.
  - **Faithful:** plain-c126 s1 and d11-c130 s2.
  - **Partly snap-carried:** plain-c126 s2 (137 rows) and dual-c131 s2 (139 rows). Both are above the bar's 71 rows.
- **One of claude-132's two faithful models reads Partly snap-carried.** On d11-c130 s1, forced-D and A still differ on 98 rows, above the bar's 71 (0.5%).
  - So the continuous residual left after forcing is not zero even on a model the residual check called faithful.
  - The other faithful model, dual-c131 s1, keeps 34 rows. The two Partly snap-carried indeterminate models keep 137 and 139.
  - A reading, not a ruling: the 0.5% line sits within the range the faithful models themselves occupy (34 to 98).
- **The exporter is not cleared.** The clearing reading needed Faithful on all four indeterminate models.
- **Forcing hides A's pre-snap arithmetic, so it does not show by itself that A's choices come from rounding.** The would-flips speak to that part: they are near-ties at the faithful models' rate on all six (Measurements).

## What changed

Against `origin/main`: float-train only. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **The snap override** (`hamilton_sheet.rs`):
  - `with_snap_override` forces the k-th call of `icosian_snap_cuda_tensor` to given roots; 255 applies root 0, the integer zero convention.
  - A capture running alongside still records D's own choice and margin.
  - Wrong block counts, bytes that name no root, calls beyond the given ones and leftover calls are all refused, and the prior state is restored on every exit.
  - `self_forced` maps a capture back to the roots it applied.
  - It is for evaluation forwards only.
- **The forced-snap mode** (`spiralcore_fbs_sheet.rs`, `--mode d11-forced-snaps`):
  - **Forcing:**
    - `forced_calls` lays A's trace out as the panel's calls: per chunk of 16 windows, per layer, per site.
    - `trace_layout` is its inverse, for `I-Forced-Layout`.
    - `wrong_root_calls` moves one site's roots, for `I-Wrong-Root`.
  - **The readings and the bar:**
    - `would_flip_count` counts the root and one-sided zero would-flips. Zero blocks on both sides are left out, since they are the same arithmetic apart only in convention.
    - `classify_forced` applies the sheet's precedence.
    - `summarize_forced` (`--mode d11-forced-summary`) sets the envelope from the faithful models, with a floor of 10⁻⁷. It refuses a missing or failed faithful model, two receipts for one model, and receipts over different windows.
- **After the run, from code review:**
  - `I-Old-Run` also pins the stored run's windows. This run's binary predates that check, so it was done by hand on the pod: all six stored runs are over `windows.json`, `083c1be6…` (store `checks/`).
  - The summary's consistency refusals were added.
  - The classifier's exact boundaries got tests.
  - These change no reading of the run.
- **Tests:**
  - **Host tests (28 in the two modules):**
    - `a_snap_override_forces_each_call_in_order_and_refuses_a_mismatch`;
    - `the_forced_calls_follow_the_panel_chunks_and_the_wrong_root_moves_one_site`, including the layout round trip;
    - `a_zero_block_on_both_sides_is_never_a_would_flip`. A negative gate was run: the all-class sum fails it;
    - `the_forced_outcome_takes_the_first_that_applies`;
    - `the_forced_summary_reads_the_faithful_models_envelope`.
  - **GPU test:** `gpu_forcing_its_own_snaps_reproduces_d_and_a_wrong_root_does_not`.
  - **Negative gates on the post-run delta:** each of the following fails its test with its comparison flipped or its refusal removed:
    - the decisive-margin boundary (≥ 1e-2);
    - the near-tie boundary (< 1e-3);
    - the envelope boundary (> twice);
    - the duplicate-receipt refusal;
    - the windows refusal.

## Measurements

**Per model,** over the full DEV panel (14,287 rows, 110,280,192 non-inert snap sites each). Would-flips are counted as the sheet defines them. The envelope is 4.31e-4, d11-c130 s1's rate, so twice the envelope is 8.62e-4.

| Model | Rows differ unforced → forced | Still differing / new | Would-flips (rate) | m (exact ties) | Would-flips below 1e-3 (off a tie) | Outcome |
|---|---|---|---|---|---|---|
| plain-c126 s1 | 580 → 25 | 12 / 13 | 55,284 (5.01e-4) | 9.0e-5 (13) | 95.7% | **Faithful** |
| plain-c126 s2 | 484 → 137 | 111 / 26 | 47,851 (4.34e-4) | 1.08e-4 (32) | 92.1% | Partly snap-carried |
| d11-c130 s1 † | 367 → 98 | 82 / 16 | 47,540 (4.31e-4) | 1.20e-4 (331) | 96.1% | Partly snap-carried |
| d11-c130 s2 | 697 → 5 | 4 / 1 | 44,456 (4.03e-4) | 3.0e-5 (17) | 94.9% | **Faithful** |
| dual-c131 s1 † | 289 → 34 | 32 / 2 | 42,452 (3.85e-4) | 9.1e-5 (23) | 97.4% | **Faithful** |
| dual-c131 s2 | 497 → 139 | 121 / 18 | 90,845 (8.24e-4) | 6.5e-5 (173) | 91.6% | Partly snap-carried |

† Faithful in claude-132. Their outcomes are readings, and their rates set the envelope.

- **Still differing / new:** of the rows where forced-D and A differ, those where unforced D also differed, and those new to forcing.
- **m** is D's upper median margin at the root would-flips off an exact tie.

**Readings, with no bar:**
- **Forcing removes 72% to 99% of the residual on every model,** the faithful two included (580 → 25, 484 → 137, 367 → 98, 697 → 5, 289 → 34, 497 → 139).
  - What remains is mostly on rows that already differed: 4 to 121 of the 5 to 139.
  - Only 1 to 26 rows are new to forcing.
- **The would-flips are near-ties on every model.**
  - The upper median margin is 3.0e-5 to 1.2e-4.
  - Off an exact tie, 91.6% to 97.4% are below 1e-3, and 97.9% to 99.8% below 1e-2.
  - The largest are 0.07 to 0.14, a small decisive tail.
  - **Exact ties:** 13 to 331 per model, at most 0.7% of the would-flips. The tie-break outcome never applies.
- **No zero block of any class** occurs on any model, unforced or forced.
- **The rates are inside the envelope.** All six are within twice the faithful envelope. dual-c131 s2 comes closest, at 8.24e-4 against 8.62e-4, with about twice the would-flips of the others.
- **The would-flips gather at a few sites.**
  - The single largest (layer, site) holds 14% to 68% of a model's would-flips. It is 68% on d11-c130 s2 and 44% on dual-c131 s2.
  - m covers root would-flips only; with no zero blocks, that is all of them.
- **The wrong-root gates move many rows.** Layer 0's output gate moved to other roots changes 13,122 to 13,897 rows, and the last layer's feed-forward gate 2,045 to 6,662.

**Every instrument check was green:**
- **`I-GPU-Tests`:** the float-train CUDA library passed 75 tests, with 5 ignored, including the forcing GPU test. The instrument's parity tests passed 4.
- **`I-Kernel`:** `qualify` passed, and its PTX `5a947f09285a359dfb0ddfdf4bdf31746b959ef39f6056807eead9b4672f87ef` and cubin `2a386d8c563516aeef10d040200867330cafb54d5525cdd7580b7969554a4c86` equal claude-134's qualified pins.
- **`I-Inputs`:** all 15 input pins match, and every fetched folder checks against its `SHA256SUMS`.
- **Per-model checks, true on all six:**
  - **Inputs and trace:** `I-Inputs`, `I-Trace` (G = H over all 428 windows) and `I-Old-Run` (the traced tokens equal the stored run at every position).
  - **Forcing:**
    - `I-Self-Force` (D forced to its own choices reproduces D on every row);
    - `I-Forced-Layout` (D's choices laid out as a trace map back to the panel's calls);
    - `I-Wrong-Root` (both sites);
    - `I-Unforced` (580, 484, 367, 697, 289 and 497, claude-132's counts).
  - **Readings:** `I-Layout` and finite margins.
  - **The chunk boundary:** each forced call spans many of the snap's 8,192-block chunks, so `I-Self-Force` also holds the forcing exact across them.
- **`I-Finite`:**
  - every step exited 0, and the forced calls were used up exactly;
  - all 215 files listed in `forced/SHA256SUMS` verify, on the pod and on the laptop (216 with the list itself);
  - the 221 files across the three store folders match the local copies.

**Evidence:** the HF store at `results/claude/d11-fbs-forced-snaps-2026-10-26/`, each folder with a `SHA256SUMS`:
- `forced/`: every step's output, `qualify.json`, the six receipts, `summary.json`, the six traces (gzipped) and `job.log`;
- `forced-bin/`: the binaries run, `spiralcore-fbs-sheet` `56eb0d14…` and `d11-fbs-eval` `ee4e361a…`, built at `c044544e3`;
- `checks/`: the stored runs' windows, checked on the pod.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** The bar was applied as fixed. A faithful model landing on Partly snap-carried is reported as it came out. It does not move the 0.5% line after the fact; any change to it is the decision review's.
- **writing-meaningful-tests.** The would-flip definition is pinned by a test that fails with the all-class sum. The forcing layout is pinned by a round trip, and the classifier at its exact boundaries.

## Still open

- **The decision review that a split calls for has passed** (#2820 owner log, comment 6426588305).
  - The classification stands, so the exporter is not cleared. The item does not park, and the 71-row line is not moved for this run.
  - Forcing shows that the snaps carry 72 to 99% of the residual. It does not show that A's choices come from rounding.
- **What the split leaves open.** The snaps carry most of the residual on every model, and D's would-flips are near-ties at the faithful rate. What separates Faithful from Partly snap-carried is a continuous residual of 5 to 139 rows after forcing, and the faithful models carry 34 and 98 of their own.
- **The next unit is the continuous localisation,** the sheet's control for Partly snap-carried. It runs on all six models, with the faithful two as its reference: forced-D's and A's Q12 queries, and the residual stream at the final norm layer by layer, at the rows that still differ.
  - **It is the last exporter-faithfulness unit, and every outcome ends the investigation:**
    - **design precision:** the leftover traces to Q16 or float32 rounding, no layer or site dominates, and the model is inside the faithful envelope. The exporter is cleared;
    - **a defect:** a layer or site departs beyond Q16 rounding. It is repaired, and the residual check is re-run once;
    - **neither:** the leftover is reported as part of the export gap.
  - **The decisive run follows** in every case.
  - **A decision-relevance bound:** it reports how many net top-1 rows the leftover could move, against F_I (60 to 73) and the FAILs' margins (+81 to +353). claude-132 found |D − A| at most 8 in net on each FAIL.
  - **Its envelope rule is fixed before the run.** It states that two reference models are thin.
- **The pivot note's order is unchanged:** the investigation settles, then the decisive run. Nothing waits on the owner (§5.1).

## Plain summary

This made the reference copy of the model take exactly the same knife-edge rounding choices as the shipped integer file, then counted where their predicted words still differed.
- **Forcing those choices removes most of the disagreement,** between 72% and 99% depending on the model.
- **The remaining choices the reference would have made differently are near-ties,** at about the rate seen on the models already judged faithful.
- **A small leftover remains on every model.** It comes from the ordinary arithmetic between the choices: 5 to 139 words out of 14,287.
- **Two of the four uncertain models now pass;** the other two leave a little more than the agreed line. One of the two models already judged faithful also leaves more than that line. The next step is a review of whether that line, or the leftover arithmetic, is the question.
