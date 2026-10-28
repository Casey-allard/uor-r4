# Prep sheet: the prompt register, learned slots over the user's turn (in-space-trunk-to-text, claude-155, 28 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-155](../../loop/tasks/in-space-trunk-to-text/claude-155/ORDER.md), with Casey's words verbatim. Code: branch `claude/prompt-register` at `7e445dae83eacf3c3e5c6a7f51c3108ef00c3479`: the register is `06dd98549` and its wiring `7e445dae8`. Host checks at that commit: the crate's host tests pass (90 library tests), two negative gates bite (picks taken from the whole window, and the checkpoint load removed), and the anchor check still agrees at all 1,226 DEV and 212,968 TRAIN openings. Each pod builds and tests its own binary from that commit.

TEST FITNESS: FIT, and PRE-RUN: PASS, by one fresh non-author reviewer (light gates, per the owner's 27 Oct direction). Its notes:
- three numbers it did not recompute were checked from their sources: (a)'s `302.0 / 228.0` from claude-152's sealed tars, and flag F's `9,328` from claude-152's REPORT; the test count is checked by I-1;
- the caps, checked before launch: one pod up, `$0.56/h` of `$8/h`;
- I-1's wording, fixed below;
- the tight starting margin, a note: a slot that stays near zero shows as a near-zero drop, which the sheet reports as a finding.

## The question

**Do learned slots over the user's turn, kept separate and reaching the readout, lift reply openings above claude-152's (a)?** And is the gain from learning which words to read, rather than from seeing all of them?

**What came before.**
- claude-145's probe: the prompt's first 8 tokens name `607` of DEV's 1,226 openings by lookup.
- claude-154 handed the trunk exactly those 8 tokens, averaged into one vector after layer 2. Openings fell to `206.0 / 156.3`. Taken out of the trained model, the read changed `25 / 19` of 14,287 tokens: the trunk ignored it.

The register changes the two things that read did not have:
- the turn arrives as 8 separate slots, each with its own weights;
- every slot also reaches the readout, so it can shape the decoded point directly.

## The mechanism

`--prompt-register Q` (the ORDER gives the exact rule). After layer 2, for each position `t`:
- **The span** is the user's turn so far: from the turn anchor (after the last `<|bos|>` or `<|eos|>`) to `t`.
- **Keys:** each position of the span has a 64-bit code. The code is the sign of each lane's real part after a learned quaternion turn of the stream there.
- **Picks:** each of Q slots has a learned 64-bit query. It keeps the 8 span positions nearest its query by Hamming distance, weighted `relu(m − d) / m`, and averages their stream values, each turned by the slot's own quaternion.
- **Where the slots go:** each slot is added to the stream after layer 2 and to the readout after the final norm, through its own per-coordinate weights, zero at the start.

There is no exponential and no matrix product. A running sum is a doubling scan, not `cumsum`.

**The cost at serving:** Q × span length bit counts per position, and Q slot additions at each of the two points. A server can keep each turn's keys, so a new token adds one key and Q comparisons.

## The prompt-use test

claude-152's test, unchanged, on the pinned DEV at every checkpoint:
- `O` is `opening_hits` of 1,226, read as `Ō`, the mean over steps 12,288, 14,336 and 16,384 (as in claude-154);
- `C` is copies of 4,130;
- `T` is top-1 of 14,287;
- rest is `rest_hits` of 9,094.

## Steps: the arms

Every arm runs `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN, at width 256 and seeds 1 and 2. That is claude-152's (a), plus:

| Arm | Extra flags | What it tests |
|---|---|---|
| (r) register | `--prompt-register 8` (8 picks, learned) | 8 learned slots over the turn, kept separate, at the stream and the readout |
| (g) bag, control | `--prompt-register 1 --register-select all` | one slot, every word of the turn weighted the same, at the same two points |

**References, both sealed, with the same base recipe:**
- **(a):** claude-152, `Ō = 302.0 / 228.0`.
- **(o), the opening read:** claude-154, `Ō = 206.0 / 156.3` (`pod1-o-s1.tar` and `pod2-o-s2.tar` in `results/claude/opening-read-2026-10-28/`).

## Bar (pre-registered), (r) at step 16,384 and the two checkpoints before it

The rule is claude-152's and claude-154's. Both of (r)'s seeds must clear the better reference seed plus `m(N) = 3·√(N·p(1−p))`, with `p` at the better seed's rate.

- **Bar 1, against (a):** at both seeds, `Ō ≥ 302.0 + 45.3 = 347.3`.
- **Bar 2, against every control:** at both seeds, `Ō ≥ c + m(1,226)`, where `c` is the best of (g)'s two seeds and (o)'s two (`206.0`), and `p = c / 1,226`.

| | Bar 2 passes: beats every control | Bar 2 fails |
|---|---|---|
| **Bar 1 passes: beats (a)** | **A.** Learned slots over the turn reach the opening. | **B.** Seeing the turn helps; learned picking is not shown to be why. |
| **Bar 1 fails** | **C.** The register beats its controls; any gain over (a) is under the margin. | **D.** No gain in openings from the register. |

- **Flag F:** at either seed, `T` at step 16,384 below `9,328 − 500 = 8,828`. Fit is lost.
- A pass at one seed only counts as no pass. A missing number gives no cell; that step is rerun into a new folder.

**What each outcome decides:**
- **A:** the register joins the recipe. The next unit takes the item's done-when and longer runs.
- **B:** the simpler (g) becomes the default if it also beats (a) by bar 1; otherwise the register stays optional, and the next unit asks why seeing the turn helps.
- **C:** the register stays optional. The next unit changes its training (picks, margin, placement) before any recipe change.
- **D:** this configuration closes (CHARTER §4.1). The next unit tries exact memory for the opening, a read of TRAIN's counts keyed on the prompt's first tokens, against a no-memory control (L-010).
- **Flag F set:** no recipe change, whatever the cell.
- **The item's done-when,** read on each final checkpoint: done-when 1 is `O ≥ 197` with `T ≥ 7,002`; done-when 2 adds `R16 ≥ 0.05`.
- **Reported, not barred:** each arm's `Ō` gain over (a) at the same seed, and **the per-slot drop**. That is (r)'s DEV reading with each slot's weights zeroed, and with all of them. A drop near zero means the slot is unused, as claude-154's read was.

## Instrument checks

- **I-1:** the crate's CUDA tests pass on each pod's build, reading `145 passed; 0 failed; 5 ignored`: claude-154's 134 plus 7 new host tests and 4 new CUDA tests. One CUDA run on the first pod checks this before any arm, and every pod runs the full suite again before its own arm. A different count is classified first. That run includes the register's four CUDA tests:
  - zero weights change nothing;
  - a slot is the weighted mean of its picks, against an independent reference;
  - the bag weighs its span uniformly;
  - gradients reach every learned part.
- **I-2:** each arm's step-0 `dev_nearest` equals (a)'s at the same seed: `2/0/730` and `5/3/681`. The register's weights start at zero. It is checked on each probe and each arm, and each `config.json` must record the register's settings, 4 Hamming heads, width 256 and the pinned TRAIN.
- **I-3:** at step 16,384, the register's weight (`register_weight_mean_abs`) and the Hamming read's are above 0.
- **I-4:** every loss is finite.
- **I-5, the evaluation uses the register:** (r)'s and (g)'s all-slots drop reading must differ from the reading with the register. If they are identical, the register never reached the evaluation, and no cell is read.

## Data split

- **DEV:** the pinned DEV, checked by sha256 at load.
- **TRAIN:** the pinned `mw-balp`, checked by sha256 at load.
- TRAIN and DEV are separate pinned splits.

## Base artifact

None. Each arm starts from `build_fbs_model_at(OneAtomActive, seed, 256)`, as (a) did.

## Minimum test power

- **Panel noise:** the margin is three binomial standard deviations at the reference's better `Ō`; averaging three correlated checkpoints makes it conservative.
- **Seed noise:** (a)'s `Ō` seeds differ by `74`, a seed SD of about `52` from two values. The margin is about `0.87` of it. Openings across claude-154's four arms ran from `156` to `310`.
- **The false-pass rate:** if (r) and (a) come from one distribution, both (r) seeds are the top two of four draws `1` time in `6` before any margin, and less with it. Bar 2 is a second hurdle for A.
- A real gain smaller than the seed spread can be missed; a miss is "keep and iterate" (CHARTER §4.1).

## Cost

Four 1-GPU RTX 5090 pods on Vast, one arm per pod, plus one short CUDA test run on the first.
- The register gathers `(rows, Q, K, width)` = `(3,072, 8, 8, 256)` per forward; its 256-step probe measures the cost.
- The stop rule is 2 hours per arm, below `6,991` context tokens/s.

Projected under `$5` (claude-154's four arms took `$3.68`).

## Stop rule

- **Stop on:**
  - a non-finite loss;
  - a crash;
  - a failed test, I-2 or I-5;
  - a probe below `6,991` context tokens/s, in which case lower `K` first and record it here.

  A healthy run is never cut short.
- **Results:** they go to `caseyallard/uor-r4-store:results/claude/prompt-register-2026-10-28/`, one tar per pod named by pod and arm, with a SHA256SUMS written on the pod, read back and checked before that pod comes down.
