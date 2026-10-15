# #1917 — Fix the served chat path protocol: the prompt format was off-distribution (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:52:27Z
- **Updated:** 2026-10-09T13:19:26Z
- **Closed:** 2026-10-08T14:54:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1917
- **Merged:** 2026-10-08T14:54:14Z
- **Merge commit:** 33d8d8ae0c112b370429988de056c3767b73bad1
- **Base / Head:** main / UOR-Foundation:deepseek/chat-protocol
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## How it was found

Round 42's panel scored the served chat model as **fluent but unresponsive**, and the failures were *specific*: it continued the prompt instead of answering, decorated replies with training-data artifacts (`<<Title>>` wrappers, markdown headings), and looped. **Those are the symptoms of an off-distribution prompt, not of a weak model.**

The corpus was built with role-template **protocol 2** (`docs/compute/ladder-runbook.md` §2.3), and the round-42 run passed no protocol argument — so `lut-chat` used its default.

## The test: one argument added

Same artifact, same panel, same greedy decoding, `protocol=2`:

| prompt | protocol 1 (default) | **protocol 2** |
|---|---|---|
| *"Hi there, good morning!"* | "…good morning routine and I'm going to have a good morning routine." | **"Hello! How can I help you today?"** |
| *"What is the capital of France?"* | "<<The Enigma of France>> The capital of France is Paris." | **"The capital of France is Paris."** |
| *"Write a short poem about the sea."* | "<<The Role of the Reds>> The sea is a beautiful and beautiful place…" | **"In a world where the sea is a dream, there is a sense of wonder…"** |
| *"I am planning a trip to Japan."* | "Yes, I can help you with that. I have a general plan…" | **"To plan a trip to Japan, let's start with a mix of must-see attractions and activities…"** |
| *"What color is the sky…?"* | "1. Ancient Egypt 2. Ancient Egypt 3. …" | "The sky on a clear day is a beautiful day, with its gentle rustle…" |

## What changed

Canonical assistant opener instead of an unrelated sentence, repetition loop gone · clean factual answer with **no wrapper** · literary prose with **no fabricated title** · a genuinely **responsive** trip answer · the numbered-list and title artifacts mostly disappear.

## What didn't

**Instruction following is still absent** (*"reply with exactly one word"* isn't obeyed; *"two plus two"* → `***`), and **multi-turn state still doesn't drive behaviour** (second turns return identical text; the name request degenerates to "Ana. Ana. Ana." — though it now at least produces the name).

## Why this matters

1. **A real chat-quality gain from a configuration fix, with no training** — the prompt format had been off-distribution for *every served reply generated in this session*.
2. **A second instance of the session-long error class**: a default adopted without checking it matched the corpus — and this one was in the chatbot path itself, not in the measurement.
3. **It narrows the real deficit to instruction-following and multi-turn conditioning** — training-objective and data properties that no protocol setting supplies.

Scope: one artifact, one small panel, greedy decoding, two protocol settings; scoring by inspection of the verbatim transcripts, not by a judge. Evidence: `docs/evidence/chat_protocol_defect_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

