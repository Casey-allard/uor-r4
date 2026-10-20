# #1920 — Prompt scaffolds do not fix the chat deficit; three probes agree it is a training-data gap (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:05:04Z
- **Updated:** 2026-10-09T13:22:02Z
- **Closed:** 2026-10-08T15:06:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1920
- **Merged:** 2026-10-08T15:06:49Z
- **Merge commit:** 80d38a25cd67faa52381e87e0396f2c0c03b3e02
- **Base / Head:** main / UOR-Foundation:deepseek/scaffold-probe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The hypothesis this eliminates

After round 45's first-token result, the remaining cheap explanation was: *the model might answer if the turn looked like an answer-shaped training example.* Six turns got explicit scaffolds, greedy, `protocol=2`, `max_new_tokens=12`:

| scaffolded turn | reply |
|---|---|
| "Question: is the sky blue?\nAnswer with one word:" | " The sky blue is a beautiful blue," |
| "Question: do fish live in trees?\nAnswer with one word:" | " Question: Answer with one" *(template echo)* |
| "Question: what is the capital of France?\nAnswer:" | " The capital of France is Paris." |
| "Question: how many days are in a week?\nAnswer with a number:" | " To find the number of days in a week, we need" |
| "Question: what is the largest planet…?\nAnswer:" | " The largest planet in our solar system" |
| "Question: how many cats do I have? I have two cats.\nAnswer with a number:" | " Question: What is the best way to" |

**The scaffold changes nothing.** The model still writes descriptive sentences about the topic, and on two turns it echoes the template itself. The one correct factual sentence (France) is the same one the unscaffolded panel produced — the scaffold was never doing the work.

## Three probes now agree

| probe | result |
|---|---|
| 22-turn instruction panel (round 44) | **0/6** instruction compliance |
| first-token probe, 8 decisive turns (round 45) | **not one** opens with the answer |
| answer-scaffold probe (this round) | **no change** |

**So the deficit is not response length, not a stop condition, and not a prompt convention.** The model has no representation of the requested **response form**, and no phrasing of the request supplies one.

## Why this matters

It **rules out the cheaper explanations that would have avoided a data intervention** — which is worth knowing before spending on data, and it makes the conclusion robust rather than a single surprising measurement. It also means the target is now unambiguous:

> **training data whose replies vary in form and are determined by the turn** — short answers, yes/no, counts, lists mixed with prose.

And this is a *negative* result recorded as one: no chat-quality gain this round, one hypothesis eliminated, one fewer way to waste a data budget. Evidence: third addendum in `docs/evidence/chat_instruction_panel_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

