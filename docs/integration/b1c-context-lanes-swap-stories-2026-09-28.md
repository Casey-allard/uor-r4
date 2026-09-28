# B1 Stage C: context-conditioned finite-group lanes on natural-text swap stories

2026-09-28 · Lab 1 (Claude) · [ROADMAP](../../ROADMAP.md) track T1(b) · References #820, #973 · PR #1447

**Status: PARKED at this scale on pilot evidence.**
- The pre-registered grid is **NOT_RUN** (owner decision, 2026-09-28; [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863149284)).
- In three pilots, no arm learned to track swaps in natural text. That includes the lane-free stack and, with dense per-token state supervision, every arm tested.
- Context-conditioned lanes also cost +0.13 to +0.20 nats of development NLL.
- The pre-registered kill rule is **not** formally triggered, because its grid did not run.
- Nothing here is a language-capability result.
- The evidence is in [`docs/evidence/b1c-context-lanes-swap-stories-2026-09-28.json`](../evidence/b1c-context-lanes-swap-stories-2026-09-28.json).

Labels:
- **Measured:** Rust runs on the owner's M1 at 2 threads per run.
- **Derived:** arithmetic from measured records.
- **Pre-registered:** fixed on #973 before the run it governs.

## 0. Findings

1. **At width 128 and 600 updates, no arm learns natural-language swap tracking** (*Measured*, pilot 3, §3).
   - With a per-token who-holds-what head, every arm learns the "nobody swapped" prior within 50 updates, and then the state loss stops falling. For three-event stories it is 0.833 at update 50 and 0.836 at update 200.
   - After a single swap, per-person state accuracy is 0.50 and exact-state accuracy 0.00–0.04. That is exactly the no-swap prediction.
   - The failure is not lane-specific. The lane-free stack fails the same way.
2. **Context-conditioned lanes cost text quality** (*Measured*, pilots 2 and 3).
   - Development NLL rises by +0.195 (pilot 2, 32 windows), and by +0.128 and +0.162 (pilot 3, 256 windows, lane learning rates 0.03 and 0.01).
   - The pre-registered text gate allows 0.05.
   - Lowering the lane learning rate did not reduce the cost, so the pilots do not support "the lane optimiser is too aggressive" as its cause.
3. **The context lanes rotate at almost every token, not once per swap** (*Measured*, pilot 3 transport witness: 32 stories at E=16, lane learning rate 0.03).
   - Per lane, 6.4–8.7 steps per event have a transport that is clearly not the identity (trace below 3). An event-driven swap automaton would move about once per event.
   - Mean traces are 0.49–2.01, against 4 for the identity. Of the moves, 0.16–0.66 are near half-turns.
   - The lanes therefore write a dense rotating signal into the residual stream, and do not implement sparse transpositions. That is consistent with both the failure to track and the text cost.
4. **Sparse answer supervision is a failed instrument at this scale** (pilots 1 and 2). About 64 answer targets per update taught the answer format, but not the state.

**What stays open.** Whether context-conditioned finite-group lanes carry exact state through natural-language transitions is **untested**. The pilots show only that this testbed does not produce learning in any arm. Reopening needs an instrument on which some arm learns first, for example stories without the text objective, or a curriculum held at one or two events.

## 1. Question and pre-registration

