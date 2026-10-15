# #1922 — Form-diversity intervention fails: the deficit is the objective, not the data (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:35:23Z
- **Updated:** 2026-10-09T13:20:09Z
- **Closed:** 2026-10-08T15:37:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1922
- **Merged:** 2026-10-08T15:37:08Z
- **Merge commit:** bf1f4fd8acceebb0b0c5c9c6c9ec4c9fa3a9b87f
- **Base / Head:** main / UOR-Foundation:deepseek/form-verdict
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The pre-registered test

Recorded before the arm ran (`form_diversity_intervention_2026-10-08.txt`): frozen 7.16 M recipe, 13,348 steps, **one change** — the training stream is 50/50 chat-v0-p2 and tinydialogues (42.8 M tokens of short turn-pair dialogue). Judged by the 22-turn instruction panel and the `max_new_tokens=1` first-token probe, both fixed in advance.

**Stated falsifier:** *"neither yardstick moving means the deficit is not the form distribution of the corpus — it would then be the objective … or capacity."*

## All three measures moved the wrong way or not at all

**1. First-token probe — no improvement:**

| turn | chat-only | form-mixed |
|---|---|---|
| one word "yes" | `" "` | `' I'` |
| is the sky blue? | `" The"` | `' The'` |
| do fish live in trees? | `" "` | `' '` |
| capital of France? | `" The"` | `' The'` |
| how many days in a week? | `" To"` | `' The'` |
| largest planet? | `" The"` | `' The'` |

Still articles and pronouns. **Not one of the eight opens with the answer, in either model.**

**2. Panel — worse, with new failure classes:** backtick code artifacts ("The \`calculate_list\` function…"), literal placeholders ("the [city] of the [city]"), echoed phrases ("a detailed and detailed description"), and more repetition loops. Only the France opener stayed correct, then degraded.

**3. Chat fit — worse:** final dev nll **2.0172** vs the chat-only control's **1.8900** — **0.127 nats (≈0.065 BPB)** on the corpus the model is actually judged on.

## Conclusion, taken from the pre-registered falsifier

**Form diversity in the training corpus does not produce response-form conditioning.** Adding 42.8 M tokens of short Q/A dialogue neither made the first token responsive nor improved a single panel category, while damaging the register and the chat fit.

**The deficit lies in the objective — what the model is asked to predict — or in capacity, not in the form distribution of the data.**

This is a *decisive and cheaply obtained negative*: it removes the data-shaped half of the option space, and it **argues against further corpus spending — including the 1.39 B-token chat-v1 rebuild**, which three independent lines of evidence now point away from (round 36's distribution argument, rounds 44–46's probes, and this).

## What the objective change would be — stated without claiming it works

The model predicts the next token of a **flattened stream**. Nothing in that objective distinguishes reply tokens from prompt tokens, so "answer the turn" is not something it is asked to learn — which is precisely what the panel shows. A loss that **marks the response region**, or an objective that **conditions generation on the turn** rather than continuing the transcript, is the class of change the evidence now calls for. The machinery exists in the repository (the corpus carries a response mask; the dialogue trainer accepts one); **no result is claimed for using it here.**

Scope: one size, one mixture weight, one corpus pair, single seed, two probes fixed in advance. The arm couldn't separate form from topic distribution by construction — but the result is negative on **both**, so the confound doesn't rescue it. Evidence: `docs/evidence/form_diversity_verdict_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

