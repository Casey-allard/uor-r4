# REPORT: d11-exporter-float-trunk, claude-130

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-d11-train-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-d11-train-2026-10-26.md) (FIT round 1; D67 pre-run review GO). Decision review: [owner log on #2820](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6417189833) (fresh, APPROVE).

References #2820.

## What did not work, first

- **`G-D11-ExportGap` FAILs on both seeds, on its top-1 half, in opposite directions.** The trunk was trained in the D11 mode, and its artifact was scored against its own plain-float forward:

  | Seed | F top-1 (plain float) | A top-1 (artifact) | b | c | Net b − c | F_I | Top-1 half |
  |---|---|---|---|---|---|---|---|
  | 1 | 3,941 | 3,663 | 719 | 441 | +278 | 69 | outside |
  | 2 | 917 | 1,249 | 258 | 590 | −332 | 59 | outside |

  - Both counts are over the 14,287 `ft/dev` targets. b is F hits with A misses, stored as `a_hit_b_miss_c`; c is the reverse, stored as `a_miss_b_hit_b`.
  - **On seed 2 the artifact beats its own float forward (c ≫ b).** Under the sheet's pre-registered reading, there the float model, not the artifact, is off: the weights adapted to the D11 forward.
  - **On seed 1 the artifact loses 278 net rows.**
  - **What it decides,** under the sheet's FAIL row: D11-mode training by this recipe (the bridges, Stage 4b's 1,024 steps and data) does not bring the gap inside the band. That closes this configuration, never the mechanism. The next step is the decision review's, below.
- **D11-mode training is unstable across seeds in plain float.** Its top-1 is 3,941 and 917, against the plain-trained trunk's 1,988 and 2,643. Its own two-seed loss band, |L_F(1) − L_F(2)|, is 0.0445, about Stage 4b's whole band of 0.0457.
- **This recipe made seed 2 worse in every forward, on top-1 and loss.** Against the plain-trained trunk:
  - plain float, 917 against 2,643;
  - export mode, 1,237 against claude-129's artifact (2,294);
  - artifact, 1,249 against 2,294;
  - L_F, 1.928 against 1.8316.

  Its admitted-target rate is also below seed 1's in each forward: 0.091 against 0.354 in plain float, and 0.115 against 0.334 in export mode.
- **The gap receipts carry the wrong sheet path.** They name the artifact-gap sheet, claude-129's scorer default, not this unit's sheet. The receipts are as the scorer wrote them, and this REPORT names the sheet that governs them.
- **The item's done-when is not met,** so #2820 stays open.

## What changed

Size against `origin/main`: float-train, in `hamilton_sheet.rs`, `geometric_stack_native.rs`, `spiralcore_fbs_sheet.rs` and the sheet binary. Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **`with_d11_training`** runs a training step's loss and backward in the D11 mode with geometric straight-through bridges:
  - **the gates** train through `Hamilton_IcosianHard`'s S³ tangent bridge;
  - **the transition snap** passes the unit normalization's gradient (`d11_transition_unit`), the same bridge;
  - **the Q12 query** passes its gradient straight through (`q12_query`).

  The flag is off outside it, so the evaluation's D11 forward is unchanged.
- **`run_fbs_arm_with(…, d11_train)`** trains an arm that way.
- **`d11-train-probe`** gives 32 D11-mode steps and the projected wall time.
- **`d11-train-export`** trains the export arm on both seeds, checks each saved model, and writes its snapshot and float predictions. Its receipt is in the shape claude-129's scorer reads.
- **Tests.**
  - The host test `d11_training_sets_and_restores_its_bridge` pins the flag.
  - The CUDA test `gpu_d11_bridges_pass_the_tangent_and_straight_gradients` pins both bridges:
    - their forwards equal the snaps;
    - the transition's gradient is the normalization's;
    - the Q12 gradient is the identity;
    - in evaluation, the snap passes no gradient.

## Measurements

**The loss half is inside the band on both seeds** (Δ_noise,L 0.0457):

| Seed | L_F | L_A | L_A − L_F |
|---|---|---|---|
| 1 | 1.8836242760872335 | 1.8859133148256773 | +0.0023 |
| 2 | 1.928099705005298 | 1.923851524814812 | −0.0042 |

**Every instrument check was green.**
- **`I-Bridges`:** the float-train library passed on the RTX 5090, 55 passed and 5 ignored. That includes the bridge test, the flag test, and claude-125's and claude-126's GPU D11 tests.
- **`I-Probe`:**
  - the D11-mode training loss fell from 2.8008 to 2.6160 over 32 steps;
  - the DEV float loss fell from 2.7939 to 2.6153;
  - a step took 0.137 s;
  - the full run projected to 377 s.
