# #1885 — Measure the Kneser-Ney gate on real held-out text: 1.280275 bits per byte (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T09:11:59Z
- **Updated:** 2026-10-09T13:20:45Z
- **Closed:** 2026-10-08T09:13:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1885
- **Merged:** 2026-10-08T09:13:38Z
- **Merge commit:** f6266097bdf7cb60fabad4f3fbc201fcc5f788df
- **Base / Head:** main / UOR-Foundation:deepseek/kn-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds

**`kn-ngram-gate`** — fits the interpolated count-pruned Kneser-Ney 5-gram (`uor_r4_training::ngram`) on a declared training stream and scores a disjoint held-out stream with the **same tokenizer and byte accounting** the model evaluations use, splitting the training stream around out-of-vocabulary tokens so no window is rejected whole.

## The gate: 1.280275 bits per byte

`chat-v0-p2` train (20M tokens) → `chat-v0-p2/heldout`, a separate file:

| | |
|---|---:|
| bits per byte | **1.280275** |
| scored positions | 399,988 |
| scored bytes | 1,088,056 |
| skipped for OOV | 8 |

Every native-mechanism measurement in this session was compared against a 4.211070 baseline computed over **256 positions / 693 bytes**. That slice cannot distinguish any effect this project has measured, and no n-gram gate had ever been recorded for this corpus — even though the transition plan sets one as its acceptance bar. This is the first trustworthy gate for real conversational text on this board.

## What an audit of the project's own history changed

- **The TinyStories gate was measured once before** (Card P3, PR #1283): matched KN 5-gram **1.2055** against the discrete artifact at **1.2372 continuous** and **1.8055 as shipped**. The geometric model lost to the n-gram by 0.0316 against a pre-registered +0.30 bar, and the card was retired.
- So the largest measured gap in this project is **not** any mechanism delta: it is the **0.5683 BPB between the continuous trainer and the artifact actually shipped**. Everything this session chased was 0.000006–0.02 BPB — 30 to 100,000× smaller than the gap the serving path already has.
- A **214M geometric stack** (`geo-214m-e448de86`, `rrarrarrarrarrar`, width 1536, L2 read, 1.85B tokens) records NLL 1.0322358 on TinyStories V2 validation, ≈**0.37 BPB** on that corpus's byte basis. If that derivation holds, the architecture is not the barrier at all.
- The audit also confirms this session's retractions and the invalidity of the 693-byte chat numbers, and notes that no ladder/214M run ever recorded `bits_per_byte` — it is `null` in every report, so none of those is a BPB measurement.

## Next, in order

1. Score the 214M stack on TinyStories V2 validation with the same byte accounting, and run this gate on the same split — a like-for-like head-to-head instead of a derived one.
2. Measure the continuous→served gap directly and locate where it is lost (read quantisation, the additive scorer's feature set, or the export path).
3. Only then revisit the mechanism. Stages 3 and 4 both aim at this gap.

**Scope**: BPB is comparable only within a corpus. The chat gate belongs to chat-v0-p2 with tokenizer `d36d3e87`; the TinyStories 1.2055 belongs to that corpus; the 214M figure is a derivation from nats-per-token, flagged. No training was run for this PR.

Evidence: `docs/evidence/kn_gate_real_text_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

