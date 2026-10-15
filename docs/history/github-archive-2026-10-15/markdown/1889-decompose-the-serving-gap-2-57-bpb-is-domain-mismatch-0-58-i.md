# #1889 — Decompose the serving gap: 2.57 BPB is domain mismatch, 0.58 is export (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T09:39:56Z
- **Updated:** 2026-10-09T13:22:09Z
- **Closed:** 2026-10-08T09:41:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1889
- **Merged:** 2026-10-08T09:41:35Z
- **Merge commit:** 0687d522d5c4ca1bb00d1bbec3d13606145139be
- **Base / Head:** main / UOR-Foundation:deepseek/serving-gap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question

Round 3 measured a **3.27 BPB gap at matched protocol** between the trained stack (`geo-8m-l2`, 1.1199 on chat) and the served artifact (4.3885, same file, same 256-token context). This separates it.

## The measurement that separates them

Same artifact, same protocol, same tokenizer — only the corpus changes:

| corpus | BPB | top-1 |
|---|---:|---:|
| chat-v0-p2/heldout (out of domain) | 4.3885 | 2.25 % |
| tinystories_train (**in domain**) | **1.8202** | **25.81 %** |

The artifact was trained on TinyStories. **In domain it is a working model with a quarter of next tokens correct; off domain it is worse than a unigram.** So the 3.27 BPB is not a serving defect.

## The decomposition, all measured

```
domain mismatch        4.3885 (served, chat)  - 1.8202 (served, TinyStories)   = 2.5683 BPB
export / discretisation 1.2372 (continuous)    - 1.8202 (served, in domain)     = 0.5830 BPB
out-of-domain sanity    3.2299 (order-0 table, 16 KB, chat train) vs 4.3885 (served, 2.7 MB)
```

Matched-protocol reference rows on the same chat file: `geo-8m-l2` **1.1199**, Kneser-Ney 5-gram ~1.23–1.28, served artifact 4.3885 (top-1 2.25 %), uniform 4.2292 (top-1 0.024 %).

## What this kills, and what it points at

1. **"The serving path loses 3.27 BPB" is wrong as stated** — the in-domain serving loss is **0.58 BPB**; the rest is the artifact being evaluated on a distribution it was never trained on.
2. **That invalidates this session's experiment design**: every mechanism delta was measured on chat, where the served scorer is below uniform with 2.25 % top-1 — it *could not* have shown a mechanism effect. Stages 3 and 4 must be iterated **in domain**, where the model has signal (1.82 BPB, 25.81 % top-1) and deltas are measurable.
3. **The prize is not repairing this artifact's chat behaviour.** It is serving the mechanism that reaches 1.1199 on chat — the 8M read-stack, whose embedding is 1536-wide against this scorer's **120-root (6.9-bit) token representation**. An artifact 10× the size for a 3.27 BPB better model is the trade worth making.

## Next

1. In-domain becomes the iteration bench for Stages 3/4, with the document-clustered protocol.
2. Convert and serve the 8M read-stack multiplier-free — that is the high-value path.
3. Every comparison must state corpus, context length and byte basis: three numbers in this project's own records were unusable for want of one of them.

Scope: single artifact, 4,096 positions per row, release build, no training. Evidence: `docs/evidence/serving_gap_decomposition_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

