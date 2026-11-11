# REPORT: one-model-end-to-end, claude-348

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3925.

Two serving changes an outside review asked for: do not choose a token while reading the prompt, and search the table for the nearest token without reading all of it. Laptop only. No pod, no training, no bar, no mechanism arm. Every claim below is [claude]'s.

| Step of the ORDER | State |
|---|---|
| 1. The scan's share of a served step | measured: 65% to 67% |
| 2. A reading step that skips the scan | built; replies identical on every proof |
| 3. The exact cell search | counted, then built as an early stop, because the count showed a saving |
| 4. `uor-chat --prompt-ids` and the reply mark | decided: left as it is |

## What did not work, first

- **The pruned search the ORDER pictured does not exist on this table.** An exact scan that stops a row early still reads 65%, 58% and 50% of the table's values on real queries (seeds 1, 2, 3), and nearly all of them on its worst query (99.7%, 99.6%, 99.4%). The best distance is a large part of a typical row's distance, so a row's partial sum passes the bound late. The early stop is kept because it is exact and measurably faster, but it saves a third to a half of the scan's reads and 18% to 25% of a reply step's time, not most of either.
- **The timing tool's own decode figure flatters the early stop at two seeds.** `fbs-toks` decodes 64 tokens after a fixed prompt. At seeds 1 and 3 its reply is a few ids and then the end-of-reply id repeated (59 and 62 of 65 ids), a state where the nearest row is very near and the stop comes early. Seed 2's reply holds no end-of-reply id. The like-for-like figure is the side-by-side run on the 1,141 real prompts (Measurements 4).
- **A full choosing step on the tool's 128-token synthetic prompt is no faster.** Those queries are near no row, the scan reads almost everything, and the time is the same within noise.
- **Timings were taken on a loaded laptop** (load average 7 to 18 from other work). Medians of 20 runs moved by a third between runs; the fastest run of each set is given beside the median. Columns that come from different moments are marked.
- **The first transcript comparison for arm r read 6 of 6 different. That was the instrument.** claude-347 ran arm r with arm r's own pinned chat program, whose rule writes a memory line before a general question; `main`'s chat does not. Against a chat program with `main`'s rule (`uor-chat-ad81e080b`) the same six are identical.
- **`uor-r4-stack cost` cannot time an FBS model** ("phase timing serves dense stacks"), so the step audit's counts of symbols and instructions were not printed. The audit itself ran on the final program and passed: the command went on past it to that refusal.
- **Local format and lint are not usable as whole gates here.** `cargo fmt --check -p uor-r4-integer` flags two places in `fbs-toks.rs` this unit did not touch (lines 2251 and 2317); local clippy stops on existing code. CI's job at the head is the check.
- **The first form of one new test did not reach its guard.** On a model with a turn memory an older reset fires first, so the test now uses the dense chat fixture, and its negative gate fails as it should.
- **No test drives the `--read` flag of `fbs-toks` itself.** Its own `replies_match` check is true at three seeds.

## What changed

- **`IntegerStackSession::read_step`** (`crates/uor-r4-integer/src/stack/session.rs`). It steps a token as `step` does. Under the nearest head it skips the whole-table scan and names no token (`last_native_token` is `None`); the query, the memories and the position are what a choosing step leaves. Under the writer head and on a dense stack it is a full step.
- **`StackChat::feed`** (`stack/chat.rs`). Every unseen history token but the last is read; the last is served with a choice, as before. The reply loop is unchanged. A feed that stopped on an error has read its last token without a choice, so the next feed of that history steps it again.
- **`stack_whole_table_nearest`** (`stack/session.rs`). It looks at its bound every 16 values and stops reading a row whose partial distance has passed the best full distance so far. Every term is non-negative, so the token is the full scan's, the smaller id on a tie. The loop stays scalar. The timed copy of the step still scans.
- **`fbs-toks --read`** times a prefill of reading steps beside the choosing prefill and checks the replies are the same.
- **`raw-id-replies`** (the instrument) steps a prompt as the chat now does. `--count-visits 1` counts how much of the table the early-stopping scan reads at every choosing step, and checks both that scan and the token the product chose against a scan of every value. It refuses an artifact without the nearest head.
- **Five tests** in `stack/tests.rs`, and a nearest-head switch on the chat fixture: the chat's two-turn replies equal sessions stepped in full under the nearest head; a read step leaves the query and every later pick unchanged; a refused read step leaves the next step choosing; a feed that stopped on an error serves its last token on the next ask; the scan keeps the smaller id when every row is in the table twice.
- **Step 4, decided.** `uor-chat --prompt-ids` stays unmarked. Its help line says it exists for parity checks on raw ids, and the unmarked float rows are what it is checked against. The marked reader is `raw-id-replies --mark-reply 1`, whose unmarked form equals `uor-chat --prompt-ids` on 1,141 of 1,141 rows at each seed (claude-347) and is byte-identical here.

