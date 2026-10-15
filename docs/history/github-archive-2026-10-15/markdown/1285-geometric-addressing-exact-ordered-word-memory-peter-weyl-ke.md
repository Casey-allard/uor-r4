# #1285 — Geometric addressing: exact ordered-word memory, Peter-Weyl kernel, route abstention (rounds 2-7)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T19:57:32Z
- **Updated:** 2026-09-19T20:42:50Z
- **Closed:** 2026-09-19T20:09:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1285
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/geometric-addressing-and-harmonic-kernel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this delivers

Rounds 2-7 of the geometric-core investigation, which were stranded on the head branch of the already-merged #1284. Main received that PR as a **squash**, so later commits to the same branch never reached main. Content check against `origin/main`: `lowbit_attention.rs`, `geometric_attention.rs` and the four receipts are absent from main; `lowbit_core.rs` (with `LowBitCoreTrainer`) and `chat.rs` are present.

**Files:** 13 changed, +5,342/-17.

| File | What |
|---|---|
| `learner/lowbit_attention.rs` | content-addressed linear-attention core + trainer |
| `learner/geometric_attention.rs` | exact addressed memory, ordered-word addressing, Peter-Weyl kernel, route abstention |
| `learner/lowbit_core.rs` | recurrent decay shift; shared quantiser/optimiser helpers |
| 4 receipts + design doc | `docs/evidence/native_geometric_*_2026-09-19.txt`, design doc |
| records | `current-state.md`, `EVIDENCE.md`, `resource-ledger-2026-09-19.md`, SpiralCore extract |

## Measured findings (held out, deterministic seeds)

**Falsified**
- Dense low-bit recurrence for memory: `i32` saturation in **16-32 steps**; burial at delay >= 8; a recurrent right-shift decay is falsified (1.00 -> 0.16 -> 0.00).
- `relu` after an **exact-address** read destroys the retrieved value's sign half (0.31 -> 0.64 when removed).
- Learned harmonic filter on clean data: loses to a fixed exact filter (0.27 vs 0.42) - robustness is not in the objective.
- Five readout levers flat on the relational ceiling: weight precision, training budget, read width, filter capacity, decode geometry (bias).

**Works**
- **Ordered-word addressing**: duplicated-key recovery 0.14 -> **0.98**; context copy **1.00** at lengths 4/8/16 vs linear attention 0.05/0.03/0.00 and vs order-1 0.86/0.64/0.47.
- **Peter-Weyl grounding**: 2I has **9 conjugacy classes** `[1,1,12,12,12,12,20,20,30]`, computed from the verified table with conjugation-invariance checked over all 14,400 pairs - so a class-function kernel is 9 ternary weights, the maximally compact rotation-invariant kernel the group admits.
- **Composition vs lookup on unseen relational facts: 0.42 vs 0.00** - the first result where the group structure is load-bearing.
- **Open/closed route abstention**: unseen queries become `0 ok / 64 abstain / 0 wrong`, seen accuracy unchanged - hallucination becomes honest abstention, with eviction proved exactly (never written).

## Honest limits

No chat capability. Not a language model: one layer, synthetic tasks. Addresses are fixed, not learned. `serving_multiplier_check.py` has not been run on the new modules, so no served-binary multiplier claim is made for them (the construction is multiplier-free and the integer path is verified equal to an `f64` reference). Energy/token remains UNAVAILABLE on this machine. Six defects were caught by tests during this work; four initially looked like mechanism results.

## Ledger

Charged cumulatively to `.uor-models/native-joint-learning-2026-09-04/model-time.json`; current **145,888,565 / 147,200,000 ms** (~21.9 min remaining). No extension used. No destructive deletion; the 128 MiB storage stop margin is untouched.

## Why this is a new PR rather than a comment on #1284

#1284 is MERGED; pushing to its head branch does not reopen it, so those comments were landing on a closed PR. Per `AGENTS.md` I am not self-merging to main - this PR is ready for your review or your explicit go-ahead to merge.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5744955643 — 2026-09-19T20:09:41Z — Casey-allard

Superseded by #1286. This PR was created from a branch whose history predates the squash-merge of #1284, so GitHub saw the already-merged stage-1 content as a conflict. #1286 carries the identical content rebased cleanly onto current main (14 files, +5,480/-17, no conflicts).


---
_Comment/review ids in this file: 5744955643_

