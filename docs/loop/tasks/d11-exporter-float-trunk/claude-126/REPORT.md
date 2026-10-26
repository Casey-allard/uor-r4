# REPORT: d11-exporter-float-trunk, claude-126

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [Stage 4b](../../../../harness/prep/d11-exporter-float-trunk-stage4b-2026-10-26.md). It was FIT in round 3, after two NOT_FIT rounds (#2820 comments `6411168367`, `6411285400`, `6411318314`). The D67 pre-run review was GO (#2820 comment `6411350681`).

References #2820.

## What did not work, first

- **`G-D11-ExportGap` is blocked.** The exported artifact does not compute its own specification on a trained trunk, so its export gap could not be measured.
  - The sheet's instrument check `I-D11-Artifact-Agreement` failed on both seeds by a wide margin. Under the sheet's rule the gap is not counted, and it is not a FAIL.
  - m1 holds the GPU panel's D11 predictions against the D11 readout; m2 holds the D11 readout's tokens against the exported artifact's integer session, on the pod with `fbs-compare --parity-only`. The bar was P ≤ 6, where P = 14,287 · (m1/n₁ + max(m2ₛ/n₂ₛ, m2_q/n₂_q)):

    | Seed | m1 (panel against readout) | m2, single tokens | m2, DEV-window positions | P | Largest query gap |
    |---|---|---|---|---|---|
    | 1 | 0 of 14,287 | 1,283 of 4,096 (31%) | 6,539 of 10,368 (63%) | about 9,011 | 10.95 |
    | 2 | 0 of 14,287 | 59 of 4,096 (1.4%) | 170 of 10,368 (1.6%) | about 234 | 13.45 |

  - m1 = 0 on both seeds. The batched panel and the one-window readout agree exactly, so the disagreement lies between the D11 readout and the artifact.
  - The artifact checked is the one served: on both seeds, the sha256 of `artifact.uorlut` equals `fbs-compare`'s `artifact_sha256`.
- **The diagnosis, on seed 1: the integer artifact departs from its own specified semantics; the D11 mode does not.**
  - **The check.** I ran `fbs-compare`'s full mode on seed 1. It holds both the D11 readout and the integer session against the f64 quantized reference, which is the artifact's specification.
  - **Where it ran.** This was a diagnostic host check on the laptop, run to localize an instrument fault. It is not a result and feeds no bar.

  | Seed 1 pairing | Single tokens agree | DEV-window positions agree |
  |---|---|---|
  | D11 GPU readout against the quantized reference | 4,095 of 4,096 (largest gap 3.46, at 2 positions over the 1e-3 bar) | 10,368 of 10,368 (largest gap 0.0063, at 1 position over the 1e-3 bar) |
  | Integer session against the same reference | 2,812 of 4,096 (largest gap 4.89) | 3,829 of 10,368 (largest gap 10.95) |
  | Plain GPU against the float reference | 4,089 of 4,096 (largest gap 8.9e-6) | 10,352 of 10,368 (largest gap 1.0e-5) |

  - **What follows, inferred.** The defect is in the integer FBS runtime or in the exporter, which is this item's own deliverable.
  - **A hypothesis, untested here:** the cause is the trained trunk's scale, because earlier trunks were far smaller. Nothing in this run varied the scale:

    | Trunk | Width | Layers | Vocabulary | Largest embedding | Largest RMS gain (any norm) | Largest decay offset |
    |---|---|---|---|---|---|---|
    | Stage 4a random-init (claude-125) | 32 | 2 | 256 | 0.074 | 1 | 0.47 |
    | This run's trained trunk, seed 1 | 256 | 6 | 4,096 | 1.71 | 2.39 | 2.83 |
    | This run's trained trunk, seed 2 | 256 | 6 | 4,096 | 1.70 | 2.41 | 3.53 |

  - **Not yet localized.**
    - The integer RMS norm requantizes to the smallest exponent that fits (`stack_rms_norm`), so it does not clip at ±8. That is from reading the code, not from a measurement.
    - The diagnostic counted 0 snaps of a zero vector, so it is not the known GPU-versus-integer zero-snap edge either.
    - Seed 2 was not diagnosed.
- **The FIT took three rounds, and each round strengthened the instrument.**
  - **Round 1 (NOT_FIT).** The only tie to the artifact was a 320-position parity with a tolerance of about 2%, against a bar resolving 0.17%. The reviewer also proposed laptop parity, which CHARTER §3.5 rules out.
  - **Round 2 (NOT_FIT).** m1 had the wrong denominator. The f64 reference sections of `fbs-compare` are host fallbacks even on the pod, so they were removed (`--parity-only`).
  - **Round 3: FIT.** It came with three stricter recommendations, all adopted before the run.

  Without that chain, this run would have reported a clean top-1 FAIL about the GPU emulation, not about the shipped file.
- **A correction to claude-125.** Its `traced/` and `ties/` analyses re-ran the f64 reference on the laptop, which CHARTER §3.5 puts on the GPU.
  - Their numbers stand: the new single-pass trace reproduces them byte for byte.
  - This unit's own laptop diagnostic is the one exception. It is labelled as such, and no result rests on it.
- **The lease watcher did not work.** The harness's lease watcher (loop drive in its watch-and-lease mode, handed the uor-pod renew command) exited at once ("renew exited exit status: 1"), while the same renew run directly succeeded. A bounded renew loop kept the lease instead. The case is in FRICTION.md.

## Readings, not verdicts

- **The D11 GPU mode's gap.**
  - On seed 1, the D11 GPU mode matches the artifact's specification on all but 1 of 14,464 positions (the diagnosis above). There, its top-1 gap to plain float is the specification's own: what post-training rounding into the D11 semantics costs.
  - On seed 2 the D11 mode was not checked against the specification, so its gap is the D11 GPU mode's only:

    | Seed | Float top-1 | D11 top-1 | b | c | Net | F_I | Loss float | Loss D11 | Δ_noise,L |
    |---|---|---|---|---|---|---|---|---|---|
    | 1 | 1,988 | 1,911 | 483 | 406 | 77 | 60 | 1.8546 | 1.8785 | 0.0457 |
    | 2 | 2,643 | 2,290 | 755 | 402 | 353 | 69 | 1.8316 | 1.8628 | 0.0457 |

  - **On top-1:** both nets are beyond F_I. If the artifact computed its specification, the top-1 side would miss the bar. For seed 2, that also assumes the D11 mode matches the specification there, which was not checked.
  - **On loss:** both gaps are inside the band.
  - This is not the gate's verdict, because the instrument check failed.
- **Stage 3's export mode on the same checkpoints.** That mode snaps the gates and rounds the writer only.
  - Top-1 is 2,027 and 2,444.
  - Against float, b and c are 379 and 418 (net −39) on seed 1, and 583 and 384 (net 199) on seed 2.
  - The D11 mode's full rounding (the transition snap and the rounded tables as well) costs more than the export mode on both seeds.
- **The trained arms' readings** match Stage 3's pattern. The frozen-routing null has the most top-1 (5,561 and 5,516); the active arm has 1,988 and 2,643; the identity-transport control has the lowest float loss (1.8185 and 1.7921). This unit compares no arms, so no fit verdict is drawn.
- **Tokens per second of the exported artifacts on the laptop** (`fbs-toks`, Apple M1 with 4 performance cores, 16 GiB, macOS 27.0, AC power; 1 warmup and 20 timed runs):

  | Seed | Context (128-token prefill), median / p95 | Decode (64 tokens), median / p95 |
  |---|---|---|
  | 1 | 232 / 155 tok/s | 231 / 134 tok/s |
  | 2 | 269 / 219 tok/s | 268 / 235 tok/s |

  These time the integer runtime for this shape (width 256, 6 layers, vocabulary 4,096). The artifact they time departs from its specification, so they are not a qualified serving rate.
- **GPU speed** (one RTX 5090):
  - training 20,113–24,524 tok/s across the 8 arm-seeds;
  - the float DEV panel at 87,265 and 89,923 tok/s;
  - the D11 panel at about 5,691 tok/s on both seeds.

## What changed

Size against `origin/main`:
- code: +994/−134 in 8 files, in-file test modules included;
- tests: +42 in `stack/tests.rs`;
- docs: the ORDER, the prep sheet, this REPORT and the notebook row.

**`uor-r4-float-train`:**
- **`spiralcore-fbs-sheet --mode d11-export-gap --save-dir DIR`** (`run_d11_export_gap`). It trains the four Stage 3 arms on both seeds, for the noise band. For `Fbs_1Atom_Active` on each seed it:
  - saves and reloads the model, with `I-Save-Load` checking predictions and loss;
  - rounds it as exported;
  - scores `evaluate_dev_panel_d11` twice (`I-D-Determinism`) and computes `panel_loss_d11`;
  - holds the panel against the D11 readout on every scored row (m1);
  - writes the snapshot, the readouts over 4,096 single tokens and every eighth DEV window, and both prediction sets.
- **`panel_loss_d11`:** the D11 loss, with each scored query at Q12. It refuses an unrounded trunk.
- **`export_gap_within`:** one implementation of the export-gap bar, shared with Stage 3's gate.
- **`fbs_float_readout_on` and `fbs_float_next_tokens`:** readouts over chosen sequences.
- **`hamilton_sheet::clear_active_arm`.**

**`uor-r4-integer`:**
- **`FbsTrunkSnapshot::forward_f32_each` and `tie_margins_each`:** the reference over many inputs, with the exporter's checks and the tables read once, and per-position tie margins in one pass. A test checks they equal the per-input runs.
- **`fbs-compare --parity-only`:** the D11 readout against the integer session alone, with no float reference.
- **`fbs-toks ARTIFACT`:** the step-6 serving-rate protocol for a stack artifact's integer session.

## Measurements

- **The run.**
  - One RTX 5090 (`vast-57786001`, driver 580.95.05, nvcc 12.8) at `80be868964622bf357b5905e93abe72ef9280a66`, from 08:17:39Z to 08:41:02Z, about 23 minutes of job time. The pod was up about 30 minutes, at about $0.99/h; it is deleted.
  - The job ran probe a (37 GPU tests ok; `synthetic-probe` pass), probe b (DEV denominator ok; projected 964 s) and the run.
  - The binaries: `spiralcore-fbs-sheet` `582357fc…`, `fbs-export` `b33848d3…`, `fbs-compare` `8834886c…`.
- **GPU instruments, all green on both seeds:**
  - `I-Save-Load`: predictions and loss identical;
  - `I-D-Determinism`;
  - `I-DEV-Denominator`: 14,287 of 14,287;
  - `I-Q12-Oracle`: 517 rows, 0 mismatches.
- **Evidence** is in the HF store under `results/claude/d11-fbs-export-gap-2026-10-26/`, each folder with a `SHA256SUMS`:
  - `d11gap/`: the run, the saved models, snapshots, readouts, predictions, artifacts and both comparisons;
  - `d11gap-bin/`;
  - `toks/`;
  - `diag/`: the laptop diagnostic, labelled as such.

## Checks run at the head

See the exact-head review record on the PR.
- **Suites:** float-train 37; integer serving library 323; no-feature library 55; `m5_full_path_cost_controls` 3/3; `memory_shape` 3/3; `fbs-compare` 1; `fbs-toks` 2.
- **The set-asides, each failing its test:**
  - the bar's loss side (also failing Stage 3's gate test) and its net side;
  - the D11 loss guard;
  - the panel-to-readout indexing and sampling;
  - the per-position margin record;
  - the `fbs-toks` median, decode feedback and reply check.
