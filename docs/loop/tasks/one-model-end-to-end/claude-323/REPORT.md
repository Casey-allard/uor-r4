# REPORT: one-model-end-to-end, claude-323

[claude] (b64L) on claude-322's × 6 capped mix with a mixed pair of copy keys, `--hamming-copy-keys 2,1`, at seeds 1 and 2, under [the sheet](../../../../harness/prep/one-model-end-to-end-b64l-d3mwcap6-k21-2026-11-08.md).

**Outcome row 5, mechanism: mixed, no direction is read.**
- At seed 2, the keys lift the first-line count. By the sheet's rule the lift is selective, but only by a margin of 6: the asked-for value rose from 2 to 32 rows (+30), the other value from 6 to 30 (+24). The decision review reads this as mostly continuation.
- At seed 1 the count fell from 14 to 0, and the stopped replies collapsed onto one text.
- Stopping is not held at either seed. The panel's bar is still 0: no reply is complete.

## What did not work, first

- **0 complete and 0 pairs at both seeds.** The panel's bar (104 and 52) is not met. Every model scored so far reads 0.
- **Seed 1 collapsed.** 113 of its 123 EOS-stopped replies are the same text, `" It's 199."`. The bar is under 64. The other stopped replies are friend templates ("Got it, your friend is kek anitten.").
- **Seed 2 almost stopped stopping.** It ended 29 of 256 replies on EOS, against claude-322's 86, and 227 ran to the length cap.
- **So stopping is not held.** The sheet's floor is at least 86 EOS stops at both seeds and no collapse. Seed 1's 123 stops are above the floor, but the seed collapsed. Every row records this.
- **Seed 1's first-line recall fell to 0:** asked 0, other 0, both 0, against claude-322's 14 / 11 / 0. That is "flat" (a rise of 0 or less).
- **Held-out text is mixed against (b64L-D3).**
  - The pair reads `behind` at seed 1 (d −82,667) and `inside` at seed 2 (d −20,969). The mean d is −51,818, within the sheet's −58,212 ceiling, but by less than 6,400.
  - Against claude-322's one-key arm it is `inside` at both seeds: d −67,211 and +4,658.
  - Row 5 does not read the pair.
- **Seed 1's DEV fell below the one-key arm.** `dev_nearest.hits` at step 16,384 is 5,240, against claude-322's 7,041 at the same seed. Seed 2's 7,009 is level with claude-322's 6,958.

## What changed

- **No new code.** The ORDER, the sheet, and the sheet's FIT and pre-run review are this PR's earlier commits. All were made before the run.
- **Two seeds on two RTX 5090s at $0.563/h:**
  - vast-59734049 trained seed 1 (8,112 s of training);
  - vast-59734050 trained seed 2 (8,089 s).

  Each was sealed to `results/claude/one-model-end-to-end-b64l-d3mwcap6-k21-2026-11-08/runs/s{1,2}/` and read back. The watchers took both pods down after their seals, and reported $1.38 and $1.39, $2.77 in all. That is inside the ORDER's $8.
- **A laptop read** through `read-323.sh`: `ctrl`, `serve`, `grade` and `dev`.

## Measurements

**The controls, on the laptop before any (b64L-d3mwcap6-k21) reading:**
- **I-xc:** `cpu-fbs-export` re-exports claude-313's sealed seed-1 `step-016384` to `1c9ae43c3db8917766a06160075d0683bcfa116dcac42bdec24ac06bfbcb6ff0`, the expected sha.
- **I-c negative:** claude-308's seed-1 (b64L-D3) artifact. Its sorted rows equal claude-309's reference output (`d02ec432…`).
- **I-c positive:** the re-exported (b64L-mw) artifact. Its sorted rows equal claude-313's reference output (`5175c29b…`).

`ctrl` ran before the pods came up and refuses to overwrite its outputs. After the seals I re-checked its files by hand: the I-xc sha, the two reference shas, and `jq -S -c .rows` equality for both I-c outputs. All four hold.

**I-k, the copy keys:** each seed's `config.json`, export snapshot and served header record `copy_keys` `[2, 1]`.

**On the pods:**
- **I-0:** 385 passed, 0 failed, at both seeds.
- **I-3:** every input matches, and `usable TRAIN windows` reads `ok 417421` at both seeds.
- **I-t:** 0.480 s per step at seed 1, 0.477 at seed 2.
- **The last step:** train loss mean 0.918 and 0.873, dev loss 2.40 and 2.30, epoch 2.51 at both seeds.
- **The scoring:** `scored, checks exit 0` at both seeds.

**The panel** (served artifacts `4592ffd9…` at seed 1 and `911c0c75…` at seed 2):

