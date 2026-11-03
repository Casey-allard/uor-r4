# REPORT: one-model-end-to-end, claude-262 (training as served learns again)

Lab: claude. [ORDER](./ORDER.md). PR #3587, item #3407.
[Prep sheet](../../../../harness/prep/one-model-end-to-end-served-repair-2026-11-03.md).

**In one paragraph.** Training as served (`--train-as-served 1`) learns again. Its step costs 1.85×
plain, against 6.83–6.96× before.

Two faults, both in `hamilton_sheet.rs`, made the gap:
- The float stand-in for the integer activation requantizer left out the step's exponent, so every
  activation below 2^15 was rounded to a whole number.
- The D11 icosian snap scored each 4-block with a candle `sum` that launched one GPU block per score:
  35.4 million for one batch-16 MLP gate.

Both are fixed, each pinned by a CUDA test. deepseek-5's FIT'd sheet was rerun at the fixed head on
two seeds.
- *Instrument door:* passed. Both served checkpoints are byte-identical through export rounding.
- *Operational door:* passed, at 1.79× and 1.84× plain wall time.
- *Quality smoke:* inconclusive, with served above plain on both seeds. Served-evaluator top-1 was
  53 / 67 against 40 / 43, and DEV nearest 305 / 357 against 239 / 248. deepseek-5's served run had
  23 / 20 nearest.

This is a 256-step smoke at two seeds, not a capability or training-law claim.

## Tokens per second first

- **Decode through the served evaluator** (`--decode-speed … --served-forward 1 --objective cell`,
  48 tokens):

  | Context | Served checkpoints | Plain checkpoints | deepseek-5's (#3407) |
  |---:|---:|---:|---:|
  | 16 | 93.5, 93.5 tok/s | 93.2, 93.9 | 49.2–49.4 |
  | 64 | 88.1, 87.2 | 87.0, 88.5 | 40.8–41.1 |
  | 192 | 86.0, 85.6 | 85.4, 86.0 | 28.6–28.8 |

- **The scoring repair alone did most of the context-192 gain.** The same step-40 served checkpoint,
  byte-identical in all five files at `3980925d1` and `4465685531`, decoded at 81.9 / 60.2 /
  38.6 tok/s before the repair and 93.6 / 88.0 / 86.1 after it.
- **Training throughput:** 18,282 and 18,243 context tokens/s served, against 33,838 and 33,807 plain.
- **Step time** (step P, profiled): 167.5 ms served against 90.5 ms plain. Before the scoring repair
  it was 657.4 ms, and 682.9 ms at the learning fix alone.

## What did not work

1. **The ORDER's account of the fault was half right.** The float stand-in did round every activation
   below 2^15 to a whole number. But the gradient was not near zero. With the fix set aside, layer 0's
   `mlp.down` gradient norm was 0.0232 as served against 0.00271 plain, 8.6× larger: distorted, not
   zeroed. On the realistic row of the reference test, the activation 0.3 reached the weight after it
   as 0.
2. **I-0's first attempt failed on the instrument.** 288 of 289 CUDA library tests passed. The
   model-level test stopped before reading any gradient: "the trunk lacks
   layers.00.mlp.down.weight". The FBS trunk holds each projection as its SpiralCore table,
   `….spiralcore_fbs`. `753e08d7b` reads those tables.
3. **FIT round 1 was NOT FIT.** The first negative gate set aside the old tensor body. Its f64
   constants fail on f32 input with a dtype error before any assertion, so the gate could not tell a
   right fix from a wrong one. `43ca02940` sets aside only the exponent with a pinned patch
   (`setaside-exponent.patch`, SHA-256 `ced6e8cb…0a57`).
4. **The first cost cause was wrong.** Step P at `753e08d7b` put the served step at 682.9 ms against
   90.2 ms plain (7.57×). Its recurrence and MLP stage times followed the snap's pass counts, so
   `3980925d1` snapped in passes of 2^18 blocks instead of 8,192. Step P there read 657.4 ms against
   90.1 ms: the passes fell from 48 and 36 to 2 and 2, and the stages fell 5%. At a fixed pass size
   the pass count and the number of scores grow together, so the proportion could not separate them.
   That commit's test pinned the pass count, which turned out not to be the cost.
5. **The evaluation commands this sheet copied from deepseek-5's were wrong.**
   - `--decode-speed EVAL` exited 1 (`unknown variant cell`). It takes the checkpoint folder, and
     without `--objective cell` it times the writer decode, not the nearest decode these runs train for.
   - Without `--retain-rates 1`, `--inspect` read the served profile run's rounded step-40 copy as
     top-1 13, nearest 1, admitted 421. The run's own evaluation read 8, 16 and 476.

   deepseek-10's REPORT does not record whether its rerun of deepseek-5's sheet passed these flags,
   so its top-1, nearest and tokens/s figures are cited only as reported.