## Measurements

Model: arm r's three exported artifacts (`r-s{1,2,3}-c8256.uorlut`: vocabulary 4,096, width 256, context 8,256, nearest head, turn memory). Machine: Apple M1 laptop, one thread.

### 1. Where a served step's time goes, before the change

`fbs-toks --stages`, median nanoseconds per stage over 1,280 decode steps.

| Seed | Whole-table scan | All stages | Scan's share |
|---|---|---|---|
| 1, at `main` | 492,937 | 740,481 | 66.6% |
| 1 | 514,062 | 779,435 | 66.0% |
| 2 | 509,812 | 764,563 | 66.7% |
| 3 | 514,875 | 787,248 | 65.4% |

The last three rows are from this branch before the early stop, when its scan was still the full scan.

### 2. The reading step

`fbs-toks --read` on the final program: a 128-token prefill, 20 timed runs after one warm-up, both columns from the same run of the program. Tokens per second.

| Seed | Choosing prefill, median (fastest run) | Reading prefill, median (fastest run) | Replies after each prefill |
|---|---|---|---|
| 1 | 1,300 (1,329) | 4,152 (4,253) | the same |
| 2 | 1,290 (1,346) | 4,218 (4,277) | the same |
| 3 | 1,309 (1,370) | 4,124 (4,217) | the same |

Reading a prompt is 3.1 to 3.3 times as fast. The chat reads every prompt token but the last this way, on every turn.

### 3. How much of the table an exact early stop reads

`raw-id-replies --mark-reply 1 --count-visits 1` over the 1,141 panel prompts of arm r's sealed float score, at every step that chooses a token (the prompt's last token and each reply token).

| Seed | Choosing steps | Values read | Values in full scans | Share read | Worst query | Early stop differs from full scan | Product's token differs from full scan |
|---|---|---|---|---|---|---|---|
| 1 | 14,295 | 9,740,055,072 | 14,989,393,920 | 64.9% | 99.7% | 0 | 0 |
| 2 | 14,132 | 8,525,528,352 | 14,818,476,032 | 57.5% | 99.6% | 0 | 0 |
| 3 | 15,700 | 8,158,705,232 | 16,462,643,200 | 49.5% | 99.4% | 0 | 0 |

The two share columns are the receipt's per-mille figures, which round down (seed 1 is 64.98% and seed 3 is 49.56% to two places). The last column is the served program's own choice, with the early stop in the library, against a scan of every value: equal at all 44,127 choosing steps.

### 4. The early-stopping scan, timed

