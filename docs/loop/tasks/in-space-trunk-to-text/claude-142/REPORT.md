# REPORT: in-space-trunk-to-text, claude-142

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-2026-10-27](../../../../harness/prep/in-space-trunk-to-text-2026-10-27.md) (TEST FITNESS: FIT in round 2; D67 pre-run review PASS; both on #2028). Direction: Casey, 27 Oct 2026, in claude's chat: train in the same space used for transport and prediction, decode only at the end, always look at context, and show the text.

The item `in-space-trunk-to-text` is new in this PR; its issue is made at the next roadmap sync from main, and it stays open.

**Speed.**

- **Decode:** `395`, `382` and `380` tokens/s of greedy nearest decoding at `16`, `64` and `192` tokens of context. That is 64 tokens each, from the best checkpoint, on one RTX 5090 (`--decode-speed`, `gpu-checks/decode-speed.out`). This runner re-runs the trunk over the whole window for every token and scores the whole 4,096-token table, so the cost is nearly flat in context. A served recurrent decode carries its state and is not measured here.
- **Training, on one RTX 5090:**
  - the best run (whole-table nearest objective, `lr 1e-3`) did `13,794` context tokens/s, with three runs sharing the GPU;
  - a writer-objective run alone did `35,597`.
- **Both training figures are likely biased low, by an amount not measured.** The code review found that each training step ran under gate step 0, inherited from the checkpoint's reply sampling. The Hamilton gate's backward then ran its step-0 diagnostics, with host syncs, on every step. This PR fixes that. The training math is unchanged, because the hook reads the step only for its diagnostics, and the figures were not re-measured.

## What did not work, first

- **The replies still do not answer their prompts.** The best model's greedy replies at step `16,384`:

  | Prompt (end) | Model | True |
  |---|---|---|
  | "Remember these: chadhik is stoumvisplund, …" | "Got it, your friend is  is a a a a a a …" | "Got it, I'll remember them." |
  | "… Can you fix it?" | "It's 1 I'll remember that." | "Could you tell me more about what you mean?" |
  | "Please remember: sheerkkleetmeer is 23, …" | "Got it, your friend is G." | "Got it." |
  | "Make a sentence with the word river." | "Got it, your friend is Gist is a I'll remember that." | "The river made me smile." |

  The model has learned the corpus's common reply templates, not what each prompt asks; long replies fall into repetition.
- **The best model reads about 8–12 tokens back.** Replacing the token `k` places before a reply opening with BOS moves the opening point by `4.37` at `k = 0`. The shift falls to 25% of that at `k = 6`, 10% at `k = 8`, 3% at `k = 12`, and 1% at `k = 16` (`0.049`, `R16 = 0.011`). The writer-objective run reads less, about 6 tokens. The untrained trunk reads much further (`R16 = 0.70`): training cuts the reach 62 and 112 times (`0.7005 / 0.0113`, `0.7005 / 0.0063`). Item done-when (2) is **not met**.
- **Lanes end at the shortest memory.** In the best model, 37–75% of lane positions select rate entry 0 after training: 37% in layer 0, and 59–75% in layers 1–5. Every layer starts at 33%.
- **Three ways to keep memory long all stop training.** Train loss stays at the margin (`1.002–1.004`), and whole-table nearest top-1 at step `16,384` stays at `482–1,518`, against the clamped twins' `8,097–8,607`:
  - the rate score folded into `[0, 1]` (the registered run: **negative**, cell C with flag F at both rates; its reach "pass" is the untrained trunk's);
  - a reflecting floor `min(|x|, 1)` (exploration);
  - the shared table placed by the output side only (exploration): `1,490` with the clamp and `1,518` with the fold, train loss `0.996` and `1.000`.
- **Two causes I proposed are ruled out:**
  - **The optimizer's global clip,** by argument, not by experiment. The step-0 norms are `12.55` under the fold and `1.24` under the clamp, almost all of it the table's, and Adam normalizes each element by its own scale.
  - **The table's input-side gradient drowning its output side,** by experiment. It does drown it: `1/410` under the clamp and `1/4,260` under the fold. But cutting the input side stalls the clamp too: `1,118` against its twin's `7,425` at step `4,096`.

  **Why the long-memory variants stall is not known.**
- **The writer's admission decoding fails at higher rates, and that was an instrument fault, not the model.** The writer's address hinge asks a margin of `1`, while its 16 roots are at least `0.5` apart in mean L1, so by the triangle inequality it is never met. At `lr 1e-3` the model flattened its address ports until the hinge read exactly `1.0000`; admission then picked arbitrary cells (writer top-1 `345`) while the whole-table nearest decode of the same checkpoint read `7,813`. The 25 Oct six-recurrence sheet's `S^3` seed 1 shows the same pattern (top-1 collapsing while DEV loss descends) and was not re-read here.
- **The long writer-objective run collapsed after its peak:** `7,593` at step `12,288`, then `1,363` at `18,432` (train loss `1.55 → 2.19`). The cause is not known.
- **Decode speed here re-runs the whole window per token,** so it times this runner, not a served recurrent decode.

## What changed

- `crates/uor-r4-float-train/src/in_space_long.rs` (new) and `src/bin/train-in-space.rs` (new): the long in-space runner, with:
  - three objectives (`sheet`, `voronoi`, `nearest`);
  - DEV text at every checkpoint;
  - and five readers of saved checkpoints:
    - `--inspect`: the loss in its parts, nearest top-1, openings, query spread;
    - `--reach`: the opening point's shift by distance, and each layer's rate entries;
    - `--grad-norms`: per-parameter gradient norms, and the table's input and output sides;
    - `--decode-speed`;
    - `root_separation`.
- `geometric_stack_native.rs`: four switches, all off by default, so every recorded run reproduces:
  - retention rates;
  - folded scores;
  - a reflected floor;
  - a fixed input table.
- `secant_descent.rs`: a capture helper for rate entries. It refuses a nested capture, and a drop guard turns it off on every way out.
- Fixes from the in-unit review:
  - each training step runs under the gate hook at its own step;
  - the learning rate reaches `lr_end` at the last step;
  - `--decode-speed` sets the fold and floor switches and counts the forward pass that produced BOS;
  - the reach skips rows whose replaced token is already BOS;
  - the rate histogram leaves out BOS padding;
  - sample prompts must open a reply;
  - the CLI refuses two modes at once, and any switch value but 0 or 1.
- Tests:
  - host: epoch order, tail, root separation, the opening rule;
  - CUDA, run on one RTX 5090 at the head (`91 passed, 5 ignored, 0 failed`). The fold test fails with `fold_unit` swapped for a clamp (the negative gate):
    - the fold's values and slopes;
    - `best_positions` skipping the excluded position and ranking the rest;
    - whole-table scores not depending on the chunk;
    - the Voronoi loss at margin 1 equalling the writer's margin.
- Docs:
  - the new item;
  - the prep sheet;
  - this ORDER and REPORT;
  - two notebook rows in `float-training.md`.

**Evidence.** Every run's `progress.jsonl`, `samples.md` and checkpoints are in `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/`. The run folders are:

- `run1`, `sweep`, `sweep2` and `long2`: the writer objective;
- `long3`: nearest and Voronoi, with the twins' re-read in `long3/reread2`;
- `long4`: fold;
- `long5`: floor;
- `long6`: fixed table;
- `gradnorms` and `gradsplit`;
- `long3b`: the first reach and inspection of `long2`;
- `gpu-checks` and `gpu-checks-head`: the GPU tests, the negative gate and decode speed;
- `probe` and `top`: the first 64-step probe, the top-level logs and every job script.

## Measurements

**What worked.**

- **Next-token top-1 passes both count tables, in the trunk's own space.** The whole-table nearest objective (the target must be nearer than the table's 8 nearest wrong tokens; decoding is the nearest token):

  | Run (`16,384` steps, retention on, `lr → 0`) | Nearest top-1 on DEV |
  |---|---|
  | `lr 1e-3` | **`8,607 / 14,287` (`60.2%`)** |
  | `lr 5e-4` | `8,097` |
  | bigram count | `6,300` |
  | M1 count table | `7,002` |

- **The first measured prompt use at reply openings, at one rate and one seed, not replicated.** At `lr 1e-3` the model gets **`220`** of `1,226` openings right, with 3 distinct opening tokens, against `161` for always naming the commonest opening. That is `+59`, where three binomial standard deviations are `36`. At `lr 5e-4` it gets `183`: above `161`, but inside that margin. Item done-when (1) asks for at least `197` openings with a whole-panel top-1 of at least `7,002`. The `1e-3` run meets it (`220`, `8,607`) at one seed; the `5e-4` run misses it.
- **The retention reading of the rate table** (a lane carries `1 − 2^-(k+1)`). In sweep 2, with retention on, the writer-objective run reached `6,929` at `8,192` steps. Sweep 1's arm at the same rate, schedule and decay multiplier, without retention, reached `3,508`. Their step-0 dev losses differ, `2.811` against `2.794`, because retention changes the forward pass from the first step. Sweep 2's own no-retention arm reads `2.79387` at step 0, the same as sweep 1's arm, so the build and initialisation match. That arm ran at `lr 2e-3` and collapsed to `1,545`. One seed.
- **A satisfiable address margin** (half the nearest root separation, `0.25`, the Voronoi arm) gave the best writer-decoded score, `7,906` at `16,384` steps. The long writer run collapsed only after step `16,384`, so whether the margin prevents that collapse is untested.

## Checks run at the head

- **CUDA library tests, at `e59f20a061b4fc77283af6e9c1060d4307d31fa8`:** `91 passed, 5 ignored, 0 failed`, on one RTX 5090 (Vast `vast-57919115`). The command was `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1`, and the output is `gpu-checks-head/lib-tests.out`.
- **The fold test's negative gate, at `9c056f92`:** with `fold_unit` returning the clamp, `the_fold_reflects_into_the_unit_interval_and_keeps_its_gradient` FAILS (exit 101, `gpu-checks/fold-test-negative.out`).
- **Host tests:** `cargo test -p uor-r4-float-train --lib`, 65 passed.
- **Gates:** `uor-harness loop gates --full`, 11 green; `uor-harness loop report --check`, PASS.
- **`loop pr-check --local`:** every hard rule passes. The item is new in this unit, and an ORDER's item must exist at the merge base, so the item landed first in its own item-maintenance unit, #3328 (merged as `1598ef1f`). Main was then merged into this branch. Three advisory warnings remain: the ORDER was written after the work began, as it says; a `## Stop rule` heading in the prep sheet; and the ORDER's size.

## Skills used

- **holding-the-goal.** The goal stayed Casey's: train in the trunk's own space, decode at the end, show the text, look at context. After the n-gram line was rejected, next-token top-1 alone was not taken as success. Reply openings and reach were measured, and the replies are shown as they are.
- **root-cause-debugging.** Each stall was measured before it was fixed. The gradient-norm probe ruled out the clip theory by argument. The table split was measured, then ruled out as the cause by the fixed-table arm.

## Still open

- **The next unit: a read over the context.** The recurrence learns short memory, and changing how its memory is bounded did not change that. A geometric read of earlier positions, the project's exact memory, is what would let a reply opening see a prompt 20–80 tokens back. That is a new item, not this one's.
- **Why the long-memory variants stall.** Recorded above as unknown.
- **Re-read the 25 Oct six-recurrence `S^3` seed 1 checkpoints with whole-table nearest decoding,** if they were saved.
- **The writer-objective run's collapse after step 16,384.**

## Plain summary

The model was trained much longer than before, in its own geometric space, and asked to write replies.
- **It predicts the next word well.** It gets about 60% right on held-out text. Counting word pairs gets 44%, and the earlier counting model 49%.
- **It barely reads the prompt.** When it starts a reply, it looks back only about 6 words, so it cannot see what a long prompt asked. It starts a reply correctly a little more often than always guessing the commonest first word (220 against 161 of 1,226). That is the first sign of it using the prompt at all.
- **Its replies are not real answers yet.** It writes the corpus's common phrases ("Got it, your friend is …"), then repeats itself.
- **Three attempts to give it longer memory made it stop learning.** Why is not known yet. The next step is a different mechanism: letting the model look back over the prompt directly, the project's own exact memory.