**Question.** B1 Stage B showed learned reflection-pair lanes tracking synthetic A5 words inside the stack (an exploratory result; a fresh replication is pre-registered on #973). Those lanes choose their transport from the current token alone. In "Mia and Leo swapped.", no single token determines the swap. Stage C asked whether lanes whose transport is conditioned on the stack's hidden state can carry exact who-holds-what state through such transitions.

**Record** (*Pre-registered*, [#973 work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5861915531)):
- Amendment 1 added a matched transformer control.
- **Amendment 2** ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5862609103)) came after pilots 1 and 2. It set a fixed cast, zero-event stories and a per-token state head for every arm.

**Planned arms** (3 seeds each): the lane-free stack; token-conditioned reflection-pair lanes (a control expected to fail); context-conditioned reflection-pair lanes; a matched transformer.

**Gates** (per seed):
- Tracking: context lanes reach first-answer accuracy ≥0.95 at E=16, and at least 0.20 above the lane-free stack.
- Text: context lanes' development NLL is within 0.05 nats of the lane-free stack.
- Kill rule: if the tracking gate fails in 2 or more seeds, context-conditioned lanes are parked at this scale.

## 2. Setup (*Measured* configuration)

**Swap stories.**
- English episodes over single-token names and objects of the #1017 tokenizer, built from the pieces " has a", " and", " swapped", ".", " Now" and " has the".
- Four people each hold one object, and E swap events follow ("Mia and Leo swapped."), with each pair drawn uniformly.
- Then comes one query per person in random order ("Now Sam has the hat.").
- The **first answer** of each story is the elimination-free measure. Later answers can be narrowed down by excluding objects already named.
- The event cap ramps from 2 to 16 over the first 60% of updates. Training draws E uniformly from 1 to the cap in pilots 1–2, and from 0 to the cap in pilot 3.

**Stack.**
- Geometric `rrar`, width 128, 4 heads, MLP 384, context 256.
- The transformer control uses pattern `aaaa` at the same size.

**Context lanes.**
- 8 reflection-pair lanes read the residual stream after layer 2 through an affine map: bias initialised to the identity transport, W std 0.01.
- They write back before layer 3 through a zero-initialised projection.

**State head** (pilot 3). A linear map from the final hidden state predicts at every story token which object each person holds, as 4 × 4 classes, with weight 1. A swap takes effect at its sentence's period.

**Training.**
- Each update has 16 text windows of 256 tokens from `train.u16` and 16 stories.
- AdamW, learning rate 0.003 with 100 warm-up updates and a cosine schedule to 10%, weight decay 0.1. The lane learning rate is 0.03 unless stated.
- The data stream depends only on the seed (`data_seed` = 3000 + seed), so every arm at a given seed sees the same batches.

**Builds.**
- Pilot 3 ran on `c13d0d2c` (binary SHA-256 `6e2b0e92…`).
- Pilots 1 and 2 ran on working-tree builds whose binaries were not hashed. Pilot 1 matches `76ef2c1a`, apart from serving-only changes. Pilot 2 already had the fixed cast that `c13d0d2c` later committed. All three pilots are exploratory.

## 3. Pilots (*Measured*, exploratory, 1 seed each)

| Pilot | Design | Updates | Evaluation |
|---|---|---|---|
| 1 | random cast, answer loss only | 300 | 64 stories per E, 32 dev windows |
| 2 | fixed cast, answer loss only | 600 | 64 stories per E, 32 dev windows |
| 3 | fixed cast + per-token state head | 600 | 128 stories per E, 256 dev windows |

**First-answer accuracy** (chance is 0.25 among a story's four objects, and 0.125 over all eight in pilot 1's random cast):

| Pilot | Arm | Dev NLL | E=1 | E=4 | E=8 | E=16 |
|---|---|---|---|---|---|---|
| 1 | lane-free | 3.2247 | — | 0.109 | 0.203 | 0.172 |
| 1 | context lanes | 3.4660 | — | 0.062 | 0.125 | 0.109 |
| 1 | transformer | 3.5166 | — | 0.062 | 0.125 | 0.109 |
| 2 | lane-free | 2.9482 | 0.312 | 0.156 | 0.344 | 0.250 |
| 2 | context lanes | 3.1432 | 0.406 | 0.172 | 0.375 | 0.219 |
| 2 | transformer | 3.1111 | 0.406 | 0.250 | 0.266 | 0.297 |
| 3 | lane-free | 2.9835 | 0.453 | 0.242 | 0.250 | 0.344 |
| 3 | context lanes, lane LR 0.03 | 3.1113 | 0.461 | 0.320 | 0.180 | 0.273 |
| 3 | context lanes, lane LR 0.01 | 3.1451 | 0.391 | 0.227 | 0.234 | 0.305 |

**Pilot 3 state head, at the last input position:**

| Arm | Per-person E=1 | Per-person E=16 | Exact E=1 | Exact E=16 |
|---|---|---|---|---|
| lane-free | 0.496 | 0.365 | 0.00 | 0.03 |
| context lanes, lane LR 0.03 | 0.502 | 0.283 | 0.00 | 0.02 |
| context lanes, lane LR 0.01 | 0.547 | 0.314 | 0.04 | 0.04 |

**State-loss trajectory** (pilot 3, lane-free; the context arms stay within 0.04 of it at every logged update):
- The state loss falls from 1.417 (about ln 4) to 0.833 by update 50. After that it only follows the number of events in each batch: 0.64 at E=1, 0.83–0.87 at E=3–4, 0.90–0.94 at E=5–7, and 0.98–1.11 at E=8–15. It does not improve at a fixed E.
- The answer loss plateaus at 0.77–0.90 in every arm from update 150 on. That is consistent with answering by elimination alone.

In pilot 1, the context and transformer arms have identical accuracies. That is consistent with both predicting the same constant object on the same evaluation stories; the predictions were not saved.

## 4. Decision and scope

- **Owner decision** ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863149284)): the Stage C grid is NOT_RUN, and context-conditioned lanes are parked at this scale.
- The compute went instead to closing B1: a fresh pre-registered replication of reflection-pair lanes, the Stage B transformer control, a same-binary anchor and the Stage A rerun with `verify_exact`. Results are in the [B1 record](b1-finite-group-lanes-2026-09-27.md).
- The Stage C harness stays in the repository as infrastructure: context lanes, swap stories, the state head and the `tracking-lanes stories` driver.
- **Not claimed:**
  - that context-conditioned lanes fail at state tracking;
  - that the stack cannot learn it at larger width or with more updates;
  - any language capability.
