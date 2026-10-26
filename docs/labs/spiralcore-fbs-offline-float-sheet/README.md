# Lab record: `spiralcore-fbs-offline-float-sheet` (`#2891`)

- **Date:** 2026-10-26 (UTC). **Lab:** `claude` (`claude-119`).
- **Sheet:** [`spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md), FIT in #3233. The pre-run review and D68 decision review are on #2891.
- **Runner:** `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs`, with bin `spiralcore-fbs-sheet`. The arm kernels are `src/spiralcore_fbs_arms.cu` (own PTX module `spiralcore_fbs_arms_f32.ptx`).
- **Source built on the pod:** `ca0572423f472c9f2b01d60a4f50cb17261281e9`.
- **GPU:** GPU 1 of `vast-57710022`, which is antigravity's 2 × RTX 5090 pod (`sm_120`, driver `595.80`, `nvcc 12.8.93`). Claude leased only GPU 1: from about 00:50 to 01:24Z for the sheet, and about 01:44 to 01:50Z for the review round's GPU tests. That is about 40 minutes, or about $0.72 at half the pod's $2.171/h.
- **Binary:** builds A and B are byte-identical, `bb88f774cc225e002c1448c6fb36eeb8f468c1ee254a7df95d02764e682bd139`.
- **Pins re-hashed on the pod:**
  - `spiralcore_fbs.cu` `517bffcc…`, `spiralcore_fbs.rs` `6eb2b09d…` and `cuda_stack.cu` `c577fce9…`; all match the sheet.
  - The arm kernel source is `43c680c67efb4106d73f035bdd9282bd09dcd160961190bec984c023b3a51f7f`.
  - The arm prelude is `6d3e6ff6f8eb483d66419a8cbe06a64de406981605835ff92b3824680405fcd7`.
  - The arm PTX is `415e0d85ea1285d27c8c8a1087048baf06f7e8f96f8bc5080dbc4a62a6b4839f`.
- **Receipts:**
  - [`receipt.json`](./receipt.json), build A: `17c5563b74f3ccd9400b5a05ae042f844d2f0e0af2f0b6ee17acc84e95edb789`.
  - [`receipt-build-b.json`](./receipt-build-b.json), build B: `d3cf226db091006a79226fdeb5529ae11206391998328e482dd02a0c56e18443`.
  - [`synthetic-probe.json`](./synthetic-probe.json): `ffff7ba3…`. [`throughput-probe.json`](./throughput-probe.json): `3780f9b7…`.
  - Store: `caseyallard/uor-r4-store:results/claude/spiralcore-fbs-offline-float-sheet-2026-10-25/`, whose `SHA256SUMS` is `17939779656db70ae40409cf06ea64a705ffd0ebe94d6822747b3dc83c759320` (24 files, both attempts and both binaries).

All numbers below are **measurements** from these receipts unless they are labelled **proof** or **hypothesis**. "Top-1" counts hits out of the 14,287 eligible `ft/dev` targets.

## Lead metric: tokens per second (one RTX 5090, context 192)

| What | Range over 8 arm-seeds × 2 builds |
|---|---|
| Training, batch 4 × 192, 1,024 steps (786,432 tokens) | 20,643–23,929 tok/s (33–38 s per arm-seed); the two-atom arm is slowest |
| `ft/dev` context evaluation at step 1,024, float | 127,123–134,225 tok/s (82,176 tokens in 428 windows) |
| `ft/dev` evaluation at step 1,024, integer-export mode | 13,886–14,531 tok/s |
| Greedy decode, batch 1, 128-token prefix, 64 new tokens | 109.1–139.5 tok/s |

- **Decode is a lower bound.** Each new token recomputes the whole prefix, because there is no incremental state, and it pays one host read-back. It is not a serving speed.
- **Evaluation at the other steps.** Over all 64 evaluation points, steps 0–512 included, the float evaluation range widens to 110,864–134,857 tok/s; the low end is a step-0 point. The export mode widens to 13,815–14,531 tok/s.
- **The training timer did not wait for the GPU** after each step's update. The next step's loss read-back waits for it, so only the 3 steps just before an evaluation (at 256, 512 and 1,024, the last step included) leave queued work outside the timer. Training tok/s is therefore overstated by at most 3 steps' worth in 1,024. The head now synchronizes before reading the clock.

## Outcomes, negatives first

1. **`G-Fbs-Sheet-Fit` = FAIL on both seeds in both builds.**
   - **Top-1 against the frozen null.** Learned routing loses to frozen random routing on paired float top-1 in all 8 comparisons. That is 2 trained arms × 2 seeds × 2 builds. Net advantages:

     | Arm | A s1 | A s2 | B s1 | B s2 |
     |---|---:|---:|---:|---:|
     | `Fbs_1Atom_Active` | −3,993 | −1,851 | −4,920 | −2,267 |
     | `Fbs_2Atom_Mix` | −4,334 | −3,000 | −5,098 | −2,685 |

   - **Loss against the frozen null.**
     - On seed 1, `Fbs_1Atom_Active` has lower DEV loss than `Fbs_FrozenQueries_Null`: 1.8586 against 1.9143 in A, and 1.8677 against 1.9099 in B. Both margins are inside the noise band (0.0557 < 0.0686 in A; 0.0423 < 0.0597 in B).
     - On seed 2, it is worse: 1.8882 against 1.8457 in A, and 1.9105 against 1.8502 in B.
   - **The identity-transport control has the lowest float DEV loss in all four cells:** 1.8159, 1.8093, 1.8290 and 1.8030.
   - This closes the 6-layer, 1,024-step `StackAdamW` float configuration with the Hamilton sheet's learning rates (CHARTER §4.1). It does not close the mechanism. SpiralCore/FBS stays ACTIVE on the discrete path and READY in float.
2. **`G-Fbs-ExportGap` = FAIL in both builds, on top-1.** The loss part passes, so only the paired top-1 part fails.
   - **The loss part passes, but it measures less than its name says.** For the retained `Fbs_1Atom_Active`, the export-minus-float DEV loss gap is 0.0080 and 0.0264 in A, and 0.0106 and 0.0168 in B, all inside the noise band. The export-mode loss snaps the Hamilton gates only: unlike the export-mode top-1, it does not round the writer table or the scored hidden state to Q12. So it measures the gate snap alone.
   - **The top-1 part fails in build A on both seeds (proof from the recorded hit counts).** The bar is |b − c| ≤ F_I with F_I = max(24, ⌈2√(b+c)⌉), where b and c are the float-versus-integer discordant hit rows. Since b + c ≤ float hits + integer hits:
     - seed 1: b + c ≤ 1,618 + 1,802, so F_I ≤ 117, but the net gap is 184 rows;
     - seed 2: b + c ≤ 3,653 + 3,172, so F_I ≤ 166, but the net gap is 481 rows.
   - **Build B records FAIL, but the failing seed can't be identified from the receipt.** Its net gaps are 25 and 79 rows. The receipt does not record the export-gap b and c, so which seed failed cannot be recomputed from it. That is a gap in the receipt, not in the gate, which computed them. The head now writes them to `export_paired`.
   - **The frozen null's own export loss gap is much larger:** 0.159–0.246.
3. **The top-1 rows are not reproducible between builds for arms that train their queries.** The binary is the same and the seed is the same, yet the results hashes differ (A `d6b2d993…`, B `ee0fd4aa…`).
   - **Trained-query arms.** `Fbs_1Atom_Active` seed 1 scores 1,618 top-1 in A and 411 in B. The identity-transport control scores 152 and 1,500 on seed 1, and 2,533 and 321 on seed 2.
   - **The sign of the transport comparison flips between builds.** `Fbs_1Atom_Active` beats the identity control on paired top-1 by +1,466 and +1,120 rows in A. In B it scores −1,089 and +2,971. So this run gives **no** stable transport-versus-identity verdict on top-1.
   - **The frozen null reproduces closely.** Top-1 is 5,611 against 5,331 and 5,504 against 5,559; DEV loss moves by 0.0044 and 0.0046.
   - **Loss of the trained arms moves by 0.006–0.099 between builds.** The largest move is `Fbs_2Atom_Mix` seed 2, 1.8265 against 1.9250. That exceeds the two-seed noise band, so the sheet's noise band understates run-to-run spread.
   - **Step 0 is not fully identical between builds.** The step-0 evaluation (losses and hits) matches exactly. The step-0 query credit already differs, at about 1e-7 relative: `Fbs_1Atom_Active` seed 1 is 0.531432228 in A and 0.531432304 in B.
   - **Hypothesis for the source.** The backward accumulates adjoints with device atomics, whose order varies between launches. A last-bit difference can flip a nearest-root choice, and hard routing then turns it into a different trajectory. The frozen arm never moves a route.
4. **Late collapse on seed 1.** In build A, every trained-query arm's top-1 rises to step 512, then falls by step 1,024, while DEV loss keeps falling:
   - `Fbs_1Atom_Active`: top-1 2,338 → 1,618; admitted-target hits 3,465 → 1,802.
   - `Fbs_2Atom_Mix`: top-1 2,520 → 1,277.
   - Identity-transport control: top-1 1,548 → 152. Its admitted hits peak earlier, at 2,250 at step 256, then fall to 2,118 and then 256.

   The frozen null's admitted hits rise throughout, 6,437 → 7,740. Build B does not repeat this exactly: its identity control's seed-1 top-1 keeps rising (445 → 1,457 → 1,500), another sign of the run-to-run spread in item 3. Antigravity's recurrence sheet showed the same signature: S³ seed 1 reached 352 top-1 despite loss descent ([notebook row](../../research/notebook/float-training.md), [record](../recurrence-gates-six-recurrence-fit/README.md)). **Hypothesis:** the margin-hinge objective lets the loss fall while the writer's cell admission degrades.
5. **`G-Fbs-Capacity`: one atom is retained.** Two-path mixing would need lower loss by the full noise band on both seeds. It is lower by 0.0099 and 0.0617 in A (band 0.0686). In B it is higher by 0.0031 and 0.0146. The two-atom arm costs about 8% of training speed.
6. **`G-Fbs-Transfer` was not reached.** It runs only after `G-Fbs-Sheet-Fit` and `G-Fbs-ExportGap` pass. Separately, the sealed `fresh_v2` draw is not in the `w2-inputs/2026-10-25/driver-data/` store bundle. The runner would have recorded that as blocked, never as a negative.

## What passed

- **Descent (the item's done-when).** Every arm's float DEV loss falls from its step-0 value (2.6487–2.8450) to 1.8030–1.9250 at step 1,024, in both builds. All arm-seeds, the frozen null included, rise from 0–5 top-1 hits at step 0 to 152–5,611 at step 1,024.
- **`I-Finite-CUDA`.** No error. The active arms have nonzero step-0 query credit: L2 0.3784–0.7065, against exactly 0 for the frozen null.
- **`I-DEV-Denominator`.** 14,287 of 14,287 rows are scored, in 400 documents and 428 windows, in both modes at every evaluation point.
- **`I-Route-Mobility`.** In the trained-query arms, 5.46%–7.95% of query blocks change nearest root over 1,024 steps; the bar is ≥ 1%. Churn at the last step is ≤ 0.0004 (bar < 0.5). The frozen null's mobility is 0.
- **`I-Export-Exactness`, for the one-atom forward only.**
  - The plan is a synthetic one: 32 inputs, 48 outputs, random untrained queries.
  - Its route reads back from a coded-carrier forward as one atom and one lawful Cl(0,6) signed-axis state per block: 48 of 48, with 0 mixed and 0 unlawful.
  - On 8 integer root-aligned carrier rows, the table evaluated in `i32` reproduces the F32 kernel: 0 mismatches in 3,072 comparisons.
  - Not covered: the identity and two-atom arms' integer forms, and the writer. The capacity gate kept one atom, so the exported arm is the one checked.
- **The Q12 `i32` writer oracle**, the writer half of the sheet's I-4. A pure-`i32` writer reproduces the GPU writer's predictions and top-1 count on 517 rows at each of 32 export-mode evaluations per build, with 0 mismatches.
- **The finite GPU tests on the pod:** 18 of 18 tests matching `spiralcore_fbs` pass at `ca057242`, the four arm tests included.
- **The arm tests after the review round,** at `abe8102a`, on the same GPU (store `review-round-1/`, `SHA256SUMS` `66885d5f…`):
  - **At head:** all 5 arm tests pass, including the new distinct-paths test.
  - **Set-aside 1, every arm replaced by the standard projection:** all 5 fail.
    - The identity and frozen tests fail on their own assertions: axes moved, and frozen queries received credit.
    - The three tests that use two-path queries fail because the standard plan refuses their widened shape. So for the two-atom forward and its 1/√2, the evidence is set-aside 2 plus the tests' expected relations, not set-aside 1.
    - The transpose property also holds for the standard projection, so that test pins the adjoint's correctness, not which arm is running.
  - **Set-aside 2, the two-atom backward writing each path's credit into the other path's slot:** only the distinct-paths test fails. The equal-paths test cannot see the swap.

## Open choices, labelled

- **The integer-export evaluation is the Hamilton sheet's D11-compatible mode.**
  - What it rounds: the Hamilton gates are snapped to the 120 icosians, and the writer table and final hidden state are rounded to Q12 `i32`.
  - What still runs in float: the stack itself.
  - The FBS projections are a gather and a sign, which is exact on integer inputs. The two-atom arm's 1/√2 is applied in float.
  - It is not an integer table export of the whole stack.
- **The ORDER's last D68 condition is only partly met.** It asks for the Q12 oracle to be checked against the existing integer SpiralCore/FBS form. `I-Export-Exactness` checks against the pinned crate's integer signed-axis table (`geometry.axes`), not against the kernel in `crates/uor-r4-discrete`, which is outside this item's scope. The two-atom arm's integer form, a sum with a shift, was not exercised.
- **The training settings are the Hamilton sheet's, reused:** base rate 0.002, decay-rate multiplier 20, weight decay 0.1. Training runs 1,024 steps, one per `mw-balp/train` batch, with 1,024 batches loaded (`train_batches` in the receipt).
- **Route mobility uses the nearest orbit root of each query block.** It is computed on the GPU over the 16 anchor roots.
- **The exported arm, after the review round, departs from the sheet's wording.**
  - The sheet scores the export gap for "the winning active arm".
  - From `abe8102a`, the runner exports the one arm that passes `G-Fbs-Sheet-Fit` when exactly one passes. Otherwise it exports the capacity winner. This keeps an arm that failed the fit out of export and transfer.
  - The receipt names the arm as `g_export_gap_arm`.
  - It does not change this run, where no arm passed the fit.
- **The noise band follows the sheet.** It is the largest two-seed loss difference over arms, which is the frozen null in both builds.
- **The paired fit comparisons use float predictions.**
- **Attempt 1** (source `dc8d2a56`) stopped at the GPU tests. The frozen-arm test required bit equality of an atomically accumulated input adjoint and failed only in the last bit. That is an instrument fault in the test, repaired in `ca057242` with a 1e-5 relative tolerance. Its logs are under `attempt1/` in the store.

## What this decides and unlocks

- **Decided:**
  - At this rung and step count, learned SpiralCore/FBS query routing does not beat frozen random routing on DEV top-1.
  - One atom per block is enough against two.
  - Post-training rounding of this checkpoint does not meet the top-1 export bar.
- **Next useful work (hypotheses to test, not results):**
  - make the FBS backward deterministic, for example an ordered reduction in place of atomics, before any further float comparison of trained routes;
  - test whether the margin-hinge loss and the writer's admission come apart late in training;
  - try a root-snapped (QAT) forward before export.
