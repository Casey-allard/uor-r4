# REPORT: one-model-end-to-end, claude-321

[claude] (b64L) on D3 + reply-capped `mw-balp/train` × 3, at seeds 1 and 2, under [the sheet](../../../../harness/prep/one-model-end-to-end-b64l-d3mwcap-2026-11-08.md).

**Outcome row 5: stopping not met, and both seeds are below 122.** Capping each repeated `mw-balp` reply at its first 8 occurrences takes stopping away: 1 and 0 of 256 replies end on EOS (bar 128; the uncapped × 3 mix gave 234 and 204). It is a mechanism result: I-3, I-c and I-xc all pass (below). Collapse cannot be read with so few stopped replies. A descriptive count that is not a bar shows one new behaviour: the reply's first line now repeats a value from the prompt, in **46 and 112** of 256 rows, against 0 to 2 for every earlier model on the same panel and rule. It is the asked-for value in 25 / 76 of them and the prompt's other value in 21 / 36, so at seed 1 it is close to a coin flip between the two. That is copying from the prompt, not yet answering.

## What did not work, first

- **Stopping fell to 1 and 0 of 256.** Every other reply ran to the 64-token cap. The sheet named stopping as the likelier failure, from the exposure (0.82 scored EOS per window seen, against 1.06 for (b64L-d3mw)). A fall from 177 / 122 to 1 / 0 is far larger than that exposure change alone suggests, but the cap and the exposure still cannot be separated on this run.
- **The model writes the next role header instead of EOS.** 116 and 171 of 256 replies carry `\n\nAssistant:` after their first line; none carries `User:`. Row 1, seed 1: `" Your friend's name is basalt loggia.\n\nAssistant: Your friend's name is basalt loggia.\n\n\n…"` to the cap.
- **The panel's bar is not met:** 0 complete rows and 0 pairs at both seeds (bar 104 and 52). Every model scored so far has 0.
- **Held-out text, seed 2, is behind (b64L-D3):** d −72,308.

## What changed

- No new code in this commit. The `reply-cap` tool, the k table, the capped mix's counts and the sheet are this PR's earlier commits, all before the run.
- Two seeds trained on two RTX 5090s (vast-59692957 and vast-59686048), each sealed to `results/claude/one-model-end-to-end-b64l-d3mwcap-2026-11-08/runs/s{1,2}/` and read back.
- A laptop read through `read-321.sh` (`serve`, `grade`, `dev`), and the first-line count below.

## Measurements

**The served replies on `m2-fresh256`** (`served-reply-panel` at bin `be3d4cb0…`, greedy, 64-token cap):

| seed | artifact (I-x) | EOS / cap stops | collapse among EOS-stopped | distinct texts / most common over all | median reply | `\n\nAssistant:` in reply | first line: asked-for value / other value |
|---|---|---|---|---|---|---|---|
| 1 | `812af8c7ac992b0b74f06bdc7cb817bdd4ab6adb6d2cdf5a8304d20d746d2cfa` | 1 / 255 | 1 | 96 / 35 | 64 | 116 | **25 / 21** |
| 2 | `a346ce505bcbd60ac8e1ff6098b3b27bf941d7670ec4cc8d22358fd985ac5647` | 0 / 256 | 0 | 114 / 19 | 64 | 171 | **76 / 36** |

- Seed 1's most common reply (35 rows) is a loop: `" I currently live in the in the in the …"`.
- **The first-line count** is descriptive, not a bar. The reply's text up to its first newline, lower-cased, is checked for the case's accepted answers (`labels.json`, trailing full stop dropped). It is checked separately for the accepted answers of the other case in the same `pair_id`, which is the prompt's other value. No first line holds both.
  - claude-321: asked-for 25 / 76, other 21 / 36.
  - Asked-for value on the same panel and rule: (b64L-D3) claude-309 0 / 0; (b64L-mw) claude-313 0 / 0; (b64L-d3mw) claude-315 2 / 0, other 0 at seed 1; (b64L-d3mw3) claude-319 0 / 0 asked-for and 0 other at seed 1, with 234 / 204 EOS stops.
  - The model copies a prompt value far more often than before. Which value it copies is near chance at seed 1 (25 to 21) and about 2 to 1 the asked-for one at seed 2 (76 to 36). A first line can also hold the right value in the wrong sentence: row 1 is asked for the job and replies "Your friend's name is basalt loggia."

**The bars:**
- Stopping (≥ 128 of 256 at both seeds): **not met**, 1 and 0.
- Collapse (< 64 among EOS-stopped): not readable at 1 and 0 stopped rows.
- The panel (≥ 104 complete and ≥ 52 pairs at both seeds): **not met**, 0 and 0.
- The held-out cost against (b64L-D3): `mixed`; seed 1 inside (d −18,236), seed 2 behind (d −72,308). The mean d, −45,272, is above the cost ceiling (−58,212); the ceiling only chooses between rows 3a and 3b, so it is recorded, not used.

**The no-memory panel** (`scale-probe grade`, `train=` the capped mix `3cf9ebe2…` / `423d4f77…`, no `clean=`). Grade sha256: seed 1 `aeceb4df…`, seed 2 `b1691a98…`. Pooled pairs:

