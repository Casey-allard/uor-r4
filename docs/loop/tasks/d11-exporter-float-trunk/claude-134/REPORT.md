# REPORT: d11-exporter-float-trunk, claude-134

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-snap-trace-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-snap-trace-2026-10-26.md) (FIT in round 3, D67 GO). This is the snap-flip localisation that [claude-133](../claude-133/REPORT.md) named as the next control.

References #2820.

**Speed and cost.**
- **The tokens/s are the instrument's, not the served product's.**
  - The traced evaluator scored 32 sequences × 192 positions in 13.9 to 14.1 s on one RTX 5090: 437 to 441 tok/s.
  - The untraced evaluator scored 428 × 192 in 14.3 s: 5,729 tok/s.
  - It runs one sequence per thread, so a batch takes about 14 s whatever its size, and the trace costs nothing measurable.
- **Times:**
  - The first job ran in 29 m 27 s, from 17:45:40Z to 18:15:07Z: the build, the GPU tests, the fetch, the qualification (19 m 34 s, mostly the host twin and the Rust session of `--verify`), and the six traces and comparisons.
  - The second-build job ran in 7 m 41 s (18:18:15Z to 18:25:56Z).
- **Cost:** the pod was up from 17:42:53Z to 18:28:28Z at $1.106/h, about $0.84. The down time is from the `uor-pod down` call, and is not in the store.

## What did not work, first

- **The outcome is Indeterminate on all six models.** The sheet pre-registered this for the case where the bar's lift cannot be read.
  - Every row where D and A differ has a root disagreement in its window at or before it (P = 100%), but so do 99.2% to 100% of the rows where they agree (B).
  - The lift P − B is 0 to 0.8 points, against the 20 the near-tie outcome needs.
  - The sheet's rule for B ≥ 80% applies: the flips are too common for this panel to place the rows.
  - **What it decides:** a decision review, and the item does not park.
- **Why B saturates: the panel, not the mechanism (instrument).**
  - **The cause is early flips in every window.** On every model, faithful ones included, every window has a root disagreement, at a median position of 0 to 8. The first differing row comes a median 43 to 63 positions later, so almost every scored row follows a flip.
  - **A shared prefix adds to it.** 30 of the 32 windows share their first 4 tokens, `[0, 55, 2728, 28]`, so the computation at positions 0 to 3 is the same in all 30. The 32-window file is in the store at `windows/`. Its sha256, `d5f8ef23…`, equals the receipts' pin, and it is rebuilt from the pinned `gap/windows.json`.
  - The FIT did not check the windows' prefixes, and the run file's FIT step has no such check.
- **Four models' first flips are effectively one event each.** In the windows where D and A differ, the first flip falls in the shared prefix on:
  - plain-c126 s1: 20 of 21, at position 0, layer 2's output gate;
  - plain-c126 s2: 19 of 21, at position 3, layer 1's feed-forward gate;
  - d11-c130 s1: 13 of 13, at position 0, layer 4's output gate;
  - dual-c131 s2: 18 of 18, at position 2, layer 4's transition.

  On those four, m rests on essentially one near-tie snap.
- **Only d11-c130 s2 (17 windows) and dual-c131 s1 (15 windows) flip after the prefix, and those flips are mostly distinct, not independent.**
  - On d11-c130 s2, 5 windows flip at (position 4, layer 4, feed-forward gate).
  - On dual-c131 s1, 4 flip at (position 5, layer 0, feed-forward gate).
  - The windows also share longer prefixes in small groups: 5 windows share their first 5 tokens, and two groups of 4 do. So each of those repeats may be one event.
