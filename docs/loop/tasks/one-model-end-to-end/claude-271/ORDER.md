# ORDER: one-model-end-to-end, claude-271 (long multi-turn conversations: chained episodes with recall across them)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `3f7f8605b`. Draft PR #3642.

## Goal

Every corpus and panel the project trains or scores on stops at 245–258 tokens per document:

| Split | Documents | Longest | Past 192 tokens |
|---|---|---|---|
| TRAIN (`mw-balp/train`) | 60,000 | 248 | 8,749 documents, 4.6% of tokens |
| DEV (`ft/dev`) | 400 | 258 | 28 documents, 3.5% of tokens |
| P-r, P-s, P-t, P-v, P-w, P-x, Fresh V2 | — | 245–258 | — |

The cap is by design. `milestone_world_v2.rs:96` sets `pub const CONTEXT: usize = 256;`, and the module doc says "Every episode of every kind fits [`CONTEXT`] tokens". So the trained model has never seen a conversation past 258 tokens. Long context (#3471) has nothing to measure past the window, and a bigger train would learn the same short templates better.

This unit adds a long-conversation mode to the generator and the corpus writer. Several episodes are chained into one document of up to a token budget (2,048 and beyond). Notes given early are asked for at the end, across the episodes between them, at recorded token distances.

**Design decisions (the top session's):**
- **No default path changes.** The per-episode cap stays at `CONTEXT = 256`. A long conversation is a chain of ordinary episodes, so every episode keeps the shape the A1 gate, the `Scorecard` and the digests were built for. Long recall comes from the chain's own notes and queries, under their own tag (`Tag.chain`) and category (`Category2::Chain`). It does not come from stretching MQAR's `DISTANCES` [16, 64, 200], which the A1 gate and `Scorecard` key on. So `STREAM_DIGEST` and `V2_DIGEST` cannot move: no default draw changes.
- **No contradictions across episodes.** An episode is redrawn when it says goodbye, when it states or abstains on a relation an earlier episode stated, or when it mentions a note's key or value. Without this, a chained abstention ("you never told me") could follow an earlier statement of the same relation, and its reference reply would be wrong.
- **Metered in the trained protocol.** Trained models use protocol 2. In chain mode under protocol 2, the generator meters version 2's layout (`Meter::spaced`), so the budget is the encoded length. The default writer path keeps its meter.
- **The item's scope drops `crates/uor-r4-cpu-train/src/{hamming,served_forward,fbs}.rs`** (claude-268 added them for the mixed copy pair). DeepSeek's `cpu-training-pilot` scope is `crates/uor-r4-cpu-train/**`, so these three paths block every claim on this item while any DeepSeek order is open; this claim waited on #3638, merged at 05:04Z on 4 November. The unit that runs the mixed pair (P6) re-adds what it needs, when no DeepSeek order holds it.

**Plan step (the claim check):**
- **Order step:** Casey's plan of 4 Nov, step P2 (a long-conversation corpus). Casey's words: "please make all of those points your current plan and goal for next steps".
- **What it changes in the model:** the data it trains and is scored on. The next unit writes the long corpus and a long panel, and P4 and P5 train and serve on them. Default corpora are unchanged, byte for byte.

## Cancel probe (run first)

1. Does any item or open order already generate documents past 256 tokens? Check with `grep` over the items and the open orders.
2. Is the cap still `CONTEXT = 256` at `:96`?
3. Does `one-model-end-to-end`'s scope on `origin/main` include `milestone_world_v2.rs` (claude-270's frontmatter line)?
4. Do the generator's own tests run? Its `mod tests` is compiled out (`#[cfg(any())]`, antigravity-40, 23 Oct), so `STREAM_DIGEST` has not been checked since then. If so, this unit's tests re-run the default-stream recipe in a live module.

**Result (4 Nov, ~05:10Z, at `3f7f8605b`). Class: instrument, no cancel.**
1. **None.** No roadmap item mentions a chained conversation or a `chain=` option. The only open PR is this one (#3642).
2. **Yes.** `milestone_world_v2.rs:96` is `pub const CONTEXT: usize = 256;`.
3. **Yes.** `milestone_world_v2.rs` is in the item's scope on `origin/main` (claude-270).
4. **No.** `#[cfg(any())]` sits on `mod tests` at `:2633`. So this unit's live `chain_tests` module re-runs both digest recipes (done when 2), and does not re-enable the old module: its fixtures may have drifted, and that is a separate change.

## Files in scope

- `crates/uor-r4-training/src/milestone_world_v2.rs`
- `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-271/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **A chained conversation.** `MWorld2::chained(rng, cell, budget, excluded, redraws)` returns one `Conversation2` of `Kind::Chain`.
   - **Its turns:** ordinary `conversation_in` episodes. A note turn comes before the first episode and before about one in three of the others, stating one fresh key and value in an MQAR lead template. At the end come queries for up to `CHAIN_QUERIES = 4` notes, the first note always among them.
   - **Its tag:** a new optional `Tag.chain` records the note count, the queried note, the episode count, the note's episode, the distance in tokens from the note's user text to the query's, and the document's tokens. It is serialized only when present, so default records are unchanged. `Tag.mqar`, the A1 gate and the `Scorecard` are not touched.
   - **Redraws:** `ChainRedraws` counts each unused episode under its first reason: excluded, farewell, contradicting, mentioning, too long. It also counts dropped finished draws.
   - **Tests, in a live `#[cfg(test)]` module:**
     - a pinned digest of 32 chained conversations at seed 7301 and budget 2,048;
     - every document within budget and longer than one episode;
     - every turn judged right by its own checks;
     - every recorded distance equal to a recount;
     - each note's key and value in no turn but the note and its query;
     - no relation asserted twice, no abstention after a statement, no farewell;
     - no turn equal to an excluded turn, with excluded redraws counted;
     - a budget under twice the episode context refused.
   - **Negative gate:** with the three uniqueness checks set aside, the uniqueness test fails. Restore, and it passes.
2. **Defaults unchanged.**
   - The live module re-runs the `STREAM_DIGEST` recipe and asserts `0167c1a0…` (the #1541 pin) and `V2_DIGEST`.
   - The writer's pinned receipts (byte tokenizer, and the ignored pinned-tokenizer test) pass unedited.
   - A default manifest has no `chain` key.
3. **The writer.**
   - `m-world-recall-corpus` takes `chain=TOKENS`: `0` (the default) is today's corpus; otherwise it must be at least 512. Chain mode requires `context=` of at least the budget and `recall=off`.
   - The manifest gains a `chain` census, and the per-kind table gains `Chain`.
   - The tests cover argument checks, a default manifest without `chain`, and a small chained corpus under both protocols. Each chained corpus stays within budget, is byte deterministic, and has a metered length equal to its encoded length.
4. **A laptop census** of a chained corpus: 2,000 conversations at `chain=2048`, protocol 2, seed 1, the pinned tokenizer (`d36d3e87…`).
   - **It reports:**
     - document lengths;
     - query distances: the histogram, and the share past 192, 512 and 1,024 tokens;
     - the redraws by reason;
     - the repeat rate of user turns within a document;
     - the per-category turn counts against a default corpus of 2,000 episodes.
   - CPU generation is host work, not training.
5. **A REPORT, negatives first,** plus one notebook row and the item paragraph. The item's scope line drops the three cpu-train paths.

## Stop and decide (never stop and wait)

- **The filler pool repeats heavily at 2,048 tokens:** record the repeat rate in the census, and decide whether the next unit widens the pool.
- **A chained query's answer is ambiguous** (a key or value repeated): it is a bug. Fix the uniqueness; never relax the judge.
- **Any default digest moves:** the change is wrong. Find the extra rng draw.
- **Training's window is shorter than the distances:** `build_document_windows` cuts targets into 192-token chunks. Then the corpus is the data a longer window needs, not a gain by itself. Record it, and the next unit pairs the long corpus with a longer training window and the Hamming read's GPU memory test.
- **The no-contradiction rule skews the kind mix** (relation episodes redrawn): record the per-category shares against the default. Decide whether the next unit adds cross-episode relation recall (an episode asking about a relation an earlier one stated) instead of redrawing.
