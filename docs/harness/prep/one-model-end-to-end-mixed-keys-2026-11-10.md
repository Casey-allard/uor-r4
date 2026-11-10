# Pre-registration: one copy head on a two-token key (arm rk)

- Lab: claude. Unit: one-model-end-to-end, claude-345. Date: 10 Nov 2026.
- Owner's go-ahead, 10 Nov, in the lab's chat: "Yes, run it" to "Run a mixed one- and two-token copy key arm after the seeds?"
- Pin: `312880e2bd81da2f0d0f7148c7251c4f9a7cb30e` on `claude/talk`.
- Earlier sheet, whose data, panels, reader and checks this one reuses: `docs/harness/prep/one-model-end-to-end-reply-line-2026-11-10.md`.

TEST FITNESS: FIT (10 Nov 2026, a fresh-context non-author reviewer, before any pod; the record is in the FIT section below).

## Question

Arm r (seed 2) answers recall questions from the memory line: 303 of 399 matched pairs exact on the panel, and on the laptop it says the line word for word. Its replies to general questions are wrong. Does putting one of the two copy heads on a two-token key reduce the wrong copies in general replies, without losing the recall?

## What was seen before this sheet (laptop, no GPU)

- **The fixed script** `local/talk/conv-4.txt` (sha256 `33d19426dc0077e300af4a4e093747c484d27da45c4cf05a3da61f46ea8989e4`): four facts, ten general questions in TRAIN's wording, four recall questions. Arm r, seed 2, as exported: recall 4 of 4, facts acknowledged 3 of 4, general questions 2 of 10. The same with a hand-written line on the recall turns only: 2 of 10. Wrong replies: "The opposite of hot is the opposite of hot is…", "2 plus 2 is 2 is 2 is", "The opposite of big is my friend's Maya.", "4 plus 4 is 1."
- **The panel by kind of reply** (float, line panel, exact replies): sums 0 of 13, colors 0 of 8, opposites 3 of 8, legs 6 of 16. Arm m, before the latest-match rule: 0 of 13, 0 of 8, 3 of 8, 2 of 15. The fault is older than arm r.
- **Why the one-token key copies wrongly here.** The tokenizer writes a number as a space token and a digit. In "4 plus ␣" the one-token key is the bare space, and its latest earlier copy is the reply's own first operand, so the model says "4 plus 4". The two tokens `us`, `␣` occur once before, in the question, followed by the right operand. After " is" the one-token key finds the question's own " is" or an earlier turn's, and the reply follows that span.
- **Arm r's weights served with keys 1,2** (snapshot re-exported with `copy_keys: [1, 2]`, nothing retrained): one of eight wrong general replies changed to the model's own word ("The opposite of big is bigger."); two of four recall replies in the hand-line script lost their first token. These weights were never trained with the key, so this is a reason to train, not a result.
- **What a copy key cannot fix.** The turn memory's key is the turn's first 8 tokens. `User:` takes three and a digit two, so the key ends before the second operand of a sum and before the word of "What is the opposite of tall?". The answer token of those replies is not a copy and the memory cannot see what it depends on. That is the next unit's change, not this arm's.

## Base artifact: the arm

Arm r's recipe at seed 2 with one option added: `--hamming-copy-keys 1,2`. Copy head 0 keeps the one-token key, copy head 1 takes the two tokens before a position. Both read under the latest-exact rule (`--hamming-copy-latest 1`).

With the option, `--turn-memory-leave-copied 1` leaves a key to the copy read only where both heads' latest exact matches name the target (commit `312880e2b`); with one-token keys it is arm r's rule unchanged. TRAIN, panels, steps (16,384), batch, width, context and seed are arm r's. No new data.

## Steps

As for arm r: the GPU tests of the rule, a 64-step smoke on the same inputs, then 16,384 training steps from a fresh start.

## Data split

The reply-line sheet's, unchanged: its TRAIN with the line as the reply, its panel with the line and the panel without, by the hashes in that sheet's table. No new data.

## Readings

All on the same two panels with the reader `recall-pairs`, and on the laptop with the chat program built at `bd2532281`, the artifact exported with the run's config.

- **E, D, G, V**: as the earlier sheet defines them. Arm r, seed 2: E 0.759, D 0.759, G 0.697, V 0.767.
- **N**: exact replies among the line panel's non-recall replies (`line_exact_all` minus the exact recall replies). Arm r: 188 of 579 (0.325).
- **Kinds**: the count of exact replies for sums, colors, opposites and legs on the line panel, shown beside arm r's 0, 0, 3 and 6. A description, not a bar.
- **L**: `local/talk/conv-2.txt`, four questions. Arm r: 4 of 4.
- **C3**: `local/talk/conv-3.txt` with the sieve, seven questions. Arm r: 5 of 7.
- **C4**: `local/talk/conv-4.txt` with the sieve: general questions right of 10, recall right of 4, facts acknowledged of 4. A general reply is right only when, letter case aside, it is one of TRAIN's own replies to that question with the right answer and nothing after it: `The opposite of hot is cold.`, `2 plus 3 is 5.`, `A lemon is yellow.` or `It's yellow.`, `The opposite of big is small.`, `A spider has eight legs.` or `It has eight legs.`, `The opposite of fast is slow.`, `4 plus 4 is 8.`, `Milk is white.` or `It's white.`, `The opposite of hard is soft.`, `No.` By this rule arm r's seed 2 has 2 of 10 and its seed 1 has 3 of 10. Every reply is shown word for word in the REPORT beside arm r's.