6. **One CUDA gate launch died at its first line.** At 05:39Z I started I-0 with `nohup` over
   `uor-pod ssh` instead of `uor-pod run`. `CUDA_COMPUTE_CAP` was unset, so the script stopped under
   `set -u`. My watcher filtered only for the script's own markers, so it missed the exit. I relaunched
   through `uor-pod run`, whose job log ends with `# exit=N`, and watched for that.
7. **R's first attempt stopped on my wrapper's own check.** The wrapper grepped `progress.jsonl` for a
   loss written as `null` and stopped after plain/s1: "plain-s1 wrote a non-finite loss". That null
   was step 0's `train_loss_mean`, which is null by design because nothing has trained yet. The run
   was healthy: finite losses at steps 64–256, and top-1 41 and nearest 359 at step 256. An
   instrument fault: the check now requires a number for every training loss after step 0 and every
   DEV loss. It passes on that file and fails on a planted null at step 64. R reran all four jobs.
   plain/s1's step-256 line was the same in both attempts.
8. **Served openings stay below plain.** Openings are 9 / 17 served against 27 / 35 plain, through
   the same evaluator. Served DEV loss rose over training on both seeds, 2.806 → 2.843 and
   2.795 → 2.856. Plain's moved 2.811 → 2.798 and 2.787 → 2.820. At seed 1, most of the gap is the
   address term: 1.841 served against 1.797 plain at step 256. These are smoke readings at
   256 steps, not a claim.
