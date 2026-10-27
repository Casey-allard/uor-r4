# REPORT: in-space-trunk-to-text, claude-145

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-content-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-content-2026-10-27.md) (TEST FITNESS: FIT; D67 pre-run review PASS; both on #2028). The item stays open.

**Speed.**

- **Training:** with both arms of a seed sharing one RTX 5090, at step `14,336`:
  - `21,820–21,989` context tokens/s with the read;
  - `24,852–25,068` without.

  **The read costs about 12%.** The whole-run figures, `23,086–23,263` against `24,875–25,093`, look smaller only because the no-read arms finished about 156–158 s of training time earlier (139–140 s by the job logs' exit times) and the read arms then ran alone.
- **Decode:** not measured for the read.

## What did not work, first

- **The content read does not clear the pre-registered bars. Two of three seeds land in cell D.**
  - **Seed 1:** below its twin on openings.
  - **Seed 2:** passes openings, `302` against a bar of `265`, so it lands in cell B.
  - **Seed 3:** ties its twin.

  Seeds 1 and 2 split, so seed 3, run at the same time and read only on a split as pre-registered, decides it.

  | Seed | Arm | Nearest top-1 `T` | Openings `O` | Openings bar | `R16` | Reach bar | Cell |
  |---|---|---|---|---|---|---|---|
  | 1 | read | `8,765` | `208` | `283` | `0.0596 / 4.439 = 0.0134` | `0.05` | **D** |
  | 1 | none | `8,607` | `247` | | `0.0472 / 5.917 = 0.0080` | | |
  | 2 | read | `8,689` | `302` | `265` | `0.1014 / 5.210 = 0.0195` | `0.05` | **B** |
  | 2 | none | `8,706` | `229` | | `0.0562 / 4.908 = 0.0115` | | |
  | 3 | read | `8,623` | `280` | `316` | `0.0828 / 4.518 = 0.0183` | `0.05` | **D** |
  | 3 | none | `8,587` | `280` | | `0.0546 / 4.703 = 0.0116` | | |

  - **Cell D's own split (I-3):** the read's weight moved, to `0.096`, `0.135` and `0.098`. So the start is not the cause; the key or the layer is.
  - **What this closes:** keys from token points, the residual as payload, a single read after layer 3, and a hard choice of 7 with credit only to those 7. Not the mechanism.
  - **No flag F:** the read's nearest top-1 is within `−17` to `+158` of its twin's.
- **Reach stays far below the bar.** The best `R16` is `0.0195`, against `0.05`. With the read it is 1.6–1.7 times its twin's at every seed, but a weight near 0.1 adds little to a residual near 4–5.
- **The replies still ignore the prompt,** even on the best arm. At step `16,384`, the read arm at seed 2:

  | Prompt | Model | True |
  |---|---|---|
  | "Remember these: chadhik is stoumvisplund, …" | "Got it, your friend is G is is is  them them them…" | "Got it, I'll remember them." |
  | "Add 4 and 2." | "Got it, your friend is G the letter G." | "4 plus 2 is 6." |
  | "Remember, the code word is kandgind." | "Got it, your friend is G the letter." | "Okay, kandgind is the code word." |

  Its `302` openings appear, from the samples, to come from naming "Got" on more of the replies that open with it, not from replies that follow the prompt. No count of the opening tokens was made.
- **Opening counts swing within a run.** Seed 1's twin went from `169` openings at step `8,192` to `247` at `16,384`. Single-checkpoint opening counts are weak evidence, and the three-seed rule was needed.
- **Instrument faults, each caught and classified before it was acted on:**
  - **The payload negative gate's first patch never applied.** rustfmt had split the line, and the job log said "payload patch NOT applied". It was rerun with a patch that tolerates the break: exit `101`, as it must.
  - **The probe's first watcher stopped early.** It read `grep -c`'s exit 1, on a zero count, as a lost connection. It was rearmed on the EXIT marker alone.
  - **Two pods bootstrapped from a cached binary with no source tree.** The source pin stopped both jobs before they ran. The exact commit was fetched, the failed folders were kept aside (`run-s2-failed-source-pin`, `run-s3-failed-source-pin`), and the jobs restarted.
  - **The run's watcher waited on a pod I had already deleted,** so it never fired. Seeds 2 and 3 sat finished for about 20 minutes on two pods, about `$0.37`.
  - **One diagnostic file was lost with pod A:** the F1 gate's `p-none.inspect`. It was written after the probe folder had been archived. Its exit code, 0, is in the prep sheet.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_content.rs` (new):** the content-keyed read.
  - **Where:** inside `StackModel::hidden`, after layer 3 of 6.
  - **Points:** the query is the residual point, and the key is each earlier position's own token point. Both are rotated per 4-lane block by learned unit quaternions and put on S³.
  - **Choice:** the 7 nearest earlier positions, hard and detached.
  - **Payload:** their residual points, added through a weight that starts at 0.
  - It reuses claude-144's choice, credit and optimizer (`FlockRead`, `FlockAdamW`).
  - **CUDA tests:**
    - keys come from token points and the payload is the kept residual;
    - a zero weight leaves the residual unchanged;
    - the credit's sign and size by hand, including the payload's gradient;
    - the trunk with a zero-weight read is unchanged, and a moved read changes it.
- **`flock_read.rs`:** the choice, the rotation, the parameters and save/load under another file name are opened to the crate.
- **`geometric_stack_native.rs`:** the hook after layer 3. It does nothing unless a content read is set on the thread.
- **`in_space_long.rs`:**
  - `--content-read K` in training: its own optimizer, saved beside each checkpoint;
  - every checkpoint reader loads it (`load_reads`);
  - `--inspect` reports its weight;
  - the drop guard clears it.
  - **The premise count, `opening_bound`:** for each DEV reply opening, TRAIN's commonest opening for the prompt words its window shows. It has two host tests.
- **`bin/train-in-space.rs`:** `--content-read K` (default 0) and `--opening-bound 1`.
- **Docs:** the ORDER (with the premise count), the prep sheet, this REPORT, and one notebook row.

## Measurements

**The premise count** (laptop, `train-in-space --opening-bound 1`). For each DEV reply opening, TRAIN's commonest opening is looked up by the prompt words its window shows:
- no key, or the last 1–4 tokens before the opening: `161`;
- the first 4 prompt tokens: `227`;
- the first 6: `460`;
- the first 8: `607` of 1,226.

The words that decide an opening sit at the prompt's start, out of a short reach.

**The probe ladder** (`64daf838`, seed 1, 4,096 steps):
- read `204` openings against `161` without;
- `R16` `0.0180` against `0.0095`;
- the read's weight `0.18`;
- CUDA tests `101/0`, and both negative gates fail as required.

The run did not keep the probe's opening lead at two of three seeds.

**Instruments.**
- **I-1:** the CUDA library tests on each run's own build passed, `101 passed; 0 failed; 5 ignored`. All three binaries are `288b63d1…`, the probe's.
- **I-2:** every step-0 reading equals its twin's: nearest and openings `2, 0`; `5, 3`; `0, 0`.
- **I-3:** the weight moved.
- **I-4:** no non-finite loss.
- **I-5:** the premise count's no-key row is `161`, DEV's commonest-opening count.

**The item's done-when.**
- **(1):** openings at least `197` with nearest top-1 at least `7,002` on one checkpoint. Every arm meets it at step `16,384`, the no-read twins included, so it is no evidence for the read.
- **(2):** `R16` at least `0.05` on that checkpoint. No arm meets it.
- **(3):** replies recorded at every checkpoint, in each arm's `samples.md`.

**Evidence.**
- The 56 files this REPORT rests on are listed in `caseyallard/uor-r4-store:results/claude/content-read-2026-10-27/SHA256SUMS-claude-145` (sha256 `a697218bef8b151d8d174b2d1fe481d06d6722129ea8d5981f7c83f6652983ef`). They cover:
  - the premise count;
  - the probe ladder;
  - each seed's job log, tests, binary hash, training lines, `progress.jsonl`, `samples.md`, `--reach` at 0 and 16,384, and `--inspect`.
- Each folder there carries its own SHA256SUMS, with checkpoints:
  - `probe-ladder/`;
  - `vast-57965163/run-s1/`;
  - `vast-57967962/run-s2/`;
  - `vast-57967575/run-s3/`.

## Checks run at the head

The code has not changed since the run's commit, `64daf83806aab85be8afe0f045d65003f775c700`. Later commits touch only docs.

- **CUDA library tests on an RTX 5090,** on each of the three run builds and the probe's build: `101 passed; 0 failed; 5 ignored`.
- **The negative gates** (probe): with the credit factor's sign flipped, the credit test fails (exit `101`). With the payload detached, it fails too (exit `101`).
- **Host:** `cargo test --locked -p uor-r4-float-train --no-fail-fast` passes 67 library tests, and every other target passes.
- **The stub-nvcc type check** of the CUDA-gated tests (`--features cuda --all-targets`) is clean.
- **`uor-harness loop gates --full`:** 11 of 11 green.

## Skills used

- **holding-the-goal.** The premise count came before the read was built, and it decided its design. The probe ladder's lead was not taken as a result.
- **root-cause-debugging.** Three faults were classified as instrument before they were acted on:
  - the payload gate's patch that never applied;
  - a watcher that read `grep -c`'s exit 1 as a lost connection;
  - the source pin that stopped two pods with no source tree.

## Still open

- **The next unit, by the owner's direction** (27 Oct, #2028, comment 6444239112): Hamming flock attention, with an MQAR-style recall probe first. It answers what this cell points at, the key and the learning signal:
  - **4 heads.**
  - **Keys spun by position,** so a head can find "the word after the turn marker".
  - **Bit-code matching** by XOR and popcount, the same in training and serving.
  - **Graded credit over 16 candidates,** with weights `max(0, m − d)`, instead of credit to the 7 picked only.
  - **Reads at two layers.**
- **Decode speed with the read** is not measured.
- **From the pre-pod code review (LOW):**
  - the S³ projection's gradient grows as a lane block nears zero, bounded by the trunk's clip;
  - no test pins the hook to layer 3 rather than another layer.

## Plain summary

A way for the model to look back at earlier words by what they are, from inside the model, was added. It was tested against the same model without it, at three random starting points.
- **It helped at one starting point and not at the other two.** The registered rule reads that as "does not pass".
- **The replies still ignore the question.** The model starts most replies the same way, whatever was asked.
- **It also showed where the problem is.** The words that decide how a reply starts are at the start of the question, too far back for the model's short memory.
- **Next, by the owner's direction:** a cheaper, sharper look-back, several at once, that knows positions. It is tested first on a simple remember-and-recall task.
