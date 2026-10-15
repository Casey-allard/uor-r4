# #1916 — First scored reply panel for the served chat model: fluent, spotty, not yet responsive (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:48:12Z
- **Updated:** 2026-10-09T13:19:24Z
- **Closed:** 2026-10-08T14:49:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1916
- **Merged:** 2026-10-08T14:49:59Z
- **Merge commit:** 77aa15b8e5caffa5fd544d79e304364a3943f1ff
- **Base / Head:** main / UOR-Foundation:deepseek/chat-panel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What was run

`geometric-stack lut-chat` on the **served chat artifact** — the model measured at **0.93277 bits/byte** on real held-out chat (4.70× better than the 4.3885 shipped), exported with GPTQ. 11 requests / 14 turns, greedy decoding, every reply recorded verbatim.

## The replies, quoted

| category | prompt | reply |
|---|---|---|
| greeting | *"Hello! How are you doing today?"* | "Yes, I'm going to take a look at the latest version of the article…" |
| greeting | *"Hi there, good morning!"* | "Yes, I can't believe I'm going to have a good morning. I'm going to have a good morning routine and I'm going to have a good morning routine." |
| knowledge | *"What is the capital of France?"* | "<<The Enigma of France>>  **The capital of France is Paris.**" |
| knowledge | *"How many days are in a week?"* | "1. \*\*Summary statistics\*\*: The data is as follows:" |
| knowledge | *"What color is the sky…?"* | "1. Ancient Egypt 2. Ancient Egypt 3. Ancient Egypt…" |
| writing | *"Write a short poem about the sea."* | "<<The Role of the Reds>>  The sea is a beautiful and beautiful place…" |
| instruction | *"Reply with exactly one word: yes"* | "Yes, that's a good example." |
| instruction | *"Answer in one short sentence: what is two plus two?"* | "\*\*\*" |
| retention | *"My name is Ana."* / *"What is my name?"* | **both turns returned identical text** |
| retention | *"I live in Lisbon."* / *"Where do I live?"* | turn 2: "Yes, I do. It's a great place to live…" — never names Lisbon |
| follow-up | *"I am planning a trip to Japan."* / *"What should I pack?"* | turn 1: "…a general plan for a trip to Japan…" (on-topic) |

## Scoring by automatic proxies

| measure | score |
|---|---:|
| fluent, grammatical English | **14 / 14** |
| degenerate repetition loops | 3 / 14 |
| correct factual answer where one exists | 1 / 3 |
| reply topically related to the turn | 5 / 14 |
| instruction followed as stated | **0 / 2** |
| second turn distinguished from the first | **0 / 1** (identical text) |

*(Proxies, not semantic judging — stated because it's a proxy.)*

## What the model is and isn't

**Is:** fluent. Multi-clause, correctly punctuated English; latches onto salient prompt words (*morning* → morning routine, *Japan* → Japan, *Ana* → Ana); one correct factual answer; volunteers conversational openers.

**Isn't:** responsive. Doesn't answer instructions, repeats itself, decorates replies with training-data artifacts (markdown headings, `<<Title>>` wrappers, numbered lists), and **multi-turn state doesn't drive behaviour** — the identical reply to both turns of *"My name is Ana." / "What is my name?"* is the panel's clearest single failure.

## Why this is the right kind of result

**The qualitative failure matches the quantitative one** — a 19.9 M model at 0.93 BPB has learned the *style* of conversation and a little of its content, and has not learned to follow instructions. That's what makes the number trustworthy and the next lever identifiable.

It also **sharpens the data question**: what the panel shows missing is **instruction-to-response binding, not volume**. Rebuilding the same templated sources (chat-v1) is therefore the *weaker* form of the data lever, and the running dose-response arms test only volume. A corpus with instruction/response structure, or an objective that binds the reply to the turn, is the better-targeted spend.

Scope: one artifact, greedy decoding, 11 requests, automatic proxies. No claim about a larger model of the same family, and none that the architecture is at fault — the same served path reproduced its float model to 0.00883 BPB. Evidence: `docs/evidence/chat_panel_first_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

