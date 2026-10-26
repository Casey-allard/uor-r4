# REPORT: d11-exporter-float-trunk, claude-131

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-dual-train-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-dual-train-2026-10-26.md) (FIT in round 2; D67 pre-run review GO). The decision review that chose this run: [owner log on #2820](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6417189833), D2. The decision review on this result: [owner log on #2820](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6419387874).

References #2820.

**Speed.** On one RTX 5090, dual training ran at 4,373 and 4,348 tokens/s (seeds 1 and 2), 180 s per seed for 1,024 steps (`run.train_tokens_per_second`). The float model's in-run GPU decode was 117 and 130 tokens/s (`run.decode_tokens_per_second`); that is not a serving rate. The artifact's own decode and context tokens/s are PASS-only steps (sheet §6 step 9), so they were not measured.

## What did not work, first

- **`G-D11-ExportGap` FAILs on both seeds on its top-1 half, this time in the same direction.** The trunk was trained on the plain-float and D11-mode losses together, 1:1. Its artifact was scored against its own plain-float forward:

  | Seed | F top-1 (plain float) | A top-1 (artifact) | b | c | Net b − c | F_I | Top-1 half |
  |---|---|---|---|---|---|---|---|
  | 1 | 3,845 | 3,492 | 827 | 474 | +353 | 73 | outside |
  | 2 | 3,128 | 2,897 | 719 | 488 | +231 | 70 | outside |

  - Both counts are over the 14,287 `ft/dev` targets. b counts rows F hits and A misses (`a_hit_b_miss_c`); c is the reverse (`a_miss_b_hit_b`).
  - **b > c on both seeds:** the artifact loses rows to its own float model. Under the sheet's reading, the artifact, not the float model, is the one off.
  - **What it decides,** under the sheet's FAIL row: dual-forward training by this recipe (1:1, the bridges, Stage 4b's 1,024 steps and data) does not bring the gap inside the band. That closes this configuration, never the mechanism. The next step is the decision review's, below.
  - **This is the line's third result that does not move its headline** (claude-129, claude-130 and claude-131). So, under CHARTER §4.4, a pivot note follows ("stop, or one decisive run"); see "Still open".
- **The guard on a PASS does not apply,** because there is no PASS. For the record: the artifacts' top-1, 3,492 and 2,897, are above claude-129's plain-trained floors (1,907 and 2,294).
- **This arm's two-seed loss band, 0.0290** (|1.9065087296576082 − 1.877547065864761|), is above plain training's own band, 0.0230. By the sheet's pre-registered rule, that reads as **less stable across seeds than plain training**, though steadier than claude-130's 0.0445.
- **The float loss is worse than plain training's,** 1.9065 and 1.8775 against claude-126's 1.8546 and 1.8316, while its top-1 is far higher (3,845 and 3,128 against 1,988 and 2,643). The two halves of the sheet's metric moved in opposite directions here.
- **The item's done-when is not met,** so #2820 stays open.

## What changed

Size against `origin/main`: float-train, in `spiralcore_fbs_sheet.rs`, `hamilton_sheet.rs` and the sheet binary. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **`TrainForward`** (`plain`, `d11`, `dual`) replaces the `d11_train` flag of `run_fbs_arm_with`. `plain` and `d11` train exactly as before.
- **`train_step`** takes one step's loss and gradients as the forward says. In `dual`:
  - the plain loss's backward runs under the normal gate hook;
  - the D11-mode loss's backward runs inside `with_d11_training`;
  - the gradients are summed tensor by tensor (`sum_grads`);
  - the reported training loss is the sum.
- **`spiralcore-fbs-sheet --train-forward dual|d11`** drives `d11-train-probe` and `d11-train-export`; the default is `d11`. The option is parsed before the GPU opens.
- **claude-130's low review findings, fixed:**
  - the export receipt carries each seed's training trajectory (`run`);
  - the Q12 bridge passes no gradient where the i16 clamp saturates, as its forward does;
  - each gap receipt names the sheet the trained models answer to (`training_sheet`).
- **Tests:**
  - the host test `the_training_forwards_name_their_sheets`;
  - the GPU test `gpu_dual_gradients_add_tensor_by_tensor`, which covers `sum_grads` alone;
  - the GPU test `gpu_a_dual_step_is_a_plain_step_plus_a_d11_step`, which checks that a dual step's loss and gradients equal a plain step's plus a D11 step's, each taken on its own;
  - the clamp case in `gpu_d11_bridges_pass_the_tangent_and_straight_gradients`.

## Measurements

