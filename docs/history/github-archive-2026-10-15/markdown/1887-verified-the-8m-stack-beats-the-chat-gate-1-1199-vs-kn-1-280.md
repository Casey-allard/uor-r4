# #1887 — Verified: the 8M stack beats the chat gate (1.1199 vs KN 1.280275); the served artifact is below uniform (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T09:27:02Z
- **Updated:** 2026-10-09T13:19:22Z
- **Closed:** 2026-10-08T09:28:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1887
- **Merged:** 2026-10-08T09:28:41Z
- **Merge commit:** 98eb27052877d6aa3629f5ec6e8121e7c667219e
- **Base / Head:** main / UOR-Foundation:deepseek/chat-gate-capacity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Verified: the project already beats the gate it was chasing

Sealed evaluation `~/uor-r4-local/ladder/evals/geo-8m-l2-chatv0-heldout/evaluation.json` (2026-10-03):

| field | value |
|---|---|
| model | `ladder/runs/geo-8m-l2/model/model.safetensors`, sha256 `50f8a175…` |
| corpus | `chat-v0-p2/heldout/tokens.u16`, sha256 `e5f400b0…` — the same file every measurement this session used |
| full | nll **2.2025344473015567** nats/token over 6,194,304 targets, 16,131 blocks |
| comparison | nll 2.2027294608723613 (parameter-matched control) — parity |
| `bits_per_byte` | **null** — the conversion had never been done |

With the tokenizer's own byte lengths on that stream (2.8374 B/token): **geo-8m-l2 = 1.1199 bits per byte** on real held-out chat.

## The gate, measured this session for the first time

`kn-ngram-gate` (PR #1885), chat train (20M tokens) → heldout, 399,988 positions:

| | bits/byte |
|---|---:|
| Kneser-Ney 5-gram | 1.280275 |
| **geo-8m-l2 (sealed, converted)** | **1.1199** |
| served discrete scorer | 4.211070 |
| uniform (4096 vocab @ 2.8374 B/tok) | 4.2292 |

The 8M stack beats the 5-gram by 0.16 BPB. **The served artifact is 3.09 BPB behind both — and 0.018 BPB *below uniform***, i.e. 0.98 BPB worse than an order-0 frequency table fitted on the same train split. Its 4.21 is a TinyStories→chat *transfer* number, not an architecture measurement, and every delta this session chased was measured on that floor.

## The capacity curve at fixed estimator (`--row-caps`)

| retained row caps | bits/byte |
|---|---:|
| 1,000 per order | 2.754200 |
| 20,000 per order | 1.856607 |
| 4M/4M/8M/8M (KN's own) | 1.280275 |

The artifact's only data-driven term carries ≈10,240 contexts / 12,800 candidates — at or below the smallest point. Capacity dominates the range the artifact lives in (1.47 BPB between 1k and 20k caps, versus 0.58 BPB from 20k to full KN).

## The pre-registered criterion fires

The independent methodology review set the rule in advance: *"Stop if KN-5 ≥ ~1.25 BPB — the existing 8M model already meets the gate, so the next spend must buy context/memory, not BPB."* KN-5 measures **1.280275**, the 8M model **1.1199**. **The criterion fires**, and further spend on beating this gate with a better n-gram-class scorer buys a result the project already has.

## What the evidence points at now

1. **The largest gap is the serving path**, not a gate: continuous 1.2372 vs shipped 1.8055 on TinyStories (0.5683 BPB), and trained stack 1.1199 vs served scorer 4.2111 here (3.09 BPB).
2. **The state channel is not the place to look**: an independent review measured the served 120-state as Haar-uniform with a full-lookup BPB of 3.3027 against 3.3050 for a *random* root map and 3.2909 unigram, with I(S;X_{t−L}) flat at the random null for lags 1…63.
3. **Buy context and memory**: the served scorer is 5-gram-class with a 6.9-bit side channel, while the stack reaching 1.1199 uses 255-token context. Long-context retention (plan's K-tuple, K≈16–64) and the continuous→served gap are the levers left.

**Scope**: BPB comparable only within a corpus and one byte basis; all numbers use the tokenizer's byte lengths on chat-v0-p2 held-out (sha `e5f400b0`). A document-clustered protocol (per-document delta sd 0.2148 BPB; clustered SE 0.003489 full-file) is required for any claim below 0.01 BPB — this session's retracted −0.0196 was 0.09σ under it. Evidence: `docs/evidence/chat_gate_and_capacity_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

