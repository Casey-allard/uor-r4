# Prep sheet: the export gap of a trunk trained in the D11 mode (d11-exporter-float-trunk, claude-130), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-130](../../loop/tasks/d11-exporter-float-trunk/claude-130/ORDER.md).
- This is the configuration that the FAIL row of [the artifact-gap sheet](d11-exporter-float-trunk-artifact-gap-2026-10-26.md) names: the trunk trained in the D11 mode before export.
- Run: one job on one leased RTX 5090:
  - `spiralcore-fbs-sheet --mode d11-train-probe`, then `--mode d11-train-export`;
  - `fbs-export` for each seed;
  - claude-128's `d11-fbs-eval run --verify`;
  - claude-129's `spiralcore-fbs-sheet --mode d11-artifact-gap`.

## 1. Question

**Does training in the D11 mode bring the exported artifact's export gap inside the band?**
- claude-129 measured the direct export of a plain-trained trunk. The loss half held, but the top-1 half missed on both seeds: net 81 and 349 against F_I 60 and 70.
- Here the same arm trains with every step in the D11 forward, so the trained weights already work under the artifact's snaps and Q12 query:
  - the Hamilton gates and the recurrence transition snapped to the 120 icosians;
  - the scored query at Q12.

## 2. Arms

There is one trained configuration, scored two ways:
- **F, its float model:** `Fbs_1Atom_Active` trained for 1,024 steps per seed on Stage 4b's data, every step in the D11 mode (`run_fbs_arm_with(…, d11_train = true)`), scored in plain float on the 14,287-row `ft/dev` panel.
- **A, its exported artifact:** `fbs-export` of each seed's saved snapshot, run by the GPU evaluator over all 428 DEV windows and scored by claude-129's scorer: the token at each scored offset, and the D11 head's loss on its Q12 queries.

**How it trains.** The gradient passes through the D11 forward's snaps by geometric straight-through bridges only, `hamilton_sheet::with_d11_training`:
- **the gates:** `Hamilton_IcosianHard`'s own S³ tangent-space bridge;
- **the transition:** the same bridge, through the unit normalization;
- **the Q12 query:** straight through.

There is no softmax, sigmoid or smooth relaxation. The exported tables are rounded at export, as before.

## 3. Bar and decision table

**The bar is Stage 4b's,** with F this configuration's own float model.
- **b** counts the `ft/dev` rows that F hits top-1 and A misses; **c** counts the reverse. Both are over the 14,287 targets.
- **F_I** = max(24, ⌈2√(b + c)⌉).
- **Δ_noise,L** = 0.045679637781173676, claude-126's two-seed DEV loss band. It is read from `d11-gap.json` and not re-measured.

| Gate | PASS when | PASS unlocks | FAIL means |
|---|---|---|---|
| `G-D11-ExportGap` | On **both** seeds, \|b − c\| ≤ F_I **and** \|L_A − L_F\| ≤ Δ_noise,L, with every instrument check of section 4 green | The trunk's D11 export is inside the band. With the audits and the tokens per second of these artifacts on the laptop (`fbs-toks`), the item's done-when is met, and #2820 closes. | D11-mode training by this recipe (the bridges, Stage 4b's steps and data) does not bring the gap inside the band. That closes this configuration. The next step is chosen by a decision review; it is not fixed here. |

**The direction of the gap, read this way, pre-registered.**
- F is the plain-float panel of a model trained under the snaps, so A may beat F (c > b).
- The bar is two-sided. A FAIL with c ≫ b means the float model, not the artifact, is the one that is off: the weights have adapted to the D11 forward. The decision review reads it so; it is not a failure of export.
- F stays the comparator, because CHARTER §3.7's gap is the float candidate against its artifact.

**Readings, with no bar:**
- per seed, A's top-1 hits and the sign of b − c;
- F's top-1 and loss, against the plain-trained trunk's (claude-126: 1,988 and 2,643; 1.8546 and 1.8316), to show how much the D11 forward costs the float model itself;
- this arm's own two-seed loss band, |L_F(1) − L_F(2)|;
- the training losses at the evaluation points, and the export-mode panel;
- the probe's step-0 query credit (`step0_query_credit_l2`), which shows a gradient reaches the routes.

## 4. Instrument checks (each must be green before the bar counts)

