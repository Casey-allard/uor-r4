# #1913 — Size at 10×: the 214M chat stack (1.0966) loses to a 7.16M chat-only model (0.9607) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:18:39Z
- **Updated:** 2026-10-09T13:22:20Z
- **Closed:** 2026-10-08T14:20:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1913
- **Merged:** 2026-10-08T14:20:05Z
- **Merge commit:** 820612f1c15cc24fb035542f9a49d4cb615ca2ad
- **Base / Head:** main / UOR-Foundation:deepseek/size-ten-x
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The four-hour question, answered

The store's chat-trained **214 M** stack (`rrarrarrarrarrar`, width 1536, read L2, pointer model) scored under the repository's own sealed `evaluate` protocol on `chat-v0-p2/heldout` (sha `e5f400b0…`):

| | nll | targets | bits/byte |
|---|---:|---:|---:|
| **full** | **2.1568440326982325** | 6,194,304 | **1.09655** |
| tune | 1.9492776837467096 | 24,576 | 0.99103 |

## The comparison

| model | parameters | training data | bits/byte |
|---|---:|---|---:|
| served artifact today | 2.7 MB | TinyStories | 4.3885 |
| Kneser-Ney 5-gram | — | chat-v0-p2 train | 1.2803 |
| sealed `geo-8m-l2` | 7,155,396 | TS/TD/chat mix | 1.1199 |
| **`chat-214m-smoltalk`** | **~214 M** | SmolTalk-derived chat | **1.0966** |
| control (round 12) | 7,155,396 | **chat-v0-p2 only** | **0.9607** |
| round-16 arm | 19,929,136 | **chat-v0-p2 only** | **0.8770** |

**Ten times the parameters with a different chat mixture loses by 0.1359 BPB to a model one-thirtieth its size trained on this corpus**, and by 0.2196 BPB to the 19.9 M model on the same data.

## Why it matters

Round 12 measured the data term at **twice** the size term (0.1592 vs 0.0837 BPB) across a 7→20 M range. **This extends it an order of magnitude and finds the ordering unchanged, with a larger margin.** Within this project's corpora and at this scale, matching the training distribution beats adding parameters by a wide margin.

It also means **the project's largest chat model is not its best chat model** — and that's a fact about training data, not architecture. The same family at 19.9 M on `chat-v0-p2` beats it.

## Caveats, stated because the protocols differ

The 214 M row is a full-stream sealed evaluation over 6,194,304 targets; the smaller rows are float scores over 512 windows. **They are not on one identical protocol** — the *ordering* is the finding, and it is large relative to any protocol difference measured this session. The 214 M model is a pointer model, which is why it needed `evaluate` rather than `snap-evaluate`. Training mixtures come from the store manifests.

## Consequence

**The data lever is the one worth unblocking — and it is blocked**: no chat corpus larger than the 82 M-token `chat-v0-p2` train is reachable in the store or on the canonical volume. This is the strongest argument yet for spending the next effort there rather than on a larger rung of the same mixture. Evidence: `docs/evidence/size_at_ten_x_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