- **The ORDER's plan asked for the windows with the most D-against-A differences.** The sheet took the first 32 DEV windows instead, a choice made before seeing data. Its FIT accepted that. It is a deviation from the ORDER's plan, recorded here.
- **The first second-build attempt stopped at its first step (instrument).** The pod's checkout has no git remote. The retry fetched the commit by URL into a fresh folder (`snap3/`), and the stopped attempt is kept in `snap2/` with its own `EXIT` 128. No GPU time was lost.
- **The run's binaries predate the code review's fixes.** The first job ran at `3cb4d3dec`. The second build at `d75b525e4` carries every fix and the `I-Layout` check, and it reproduced the first byte for byte (Measurements).

## What changed

Against `origin/main`: the evaluator instrument and the float-train crate. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **The evaluator's snap trace** (`docs/labs/d11-exporter-float-trunk/instrument`):
  - **The core.** `fbs_core.h`'s `fbs_snap_rotation` returns the chosen root's index. `fbs_recurrence`, `fbs_hamilton_gate`, `fbs_step` and `fbs_sequence` take an optional trace pointer. When it is null, the old path runs unchanged.
  - **The layout.** Per position, per layer: the transition's root per lane, the output gate's root per lane, then the feed-forward gate's root per 4 MLP channels. A zero raw is 255 (`fbs_trace_root`).
  - **The kernels.** A second kernel, `fbs_trace_sequences`, and its host twin. The command `d11-fbs-eval trace ARTIFACT SEQ OUT` holds the GPU's outputs and trace against the host twin's (G = H), and checks the buffer against the documented stride.
  - **The host prefill.** The host trace is prefilled with 0xFE, a byte the core never writes. A slot it misses therefore fails the parity test, and fails G = H against the GPU's zeroed buffer.
- **The float capture** (`hamilton_sheet.rs`):
  - `with_snap_capture` records each `icosian_snap_cuda_tensor` call's chosen root (255 for a zero block) and its relative margin, (best − second) / best.
  - It restores its state on every exit, a panic included.
  - `is_capturing_snaps` is CUDA-only.
- **The comparison** (`spiralcore_fbs_sheet.rs`, `--mode d11-snap-trace`):
  - **Instruments first:**
    - D's tokens and snap choices come from one captured D11 panel, its two chunks joined per call (`I-Capture`).
    - A snap byte that names no root is refused.
  - **The readings:**
    - Disagreements are classed by the root each side applies. Root 0 is −1 and root 1 is the identity, so a zero block can disagree.
    - A transition at position 0 rotates a zero state and is counted as inert.
    - Root and zero-block disagreements are placed in separate masks, each with its base rate and lead.
    - Each window's first root disagreement in causal order holds its primaries.
  - **The checks and the bar:**
    - `I-Layout` requires each (layer, site) to agree on at least 50% of its snaps.
    - Finite margins are required.
    - `classify_snap_trace` applies the sheet's precedence.
- **Tests:**
  - **Instrument:** the host parity test `the_snap_trace_leaves_the_outputs_unchanged_and_names_roots` (prefill, range, stride, repeatability).
  - **Float-train host tests:**
    - `the_snap_comparison_places_each_disagreement`;
    - `a_snap_class_reads_the_applied_roots`, which pins root 0 as −1 and root 1 as the identity;
    - `a_windows_first_flip_is_first_in_causal_order`;
    - `the_snap_trace_outcome_takes_the_first_that_applies`.
  - **The GPU test:** `gpu_the_snap_capture_records_each_site_and_changes_nothing`.
  - **Negative gates, each failing with its fix set aside:**
    - the inert exclusion;
    - the zero mask's lift;
    - the 0xFE prefill, with the feed-forward offset moved to `d/4`: "a trace slot the core never wrote".

## Measurements

**Per model,** over the first 32 DEV windows (996 scored rows each). P, B and L read the root mask. The two models that claude-132 found faithful are d11-c130 s1 and dual-c131 s1.

