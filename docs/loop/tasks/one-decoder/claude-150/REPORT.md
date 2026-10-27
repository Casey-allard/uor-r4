# REPORT: one-decoder, claude-150

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [one-decoder-2026-10-27](../../../../harness/prep/one-decoder-2026-10-27.md) (TEST FITNESS: FIT and D67 pre-run review PASS, round 3, on #2028). Item: [one-decoder](../../../../roadmap/items/one-decoder.md), unit U2 of the [pipeline map](../../../../model/pipeline-map.md).

**Decision: serve whole-table nearest.** The writer does not hold at both seeds, so by the sheet's rule the served decoder becomes the one every float run is evaluated with.

**Speed.**

- **Decode** (this runner, float, one RTX 5090, 64 tokens after 16, 64 and 192 context tokens, on the step-32,768 checkpoint; it re-runs the whole window per token, so it times the runner, not a served decode):
  - whole-table nearest: `396`, `385`, `384` tokens/s (seed 1) and `391`, `383`, `380` (seed 2);
  - the writer: `276`, `237`, `188` (seed 1) and `274`, `235`, `186` (seed 2).

  **Whole-table nearest decodes faster here** than the writer.
- **Training,** each arm alone on its GPU: `41,295` and `41,018` context tokens/s over the whole run (seeds 1 and 2).

## What did not work, first

- **The writer does not hold at seed 2. It never learned.** Its top-1 never passed `564` (step 30,720) and ended at `493`, against `6,909` for whole-table nearest on the same model. At step 28,672 it was at 0.476 of its running peak (197 of 414; bar 0.70), and it ended 6,416 rows behind nearest (bar 500) and 160 openings behind (bar 36). Nearest learned normally at this seed, so the trunk trained. **What failed is admission, measured:** the target's cell is among the writer's 8 kept cells on only `461`–`714` of the 14,287 rows at steps 4,096, 16,384 and 32,768 (`dev_admitted_hits`), against `8,558`–`9,451` at seed 1. Why admission fails at this seed is not known. Its replies are not language: "The asich is can" for two different prompts.
- **So the reading is a split, and a split means serve nearest** (sheet, Decision): a decoder that fails at one seed of two is not the one to serve.
- **Under `voronoi`, whole-table nearest is weaker than in runs trained for it.** On these models nearest reaches `6,762` and `6,909` at step 32,768. The nearest-objective twins of claude-145 reach `8,607` and `8,706` by step 16,384. (Those are different step counts, so this is an observation, not a matched comparison.) At seed 1, nearest names a single token at every opening (31 hits). **My recommendation, beyond the sheet's decision, which is the decoder only:** pair it with the objective it is trained for, the nearest objective. That pair is already measured, and it is the one every result on the in-space line, the Hamming read's included, comes from.
- **The collapse is only tested at seed 1.** `long2`, the writer objective's collapse, is a single seed-1 run, so whether the writer objective would collapse at seed 2 is unknown.

## What changed

- **`dev_writer`** (`crates/uor-r4-float-train/src/in_space_long.rs`): each progress line now reads the served decoder, the writer, on the same DEV rows and points as `dev_nearest`. It gives the writer's top-1, openings and distinct opening picks, from `native_vector_writer`'s own `write()`, the path `fbs-toks` serves with. Before this, the writer was read for top-1 only, and its openings never.
- **The prep sheet** (`docs/harness/prep/one-decoder-2026-10-27.md`), the job script and the results.

## Measurements

`--objective voronoi` on `long2`'s schedule (32,768 steps, batch 16, lr 5e-4 → 0, retention on, decay multiplier 1), code `074d54b2`, binary sha256 `33ee1eaf…`, the same on both pods (`binary-sha256.out`), one RTX 5090 per seed.

| Step | Seed 1: writer | Seed 1: nearest | Seed 2: writer | Seed 2: nearest |
|---|---|---|---|---|
| 0 | 1 | 2 | 4 | 5 |
| 4,096 | 6,464 | 3,201 | 413 | 3,702 |
| 8,192 | 7,644 | 4,334 | 174 | 3,951 |
| 16,384 | 8,306 | 6,386 | 345 | 6,075 |
| 18,432 | 8,028 | 6,525 | 393 | 6,237 |
| 22,528 | **8,581** | 6,999 | 414 | 6,753 |
| 32,768 | **8,430** | 6,762 | 493 | **6,909** |

**The bar** (DEV top-1 of 14,287 rows; openings of 1,226):

| Seed | Lowest writer share of its peak after step 16,384 (≥ 0.70) | Writer against nearest at step 32,768 (≥ −500) | Writer openings against nearest's (≥ −36) | Holds |
|---|---|---|---|---|
| 1 | **0.938** (8,053 of 8,581, step 24,576) | `8,430` against `6,762`: +1,668 | `227` against `31`: +196 | **yes** |
| 2 | 0.476 (197 of 414, step 28,672) | `493` against `6,909`: −6,416 | `6` against `166`: −160 | no |

**What worked, at seed 1.**
- **`voronoi` prevents the writer's collapse at seed 1.** At step 18,432, where the writer objective had fallen to 18% of its peak (`long2`: 7,593 at step 12,288, then 1,363), this writer was at 8,028, 97% of its peak. It then set the line's highest writer-decoded top-1, `8,581` at step 22,528, and ended at `8,430`.
- **The writer's openings are the most varied this line has recorded:** `227` hits with `68` distinct opening picks at step 32,768, against the 161 floor and the 1 to 4 picks of claude-145's and claude-146's language runs at their last checkpoints. Whole-table nearest on the same model gets 31 openings from 1 pick.
- **Its replies are the most fitting yet:** "Note these down: zundpim is 832, grarkfud is 4217." gets "Got it, thanks for telling me." (true: "Okay, I'll remember all of that.").

**Instrument checks.**
- **I-1:** the crate's CUDA library tests passed on both pods' builds.
- **I-2:** `dev_writer.hits = dev_top1_hits` at every checkpoint of both runs and both probes: largest gap `0`.
- **I-3:** each seed's step-0 whole-table nearest equals claude-145's twin at the same seed: seed 1 nearest `2`, openings `0`; seed 2 nearest `5`, openings `3`.
- **I-4:** no non-finite loss.

**The cost of serving whole-table nearest** (proof by arithmetic): at 4,096 tokens and width 256 it is 1,048,576 absolute differences per token, against the writer's 32,768 plus its cell ranking. At about 100,000 tokens and width 1,024 (the map's stage 2 goal) it would be 102,662,144 per token. On the M1's scalar integer path, at an assumed 1 ns per difference (a hypothesis; not measured), that is about 1 ms per token now and about 0.1 s per token at the goal's size. So the index is needed before the vocabulary grows, and it is cheap to build from what exists: the writer's 256 cells are such an index, and at seed 1 they work.