9. **This contradicts `served-rounding-cost` (#3586).** Its host probe put most of the served step's
   cost in per-step `grid_code` and host table copies (`fbs_export`). The GPU profile puts export
   rounding (`served.round`) at 14.5–14.6 ms, 0.16 of a plain step, at every head. The cost was in the D11
   snaps.

## What changed

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`:
  - `activation_quantize16` and `activation_scale_scalar` scale by `2^−ACTIVATION_EXP` before the
    integer formula, and find the shift in six exact halvings, not 49 passes (`67075512e`).
  - `icosian_scores` scores each 4-block against the 120 roots with four elementwise products, added
    as `(q0 r0 + q2 r2) + (q1 r1 + q3 r3)`, the order candle's `fast_sum` adds them in, so every score
    keeps its f32 value. That `sum` over the 4-axis of a `(len, 120, 4)` product launched one GPU block
    of four threads per score (`4465685531`). The repair replaces the product and its sum together.
    Step P confirms the scoring as the cost, not the sum's launches alone.
  - Tests:
    - the host test `served_activation_quantizer_is_the_integer_step_at_product_exp`;
    - the CUDA tests `served_activation_quantizer_is_its_integer_reference_on_cuda` and
      `icosian_scores_are_the_four_axis_sum_on_cuda`;
    - `served_surrogate_gradients_are_exact_on_cuda`, its literals updated.
- `crates/uor-r4-float-train/src/in_space_long.rs`: the CUDA test
  `a_step_as_served_reaches_every_projection_after_a_requantized_activation`.
- `crates/uor-r4-integer/src/fbs_snapshot.rs`: `FBS_ACTIVATION_EXP = −32`. A stack test in
  `stack/tests.rs` holds it equal to `PRODUCT_EXP`.
- The prep sheet and its Amendment 1, FIT rounds 1–3, the pre-run reviews and the run result.
- `docs/research/notebook/float-training.md`: one row.
- `docs/roadmap/items/one-model-end-to-end.md`: the first "What is missing" bullet.

## Cancel probe

From the ORDER, unchanged. `origin/main`'s `activation_quantize16` had no exponent. Open PR #3579
(claude-261) changes only its constants' dtype, values unchanged, which this unit's body keeps. The
sheet's inputs are in the HF store. A trial merge of this branch with `origin/main` (`de9cfc84a`) is
clean. Classed mechanism-instrument: the served graph's float stand-in was wrong.

## Measurements

All on `vast-58920706` (RTX 5090, $0.563/h, about 57 minutes from 05:02Z). Binaries built at
`4465685531`: `train-in-space` `443f9764…0586`, `fbs-snapshot` `f031c7ce…8083`.

**I-0 at `4465685531`: PASS.**
- 290 CUDA library tests passed (7 ignored), with the four named tests. Among them,
  `a_step_as_served_reaches_every_projection_after_a_requantized_activation` holds every layer's
  `mlp.down` and `rec.out` gradient norm within 4× of plain and above 0 (done-when 2(b)).
- The pinned exponent patch failed the first and third named tests on their named assertions.
- Restored, all three passed, the binaries were byte-identical, and the data pins matched.

**Step P at `4465685531`.** Served 167.5 ms against plain 90.5 ms (1.85×). Per stage:
- each recurrence 3.50 ms (plain 0.57); each MLP 2.48 ms (plain 0.18);
- `backward` 84.0 ms (plain 53.6); `served.round` 14.6 ms.

No stage adds a plain step.

**Step R:**

| Seed | Mode | Wall s | Train s | Evaluator top-1 | Nearest | Openings | Admitted |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | plain | 52 | 23.24 | 40 | 239 | 27 | 445 |
| 1 | served | 93 | 43.02 | 53 | 305 | 9 | 491 |
| 2 | plain | 51 | 23.26 | 43 | 248 | 35 | 436 |
| 2 | served | 94 | 43.11 | 67 | 357 | 17 | 490 |

- **Instrument door: PASS.** Both served checkpoints are byte-identical after `--round-as-exported`;
  the round adds only its marker. Four of five plain files change. Each served run's own step-256 DEV
  line equals its evaluator reading on the rounded copy; a plain run's does not. Plain's own lines
  are [41, 359, 52, 404] and [86, 422, 43, 471] before the round.
- **Operational door: PASS.** 1.79× and 1.84× plain wall time; 1.85× plain training time at both
  seeds; bar 2×.
- **Quality smoke: INCONCLUSIVE.** Served minus plain is +13 and +24, against a band of
  max(24, 11) = 24. Served is not below plain on either seed.
- **Unit reading: the repair holds.**

**Seal:** `results/claude/one-model-end-to-end-served-repair-2026-11-03/claude-262.tar`, SHA-256
`d04505ac64af7c9f37c60c1e26b00943cad16b3bd36aebde37541da8cabb00e3`. It was read back and checked
before `down`.

## Checks run at the head

- `cargo test -p uor-r4-float-train --lib` (laptop): 209 passed.
- `cargo test -p uor-r4-integer --features serving-wasm --lib fbs_activation_exponent_is_the_requantized_product`:
  1 passed.
- Stub-`nvcc` `cargo check -p uor-r4-float-train --features cuda --lib --tests`: clean.
- On the pod at `4465685531`: `cargo test --release -p uor-r4-float-train --features cuda --lib`,
  290 passed, with the negative gate as above.
- `git diff --check origin/main...HEAD`: clean. `uor-harness check`: warnings only, none in this
  unit's files.

## Skills used

None applied. The candle reduce kernel and its launch were read in the vendored source
(`third_party/candle-kernels-0.9.2/src/reduce.cu`, `third_party/candle-core-0.9.2/src/cuda_backend/mod.rs`).

## Still open

1. **The served step is inside the bar, but close: 1.85× plain.** What remains:
   - the backward, +30.4 ms;
   - `served.round`, 14.6 ms;
   - the snaps, about 5.2 ms a layer beyond plain (2.9 ms in each recurrence, 2.3 ms in each MLP).

   `served-rounding-cost` (#3586) registered a 2× repair whose premise, host copies in `fbs_export`,
   step P does not show. That unit should start from P's GPU profile, in the seal.
2. **Longer served runs at this head.** claude-261's 2,048-step served runs (DEV nearest 921–2,314) ran
   with the activation fault; this unit is the cause of its finding A. Its arms should be rerun at this
   head before they are compared with claude-259's.
3. **Served openings stay below plain** (9 / 17 against 27 / 35). That is a quality question for a
   longer run, not this smoke.
4. **Capture mode.** A zero block's snap margin can now be written as −0.0 where it was +0.0 (FIT
   round 3, note 3). This matters only to something that pins a capture file's hash.
5. **Two one-unit edges in the float stand-in**, found by the exact-head review. Both are older than
   this unit, and the scalar reference shares them, so the CUDA equality test cannot see them.
   - Rounding as `floor(x + 0.5)` in f32 serves 1 at the scaled value `0.5 − 2^−25`, where the integer
     step serves 0: the sum is an exact f32 tie that rounds up to 1.0.
   - The shift is chosen from the unrounded row maximum, not the held `round(v·2^32)`. So a maximum
     within half a unit of a power of two can take a shift one less than the integer step's.

References #3407.

## Plain summary

The model trained "as served" rounded its internal activations to whole numbers, so its training
signal was distorted (one layer's gradient came out 8.6× the plain one), and each training step spent most of its time launching tiny GPU jobs, 35 million
for one gate, to pick the nearest of 120 fixed rotations. Both are fixed. A served training step now costs 1.85× a plain
one. On two seeds the served model scored above plain on the served evaluator's top-1 and nearest
readings but below it on reply openings, so the 256-step smoke is inconclusive. Its saved files stay byte-identical through export, and decode at a 192-token context runs
at 86 tokens per second, three times deepseek-5's 28.7.
