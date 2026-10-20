# Live geometric carrier credit

**Status: PARKED.** The forms that were run either failed or broke a project rule. Three fixed-length runs fell from 437 of 512 correct replies on the development panel (a score that was itself recorded as panel fitting: 0 of 128 on fresh rows) to 0 of 512, the September pilots failed, and the sampled-choice credit used a softmax, which the rules ban ([unified integration record](../../archive/labs/unified-geometric-integration-2026-10-11/README.md)). One part of the idea survives: a single shared token table, now live in [discrete learning](./discrete-learning.md).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

When a model reads a memory and then writes an answer, the lesson from a mistake should reach the pieces that actually carried the information, not only the last few scoring numbers. This idea pushes the error signal back through those carriers: the geometric objects that hold the read, and the prototypes the answer is chosen from. A later version also ties input, memory and output to one shared table of vectors. The runs so far have not shown a benefit. The arm with carrier credit and the arms without it all lost every answer the starting model had right, so the runs cannot blame the credit rule or clear it.

## What it replaces in a transformer

Backpropagation's job of blame assignment. In a transformer, the error flows back through every layer by the chain rule. Here the path is a chain of hard choices (which memory, which key, which answer) and geometric carriers, so the credit has to be designed rather than inherited.

![A pipeline strip from left to right: query, compiler spans, exact store, E8 index, H4 carriers, Generate prototypes, answer. Two arrows run backwards beneath it. The upper arrow, labelled frozen-carrier fit, reaches only the last scoring coefficients. The lower arrow, labelled unified fit, reaches the H4 carriers and the prototypes as well. The layers that changed in the executed runs are shaded.](../figures/mechanisms/carrier-credit.svg)

*Where the error signal lands: the frozen-carrier fit changes only the final coefficients, while the unified fit also moves the carriers and prototypes.*

## How it works

**September form: cost-to-go credit (definition).** The addressed-attention model samples four hard trajectories, each a chain of decisions (read, key, emit). The old estimator credited every parameter event with the loss of the whole trajectory. The new one credits an event with the cost that follows it, measured from its next cross-entropy boundary, minus a baseline taken from the other three trajectories. Schematically, for decision $t$ of trajectory $i$,

$$\hat g^{(i)}_t=\Big(G^{(i)}_t-\bar G^{(-i)}_t\Big)\,s^{(i)}_t,\qquad G^{(i)}_t=\sum_{u\ge t}c^{(i)}_u,$$

with $c$ the per-boundary cost and $s$ the recorded score of the parameter event (the exact normalisation is in the source).

**October form: unified credit (definition).** The saved native model (called saved437) scores relations between H4 elements (the 120 vertices of the 600-cell, a four-dimensional shape) and *Generate prototypes*. The unified fit computes independent conditional utilities over all 120 H4 elements. It restores the saved fractional masters, whose served form is quarter-step codes $q/4$ with $q\in\{-7,\dots,7\}$. It then fits the context carriers, the prototypes and the coefficients together on the pooled-ranking plus end-of-reply objective. A lane reads at most 120 encountered prototypes.

**Whole-path design (hypothesis).** Extend credit to compiler spans, exact store writes, E8 root choices and consumed donors through an explicit graph of hard decisions. Exact token, occurrence and version identity stay separate from the geometric summaries.

**Shared writer (definition).** One learned vector table encodes the input token, ranks the output and carries the compiler/store/emitter channel. Tokens get a product cell from the root numbers of their first and last 8-dimensional block, and a target omitted from the shortlist still receives score credit.

**Rule check.** The sampled-choice credit used by the whole-path producer uses softmax, so it is blocked by D23, and the pooled-ranking loss used softplus. The shared-writer margin form and the integer writer are compliant. No dense matrix multiplication is needed (D30): the finite factors use table reads and quarter-step sums, and 31 vendor-boundary controls passed for the writer. The served M2 form is already integer (D11), but the export gap of the unified form was never measured, and every position still scores all 4,096 answer choices, so per-token access is not sparse. D37 allows a float re-registration with a fresh review, an owner release, a scanned module and a reported export gap.

## What has been tried

