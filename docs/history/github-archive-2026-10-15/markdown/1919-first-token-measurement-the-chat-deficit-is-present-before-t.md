# #1919 — First-token measurement: the chat deficit is present before the first word (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:00:55Z
- **Updated:** 2026-10-09T13:20:03Z
- **Closed:** 2026-10-08T15:02:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1919
- **Merged:** 2026-10-08T15:02:41Z
- **Merge commit:** 4e522e7fa1bbc0c8696d6b2c8919691840602d2f
- **Base / Head:** main / UOR-Foundation:deepseek/first-token
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The diagnostic

Re-ran eight decisive panel turns with **`max_new_tokens=1`**, greedy, `protocol=2`, to separate *"the model knows what to say but doesn't stop"* from *"the model never commits to an answer"*:

| turn | first token |
|---|---|
| "Reply with exactly one word: yes" | `" "` — not "Yes" |
| "Answer yes or no: is the sky blue?" | `" The"` — not "Yes" |
| "Answer yes or no: do fish live in trees?" | `" "` — not "No" |
| "What is the capital of France?" | `" The"` — not "Paris" |
| "How many days are in a week?" | `" To"` — not "Seven" |
| "What is the largest planet…?" | `" The"` — not "Jupiter" |
| "Good morning!" | `" I"` — a sentence about the speaker |
| 2nd turn of the cats pair | `" It"` — not "Two" |

**Not one of the eight opens with the answer.** Seven open with an article, preposition or pronoun — the start of a **descriptive sentence about the topic**.

## What it establishes

The deficit is **not** response length and **not** a missing stop condition: it's present in the **first token**. The model treats every turn as *a topic to write about* and has **no representation of the response form** the turn requests — one word, yes/no, a number, a name. That's exactly what training on conversational text whose replies are uniformly prose produces.

## Consequences, recorded

- A **boundary/EOS objective alone will not fix it**.
- **Rebuilding more of the same conversational sources (chat-v1) will not fix it either** — the *form* distribution would be unchanged. This is now the third independent line of evidence against volume being the lever (the panel's instruction failures, the expert's distribution-vs-volume argument, and now the first token).
- What the failures point to is **training data with response-form diversity** — short answers, yes/no, counts, lists mixed with prose — so the first token is conditioned on the **request** rather than the **topic**.

## Why this is useful

It is the **sharpest available statement of the chat path's deficit** and the most specific target the session has produced — and it's **checkable in one command** (`max_new_tokens=1` on the same panel) after any change, which makes it a yardstick rather than a one-off observation.

Scope: eight turns, one artifact, greedy, single protocol; the finding is about the first token of those turns and is stated as such. Evidence: addendum in `docs/evidence/chat_instruction_panel_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