- **The new single-pass trace** reproduces claude-125's stored trace byte for byte, and `--parity-only` reproduces the full mode's D11 pairings.
- **Build checks:**
  - CUDA type-check with a stub nvcc: exit 0;
  - clippy: no findings in the changed code;
  - `cargo fmt` clean;
  - zero-multiply audits FULL PASS (rlib 56 and 40; `uor-r4-stack` 44 and 98).
- **Loop checks:** `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** The goal was the artifact's own gap, not the GPU emulation's. That is why three FIT rounds tightened the tie to the artifact, and why a clean-looking FAIL was not reported.
- **writing-meaningful-tests.** Every new check was set aside to show that its test fails.

## Still open

- **Next unit: repair the integer FBS runtime at trained scale.**
  - Localize where `IntegerStackModel`'s FBS step departs from the quantized reference: a finite host control at trained scale (width 256, 6 layers, trained value ranges), compared stage by stage.
  - Repair it, with the control as its failing-first test.
  - Re-run only the pod parity on the stored snapshots and readouts. No retraining is needed, because the saved models are in the store.
  - Then read the verdict. The readings above point to a top-1 FAIL, which would move this trunk to training in the D11 mode.
- **The mechanism page and the item's "What is missing" text** are outside this ORDER's scope (claude-121's note). The next REPORT carries them.

## Plain summary

The plan was to measure how much a trained model loses when it is shipped as the integer file that runs on devices.
- **The GPU's emulation of that file** works. It matches the file's own specification almost exactly.
- **The file itself does not.** On a trained model, which is far larger than the test models used before, the integer runtime computes something different from its specification. So the loss could not be measured yet.
- **If the file did compute its specification,** the emulation suggests it would lose about 77 and 353 correct answers out of about 14,000 on the two training runs, more than the allowed margin. The emulation was checked against the specification on the first run only.
- **Next:** fix the integer runtime for models of this size, check it again on the GPU, and then read the result.