"Complete correct" means a reply that matches the reference in full on the 512-row development panel. CE is cross-entropy; rising CE is worse. "Conditional" counts correct positions in one 15-position episode. All records are archived.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-09-13 | Addressed attention, four trajectories, 64 updates, whole-trajectory credit | No useful generation; development exact answers 0 of 8; CE 5.5530 to 5.5516 | negative | [pilot](../../archive/native_geometric_addressed_attention_pilot_973.md) |
| 2026-09-13 | Cost-to-go credit against the old estimator, frozen endpoints | Gate-gradient covariance ratio 0.526 (down 47.4%); every family improved; model gate still failed | positive (estimator only) | [causal credit](../../archive/native_geometric_addressed_attention_causal_credit_973.md) |
| 2026-09-13 | Same 64-update recipe with cost-to-go credit | Training CE slightly better, development CE worse (5.5710 against 5.5530), 0 of 16 exact | negative | [causal pilot](../../archive/native_geometric_addressed_attention_causal_pilot_973.md) |
| 2026-10-09 | Learn only the Generate pair table (960 coefficients) | Conditional 5 to 6 of 15; CE 5.1115 to 4.0429; accepted model stays 8 of 512 | inconclusive | [bounded pair](../../archive/labs/bounded-pair-run-2026-10-09/README.md) |
| 2026-10-09 | Complete donor utility on 1,920 coefficients; a bound on pair changes; a search at the earliest error | 9 of 15, as an older negative; target's best case below the rival's worst case at all nine reference points; 0 of 13,440 alternatives fixed the error | negative | [full donor](../../archive/labs/full-donor-run-2026-10-09/README.md), [bound](../../archive/labs/pair-copy-bound-2026-10-09/README.md), [attribution](../../archive/labs/pair-proposal-attribution-2026-10-09/README.md) |
| 2026-10-11 | Bind compiler, store and reader into one session, source only | Source kept; no training; panel numbers unchanged | inconclusive | [bound content](../../archive/labs/native-bound-content-2026-10-11/README.md) |
| 2026-10-11 | Matched carrier credit ON against OFF, 128 updates, seeds 1001 and 2001 | ON1001, OFF1001, OFF2001 all 437 to 0 of 512; CE 2.1828 to 7.7924, 7.7373, 7.7373. ON2001 stopped by the owner before its optimiser started | inconclusive | [unified integration](../../archive/labs/unified-geometric-integration-2026-10-11/README.md) |
| 2026-10-12 | Whole-path credit reference and CUDA-shaped producers | 15 and then 28 CPU fixtures pass; CUDA never run; sampled producer blocked by D23 | blocked | [implementation](../../archive/labs/whole-path-implementation-2026-10-12/README.md), [recovery](../../archive/labs/whole-path-recovery-2026-10-12/README.md) |
| 2026-10-13 | Float shared writer with margin credit | 5 controls pass; 31 of 31 at the vendor boundary (infrastructure only) | positive (construction) | [shared writer](../../archive/labs/geometric-head-shared-writer-2026-10-13/README.md) |
| 2026-10-13 | Integer shared writer carrying tokens, with the Flock read on the same table | 17 of 17 controls pass, including shared-parameter identity; this is the surviving form | positive (construction) | [flock writer](../../archive/labs/discrete-flock-writer-integration-2026-10-13/README.md) |

**Reading the 437-to-0 collapse.** The ON arm lost every saved success, but so did both OFF arms, so these runs do not isolate carrier credit as the cause. The [unified-integration audit](../../archive/labs/unified-geometric-integration-2026-10-11/AUDIT.md) lists the risks without naming a cause: a fixed 128 updates, a frozen readout while carriers change, mixed finite and ambient credit, and a component-only panel. That panel reuses 128 banks, 12 query strings and 48 exact answers for 512 inputs (same audit). Nothing was promoted and no mechanism was closed.

## What has never been tested

- Unified credit with one coherent typed binding. The executed runs kept the readout frozen while the carriers and prototypes moved.
- The fourth arm (ON, seed 2001), so the replicated ON-against-OFF contrast is incomplete.
- The opened 128-row transfer check and a truly fresh 128-row draw for any carrier-credit candidate.
- The whole-path credit on a GPU, and a credit estimator that avoids softmax.
- A compiler, exact store and E8 admission trained together on a realistic, non-component panel, with joint credit for spans, memory writes and root choices.
- A carrier-credit comparison on the discrete path, with the shared table moved by counterfactual moves.

## What would reopen it

1. A rule-compliant replacement for the sampled-choice credit, such as hard selection by address or integer counterfactual moves, with the credit path checked end to end.
2. One coherent typed binding, frame and native credit boundary, qualified first, so carriers and prototypes change under a readout trained with them.
3. A panel that is not component-only, with at least two seeds and a fresh draw of at least 128 rows.
4. A matched ON against OFF comparison with identical active parameters, all 4,096 answer choices and physical copy aliases scored, and carriers rebuilt after each update.
5. A D37 float re-registration (fresh review, owner release, scanned module, export gap against the D11 form) or an integer counterfactual form on the discrete path.
6. Owner release of the separate hold on training and rental for milestone M2 (answering from memory; see the [milestones](../start/glossary.md)).

## Also known as

Unified geometric carrier credit; live H4/Generate prototype credit; whole-path learning; the shared native vector writer (float form); causal cost-to-go credit (the September addressed-attention form); donor utility credit.

## Code

- [`causal_credit.rs`](../../crates/uor-r4-core/src/native_geometric/addressed_attention/causal_credit.rs): the September cost-to-go estimator.
- [`geometric_continuation_learning.rs`](../../crates/uor-r4-training/src/geometric_continuation_learning.rs) and [`geometric_bank_generate.rs`](../../crates/uor-r4-training/src/geometric_bank_generate.rs): conditional utilities and live carriers. [`unified_geometry.rs`](../../crates/uor-r4-training/examples/geometric_frozen_map_fit/unified_geometry.rs): the executed ON and OFF arms.
- [`whole_path_learning.rs`](../../crates/uor-r4-training/src/whole_path_learning.rs) and [`whole_path_device.rs`](../../crates/uor-r4-training/src/whole_path_device.rs): the CPU reference and the never-run CUDA-shaped producers.
- [`native_vector_writer.rs`](../../crates/uor-r4-training/src/native_vector_writer.rs) and [`native_vector_channel.rs`](../../crates/uor-r4-training/src/native_vector_channel.rs): the float shared writer and channel.

## Related mechanisms

[discrete-learning](./discrete-learning.md) carries the single shared table forward in integer form. [rank-margin](./rank-margin.md) supplies the omitted-target and address credit inside the writer. [legal-construction](./legal-construction.md) ran on the same learner and panels. [e8-orbit-memory](./e8-orbit-memory.md) and [spiralcore-fbs](./spiralcore-fbs.md) supply the E8 admission and transport that whole-path credit was meant to train. [exact-memory](./exact-memory.md) is the occurrence store whose identity must stay separate. [read-binding](./read-binding.md) is a teacher-side signal that would train which value a read binds to.