1. **`I-Bridges`,** before any training. On the pod, `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes:
   - `gpu_d11_bridges_pass_the_tangent_and_straight_gradients`, where the bridged forwards equal the snaps and the gradients are the normalization's and the identity;
   - claude-125's and claude-126's GPU D11 tests, so the evaluation's D11 mode is unchanged.
2. **`I-Probe`.** `d11-train-probe` must show all of these:
   - finite losses;
   - the D11-mode training loss falling from step 0 to step 31 (`train_learns`);
   - the DEV float loss at step 32 below step 0's (`learns`);
   - a projected full run under an hour per seed (`under_one_hour_per_seed`).

   Any one false is a stop.
3. **`I-Save-Load`.** Each saved model reproduces its own float panel and loss exactly.
4. **`I-Verify-All`.** `d11-fbs-eval run --verify` gives G = H = R at all 82,176 window positions on each seed.
5. **`I-Inputs`.**
   - The data pins are checked by the loader.
   - The scorer gets each artifact's sha256, as `fbs-export` prints it, together with the windows file's.
   - The receipts record the models' sha256s.
6. **`I-DEV-Denominator`.** 14,287 rows over 428 windows.
7. **`I-Finite`.** Every step exits 0, and each step's stdout, stderr and exit code are saved. The log's last line is written before `SHA256SUMS`.

## 5. Base artifact and data split

**Base artifact:** none is loaded. The arm trains from step 0 at the pushed head. Its saved models, snapshots, artifacts and predictions are this run's own, and go to the store.

**Data.** Stage 3's and Stage 4b's, from `w2-inputs/2026-10-25/driver-data/`:
- **train,** `mw-balp/train`: `tokens.u16`, sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; `response_mask.u8`, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`;
- **DEV,** `ft/dev`: `tokens.u16`, sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8`, sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

The noise band comes from claude-126's `d11-gap.json`, sha256 `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build, with nvcc:
   - `spiralcore-fbs-sheet` (float-train, `--features cuda`);
   - `fbs-export` (`uor-r4-integer`, `--features serving-wasm`);
   - `d11-fbs-eval` (the instrument, `--features cuda`).
   Copy all three with their sha256s.
3. Fetch the section 5 files and check them against their pins.
4. `I-Bridges`: the float-train CUDA tests.
5. `I-Probe`: `spiralcore-fbs-sheet --mode d11-train-probe`. Stop if it fails.
6. `spiralcore-fbs-sheet --mode d11-train-export --data-root <driver-data> --save-dir save --stage4b d11-gap.json --out train.json`.
7. For each seed:
   - `fbs-export save/seed-<s>/snapshot.json artifact-seed-<s>.uorlut`, recording its sha256;
   - `d11-fbs-eval run artifact-seed-<s>.uorlut windows.json run-seed-<s>.json --verify`, with `windows.json` from `d11-artifact-windows`;
   - `spiralcore-fbs-sheet --mode d11-artifact-gap … --model-dir save/seed-<s>/model --evaluator run-seed-<s>.json --windows-sha256 <sha> --artifact-sha256 <sha> --stage4b train.json --stored-preds save/seed-<s>/preds-float.json --seed <s> --out gap-seed-<s>.json`.
8. Save the run folder to the store at `results/claude/d11-fbs-d11-train-2026-10-26/`, with a `SHA256SUMS`.
9. **On a PASS only,** for the item's done-when:
   - `fbs-toks` on each seed's artifact on the laptop, reporting decode and context tokens per second separately (the step-6 protocol of claude-126);
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
- training projected by the probe; the plain arm took about 3 minutes per seed in Stage 4b, and the D11 forward adds the snaps;
- about 19 minutes per seed of verified evaluation (claude-129);
- in all, under $3 when training takes about 10 minutes per seed. If the probe projects nearer its one-hour stop, the lease is renewed, and the total could reach about $4. That is still inside the caps.

**Stop rule:**
- Stop and save on a failed `I-Bridges` or `I-Probe`, a non-finite loss, or any nonzero exit. Classify each as instrument or mechanism before calling it anything; a non-learning probe is a negative of this recipe only.
- Never cancel a healthy run because the lease ran short. Renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

**Round 1 (fresh non-author reviewer, 2026-10-26): FIT, with two required text changes, both made.**
1. **The direction of the gap.** The sheet now says how to read it: a FAIL with c ≫ b means the float model is off. A's top-1 and the sign of b − c are readings.
2. **The PASS-only steps.** `fbs-toks` (decode and context), the zero-multiply audit, and the integer FBS tests are now listed.

**Also taken from its recommendations:**
- this arm's own two-seed loss band, as a reading;
- `I-Probe` now records step 0's D11-mode training loss (`train_learns`) and stops on a projection over an hour per seed;
- the cost line now covers a lease renewal;
- the step-0 query credit is a reading.

**What it confirmed:**
- The training signal is lawful. The transition bridge is the gate's own straight-through S³ pattern, the Q12 rounding has no smooth part, and the loss stays MarginHinge.
- The backward runs inside `with_d11_training`.
- With the flag off, the evaluation's D11 forward is unchanged.
- The scorer reads `train.json`.

TEST FITNESS: FIT
