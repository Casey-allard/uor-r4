# REPORT: route-plus-memory, claude-161

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [route-plus-memory-2026-10-28](../../../../harness/prep/route-plus-memory-2026-10-28.md) (TEST FITNESS: FIT, PRE-RUN: PASS). Run head: `05fbfd5ba461bf8c3a76e16287467885be23a79e`.

**Cost per token first.**
- **Training:** the turn memory costs no measurable training time. (m) trained at `31,851 / 29,564` context tokens/s, against its empty control's `31,852 / 29,543`, on the same pods.
- **The table:** `26,450` keys of 8 ids each. That is `952,200` bytes as stored (u32 ids), or about 476 KB at the ids' u16 width (arithmetic).
- **Per token:** one key of 8 ids, one hash lookup and one 256-coordinate move, at a reply's opening row only. Every other token reads nothing.

**The result: cell A at both seeds.**
- **Openings:** with an exact memory of TRAIN, keyed by the user's turn and read at the readout, the model names `Ō = 598.7 / 598.3` of DEV's 1,226 reply openings. That is against `302.0 / 228.0` for (a), `266.3 / 281.0` for the empty control and `234.7 / 266.3` for claude-155's bag. The bar was `347.3`.
- **Fit:** top-1 holds at `9,490 / 10,368`, against (a)'s `9,328 / 10,573`.
- **The model uses it:** taking the memory out of the trained model drops openings to `84 / 108`.

## What did not work, first

- **The first launch stopped at its own CUDA test.** The test asserted that, at weight 1, the moved point equals the stored token's point bit for bit. Float rounding put it one ulp away (`3.7500002` against `3.75`).
  - This is an instrument fault in the test, not in the read. Nearest decoding names the stored token by a margin of about 1e-7 against table spacings near 0.25.
  - The zero-weight check, which is exact, passed.
  - The test now allows `1e-6`, and the doc says "to within float rounding". Both jobs were rerun from the fixed head `05fbfd5b`, into new folders on the same leased pods (L-002), and the 2 minutes of GPU cost nothing else.
- **The copies do not rise, and at seed 2 they fall.** (m) names `1,713 / 2,333` of 4,130 copied rows, against (m0)'s `1,750 / 3,029`.
  - The memory moves only opening rows, but DEV's opening and copy rows overlap: 163 rows are both (1,226 + 4,130 + 9,094 = 14,450 against 14,287). Taking the memory out costs `42 / 30` copies (`1,713 → 1,671`, `2,333 → 2,303`), and the drop in `T` equals the drop in openings exactly (`513`, `489`), so no other row moves.
  - The rest of seed 2's gap to (m0), about 700 copies, is training's own spread or a side effect of the trunk leaning on the memory. This run cannot tell which.
  - (a)'s own seeds read `1,517 / 2,933`. No bar was set on copies.
- **The replies are still templated after their first word.** The memory sets the opening, and the trunk writes the rest:
  - "Got it, the code word is kandg." (seed 2);
  - "Got it, your friend is  gets whations …" (seed 1).