**Side by side on the real prompts.** Two builds of the reader from the same source, one with the full scan put back in the library, ran at the same moment on the 1,141 prompts (`--mark-reply 1 --time-steps 1`). Each clocks its choosing steps (the prompt's last token and every reply token) and its reading steps apart. The two ran together on a busy machine, so the ratio is the measurement, not the absolute time; the reading steps, which the scan does not touch, are the control.

| Seed | Choosing steps | Full scan: ns a step (tokens/s) | Early stop: ns a step (tokens/s) | Step time saved | Reading step, ns: full-scan build / early-stop build |
|---|---|---|---|---|---|
| 1 | 14,295 | 820,312 (1,219) | 669,710 (1,493) | 18.4% | 260,595 / 261,524 |
| 2 | 14,132 | 876,389 (1,141) | 686,594 (1,456) | 21.7% | 282,218 / 280,784 |
| 3 | 15,700 | 1,139,762 (877) | 860,312 (1,162) | 24.5% | 352,703 / 357,006 |

Both builds' replies are byte-identical to claude-347's at each seed. The same pair on the unmarked panel, as whole-run CPU seconds (`/usr/bin/time -p`, user): 52.69 and 51.87, 46.97 and 45.38, 49.60 and 46.53, so 1.6%, 3.4% and 6.2% less. Most of a panel run is prompt reading.

**The timing tool's own figures** (different moments; see the first section for what its decode measures). Tokens per second, median (fastest run), 64 decode tokens.

| Seed | Decode, full scan | Decode, early stop | Scan stage, full scan | Scan stage, early stop |
|---|---|---|---|---|
| 1 | 965 (1,289) | 1,783 (1,815) | 514,062 ns | 290,750 ns |
| 2 | 1,161 (1,340) | 1,672 (1,725) | 509,812 ns | 340,395 ns |
| 3 | 1,299 (1,342) | 2,101 (2,181) | 514,875 ns | 196,416 ns |

The choosing prefill on the tool's synthetic prompt did not move: 1,296, 1,361, 1,359 tokens per second (fastest run) before the early stop and 1,329, 1,346, 1,370 after.

### 5. Replies are unchanged

| Proof | Result |
|---|---|
| `raw-id-replies` on 1,141 prompts, three seeds, marked and unmarked, against claude-347's files | 6 of 6 byte-identical, on the build at `07ae907c3d0a43b1e58e430c6aca9bf1816312c0` and again on the final build |
| `uor-chat` on the two 18-turn scripts, three seeds, arm r and arm rt, speed figures stripped | 12 of 12 identical to the like-for-like earlier transcripts |
| `fbs-toks --read`: replies after a reading prefill against a choosing prefill | the same at three seeds |
| `fbs-toks --stages`: the timed copy of the step against the plain step | the same at three seeds |
| The product's chosen token against a scan of every value, on every choosing step of the panel | see table 3, last column |

Output checksums (sha256, first 16 hex digits), the same in claude-347's files: seed 1 unmarked `ebf121233f0d4e32`, marked `f3a9c74bdabdc942`; seed 2 `76a91cc14ceb46d6`, `62872ee5d782eb97`; seed 3 `983c7918ea0d496a`, `95c47ef132a59e6b`.

### Evidence

Laptop files, not committed: `local/claude/claude-348/` (`toks-final/` timing reports; `fi-final/` reader outputs at the first code commit; `final2/` reader outputs, receipts and counts on the final build; `ab2/` the timed pair; `ab/` the unmarked pair with CPU times; `conv-final/` transcripts). Inputs: `local/claude/claude-345r/art/`, `local/claude/claude-345t/art/`, the float score's `samples-p-greedy-memory.json` per seed, claude-347's `fi2/` and `r3/`.

Programs (sha256). Built from the library at `07ae907c3d0a43b1e58e430c6aca9bf1816312c0`, which later commits do not change: `fbs-toks` `2554be9d8ffe2bae7b055eb41592020d38f8b96f8a233db3a1c32aa45209ea07`, `uor-chat` `caf1f91722f2a4024eef59f7bb0aa95e63a4828628db7d2c083e276785dd4b0e`, `uor-r4-stack` `e0655116f1bbc0b1968d1a2f1ca980b37e8ed6f6a4ba603d4cdadcbce9dd2f89`. The final reader: `raw-id-replies` `24d658b109f48843467412781a41cedf63ed0eae33d705120b42b862ba0e9bd0`; its twin with the full scan put back: `745776bbc0cd6f5416f2accb2dba7be578d431dfe43c863a2bf4072ff55310de`.

## Checks run at the head

- **Tests at the final code:** `cargo test --release -p uor-r4-integer --features serving-wasm --lib -- nearest stack_chat read_step early_stopping`: 30 passed, 0 failed.
- **Negative gates, each run once and then restored.** The tie rule flipped to keep the larger id: the tie test fails, and the older nearest-head test still passes, so the new test covers what it did not. The recovery guard in `feed` removed: the feed test fails with `EmptyHistory`. The mark left set after a refused read step: its test fails.
- **The step's audit** (no vector or matrix multiply, no floating point) ran on the final `uor-r4-stack` and passed.
- **Fresh-context review, two reviewers in this session at `07ae907c3d0a43b1e58e430c6aca9bf1816312c0`**, each given the spec and the diff, never the plan.
  - Serving step, chat and tests: APPROVE, no correctness defect. Its two missing tests are added. Its two notes are under Still open (a panic would leave the mark set) or harmless (the no-choice value equals what the scan returns for an empty table, which no parsed model has).
  - Instruments: CHANGES, one measurement item: the count described a scan the product does not run under the writer head. The count now refuses an artifact without the nearest head. Its suggestion to check the product's own choice is taken: 0 of 44,127 differ. Its question, whether skipping the content readout on read tokens can change a reply, is answered by the code: the readout keeps only scratch between calls, so its answer does not depend on earlier calls.
- **Confirmation at `4734d050dd20dd72fec1d9703a61c9e1844f631d`.** A third fresh-context reviewer read the code added since the first review (two tests, the reader's count and step clock) against its spec, without building: no findings; each test fails if its rule is broken, and the count's scans are outside the clock. It ran out of room before the recount, so the recount is the author's, not a fresh-context one: every figure in Measurements was recomputed by command from the laptop files and matches (the five tables and the figures in the text under table 4; 24 reply-file comparisons, 12 transcripts, 6 output checksums, 5 program checksums). It found one thing, the round-down note now under table 3.
- **Ratchets:** `uor-harness ratchet d23`, `d25` and `d32` each answer OK with nothing added (d32: 31 sites in 1 file, the baseline).
- **Format:** `rustfmt --check --edition 2021` on the reader is clean. `cargo fmt --check -p uor-r4-integer` flags no line of this unit.
- **CI at `07ae907c3d0a43b1e58e430c6aca9bf1816312c0`:** every check that ran passed (`loop policy`, `loop gates / tests`, `docs archive and links`, `fmt / clippy / tests / no_std / κ`, `cargo audit`, `fuzz smoke`, `wasm-pack build`, `Gate C trend alarm`). CI at the head is read before the queue and stated in the review record.
- **`uor-harness check`** was not run on the laptop; CI's `loop gates` job runs it at the head.

## Skills used

None beyond the project loop. The three fresh-context reviewers were subagents of this session.

## Still open

- **The scan is still the largest stage.** It is scalar on purpose: the step's audit refuses vector multiplies, and the guard that keeps the compiler from vectorising the scan also keeps it slow. An integer scan in vector instructions without a multiply would be several times faster and is a decision about the audit's rule, not a change for this unit.
- **A read step that panics would leave its mark set.** Nothing in serving catches a panic, so this has no path today.
- **`fbs-toks --read` has no test of its own.**
- The questions put to the owner after the review stand: whether "which fact answers" moves from the rule to a learned selector, and what M4's BPB bar should be.

## Plain summary

A served step spent two thirds of its time finding the nearest token in the whole table. While the chat reads what the user typed, that answer is thrown away for every token but the last, so the chat now skips the search there: reading a prompt is about three times as fast. The search itself now stops reading a table row as soon as that row cannot win, which is exact, reads a third to a half less of the table on real prompts, and makes each reply token 18% to 25% quicker. No reply changed by one token on any of the proofs: 1,141 prompts at three seeds, marked and unmarked, and twelve 18-turn conversations on two models.
