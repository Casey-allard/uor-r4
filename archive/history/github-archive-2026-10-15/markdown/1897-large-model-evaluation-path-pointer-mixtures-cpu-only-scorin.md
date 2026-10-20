# #1897 — Large-model evaluation path: pointer mixtures, CPU-only scoring, blocked data lever (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T12:05:04Z
- **Updated:** 2026-10-09T13:20:48Z
- **Closed:** 2026-10-08T12:06:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1897
- **Merged:** 2026-10-08T12:06:44Z
- **Merge commit:** 755a9027c6a7c935215ef4dfcce4248f7ab58443
- **Base / Head:** main / UOR-Foundation:deepseek/large-eval-path
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why this document exists

Round 12 measured the **data** term (0.1592 BPB) larger than the **size** term (0.0837). Testing whether size grows with scale needs no training — the private store already holds chat-trained 214 M stacks. Preparing that test surfaced three facts that change what the project can cheaply do, so they're recorded rather than worked around.

## 1. The data lever has no corpus

`caseyallard/uor-r4-store` `data/` holds only `ft-balp.tar` (327 M), `ft-dev.tar`, `hot/sieve-panel.tar`, `step5-inputs.tar` (1.5 G), `tokenizer.json.tar`, `MD5SUMS`; `corpora/*` matches nothing. The store's chat content is **models**: `models/chat-214m-smoltalk-b16` and `models/chat-214m-planA-af-b8` (856.1 MB each, `rrarrarrarrarrar`, width 1536, heads 24, ctx 384, read L2). The canonical volume's `chat-v1-p2-train` is empty.

**The larger of the two measured levers currently has nothing to spend on.**

## 2. `snap-evaluate` refuses pointer models — by design

> *"snap-evaluate reads the model's raw logits, which are not a pointer model's distribution (its head mixes a copy distribution in); score a pointer model with evaluate, dialogue-train's development or m-world evaluate, which use the mixture."*

The chat-214 M checkpoints **are** pointer models. So every `snap-evaluate` number from tonight (0.8770, 0.9138, 0.9607) belongs to a different scoring path — **cross-path comparisons must go through `evaluate`**. That's a real scoping constraint on the league table, and it's now written down.

## 3. The evaluation modes are CPU-only in this build

Neither `evaluate` nor `snap-evaluate` accepts `device=` (though `train` does, and the name is a known option). Measured: **625 % CPU across six-plus cores, 57 minutes of CPU time in nine minutes of wall time**, with no progress output before completion — hours for a 16,384-target scan.

## What would make this cheap

1. **Device selection for the evaluation modes** (or a build that uses the GPU `train` already uses). The 214 M model is the project's strongest chat artifact and currently the most expensive thing to score.
2. **A smaller declared scan** (`tune_blocks`) for coarse decisions — the question is whether 214 M lands near 0.8770 or far below it, which a few thousand targets answer.
3. **A chat corpus in the store** — the one thing the data lever needs.

Scope: **no model-quality number is claimed**; the 214 M evaluation had not completed and was left running rather than cancelled. Evidence: `docs/evidence/large_model_evaluation_path_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

