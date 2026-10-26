# REPORT: spiralcore-fbs-offline-float-sheet, claude-119

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md) (antigravity-81, FIT in #3233). Record: [lab record](../../../../labs/spiralcore-fbs-offline-float-sheet/README.md).

Closes #2891.

## What did not work, first

- **`G-Fbs-Sheet-Fit` failed on both seeds, in both builds.**
  - **Top-1 against frozen routing.** Learned SpiralCore/FBS routing loses to frozen random routing (`Fbs_FrozenQueries_Null`) on paired float DEV top-1 in all 8 comparisons:

    | Arm | A s1 | A s2 | B s1 | B s2 |
    |---|---:|---:|---:|---:|
    | `Fbs_1Atom_Active` | −3,993 | −1,851 | −4,920 | −2,267 |
    | `Fbs_2Atom_Mix` | −4,334 | −3,000 | −5,098 | −2,685 |

  - **Loss against frozen routing.**
    - On seed 1, `Fbs_1Atom_Active` has lower DEV loss than the frozen null, but only inside the noise band: 0.0557 against a band of 0.0686 in A, and 0.0423 against 0.0597 in B.
    - On seed 2 it has higher loss.
  - **Loss across arms.** The identity-transport control has the lowest float DEV loss in all four seed-build cells (1.8030–1.8290).
  - **Verdict.** A negative for this configuration only: 6 layers, 1,024 steps, `StackAdamW` at the Hamilton sheet's rates (CHARTER §4.1). SpiralCore/FBS stays ACTIVE in its integer form and READY in float.
- **`G-Fbs-ExportGap` failed in both builds, on its top-1 part.**
  - **The loss part passes, but it measures only the gate snap.** The one-atom arm's export-minus-float DEV loss gap is 0.0080–0.0264, inside the band. The export-mode loss snaps the Hamilton gates and does not round the writer table or hidden state to Q12; only the export-mode top-1 does.
  - **The top-1 part fails in build A (proof).** The top-1 gaps are 184 and 481 rows. The hit counts allow F_I of at most 117 and 166, since b + c cannot exceed float hits plus integer hits.
  - **Build B records a FAIL, but which seed failed can't be recomputed.** Its net gaps are 25 and 79 rows. The receipt does not store the export gap's b and c. That is a receipt gap, not a gate gap.
- **Trained-query arms do not reproduce between two byte-identical builds.**
  - Both builds use the same binary, `bb88f774…`, and the same seeds, yet the results hashes differ (`d6b2d993…` against `ee0fd4aa…`).
  - Trained-query top-1 moves by up to 2,212 rows. For example, the identity control on seed 2 scores 2,533 against 321. The frozen null moves by at most 280.
  - The one-atom-versus-identity top-1 comparison changes sign between builds, so this run gives no transport verdict on top-1.
  - **Hypothesis.** Atomically accumulated adjoints flip hard root choices. The step-0 evaluation matches exactly between builds, but the step-0 query credit already differs at about 1e-7 relative.
- **Late collapse on seed 1 in build A (measurement; the cause is a hypothesis).** In build A, the trained-query arms' top-1 peaks at step 512 and falls by step 1,024 while loss falls. For example, the identity control goes from 1,548 to 152 top-1, while its admitted hits peak at step 256 (2,250) and end at 256. In build B its seed-1 top-1 keeps rising instead (445 → 1,457 → 1,500), part of the run-to-run spread above. Antigravity's recurrence sheet showed the same pattern.
- **The exact-head review asked for changes. A fix round followed, and no recorded verdict changed.**
  - **Slice 1, the kernels:** APPROVE.
  - **Slice 2, the hook and stack:**
    - The two-atom tests could not tell the paths' query slots apart. Added a distinct-paths GPU test.
    - `with_gate_mode` left its flags changed. It now restores them.
    - The identity arm's query credit has no exact reference test. That is still open.
  - **Slices 3a and 3b, the runner:**
    - I-Export-Exactness was narrower than its docstring said. It now states the one-atom, random-query coverage.
    - The training timer did not synchronize the GPU. That overstates training tok/s by at most 3 of 1,024 steps' work (the steps just before the evaluations at 256, 512 and 1,024); the timer now synchronizes.
    - The export-mode loss is gate-snap only. That is now stated.
    - The gate logic had no tests. It moved into `evaluate_gates`, with unit tests.
    - The receipt lacked the export gap's b and c. They are now recorded.
    - A transfer-branch error could discard every DEV result. Any error there is now recorded as blocked.
    - A capacity winner that failed the fit could be exported. A lone fitting arm is now the one exported.
    - A zero noise band let a tie count as a win. A loss win now needs a strict gap.
    - `write_receipt` could race with a concurrent writer. It now creates the file exclusively.
  - **The claims check found four wrong claims, all corrected here:**
    - step-0 query credit called identical;
    - the identity control's admitted peak;
    - a `job.log` checksum sentence;
    - evaluation speed ranges that mixed in warm-up points.
- **The ORDER's last D68 condition is only partly met.** `I-Export-Exactness` checks the one-atom table against the pinned crate's integer signed-axis table (`geometry.axes`). It does not check against the integer kernel in `crates/uor-r4-discrete`, which is outside this item's scope.
- **`G-Fbs-Transfer` was not reached,** because the fit and export bars failed. Separately, the sealed `fresh_v2` draw is not in the `w2-inputs/2026-10-25/driver-data/` store bundle. The runner records a missing draw as blocked, never as a negative.
- **Attempt 1 stopped at the GPU tests (instrument fault, repaired).** The frozen-arm test demanded bit equality of an input adjoint that is summed with device atomics, and the values differed only in the last bit. The forward was bit-identical, and the other three arm tests passed. `ca057242` compares with the 1e-5 relative tolerance the two-path test already used. Attempt 1's logs are in the store under `attempt1/`.
- **The notebook row is staged, not appended.**
  - The installed `~/.cargo/bin/uor-harness` was built at 2026-10-25T01:10Z, before #3194. Its guard therefore refused `docs/research/notebook/float-training.md`, which docs/loop/README.md lists as always allowed.
  - The row is in [NOTEBOOK-ROW.md](NOTEBOOK-ROW.md), ready to append.
  - It is logged in `local/claude/FRICTION.md`, with one note for Casey on #2028.

## What changed

- **Code (about 1,600 lines, `crates/uor-r4-float-train`):**
  - **Arm kernels.** `src/spiralcore_fbs_arms.cu` and `src/spiralcore_fbs_arms_prelude.cu`, built as their own nvcc PTX module (`spiralcore_fbs_arms_f32.ptx`, `build.rs`). They provide identity transport, and two paths combined with a constant 1/√2.
  - **Arm hook.** `src/spiralcore_fbs_arms.rs`: the arm hook and the projection op. The frozen arm uses the standard kernel with `dq = 0`.
  - **Stack changes.** `StackModel::linear` dispatches through the hook. `widen_fbs_queries_two_path` builds the two-path queries.
  - **Runner.** `src/spiralcore_fbs_sheet.rs` and the bin `spiralcore-fbs-sheet`, with modes `synthetic-probe`, `throughput-probe` and `full`.
  - **Gate wrapper.** `hamilton_sheet::with_gate_mode`.
- **The D68 conditions held.**
  - The three §5-pinned files are byte-identical, and were re-hashed on the pod.
  - The receipt records the arm kernel, prelude and PTX sha256.
  - There are no new dependencies and no softmax, squashing or matmul. The kernels are built ahead of time by nvcc.
- **Tests (about 560 lines).**
  - Four `#[ignore]`d CUDA tests pin each arm against a reference or an exact invariant: identity selection, √2 × standard on equal paths, frozen credit 0 with the standard input adjoint, and adjoint-transpose on every arm.
  - The review round added a fifth GPU test: distinct paths give the 1/√2 sum of two standard projections, with each path's own query credit.
  - The host tests cover arm names and codes, the per-thread hook, the kernel inventory, F_I, the schedule, the pins and synthetic batches.
  - Six gate tests cover `evaluate_gates`, extended after round 2 of review. Each of nine mutations turns at least one test red:
    - a reversed paired count;
    - a non-strict tie at a zero band;
    - the band dropped from the capacity rule;
    - an unconditional two-path capacity winner;
    - a band taken from one arm;
    - the last seed alone deciding the fit;
    - no descent check;
    - the last seed alone deciding the export gap;
    - no absolute value on the export loss gap.

    The restored suite passes.
- **Docs:**
  - the lab record and four receipts;
  - the staged notebook row;
  - a dated row and updated open questions on `docs/mechanisms/spiralcore-fbs.md`.
- **Generated:**
  - receipts of about 3,690 JSON lines;
  - the roadmap item set to `status: done`, and `docs/roadmap/README.md` re-rendered.
- **Size against estimate.** The ORDER gave no line estimate. It anticipated that the build might fill one PR. It did not, and the run landed in this unit.

## Measurements

All of these come from one RTX 5090: GPU 1 of `vast-57710022`, `sm_120`, driver 595.80, nvcc 12.8.93. They were produced by `/workspace/fbs-job.sh ca0572423f472c9f2b01d60a4f50cb17261281e9` (local copy `local/claude/fbs-job.sh`, sha256 `f2231b40…`). That job runs, in order:
1. the pins;
2. the store inputs;
3. build A, then `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --include-ignored spiralcore_fbs`;
4. `spiralcore-fbs-sheet --mode synthetic-probe`, then `--mode throughput-probe`, then `--mode full` on build A;
5. a clean build B, then `--mode full` again.

| Measurement | Value |
|---|---|
| Training (batch 4 × context 192, 1,024 steps; timer unsynchronized, see above) | 20,643–23,929 tok/s |
| Float DEV evaluation at step 1,024 (14,287 targets, 82,176 tokens) | 127,123–134,225 tok/s |
| Integer-export-mode DEV evaluation at step 1,024 | 13,886–14,531 tok/s |
| Greedy decode (batch 1, 128-token prefix, 64 tokens; full recompute per token, a lower bound) | 109.1–139.5 tok/s |
| Float DEV loss, step 0 → step 1,024 | 2.6487–2.8450 → 1.8030–1.9250, every arm |
| Route mobility (trained-query arms) | 5.46%–7.95%; last-step churn ≤ 0.0004 |
| Noise band (largest two-seed loss difference; the frozen null in both builds) | 0.0686 (A), 0.0597 (B) |
| GPU tests matching `spiralcore_fbs`, at `ca057242` | 18 of 18 pass |
| Arm GPU tests after the review round, at `abe8102a` | 5 of 5 pass; with set-aside 1 (no arm dispatch) 5 of 5 fail; with set-aside 2 (path credit swapped) only the distinct-paths test fails |
| `I-Export-Exactness` | 48 of 48 lawful blocks; 0 of 3,072 integer mismatches |
| Q12 `i32` writer oracle | 0 mismatches on 517 rows × 32 evaluations per build |
| GPU time | about 40 min on one leased GPU of antigravity's pod (34 for the sheet, 6 for the review round's tests); about $0.72 |