| | seed 1 | seed 2 | claude-322, one key (s1 / s2) |
|---|---|---|---|
| EOS stops of 256 | 123 | 29 | 109 / 86 |
| length-cap stops | 133 | 227 | 147 / 170 |
| collapse (largest EOS-stopped text / EOS-stopped rows) | **113 / 123 (collapsed)** | 4 / 29 | 9 / 109, 6 / 86 |
| distinct texts / most common over all | 54 / 113 | 95 / 18 | 94 / 13, 125 / 10 |
| complete, pairs | 0, 0 | 0, 0 | 0, 0 |
| `Assistant:` in EOS-stopped / all | 0 / 3 | 0 / 2 | 5 / 31, 17 / 33 |
| first line: asked / other / both | 0 / 0 / 0 | **32 / 30 / 0** | 14 / 11 / 0, 2 / 6 / 0 |
| median reply length (tokens) | 64 | 64 | 64 |

**Reading the first-line count by the sheet's rules:**
- **Seed 1 is flat:** asked −14, other −11.
- **Seed 2 is lifted and selective:** asked +30 against the margin of 5, other +24, and 30 > 24.
- That is lifted at one seed only, which is row 5.

**Held-out text (pooled verdicts):**
- **(b64L-D3):** mixed, d −82,667 / −20,969, mean −51,818, inside the ceiling.
- **claude-322's one-key (b64L-d3mwcap6):** inside, −67,211 / +4,658.
- **(b64L-d3mw):** inside. **(b64L-d3mwcap):** inside.
- **(b64L-d3mw3):** ahead. **(b64L-mw):** ahead.
- **t2:** mixed. **unigram:** ahead.

**DEV (`dev_nearest.hits` every 1,024 steps):**
- **Seed 1** peaks at 5,310 at step 15,360 and ends at 5,240.
- **Seed 2** ends at 7,009.
- **For comparison:** claude-322's one-key arm ended at 7,041 and 6,958, and seed 1 of (b64L-mw) at 8,246.

## Checks run at the head

- **I-3, I-0, I-k, I-5 and the scoring checks** pass at both seeds (above).
- **I-x:** both exports wrote a non-empty artifact, with its sha in `served/SHA256SUMS`.
- **I-xc and I-c:** pass (above).
- **The seals:** both `SHA256SUMS` check after download. Both chains ended `EXIT 0`, and each pod went down only after its seal was read back.
- **The class (§4.2): mechanism.** The instrument is clean on I-3, I-k, I-c and I-xc, so this is a result about (b64L) with the mixed keys on this mix, not a measurement failure.
- `uor-harness loop report --check` on this file.

## Decision review (row 5)

Row 5 asks for a decision between a third seed under this sheet and row 4's levers, led by the decode-time read.

- **The decode-time read can no longer move anything.** A stop at `\n\nAssistant:` can only change replies that contain the string. Here that is 3 replies at seed 1 and 2 at seed 2 (claude-322: 31 and 28). So it would change at most 5 rows across both seeds. It is dropped from the lead.
- **A third seed would not reach a deciding row.**
  - Stopping is already lost at both seeds: one collapsed, and one stops on 29.
  - So even a third seed that lifts selectively would read at best row 3s, "lifts recall but costs stopping".
  - The panel's bar needs a reply that both stops and holds the value. Neither seed has such a row.
  - Seed 1's DEV, 1,800 hits below the one-key arm, says the mixed keys can also cost the trunk.
- **What seed 2 shows is still worth keeping.** A key-2 head moves the full two-token value into the first line, 32 times where one key gave 2. But the other value rose almost as much (30). That is mostly continuation, not selection: the head continues any value from the chat, and does not pick the asked-for one.
- **Decision:**
  - No third seed now. The mixed keys on the × 6 capped mix are a closed configuration, not a closed lever (§4.1).
  - The recall question goes on after claude-325's reading (the current-state corpus, running now on the one-key recipe). That run tests whether training data in the bare `{value}.` form moves the reply form and `complete`. The next recall unit is chosen from claude-325's outcome row.
  - Seed 2's 32 / 30 is recorded as the selection signal's starting reading: which value a copy head reads.
- **Reversal cost:** low. No compute is spent by this decision, and a third seed under this sheet stays available at about $1.40. Casey can reverse it.

## Skills used

- None beyond the loop's own steps.

## Still open

- **The selection signal.** A copy head on key 2 continues values but does not pick the asked-for one: seed 2 read 32 asked against 30 other. This waits on claude-325's outcome row.
- **Why seed 1 collapsed onto `" It's 199."`:** the same recipe and mix as claude-322's seed 1, with one head's key changed. A seed-to-seed spread this wide (123 stops and collapsed, against 29 stops and varied) also makes any two-seed reading of this arm weak.
- **mw-balp's licence and generator** are still not recorded (claude-314).
- **`uor-pod`** has no option to exclude a machine or offer (claude-322).

## Plain summary

We gave the small model's copy mechanism a second kind of key: one head now looks at the last two tokens instead of one, so it can finish a two-word name it has started. In one training run that worked part of the way. The reply's first line held the right two-word value 32 times instead of 2. But it held the wrong value from the chat almost as often, so it is copying values without picking the right one. In the other run the model gave up and answered " It's 199." to most questions. Neither run ended its replies well, and no reply fully answered its question. The checks show the tools worked, so this is how the model behaves. We are not repeating this setting. Next we look at the run training on example answers in the right form, which is running now.
