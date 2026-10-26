# Prep sheet: the export gap of a dual-forward-trained trunk (d11-exporter-float-trunk, claude-131), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-131](../../loop/tasks/d11-exporter-float-trunk/claude-131/ORDER.md).
- This is the configuration chosen by [claude-130's decision review](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6417189833) (D2), after the [D11-mode training sheet](d11-exporter-float-trunk-d11-train-2026-10-26.md) FAILed.
- Run: one job on one leased RTX 5090:
  - `spiralcore-fbs-sheet --train-forward dual`, with `--mode d11-train-probe` and then `--mode d11-train-export`;
  - `fbs-export` for each seed;
  - claude-128's `d11-fbs-eval run --verify`;
  - claude-129's `spiralcore-fbs-sheet --mode d11-artifact-gap`.

## 1. Question

**Does training on the plain-float and D11-mode losses together bring the exported artifact's export gap inside the band?**
- **claude-129,** the direct export of a plain-trained trunk: the loss half held, but the top-1 half missed, net 81 and 349 against F_I 60 and 70.
- **claude-130,** D11-mode training: the artifact landed within 25 and 12 rows of the model's own export-mode panel. But the plain-float forward, the comparator, drifted: net +278 and −332 against F_I 69 and 59, in opposite directions.
- **Here** each step trains both forwards, so the comparator is held while the artifact's own forward is trained.

## 2. Arms

There is one trained configuration, scored two ways:
- **F, its float model:** `Fbs_1Atom_Active`, trained for 1,024 steps per seed on Stage 4b's data, every step dual (`run_fbs_arm_with(…, TrainForward::Dual, …)`), scored in plain float on the 14,287-row `ft/dev` panel.
- **A, its exported artifact:** `fbs-export` of each seed's saved snapshot, run by the GPU evaluator over all 428 DEV windows, and scored by claude-129's scorer. The scorer reads the token at each scored offset, and the D11 head's loss on its Q12 queries.

**How it trains.** Each step's loss is **L = L_plain + L_D11, weighted 1:1, pinned here.**
- **L_plain** is the plain forward's `MarginHinge`, with its backward under the normal gate hook.
- **L_D11** is the D11-mode forward's `MarginHinge`: the scored query at Q12, with the gates and the transition snapped to the 120 icosians. Its backward runs inside `hamilton_sheet::with_d11_training`.
- **The two gradients are summed tensor by tensor** (`sum_grads`). Each backward runs in its own mode because the icosian gate's custom backward reads the hook state.
- **The bridges are claude-130's:**
  - the gates and the transition pass the S³ tangent bridge;
  - the Q12 query passes straight through where the i16 clamp does not bind, and passes nothing where it saturates, as its forward does.
- There is no softmax, sigmoid or smooth relaxation. The optimizer, data, steps, seeds and the rounding at export are all unchanged.

## 3. Bar and decision table

**The bar is Stage 4b's,** with F this configuration's own plain-float model (CHARTER §3.7's comparator, kept).
- **b** counts the `ft/dev` rows that F hits top-1 and A misses; **c** counts the reverse. Both are over the 14,287 targets.
- **F_I** = max(24, ⌈2√(b + c)⌉).
- **Δ_noise,L** = 0.045679637781173676, claude-126's two-seed DEV loss band: the largest |L_f32(a, 1) − L_f32(a, 2)| over Stage 4b's four arms, controls and nulls included. It is read from `d11-gap.json`, through the training receipt, and not re-measured.

| Gate | PASS when | PASS unlocks | FAIL means |
|---|---|---|---|
| `G-D11-ExportGap` | On **both** seeds, \|b − c\| ≤ F_I **and** \|L_A − L_F\| ≤ Δ_noise,L, with every instrument check of section 4 green | Unless the guard below flags it: the trunk's D11 export is inside the band. With the audits and the tokens per second of these artifacts on the laptop (`fbs-toks`), the item's done-when is met, and #2820 closes. | Dual-forward training by this recipe (1:1, the bridges, Stage 4b's steps and data) does not bring the gap inside the band. That closes this configuration, never the mechanism. The next step is chosen by a decision review; it is not fixed here. |

**The guard on a PASS (the decision review's condition 1), pre-registered.**
- A PASS on which either seed's artifact top-1 falls below claude-129's plain-trained artifact is **flagged**. The floors are 1,907 on seed 1 and 2,294 on seed 2.
- A flagged PASS does not close #2820. It goes to a decision review, which reads whether the gap closed only because both forwards got worse.
- An unflagged PASS closes #2820, with the PASS-only steps of section 6.
- The scorer does not compute the guard. The REPORT applies it, quoting each gap receipt's `top1_artifact` next to its floor.

**The direction of the gap, read as claude-130's sheet reads it.** The bar is two-sided. A FAIL with c ≫ b means the float model, not the artifact, is off. A FAIL with b ≫ c means the artifact loses rows to its float model.

**The seed spread, read out (the decision review's condition 2), pre-registered.** For each seed, the REPORT gives:
- the plain-float top-1 and the export-mode top-1;
- the admitted-target rate (`admitted_target_rate`) in each forward;
- the training losses at the evaluation points (`run.eval`).

It also gives this arm's own two-seed loss band, |L_F(1) − L_F(2)|, and reads it against two references:
- **The plain-trained export arm's own band, 0.0230** (claude-126's `Fbs_1Atom_Active`, |1.8546 − 1.8316|). A band at or below it reads as "no less stable across seeds than plain training". A band above it reads as "less stable", and the REPORT says so whatever the gate shows. By this rule claude-130's band, 0.0445, was less stable. Δ_noise,L (0.0457, the four-arm maximum) is the bar's figure, not this reference.
- **The plain-float top-1 spread, |F(1) − F(2)|,** against plain training's 655 (1,988 and 2,643) and claude-130's 3,024 (3,941 and 917). A spread nearer claude-130's than plain training's reads as "the dual loss did not cure the seed instability", and is reported as such.

**Other readings, with no bar:**
- per seed, A's top-1 hits and the sign of b − c;
- F's top-1 and loss against the plain-trained trunk's (claude-126: 1,988 and 2,643; 1.8546 and 1.8316) and against claude-130's (3,941 and 917; 1.8836 and 1.9281);
- the probe's step-0 query credit (`step0_query_credit_l2`).

## 4. Instrument checks (each must be green before the bar counts)

1. **`I-Bridges`,** before any training. On the pod, `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes. That includes:
   - `gpu_d11_bridges_pass_the_tangent_and_straight_gradients`: the bridged forwards equal the snaps, the gradients are the normalization's and the identity, and none passes where the Q12 clamp saturates;
   - `gpu_dual_gradients_add_tensor_by_tensor`: the dual step's gradient is the sum of the two backwards' gradients;
   - claude-125's and claude-126's GPU D11 tests, so the evaluation's D11 mode is unchanged.
2. **`I-Probe`.** `d11-train-probe --train-forward dual` must show all of these:
   - finite losses;
   - the dual training loss falling from step 0 to step 31 (`train_learns`);
   - the DEV float loss at step 32 below step 0's (`learns`);
   - a projected full run under an hour per seed (`under_one_hour_per_seed`).

   Any one false is a stop.
3. **`I-Save-Load`.** Each saved model reproduces its own float panel and loss exactly.
4. **`I-Verify-All`.** `d11-fbs-eval run --verify` gives G = H = R at all 82,176 window positions on each seed.
5. **`I-Inputs`.**
   - The loader checks the data pins.
   - The scorer gets each artifact's sha256, as `fbs-export` prints it, together with the windows file's.
   - The receipts record the models' sha256s, and each gap receipt names this sheet as its `training_sheet`.
6. **`I-DEV-Denominator`.** 14,287 rows over 428 windows.
7. **`I-Finite`.** Every step exits 0, and each step's stdout, stderr and exit code are saved. The log's last line is written before `SHA256SUMS`.

## 5. Base artifact and data split

**Base artifact:** none is loaded. The arm trains from step 0 at the pushed head. Its saved models, snapshots, artifacts and predictions are this run's own, and they go to the store.

**Data.** Stage 3's and Stage 4b's, from `w2-inputs/2026-10-25/driver-data/`:
- **train,** `mw-balp/train`: `tokens.u16`, sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; `response_mask.u8`, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`;
- **DEV,** `ft/dev`: `tokens.u16`, sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8`, sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

The noise band comes from claude-126's `d11-gap.json`, sha256 `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build, with nvcc, and copy all three with their sha256s:
   - `spiralcore-fbs-sheet` (float-train, `--features cuda`);
   - `fbs-export` (`uor-r4-integer`, `--features serving-wasm`);
   - `d11-fbs-eval` (the instrument, `--features cuda`).
3. Fetch the section 5 files and check them against their pins.
4. `I-Bridges`: the float-train CUDA tests.
5. `I-Probe`: `spiralcore-fbs-sheet --mode d11-train-probe --train-forward dual --data-root <driver-data> --out probe.json`. Stop if it fails.
6. `spiralcore-fbs-sheet --mode d11-train-export --train-forward dual --data-root <driver-data> --save-dir save --stage4b d11-gap.json --out train.json`.
7. For each seed:
   - `fbs-export save/seed-<s>/snapshot.json artifact-seed-<s>.uorlut`, recording its sha256;
   - `d11-fbs-eval run artifact-seed-<s>.uorlut windows.json run-seed-<s>.json --verify`, with `windows.json` from `d11-artifact-windows`;
   - `spiralcore-fbs-sheet --mode d11-artifact-gap … --model-dir save/seed-<s>/model --evaluator run-seed-<s>.json --windows-sha256 <sha> --artifact-sha256 <sha> --stage4b train.json --stored-preds save/seed-<s>/preds-float.json --seed <s> --out gap-seed-<s>.json`.
8. Save the run folder to the store at `results/claude/d11-fbs-dual-train-2026-10-26/`, with a `SHA256SUMS`.
9. **On an unflagged PASS only,** for the item's done-when:
   - `fbs-toks` on each seed's artifact on the laptop, reporting decode and context tokens per second separately (claude-126's step-6 protocol);
   - the zero-multiply audit, `scripts/audit_zero_matmul_serving.py <release rlib and uor-r4-stack> --stack`, at the head;
   - the integer serving library's FBS tests.

   The closing REPORT says `Closes #2820` only with all three.

**Minimum test power:**
- 2 seeds;
- 14,287 frozen DEV targets;
- 1,024 steps (786,432 tokens) per seed from step 0;
- every scored row read from the artifact's verified run, so P = 0.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.56 to $1.11/h, under a 2.5 h lease;
- about 10 minutes of builds;
- training: two forwards and two backwards per step. claude-130's D11-mode step took 0.137 s, and its 1,024 steps about 3 minutes per seed, so dual training should take about 6 minutes per seed. The probe projects the real figure.
- about 10 minutes per seed of verified evaluation (claude-130: 9 m 44 s);
- in all, about 1 h 10 m of pod time, under $2. If the probe projects nearer its one-hour stop, the lease is renewed, and the total could reach about $4. That is still inside the caps.

**Stop rule:**
- Stop and save on a failed `I-Bridges` or `I-Probe`, a non-finite loss, or any nonzero exit. Classify each as instrument or mechanism before calling it anything. A non-learning probe is a negative of this recipe only.
- Never cancel a healthy run because the lease ran short. Renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

**Round 1 (fresh non-author reviewer, 2026-10-26): NOT FIT, with two required text changes, both made.**
1. **The seed-spread loss reference.** The sheet compared this arm's two-seed band against 0.0457. That figure is Stage 4b's four-arm maximum, not plain training's own band. The reference is now the plain-trained export arm's band, 0.0230 (|1.8546 − 1.8316|), and Δ_noise,L is described as the four-arm maximum.
2. **The probe's command.** Step 5 now passes `--data-root` and `--out`.

**Also taken from it:** the REPORT applies the guard by hand, quoting each gap receipt's `top1_artifact` next to its floor.

**What it confirmed:**
- Every outcome changes a decision: an unflagged PASS closes #2820, a flagged PASS goes to a decision review, and a FAIL closes this configuration.
- The bar, F_I and Δ_noise,L match claude-129's scorer sheet. F is the candidate's own plain float (§3.7). The band flows from `d11-gap.json` into the scorer, unmeasured.
- The cited numbers hold against their sources: 1,907, 2,294, 655, 3,024, 1.8836 and 1.9281.
- The dual step is as stated: the plain backward runs outside the hook, the D11 backward inside `with_d11_training`, and the gradients are summed per tensor. The signal is lawful under §3.
- `I-Bridges` covers the gradient sum and the Q12 clamp. The cost and power are stated and inside the caps.

**Round 2 (fresh non-author reviewer, 2026-10-26): FIT.**
- All three round-1 changes are made, and the new text agrees with its sources.
  - The 0.0230 band is |1.8546 − 1.8316|.
  - claude-130's 0.0445 is above it, so it reads as less stable.
  - The receipt field is `top1_artifact`.
- Steps 5 and 6 run commands the binary accepts.
- One nit, kept for the REPORT: it computes |L_F(1) − L_F(2)| from the full-precision losses, not the four-place figures quoted here.

TEST FITNESS: FIT
