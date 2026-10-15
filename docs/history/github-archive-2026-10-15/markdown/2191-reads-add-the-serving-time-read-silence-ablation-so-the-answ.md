# #2191 — reads: add the serving-time read-silence ablation, so the answers can be attributed to a read head (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T06:27:07Z
- **Updated:** 2026-10-11T06:36:12Z
- **Closed:** 2026-10-11T06:32:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2191
- **Merged:** 2026-10-11T06:32:49Z
- **Merge commit:** 44f77f830bab5337493b97d2d159ebe11ae0e97d
- **Base / Head:** main / UOR-Foundation:deepseek/head-ablation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**The read-binding brief asks for *"a head-ablation check that the bound head drives the answer"* — the one causal check the powered cycle could not run, because nothing could silence a specific read head at read-out. `set_read_silence(Some(layer))` makes that layer's read contribute nothing to the residual (`x = x + 0`) with the rest of the forward unchanged: a read-out change only, no parameter, no gradient, no training path, not saved, and a non-read layer is refused. `chat-grade reply` takes `read_silence_layer=L` and records it in `replies.json`.**

Line: read binding / head ablation (D22 order 4) · count 0/3 · headline unchanged (preparation)

## What it prepares, and what it found

The head ablation on the read-bound artifact **S2** (24/40 unablated), silencing one read layer at a time on the frozen v5 panel:

| condition | memory | wrong-value | no value | mean reply words |
|---|---:|---:|---:|---:|
| **unablated** | **24/40** | 5 | 7 | 4.8 |
| silence the **first** read (layer 2) | 2/40 | 9 | 29 | 18.2 |
| silence the **middle** read (layer 5) | 7/40 | 12 | 21 | 5.5 |
| **silence the supervised (last) read (layer 8)** | **0/40** | **0** | **37** | 6.3 |

Silencing the layer the read-binding supervision targets does not merely degrade the answers — it removes them: **no row produces a value at all**. That is the brief's causal check answered, and it agrees with the mechanism's own result.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

