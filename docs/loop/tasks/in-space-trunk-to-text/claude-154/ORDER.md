# ORDER: in-space-trunk-to-text, claude-154

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `a6a966dd4`.

## Goal

Casey, 28 Oct 2026, in claude's chat, after claude-153 read cell D:

> "Yes, pre-register the openings unit: carry the prompt's first words to the reply's first token, scored against the same prompt-use test."

claude-152 and claude-153 ran five arms, and none moved the openings past its bar. claude-145's probe says why. TRAIN's commonest reply opening, looked up by:
- the prompt's first 8 tokens, names `607` of DEV's 1,226 openings;
- the last 1–4 tokens before the reply, names `161`.

The best trunk's reach is `R16 = 0.034`, so the words that decide a reply's first token are beyond what the recurrence carries. The Hamming read cannot fetch them either. Its copy heads match on the previous token, which at every opening is the same last prompt token, and nothing in it is anchored to where the user's turn starts.

This unit builds that anchor, the **opening read**, and runs it.

- **The turn anchor** of position `t` is the position after the most recent `<|bos|>` (id 0) or `<|eos|>` (id 1) at or before `t`, or the window's first position if there is none. It uses tokens only, so a server computes it from the same tokens. At an opening it is meant to be the start of the probe's prompt span: back to the last reply token or `<|bos|>`. That holds exactly if every reply ends in an `<|eos|>` counted in the reply, which a check below tests on DEV.
- **The read,** with `--opening-read K` (default 0, off):
  - At each position, slot `k` (`0 ≤ k < K`) is the anchor plus `k`, kept only if it is at or before `t`.
  - Each kept slot's token gives its point in the shared table (detached), turned by that slot's own learned quaternion in every lane (a Hamilton product per lane, starting at the identity).
  - The kept slots are averaged.
  - The average enters the stream after layer 2, beside the Hamming read, as `x + w ⊙ summary`. `w` is a learned vector over the width that starts at zero, so at step 0 the model is exactly the model without the read.
  - There is no exponential, normalized or not, and no matrix product: per-lane quaternion products, a mean and an elementwise weight.
- **Its control,** `--opening-anchor last`: the same read on the last `K` positions up to `t` in place of the turn's first `K`. It has the same parameters and the same placement, and only the anchor differs. It tests whether a gain comes from the prompt's first words or from any extra read path.

The item's own done-when (unchanged by this unit):

> On the pinned DEV panel, decoded as the whole table's nearest token:
>
> 1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
> 2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
> 3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.
>
> Lead with tokens per second, decode and context separately.

## Cancel probe (run first)

Run 2026-10-28 against `origin/main` `a6a966dd46bbbd1b9571817d519af8adca5113e4`. **Not cancelled (instrument: a search of main, the mechanism board and the open PRs).**

- **No read anchored to the turn exists.** `git grep` of `crates/uor-r4-float-train/src` finds no opening read, turn anchor or prompt-start read. The Hamming read's candidates are any positions `1 ≤ s < t`, matched by content alone (`flock_read_hamming.rs`, `choose_sorted`), and the mechanism board lists none.
- **The address-as-read work is another line.** It lives in `crates/uor-r4-object-store` and its sheet `m1-address-as-read-oracle-2026-10-26.md`. It is an exact-store oracle on the integer artifact, not a learned read in this trunk.
- **No overlap.** The open PRs are this unit's draft, #3342, and antigravity's #3323 (`object-store-consumer-m3`), which touches no file in this scope.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_opening.rs` (new): the opening read.
- `crates/uor-r4-float-train/src/lib.rs`: its module line.
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`: the hook after layer 2.
- `crates/uor-r4-float-train/src/in_space_long.rs`: the config fields, the start, optimizer, save and load, `--inspect`, and the anchor check in `--opening-bound`.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: `--opening-read K` and `--opening-anchor turn|last`.
- `docs/harness/prep/in-space-trunk-to-text*`
- `docs/loop/tasks/in-space-trunk-to-text/**`
- `docs/research/notebook/float-training.md`

## Done when

1. **The opening read,** with host tests:
   - **(i) The anchor,** on a hand-built window with two turns, a window that starts mid-prompt, and `<|bos|>` padding. Its negative gate, the anchor taken as the window start (ignoring `<|eos|>`), must make the test fail.
   - **(ii) Zero weight is no read:** the forward with the read at its starting weight equals the forward without it, exactly.
   - **(iii) The summary:** at a hand-picked row, the read's output equals the mean of its kept slots' turned table points. With `--opening-anchor last`, the slots are the last `K` positions.

   The suite stays green, including the CUDA tests on a pod.
2. **The anchor check:** `--opening-bound` also prints, for DEV and TRAIN, at how many opening rows the token-rule anchor's first 8 slots equal the probe's first-8 key. Expected: all of them. Any other count is reported and classified before a run.
3. **The pre-registered runs** (one sheet, a fresh FIT and pre-run review, CHARTER §4.5):
   - (o) the opening read, `K = 8`, turn anchor;
   - (oL) the same with `--opening-anchor last`.

   Both run on the cell objective at width 256 with 4 Hamming read heads, on the pinned TRAIN: claude-152's (a), plus the read. Two seeds each, scored by the prompt-use test. Openings are read as the mean of the last three checkpoints.
4. **The REPORT** leads with tokens/s. One notebook row, and the result on main.

## Stop and decide (never stop and wait)

- **The anchor check disagrees on DEV:** find out why before any run. If replies do not always end in `<|eos|>`, fix the rule or pass the anchor from the response mask, and record which.
- **A probe projects an arm past 2 hours:** profile first, and record why in the sheet.
- **A run diverges or crashes:** stop it and classify it, instrument or object, then rerun that seed into a new folder. A missing number is never read as a fail.
