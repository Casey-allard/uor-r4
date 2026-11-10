# Pre-registration: a turn memory that sees the end of the question (arm rt)

- Lab: claude. Unit: one-model-end-to-end, claude-345. Date: 10 Nov 2026.
- Owner's go-ahead, 10 Nov, in the lab's chat: "Yes, run it" to the mixed-key arm, whose sheet names this as the next unit in rows 1 and 2 and keeps arm r as the base in row 3. No bar, scope or stop rule changes here, so this runs on the lab's own pre-run review (§5.1).
- Pin: `05e50006b9b1dcd1799166df478e08139c0a9e48` on `claude/talk`.
- Earlier sheets, whose data, panels, reader and checks this one reuses: `docs/harness/prep/one-model-end-to-end-reply-line-2026-11-10.md` and `docs/harness/prep/one-model-end-to-end-mixed-keys-2026-11-10.md`.

TEST FITNESS: FIT (10 Nov 2026, a fresh-context non-author reviewer, before any pod; the record is in the FIT section below).

## Question

Arm r answers recall questions from the memory line and gets general questions wrong: on the fixed script, recall 4 of 4 and general questions 2 of 10. Its turn memory keys a reply on the turn's first 8 tokens, which end before the word a general question turns on. Does a reply table that also keys on the end of the turn make general replies right, without losing the recall?

## What was seen before this sheet

- **Arm rk (keys 1,2), run and read 10 Nov: row 3 of its sheet.** Panel E 0.406 (162 of 399; arm r 303), V 0.434, N 134 of 579 (arm r 188). Laptop: L 0 of 4, C4 recall 0 of 4, general 1 of 10. 64 of 72 wrong "Your/You" recall replies are wrong at the reply's first token (arm r: none there). With the turn memory off the same weights are exact on 289 of 399. Cause: the both-heads rule left 158 opening keys to the copy read (arm r 6,040), so the memory's stored opening is written over the line. Arm r stays the base.
- **Why general replies are wrong in every arm.** `User:` is 3 tokens and a number is a space token and a digit, so the first 8 tokens of "What is the opposite of tall?" and "What is 2 plus 3?" end before "tall" and before the second operand. The memory cannot see what the answer depends on, and the answer is not a copy.
- **The count on arm r's own data (laptop, CPU, `train-in-space --reply-bound 1 --reply-chain-tail 9`).** The reply table built from TRAIN at share 0.5 and asked on the line panel's 37,133 rows past their openings:

| | Arm r's table (first 8) | Three keys, tail 9 |
|---|---|---|
| Keys at share 0.5 | 327,257 | 1,591,205 |
| Left to the copy read | 187,254 | 944,018 |
| Removed by the yield rule | 0 | 16,254 |
| Keys kept | 140,003 | 630,933 |
| TRAIN rows named right, of 2,244,879 | 1,345,704 | 1,439,251 |
| Panel rows the copy read owns (12,923): named by the table | 911 | 925 |
| Other panel rows (24,210): named right | 19,856 | 21,387 |

  The first column equals the pod's own log for arm r (140,003 keys). Rows not named right fall from 4,354 to 2,823. The rows where the table would write over a copy stay flat.

## Base artifact: the arm

Arm r's recipe at seed 2 with one option added: `--turn-memory-tail 9`. Copy keys stay `[1]`.

The reply table then holds three key forms, and a read asks them in this order and takes the first one stored. With T the turn's ids at the prompt's last token, L the reply's length so far and x its last id:

1. **ends**: T's first 8 ids, then the last 8 of the rest, then `PAD, L, PAD, x`.
2. **tail**: `PAD`, T's last 9 ids, then `PAD, L, PAD, x`. The role line and "?" take 7 tokens, so 9 reaches the question's last two.
3. **head**: T's first 8 ids, then `PAD, L, PAD, x`. This is arm r's key.

A key the copy read owns is left out as in arm r. A wider key that a row would reach past a key the copy read owns is removed too (the yield rule), so a wider key never writes over a copied value. The opening table is unchanged. The served artifact carries the tail in a `memory_chain` record under `uor-r4.lut-stack/9` and asks the same three keys in the same order; an artifact without the record is read as before.

TRAIN, panels, steps (16,384), batch, width, context and seed are arm r's. No new data.

## Steps

As for arm r: the GPU tests of the rule, a 64-step smoke on the same inputs, then 16,384 training steps from a fresh start. Here the smoke also exports its own checkpoint (I-r).

## Data split

The reply-line sheet's, unchanged: its TRAIN with the line as the reply, its panel with the line and the panel without, by the hashes in that sheet's table. No new data.

## Readings

As the mixed-key sheet defines them, on the same two panels with the reader `recall-pairs`: **E, D, G, V, N**, the **kinds** (sums, colors, opposites, legs; arm r 0 of 13, 0 of 8, 3 of 8, 6 of 16), **L** (`conv-2`), **C3** (`conv-3`), **C4** (`conv-4`, with the same rule for a right general reply). Arm r, seed 2: E 0.759, D 0.759, G 0.697, V 0.767, N 188 of 579 (0.325), L 4 of 4, C3 5 of 7, C4 general 2 of 10 and recall 4 of 4.