The receipts are [`receipt.json`](../../../../labs/spiralcore-fbs-offline-float-sheet/receipt.json) (`17c5563b74f3ccd9400b5a05ae042f844d2f0e0af2f0b6ee17acc84e95edb789`) and [`receipt-build-b.json`](../../../../labs/spiralcore-fbs-offline-float-sheet/receipt-build-b.json) (`d3cf226db091006a79226fdeb5529ae11206391998328e482dd02a0c56e18443`). The store copy is `caseyallard/uor-r4-store:results/claude/spiralcore-fbs-offline-float-sheet-2026-10-25/`, with top-level `SHA256SUMS` `17939779656db70ae40409cf06ea64a705ffd0ebe94d6822747b3dc83c759320`, which covers every file as stored. The review round's GPU tests and set-aside patches are under `review-round-1/` there (`SHA256SUMS` `66885d5fa0a258f73cb8f3492001fa7709719cf260e6ac6e9c520277eebdc613`), run by `local/claude/fbs-tests-job.sh` (sha256 `6402b13a…`). The pod-written `attempt2/SHA256SUMS` was computed before the job appended its closing `DONE` line to `attempt2/job.log`, so it lists an earlier sum for that one file.

**Open choices, labelled:**
- **The integer-export mode is the Hamilton sheet's D11-compatible evaluation.**
  - It snaps the Hamilton gates to the 120 icosians.
  - Its top-1 evaluation also rounds the writer table and the scored hidden state to Q12; its loss does not.
  - The rest of the stack stays float, and the two-atom arm's 1/√2 is applied in float.
