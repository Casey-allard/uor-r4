# #1921 — Launch the response-form intervention: 50/50 chat and short-turn dialogue (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:16:49Z
- **Updated:** 2026-10-09T13:20:07Z
- **Closed:** 2026-10-08T15:18:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1921
- **Merged:** 2026-10-08T15:18:33Z
- **Merge commit:** e578ad7dfc0175d736c49744b5f17a3e4372eef3
- **Base / Head:** main / UOR-Foundation:deepseek/form-intervention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The diagnosis this answers

Three independent probes agree the served chat model has **no representation of the response form a turn requests**:

| probe | result |
|---|---|
| 22-turn instruction panel (round 44) | **0/6** compliance |
| first-token probe, 8 decisive turns (round 45) | **not one** opens with the answer |
| answer-scaffold probe (round 46) | **no change** |

The model writes descriptive prose about the topic of every turn. With serving proven faithful to **0.00883 BPB** (round 37) and the prompt format corrected (round 43), the remaining explanation is **training data whose replies are uniformly prose**.

## The intervention, now training

Same frozen 7.16 M chat-only recipe — one change:

```
train=chat-v0-p2/train/tokens.u16,tinydialogues-train/tokens.u16  train_weights=0.5,0.5
```

`tinydialogues` is **42.8 M tokens of short turn-pair dialogue** — question/answer shapes, minimal prose, exactly the register the chat corpus lacks. It's the only form-diverse tokenised corpus on this machine; every other large stream is one of the **eight copies of the same 82 M-token chat-v0-p2 set** that round 36's data review enumerated.

**Honest limitation stated up front:** tinydialogues is low-register synthetic child dialogue, and the mixture changes topic distribution as well as form, so this arm alone doesn't separate the two. If it moves the yardsticks, it says *form diversity helps* and a better corpus of that kind is worth assembling — not that the model is now a good assistant.

## Judged by instruments fixed before launch

1. **22-turn instruction panel** — compliance above 0/6, factual above 1/4, answered second turns above 0/5.
2. **`max_new_tokens=1` first-token probe** — the sharpest test: anything other than `" The"` / `" To"` / `" I"` / `" "` on the yes-no and one-word turns is evidence form conditioning exists.

Both were recorded before the arm was launched, so there's no selection.

## Recorded decisions

- **The volume arms are retired** to make room: the 2-epoch arm completed its work and the queued 4-epoch arm was stopped, because rounds 44–46 all point at form rather than volume and the GPU is the scarce resource. **A scientific decision, not a budget one.**
- **What would falsify it:** neither yardstick moving means the deficit is the *objective* or *capacity*, not the corpus's form distribution — and the next step becomes an objective change rather than a data change. A fall in fluency would mean the mixture damaged the register.

Scope: one size, one mixture weight, one corpus pair, single seed. **No result is claimed — the arm is training.** Evidence: `docs/evidence/form_diversity_intervention_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

