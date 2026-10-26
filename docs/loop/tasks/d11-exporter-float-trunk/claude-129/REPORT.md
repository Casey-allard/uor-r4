# REPORT: d11-exporter-float-trunk, claude-129

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-artifact-gap-2026-10-26](../../../../harness/prep/d11-exporter-float-trunk-artifact-gap-2026-10-26.md) (FIT round 1; D67 pre-run review GO).

References #2820.

## What did not work, first

- **`G-D11-ExportGap` FAILs on both seeds, on its top-1 half.** The exported artifact of the trained `Fbs_1Atom_Active` trunk loses more top-1 rows than the paired bar allows:

  | Seed | Float top-1 | Artifact top-1 | b (F hits, A misses) | c (A hits, F misses) | Net b − c | F_I | Top-1 half |
  |---|---|---|---|---|---|---|---|
  | 1 | 1,988 | 1,907 | 477 | 396 | 81 | 60 | outside |
  | 2 | 2,643 | 2,294 | 772 | 423 | 349 | 70 | outside |

  Both counts are over the 14,287 `ft/dev` targets. The receipts store b under the key `a_hit_b_miss_c` and c under `a_miss_b_hit_b`, with float as A and the artifact as B. Their `_c` and `_b` suffixes run opposite to the sheet's letters.
  - **This counts as a result:** every instrument check was green (below).
  - **What it decides,** under the sheet's FAIL row: direct post-training rounding into the artifact is disqualified for this trained configuration (CHARTER §3.7). The next configuration trains in the D11 mode before export.
  - **What it does not do:** it closes this configuration, never the mechanism.
- **The item's done-when is not met,** so #2820 stays open.

## What changed

Size against `origin/main`: float-train +339/−16, in `spiralcore_fbs_sheet.rs` (+252/−12, including the test) and the sheet binary (+87/−4). Docs: the ORDER, the prep sheet, this REPORT and the notebook row.

- **`load_dev`:** the DEV half of `load_inputs`, factored out, so the new modes read no training split.
- **`d11_artifact_windows`:** writes the 428 DEV windows in panel order, refusing any count other than 14,287 rows. It runs as the `--mode d11-artifact-windows` of `spiralcore-fbs-sheet`.
- **`run_d11_artifact_gap`:** scores one seed's `d11-fbs-eval run --verify` output under Stage 4b's unchanged bar, as `--mode d11-artifact-gap`, with `--model-dir`, `--evaluator`, `--windows-sha256`, `--artifact-sha256`, `--stage4b`, `--stored-preds` and `--seed`.
  - It takes the artifact's prediction at each scored offset.
  - The loss is the D11 head on the exported tables, fed the artifact's own `q / 4096`.
  - It checks the instruments below.
- **`artifact_rows`:** the row mapping, pinned by `the_artifact_is_read_at_each_scored_offset_in_panel_order`, which fails when the mapping is read one position late.

## Measurements

**The loss half is inside the band on both seeds.** The noise band is Δ_noise,L = 0.045679637781173676, claude-126's two-seed band.

| Seed | L_F (float) | L_A (artifact) | L_A − L_F |
|---|---|---|---|
| 1 | 1.854587143852753 | 1.8783489453447968 | 0.0238 |
| 2 | 1.8315854149084358 | 1.862535382815703 | 0.0309 |

**Every instrument check was green on both seeds:**
- **`I-Verify-All`:** G = H = R at all 82,176 positions of the 428 DEV windows of 192: the GPU evaluator, its host twin and the Rust integer session. So every scored row is the shipped runtime's own output (P = 0).
  - The evaluator's PTX `013d19fd…` and cubin `73eccb70…` equal claude-128's qualified ones.
- **`I-Save-Load`:** each saved model reproduces claude-126's stored float panel row for row, and its loss bit for bit (L_F equals the Stage 4b receipt's `loss_float` exactly).
- **`I-Inputs`:**
  - The evaluator read the sheet's windows (`windows.json` sha256 `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e`).
  - It ran each seed's pinned artifact (`314c3909…` and `d5b43596…`).
  - All 11 store inputs matched their pins.
  - The model files' sha256s are in each receipt.
- **`I-DEV-Denominator`:** 14,287 rows over 428 windows.
- **`I-Finite`:**
  - every step exited 0;
  - all 81 saved files verify against `SHA256SUMS`, including the log, whose last line was written before the hashes (the fix from claude-128).