- **`I-Save-Load`:** both seeds' saved models reproduce their own float panel and loss.
- **`I-Verify-All`:** G = H = R at all 82,176 window positions on each seed. The evaluator's PTX `013d19fd…` is claude-128's qualified one.
- **`I-Inputs`:** the windows, each seed's exported artifact and the five data and receipt pins all match.
- **`I-DEV-Denominator`:** 14,287 rows.
- **`I-Finite`:** every step exited 0, and all 92 saved files verify against `SHA256SUMS`.

**Readings, with no bar:**
- **The artifact against the model's export-mode panel.** The artifact's top-1 (3,663 and 1,249) is within a few dozen rows of the same weights' export-mode panel (3,638 and 1,237). So after D11 training, what is left of the gap lies between the plain-float forward and the D11-like forwards of the same weights, not between the D11 forward and the artifact.
- **Against claude-129's direct export of the plain-trained trunk.** That artifact scored 1,907 and 2,294. The D11-trained artifact scores 3,663 on seed 1, almost twice as many rows, and 1,249 on seed 2.
- **The step-0 query credit** is in the probe receipt.
- **Wall time:**
  - 1,024 steps per seed took about 3 minutes.
  - The training step, both seeds with their in-training float and export-mode evaluations, took 6 m 19 s.
  - Each seed's verified evaluator run took about 9 m 44 s, mostly the CPU verify.

**Evidence:** the HF store at `results/claude/d11-fbs-d11-train-2026-10-26/`, each folder with a `SHA256SUMS`:
- `d11t/` holds every step's stdout, stderr and exit code, the probe, the training receipt, the saved models and snapshots, both artifacts, the windows, both evaluator runs, both gap receipts and `job.log`;
- `d11t-bin/` holds the three binaries run.

## Checks run at the head

See the exact-head review record on the PR.

**The code review's low findings,** recorded for the dual-forward unit and not changed after the run:
1. **No training trajectory.** `run_d11_train_export`'s receipt leaves out each seed's trajectory (`out.run`). So the training losses at the evaluation points, a reading the sheet names, are in the probe receipt only. The next unit should add `"run": out.run` to each seed's entry.
2. **The Q12 bridge and the clamp.** The straight-through Q12 bridge passes the gradient even where the clamp pins the forward (|q| ≥ 8). This is probably unreachable at these query scales, but worth a note.
3. **Test coverage.** The Q12 bridge is pinned only by the CUDA test, beside the transition bridge.

**Checks:**
- float-train library 39 on the host, and 55 on the GPU with 5 ignored, including both bridge tests;
- a stub-nvcc CUDA check, clean; `cargo fmt`, clean;
- `loop report --check` and `loop pr-check --local` pass, with advisory WARNs;
- `loop gates --full` exits 0.

## Skills used

- **holding-the-goal.** The bar stayed Stage 4b's, and so did the comparator: F, this candidate's own float forward. The pre-registered reading of the gap's direction was applied as written. Seed 2's c ≫ b is reported as the float model being off, not as an export success.
- **writing-meaningful-tests.** The bridge test pins both the forward and the gradient of each bridge, together with the no-gradient case in evaluation. It ran on the GPU before any training.

## Still open

- **The next configuration, by the decision review: dual-forward training.**
  - Each step's loss is the sum of the plain-float and the D11-mode `MarginHinge` losses, weighted 1:1 and pinned before the run, with the same bridges, data and steps. The gap is measured as before.
  - **Its sheet must:**
    - flag a PASS where either seed's artifact top-1 falls below the plain-trained artifact's (1,907 or 2,294); such a PASS does not close #2820 without a decision review;
    - report each seed's plain-float and export-mode top-1, admitted rate and two-seed loss band, with a stated reading.
  - CHARTER §3.7's comparator, the plain-float forward, is kept, so there is no owner question.
- **Seed 1's writer-address collapse** (claude-127) is still a hypothesis for the training line.
- **The item's "What is missing" text** is outside this ORDER's scope.

## Plain summary

The model was trained so that it already works the way the shipped integer file computes, and then exported and scored again.
- **The file is much closer to the model's own way of working.** Its accuracy matches the model's integer-like mode within a few dozen answers.
- **The comparison still fails.** The pre-registered test compares the file against the model's ordinary float mode, and training this way pulls that float mode away. In one run the float mode got 278 more answers right than the file; in the other it got 332 fewer.
- **The two training runs also disagreed with each other a lot.** So the next step is a decision about which of these to fix first.