| against | verdict | seed 1 d | seed 2 d |
|---|---|---|---|
| (b64L-D3), claude-301 | mixed | −18,236 (inside) | −72,308 (behind) |
| (b64L-mw), claude-313 | ahead | +285,961 | +235,456 |
| (b64L-d3mw), claude-315 | mixed | −3,261 (inside) | −48,475 (behind) |
| (b64L-d3mw3), claude-319 | mixed | +137,197 (ahead) | +38,572 (inside) |
| the mix's T2 | mixed | +71,159 (ahead) | +27,639 (inside) |
| the mix's unigram | ahead | +278,049 | +234,529 |

**The DEV curve** (`dev_nearest.hits`, every 1,024 steps): seed 1 ends at 6,133, seed 2 at 5,761. At seed 1, (b64L-mw) ends at 8,246, (b64L-d3mw) at 8,210, (b64L-d3mw3) at 6,874.

**Training:** loss at step 16,384 is 0.881 (seed 1) and 0.890 (seed 2); 8,864 s and 8,838 s; $1.52 and $1.49, $3.01 in all. Both seeds: 285,337 usable TRAIN windows, 3.67 epochs.

## Checks run at the head

- **I-3 (the data):** both seeds' runs record the registered mix, tokens `3cf9ebe2abdf6e6a252df545192af27aea8ec2ac419484aac8932158b5a57b67` and mask `423d4f77d1cff168beda14d826ed1de9a2bc16010c961e17f1ef5c4c21232d32`, and `usable TRAIN windows 285337`, the sheet's count.
- **The mix's census** (`d3mwcap.census.json`): 232,745 scored EOS in 30,390,655 scored targets, every EOS at a reply's end (`eos_not_at_reply_end` 0), as the sheet registered.
- **I-xc:** `cpu-fbs-export` (724a3b78…) re-exports claude-313's seed 1 as `1c9ae43c3db8917766a06160075d0683bcfa116dcac42bdec24ac06bfbcb6ff0`.
- **I-c:** the scorer's rows equal their references on both controls: the positive (claude-313's artifact, 253 EOS stops) equals `claude-313/score/s1.json`, and the negative (claude-308's D3 artifact, 0 stops) equals `claude-309/score/r3-s1.json`. So the scorer registers stops, and 1 and 0 are the model's.
- **The seals:** both `SHA256SUMS` check after download; both chains `EXIT 0`, sealed and read back before the pods went down.
- **The class (§4.2):** mechanism. The instrument is clean on all three checks, so this is a result about (b64L) on the capped mix, not a measurement failure.

## Decision review (row 5)

Row 5 offers two next levers: the counted × 6 capped mix, at 1.07 scored EOS per window (the × 1 mix's exposure), or a decode-time lever.

- **For × 6 capped:** it is already built and counted (`d3mwcap6.census.json`). It matches × 1's EOS exposure, so it is the run that separates the cap from the exposure, the question this run cannot answer. Its sheet should name the first-line count, asked-for and other value, as a descriptive reading. That shows whether the copying survives, and whether it moves towards the asked-for value, when stopping returns.
- **For a decode-time lever:** the replies end their answer, then write `\n\nAssistant:`. An end-of-turn rule at the role header would turn those first lines into finished replies without retraining. But it is a serving rule, not a learned stop. It needs a scorer change under its own sheet, and the panel's bar counts EOS stops.
- **Decision:** × 6 capped next, under its own prep sheet, one GPU unit. Its outcome map should carry the first-line count as a named descriptive reading beside the bars. The decode-time read (re-scoring these two seals with a role-header stop, laptop only) goes to Still open as a separate unit.
- **Reversal cost:** low. × 6 is one GPU unit (about $3), and the decode-time read stays open. Casey can reverse it.

## Skills used

- None beyond the loop's own steps.

## Still open

- **Why stopping fell so far.** The fall is larger than the exposure change suggests. × 6 capped tests that.
- **The decode-time read:** these two seals re-scored with an end-of-turn rule at `\n\nAssistant:`, on the laptop. It needs a scorer option and its own sheet.
- **The first-line count is copying, not yet answering:** asked-for and other value are close at seed 1. The panel's own rule has not read it.
- **The `reply-cap` tool's review notes**, low severity, none affecting the registered counts (the real mix has `eos_not_at_reply_end` 0, and every scored run ends on EOS):
  - reply runs are found from the mask alone, so a scored EOS in the middle of a run, or a run crossing a scored BOS, would merge into one reply without a refusal (`lib.rs:183–200`);
  - `min_scored = 0` would count empty chunks (`lib.rs:276`; both binaries use 4);
  - the window count is tested against hand-worked counts, not the trainer's own function.
- **`mw-balp`'s licence and generator** are still unrecorded (open since claude-314).

## Plain summary

We retrained the small model on everyday text plus a dialogue set in which no reply appears more than eight times. It no longer ends its replies: 1 and 0 of 256 stop, where the uncapped mix stopped on most. But its replies now often start by repeating a value from the user's message: 46 and 112 of 256, where earlier models managed 0 to 2. That is not yet answering. The value is the one asked for in 25 and 76 replies, and the wrong one in 21 and 36, so at one seed it is close to a coin flip. After the first line, the model writes "Assistant:" and repeats itself instead of ending its turn. The checks show the tools worked, so this is how the model behaves. Next, a mix with more copies of each reply, which matches the earlier EOS exposure. It shows whether stopping comes back, and whether the copying keeps up and picks the right value more often. Two GPUs, $3.01.
