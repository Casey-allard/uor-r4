# Prep sheet: how the one model grows with width and data, on human-written text (scale-probe, P5, claude-294, 5 Nov 2026)

Lab: claude. Item: [scale-probe](../../roadmap/items/scale-probe.md) (#3472). Casey's 4 November plan (the lab's list, which Casey adopted that day), P5: the bigger train is `scale-probe`, bounded at one sheet unit and one or two run units. It comes in the order decided on #2028 (6634218223): step 6, "P5 on real text", after the real-text corpus (claude-292, claude-293) and P4.

TEST FITNESS: FIT WITH CHANGES, from three rounds of fresh non-author review, with every change applied by `2c3a96147c98791dfd01d3304c6b0554ae77a12e` (#3472 comment `6666302038`). No round returned a plain FIT. Round 3 read only the round-2 delta, and no reviewer re-read its one applied sentence. Round 1 returned FIT WITH CHANGES with seven items, fixed in `fc255e93a9a3da0c72c27e41b5596f589368e2f7` and `50048830f441ede6a495a755270d5300df12fec8`. Round 2, four narrow reviews, returned FIT WITH CHANGES, fixed in `1402f3cb66a889f27e15fa0b0de5c518572d16ea`. Round 3 returned FIT WITH CHANGES with one must-fix, applied verbatim in `2c3a96147c98791dfd01d3304c6b0554ae77a12e`. This line was added after those reviews. So were two wording edits from the card's decision review: the scores' departure from the order's words, and OASST2's English filter. They change no arm, reading or bar.

PRE-RUN: owed, by a fresh non-author review in the run's unit, right before launch (D67), as P2b's sheet had (claude-276).

**What this run measures.** Every reply result on this line comes from one width (256, once 512) and one templated corpus of 7.5M tokens (the pipeline map's gap 9). This run trains the one model at widths 256, 512 and 1,024, on a quarter, a half and all of a 28M-token corpus of human-written text, at two seeds, for equal positions. It then scores every held-out position of that corpus's sealed held-out splits, which no model has read. The question is the slope: whether the model gets measurably better as it gets wider, and as it sees more text. The object is the float model, trained as served, scored by the float scorer. The integer exports are timed and checked for parity; no integer score is claimed.

## Question

- **Width:** at all of the data, does width 1,024 get more held-out next tokens right than width 256, by more than the paired band, at both seeds?
- **Data:** at width 1,024, does all of the data get more right than a quarter of it, by more than the band, at both seeds?
- **Baseline:** does the widest, fullest cell beat the count tables built from its own TRAIN (T2, the bigram on the previous token; and the unigram)?
- **Cost, led by tok/s:** training positions a second at each width, and the exported artifact's decode tok/s against context at each width.

## Base artifact

A training comparison from scratch; the candidates are the checkpoints each cell produces.
- **The pin** is this sheet's merge commit on `main` (claude-294), recorded in the run unit's ORDER at launch. Training, the smokes, I-0, scoring and the snapshots use `train-in-space` and `fbs-snapshot` built `--locked --features cuda` at the pin. Each pod fetches the pin, checks it is that commit and clean, builds, and records each binary's SHA-256.
- **The recipe is claude-266's (b) without its turn memory:** `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1`, with `--turn-memory` left at its default 0.
  - (b) is what this line trains and serves today: P2b trained it (claude-275, claude-276), and it is served as claude-243's a7r and P4's carry exports (claude-288).
  - **Why the turn memory is out:**
    - this probe asks how the trunk scales, and the memory stores answers beside the trunk;
    - Wikipedia's articles have no prompt for the memory to key on;
    - and the memory's host cost on 28M tokens of real text is unmeasured (the census's reply rows cost 0.67–0.75 GB per million real TRAIN tokens, claude-293).

    A recipe `one-model-end-to-end` picks later is a new configuration of this probe, not a fault in it.
- **The scale-probe instrument** (`docs/labs/scale-probe/instrument`, bin `scale-probe`, laptop counting only) builds the mixes (`mix`), counts each panel row's overlap with a TRAIN (`seen`), grades each checkpoint's rows against its own TRAIN's count tables (`grade`), and pairs cells on all rows and on the unseen rows (`pair`). It is merged with this sheet.
- **The outputs** go under `results/claude/scale-probe-2026-11-05/runs/`.

## Arms

**The grid:** 3 widths × 3 data sizes × 2 seeds = 18 runs. Every run takes the same positions: `--context 192 --batch 16 --steps 8192 --eval-every 1024`, that is 25,165,824 positions.
- **Widths:** `--width 256 --hamming-read 4`, `--width 512 --hamming-read 8`, `--width 1024 --hamming-read 16`. `--hamming-read` is the width over 64, so each Hamming head is 64 wide, the most a head may be, and half the heads are copy heads (`HammingRead::new`, `flock_read_hamming.rs:956-964` and `:1028-1030`).
  - **Every limit admits 1,024, inclusively:** `HammingRead::new` asks for at least 2 heads that divide the width, each a multiple of 4 and at most 64 wide; the stack asks for a width divisible by 4 that its heads divide into an even head width (`StackConfig::validate`, `geometric_stack_native.rs:945-957`); the readout's writer admits at most 1,024 in whole 8-wide blocks (`WriterSpec::validate`, `native_vector_writer.rs:32-40`).
  - **The model:** the trainer builds it with `build_fbs_model_at` (`in_space_long.rs:870`), which reaches `build_hamilton_model_at` (`hamilton_sheet.rs:3111-3133`) with only the width changed, as claude-153 ran 512.
  - **What grows with the width:** the stream; the 4 stack heads' width, width/4 (64, 128, 256); the Hamming heads' count; and the writer's 8-wide blocks, width/8.
  - **What stays:** the six layers, the 4 stack heads' count, the Hamilton gate's hidden width of 384 (`SHEET_MLP_HIDDEN`), and the vocabulary of 4,096.

  So "width" here means the stream's width and what is sized from it, not every part's.
- **Data:** `--train-split Dk --train-sha256 HEX --train-mask-sha256 HEX` with D1, D2 or D3 (Data split). The run's `--data-root` is claude-261's pinned DATA, for the trainer's in-run DEV evaluation, which is reported only.
- **Seeds:** `--seed 1` and `--seed 2`.
- **The contrast cell** (the decided order: "The templated data stays as one contrast cell"). Widths 256 and 1,024 on the pinned templated TRAIN (`mw-balp`, `--train-split` omitted), seed 1, the same positions and flags. 2 runs, reported only.

**Scoring.** Each checkpoint at step 8,192 is scored once: `--score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed S --context 192`.
- A grid checkpoint is scored with the four held-out panels: `--panel oasst2=H1 --panel wiki-00000=H2 --panel wiki-00020=H3 --panel wiki-00040=H4`, each `Hk` the matching root's `heldout/`.
- A contrast checkpoint is scored with the same four panels, and graded against its own templated TRAIN's tables. It is reported only.

`--score-rows` writes each split's rows twice, `rows-{split}-memory.json` and `rows-{split}-no-memory.json`. With no turn memory both passes read the same model (I-5). The `no-memory` files are graded.

**Pod plan.** Three 1 × RTX 5090 pods; each runs its cells alone on GPU 0, one after another:
- **pod A** (seed 1): width 1,024 at D1, D2, D3, then width 256 at D1, D2, D3;
- **pod B** (seed 2): the same cells;
- **pod C:** width 512 at D1, D2, D3 for seed 1 and seed 2, then the contrast cell's two runs.

## Readings

All readings are taken on the laptop from the sealed `no-memory` rows files, through `scale-probe grade` (one checkpoint against its own TRAIN's tables) and `scale-probe pair` (two cells at both seeds). A row is one scored held-out position. A hit is the whole-table nearest pick (`nearest`) equal to the target. Rows are pooled over the four panels unless a reading says otherwise.

**Not §4.8's integer top-1.** The decided order scores real text by "integer top-1 against unigram and T2 count tables". The decision card on #3472 departs from those words: the deciding count is an exact count of the float model's whole-table nearest top-1 hits, with no likelihood, not §4.8's integer top-1 through the D11 export. The pick is the float model's, trained as served (`--train-as-served 1`). The slope is therefore a float-model measurement. The integer artifact is checked by R9's parity, not scored on the panel, and the D11 export's panel top-1 is owed before any width is served or any claim is made (the decision card on #3472).

**The two row sets.** Every row of a panel is in exactly one set:
- **unseen:** at least 8 tokens of its own document precede it, counting its `<|bos|>`, and those 8 tokens do not occur anywhere in the TRAIN the seen set is built against (cell X's, below) as a run inside one document;
- **excluded:** every other row.

`scale-probe seen` writes this bit for every row (`excluded.bin`) against one TRAIN. `pair clean=` reads every comparison on all rows and again on the unseen rows, with the seen set of cell X's TRAIN:
- **X is always the larger mix.** Since D1 ⊂ D2 ⊂ D3, a row unseen in X's TRAIN is unseen in Y's too, so the unseen rows are unseen by both cells.
- **So R1, R2 and R3 read D3's set,** and a pair at D1 or D2 reads that mix's set.
- **`pair` refuses** a seen set built on any TRAIN but X's.
- **R1 (deciding): width.** (1,024, D3) against (256, D3), paired by row at each seed.
- **R2 (deciding): data.** (1,024, D3) against (1,024, D1), paired the same way.
- **R3 (deciding): the tables.** (1,024, D3) against its own TRAIN's T2, and against its unigram (`pair table=t2`, `table=unigram`).
- **R4: tok/s, first in the REPORT.**
  - Training positions a second at each width, from every run's trainer clock.
  - Served decode tok/s against context at each width. The D3 seed-1 checkpoint is snapshotted on its pod (`fbs-snapshot --dev DATA`) and exported on the laptop (`fbs-export --raw-gates --raw-transition --run-config CONFIG`), as claude-243 exported a7r. Then `fbs-toks ARTIFACT --context-sweep --tokens TOKENS_U16` times it on the laptop with p-long4k's ids, as claude-288 ran it.
- **R5: the slopes.**
  - **Per doubling:** the width slope at each data size and the data slope at each width, in rows a doubling. With three evenly spaced points, the least-squares slope is half the end-to-end difference, `(y_last − y_first)/2`. Paired by row, that difference is the end pair's `d`, so a slope is distinguishable from flat exactly when that pair's verdict is ahead or behind. The two deciding slopes are R1's and R2's.
  - **The end pairs' verdicts:** each other width pair (1,024 against 256 at D1 and at D2) and data pair (D3 against D1 at widths 256 and 512) is read by `pair`, reported.
- **R6: the shape.** Adjacent pairs (512 against 256 and 1,024 against 512 at D3; D2 against D1 and D3 against D2 at 1,024) by `pair`, and the curvature `y_last − 2·y_mid + y_first`. Reported only.
- **R7: by source.** R1 to R3 on OASST2's replies alone, and on the three Wikipedia panels together; `pair` prints each split. Reported only.
- **R8: the contrast.** Reported only.
  - DEV top-1 for the contrast cell's two widths, of 14,287, from three sources:
    - **`T`,** the writer's top-1 in the trainer's in-run DEV evaluation at step 8,192 (`dev_top1_hits`, `in_space_long.rs:11586`). The writer reads at Q12 in the D11 mode, on tables rounded as exported, and picks from its admitted cells and their rivals, not the whole table. It is the reading claude-152's and claude-153's `T` came from, and it is set beside claude-152's `9,328 / 10,573` (arm (a), seeds 1 and 2, width 256, step 16,384) and claude-153's `10,152 / 11,140` at 512, as writer top-1 context: other recipes and steps, not a series.
    - **The trainer's whole-table nearest count** (`dev_nearest.hits`) on the same `progress.jsonl` line.
    - **`--score-rows`' DEV rows** (`rows-dev-no-memory.json`): their nearest count, and `grade`'s count against the cell's TRAIN tables.

    The two nearest counts should agree. A gap is reported and classified (the rows or the served rounding) before R8 is read. The panel readings come from `--score-rows` alone and do not rest on it.
  - The contrast cell's top-1 on the four real-text panels, beside the grid's (256, D3) and (1,024, D3): what a model trained on templated text carries to human text.
- **R9: export parity.** `fbs-compare --parity-only` of each R4 export against its own D11 readout, on its pod. An integer panel top-1 is not run here; it is owed for any width served later.
- **Bytes per token,** per source under the pinned tokenizer, from each held-out manifest's `content_bytes` and `tokens`: OASST2 560,402 / 214,758 (2.609), Wikipedia 3,100,628 / 1,276,860 (2.428), pooled 3,661,030 / 1,491,618 (2.454). Reported beside every top-1, so a reading per token can be read per byte. Bits per byte waits for a lawful likelihood (§4.8).

## Bar

For cell X against cell or table Y, pooled, at each seed `s` in {1, 2}:
- **The counts.** `b(s)` is the rows only X gets, `c(s)` the rows only Y gets, and `d(s) = b(s) − c(s)`, which is X's hits less Y's on the same rows.
- **The band:** `band(s) = max(⌈2·√(b(s)+c(s))⌉, max_s d(s) − min_s d(s))`. That is the paired floor, or the spread of `d` over the two seeds; a table has no seed, so against a table `d(s)` varies only with X's seed. This is claude-205's and the one-model protocol's rule, at two seeds.
- **The verdict.**
  - **ahead:** `d(s) > band(s)` at both seeds;
  - **behind:** `d(s) < −band(s)` at both;
  - **inside:** `|d(s)| ≤ band(s)` at both;
  - **mixed:** otherwise.

  "Inside" means not distinguishable from flat at this power. It is never read as "no difference" (L-007).
- **At three seeds** (only when the table below sends a reading to seed 3), `s` runs over {1, 2, 3}. The spread is taken over all three, and ahead, behind and inside each need all three seeds. `scale-probe pair a1= a2= a3= b1= b2= b3=` computes it, and `a1= a2= a3= table=` against a table. Its tests pin the three-seed spread, the three-seed agreement, and that a third seed trains on its cell's TRAIN, each with a negative gate.
- **Both row sets.** R1, R2 and R3 each take the verdict above on all rows and on the unseen rows.
  - If either set's verdict is mixed, the reading is **mixed (seeds)**.
  - Otherwise, if the two verdicts agree, the reading is that shared verdict.
  - If they differ, it is **mixed (row sets)**: what the model gains or loses depends on whether TRAIN holds the context. Nothing is named, the next unit classifies it (overlap or power), and seed 3 does not run for it.

**What each outcome decides.** R1 and R2 each get their reading first. The rows are then tried in order, and the first that matches decides. After rows 1 to 3, both readings are ahead or inside, so rows 4 to 6 cover what is left, each case once.

| Order | R1 (width) | R2 (data) | Reading | What it unlocks |
|---|---|---|---|---|
| 1 | mixed (seeds) | or mixed (seeds) | **Undecided.** | A pre-registered seed 3 runs for the three cells R1 and R2 read: (1,024, D3), (256, D3) and (1,024, D1). The table is then read again at three seeds, on both row sets. If row 1 still matches, the result is inconclusive, and nothing is named or closed. |
| 2 | mixed (row sets) | or mixed (row sets) | **Seen and unseen text disagree.** | Nothing is named. The next unit classifies it (overlap or power); no seed 3. |
| 3 | behind | or behind | **Worse with size.** | The next unit classifies it, instrument or mechanism, before anything is scaled: equal positions may under-train a wider model. The other reading's verdict is reported beside it. |
| 4 | ahead | ahead or inside | **Width 1,024 beats 256 at D3 (the float model, these panels).** | Scaling the trunk is next, with R2's verdict reported beside it. A width past 1,024 needs the writer changed first (`native_vector_writer.rs:32-40` admits at most 1,024), so that item is opened. |
| 5 | inside | ahead | **All of D3 beats a quarter of it at width 1,024 (the float model, these panels); width is inside the band.** | Growing the data is next: more Wikipedia shards, with the same bin. Width is not read as flat (L-007). |
| 6 | inside | inside | **Neither is distinguishable from flat at this power.** | Nothing is named or closed yet. The next unit classifies it first, instrument or mechanism (§4.2): the band against the gain these sizes could show, under-training at equal positions, and the learning rate held at 0.001 for every width. Only a mechanism classification closes this configuration, recipe (b) without memory at 25M positions and these widths and fractions (§4.1), and never the mechanism. |

- **R3, read on its own,** with the both-row-sets rule above. The same forms are used against the unigram.
  - **Ahead:** the REPORT says "Measured: the float model trained as served at width 1,024 on D3 gets more whole-table nearest top-1 hits than D3's bigram table (T2) on these four held-out panels, on all rows and on the rows whose context D3 does not hold, beyond the paired band at both seeds."
  - **Inside:** it says the model and T2 are not distinguishable at this power on these panels. That is never read as "no better than T2" (L-007).
  - **Behind:** it says T2 gets more of these panels' rows right than the model, beyond the band at both seeds. The next unit classifies it, instrument or mechanism, before anything is built on it.
  - **Mixed (seeds):** it says the two are not distinguishable at these seeds. No seed 3 runs for R3.
  - **Mixed (row sets):** it gives each row set's verdict by name (for example, ahead on all rows and inside on the unseen rows), and names nothing.
- **Whatever the outcome:**
  - R4 to R9 are reported with their numbers.
  - No reading here is a served result or a milestone claim.
  - The pipeline map's gap 9 records the slope as measured on this corpus, not as a continuation of its templated points.
  - An instrument check that fails leaves every reading unread, and the run is repaired, not graded (§4.1).

**Reversal.** Casey can say "hold": no further pod launches, and nothing is named or closed. The checkpoints and rows stay sealed under the results root.

## Instrument checks

- **I-0:** on each pod, the CUDA library suite at the pin, `cargo test --locked --features cuda -p uor-r4-float-train --lib`. It passes if at least one test ran and none failed.
- **I-1 (the smoke):** on each pod, each width it trains runs 64 steps on D1 (`--steps 64 --eval-every 64`, its own flags otherwise), exits 0, and its loss is finite. Its peak GPU memory is recorded. One smoke per width is then scored as above, with the four panels (pod A's at 256 and 1,024, pod C's at 512), and graded on the laptop: every check in I-4 passes, and the three smokes score the same rows.
- **I-2 (the projection):** each smoke's step time projects that pod's training. If the three pods project past 20 GPU-hours in all, training and scoring together, every run's steps are cut by the same factor. The cut is rounded down to a multiple of `--eval-every` (1,024, or 2,048 for a batch-8 width), so every run ends on an evaluation. The trainer's only refusal tying `--steps` to `--eval-every` is for `--profile` (`train-in-space.rs:1484`), which these runs do not pass. The cut keeps every cell and every seed, and it is recorded.
- **I-3 (the inputs):** on each pod, `shasum -a 256 -c SHA256SUMS` passes in each input folder, and every SHA-256 the Data split names equals its file's. Each run's log names the TRAIN sha it loaded.
- **I-4 (the grade):** `scale-probe grade` refuses, and so fails this check, on any of these:
  - a row whose target is not the panel's token at its position, or a row on a position the scorer does not score (mask 0, `<|bos|>`, position 0);
  - a duplicate position;
  - a panel or TRAIN file whose SHA-256 is not the one given.

  `scale-probe pair` refuses two checkpoints whose rows are not the same positions, in the same order. So all 18 grid checkpoints must score the same rows. Coverage, rows over the panel's scored positions (1,460,697), is reported. It should be all of them (Data split); a shortfall is reported with the positions it misses.
- **I-5 (the scorer's determinism):** for each checkpoint, the `memory` and `no-memory` rows files are compared. With no turn memory they read the same model. The scoring command passes no decision-trace option: the trace is taken in the memory pass alone (`in_space_long.rs:7871-7873`), so it would make the two passes differ. Reported: whether their SHA-256s match, and if not, how many rows differ and between which picks. A mismatch does not stop the run; the `no-memory` files are graded either way.

## Data split

- **The TRAIN roots,** claude-293's real-text corpus, from the store's `results/claude/real-text-corpus-2026-11-05/` (61-line `SHA256SUMS`, read back in claude-293):

  | Root | Source and licence | Docs | Tokens | Scored | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 |
  |---|---|---|---|---|---|---|
  | `oasst2/train` | OpenAssistant/oasst2, Apache-2.0 | 5,071 | 3,773,419 | 3,234,805 | `8bdcabfb41284a54575ea2f730f532a896b29dc69250098957b46c6bcae40b90` | `a2b0b2a3ce8cc7491b8ced86eda487353ba4e70da0f9aa5a63161ca6802de7a1` |
  | `wiki-00000/train` | wikimedia/wikipedia 20231101.en, CC BY-SA 3.0 and GFDL | 864 | 8,011,538 | 8,010,674 | `4dd788cc79082dc6c8520acbcd6c2287e5c5a1c0e82780a879126afd05b32da2` | `95bc08133735a66f6aa98efa37d4196b1fd1769f4cca2e6828090c888eac712b` |
  | `wiki-00020/train` | the same | 7,970 | 8,255,877 | 8,247,907 | `e1945fa285b6e0abd586cc6b1e13854889d73c5d8ebd5b36a1ce90a1f3e081a2` | `8e2a012d67d366f7a668b6188cf0841083f5ff3a0a11d3e9ea1337ea0470aed4` |
  | `wiki-00040/train` | the same | 4,111 | 7,927,279 | 7,923,168 | `51b11b291fc819ec0cd3b3c5f29b07e4e30bb5f619b12dff42c3ed8c17b3dd99` | `ce266ee53a55431b71d9b4f854708ffb4ccd098569431be2b4f4ab0360aabf63` |

  "Scored" counts the mask-1 positions, which are the positions the trainer scores: OASST2's replies, and every Wikipedia token after `<|bos|>`.
- **The mixes,** written on the laptop by `scale-probe mix fraction=F` over the four roots in that order. For each root it takes the shortest prefix of whole documents that holds at least F of its tokens, so D1 ⊂ D2 ⊂ D3, root by root. Concatenation order does not matter: the trainer shuffles windows every epoch.

  | Mix | F | Tokens | Scored | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 | Passes in 25.2M positions |
  |---|---|---|---|---|---|---|
  | D1 | 1/4 | 7,004,928 | 6,865,470 | `21cb8d3ba059de022ca3c5897658dc0982af9c0cf7154295754bba480591d61b` | `8c2f4d1184509aef5221da522b161bb684dc40a19f86710c1817a6391661b1ac` | 3.67 |
  | D2 | 1/2 | 13,989,169 | 13,708,035 | `a83f7382637bac1f46c339c67e718909cf7092bf70b522ac684fcc9e698057b5` | `37f6bec1dc4fe317417924f29499040e354f7fdb6cd430d3dd59411482d6d3b4` | 1.84 |
  | D3 | 1 | 27,968,113 | 27,416,554 | `ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52` | `ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875` | 0.92 |

  D1 holds 4,653 documents, D2 9,053 and D3 18,016. D3 is the four roots whole. "Passes" is 25,165,824 positions over the mix's scored positions. Each mix's `manifest.json` is pinned too: D1 `a30252c026d3701a61ca9ab2b255201faae1ef4561a0f35ed48bbc721d39b764`, D2 `4ea8b84d77959eb5b2b4031a57da01afae294bf0f0303ddad347b53d4ce89b90`, D3 `57321710bb37801580628f6630e6d2ce184ff7793d99f89de607fa0fa714f011`.

  Store: `results/claude/scale-probe-2026-11-05/inputs/`, with a `SHA256SUMS` (its own SHA-256 `56a7db05a8f35b01e7231e90f95c2e7a095415263e2c16adcd93a20968c60b9c`), fetched back and checked byte for byte in this unit.
- **Human-written text only,** under the decided order's interim §3.10 reading, until Casey's readings are given; another answer is a new configuration.
  - OASST2's node filter drops every message marked `synthetic`, and found none in either split (its manifests' `synthetic: 0`).
  - It keeps English threads only (`lang == "en"`, `real_text.rs:574`): claude-293 dropped 8,451 roots and 4 child messages as not English.
  - The Wikipedia articles are its editors' text, from the English edition (`20231101.en`).
  - UltraChat, SmolTalk and FineWeb-Edu stay out.
- **The panel: the four held-out splits, in full,** from the same store root. Each was cut by source id (OASST2 tree, Wikipedia article) at 50 per mille. No model has read any of them. Only counting has: claude-293's census and its host-memory probe (which read an earlier build of `wiki-00000/heldout`), and this unit's `seen` census.

  | Panel | Docs | Tokens | Scored | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 |
  |---|---|---|---|---|---|
  | `oasst2` | 278 | 214,758 | 184,542 | `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e` | `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833` |
  | `wiki-00000` | 36 | 396,081 | 396,045 | `82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7` | `34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f` |
  | `wiki-00020` | 430 | 443,682 | 443,252 | `9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640` | `a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d` |
  | `wiki-00040` | 239 | 437,097 | 436,858 | `c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d` | `9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625` |

  The panel holds 1,460,697 scored positions. `--score-rows` cuts each document's targets into windows of at most 192 (`build_document_windows`, `hamilton_sheet.rs:1880-1920`). It drops only a document of one token or none and a window with no scored target, and it scores every other scored target exactly once. So the rows should be all 1,460,697 (I-4).
- **Its census (claude-293):**
  - **OASST2:** 2 of its held-out split's 511 prompts are also in its TRAIN, "Thank you" and "Can you write stories?", in other trees. The split is by tree, and people repeat openers. They are kept; they are two prompts, and the rows scored are replies.
  - **Wikipedia:** a split has no prompt runs, so the prompt census measures nothing. The builder's exclusion screen dropped 5 articles across the corpus against the frozen panels.
- **The overlap with TRAIN,** counted in this unit by `scale-probe seen` (see "The two row sets"). A held-out article can quote a TRAIN article, as Wikipedia articles quote each other, and a reply can repeat a common phrase. So the census counts every scored panel position (`rows=NAME:all`) against each mix:

  | Against | Seen | Short | Unseen | Unseen share | Distinct TRAIN 8-grams | `seen.json` SHA-256 |
  |---|---|---|---|---|---|---|
  | D1 | 129,173 | 4,935 | 1,326,589 | 90.8% | 6,174,698 | `2d74739a04ce2cd74e45b62b0ab24d8aff17c9ebeee23b857021232dc0e90dbd` |
  | D2 | 176,210 | 4,935 | 1,279,552 | 87.6% | 12,079,604 | `7262efe869962beba3533c69ca9890cfdafbbf041154bbccf81bc5d34509ca92` |
  | D3 | 227,312 | 4,935 | 1,228,450 | 84.1% | 23,521,412 | `0e7026ee3607b0863d79e8a50777d42f053de838fa926b70ec1b7b9a538be38b` |

  - **Against D3, by panel:** OASST2 26,155 seen of 184,542 (14.2%), no row short; wiki-00000 53,638 of 396,045 (13.5%); wiki-00020 85,342 of 443,252 (19.3%); wiki-00040 62,177 of 436,858 (14.2%). The short rows are the first 7 positions of each article.
  - **The seen share grows with the mix,** 8.8% to 12.1% to 15.6%, as nesting requires. So part of any gain R2 reads on all rows could be recall of contexts D3 holds and D1 does not. That is why R2 is also read on the rows unseen in D3.
  - **"Seen" is a screen, not a finding of copying.** It marks a row whose 8-token context occurs anywhere in TRAIN, whatever token followed it there. Common phrases count. So the unseen set is the conservative one.
  - Stored with a `SHA256SUMS` (its own SHA-256 `88929e87931bef0fd91213ffbeb4701038354aa6cceca8be3c1621f478399f67`) at `results/claude/scale-probe-2026-11-05/census/`, read back in this unit.
  - **The readings' bits are made in the run unit.** The census reads the panels' positions (`rows=NAME:all`), not a checkpoint's rows files, and `pair clean=` refuses a `seen` directory whose row identity is not the grade's. So step 8 runs `seen` again against D1, D2 and D3, with a grid checkpoint's own rows files (I-4 makes every grid checkpoint score the same rows), and `pair clean=` reads those. If the rows are every scored position, their counts equal the census's; any difference is reported.
- **The contrast cell:** the pinned templated TRAIN (`mw-balp/train`, tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`) and DEV (tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`), under claude-261's DATA pins. The tokenizer is `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **P-dev stays reserved** for `one-model-end-to-end`'s run, and nothing here reads it.

No model output was read to set any panel, mix or bar.

## Steps

1. On the laptop (this unit): build D1 to D3, seal them under `inputs/` with a `SHA256SUMS`, and read it back. Run `scale-probe seen` for each mix against every scored position of the four panels, and store it under `census/` (done; Data split).
2. On each pod: fetch the pin, build, record the binaries' SHA-256s, and run I-0.
3. Fetch the inputs and run I-3.
4. Run I-1's smokes; score and grade pod A's (on the laptop); project I-2.
5. Train each pod's cells in its order, then score each checkpoint.
6. On pods A and C, snapshot the D3 seed-1 checkpoints at their widths, and run R9's parity once each export is made on the laptop.
7. Seal every output with a `SHA256SUMS` under the store root, read it back, and only then take the pod down.
8. On the laptop: grade every checkpoint, and check I-4 and I-5. Run `seen` against D1, D2 and D3 with one grid checkpoint's rows files. Then compute R1 to R9 (each pair with and without `clean=` the directory of its cell X's TRAIN), apply the Bar, and time the exports (R4).

## Minimum test power

The panel holds up to 1,460,697 rows. The band's floor, `⌈2·√(b+c)⌉`, is two paired standard errors of the discordant rows. If 10–20% of rows are discordant between two cells (146,000–292,000), the floor is about 765–1,081 rows, under 0.08% of the panel. On the rows unseen in D3, at most 1,228,450 by the census, the same shares give 701–992. So the floor is unlikely to set the band. The expectation, not a measurement, is that the spread of `d` over the two seeds will be the limit. claude-152's width-256 seeds differed by 1,245 of DEV's 14,287 hits, 8.7%, but that was a spread of hits, not of a paired `d`. A cell is read only beyond the band, and a cell inside it is underpowered, never a negative.

## Cost

By claude-276's 0.1253 s a step for width 256 at 192 × 16:
- **256:** about 17 minutes of steps a run;
- **512:** about 34, if it is half the speed (claude-153; the item: "about half the speed");
- **1,024:** about 68, if it is a quarter, which no run has measured. I-2 measures it.

Training is then about 255 minutes on pod A, 255 on pod B, and 204 plus 85 for the contrast cell on pod C.

Each pod adds about:
- 10 minutes to fetch and build, and 10 for I-0;
- 10 for the smokes;
- 20–60 to score its checkpoints (6, or 8 on pod C);
- 15 for the snapshots and parity, on pods A and C;
- 15 to seal and read back.

That is about 5.5 to 6.5 hours a pod, 17 to 20 GPU-hours in all. At claude-274's $0.975 an hour, the run is about $16–19, at most $2.93 an hour across the three pods. A seed 3 for R1 and R2 would add about 2.5 GPU-hours.

`uor-pod status` is read at launch, inside 4 pods and $8 an hour for all labs. If fewer than three pods fit, the same cells run on two pods, one after another. Each watcher's wall limit is 9 hours and its spend limit $12 a pod. Each lease is taken for 8 hours and renewed. A budget or time stop wakes the session to extend a healthy run, never to cancel it.

## Stop rule

- **A width that runs out of memory at batch 16** in I-1 trains at `--batch 8 --steps 16384`, the same positions, at every data size and seed of that width. The REPORT records it as a difference between widths.
- **A width the trainer refuses** in I-1 (an error building the model or its reads, not a memory error) is an instrument item. The shape checks cited under Arms admit all three widths. If 1,024 refuses anyway, it is replaced by 768 with `--hamming-read 12`. That keeps the shape: 12 Hamming heads of 64, an even split between copy and type heads, and 4 stack heads of 192. (960 would need 15 heads, an odd split.) A refused 512 is an instrument fault in a width claude-153 ran, and stops the run for repair. The REPORT records it as a new configuration, and R1 reads that width against 256.
- **A run that diverges** (a non-finite loss) is absent. A reading that needs it is unread at that seed. If R1 or R2 is unread at a seed, the Bar reads it as mixed.
- **A failed instrument check** stops the reading, not the run. The run's outputs are sealed, the instrument is repaired, and the readings are taken again from the sealed files where the fault allows. Otherwise the run is repeated at the repaired pin.
- **An export or parity failure** (R4, R9) is reported as NOT_RUN at that width, with the failing step. It is an instrument item and never moves the Bar.
- **A pod that fails to start** is replaced by one more, once, inside the caps.
