# #1925 — Response mask extraction specified to the line: literal role markers make it a scan (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:48:59Z
- **Updated:** 2026-10-09T13:21:15Z
- **Closed:** 2026-10-08T15:50:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1925
- **Merged:** 2026-10-08T15:50:47Z
- **Merge commit:** 3d2eb5ee4a037433532713f1d177a09b70142256
- **Base / Head:** main / UOR-Foundation:deepseek/mask-spec
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What was pinned down

Round 50 showed the mask is *recoverable*; this round fixes **exactly what to scan for**, from source:

1. **Protocol version 2, named.** `crates/uor-r4-tokenizer/src/dialogue.rs:22`: `pub const SCHEMA_V2: &str = "uor-r4.literal-role-dialogue/2";` — reached via `literal_roles_version(&tokenizer, 2)` (`dialogue.rs:110-125`).

2. **The markers are literal text.** `dialogue.rs:16-17` documents v2 as *"each role marker ends at its colon (`User:`, `Assistant:`)"*. So the assistant marker is the literal string **`Assistant:`**, encoded as ordinary tokens in the stream.

3. **The contract's marker** comes from `encode_assistant_prefix` (`dialogue.rs:250-252`) → `protocol.assistant_marker()` (`:367`), exposed as `assistant_marker_ids` (`stack_dialogue.rs:333-352`).

4. **Independent confirmation the corpus carries it:** the served model answers coherently under `protocol=2` (round 43) and degenerates into template echoes without it (round 46) — impossible if the literal markers weren't in the training stream.

## The extraction, implementable in one small tool

- **Inputs:** `chat-train.u16`, the local tokenizer (sha `d36d3e87…`), version 2.
- **Steps:** encode the literal markers → scan the stream → mark tokens after each `Assistant:` as response=1 until the next role marker or BOS → write **`response_mask.u8`**, the same filename the builder itself uses (`prepare-chat-parquet.rs:210`).
- **Checks:** marker counts and spacing (conversation count, response fraction), plausibility against the builder's own `response_tokens` statistic, and what `dialogue-train` requires in its manifest (it takes tokens, mask and manifest together).

## Why this is the step

Round 48 **falsified the data hypothesis** with a pre-registered arm, leaving the **objective** as the route; round 50 **removed the cost** of that route; this round **removes the ambiguity** about its input — a marker scan over a stream that already contains literal markers. Not a rebuild, not a retokenisation, not a new corpus.

The judging instruments were fixed in rounds 44–45 and are unchanged: the **22-turn instruction panel** and the **`max_new_tokens=1` first-token probe**.

## Scope

Read from source. **No scan has been run, no mask produced, no objective change trained.** The claim is that the mask is a scan away, with the lines that say so — and the verification of that claim (the marker-count check) **has not been executed**. Evidence: `docs/evidence/mask_extraction_spec_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

