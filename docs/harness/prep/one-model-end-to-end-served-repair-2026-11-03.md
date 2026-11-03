# Served-training repair: CUDA gate, profile and the matched rerun

**Status:** REGISTERED (claude-262, [ORDER](../../loop/tasks/one-model-end-to-end/claude-262/ORDER.md), PR #3587); amended after step P, before step R ([Amendment 1](#amendment-1-after-step-p-before-step-r)).

TEST FITNESS: FIT (see Verdict)

## Question

claude-262 found that the float stand-in for the integer step's activation requantizer
(`activation_quantize16`) left out the step's exponent (`PRODUCT_EXP = −32`). So a served training
forward rounded every activation below 2^15 to a whole number, and `mlp.down` and `rec.out` got
almost no gradient. The fix scales by `2^−ACTIVATION_EXP` first and finds the shift in six exact
passes instead of 49. With it, on RTX 5090:

1. does a served step give every layer's `mlp.down.weight` and `rec.out.weight` a gradient within
   4× of plain, never 0, and does the tensor quantizer equal its scalar reference value for value?
2. where does a served step's time go, stage by stage, against a plain step's?
3. on [deepseek-5's sheet](one-model-end-to-end-served-comparison-2026-11-02.md), unchanged, does
   served training pass the instrument and operational doors, and where does it read against plain
   in the quality smoke?

This is an engineering repair and a matched short comparison. It does not establish language
capability, a milestone bar, or a training law.

## Base artifact

The exact PR #3587 head that passes the pre-run review. If step P (profile) leads to a repair inside
the unit's scope, that repair is a new commit: step I-0 reruns at the new head before step R, and
step R runs only at a head whose I-0 passed. The pod records the source SHA, the release
`train-in-space` and `fbs-snapshot` SHA-256 and every data file's SHA-256 before the first run.

## Data split

deepseek-5's pins, unchanged:

- Data root: `/workspace/w2-inputs/2026-10-25/driver-data`.
- TRAIN tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`.
- TRAIN mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

On the pod the data root is staged from the HF store with
`/root/uor-store get w2-inputs/2026-10-25/ /workspace`, and the five files' SHA-256 are checked
against these pins before the first job. A data hash that does not match stops the run.

## Steps

**I-0. CUDA gate.** Build with `--features cuda` and run
`cargo test -p uor-r4-float-train --features cuda --lib`. Every test passes, and these three by name:

- `served_activation_quantizer_is_its_integer_reference_on_cuda`: rows at every power-of-two edge
  from 2^−20 to 2^31 and a saturated row; quantized values, straight-through gradient and the
  following weight's gradient equal the scalar reference bit for bit; on a realistic row
  (0.3, −0.12, 0.004, −1.5·10^−5) the following weight's gradient is within 2^−17 of the activation.
- `served_surrogate_gradients_are_exact_on_cuda` (existing, its literals updated).
- `a_step_as_served_reaches_every_projection_after_a_requantized_activation`: one served forward
  and backward against one plain, on the same model and batch; every layer's `mlp.down.weight` and
  `rec.out.weight` gradient norm is above 0 and served/plain is within [0.25, 4].

**Negative gate:** set aside only the fix's exponent with
[`setaside-exponent.patch`](../../loop/tasks/one-model-end-to-end/claude-262/setaside-exponent.patch)
(SHA-256 `ced6e8cbd0e6d6101ac2a27900c873b2f40be05f4dd25583ba69dc7bdb280a57`, committed in the head):
in `activation_scale_scalar` and `activation_quantize16` the `2^−ACTIVATION_EXP` factor becomes 1,
and the six passes, the dtype handling and the constants stay. That is the old fault exactly, without
the old body's f64 constants, which would fail on f32 input with a dtype error before any assertion.
Run `git apply` on the pod, rebuild the test binary and read its build line. The first and third tests
above must fail **on their named assertions**:
- the first with the message "… reached the weight after it as …" on the realistic row (the edge
  rows still agree, since scalar and tensor both lose the exponent);
- the third with "…: gradient norm … as served against … plain".

A candle `Err`, a panic elsewhere, or a build failure does not count. Restore with
`git checkout` of the file, rebuild, and rerun all three green. On the laptop the same patch makes
both host tests (`served_activation_quantizer_is_the_integer_step_at_product_exp`,
`served_surrogate_scalar_and_state_controls_are_exact`) fail on their `assert_eq!` lines.

**P. Profile.** One plain and one served run on seed 1 with the R recipe below, `--steps 40
--profile 16 --eval-every 40`. Report each run's `profile.json`: stage medians (GPU and host),
groups, `top3`, `median_unprofiled_ms`, and served/plain median step time. A stage whose median
alone adds more than one plain step to the served step is named with its cause. If it is in the
ORDER's files, it is repaired in this unit and I-0 reruns; if not, it is named in the REPORT and R
runs at the current head.

**R. The matched rerun.** deepseek-5's four jobs, in order plain/s1, served/s1, plain/s2,
served/s2:

```text
train-in-space --data-root DATA --out RUN --objective cell --hamming-read 4 \
  --retain-rates 1 --decay-mult 1 --batch 16 --steps 256 --eval-every 64 \
  --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 \
  --turn-memory-dropout 500 --train-as-served MODE --seed SEED
```

After each job, stop on a non-finite loss or a missing checkpoint. Then, with deepseek-10's corrected
evaluation step, `EVAL` absent before each command and `fbs-snapshot` creating it:

```text
fbs-snapshot --round-as-exported RUN/models/step-000256 EVAL/models/step-000256
train-in-space --inspect EVAL --data-root DATA --served-forward 1 --retain-rates 1 --seed SEED
train-in-space --decode-speed EVAL/models/step-000256 --data-root DATA --served-forward 1 \
  --objective cell --retain-rates 1 --seed SEED
```

`--inspect` and `--decode-speed` take the run's rate flag and seed from the command line, and
`--decode-speed` takes the checkpoint folder; `--objective cell` makes it time the nearest decode
these runs train for (Amendment 1). A wrapper checks every command's exit status and writes `DONE`
only after all of them succeed.

**S. Seal.** Logs, progress, configs, `profile.json` files, source, binary, data and copy hashes,
reports and `SHA256SUMS`, in one tar under `results/claude/one-model-end-to-end-served-repair-2026-11-03/`;
upload, read back and check against `SHA256SUMS` before `down`.

## Bar

- **I-0:** the three named tests pass at the head; with the exponent set aside, the first and third
  fail on their named assertions; restored, they pass. A failure here stops the run before P.
- **R, deepseek-5's bar unchanged:**
  1. *Instrument door:* all four runs finish with finite losses and load through the served
     evaluator; both served checkpoints are byte-identical after `--round-as-exported`; at least one
     plain file changes.
  2. *Operational door:* served training time is at most 2× plain at the same seed.
  3. *Quality smoke:* served-evaluator top-1, served minus plain, per seed. Served is "materially
     below" only if it is below plain by at least `max(24, two-seed spread)` on both seeds; likewise
     for plain. Otherwise the reading is inconclusive.
- **Unit reading:** the repair holds when I-0 passes, both R doors pass, and served is not materially
  below plain. DEV nearest hits are reported beside top-1, against deepseek-5's 23/20 served and
  225/229 plain.

Decode tokens/s (the `--decode-speed` command above, contexts as it reports them) are reported
first in the REPORT.

## Minimum test power

Two seeds, 256 steps: enough for finite operation, export identity, a time ratio and a smoke-level
quality reading, not for a training-law claim. The gradient check is one step on one batch; it shows
the gradient reaches the projections, not that training converges.

## Stop rule

- Stop on a non-finite loss, a failed checkpoint load, a data-pin mismatch or an output hash mismatch.
- Stop if the work would exceed one pod, two GPU-hours or $1.50; more is a new ORDER.
- Do not open a fresh panel; DEV is diagnostic.
- If I-0 fails, or the served arm fails the instrument door, keep every output and classify instrument
  against mechanism before any retry.
- If served is still materially below plain, R's outputs stand as the negative. The next cause is
  found by turning the D11 snaps, the Q16 residual and the Q12 query off one at a time in a finite
  CUDA gradient check (ORDER, "Stop and ask if").

## Cost

One RTX 5090 on Vast, about $0.56/h, at most two GPU-hours and $1.50, inside the project-wide
4-pod/$8 per hour caps (two pods, $1.12/h, were running at registration). Projection: build and tests
25 minutes, negative gate 10, profile 5, four runs 10, evaluation 20.

```text
scripts/pod/uor-pod up --lab claude --session claude-262 --count 1 --gpu 5090 --hours 2 \
  --purpose "claude-262 served-training repair: CUDA gate, profile, matched rerun" \
  --card https://github.com/UOR-Foundation/uor-r4/pull/3587
```

A completion watcher is armed on every detached job. The lease is renewed at least every 30 minutes
and released, and the pod destroyed, only after the sealed tar is uploaded, read back and checked.
claude-261's pods (`vast-58896165`, `vast-58912154`) are not touched.

## Amendment 1 (after step P, before step R)

Step P ran at `753e08d7b`, where I-0 passed, on `vast-58920706`. It found two things that change
what runs next. Neither changes the question, the data, R's training recipe, the bar or the stop
rule.

**Step P's profile.** 16 profiled steps of 40; each run's stages sum to its unprofiled median within
0.7%. The third column is step P again after the first repair below; its plain run read 90.1 ms.

| Stage (median) | plain, `753e08d7b` | served, `753e08d7b` | served, `3980925d1` |
|---|---:|---:|---:|
| Step, unprofiled | 90.2 ms | 682.9 ms (7.57×) | 657.4 ms (7.29×) |
| `served.round` | — | 14.5 ms | 14.5 ms |
| Each layer's recurrence | 0.57 ms | 26.9 ms | 25.5 ms |
| Each layer's MLP | 0.18 ms | 20.0 ms | 19.0 ms |
| `backward` | 53.6 ms | 353.2 ms | 338.3 ms |
| `layer1.hamming_read`, `readout.hamming`, `objective.table_search` | 8.1, 8.1, 12.8 ms | 8.6, 8.5, 12.8 ms | 8.2, 8.2, 12.8 ms |

At `753e08d7b` the backward alone adds 299.6 ms, 3.3 plain steps; the six recurrence and six MLP
stages add 277 ms together. Export rounding (`served.round`, 0.16 of a plain step) is not the cost,
so this is not the host-copy cause that `served-rounding-cost` (#3586) probed.

**A first repair, refuted.** In the D11 mode every Hamilton gate runs the icosian-hard arm, which
snaps each 4-block to its nearest icosian (`icosian_snap_cuda_tensor` in `hamilton_sheet.rs`), then
in passes of 8,192 blocks. At batch 16 a recurrence makes two snaps (the transition and the output
gate, 24 + 24 passes) and an MLP one (36 passes), and the icosian-hard backward snaps each gate
again. The stage times followed the pass counts (26.9 / 20.0 = 1.35 against 48 / 36 = 1.33), so
`3980925d1` snapped in passes of 2^18 blocks, two passes per stage, with a CUDA test holding the
roots bit for bit against 8,192-block passes. I-0 passed at `3980925d1`. Step P there refuted the
cause: the passes fell from 48 and 36 to 2 and 2, but the stages fell by only 5% (third column).
At a fixed pass size the pass count and the block count grow together, so the proportion could not
tell a cost per pass from a cost per block.

**Named cause.** Each pass scored its blocks against the 120 roots as a `(len, 120, 4)` product
summed over the 4-axis. candle 0.9.2's CUDA `sum`
(`third_party/candle-core-0.9.2/src/cuda_backend/mod.rs` lines 320–335, kernel `fast_sum`) launches
one GPU block per output value, with `next_power_of_two(min(1024, values summed))` threads. So that
sum launched one block of four threads per score: 35.4 million blocks for a batch-16 MLP gate
(294,912 4-blocks × 120 roots), in every forward snap and again in the backward. The cost follows
the scores, not the passes, as step P at `3980925d1` showed. It is read from the launch shape, not
timed on its own; step P at the new head tests it. The repair replaces the `(len, 120, 4)` product
and its sum together, so step P can confirm the scoring as the cost, not the sum's launches alone.

**Repair, in the ORDER's files.** `4465685531` (`icosian_scores`) adds the four products as
`(q0 r0 + q2 r2) + (q1 r1 + q3 r3)` in four elementwise products and three additions. That is the
order `fast_sum` adds them in with four threads (each thread loads `0 + p_t`, then halving at
`s = 2` and `s = 1`), so every score keeps its f32 value; only a −0 score can come out −0 where the
sum gave +0, which compares equal and changes no argmax. The pass size becomes 2^16 blocks, which
only bounds each pass's `(blocks, 120)` tensors to 32 MB. The test
`icosian_scores_are_the_four_axis_sum_on_cuda` replaces the pass-size test. Over two passes of a gate
it checks that no score differs from the old 4-axis sum, and that every snapped root is bit for bit
the root the old sum's argmax picks, on spread, zero, −0.0 and exactly tied blocks. It pins values,
not cost; step P reads the cost. The repair does not touch lines 669–712, so
`setaside-exponent.patch` applies unchanged (`git apply --check`; its hash is unchanged).

**Evaluation commands.** Step P's evaluation dry run found two faults in the registered commands,
which this sheet copied from deepseek-5's:
- `--decode-speed EVAL` exited 1 (`unknown variant cell, expected ce or margin_hinge`). It takes
  the checkpoint folder, `EVAL/models/step-000256`, and read the run folder's `config.json` as a
  model config. Without `--objective` it also decodes with the writer (`sheet`), not the nearest
  decode these runs train for.
- `--inspect` takes the rate, fold and floor flags from the command line; the binary's usage says
  they "must be the checkpoint's run's". Without `--retain-rates 1` it read the served profile run's
  rounded step-40 copy as top-1 13, nearest 1, admitted 421; the run's own evaluation at step 40
  reads top-1 8, nearest 16, admitted 476.

The amended commands in step R add `--retain-rates 1 --seed SEED` to both, add `--objective cell`
to the decode, and pass the decode the checkpoint folder. The fold and floor flags stay at their
defaults, which the runs train with. deepseek-10's REPORT does not record whether the earlier run of
deepseek-5's sheet passed these flags, so its top-1 and nearest figures are cited only as reported.

The amended commands ran in step P's dry run at `3980925d1`, on its served run's step-40 checkpoint.
The round changed no saved file and added only its `rounded-as-exported.json` marker. `--inspect` on
the rounded copy read top-1 8, nearest 16, openings 0, admitted 476, the run's own step-40 line. The
decode read 81.9, 60.2 and 38.6 tokens/s at contexts 16, 64 and 192 (48 tokens each).

**What runs, in order.** I-0 and step P run at the repair head `4465685531` under the base sheet's
rule for a repair (a new commit, I-0 again before R); R runs at the amended head, which carries this
amendment and its FIT and differs from `4465685531` in docs only, with I-0's binaries.
1. I-0 as registered, with `icosian_scores_are_the_four_axis_sum_on_cuda` also required by name.
2. Step P again, same command. The served step is read against 2× plain, stage by stage. If it is
   still past 2× and a stage in the ORDER's files adds more than one plain step, that stage is
   repaired the same way before R (a new head, I-0 again). If the stage is outside the ORDER's
   files, it is named and R runs.
3. The evaluation dry run on P's served checkpoint with the amended commands. Instrument check: on
   the rounded copy, byte-identical to the served run's checkpoint, `--inspect` must reproduce the
   run's own step-40 DEV line (top-1, nearest hits, openings, admitted rows). A mismatch stops the
   work before R.
4. R and S as registered, with the amended evaluation commands.

Each added I-0 and P takes about 15 minutes, inside the two GPU-hour and $1.50 cap.

## Verdict

TEST FITNESS: FIT [claude] — three fresh non-author review rounds; round 3 covers Amendment 1.

- **Round 1** (sheet SHA-256 `a67d1555…6896c6`): NOT FIT, on one BLOCKER. The negative gate set aside
  the old tensor body, whose f64 constants fail on f32 input with a dtype error before any assertion,
  so the gate could not tell a right fix from a wrong one. Everything else was FIT: the profile window
  (8 + 2·16 = 40 steps), the `served.round`/`served.restore` stages, R's recipe, data pins, bar and
  evaluation step identical to deepseek-5's, the cost cap, no banned technique.
- **Round 2** (sheet SHA-256 `ec6bd88d…96f2`): FIT. The pinned patch removes only the exponent and
  keeps the input's dtype. With it, the reference test's edge rows still agree bit for bit and the
  realistic row fails its named assertion. Both host tests fail on `assert_eq!` lines. Whether the
  model-level test fails is decided on the pod, and a pass there stops the run at I-0, so it cannot
  give a false pass. If a repair touches lines 669–712 of `hamilton_sheet.rs`, the patch is made
  again and its hash pinned again before I-0 reruns.
- **Round 3** (Amendment 1, sheet SHA-256 `e17e2e2d…67ca` at `07dd38a03`): FIT, no blocker.
  - *Values:* the old score was `(p0+p2)+(p1+p3)` (`cuda_backend/mod.rs:322`, `reduce.cu:79–97`).
    Each product and each add is its own kernel, built with `-ftz=false`, so no fused multiply-add
    can change a value. `fast_argmax` compares with a strict `>`, so ±0 changes no root.
  - *The test:* it covers two passes, the exact four-way golden tie and the ±0 blocks. About
    10 million random scores would expose another pairing. It does not exercise the forced-call or
    capture paths, which R does not use.
  - *The cause:* the 5% drop at 2^18 is a per-pass overhead of about 30 µs, consistent across the
    MLP, recurrence and backward. What remains scales with blocks. Step P at the new head decides it,
    and a cost repair needs no negative gate when a value-identity test pins its values.
  - *The commands and the order:* the evaluation flags are confirmed in both binaries, and the
    instrument check can fail. R's head differs from the repair head in docs only.
  - *Charter:* no new product. These are the same 4-term inner products against the 120 fixed
    icosians, the nearest-root search inside the Hamilton gate (§3.3).
  - *Notes applied:* the cause is claimed for the scoring, the `(len, 120, 4)` product with its sum,
    not for the sum alone; "returns to 2^16" now reads "becomes". Note 5 is taken in the REPORT:
    each R run's `--inspect` is also compared with its own step-256 DEV line. Note 3: in capture
    mode, a zero block's margin can now be written as −0.0.

PRE-RUN: PASS [claude] — fresh adversarial review under D67 at `43ca02940`:
- *chain:* Casey's 2 Nov chat-model goal, every path model trains as served, and deepseek-10's
  decision names this unit next;
- *mechanism and tools present:* the fix, the three CUDA tests and the stack test are at the head, as
  are both binaries, every flag, the data pins equal to `PINNED_*`, the store route for the data and the
  patch hash; there is no checkpoint input;
- *every outcome changes the next step:* a pass opens longer served runs; an I-0 fail reclassifies the
  cause; an operational fail comes with the stage profile; a quality fail starts the surrogate
  isolation;
- *caps:* 3 pods, $1.68/h, two GPU-hours at most.

Two notes were taken:
- the staging command is now written out above;
- main registered `served-rounding-cost` (#3586) after this unit's base. Its host probe puts most of
  the served step's cost in per-step `grid_code` and host table copies (`fbs_export`), with about 2% in
  the 49-pass loop, so the operational door may fail again at this head. Step P's GPU profile is that
  item's first deliverable. A repair of `fbs_export` is that item's unit, not this lease's, and the
  REPORT records the overlap.

PRE-RUN: PASS [claude] — fresh adversarial review under D67 at `07dd38a03`, with I-0's binaries from
`4465685531` (`train-in-space` `443f9764…0586`, `fbs-snapshot` `f031c7ce…8083`), conditional on
round 3, which then passed:
- *chain:* unchanged from the review at `43ca02940`. A longer served run first would measure the fault,
  not the mechanism;
- *tools and inputs:* every flag in R's commands parses at `4465685531`, the data pins and the store
  route are present, and there is no checkpoint input;
- *outcomes:* a pass opens longer served runs; served still materially below plain starts the
  surrogate isolation; an operational fail comes with P's stage profile;
- *caps:* 2 pods, $1.12/h. The pod came up about 05:02Z, so the unit's two GPU-hours end at about
  07:02Z, before the lease's 07:15:57Z. No further repair cycle fits: had step P found another stage,
  the stop rule would outrank the amendment's repair step.

Its condition that the dry run be redone at the repair head was met by step P there: the inspect of
the rounded copy read [8, 16, 0, 476], the run's own step-40 line.

## Run result

[claude] On `vast-58920706` (RTX 5090), with binaries built at `4465685531`, from R's head `b15b653b7`,
which differs from it in the sheet only.

**I-0 at `4465685531`: PASS.**
- 290 CUDA library tests passed (7 ignored), with the three named tests and
  `icosian_scores_are_the_four_axis_sum_on_cuda`.
- With the pinned patch, the first test failed on "0.3 reached the weight after it as 0" and the
  third on "layers.00.mlp.down.spiralcore_fbs: gradient norm 0.023247106 as served against
  0.0027099452 plain".
- Restored, all three passed, the binaries were byte-identical, and all five data pins matched.

**P at `4465685531`.** Served 167.5 ms against 90.5 ms plain (1.85×). Per stage:
- each recurrence 3.50 ms (0.57 plain), each MLP 2.48 ms (0.18 plain);
- `backward` 84.0 ms (53.6 plain), `served.round` 14.6 ms.

No stage adds a plain step, so R ran at this head. The 40-step served checkpoint is byte-identical
in all five files to the one at `3980925d1`, before the scoring repair. The dry run's instrument check
passed: [8, 16, 0, 476] from the run, the same from `--inspect`.

**R.** All four jobs and every evaluation command exited 0, and `DONE` was written at 05:56:50Z. The
first attempt stopped after plain/s1 on the wrapper's own null check: step 0's training loss is null
by design. The check was repaired to require a number after step 0, and all four jobs reran. plain/s1's
step-256 line was the same in both attempts.

| Seed | Mode | Wall s | Train s | Context tok/s | Evaluator top-1 | Nearest | Openings | Admitted |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | plain | 52 | 23.24 | 33,838 | 40 | 239 | 27 | 445 |
| 1 | served | 93 | 43.02 | 18,282 | 53 | 305 | 9 | 491 |
| 2 | plain | 51 | 23.26 | 33,807 | 43 | 248 | 35 | 436 |
| 2 | served | 94 | 43.11 | 18,243 | 67 | 357 | 17 | 490 |

The evaluator columns are `--inspect --served-forward 1` on each rounded copy. Decode through the
served evaluator, 48 tokens, read 93.2–93.9, 87.0–88.5 and 85.4–86.0 tok/s at contexts 16, 64 and 192
across the four checkpoints.

- **Instrument door: PASS.** All four runs are finite and load through the served evaluator. Both
  served checkpoints are byte-identical after `--round-as-exported`: the round adds only its marker.
  Four of five files change for each plain checkpoint. Each served run's own step-256 DEV line equals
  its evaluator reading on the rounded copy: [53, 305, 9, 491] and [67, 357, 17, 490]. A plain run's
  does not: [41, 359, 52, 404] and [86, 422, 43, 471] before the round.
- **Operational door: PASS.** Served took 1.79× plain wall time at seed 1 and 1.84× at seed 2, and
  1.85× plain training time at both (bar 2×).
- **Quality smoke: INCONCLUSIVE, with served above plain.** Served minus plain is +13 at seed 1 and
  +24 at seed 2, against a band of max(24, two-seed spread 11) = 24. Served is not below plain on
  either seed. Plain is below served by the band on seed 2 only.
- **Unit reading: the repair holds.** I-0 passed, both doors passed, and served is not materially
  below plain. Against deepseek-5's run, DEV nearest went from 23 / 20 to 305 / 357 served, beside
  225 / 229 → 239 / 248 plain. Served openings stay below plain: 9 / 17 against 27 / 35.

**S.** The seal is at `results/claude/one-model-end-to-end-served-repair-2026-11-03/` in the HF store:
- `claude-262.tar` (206 MB, 175 members), SHA-256 `d04505ac64af7c9f37c60c1e26b00943cad16b3bd36aebde37541da8cabb00e3`;
- `MEMBERS.sha256` `a698f9a3…65ff`, the hash of each member;
- `SHA256SUMS`.

It holds the logs, job logs, progress, configs, `profile.json` files, binary, data and copy hashes,
and R's four step-256 checkpoints. It was read back and checked against `SHA256SUMS` before `down`.

The lease was released and `vast-58920706` destroyed at 05:59:21Z, after about 57 minutes (about
$0.54). claude-261's pod was not touched.
