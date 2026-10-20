# #1617 — Add a synthetic in-context memory corpus generator and a store inspector

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T14:05:17Z
- **Updated:** 2026-10-02T14:07:36Z
- **Closed:** 2026-10-02T14:07:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1617
- **Merged:** 2026-10-02T14:07:35Z
- **Merge commit:** c8b684b6052d4e3cc47e8c2b6ee5a0df7d5eeb61
- **Base / Head:** main / UOR-Foundation:codex/synthetic-memory-corpus-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

Two tools, both **enablers for the emitter arm**, neither claimed to be a result.

**Retracted premise this fixes.** My earlier card on #1512 said to fit the response phase on *"the chat-v0 response store **plus the memory rows**"*. That store is **82,529,690 tokens across 73,632 rows**; the ten `dev-mem-*` rows are development-panel entries and are about **500 tokens — 0.0006%**. Training on them would change nothing. Testing whether the emitter's competing-turn deficit is trainable needs a **distribution** of such bindings, which is what `synthetic-memory-corpus` produces.

**`synthetic-memory-corpus`** builds fact / competing-statement / question documents whose assistant turn is the correct answer, in the corpus format the trainer already loads (`uor-r4-chat-corpus/v1`): `<|bos|>` then turns joined by one newline, each turn `<marker><content>`, every assistant turn ending `<|eos|>`; `tokens.u16` and `response_mask.u8` with the mask 1 on the response and its EOS. Encoding goes through the project's own `DialogueProtocol`/`DialogueEncoder`, so role markers, separators and masking are **not** re-implemented.

Properties that make it a fair test rather than an easy one: the asked relation **and** the competing statement are always both stated; the answer is always the asked relation's value; the competing relation is **never** the asked relation; the asked statement is **first half the time and second half the time**, so recency cannot be the rule the corpus teaches; and the overlapping relations (colour/car) sometimes share a competing value, so the distractor is a plausible answer.

**`inspect-chat-corpus`** decodes a store with its own tokenizer and reports whether each document starts at BOS, has a masked token, and has a non-empty masked region. It exists because **token ids are not characters** — reading `tokens.u16` as text reports nonsense — and because a store whose mask misses the answer is **silently useless rather than visibly broken**.

## Exact-head evidence

**Reviewed head:** `9a3968bc8b68539e1f95b904aad3bfc2b9ecaa8e` (two commits on `origin/main` `46fd32b1`).

- **Compile:** `cargo check --features cpu-accelerate -p uor-r4-training --bin synthetic-memory-corpus --bin inspect-chat-corpus` → Finished (33.61s). `cargo fmt` applied.
- **Generated and inspected, executed:** `rows=2000 seed=7` → **2,000 documents, 103,175 tokens, 17,006 response tokens (16.5%)**. The inspector reports **0 documents not starting at BOS, 0 with no masked token, 0 whose masked region decodes to nothing**.
- **Decoded output, which is the real evidence:**

```
<|bos|>User: I drive a blue car.\nUser: I was born in January.\nUser: What color is my car?\nAssistant: You drive a blue car.<|eos|>
<|bos|>User: My birthday is in July.\nUser: I have three brothers.\nUser: How many brothers do I have?\nAssistant: You have three brothers.<|eos|>
<|bos|>User: My job is farmer.\nUser: My car is red.\nUser: What color is my car?\nAssistant: Your car is red.<|eos|>
```

Scored region is `You drive a blue car.<|eos|>` — **the asked relation's value**, with a competing relation stated in the same context.

## Two defects found and fixed by running the tools against each other

1. **The inspector read `tokens.u16` as a bare token array.** It is a corpus file with a **64-byte `CORPUS_HEADER_SIZE`** header, so every boundary was shifted and it reported a false 32-byte mismatch and 27 masked-token-free documents **against a corpus that was in fact correct**. It now reads through `MmapCorpusReader`, the same reader the trainer uses. **The tool was wrong, not the corpus** — I had already reported it as a corpus defect in a commit message and corrected it in the next commit.
2. **Answers were ungrammatical** — *"Your brothers is seven."* — because the generator interpolated the relation id into one sentence. Each relation now carries its own templates (*"You have {v} brothers."*, *"Your sister lives in {v}."*), verified by decoding.

Both are recorded rather than squashed away, since the first one is an example of a validator raising a false alarm on correct data.

## Scope, and what this does NOT establish

Both tools are **instruments**. **No model has been fitted on this corpus and no capability claim is made.** 2,000 rows at 16.5% response fraction is about 17k supervised response tokens against the chat-v0 store's 56.7M — **small on purpose**, and a deliberate fraction must be chosen and stated before any fit rather than inherited from a smoke test. The corpus is synthetic: it measures the emitter's **in-context binding**, not language ability, and nothing here qualifies any memory or serving claim.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