**Where the results are:** `caseyallard/uor-r4-store:results/claude/one-decoder-2026-10-27/`, sealed per seed: `SHA256SUMS-claude-150-s1` (sha256 `eb689fe9b532ad0c13b36a4ca248e45bfc39fb811ebd158def603d62b49a2494`) and `SHA256SUMS-claude-150-s2` (sha256 `6e63f250db2f987f978f20f7aaaaf63510158c6766da06547d1e52775582ff31`), each read back at 87 files against its pod before the pods came down.

## Checks run at the head

- **CUDA library tests on an RTX 5090,** on both pods' builds at `074d54b2`: `113 passed; 0 failed; 5 ignored`.
- **Host:** `cargo test --locked -p uor-r4-float-train --lib` passes 71 tests.
- **The stub-nvcc type check** (`--features cuda --all-targets`) is clean. Clippy (`--no-deps -D warnings`) reports one error, `decode_speed`'s nine arguments, which is the same on `main`.
- **`uor-harness loop gates --full`, `loop report --check`, `loop pr-check --local`:** see the review record.

## Skills used

- **holding-the-goal.** The unit is the map's U2, under the owner's 27 Oct refocus direction. It answers which decoder serves, with a pre-registered rule, and stops there; the serving change is its own unit.
- **root-cause-debugging.** I-2 was classified before the run, not after it: two review rounds moved it from a tolerance to strict equality, because the two counts decode the same forward pass with the same tie rule.

## Still open

- **The serving change:** `fbs-toks` and `stack/session.rs` decode with the writer. Moving them to whole-table nearest, under D11's rules, is its own unit.
- **The writer at seed 2:** its admission fails (measured, above) while the trunk learns; why is not known. At seed 1 the writer under `voronoi` is the line's best decoder at openings; a writer that worked at both seeds would be the cheap index the large vocabulary needs.
- **The owner's next unit,** given in claude's chat on 27 Oct while this run was finishing: "After the writer unit closes, go straight back to the Hamming read: give its type heads a position signal from SpiralCore's spin, so replies can use the prompt."

## Plain summary

The model had two ways of picking its next word. The one the laptop used (the writer) and the one every score was measured with (whole-table nearest) disagreed, so the laptop wasn't running what the scores described. This run tested whether the writer could be trusted, with a training setup that fixes its known collapse. At one seed it worked better than ever, and gave the most varied, fitting reply openings yet. At the other it never learned at all. A decoder that works at one seed of two can't be the one served, so the decision is to serve whole-table nearest: the same decoder all the measurements, the Hamming read's included, have used all along.
