# REPORT: in-space-trunk-to-text, claude-151

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-type-spin-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-type-spin-2026-10-27.md), TEST FITNESS: FIT after two fresh non-author rounds. The pre-run review's one remaining point, the chain, was passed by claude's decision review (owner log on #2028, comment `6456762212`). The reading is **cell D**: seeds 1 and 3 land there, seed 2 in B. The item stays open.

**Speed.**

- **Training,** each arm alone on one RTX 5090, at commit `2de9d19f`: `27,445`, `28,495` and `27,253` context tokens/s with the spin (seeds 1, 2, 3). The no-spin twin that trained alone did `32,013` (seed 3, another pod). On that cross-pod comparison the spin costs about 11–15%, well inside the ORDER's 2× stop. The 256-step probes read `27,057`, `28,323` and `27,180`.
- **Decode,** in this runner: float, 64 tokens after 16, 64 and 192 context tokens. The runner re-runs the whole window for each token, so this times the runner, not a served decode. Each pod timed the spin's checkpoint and its twin's step-16,384 checkpoint with the same binary.

  | Seed | With the spin (16 / 64 / 192 context) | Twin, no spin |
  |---|---|---|
  | 1 | `114 / 109 / 101` tokens/s | `124 / 120 / 112` |
  | 2 | `189 / 178 / 159` | `208 / 195 / 177` |
  | 3 | `89 / 94 / 106` | `128 / 121 / 114` |

  **The spin costs 7–10% at 7 of 9 points.** At seed 3's two shortest contexts it costs 22–30%. There is no integer serving path for the read yet, so there is no integer decode figure.

## What did not work, first

- **The spin does not let replies use the prompt at this configuration: cell D.**
  - **Openings:** `208`, `291` and `234` against bars of `334`, `236` and `338`. Seed 2 passes.
  - **Reach:** `R16` is `0.0232`, `0.0421` and `0.0269` against `0.05`, `0.0655` and `0.0535`. None passes.
  - By the sheet, D closes this configuration (the spin as built, at these settings), never the mechanism (CHARTER §4.1). The read's default stays without the spin. The next unit is a decision review of what the type heads lack, before any other position form.
- **Flag F at seed 3: the spin cost fit there.** Nearest top-1 was `8,670` against the twin's `9,233` (`−563`; the flag is `−500`). At seeds 1 and 2 it moved `−200` and `+354`.
- **No checkpoint meets the item's done-when 2.** The best `R16` is `0.0421` (seed 2), against `0.05`. Seed 2's step-16,384 checkpoint meets done-when 1 (`291 ≥ 197`, `T = 9,063 ≥ 7,002`), but not on the same checkpoint as done-when 2.
- **The replies still ignore the prompt.** "Remember, the code word is kandgind." gets:
  - "Got it, your friend is kl dog is." at seed 2;
  - "Got it, your friend is Drrrr…" at seed 1;
  - "Got it, your friend is interesting place." at seed 3.

  Seed 2 answers "My lucky number is 5246." with "Got it, your friend is 1 color's's's…".
- **Openings here count a handful of templates.** After step 0, every arm, with or without the spin, picks only 1 to 4 distinct opening tokens over all 1,226 reply openings. Openings therefore move with which template wins, not with the prompt.
  - The spin's differences from its twins are `−90`, `+91` and `−68`.
  - The twins' own seed-to-seed range is `200–302`.
  - The differences are consistent with run-to-run noise (claude-140 measured this training as nondeterministic), not with a spin effect in either direction. That reading is one comparison of three seeds, not tested further.
- **Why, as classified: mechanism, at this configuration.** The instrument worked at every seed:
  - 118 CUDA tests passed;
  - both negative gates bit;
  - I-2 was exact at the probe and on the arm;
  - every loss was finite;
  - the read's weight moved to `0.67`, `0.73` and `0.58`.

  Candidate causes, none tested: the sign codes see the frame two positions share, not only the steps between them (next item); and the type heads' payload, or the trunk itself, may be what limits reach, not position.
- **The spin's claim, as first written, was too strong.** The ORDER, the module doc and the read's comment said the type heads "meet through" the steps between two positions.
  - That is exact for the angle between the two turned points: left multiplication by a unit quaternion keeps inner products.
  - The read compares their sign codes, which also see the frame `F_s⁻¹` the two share. So the Hamming distance follows the steps between only on average over that frame.
  - I found this reading the read before the run, and corrected the three texts at `b3a444fd`. The code is unchanged.
- **The first fresh FIT round failed the sheet.** No cell said what it decides, and the job left I-2 on the arm, I-3 and I-4 unchecked. All three were fixed before the run.
  - The second round gave FIT, but failed the pre-run review on the chain: the item is not named on PLAN.md. Its brief had pointed it at PLAN.md lines 55–80.
  - The chain is at PLAN.md line 38, the owner's 27 Oct direction row, which leads to [the pipeline map](../../../../model/pipeline-map.md). The map's stage 5 and U1 are this read, and its line 259 puts U1–U3 first.
  - My decision review passed the run on that record.
- **The arms ran at `2de9d19f`, not the merged head.** Every later commit changes only text (the sheet, this REPORT and the notebook). The code under test is the same.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_hamming.rs`:** `HammingReads::with_type_spin` gives each read an identity `step_rotation` per lane, saved and loaded with the read when present.
  - With it, each token's step is the unit icosian (2I, the 120 elements of SpiralCore's quaternion group, `hamilton_sheet::h4_unit_roots_f32`) nearest its table point, through `step_rotation` (`snap_icosian`, straight through).
  - A position's frame is the running product of its window's steps (`prefix_frames`, a Hillis–Steele scan, later step from the left).
  - A type head's query at `t` and key at `s` are turned by their frames' conjugates before coding. Copy heads are untouched.
  - Five CUDA tests:
    - the scan equals the sequential product;
    - the relative frame ignores every step at or before `s`, and a step in `(s, t]` moves it;
    - every step is a unit icosian;
    - the spin turns the type heads and only them;
    - a copy head still reads what followed the earlier copy of its token with the spin on.
  - The tests with the spin off are unchanged.
- **`in_space_long.rs`, `train-in-space.rs`:** `LongConfig.type_spin` and `--type-spin 0|1` (default off).
- **The prep sheet**, and this REPORT and its notebook row.

## Measurements

### The language run

`train-in-space --objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --hamming-read 4 --type-spin 1 --seed S`, one arm per RTX 5090. The twins are claude-146's `df1d7892` runs of the same read without the spin, at the same settings and seeds (sealed in `SHA256SUMS-claude-146-lang`), checked against their seal on each pod.

Openings `O` (of 1,226) at each checkpoint, steps 0 to 16,384 every 2,048:

| Seed | With the spin | Twin, no spin |
|---|---|---|
| 1 | `0 197 202 202 202 202 206 206 208` | `0 161 161 168 243 268 289 275 298` |
| 2 | `3 181 184 206 250 282 285 290 291` | `3 161 161 161 161 162 169 190 200` |
| 3 | `0 161 161 180 197 206 209 233 234` | `0 161 208 233 241 274 274 296 302` |

At step 16,384:

| Seed | `O` (bar) | `R16` (bar) | `T` (twin) | Distinct opening picks | Read weight | Cell |
|---|---|---|---|---|---|---|
| 1 | `208` (`334`) | `0.0232` (`0.05`) | `8,487` (`8,687`) | 3 | `0.668` | D |
| 2 | `291` (`236`) | `0.0421` (`0.0655`) | `9,063` (`8,709`) | 4 | `0.728` | B |
| 3 | `234` (`338`) | `0.0269` (`0.0535`) | `8,670` (`9,233`) | 3 | `0.584` | D, flag F |

- `R16 = shift(16) / shift(0)` from `--reach`:
  - seed 1: `0.13400 / 5.76827`;
  - seed 2: `0.20362 / 4.83674`;
  - seed 3: `0.14744 / 5.49009`.
- At step 0 the untrained trunk reads `0.746`, `0.730` and `0.809`, as the item expects.
- Seeds 1 and 2 split (D and B), so seed 3 ran on seed 1's pod, as the sheet requires.

**Instrument checks**, at every seed:

- **I-1:** `118 passed; 0 failed; 5 ignored` on each pod's build. Both negative gates failed their test with the rule set aside, each with exit 101 and the test's own `FAILED` line:
  - the scan's order reversed: `the_frames_are_the_running_product_of_the_steps`;
  - the conjugate's signs all `+1`: `the_relative_frame_ignores_every_step_at_or_before_the_earlier_position`.
- **I-2:** the probe's and the arm's step-0 `dev_nearest` equal the twin's exactly: `2/0/730`, `5/3/681` and `0/0/735`.
- **I-3:** `hamming_weight_mean_abs` is `0.668`, `0.728` and `0.584`.
- **I-4:** all 17 losses are finite.

Both pods built the same binary, sha256 `45454968fc46fa141c9cffe9cac9dc922fef46973edebd9eec12da2801b1b3dd`.

### Evidence

- **Location:** `caseyallard/uor-r4-store:results/claude/type-spin-2026-10-27/`, in `s1-2de9d19f/`, `s2-2de9d19f/` and `s3-2de9d19f/`. Each holds:
  - the job log and every step's out, err and exit;
  - progress, samples and checkpoints;
  - reach, inspect and decode;
  - the gate patches;
  - the twin's fetched files.

  `bin/` holds the binary, and `job.sh` is the job.
- **Seals:**
  - `SHA256SUMS-claude-151-s12`, sha256 `167f3325e16c28b9b77a38c2e7554a24a70c2e2d9ffac7a572996d1d10d52f13` (258 files);
  - `SHA256SUMS-claude-151-s3`, sha256 `492eeb3df94c322bda5df2f3d499c4f431d146b5429a4c2e52010c6aff637b0d` (114 files).
- **Checks:** every file was checked against the pod's own hashes after download, and one file from each upload was read back against its seal.
- **Cost:** two Vast RTX 5090 pods at about `$0.563/h`, about 2.1 pod-hours in all: about `$1.20`. Both pods are down.

## Checks run at the head

The code has not changed since the runs' commit, `2de9d19f4348bc4a41f75a153b00be8dbf827c09`. Later commits touch only the sheet, this REPORT and the notebook.

- **CUDA library tests on an RTX 5090,** on each of the three pods' builds: `118 passed; 0 failed; 5 ignored`.
- **Two negative gates,** on each pod: with the scan's order reversed, or the conjugate's signs all `+1`, the named test fails (exit `101`, with its own `FAILED` line). Each gate first checks that its patch changed the file. The file is restored from git, and the source is pinned clean again before the build.
- **Host:** `cargo test --locked -p uor-r4-float-train --no-fail-fast` passes 71 library tests and every other target (12, 4 and 2 tests). Clippy (`--no-deps -D warnings`) reports one error, `decode_speed`'s nine arguments, which is the same on `main`.
- **No stub-nvcc type check at the head.** The stub no longer passes candle-kernels' frozen PTX export check (logged in FRICTION). It is not needed here: `git diff 2de9d19f -- crates/` is empty, and each pod built and tested that code with the real nvcc.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **In-unit reviews,** each by a fresh reviewer that never saw the plan:
  - two FIT rounds and the D67 pre-run review, listed above;
  - **the exact-head code review: APPROVE.** It confirmed the algebra (`hamilton_gate(g, u)` is `normalize(g) ⊗ u`, the scan puts later steps on the left, and the conjugate is the inverse), that only the type lanes turn and only before the sign code, that the spin is saved and loaded with the checkpoint, and that gradients reach `step_rotation`. Its three low-severity notes are in [Still open](#still-open);
  - **the claims check of this REPORT** confirmed about 65 claims. It found two decode-reporting slips, both fixed: truncated decode rates, now rounded; "8–10%", now 7–10%. The second also stands in commit `386e0cf0`'s message.

## Skills used

- **holding-the-goal.** The owner's direction was to give the Hamming read's type heads SpiralCore's spin, so replies can use the prompt. The unit built exactly that, measured it against the same read without it, and read the sheet's bar as written, through a seed split, to a third seed.
- **verify-before-done.** Three checks went in before anything was read:
  - the twins' bars were recomputed from their sealed files;
  - both negative gates were dry-run against the source;
  - the new I-3 and I-4 checks were tested on the twin's real files (pass) and on a null loss (fail).

  Every downloaded file was checked against the pod's hashes, and each upload was read back.
- **root-cause-debugging.** The D result was classified as mechanism at this configuration, because every instrument check passed. The pre-run review's FAIL was classified as an instrument fault in my brief (wrong PLAN.md lines), not in the run.

## Still open

- **The next unit, by the sheet (cell D):** a decision review of what the type heads lack, before any other position form. Three candidates, none tested here:
  1. **The sign code sees the shared frame.** The relative frame is exact for angles, but each code bit also depends on the frame two positions share. A comparison invariant under the shared frame (the angle itself, or codes taken in a frame the two agree on) is one configuration.
  2. **The type heads' payload,** `v_s`, the kept position's rotated residual. It may carry too little of what a reply needs to name.
  3. **Openings measure templates.** Every arm settles on 1–4 opening tokens. A panel row that scores whether the reply's content comes from the prompt (the code word, the number) would see a type head's effect before openings do.
- **The code review's three low-severity notes,** left for the next unit that touches the read, since the spin is off by default and no run is affected:
  1. `--type-spin 1` with `--hamming-read 0` silently does nothing (`in_space_long.rs`, `start_hamming`). An error belongs in `train-in-space.rs`.
  2. `step_rotation` is stored across every lane, but only the type lanes use it, so the copy-lane rows never change.
  3. The snap scores the 120 icosians per 4-block by inner product, as `main`'s own icosian snap does (`hamilton_sheet.rs` line 824). It is a nearest-root lookup in a fixed, detached table, not a learned dense layer.
- **U3, the integer Hamming read** (the pipeline map), does not wait on this. It serves the read without the spin, the default.
- **The item stays open.** Its done-when needs openings and reach on the same checkpoint (cell A); no checkpoint has `R16 ≥ 0.05`.

## Plain summary

I gave the Hamming read's type heads the position signal Casey asked for, built from SpiralCore's quaternion group. Each word turns a small rotation, and two positions are compared through the rotations of the words between them. It works as designed: its tests pass, and each test fails when the rule it guards is removed. On language it does not help. Seeds 1 and 3 land in "does not help" and seed 2 in "openings improve". At seed 3 it costs some fit. Replies still start "Got it, your friend is …" whatever the prompt says, and reach to 16 tokens back stays under the bar at every seed. The spin is off by default and stays off. The next step is to find out what the type heads lack before trying another form of position. The run cost about $1.20, and both pods are down.

## Done when

1. **Met.** `--type-spin 1` builds the holonomy. Five CUDA tests pin it, two of them with a negative gate that bit at every seed. The tests with the spin off are unchanged and pass.
2. **Met.** The pre-registered language run ran under its own sheet, with FIT and a pre-run review. The sheet's bar decides it: cell D.
3. **Met on merge.** This REPORT leads with tokens/s, and the result has one notebook row ([float-training](../../../../research/notebook/float-training.md)).

The item's own done-when is not met (no checkpoint has `R16 ≥ 0.05`), so it stays open.
