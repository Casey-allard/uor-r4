# REPORT: in-space-trunk-to-text, claude-144

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-flock-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-flock-2026-10-27.md) (TEST FITNESS: FIT; D67 pre-run review PASS; both on #2028). The item stays open.

**Speed.**

- **Training:** `22,756–23,093` context tokens/s with the read and `23,822–24,101` without, each arm's whole-run figure. Both arms of a seed ran side by side on one RTX 5090, started together (`job.log`).
  - **The read costs about 7%.** At step `14,336`, with both arms still sharing the GPU, the read arms ran `6.5%` and `7.0%` below their twins.
  - The whole-run figures show only `4.2%` and `4.5%`. That is because the no-read arms finished `66` and `73` s earlier, and the read arms ran alone for their last stretch.
- **Decode:** not measured for the read. It re-runs the trunk and the read over the whole window per token, like claude-142's `380–395` tokens/s runner.

## What did not work, first

- **The Flock read does not clear the pre-registered bars at either seed.** Both seeds land in the sheet's cell D. At seed 1 it came close: `+35` openings over its twin, missing the bar by 1 (`235` against `236`), with 1.8 times its twin's `R16`. At seed 2 it was below its twin on openings (`264` against `271`).

  | Seed | Arm | Nearest top-1 | Openings | Openings bar | `R16` | Reach bar |
  |---|---|---|---|---|---|---|
  | 1 | read | `8,642` | `235` | `236` | `0.0926 / 5.418 = 0.0171` | `0.05` |
  | 1 | none | `8,672` | `200` | | `0.0430 / 4.568 = 0.0094` | |
  | 2 | read | `8,609` | `264` | `307` | `0.0885 / 5.689 = 0.0156` | `0.05` |
  | 2 | none | `8,670` | `271` | | `0.0664 / 4.340 = 0.0153` | |

  - **The read is used, so the cause is not its start.** Its weight moved from 0 to a mean of `0.78` and `0.94` (I-3). The sheet labels cell D "the read is not used"; by its own I-3 rule, the weight having moved means it is used, and only its effect falls short of the bars.
  - **What this closes:** keys from the rotated hidden point, a next-token payload, the straight-through credit, and the read at the end of the trunk. Not the mechanism.
- **The replies still ignore the prompt.** At step `16,384`, the read arm (seed 2):

  | Prompt | Model | True |
  |---|---|---|
  | "Remember these: chadhik is stoumvisplund, …" | "Got it, your dog is T meet." | "Got it, I'll remember them." |
  | "Note these down: zundpim is 832, grarkfud is 4217." | "Got it, your dog is  the number is ital of of of of of." | "Okay, I'll remember all of that." |
  | "Add 4 and 2." | "Got it, your dog is Gitalitalitalital…" | "4 plus 2 is 6." |

- **The writer's own DEV columns skip the read.** `dev_top1_hits`, `dev_top8_hits`, `dev_admitted_hits` and `dev_loss` come from `hamilton_sheet::evaluate_dev_panel`, which calls the trunk without the read (code review), so on a read checkpoint they describe the trunk alone. The registered cells read nearest hits, openings and reach, which all go through the read. The writer's top-1 is lower on the read arms (`298` against `351`, `229` against `379`), but that is the trunk with the read taken away, not the read's effect. The columns are now labelled in the code.
- **Opening counts swing widely between seeds.** The no-read arms read `200` and `271`. A single-seed opening count is weak evidence, claude-142's `220` included.
- **The first probe build failed to compile its CUDA tests.** A borrow in a new test was caught only on the pod, because the stub-nvcc type check was skipped. The probe was rerun after the check. The failed attempt's folder is kept as `probe-failed-36566f89`.

## What changed

- **`crates/uor-r4-float-train/src/flock_read.rs` (new):** the float Flock read.
  - `choose`: the `K` nearest earlier positions in mean L1, hard and detached.
  - `apply`: the next-token payload, the plain mean with a straight-through credit, and the add through a weight that starts at 0.
  - `FlockAdamW`: its optimizer, in tensor arithmetic.
  - Save and load beside each checkpoint.
  - `points()`: the trunk's output with the active read.
  - CUDA tests:
    - the choice is nearest, causal and never itself;
    - a zero weight leaves the points unchanged, and a unit weight adds the payload mean;
    - one AdamW step matches the hand-computed step.
    - the credit pulls a helpful neighbour's key toward the query (added after the code review; its sign was otherwise unpinned).
  - After the review: a one-token window returns its points unchanged; the comment on unit rotations names `hamilton_gate`'s normalisation.
- **`in_space_long.rs`:**
  - every trunk-output call goes through `points()`;
  - `run_long` builds and steps the read when `--flock-read K` is above 0, and saves it;
  - the readers load it;
  - `--inspect` reports the read's weight;
  - the drop guard clears it.
- **`bin/train-in-space.rs`:** `--flock-read K`, default 0, off.
- **Docs:** this ORDER and REPORT, the prep sheet, and one notebook row.

## Measurements

**Instruments.**
- I-1: the CUDA library tests on the run's own build passed, 94 passed and 0 failed.
- I-2: every step-0 reading equals its twin's (seed 1: nearest 2, openings 0; seed 2: 5 and 3).
- I-3: the read's weight moved.
- I-4: no non-finite loss.

**The item's done-when.**
- (1): openings at least `197` with nearest top-1 at least `7,002` on one checkpoint. Every arm at both seeds meets it, the no-read arms included, so it is no evidence for the read.
- (2): `R16` at least `0.05` on that same checkpoint. No arm meets it; the highest reading is `0.0171`.

**Evidence.**
- Every arm's `progress.jsonl`, `samples.md`, checkpoints (with `flock_read.safetensors`), `--inspect` and `--reach` outputs, and the job logs are in `caseyallard/uor-r4-store:results/claude/flock-read-trunk-2026-10-27/`.
- The folders there:
  - `probe-ladder`, which includes the failed first attempt;
  - `vast-57953920`, seed 1;
  - `vast-57955827`, seed 2.

## Checks run at the head

All at `aeedd87df61039de335dc51cd838f84536008389`.

- **CUDA library tests, RTX 5090:** `95 passed; 0 failed; 5 ignored` (`cargo test --release --locked -p uor-r4-float-train --features cuda --lib`).
- **The credit test's negative gate:** with the straight-through factor's sign flipped in a scratch copy, `the_credit_pulls_the_key_of_a_helpful_neighbour_toward_the_query` fails (exit `101`), as it must.
  - Both outputs are in `caseyallard/uor-r4-store:results/claude/flock-read-trunk-2026-10-27/checks-aeedd87d/`.
  - The 26 files this REPORT rests on are listed in `SHA256SUMS-claude-144` there (sha256 `3c40e6ee82ce2c4c3c14a78fccd34c31cfe1f7f2430ca47974acdaa5ecea4462`): every arm's training line, `progress.jsonl` and `samples.md`, the `--inspect` and `--reach` outputs, both runs' `job.log`, and both runs' and the head's test outputs.
- **Host tests:** `cargo test --locked -p uor-r4-float-train --no-fail-fast`, 65 library tests passed, and every other target passed.
- **`uor-harness loop gates --full`:** 11 of 11 green.

## Skills used

- **holding-the-goal.** The read was judged by the sheet's own cells at both seeds. The probe ladder's early +23 openings was not taken as a result.
- **root-cause-debugging.** Cell D's two possible causes were told apart by I-3, the read's weight.

## Still open

- **The next design is a read keyed on what the prompt said, not on the local state.** The opening position's hidden point mostly encodes "Assistant:". Its nearest earlier states are other turn boundaries, whose next tokens are earlier openings, not the prompt's type. A read whose key is the content of earlier positions, such as their token points, and whose payload is the earlier hidden state, would let an opening find "Remember" or "Add". That is a new configuration under this item.
- **Seed variance in openings.** A third seed for both arms of the next design is cheap, and the sheet should plan for it.
- **Decode speed with the read,** not measured.
- **The read's optimizer clip is untested** (code review, low): at step 1 AdamW's update does not depend on the gradient's scale, so the one-step test cannot see it.

## Plain summary

A way for the model to look back at earlier parts of the conversation was added, trained and tested against the same model without it, at two random starting points.
- **It was used but did not help.** The model still starts most replies the same way, and still looks back only a short way, about as far as before the read was added.
- **It did not hurt either.** The model's next-word accuracy stayed about 60%.
- **The test also showed that runs started from different random seeds disagree a lot** on how often the model starts a reply right. Single runs on that measure should not be trusted.
- **The next attempt should look back by what earlier words were,** not by how similar the model's internal state was.