- **`I-Export-Exactness` covers the one-atom forward only,** with random untrained queries, against the pinned crate's integer signed-axis table. The writer half of the sheet's I-4 is the pure-`i32` writer oracle inside each export-mode evaluation.
- **The exported arm departs from the sheet's wording.** From `abe8102a`, the export gap and transfer score the one arm that passes the fit when exactly one does; otherwise they score the capacity winner. The sheet says "the winning active arm". The receipt names the arm as `g_export_gap_arm`, and this run, where no arm fit, is unaffected.
- The Hamilton sheet's learning rates, weight decay and clip are reused.
- Training uses 1,024 batches.
- Route mobility is measured as the change in each query block's nearest orbit root.
- The paired fit comparisons use float predictions.

## Checks run at the head

Filled in by the exact-head review record: `loop report --check`, `loop pr-check --local`, the host tests, the stub-nvcc `--features cuda` clippy, and CI.

## Skills used

- **holding-the-goal.** The frozen-arm test failure and the stale guard were each read before acting. The first was classed an instrument fault, and the second as the guard working from an old binary. Neither was routed around.

## Still open

- **Append the staged notebook row** once the shared harness binary is current (FRICTION line, #2028 note).
- **Make the FBS backward deterministic** (an ordered reduction in place of atomics) before any further float comparison of trained routes. This is a hypothesis, not yet tested.
- **Test whether the margin-hinge loss and the writer's admission come apart late in training.** This is a hypothesis.
- **Add `fresh_v2` to the `w2-inputs` store bundle** before any sheet that reaches its transfer gate.
- **An exact reference test for the identity arm's query credit** (slice 2's recommended finding). Its forward and adjoint are pinned; its credit is checked only as finite and nonzero.
- **Check the float form's integer export against `crates/uor-r4-discrete`'s kernel** (the partly met D68 condition), in an item whose scope includes that crate.
- **Include the gate verdicts and paired counts in `results_sha256`** (a review nit). Today the digest covers losses and prediction hashes only.

## Plain summary

SpiralCore/FBS is the project's replacement for matrix multiplication in a layer's projections. Each output block picks one 8-number input block by walking a small tree of fixed geometric points, then flips and permutes its entries. This unit trained a small language model that uses it, in ordinary floating point and offline, as a pre-registered test allows. It compared four versions:
- the standard one;
- one that mixes two picked blocks;
- one that picks but never flips or permutes;
- one whose picking is frozen at random.

Every version learned: its error on held-out text fell. Training ran at about 21,000–24,000 tokens per second on one gaming-class GPU. But learning where to pick did not beat the frozen random picks on the main accuracy count, and rounding the model to integers moved that count too much.

There is a deeper problem. Two identical builds of the same program gave noticeably different results whenever the picks were being learned. That fits tiny rounding differences in the GPU's order of addition flipping picks. So the next step is to make that addition order fixed before comparing learned picks again. This closes this one training setup, not the idea.
