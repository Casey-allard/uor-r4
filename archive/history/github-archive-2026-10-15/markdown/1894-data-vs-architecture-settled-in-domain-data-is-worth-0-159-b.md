# #1894 — Data vs architecture settled: in-domain data is worth 0.159 BPB, size 0.084 (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T11:30:12Z
- **Updated:** 2026-10-09T13:19:20Z
- **Closed:** 2026-10-08T11:31:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1894
- **Merged:** 2026-10-08T11:31:51Z
- **Merge commit:** 50ea9232f112b1f3ef772ac9074af5393aa8b896
- **Base / Head:** main / UOR-Foundation:deepseek/chat-data-control
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The confound round 10 left

The 19.9 M chat-only stack reached **0.8770 BPB** against the sealed 8M mix-trained model's **1.1199** — but size, data and recipe all differed at once, so nothing could be attributed. This closes it.

## The control

Same parameter count as the sealed model — **7,155,396, the exact figure in the sealed `geo-8m-l2` record** — same architecture (`rrarra`, width 288, heads 6, ctx 384, read L2, float), same recipe (batch 32, lr 4e-4, warmup 500, 12,207 steps = 150 M tokens), same evaluation (`snap-evaluate`, 512 windows, 196,608 targets), same held-out stream `e5f400b0…`. **The only difference from the sealed record is the training data.**

## Result

| arm | parameters | data | fused float BPB |
|---|---:|---|---:|
| sealed `geo-8m-l2` | 7,155,396 | TS/TD/chat mix | 1.1199 |
| **this control** | 7,155,396 | **chat-v0-p2 train only** | **0.9607** |
| round-10 arm | 19,929,136 | chat-v0-p2 train only | 0.8770 |

Snapped reference for the control: nll 1.962771862745285, top-1 agreement 0.8675.

## Decomposition of 1.1199 → 0.8770

```
in-domain training data (mix → chat-only, at matched size)   −0.1592 BPB
model size (7.16 M → 19.93 M, at matched data)               −0.0837 BPB
```

Both real. **The data term is about twice the size term at this scale.**

## What it settles

1. **The project's best sealed model was itself handicapped by data**, not only the served artifact: the same architecture trained on the evaluation distribution is **0.16 BPB better** than the mix-trained one the board has been quoting.
2. The 19.9 M chat-only model at **0.8770** — and **0.9550 through the multiplier-free D11 engine** — is the best measured real-text result here, and both of its advantages are now attributed.
3. **This inverts the usual instinct for scale decisions**: at this size, matching the training distribution to the evaluation one buys *twice* what doubling the parameters buys. The next spend should buy **data coverage first, size second**.

## Scope

Single seed per arm, one corpus, one held-out stream, one byte basis (2.837427 B/token), 512 windows — a claim finer than ~0.01 BPB needs round 2's document-clustered protocol (clustered SE 0.003489). Evidence: `docs/evidence/chat_data_control_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

