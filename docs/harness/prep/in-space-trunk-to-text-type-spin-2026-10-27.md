# Prep sheet: the Hamming read's type heads with SpiralCore's spin (in-space-trunk-to-text, claude-151, 27 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-151](../../loop/tasks/in-space-trunk-to-text/claude-151/ORDER.md). Code: branch `claude/type-head-spin`. Every pod builds and tests its own binary from the pinned commit first.

TEST FITNESS: FIT, after two fresh non-author rounds.
- **Round 1** found that no cell said what it decides, and that the job left I-2 on the arm, I-3 and I-4 unchecked. Both are fixed.
- **Round 2** closed those findings. Its D67 pre-run review failed on one point: that the chain is not on PLAN.md. Its brief had pointed it at PLAN.md lines 55–80. The chain is at PLAN.md line 38: the owner's 27 Oct direction row, which names [`docs/model/pipeline-map.md`](../../model/pipeline-map.md). The map's stage 5 and unit U1 are this read, and its line 259 puts U1–U3 first. The claude decision review that passed the run on that record is logged on #2028.

## The question

**Does a position signal from SpiralCore's spin in the Hamming read's type heads let replies use the prompt, measured against the same read without it?**

The owner's direction (27 Oct): "give its type heads a position signal from SpiralCore's spin, so replies can use the prompt." claude-146's read (`df1d7892`, merged) solves recall through its copy heads, but on language it landed in cell D at two seeds of three: openings passed at seed 1 only, reach at none. The type heads must learn which prompt word matters, and have no position signal.

**The chain:** PLAN.md's 27 Oct owner-direction row (line 38) leads to [the pipeline map](../../model/pipeline-map.md). There, stage 5 (look-back) is this read, and U1–U3 come first "because nothing downstream means anything until the prompt reaches the reply" (line 259). The one hint: the pre-review form, whose type heads were turned by a rotary per-index code, had seed 2 at 214 openings at step 6,144 where the merged form had 161.

## Steps: the arms

One arm against its twin, at each seed:

`train-in-space --objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --hamming-read 4 --type-spin 1 --seed S`, for `S` in 1 and 2 (3 on a split), one arm per RTX 5090.

**The twins** are claude-146's runs of the same read without the spin (`--type-spin` absent), at the same settings and seeds, already measured: `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/` (`lang-df1d7892/hamming-s1`, `hamming-s2`; `lang-s3-df1d7892/hamming-s3`; sealed in `SHA256SUMS-claude-146-lang`).

| Seed | Twin nearest top-1 `T` | Twin openings `O` | Twin `R16` |
|---|---|---|---|
| 1 | `8,687` | `298` | `0.0218` |
| 2 | `8,709` | `200` | `0.0327` |
| 3 | `9,233` | `302` | `0.0268` |

The spin (`flock_read_hamming.rs`, `with_type_spin`): each token steps each type-head lane by the unit icosian (2I) nearest its table point through a learned per-lane rotation; a position's frame `F_t` is the running product of its window's steps; a type head turns its query at `t` by `F_t⁻¹` and its key at `s` by `F_s⁻¹`, so the angle between them depends only on `F_t ⊗ F_s⁻¹`, the steps of the tokens between them. The read compares sign codes, which also see the frame the two share, so the Hamming distance follows those steps on average over that frame, not exactly. Copy heads are untouched.

Each arm also runs `--reach` on its step-0 and step-16,384 checkpoints, `--inspect`, and `--decode-speed` on the last. The twins never ran `--decode-speed`, so each pod also fetches its twin's step-16,384 checkpoint and times it with the same binary (a checkpoint without `step_rotation` loads with the spin off), so the decode comparison is one build on one GPU.

## Bar (pre-registered), each arm against its twin at step 16,384

claude-145's decision table, with the twin now the read without the spin. `O` is `dev_nearest.opening_hits`, `R16 = shift(16) / shift(0)` from `--reach`, `T` is `dev_nearest.hits`.

- **Openings pass:** `O ≥ max(197, O_twin + 36)`: `334`, `236` and `338` at seeds 1, 2 and 3.
- **Reach pass:** `R16 ≥ max(0.05, 2 · R16_twin)`: `0.05`, `0.0655` and `0.0535` (twin `shift(16) / shift(0)`: `0.11904 / 5.46192`, `0.17832 / 5.44788`, `0.14712 / 5.49805`).

| | Reach pass | Reach fails |
|---|---|---|
| **Openings pass** | **A.** The spin lets openings use the prompt. | **B.** Openings improve from nearer context. |
| **Openings fail** | **C.** The spin reaches farther, but openings do not follow. | **D.** The spin does not help at this configuration. |

