# REPORT: one-model-end-to-end, claude-271 (long multi-turn conversations: chained episodes with recall across them)

Lab: claude. [ORDER](./ORDER.md). PR #3642, item #3407. Plan step: Casey's plan of 4 November, step P2.

**In one paragraph.** The M-world v2 generator can now write a chained conversation: ordinary episodes back to back up to a token budget, with notes stated early and asked for at the end. `MWorld2::chained` writes it, and the corpus writer's new `chain=TOKENS` option exposes it.
- **The census:** 2,000 conversations at a budget of 2,048 tokens, with protocol 2 and the pinned tokenizer, run on the laptop.
  - Documents run 1,728–2,048 tokens, a mean of 1,964. Before this unit, no document the project trained or scored on passed 258 tokens.
  - 7,537 queries sit 34–2,010 tokens from their notes, a mean of 1,190; 4,467 of them lie past 1,024 tokens.
- **Defaults are unchanged byte for byte:** the default-stream digest, the v2 table digest and the writer's pinned receipts all pass unedited.
- **Not done:** no model was trained. No corpus was written for training: that is the next unit, and under today's 192-token training window it could not use these distances (What did not work, 1).

## Tokens per second first

No model ran, so there is no decode speed to report. The census measures the data a later unit will decode against: 3,928,381 tokens in 2,000 documents, against a 245–258 ceiling before. Generating it took 349 s on the laptop in a debug build. CPU data generation is host work, not training.

## What did not work

1. **Today's trainer cannot see these distances.** `build_document_windows` (`hamilton_sheet.rs:1577`) cuts each document's targets into back-to-back chunks of `SHEET_CONTEXT = 192` (`:66`) from its `<|bos|>`, so a row sees at most its own chunk. `train-in-space.rs:1050` and `in_space_long.rs:772` both call it.
   - 7,223 of the census's 7,537 queries (95.8%) lie past 192 tokens from their notes, so under today's window none of them can be answered from context.
   - The chained corpus is the data a longer window needs, not a gain by itself. A longer training window is `integer-long-context`'s rule (c) (P4), and with it the Hamming read's memory at 512–2,048 tokens, which P2's next unit tests on the GPU.
   - (`integer-long-context` cites this cut as `hamilton_sheet.rs:1351–1360`, which has since moved.)
2. **The chained corpus has fewer relation turns than the default mix.**
   - **The counts:** 4,623 relation turns over 26,669 episodes (0.17 per episode), against 597 over 2,000 (0.30) in a default corpus of the same seed, protocol and mix.
   - **The cause:** this unit's no-contradiction rule redraws any episode that states or abstains on a relation an earlier episode stated: 11,466 redraws.
   - **What rises instead:** MQAR turns (0.43 per episode, against 0.38) and copy turns (0.13, against 0.09).
   - **Classification:** a data-mix property of the design, not a defect. Cross-episode relation recall, where an episode asks about a relation an earlier one stated, would keep those turns. Whether to add it is the next unit's decision (Next).
3. **The generator's own test module has been compiled out since 23 October.** `milestone_world_v2.rs`'s `mod tests` carries `#[cfg(any())]` (antigravity-40, `e8706c545`). So `STREAM_DIGEST`, `V2_DIGEST` and the module's other tests had not run for 11 days.
   - This unit does not re-enable that module: its fixtures may have drifted, and that is a separate change.
   - Its new live `mod chain_tests` re-runs the two digest recipes, and both still pass: `0167c1a0…` (the #1541 pin) and `04ad3bb0…`.
4. **User turns repeat inside a long document.** 3,265 of 111,181 user turns (2.9%) repeat an earlier user turn of the same document. The filler pool was sized for one 256-token episode. Classification: a data property. The next unit decides whether to widen the pool before training on it.
5. **The claim waited on DeepSeek's #3638.**
   - **Why:** the item's scope held three `crates/uor-r4-cpu-train` paths (claude-268's, for the mixed copy pair). DeepSeek's `cpu-training-pilot` scope is `crates/uor-r4-cpu-train/**`, so the claim's overlap check, which compares the two items' scopes on `origin/main`, refused until #3638 merged at 05:04Z on 4 November.
   - **The same overlap blocks DeepSeek's claims** while any order of ours on this item is open.
   - **The fix:** this unit drops those three paths from the item's scope (What changed).