## Bars: the rows, fixed before the run

Recall is **kept** when E ≥ 0.60, V ≥ 0.60, L ≥ 3 and C4's recall ≥ 3 of 4.

| Row | Reading | What it unlocks |
|---|---|---|
| 0 | An instrument check fails | Repair the instrument and rerun; nothing is read |
| 1 | Recall kept, and C4's general questions ≥ 5 of 10 or N ≥ 0.42 | Keys 1,2 become the base recipe; seeds 1 and 3; then the memory key that sees the end of the question |
| 2 | Recall kept, C4's general questions < 5 of 10 and N < 0.42 | This configuration closes. Arm r's keys stay the base, and the next unit is the memory key that sees the end of the question |
| 3 | Recall not kept | Arm r stays the base. The REPORT says which reading fell and where in the reply (first token or later), before any other key is tried |

A negative here closes this configuration, not two-token keys (§4.1).

**Prediction, written before the run:** row 2 is the likeliest. The second operand of a sum should come out right when the question says "plus", and remembered values should leave fewer general replies; the sums, colors and opposites themselves should stay wrong, because their answer is not a copy. Row 3 is possible: on a turn whose line says the value is absent, the memory now keeps the commoner opening.

## Instrument checks

The earlier sheet's I-p, I-3, I-k, I-s, I-t, I-x and Seal, at this pin, and:

- **I-g:** `cargo test --release -p uor-r4-float-train --features cuda --lib -- latest copy_owned copy_names` passes on the pod before the smoke, with at least 8 tests: arm r's 6, the new GPU test `the_latest_rule_under_copy_key_two_needs_both_tokens` and the host test `copy_names_by_asks_every_head_by_its_own_key`. A failed test, fewer than 8 passed, or a test build that does not compile is row 0: the job stops before the smoke and nothing is trained.
- **I-c:** the run's `config.json` has `hamming_copy_keys: [1, 2]` and otherwise equals arm r's seed-2 config field by field.
- **I-k:** both "left to the copy read" lines are printed. The counts are expected below arm r's 6,040 opening keys and 187,254 reply keys, since both heads must now agree; zero reply keys left is row 0.
- **I-h:** the exported artifact's Hamming record holds `copy_keys` `[1, 2]` and `copy_latest`.

## Minimum test power

One seed and one arm: a first reading. No milestone claim rests on it. C4 has ten general questions and is the product reading, shown word for word; N rests on 579 replies.

## Cost, compute and stop rule

One pod at no more than $1.60/h, about 60 minutes with the test build, under $2; ceiling 3 hours, $4.80. One of the four pods. The watcher is armed before anything is sent (L-004), the run is written into a new root `results/claude/one-model-end-to-end-mixed-keys-2026-11-10/run/rk-s2/<try>/` (L-002), and the pod comes down by itself when its four checks pass. A healthy run is not stopped for time.

## FIT

**FIT**, 10 Nov 2026, [claude] fresh-context non-author reviewer (a read-only subagent that saw the sheet, the two commits and the job script, and none of the reasoning behind them). It confirmed: the job trains exactly this arm at this pin and refuses any other; the rule-test filter matches exactly 8 tests at the pin; `copy_names_by` agrees with the GPU test's window and leaves the one-token case unchanged; the rows are exhaustive and exclusive. Its four notes, and what was done:

1. A test build that does not compile was flagged and the run went on. Changed before launch: the job now stops, and the sheet says so (I-g).
2. C4's "holds the answer word" could be argued after the run. Changed before launch: a general reply is right only as one of TRAIN's own replies (C4).
3. `copy_names_by` treats any key but 2 as one-token. Left: the trainer accepts only 1 and 2.
4. The job accepts seeds 1 to 3. Left: this sheet runs seed 2, and row 1 names seeds 1 and 3 as its follow-up.

Taken on trust, not checked by the run itself: I-c's field-by-field comparison and the laptop readings are done by hand after the run; the pod coming down is the watcher's work.

**Shape, added 10 Nov after the run was read.** The harness's sheet rule (`prep.sheet_shape`) failed on this sheet in CI on PR #3911: it wants headings named for the bar, the base artifact, the data split and the steps, and a verdict line. The headings "Base artifact: the arm", "Steps", "Data split" and "Bars: the rows, fixed before the run" and the `TEST FITNESS` line were added for it. They restate what the sheet already said. No row, bar, reading or check is changed; the sheet as it stood at launch is in this file's history on the branch.