What each cell decides:

A, B and C each make the spin the read's default (`--type-spin` on in every later language run); D does not. With flag F set, the default waits until fit is restored.

- **A:** the spin lets openings use the prompt. A step-16,384 checkpoint of a seed in A that also has `T ≥ 7,002` (the M1 count table) meets the item's done-when 1 and 2, since every openings bar is at least `197` and every reach bar at least `0.05`. The item's next unit is then done-when 3 and the serving change to whole-table nearest. U3, the integer Hamming read, carries the spin.
- **B:** the next unit is reach: why the spin's gain is near, read at distances past 16.
- **C:** the next unit is the type heads' payload (what they bring back, `v_s`), not position.
- **D** closes this configuration (the spin as built, at these settings), never the mechanism. The next unit is a decision review of what the type heads lack, before any other position form.
- **Flag F**, reported beside the cell: `T < T_twin − 500`, meaning the spin costs fit. In A, B or C with F set, the next unit first restores fit with the spin on.
- Seeds 1 and 2 must land in the same cell for the reading to count. If they split, seed 3 runs and the cell two of three share is the reading. If all three differ, there is no cell.
- A missing number gives no cell: that step is rerun, never read as a fail.

**Reported, not barred:** openings and distinct picks at every checkpoint; the greedy replies (`samples.md`); the read's weights; training and decode tokens/s against the twin's.

## Instrument checks

- **I-1:** the crate's CUDA library tests pass on each pod's build, including the spin's five tests. Two negative gates each set one rule aside and must make their test fail (code 101), each checking first that its patch changed the file:
  - the scan's order reversed (`earlier ⊗ later`): `the_frames_are_the_running_product_of_the_steps`;
  - the inverse dropped (the conjugate's signs all `+1`): `the_relative_frame_ignores_every_step_at_or_before_the_earlier_position`.
- **I-2:** each arm's step-0 `dev_nearest` equals its twin's (every read starts with a zero weight, so the trunk is the same at step 0): seed 1 nearest 2, openings 0, 730 picks; seed 2 nearest 5, openings 3, 681 picks; seed 3 nearest 0, openings 0, 735 picks. The job checks it at the probe, before the arm, and again on the arm.
- **I-3:** the read's weight moves off 0: `--inspect`'s `hamming_weight_mean_abs` at step 16,384 is above 0. The job checks it.
- **I-4:** no non-finite loss: every `train_loss_mean` and `dev_loss` in the arm's progress is a finite number. The job checks it.
- A failed I-2, I-3 or I-4 is an instrument fault: the arm gives no cell, and is repaired and rerun.

## Probe ladder

1. **The build, the CUDA tests and the two negative gates** on each pod.
2. **A 256-step throughput probe** at each seed, before its long arm. Each arm runs alone on its GPU. The one twin that trained alone did 32,013 context tokens/s (seed 3; seeds 1 and 2 shared a GPU at 14,959). If the probe is under 16,000 tokens/s, half the alone rate, the job stops and the scan is profiled first (ORDER, stop and decide).
3. **The arm.**

## Data split

The pinned TRAIN and DEV splits the in-space trunk always uses, checked by sha256 at load (`hamilton_sheet.rs`: `PINNED_TRAIN_TOKENS_SHA256`, `PINNED_TRAIN_MASK_SHA256`, `PINNED_DEV_TOKENS_SHA256`, `PINNED_DEV_MASK_SHA256` and `PINNED_TOKENIZER_SHA256`), from `caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`. DEV has 14,287 scored rows, 1,226 of them reply openings.

## Base artifact

None. Each arm starts from `build_fbs_model(OneAtomActive, seed)`, as its twin did.

## Minimum test power

Openings: 1,226 rows; 36 is three binomial standard deviations at the commonest opening's rate, as in claude-145's sheet. The twin is a separate run, so run-to-run noise does enter the comparison: claude-140 measured this training as nondeterministic. That is why two seeds must agree and a split goes to a third.

## Cost

Two RTX 5090 pods at about `$0.56/h`. 16,384 steps × 16 × 192 tokens is about 50M context tokens per arm; at the twins' 32,000 tokens/s alone that is about 26 minutes, plus 9 evaluations. Projected under `$1.50`, a third seed included; the probe checks the rate before the long arms.

## Stop rule

- Stop on a non-finite loss, a crash, a failed negative gate, or a probe slower than half the twin's rate. A healthy run is never cut short.
- Results go to `caseyallard/uor-r4-store:results/claude/type-spin-2026-10-27/`, sealed with a SHA256SUMS, and are read back against the pod before any pod comes down.
