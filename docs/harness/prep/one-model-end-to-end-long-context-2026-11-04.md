# Prep sheet: the one model trained at the 2,048-token window, recall across turns (one-model-end-to-end, P2b, claude-275, 4 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Casey's 4 November plan, P2b: train at 2,048 × 4 on the long corpus, score recall across turns against a no-memory control (L-010), and time a step at 2,048 against 192.

TEST FITNESS: FIT - fresh non-author adversarial review, round 1, of this sheet at `3d466b3a9`. It found no blocker or major. Its four minors are fixed in this text; no bar moved:
- R4's decode tok/s is read from the `decode` pair `--score-rows` prints, not from the samples file;
- (S)'s positions are 67,107,840, with its evals at about (L)'s;
- I-6, which could not fail as written, now checks that the passes pair query by query and that each ran at its own window;
- R1 reports the under-192 bin at both windows, as a check on scoring at 192.

PRE-RUN: GO WITH FIXES, by a fresh non-author review in the run's unit, claude-276, right before launch ([#3407](https://github.com/UOR-Foundation/uor-r4/issues/3407#issuecomment-6627609246)). Its fixes were in the laptop grade and were made before launch; no bar moved. The pin is claude-275's merge commit, `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`.

OUTCOME: **Negative** at both seeds ([claude-276](../../loop/tasks/one-model-end-to-end/claude-276/REPORT.md)). R1's b + c is 0 at seed 1 and 1 at seed 2, under 10, overall and on the far bin; I-0 to I-6 hold. As the Bar says, this closes this configuration only, and its classification, instrument or mechanism, is the next unit on this line. That unit is past P2b's bound, so it is parked behind P3 and comes before P4 (claude-276's Next).

**What this run measures.** Up to now every model on this line was trained and scored with a 192-token window ([claude-273](../../loop/tasks/one-model-end-to-end/claude-273/REPORT.md), [claude-274](../../loop/tasks/one-model-end-to-end/claude-274/REPORT.md)). A note stated more than 192 tokens before it is asked for is out of view. This run trains the one model at 2,048 on conversations that ask for notes up to about 2,000 tokens later. It then asks whether the reply states the note's value, with the window open and closed. The object is the float model, trained as served, scored by the float scorer. No integer export or served claim rests on this run.

## Question

- Does the model trained at the 2,048 window state a held-out note's value more often when it is scored with the 2,048 window than when the same model is scored with 192, above all for notes stated 512 or more tokens earlier?
- What does the turn memory add to that, against the same model with the memory out (L-010)?
- What does the window cost: decode tok/s at 2,048 against 192, and training positions a second?

## Base artifact

A training comparison from scratch; the candidates are the checkpoints each arm produces.
- **The pin** is claude-275's merge commit on `main`, recorded in the run's unit at launch. Training, smoke, I-0 and scoring use `train-in-space` built `--locked --features cuda` at the pin. Each pod fetches the pin, checks it is that commit and clean, builds, and records the binary's SHA-256.
- **The recipe** is claude-266's (b): `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --train-as-served 1 --width 256 --hamming-read 4`, with `samples` and `reply_tokens` left at their defaults.
- **The outputs** go under `results/claude/one-model-end-to-end-long-context-2026-11-04/sN/`.

## Arms

Both arms train on the long TRAIN (Data split) through `--train-split TRAIN --train-sha256 HEX --train-mask-sha256 HEX`, for equal positions:
- **(L)** `--context 2048 --batch 4 --steps 8192 --eval-every 1024`: 67,108,864 positions.
- **(S)** `--context 192 --batch 16 --steps 21845 --eval-every 2731`: 67,107,840 positions, with its evals at about (L)'s positions (8,389,632 against 8,388,608). Reported only (R3).

Seeds 1 and 2, `--seed S`, one 1 × RTX 5090 pod per seed. On each pod, (L) then (S) run alone on GPU 0 at every stage.

**Scoring**, each pass `--score-rows CKPT --out DIR --samples 16 --reply-tokens 32 --retain-rates 1 --seed S --panel p-long=PLONG --panel-queries p-long=PLONG/queries.json`:
- **(L) at 2,048:** `--context 2048`;
- **(L) at 192 and (S) at 192:** `--context 192 --panel p-v=PV`.

Each pass decodes every p-long query with the memory on and out, writing `samples-p-long-queries-{memory,no-memory}.json` and `recall-p-long-{memory,no-memory}.json`. At 2,048 and at 192 every query is decoded: its reply is at most 21 tokens.

## Readings

- **R1 (deciding): the window.** (L) at 2,048 against (L) at 192, memory on, paired by query. From the two `recall-p-long-memory.json` files, join `outcomes` on `query` over the queries `decoded` in both, less the census's 6 (queries 0, 42, 93, 401, 471 and 487). b counts those `stated` at 2,048 and not at 192, c the reverse. R1 is read overall (603 queries) and on the far bin, `distance` ≥ 512 (494). It passes where b − c > 2√(b + c) and b + c ≥ 10. Beside it, reported only: the under-192 bin's recall at each window (23 queries, whose note both windows see), so a scorer broken at 192 cannot pass as "window shut".
- **R2: the memory (L-010).** (L) at 2,048, memory on against out, paired the same way. Reported, with R2b, the same at 192.
- **R3: training at 192.** (S) at 192 against (L) at 192, memory on, paired the same way. Reported only.
- **R4: the cost, led by tok/s.** Decode tok/s for each queries file, from the `decode` pair (tokens, seconds) on the line `--score-rows` prints for it, at 2,048 against 192 on (L). Each pass's printed lines are kept as `scored.jsonl`. Training positions a second from each arm's trainer clock, against claude-273's 0.6026 s and 0.1349 s a step.
- **R5: short context.** DEV and p-v rows and samples at 192 for (L) and (S), set beside each other. Reported only.
- **By kind:** every reading above is also split by answer kind, invented word or number. The census (Data split) counts how many of each are found in TRAIN's replies.

## Bar

Read per seed, then across both:
- **Positive:** R1 passes overall and on the far bin, at both seeds. The model trained at 2,048 states notes from beyond 512 tokens that it does not state with the window shut. P4 (#3471) takes (L)'s recipe as its base.
- **Partial:** R1 passes overall at both seeds, and the far bin fails at either. The window carries recall only part of the way. P4 still goes ahead; the next unit on this line asks where the far recall stops.
- **Negative:** R1 fails overall at both seeds, b + c < 10 included. This closes this configuration only: the recipe at 8,192 steps of 2,048 × 4 on this TRAIN (§4.1). It never closes the window or the read (§4.2). The next unit classifies it first: instrument or mechanism.
- **Inconclusive:** anything else, the seeds disagreeing included. Nothing is named or closed.

Whatever the outcome:
- R2 to R5 are reported with their numbers.
- No reading here is a served result, a step-4 claim or a milestone claim.
- An instrument check that fails makes every reading unread, and the run is repaired, not graded (§4.1).

**Reversal.** Casey can say "hold": no further pod launches, and nothing is named or closed. The checkpoints and score files stay sealed under the results root.

## Instrument checks

- **I-0:** the CUDA library suite at the pin on each pod, `cargo test --locked --features cuda -p uor-r4-float-train --lib`. It passes only if at least one test ran, none failed, and each of these is `ok`, matched as the last segment of `test <module path>::<name> ... ok`:
  - claude-274's `the_hit_rate_reads_the_window_the_decoder_sees`, `the_reply_table_rows_follow_the_forward_s_window`, `a_scoring_forward_holds_sixteen_windows_of_192_positions` and `exact_length_batches_cap_the_reach_at_the_model_s_window`;
  - claude-275's `the_row_stride_is_refused_where_windows_share_a_forward`, `a_query_decodes_from_the_window_that_ends_at_its_reply`, `the_recall_grade_counts_whole_word_answers_by_distance`, `the_answer_census_counts_numbers_and_words_apart` and `panel_queries_are_read_only_for_their_own_split`.
- **I-1 (the smoke):** each arm trains 64 steps on its own command with `--steps 64 --eval-every 64`, exits 0, and its loss is finite.
- **I-2 (the projection):** the smoke's step time projects each arm's training. If (L) projects past 3 hours, both arms are cut to equal positions: (L) at the steps that fit 3 hours, and (S) at 8/3 of that.
- **I-3 (the inputs):** on each pod, `shasum -a 256 -c SHA256SUMS` passes in each input folder, and every SHA-256 the Data split names equals its file's. `heldout-panel check-queries split=PLONG tokenizer=TOKENIZER` prints `checked 609 queries, 0 failed`.
- **I-4 (the grade):** in every `recall-p-long-*.json`, `total.reference_states` equals `total.decoded`. Every reference reply states its answer, so the grade can see one.
- **I-5 (the window):** at 2,048, `total.whole_document` equals `total.decoded`. Each conversation is at most 2,048 tokens, so every prompt holds its whole conversation.
- **I-6 (the pairing):** every `recall-p-long-*.json` has the same `outcomes[].query` and `outcomes[].distance` lists, 609 long, and its `context` is its pass's: 2,048 for (L) at 2,048, 192 for the other two. A pass run with the wrong window fails here.

## Data split

- **The long TRAIN:** `m-world-recall-corpus out=TRAIN tokenizer=TOKENIZER conversations=16000 seed=2752 chain=2048 context=2048 protocol=2 exclude=PLONG/user_turns.json`, written on the laptop in `Cell::same(Split::Train)`.
  - SHA-256s: `train/tokens.u16` `61019954ef82d7b33d2d5a101f436fece29452f8580f8f3268e35229ccd0cb3c`, `train/response_mask.u8` `29b34d2c9ed3a56bf638a3afd6863e5744b2c1d3f838a1034f19a77d1e02818d`, `train/manifest.json` `0a7f32d2aa54edf0169c3174473deccdb099f17487fe9140b08ef0b369cd8e65`.
  - It has 16,000 conversations of 1,703–2,048 tokens, 31,428,480 tokens and 9,988,607 reply tokens, and 0 metered mismatches.
  - It has 60,444 queries, 58,118 of them past 192 tokens, 49,748 past 512 and 36,037 past 1,024; at most 2,012.
  - Redraws: 91,962 contradicting, 32,869 farewell, 1,370 mentioning, 128,000 too long, 0 dropped. The exclusion read p-long's 4,691 user turns and rejected 0 draws: the panel's phrasings are DEV's.
  - Store, with a `SHA256SUMS` read back: `results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/train-long/`.
- **p-long, the held-out panel:** `heldout-panel cell=dev,dev conversations=160 seed=2751 protocol=2 chain=2048`, phrasings and values both held out. It has 313,980 tokens, 609 queries and 0 metered mismatches.
  - SHA-256s: tokens `8b2529b1a1385112ef41a42eca50dc982dc474f3b40e8948b8887b6b4158ab36`, mask `b17ff53fc37dfe2910b1ffca820538ca5473d1b0b707da3f06eb83c25c58fed0`, `queries.json` `cf5c30b96e98f7770314f96be5114cf5eeb21e3f9657aeb84b4c1d6168ca2390`, `user_turns.json` `4d21bc632073c6384e0193960c862ddbb326ecedf4ed27b6af30811319207800`.
  - Distances: 23 under 192, 86 at 192–511, 138 at 512–1,023, 362 at 1,024 and over; at most 1,994.
  - Answers: 290 numbers and 319 invented words, 511 distinct. Replies are 6–21 tokens.
  - Store, with a `SHA256SUMS` read back: `results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/p-long/`.
- **The census:** `train-in-space --census 1 --train-split TRAIN --train-sha256 HEX --train-mask-sha256 HEX --panel p-long=PLONG --panel-queries p-long=PLONG/queries.json`, on the laptop (23 s).
  - **Words:** 0 of the 319 invented-word answers appear in any TRAIN reply.
  - **Numbers:** 6 of the 290 number answers do, with values 10, 11 and 16: queries 0, 42, 93, 401, 471 and 487, at distances 919–1,863. TRAIN's replies say these small numbers elsewhere, so a reply could state one without reading its note.
  - **Classed instrument**, as the ORDER's stop rule says. The rule's fix, a new cell or an exclusion before this sheet, was not made; in its place this sheet pre-registers leaving the 6 out of every paired reading and reporting them apart (claude-275's REPORT, What did not work 1). The 603 left hold no answer TRAIN's replies say.
- **DEV, the tokenizer and p-v:** claude-261's pins, unchanged, checked against their `SHA256SUMS`.

No model output was read to set any panel, query or bar.

## Steps

1. On each pod, fetch the pin, build, record the binary's SHA-256, and run I-0.
2. Fetch the inputs and run I-3.
3. Run I-1's smoke for (L), then (S); project I-2.
4. Train (L), then (S).
5. Score (L) at 2,048, (L) at 192 and (S) at 192, in turn.
6. Seal every output with `SHA256SUMS` under the store root, read it back, and only then take the pod down.
7. On the laptop: check I-4 to I-6, compute R1 to R5 and apply the Bar.

## Minimum test power

603 paired queries a seed, 494 of them far. At 192 a far note is out of view, and the turn memory, built from TRAIN, holds no held-out value, so c on the far bin is expected near 0. Then R1 passes once (L) at 2,048 states about 10 far answers more than at 192, 2% of the far bin. That is the bar for "the window carries recall at all", not for a useful rate. R1 reports the rate itself beside its verdict.

## Cost

Each pod takes about:
- 10 minutes to fetch and build, and 10 for I-0;
- 8 minutes for the two smokes;
- 85–100 minutes for (L): 82 minutes of steps at claude-273's 0.6026 s, plus evals;
- 50–60 minutes for (S): 49 minutes at 0.1349 s, plus evals;
- 40–65 minutes for (L) at 2,048. Each query decodes at most 32 tokens at about 85 ms a token at a 1,960-token prompt (claude-274), so each memory setting takes at most 28 minutes, plus the rows;
- 15–25 minutes for each pass at 192 (claude-274: 54 tok/s at about 180 tokens);
- 15 minutes to seal and read back.

That is about 4.2 to 5.4 hours a pod. At claude-274's $0.975 an hour, two pods cost about $9–12, at most $2.20 an hour together. `uor-pod status` is read at launch, inside 4 pods and $8 an hour. The watcher's wall limit is 7 hours (`MAXSEC=25200`) and its spend limit $16. Each lease is taken for 6 hours and renewed. A budget or time stop wakes the session to extend a healthy run, never to cancel it.

## Stop rule

- **An arm that diverges** (a non-finite loss) or runs out of memory alone on its card is absent at that seed. If (L) is absent at a seed, R1 there is unread, and the Bar reads only the seed that has it, as inconclusive.
- **A failed instrument check** stops the reading, not the run. The run's outputs are sealed, the instrument is repaired, and the readings are taken again from the sealed files where the fault allows. Otherwise the run is repeated at the repaired pin.
- **A pod that fails to start** is replaced by one more, once, inside the caps.