6. **MQAR keys can repeat across episodes, uncounted.** Nothing stops two MQAR episodes in one document from using the same key (the fresh review's note). Each reference reply is still right: an MQAR query follows its own episode's statement and takes its value. But an earlier, stale value for the same key can sit in the same document, and the census does not count how often. The next unit counts it before training on the corpus.
7. **The episode loop leaves no room for the queries.** The final document almost always runs over the budget once its queries are added, so the last episode is dropped. This is deterministic and correct, and costs about one episode per document.
8. **The census excludes nothing.** Its `exclude=` list is empty, so `excluded` redraws are 0. The real long TRAIN corpus must exclude the long panel's turns, as the default TRAIN excludes DEV's. The exclusion path is tested (`chained_draws_skip_excluded_turns_and_count_them`) but not exercised in the census.

## What changed

- **`milestone_world_v2.rs`:**
  - `MWorld2::chained(rng, cell, budget, excluded, redraws)` and its helpers `try_chained`, `chain_note` and `chain_query`;
  - new types and fields:
    - `Kind::Chain` and `Category2::Chain`;
    - `CHAIN_QUERIES = 4`;
    - the redraw counts, `ChainRedraws`;
    - an optional `Tag.chain` (`Chain`: notes, the queried note, episodes, the note's episode, distance, tokens), serialized only when present;
  - `with_spaced_meter()`, so a writer can meter protocol 2's layout;
  - the free functions `relation_facts`, `shuffle` and `mentions`;
  - a live `#[cfg(test)] mod chain_tests` with seven tests.
  - From the fresh review:
    - a chain that reaches the draw cap of 4,096 ends where it is, instead of failing the whole corpus;
    - the first episode always opens with a note;
    - the uniqueness test asserts the redraw counts, so each check shows on its own.
- **`bin/m-world-recall-corpus.rs`:**
  - a new option, `chain=TOKENS`. 0, the default, is today's corpus. Otherwise the budget must be at least 512, with `context=` at least the budget and `recall=off`;
  - a `chain` census in the manifest;
  - three new tests.
- **The item's scope** drops `crates/uor-r4-cpu-train/src/{hamming,served_forward,fbs}.rs`. The unit that runs the mixed copy pair (P6) re-adds what it needs.

## Measurements

| Check | Result |
|---|---|
| `uor-r4-training` library tests, the 7 `chain_tests` among them | 87 passed, 0 failed |
| `m-world-recall-corpus` tests | 8 passed, 1 ignored |
| The ignored pinned-tokenizer receipt, run with `UOR_R4_PINNED_TOKENIZER` | 1 passed |
| `chat-grade` tests | 26 passed |
| `rustfmt --check`, both files (the base files are clean too) | clean |
| `loop gates --full` | 11 of 11 green |

**The negative gates.** Each patch sets checks aside from `try_chained` and `chain_note`. Each run covers `a_chained_note_is_stated_and_asked_nowhere_else` (U) and `the_chained_stream_is_pinned` (P). After every run the file was restored and `cmp`-equal to the kept copy. At this unit's base `3f7f8605b` all five patches still apply, and the "all three" patch was run again: both tests fail, and after the restore all 7 `chain_tests` pass.

| Set aside | Patch sha256 | U | P |
|---|---|---|---|
| The episode redraw for naming a note | `d478f370…` | FAILS: "no episode was redrawn for naming a note" | FAILS |
| `chain_note`'s check against earlier turns | `a4e0d6d6…` | FAILS: 11 finished draws dropped | FAILS |
| The final clash check | `3c865ed7…` | passes | passes |
| All three | `47d16776…` | FAILS | FAILS |
| All three, and U's two count assertions | `ef94a281…` | FAILS: note 4's value `758` also in a "lucky number" turn | — |

The final clash check never fires on this stream, so it is a backstop, not a separately tested check. The 2,000-document census was written twice, before and after the review's fixes, and the two `tokens.u16` and `response_mask.u8` files are byte-identical (`50d6e0f2…`, `6067392c…`).

## The census

2,000 conversations, `chain=2048 context=2048 protocol=2 seed=1`, with the pinned tokenizer (`d36d3e87…`).

| Measure | Value |
|---|---|
| Documents, tokens | 2,000; 3,928,381 in all; 1,728 min, 2,048 max, 1,964 mean |
| Episodes per document | 9–20; 26,669 in all |
| Notes, queries | 9,931 notes; 7,537 queries, at most 4 per document |
| Query distance, tokens | 34 min, 2,010 max, 1,190 mean |
| Queries past 192 / 512 / 1,024 tokens | 7,223 (95.8%) / 6,190 (82.1%) / 4,467 (59.3%) |
| Metered length against encoded length | 0 mismatches |
| User turns repeating within a document | 3,265 of 111,181 (2.9%) |

The query distances in 256-token bins:

| 0–255 | 256–511 | 512–767 | 768–1,023 | 1,024–1,279 | 1,280–1,535 | 1,536–1,791 | 1,792–2,047 |
|---|---|---|---|---|---|---|---|
| 508 | 835 | 825 | 899 | 832 | 825 | 999 | 1,814 |

The redraws, by first reason:

| Reason | Count |
|---|---|
| Contradicting | 11,466 |
| Too long | 16,000 |
| Farewell | 4,025 |
| Mentioning | 183 |
| Excluded | 0 |
| Dropped | 0 |

"Too long" is 8 per document, which is the chain's stop rule: each chain ends after 8 draws that do not fit.

The turns by category, chained against a default corpus of 2,000 episodes (same seed, protocol and mix):

| Category | Chained | Per episode | Default | Per episode |
|---|---|---|---|---|
| Relation | 4,623 | 0.17 | 597 | 0.30 |
| MQAR | 11,540 | 0.43 | 754 | 0.38 |
| Copy | 3,579 | 0.13 | 183 | 0.09 |
| Instruction | 8,721 | 0.33 | 585 | 0.29 |
| Responsive | 75,181 | — | 5,204 | — |
| Chain (queries) | 7,537 | — | — | — |

## What this shows, and what it does not

- **Shows:** the generator can write conversations of up to 2,048 tokens whose answers lie up to 2,010 tokens back.
  - Every turn is judged right by its own checks.
  - Each note's key and value appear nowhere but the note and its query.
  - No default output moved.
- **Does not show:**
  - that any model can recall across these distances. Nothing was trained or scored;
  - that today's trainer would see them. It cannot (What did not work, 1);
  - that the Hamming read's quadratic distance tensor fits a 2,048-token window on the GPU. P2 still owes that finite test;
  - a long held-out panel. The panel needs its own cell, exclusion and census, which is the next unit's work;
  - realism past the generator's own templates. The episodes are the same short templates, chained.

## Next

- **P2, unit 2:**
  - a training window past 192, with a finite GPU test of the Hamming read's memory and time at 512, 1,024 and 2,048 tokens.
    - **Why the test is needed** (a projection from the code's shapes, not a measurement): `choose_sorted` (`flock_read_hamming.rs:1361`) builds a `(batch, time, time, head_width)` f32 difference for each head, then a copy of the same size for its absolute value.
    - At head width 64 and the training batch of 4, one copy is 38 MB at 192 tokens, 1.07 GB at 1,024 and 4.3 GB at 2,048. At the scoring batch of 16, the two copies at 2,048 come to about 34 GB, past the 5090's 32 GB.
    - The candidate mask is also built in a host loop, with rows × time entries per head on every forward pass: 67 million at 2,048 with the scoring batch.
    - So the 2,048 window likely needs a smaller scoring batch, or a distance computed in chunks or from packed sign bits;
  - the long TRAIN corpus and a long held-out panel (its own cell, excluded from TRAIN, with a census).
- **Decide before that corpus is written:** cross-episode relation recall (What did not work, 2) and the filler pool (4).
- **Then P3:** faster scoring.

## Checks run at the head

- **On the laptop**, at base `3f7f8605b`, code byte-identical to `a5c3f4b2d`: `cargo test -p uor-r4-training --lib --bin m-world-recall-corpus --bin chat-grade --no-fail-fast` gives the library 87 passed (the 7 `chain_tests` among them), `chat-grade` 26 and the writer 8 with 1 ignored, 0 failed. The ignored pinned-tokenizer receipt, run with `UOR_R4_PINNED_TOKENIZER` set to the tokenizer of sha `d36d3e87…`, passes. `rustfmt --check` is clean on both changed files.
- **Negative gate:** the "all three" patch above, run again at this base: both tests fail; restored byte for byte (`cmp`), all 7 `chain_tests` pass.
- **`loop gates --full`:** 11 of 11 green.
- **Review:** a fresh-context review of the diff and this REPORT, recorded on PR #3642 by `loop review`.

## Skills used

None applied.

## Still open

1. **The generator's old test module stays compiled out** (What did not work, 3). Re-enabling it is a separate change.
2. **Cross-episode relation recall and the filler pool** (What did not work, 2 and 4) are the next unit's decisions, before the long TRAIN corpus is written.
3. **MQAR key repeats across episodes are uncounted** (6).
4. **claude-270's served-mode parity check** is still owed, by the next GPU unit that scores a served checkpoint.
5. **The distance test recounts with the code's own call** (`meter.distance` over the same turns, the fresh review's note). So it pins the tag's wiring, not the slice's bounds, which the review read as right. The meter's per-turn costs are checked against the encoder (0 mismatches over the census). A recount from the encoder's token offsets, as the MQAR test does with the pinned tokenizer, is owed.
6. **Sixteen dropped draws in a row fail the whole corpus** (`chained`'s retry limit, the fresh review's note). The census dropped none, but it excluded nothing (What did not work, 8). The long TRAIN corpus, which must exclude the panel's turns, checks the drop rate first.

## Plain summary

Every conversation the model has trained on or been tested on so far is short: at most about 258 word pieces, a few turns. A chatbot has to remember what was said many turns back. This unit taught the program that writes the training conversations to write long ones. It strings ordinary short conversations together into one of up to 2,048 word pieces, plants a few facts early, and asks about them at the end. A check of 2,000 such conversations found the answers on average 1,190 word pieces before the question, and up to 2,010. Everything the program wrote before is unchanged, byte for byte. No model was trained: today's trainer reads only 192 word pieces at a time, so the next unit widens that window, after checking that the GPU's memory holds it.

References #3407.