- **Reach does not grow.** `R16` is `0.012 / 0.027` for (m) and `0.018 / 0.035` for (m0), as expected of a read that acts only at openings.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_memory.rs` (new): the turn memory.**
  - **`build_table`:** for each first-`k` key, the commonest opening, ties to the smaller id.
  - **`TurnMemory::stored`:** at the caller's opening rows, the key is the turn anchor's first `k` ids.
  - **`TurnMemory::apply_readout`:** `n + w ⊙ (c − n)` at hit rows, with `w` per coordinate starting at 0.
  - **Save and load:** the table and the weight, with an empty table round-tripping as empty.
  - **Thread-locals:** one for the memory, one for each forward's opening rows.
  - **Tests:** 4 host tests (the table rule, the key, named rows only, the refusals) and 2 CUDA tests (zero weight is no read, bit for bit; the move and its gradient equal the stored point's pull; save and load).
- **`geometric_stack_native.rs`:** the memory moves the readout after the final norm and the existing readouts, at the named rows, with a stage-clock mark.
- **`in_space_long.rs`:**
  - `LongConfig.turn_memory` and `turn_memory_empty`, left out of `config.json` when off;
  - `start_memory`, which builds the table from every TRAIN window as the probe does, and each usable window's opening offsets;
  - each training batch names its opening rows;
  - `nearest_reading` names DEV's opening rows;
  - `greedy_reply` names the reply's first step;
  - checkpoints save the memory;
  - `--inspect` gives `memory_weight_mean_abs` and the removal reading `memory_drop`;
  - `run_long` logs the read's own TRAIN and DEV hit counts (the FIT's note);
  - two host tests: the config keys, and the refusal of an empty control with no memory.
- **`bin/train-in-space.rs`:** `--turn-memory K` and `--turn-memory-empty 0|1`.
- **The item** is set `done`.

## Measurements

**The run:** two RTX 5090 pods (Vast `vast-58156828`, `vast-58157501`), one per seed, at the pinned TRAIN. Each ran (m) then (m0), 16,384 steps each, the binary built and tested on the pod from the run head. The pods were up about 1.4 and 1.5 hours, at about $1.0/h each: about 2.9 pod-hours and $2.9.

**Instrument checks:**

| Check | Seed 1 | Seed 2 |
|---|---|---|
| I-1 CUDA tests on the pod's build | `158 passed; 0 failed; 5 ignored`, the memory's two among them | the same |
| I-2 step 0 equals (a)'s, both arms | hits 2, openings 0, 730 picks | hits 5, openings 3, 681 picks |
| I-2 `config.json` | width 256, 4 heads, `turn_memory` 8, (m0) empty, pinned TRAIN | the same |
| I-3 the table: keys | (m) `26,450`, (m0) `0` | the same |
| I-3 the read's own hits, TRAIN openings | (m) `212,905 / 212,905`, (m0) `0` | the same |
| I-3 the read's own hits, DEV openings | (m) `1,194 / 1,226` (32 opening rows whose key TRAIN never saw), (m0) `0` | the same |
| I-3 the weight's mean magnitude at the end | (m) `0.563`, (m0) `0.0` | (m) `0.656`, (m0) `0.0` |
| I-4 losses | all 17 finite, every arm | the same |
| I-5 the removal reading, (m) | openings `597 → 84`, T `9,490 → 8,977` | openings `597 → 108`, T `10,368 → 9,879` |
| I-5 the removal reading, (m0) | unchanged | unchanged |

The TRAIN hits answer the FIT's note: the table's key (the probe's prompt cut) and the read's (the turn anchor) agree at all 212,905 TRAIN openings.

**The readings,** on DEV, of 14,287 rows:

| Arm | `T` at 16,384 | Openings at 12,288 / 14,336 / 16,384 | `Ō` | Copies (of 4,130) | Rest (of 9,094) | Distinct opening picks |
|---|---|---|---|---|---|---|
| (m) seed 1 | 9,490 | 600 / 599 / 597 | **598.7** | 1,713 | 7,255 | 76 |
| (m) seed 2 | 10,368 | 600 / 598 / 597 | **598.3** | 2,333 | 7,512 | 65 |
| (m0) seed 1 | 9,252 | 273 / 257 / 269 | 266.3 | 1,750 | 7,330 | 81 |
| (m0) seed 2 | 10,566 | 311 / 254 / 278 | 281.0 | 3,029 | 7,368 | 90 |
| (a), claude-152 | 9,328 / 10,573 | — | 302.0 / 228.0 | 1,517 / 2,933 | 7,600 / 7,490 (claude-153's sheet) | — |
| (g), claude-155 | 8,993 / 9,253 | — | 234.7 / 266.3 | 1,158 / 1,560 | — | — |

**The bar** (the sheet's), for both (m) seeds:
- **Against (a):** `Ō ≥ 347.3`.
- **Against (g):** `Ō ≥ 309.7`.
- **Against (m0):** `Ō ≥ 281.0 + 3·√(1,226 · 0.2292 · 0.7708) = 281.0 + 44.2 = 325.2`.

The largest is `347.3`, and **both seeds pass, at `598.7` and `598.3`.** Flag F (`T < 8,828`) does not fire (`9,490`, `10,368`). That is **cell A**: the memory carries what the trunk could not store.

**The ceiling:** the probe's lookup alone names `607` of 1,226. The model, with the memory, names about `598`, so the trunk and the memory together reach about 98.6% of what the table can name.

**Cost:**
- **Training rate:** (m) `31,851 / 29,564` against (m0) `31,852 / 29,543` context tok/s, and (a)'s `32,049` (claude-152, as the `fbs-cost-profile` item quotes its notebook row).
- **The runner's decode:** (m) `179 / 116` against (m0) `177 / 116` tok/s.
- **The table:** `26,450 × (8 + 1) × 4 bytes = 952,200` bytes stored, about 476 KB at u16 ids (arithmetic).
- **Per token:** at a reply's opening row, a key of 8 ids, a hash lookup, a row load (256 values) and a 256-coordinate move. Other tokens: nothing.
- **The move's integer form** is the item's open part. It needs the integer runtime's read path (the pipeline map's R5).

**Sealed:** `caseyallard/uor-r4-store:results/claude/route-plus-memory-2026-10-28/`, read back with the same sha256:
- `claude-161-s1-05fbfd5b.tar`, sha256 `026125e43d5edb79cb937ccfd46ff5b3d478f9358abc449c56651ebe987954a9`;
- `claude-161-s2-05fbfd5b.tar`, sha256 `ddb95f20008631aa48417d5a2b4416f1540ab53176b2efcf2c57b2df026ba7f1`;
- `SHA256SUMS-claude-161-s1` and `SHA256SUMS-claude-161-s2`, with `job.sh` and `watch.sh` beside them.

## Checks run at the head

- **`cargo test -p uor-r4-float-train --lib -- flock_read_memory in_space_long`** (host): `28 passed; 0 failed`. On the pods, the crate's full CUDA suite: `158 passed`.
- **`cargo clippy -p uor-r4-float-train --no-deps --all-targets -- -D warnings`:** no finding in this unit's lines. The four that remain predate it.
- **`cargo fmt --check`:** clean. **The banned-word scan of the diff:** clean.
- **`loop report --check`:** PASS. **`loop gates --full`:** 11 of 11 green.
- **The fresh in-unit review, in two parts at `e06f3b0f2`:**
  - **The code:** a provisional APPROVE, with no finding above low. The reviewer's four open checks were closed by the top session:
    - training windows are context-long by `load_train`'s refusal, so `b · SHEET_CONTEXT + offset` is `assemble`'s layout;
    - `points` requires the same layout on DEV;
    - the weight is always in the graph, so its gradient exists when no row hits;
    - sample prompts hold at least 16 tokens.

    Its two cheap guards are added, and neither changes a measured path: a batch that is not context-long is refused, and an empty reply window names no row. Its third note, a forward test of named rows, is covered on GPU by the run's I-5 and by the 212,905 TRAIN hits.
  - **The numbers against the sealed outputs:** CHANGES. Every listed figure matched; one medium and three low findings, all fixed:
    - **medium:** opening and copy rows overlap (163 rows), so the memory reaches some copies, `42 / 30`;
    - "no loss of accuracy" now names seed 2's 200-row gap;
    - the 32 unseen are opening rows, not keys;
    - two reference figures are cited.

## Skills used

- **holding-the-goal.** The owner voted for the diagnosed roadmap, and this is its decisive run. It tested storage, with a no-memory control, and nothing else.

## Still open

- **The move's integer form,** for serving: the integer runtime has no read path yet (R5).
- **Beyond the opening:** the memory names a reply's first word. The rest of the reply is still the trunk's templates.
- **The table's reach:** it names openings for keys TRAIN has seen. `32` DEV opening rows have a key TRAIN never saw, and a lookup can only be as good as the commonest opening per key (the `607` ceiling).
- **The copy drop at seed 2,** unexplained, with no bar on it.
- **PLAN's 28 Oct pivot direction** ("Done when all three are merged") can now be set `done`. `PLAN.md` is outside this scope, so that is the next item-maintenance unit's.

## Done when

1. **Met.** The read is built with the item's tests. Host tests pin the key and table; CUDA tests pin zero weight as no read and the move as the stored point's pull. With no new flag, every config key is left out (a host test).
2. **Met.** The sheet had a fresh FIT and pre-run review. (m) and (m0) ran at two seeds, read as the last-three mean, and were barred against (a), (m0) and (g). The removal reading is reported.
3. **Met.** This REPORT leads with cost per token. There is one notebook row, and the result lands on main with the merge.

## Plain summary

The model could route a user's message into itself, but it had nowhere to keep the "this kind of question starts its answer with this word" associations. This unit gave it an exact memory, built from the training data: the first 8 words of the user's turn look up how such replies usually start.
- **The effect:** the model now gets the first word of a reply right about 599 times in 1,226, about twice as often as ever before. That is close to the 607 the lookup itself can manage, with overall accuracy kept (at seed 2 about 200 rows below (a), inside the pre-registered flag) and no extra training time.
- **The model relies on it:** with the memory taken away, it gets fewer than 110 right.

The rest of each reply is still weak.