| Model | Rows differ (R) | P | B | Lift | L | Primaries (windows) | Largest primary margin (windows with rows differ) | Upper median | Root disagreements (of 8,257,536 sites) | Worst site's disagreement |
|---|---|---|---|---|---|---|---|---|---|---|
| plain-c126 s1 | 51 (5.1%) | 100% | 100% | 0 | 21/21 | 121 (21) | 5.4e-5 | 5.4e-5 | 47,230 (0.57%) | 2.3% |
| plain-c126 s2 | 34 (3.4%) | 100% | 100% | 0 | 21/21 | 79 (21) | 4.1e-5 | 1.7e-6 | 17,376 (0.21%) | 0.7% |
| d11-c130 s1 | 25 (2.5%) | 100% | 100% | 0 | 13/13 | 13 (13) | 4.2e-5 | 4.2e-5 | 27,101 (0.33%) | 1.1% |
| d11-c130 s2 | 54 (5.4%) | 100% | 99.2% | 0.8 | 17/17 | 30 (17) | 1.7e-4 | 2.6e-5 | 26,781 (0.32%) | 1.4% |
| dual-c131 s1 | 23 (2.3%) | 100% | 99.9% | 0.1 | 15/15 | 22 (15) | 1.3e-4 | 6.2e-5 | 14,100 (0.17%) | 0.7% |
| dual-c131 s2 | 37 (3.7%) | 100% | 100% | 0 | 18/18 | 18 (18) | 2.2e-5 | 2.2e-5 | 24,560 (0.30%) | 1.5% |

**Readings, with no bar:**
- **Every primary on every model is a near-tie.**
  - In the windows where D and A differ, the largest primary margin is 1.7e-4.
  - Over all 32 windows it is 6.8e-4, on dual-c131 s1.
  - All are below 1e-3. None is an exact tie, and none is a zero block.
  - The two models whose first flips come after the prefix (d11-c130 s2 and dual-c131 s1) read the same.
- **Later disagreements are decisive.** Over all root disagreements, the upper median margin is 0.009 to 0.020, and 6,174 to 18,602 per model are below 1e-2. That fits a state that has already parted, and makes the first flip the only place a near-tie reading means anything.
- **No zero blocks at all,** on either side, on any model. So the zero convention, root 1 against −1, never arises on this panel.
- **The faithful models read like the others.** d11-c130 s1 and dual-c131 s1 have the same near-tie primaries, flips in every window and saturated B. So how a parting starts does not separate the faithful models from the indeterminate ones.
- **Inert transitions at position 0** number 120 on plain-c126 s1, 1 on dual-c131 s2, and 0 on the rest.

**The second build reproduced the first byte for byte.** It ran at `d75b525e4`, with every review fix, on the same pod. For all six models, the trace bytes, the tokens and every reading are identical. The comparison leaves out only the build hashes, `i_layout`, and the new below-1e-3 count. `same-kernel` confirmed the PTX and cubin equal the qualified ones.

**Every instrument check was green:**
- **`I-GPU-Tests`:** the float-train CUDA library passed 69 tests, with 5 ignored, the snap-capture GPU test included. The instrument's parity tests passed 4, and passed 4 again with the prefill in the second build.
- **`I-Qualify`:**
  - `qualify` passed.
  - `run --verify` on plain-c126 s1 gave G = H and G = R at all 82,176 positions.
  - **The new pins:** PTX `5a947f09285a359dfb0ddfdf4bdf31746b959ef39f6056807eead9b4672f87ef`, cubin `2a386d8c563516aeef10d040200867330cafb54d5525cdd7580b7969554a4c86`.
- **`I-Trace`:** outputs and trace G = H on all six, in both builds.
- **`I-Old-Run`:** the traced tokens equal the stored, qualified runs on all six.
- **`I-Inputs` and `I-Fetch`:** every input against its pin and its folder's `SHA256SUMS`.
- **`I-Capture`:** passed.
- **`I-Layout`** (second build): passed on all six. Each site's snaps agree at least 97.7% between D's capture and A's trace, where a misplaced site would agree on about 1 in 120.
- **Finite margins:** all finite.
- **`I-Finite`:** every step exited 0. The 282 data files in the five folders verify against their folders' `SHA256SUMS`, on the pod and on the laptop.

