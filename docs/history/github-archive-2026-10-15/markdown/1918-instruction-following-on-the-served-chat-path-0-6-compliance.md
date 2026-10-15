# #1918 — Instruction following on the served chat path: 0/6 compliance, 22/22 fluency (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:56:47Z
- **Updated:** 2026-10-09T13:20:42Z
- **Closed:** 2026-10-08T14:58:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1918
- **Merged:** 2026-10-08T14:58:34Z
- **Merge commit:** 2c6bcf41ab808c8c2faee667a9a62b48aaa300af
- **Base / Head:** main / UOR-Foundation:deepseek/instruction-panel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What was run

A **mechanically scoreable** 17-request / 22-turn panel through `lut-chat protocol=2` on the **0.93277 BPB** served artifact — one-word answers, exact counts, yes/no, factual answers with expected content, three two-turn retention pairs, two follow-ups.

| measure | score |
|---|---:|
| **instruction compliance** | **0 / 6** |
| factual correctness | 1 / 4 |
| greeting response | 0 / 2 |
| **second-turn question answered** | **0 / 5** |
| fluency | **22 / 22** |
| degenerate repetition | 2 / 22 |
| second turn differs from first | 2 / 5 (was 0/1 in round 42) |

Decisive replies: *"Reply with exactly one word: yes"* → "Yes, that's a good example." · *"Answer yes or no: do fish live in trees?"* → "Yes, fish live in trees…" (right format, wrong answer) · *"What is two plus two?"* → "The first two sentences are indeed two." · *"What is the largest planet?"* → "…is a complex and dynamic process…" · retention: both turns return "Ana. Ana. Ana." and "My My My".

## What the model does and doesn't do

**Does:** fluent English in every case, and it's **topically conditioned** — "Japan" → itinerary opener, "bake a cake" → recipe opener, "planets" → a sentence about planets.

**Doesn't:** perform the *act* the turn requests — not one-word answers, not counts, not yes/no — and it doesn't answer a question about something it was just told.

**The distinction is now measured:** this is not fluency, not formatting (round 43's protocol fix removed that), and not serving (round 37: the served path reproduces its float model to **0.00883 BPB**). **It is a failure to bind the response to the request.**

## Why this is the right target

Everything measured tonight says the pipeline is faithful and the model is limited by what it learned. **The panel says precisely what it failed to learn.** The three retention pairs are the sharpest evidence — the model repeats the entity ("Ana") but cannot answer about it — and they point at **training data and objective, not volume**. More tokens of the same conversational register buy more fluent topical text; binding needs examples where the reply is *determined* by the turn.

Scope: one artifact, greedy decoding, one protocol, mechanical checks on the verbatim transcripts; small panel, single model — exact for this panel, not a benchmark. Evidence: `docs/evidence/chat_instruction_panel_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