**Against Stage 4b's D11 emulation,** which could not be counted:
- The artifact's own top-1 lands within a few rows of the emulation's: 1,907 against 1,911, and 2,294 against 2,290.
- So do its net gaps (81 against 77, 349 against 353) and its loss gaps (0.024 and 0.031 for both).

The emulation's disagreement with the artifact, about 1% of tokens, barely moved the counts. What Stage 4b lacked was the right to count its reading, not a different answer.

**Evaluator throughput, on one RTX 5090** (driver 595.80, nvcc 12.8):
- 428 windows of 192 took 14.39 s and 14.36 s of GPU wall time, in one launch.
- claude-128's REPORT inferred about 15 s for that launch and projected 115 s linearly; the measured times fit the inference.
- Each seed's evaluator step, GPU and the CPU verify against both references together, took about 19 minutes. Nearly all of that is the CPU verify.

**No tokens per second are newly measured.** This diff changes no runtime code, and claude-128 added only a read-only accessor. So the serving arithmetic is claude-127's, the one its laptop figures time.

**Evidence:** the HF store at `results/claude/d11-fbs-artifact-gap-2026-10-26/`, each folder with a `SHA256SUMS`:
- `gap/` holds every step's stdout, stderr and exit code, `windows.json`, both evaluator runs, both receipts (`gap-seed-<s>.json`) and `job.log`;
- `gap-bin/` holds the two binaries run: `spiralcore-fbs-sheet` `5a1e925c…` and `d11-fbs-eval` `67dd684a…`.

## Checks run at the head

See the exact-head review record on the PR.

**The code review's low findings, recorded and not changed after the run,** so that the head's scorer is the one that produced the receipts:
1. **The windows hash.** The scorer compares the evaluator's `sequences_sha256` with the `--windows-sha256` it is given; it does not rebuild the windows itself. In this run, the job wrote `windows.json` with the sheet's own `d11-artifact-windows` mode and hashed it in the same job (`windows.sha256`), and the evaluator recorded that hash. A later scorer should rebuild the hash from the panel.
2. **The window count.** The code checks the 14,287 rows, not the 428 windows the sheet names. The receipts record `dev_windows` 428.
3. **The sheet's step 5** leaves out `--artifact-sha256`, which the binary requires. The job passed it for each seed.
4. **The new options** are accepted by every mode, and ignored outside `d11-artifact-gap`.

**Checks:**
- **Suites:** float-train library 38, including `the_artifact_is_read_at_each_scored_offset_in_panel_order`, which fails when the mapping is read one position late.
- **Build:** a stub-nvcc CUDA check of float-train, clean; `cargo fmt`, clean.
- **Loop:** `loop report --check` and `loop pr-check --local` pass, with an advisory WARN for the sheet's "Cost and stop rule" heading; `loop gates --full` exits 0.

## Skills used

- **holding-the-goal.** The goal was the artifact's own gap, so the evaluator's output was verified at every position before anything was scored. The emulation's near-identical reading is reported, but it is not counted.
- **writing-meaningful-tests.** The one piece of new logic a GPU run could not check by itself, the row mapping, has a host test that fails when the mapping is set one position off.

## Still open

- **The next configuration: train in the D11 mode before export** (the sheet's FAIL row; CHARTER §3.7).
  - The trunk would see the artifact's computation during training: snapped gates and transport, rounded tables, and the Q12 query.
  - Its gap would then be read with this same evaluator and scorer.
  - That is a new training configuration, so it needs its own prep sheet and FIT.
- **Seed 1's address collapse** (claude-127) is still a hypothesis for the training line. A D11-mode trunk would show whether training keeps the address channels on the root shell.
- **The item's "What is missing" text** is outside this ORDER's scope. It should now say that the GPU evaluator exists, and that the direct export of this configuration has been measured and is outside the band.

## Plain summary

The shipped integer file of the trained model was run on a GPU, checked against the laptop runtime on every one of 82,176 positions, and scored against the original float model.
- **Fine on loss:** its loss on the held-out panel rises by only 0.024 and 0.031, inside the run-to-run noise.
- **Not on accuracy:** it gets 81 and 349 more held-out answers wrong than the float model does. The pre-registered limit was 60 and 70.
- **The decision:** rounding the trained model straight into the integer file costs too much accuracy for this configuration. The next attempt trains the model on the integer file's own arithmetic before exporting it.