Two additions:

- **The same readings with the turn memory off** (the reader's `-no-memory` samples), beside the ones with it on, so what the memory adds is shown against its own control (L-010).
- **The laptop readings use the chat program and exporter built at the pin.** The registered `bd2532281` binaries do not read a memory chain. Check I-b shows that the new binaries serve arm r's artifact exactly as the registered ones.

Every reply of L, C3 and C4 is shown word for word in the REPORT beside arm r's.

## Bars: the rows, fixed before the run

Recall is **kept** when E ≥ 0.60, V ≥ 0.60, L ≥ 3 and C4's recall ≥ 3 of 4.

| Row | Reading | What it unlocks |
|---|---|---|
| 0 | An instrument check fails | Repair the instrument and rerun; nothing is read |
| 1 | Recall kept, and C4's general questions ≥ 5 of 10 or N ≥ 0.42 | The three-key memory becomes the base recipe; seeds 1 and 3; then questions and statements in wording TRAIN does not hold |
| 2 | Recall kept, C4's general questions < 5 of 10 and N < 0.42 | The table stays unless a reading fell. The REPORT counts, for each wrong general reply, whether the table named the right token at the first wrong position (a key miss) or named it and the model said another (a read miss); the next change follows that count |
| 3 | Recall not kept | Arm r stays the base. The REPORT says which reading fell and where in the reply, and checks the yield rule on those rows first |

A negative here closes this configuration (tail 9, share 0.5, this order), not a memory keyed on the end of the turn (§4.1).

**Prediction, written before the run:** row 1. Recall should hold within 0.05 of arm r's E, because the table names almost the same copied rows (911 to 925 of 12,923). N should rise above 0.42 and sums and colors above 0, because the table now names 1,531 more panel rows right and those are the rows a general answer sits on. Row 2 is possible: the count is under teacher forcing, a greedy reply compounds an early miss, and a table that names the token does not make the model say it. Row 3 would be a surprise and would point at the yield rule or at the serving side.

## Instrument checks

The earlier sheets' I-p, I-3, I-s, I-t, I-x and Seal, at this pin, and:

- **I-g:** `cargo test --release -p uor-r4-float-train --features cuda --lib -- latest copy_owned copy_names chain_table yields greedy_reply_key reply_tail` passes on the pod before the smoke with at least 13 tests (the mixed-key arm's 8, three of them GPU-only, and five new; the laptop runs 10), and the five new ones each show `ok` by name: `a_chain_table_holds_three_forms_and_the_read_asks_them_in_order`, `a_wider_key_yields_where_the_copy_read_owns_the_exact_key`, `the_greedy_reply_key_is_the_rows_key_under_a_tail`, `the_greedy_reply_key_reads_the_end_of_a_long_turn`, `a_reply_tail_is_saved_with_the_table_and_a_chained_key_is_asked_in_order`. A failed test, a missing name or a test build that does not compile is row 0: the job stops and nothing is trained.
- **I-c:** the run's `config.json` has `turn_memory_tail: 9` and otherwise equals arm r's seed-2 config field by field. The job checks three fields; the full comparison is a `jq -S` diff on the laptop against `local/claude/claude-345r/runs/r/oh/run/snap/config.json`, after the run and before any reading.
- **I-k:** the smoke prints the `[memory] reply key chain: tail 9` line and both "left to the copy read" lines. A missing chain line, or a reply table with no more keys than arm r's 140,003, is row 0 and the job stops before the long run. The laptop's counts are the expectation: 944,018 of 1,591,205 reply keys left, 16,254 yielded, 630,933 kept, 6,040 of 13,454 opening keys left. The log's `reply table: … keys at share 0.5 N` line prints the kept count, so N is expected to be 630,933, not 1,591,205 (arm r's log printed 140,003 there). A count that differs is traced to its cause before any reading is read: a table built another way than this sheet's (tail, share, order or yield rule), or a difference with no cause found, is row 0; the same construction with a different count is reported and the readings stand.
- **I-r, on the pod, in the smoke:** the path a chain table takes to the laptop runs once before the long run is spent: the smoke's checkpoint through `fbs-snapshot`, and that snapshot through `fbs-export --run-config` built at the pin, which writes the `memory_chain` record and reads its own keys back at tail 9. A refused snapshot or a refused export is row 0 and the job stops. An exporter that does not build on the pod is flagged and the run goes on, since the laptop exports the artifact. This is a check of the tools, not a reading. It runs on the pod because the trainer does not train without the GPU, so no chain table can be made on the laptop.
- **I-b, before launch, on the laptop. Done 10 Nov at the pin:** the exporter reproduces arm r's seed-2 artifact byte for byte (sha256 `707dfdc3b564df95497d57303decccebe42ac382045d7a0aae461137fec18ad0`), and the chat program gives the same output as `uor-chat-ad81e080b` on `conv-2`, `conv-3` and `conv-4` with that artifact, the speed figures on the `[served]` lines aside.
- **I-m:** the run's exported artifact has schema `uor-r4.lut-stack/9` and `memory_chain.tail` 9. Its size is reported (about 56 MB expected, against arm r's 14.9 MB: the table is 4.5 times larger).

