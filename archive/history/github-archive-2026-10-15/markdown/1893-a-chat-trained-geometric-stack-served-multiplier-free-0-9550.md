# #1893 — A chat-trained geometric stack served multiplier-free: 0.9550 bits per byte vs 4.3885 today (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T11:09:17Z
- **Updated:** 2026-10-09T13:19:27Z
- **Closed:** 2026-10-08T11:10:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1893
- **Merged:** 2026-10-08T11:10:55Z
- **Merge commit:** addb2a7e4d76b7916ec68084798cca415e839f0f
- **Base / Head:** main / UOR-Foundation:deepseek/chat-served
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this lands

One end-to-end chain on one RTX 5090, chat distribution, ~15 minutes of training: **train → snap-evaluate → export → d11-evaluate → lut-evaluate**. Held-out stream sha256 `e5f400b0…` — the same file the sealed 8M evaluation used — byte basis 2.837427 B/token.

| stage | measurement | bits/byte |
|---|---|---:|
| train 12,207 steps, chat-only | dev nll 1.9008 at step 8,000 | — |
| snap, **fused float reference** | nll 1.7250754982233047 (196,608 targets) | **0.8770** |
| snap, transport-snapped | nll 1.7975700236856937 | 0.9138 |
| export, round-to-nearest | `model.lut` **11,891,652 bytes**, sha256 `2ac4c306…` | — |
| **d11, multiplier-free engine** | **d10 = d11 = 1.8345338586523747** (24,576 targets) | **0.9550** |
| | `d11_minus_d10_nll` **0.0**, `top1_agreement` **1.0**, `max_abs_logit_difference` **0** | |
| lut, artifact vs float parent | `top1_agreement` 0.9294 | |

Model: `rrarrarr`, width 512, heads 8, ctx 384, read L2, float, 19,929,136 parameters, chat-v0-p2 train only, 164,277 tokens/s.

## Where it sits — same corpus, same byte basis

```
served artifact today (120-root, TinyStories-fit)     4.3885
Kneser-Ney 5-gram                                     1.2803
sealed geo-8m-l2 (TS/TD/chat mix)                     1.1199
THIS RUN, served by the multiplier-free D11 engine    0.9550
THIS RUN, fused float                                 0.8770
```

**4.6× better than what the project serves today**, and the engine reproduces its reference **exactly** — zero nll difference, every top-1 identical, not one logit differing. Round 5's bit-exactness evidence therefore holds on a second, independently trained artifact on a different distribution.

## What it settles

The barrier this session spent nine rounds locating was never the mechanism, the data, or the serving kernel. **It was that the pipeline had never been run end to end**: the project trained models it did not serve (`geo-8m-l2` at 1.1199 idle on disk), served a model it never trained for the distribution in front of it (2.7 MB, 120 roots, TinyStories-fit, 4.3885 on chat), and had already sealed a record proving its integer engine is exact.

It also retro-explains every null tonight: each mechanism delta was measured on a model **below uniform** on that corpus (top-1 2.25 %) in a 120-root feature space, while a trained stack with a 1536-wide-class embedding sat idle.

## Honest scope

Single seed. **Chat-only training**, so part of the gain over the sealed mix-trained model is in-domain data rather than architecture — not separated here. d11 covered 24,576 targets, snap 196,608, the sealed 8M 6,194,304; a claim finer than ~0.01 BPB needs round 2's document-clustered protocol (clustered SE 0.003489). The export is round-to-nearest — GPTQ measured 0.9379 agreement on the earlier artifact, so quantisation choice is a live lever. And the D11 engine runs at **61 tokens/s on CPU**: correctness established, throughput not.

## Next three

1. Re-score this artifact and the sealed 8M on **one common position list** with the clustered protocol.
2. **Separate in-domain data from architecture** — same stack on the sealed mix, same data on a larger rung.
3. **Raise served fidelity**: 0.9550 served vs 0.8770 fused is quantisation and snapping — levers, not limits.

Evidence: `docs/evidence/chat_stack_served_d11_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