**The loss half is inside the band on both seeds** (Δ_noise,L 0.0457):

| Seed | L_F | L_A | L_A − L_F |
|---|---|---|---|
| 1 | 1.9065087296576082 | 1.924135561986309 | +0.0176 |
| 2 | 1.877547065864761 | 1.8963910506889219 | +0.0188 |

**Every instrument check was green.**
- **`I-Bridges`:** the float-train CUDA library at the run's head (`faecaaac5`) passed on the RTX 5090, 57 passed and 5 ignored. That includes both bridge tests, the clamp case, the gradient-sum test, and claude-125's and claude-126's GPU D11 tests.
- **`I-Probe`:**
  - the dual training loss fell from 5.6042 to 4.9343 over 32 steps;
  - the DEV float loss fell from 2.7939 to 2.5162 (claude-130's D11-only probe reached 2.6153);
  - a step took 0.176 s;
  - the full run projected to 462 s;
  - the step-0 query credit was 0.6138.
- **`I-Save-Load`:** both seeds' saved models reproduce their own float panel and loss.
- **`I-Verify-All`:** G = H = R at all 82,176 window positions on each seed. The evaluator's PTX is `013d19fd…` and its cubin `73eccb70…`, claude-128's qualified ones.
- **`I-Inputs`:** the windows, each seed's artifact pin (`0ae7dcd4…` and `8394a4d3…`) and the five data and receipt pins all match. Each gap receipt names this sheet as its `training_sheet`.
- **`I-DEV-Denominator`:** 14,287 rows.
- **`I-Finite`:** every step exited 0, and all 92 files listed in `dual/SHA256SUMS` verify, on the pod and again on the laptop.

**The seed spread, read out (the decision review's condition 2):**

| Seed | Plain-float top-1 | Export-mode top-1 | Artifact top-1 | Admitted rate, float | Admitted rate, export mode |
|---|---|---|---|---|---|
| 1 | 3,845 | 3,511 | 3,492 | 0.379 | 0.375 |
| 2 | 3,128 | 3,023 | 2,897 | 0.284 | 0.269 |

The top-1 counts and admitted rates come from `train.json`: `seeds[].float` and `seeds[].export_mode`, fields `top1_hits` and `admitted_target_rate`. The artifact's top-1 comes from each gap receipt's `top1_artifact`.

- **The top-1 spread, |F(1) − F(2)| = 717,** is near plain training's 655 and far from claude-130's 3,024. So the sheet's "the dual loss did not cure the seed instability" reading does not apply.
- **The loss band, 0.0290,** is above plain training's 0.0230: less stable by the pre-registered rule, as stated above.
- **The admitted rate** is 0.38 and 0.28 in plain float, against claude-130's 0.354 and 0.091. Seed 2 no longer collapses.

**Readings, with no bar:**
- **The artifact against the model's export-mode panel (E).**
  - E is `evaluate_dev_panel(…, export_mode = true)`: the gates at Q12, then snapped, and the query and writer table at Q12. It is not the artifact's emulation. The recurrence transition is not snapped and the tables are not rounded as exported; the D11 panel (`evaluate_dev_panel_d11`) does both, and this run did not compute it.
  - **Seed 1:** F − E is 334 rows and E − A is 19, so most of the gap lies between the plain-float forward and E.
  - **Seed 2:** F − E is 105 rows and E − A is 126, so most of it lies between E and the artifact. That E − A alone is wider than F_I (70). claude-130's E − A were −25 and −12, the artifact above E.
  - What the seed-2 E − A comes from (the transition snap, the table rounding, or an exporter departure) is not settled by this run. It needs the D11 panel of the saved models; see "Still open".
- **Against the earlier configurations:** on seed 2, the artifact's 2,897 is the highest of the three configurations (claude-129 2,294, claude-130 1,249). On seed 1, its 3,492 is second to claude-130's 3,663. Against claude-129's plain-trained artifacts it is +1,585 and +603 rows.
- **The training trajectory** (DEV float loss at steps 0, 256, 512 and 1,024): 2.794, 2.071, 1.986, 1.907 on seed 1; 2.761, 2.123, 1.955, 1.878 on seed 2. Both are still falling at step 1,024.
- **Gradient clipping** applies to the summed gradient, so it may bind more often than in single-loss training. The run did not record how often it did.
- **Wall time:** training both seeds, with their in-training evaluations, took 7 m 50 s; each verified evaluator run took about 20 minutes, mostly the CPU verify.

**Evidence:** the HF store at `results/claude/d11-fbs-dual-train-2026-10-26/`, each folder with a `SHA256SUMS`:
- `dual/` holds every step's stdout, stderr and exit code, the probe, the training receipt, the saved models and snapshots, both artifacts, the windows, both evaluator runs, both gap receipts and `job.log`;
- `dual-bin/` holds the three binaries run (`spiralcore-fbs-sheet` `a57a752d…`, `fbs-export` `baf3214e…`, `d11-fbs-eval` `d723c41d…`).

**The code review's dual-step test, run on the same pod after the run.**
- The code review found that nothing pinned the order of modes in the dual step. So `06def0a4f` moves the step's forward choice into `train_step`, unchanged, and adds `gpu_a_dual_step_is_a_plain_step_plus_a_d11_step`.
- The run's own source (`faecaaac5`) was copied on the pod, and the two files that differ at `06def0a4f` were overlaid. Their sha256s, recorded on the pod, match `git show 06def0a4f:<path>`: `f5cfeece…` and `526844c1…`.
- **Positive:** the float-train CUDA library passed, 58 passed and 5 ignored, the new test included.
- **Negative:** with the dual step's plain half moved inside `with_d11_training`, the new test failed (exit 101), at its loss check, `dual == plain + d11`: the dual loss was 5.6414 against 5.6364.
- The training path that ran is the same code. `train_step` holds the run's `match` as it was.
- Evidence: `t131/` in the same store folder, with its `SHA256SUMS`.

## Checks run at the head

See the exact-head review record on the PR.

## Skills used

- **holding-the-goal.** The bar stayed Stage 4b's, the comparator stayed the candidate's own plain float (§3.7), and the weight stayed 1:1 as pinned. The guard and the seed-spread reading were applied as the sheet fixed them, including the less-stable reading of the loss band.
- **writing-meaningful-tests.** The dual-step test was added when the code review found nothing pinned the order of modes, and it was run with the plain half moved into the D11 mode to show it fails.

## Still open

- **The §4.4 pivot note: one decisive run, not now.** Three results on this line (claude-129, claude-130, claude-131) have each passed the loss half and missed the top-1 half, by 1.35 to 5.6 times F_I (nets +81 and +349, +278 and −332, +353 and +231). The decision review (owner log, above) sets the order:
  1. **The residual check first, as the next unit on this item.** The D11 panel of claude-131's saved models, paired against their artifacts, classifies the seed-2 E − A residual as the D11 forward itself or an exporter departure. Scoring a model is GPU evaluation (§3.5), so this check needs its own sheet with a FIT (§4.5), though it is minutes of GPU. An exporter departure is a repair item, and the repaired artifact is then scored under the unchanged bar.
  2. **The owner's reading of §3.7.** "A gap above the two-seed noise band disqualifies it" can be read as the sheets' paired F_I for top-1, or as the two-seed top-1 spread, 655 to 717 rows. Under the spread reading, all three configurations would pass. Which reading applies is the owner's (§5.1), so one `[claude, for Casey]` note is on #2028, and no old receipt is re-scored.
  3. **The one decisive run, only after both.** Its sheet keeps Stage 4b's bar unless Casey answers, makes one change only (for example longer training at 1:1, since both DEV losses are still falling at step 1,024), and keeps the guard and the seed-spread rule. A PASS closes #2820. A FAIL makes the pivot note's verdict "stop": the item parks, and only the owner can stop the mechanism (§4.1).
- **After the residual check, the item parks** behind the owner's reading, and GPU effort moves to the M1 path, as PLAN.md orders when earlier-path work is free. #2820 stays open with no new training sheet. The residual check runs first because it does not depend on the owner's answer, and an exporter departure would bear on all three results (§4.2, L-007).
- **The Q12 clamp mask at the negative edge** (code review, low). The mask tests the scaled value before rounding, so values in [−32768.5, −32768) get no gradient, though the clamp leaves them alone. The band is 0.5/4096 wide; the trained path is left as it ran.
- **`with_d11_training` leaves the seed and step set** (code review, medium, traced). For this arm it changes no training math: the gate's forward reads the seed only for `RandomGateNull`, its backward reads neither, and only the `step == 0` gate telemetry, which the FBS sheet does not report, reads the step.

## Plain summary

The model was trained to do well both in its ordinary float mode and in the integer-like mode the shipped file uses, at the same time.
- **The model got much better at its task,** and the two training runs now agree with each other far more than in the last attempt.
- **The comparison still fails.** The shipped integer file gets 231 to 353 fewer answers right than the model's float mode, where the test allows about 70.
- **The other half of the test passes:** the file's loss is within the allowed band of the model's.