No check compares a chain artifact's replies with the float model's token for token. If the laptop readings fall far below the panel's, that gap is classified as instrument or model before the result is called (§4.2).

## Minimum test power

One seed and one arm: a first reading. No milestone claim rests on it. C4 has ten general questions and is the product reading, shown word for word; N rests on 579 replies and E on 399 matched pairs.

## Cost, compute and stop rule

One pod at no more than $1.60/h. Arm r took about 50 minutes; the larger table adds to the table build and to each lookup, so about 70 minutes and under $2 is expected; ceiling 3 hours, $4.80. One of the four pods. The watcher is armed before anything is sent (L-004), the run is written into a new root `results/claude/one-model-end-to-end-memory-chain-2026-11-10/run/rt-s2/<try>/` (L-002), and the pod comes down by itself when its four checks pass. A healthy run is not stopped for time. The smoke's 64 steps give the step rate before the long run is spent.

## FIT

**FIT**, 10 Nov 2026, [claude] fresh-context non-author reviewer (a read-only subagent that saw the sheet, the six kit scripts beside the mixed-key arm's, the code at the pin and arm r's config and log, and none of the reasoning behind them). It confirmed: the job trains exactly this arm at this pin and refuses any other; `--turn-memory-tail` is parsed and validates with the recipe; every pattern the job applies to the log and to `config.json` matches the format string or field at the pin, and each required field is written for this arm; the test filter selects the five named tests and at least 13 on the GPU (10 without it); the exporter's flags and the header's `memory_chain` key and `/9` schema; the six scripts name only this kit, this pin and the new store root, and the watcher starts before anything ships; the rows are exhaustive and exclusive. Its three notes, and what was done:

1. The sheet's table and the log use "keys at share" for different counts. Changed before launch: I-k says the log prints the kept count.
2. I-c named no step for the full config comparison, and I-k left an unexplained count open. Changed before launch: I-c names the laptop diff; I-k says which differences are row 0.
3. If the smoke export is refused, the smoke's snapshot and artifact are sealed with the failed run. Left: failure path only, and the files would help the diagnosis.

Taken on trust by the reviewer: the test count (read from source, not run on a GPU) and that the scorer takes the tail from the checkpoint. Checked since: the artifact header is 8,777 bytes, inside the first 1 MiB the job searches.

**Found while building the kit, before the FIT:** the copied job required `hamming_copy_keys` in `config.json`, a field written only when that option is passed. It would have ended the job with exit 8 after training. The line was removed after reading arm r's own config.

**Two fresh code reviews of the three-key table** (trainer side and serving side, 10 Nov, each given the spec and the diff only): both approve with low items. Fixed at the pin: `grade` asked a chained key of a first-8 table; a literal-key test on a 19-id turn, with two negative gates that fail as they should; the tail's range is refused early; a chain stack keeps its schema under a point key; the exporter refuses a tail that is not a whole number. Not fixed: no test for a chain across a second turn in one session, and none for a chain stack with a point key.

**Pre-run review**, 10 Nov 2026, [claude] top session (§5.1, §5.10). The chain: arm r is the base on three seeds; the mixed-key arm closed one configuration and left the memory key as the diagnosed cause; the laptop count says the three-key table names 1,531 more panel rows right and writes over no more copies. Tools and inputs: the pin is on `origin/claude/talk`; TRAIN and both panels are in the store and listed today; the run root does not exist yet; the trainer suite (352) and the serving stack suite (220) pass at the pin; I-b is done. What each row unlocks is in the table above. The FIT is recorded. Lease: one pod is up across all labs at $0.56/h, so this one makes two pods and at most $2.16/h, inside 4 pods and $8/h. The run goes ahead; Casey can reverse it.

**Shape, added 10 Nov after launch and before any reading.** The harness's sheet rule (`prep.sheet_shape`) failed on this sheet in CI on PR #3911: it wants headings named for the bar, the base artifact, the data split and the steps, and a verdict line. The headings "Base artifact: the arm", "Steps", "Data split" and "Bars: the rows, fixed before the run" and the `TEST FITNESS` line were added for it. They restate what the sheet already said. No row, bar, reading or check is changed; the sheet as launched is at commit `4ce202fd2` in the file's history.

**Launch, 10 Nov.** try-1 (`vast-60106351`, South Korea, $0.60/h) answered on its SSH port and refused the key for the tool's whole 900 s wait: the launcher was given the wait and network settings under the wrong variable names, so the tool used its defaults. try-2 (`vast-60107071`, Alberta, $1.04/h) was started with the right names while try-1 waited, reached SSH in 34 s and started the job at 18:33:45Z. Written before any result: try-2 is the run of record unless try-1 starts its job.