**Evidence:** the HF store at `results/claude/d11-fbs-snap-trace-2026-10-26/`, each folder with a `SHA256SUMS`:
- `snap/`: the first job, holding every step's output, `qualify.json`, the verify run, the six traces, the six receipts and `job.log`;
- `snap3/`: the second build, the same layout;
- `snap2/`: the stopped attempt;
- `snap-bin/` and `snap2-bin/`: the binaries run. `spiralcore-fbs-sheet` is `7f405aeb…` and then `9b2de6d4…`; `d11-fbs-eval` is `99aadfbe…` and then `258c8ba0…`.
- `windows/`: the 32-window file, `d5f8ef23…`.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** The rule was applied as fixed. The sheet's B ≥ 80% clause, written before the run, decides the outcome. It is not reread after the fact as "near-tie".
- **writing-meaningful-tests.** Each new reading has a test that fails with its fix set aside: the inert exclusion, the zero lift, and the 0xFE prefill against a moved offset.

## Still open

- **The decision review that Indeterminate calls for has passed.** It is recorded on #2820 as an owner log. It confirms that the classification stands as pre-registered and that the item does not park. The next unit is an intervention, not a placement, and it gets its own sheet, FIT and D67 review:
  - **Force D to A's traced snap choices on the full panel:** all 428 DEV windows re-traced (about 14 s) and all 14,287 rows. A 32-window trace cannot reclassify a 14,287-row residual.
  - **Would-flips.** At every forced site, record D's own choice and its margin. With the states held together, each site where D would choose another root is a flip on its own, not a carried one. That sidesteps the base-rate and shared-prefix limits here.
  - **Instrument gates:**
    - D forced to its own captured choices reproduces D exactly;
    - forcing a wrong root changes the tokens, as a negative gate.
  - **Numeric bars fixed before the run,** with an intermediate outcome between "forced-D ≈ A" and "a few percent remain". Each outcome either ends the investigation or names exactly one more control.
  - **What clears the exporter:**
    - forced-D ≈ A shows the residual is carried by snap choices, but not that A's choices come from rounding;
    - the exporter is settled faithful only if forced-D ≈ A, the would-flips are near-ties (m < 1e-3), and their rate is inside the faithful models' own.
  - **Wording.** If the exporter is cleared, the three FAILs stand as configuration results, never a mechanism result (§4.1).
- **The pivot note's order is unchanged:** the investigation settles, then the decisive run. That run goes ahead under Stage 4b's unchanged bar unless Casey reads §3.7 otherwise (#2028), so nothing waits on the owner (§5.1).
- **Code-review notes on `I-Layout`, both MINOR:**
  - a zero block on both sides counts against it, though it is evidence for the layout. No zero blocks occurred here.
  - its host test has one layer and as many lanes as feed-forward blocks, so it would not catch a check that read only layer 0, or `lanes` for the feed-forward site.
- **A future window panel should differ from position 0,** or should read only flips after the shared prefix.

## Plain summary

- **What it looked at:** the exact points where the float reference and the shipped integer file first part ways.
- **Where they part:** every time, at a rounding choice that sat almost exactly on a knife-edge between two options. That is within about one part in 1,500 everywhere, and one part in 6,000 in the texts whose words differ. After that, the two sides drift apart as expected.
- **What it could not show:** that those knife-edge choices cause the word-level differences.
  - Such choices happen early in every text, so they come before both the differing and the matching words.
  - Most of the texts tested begin with the same four words, so many "first partings" were really one event.
- **The two models that had already passed look the same,** so this reading does not separate them from the four uncertain ones.
- **The next step** forces the reference to make the integer file's choices, and sees whether the differences disappear. At each choice it also records what the reference would have picked on its own.
