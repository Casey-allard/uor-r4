# Rank-margin hinge objective

**Status: ACTIVE.** The integer form is the loss that the current discrete model trains on. A floating-point form is built and tested but has never been trained. The choice between the two integer objectives (objective A and objective R, described below) is open after the 19 October runs ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

Most language models are scored by turning every possible next word into a probability and punishing surprise. Here the score is simpler. For each example, the right word has to beat its strongest wrong rivals by a fixed gap. If it does, the loss is zero and there is nothing to learn from that example. If it does not, the loss is the size of the shortfall. Every step is a comparison and a subtraction, with no probabilities and no logarithms.

## What it replaces in a transformer

The loss function. A transformer minimises cross-entropy, which needs a softmax over the whole vocabulary. This objective compares the target with a few named rivals only.

![A number line of raw scores for one example. The target is a dot, eight rival dots sit to its left and right, and a shaded band one unit wide sits below the target. Rivals inside the band contribute a red overshoot bar whose length is the hinge loss. A second panel shows the same example after learning, with every rival outside the band and no red bars.](../figures/mechanisms/rank-margin.svg)

*Hinge loss on one example: only rivals that come within the margin of the target contribute, and their contribution is the length of the overshoot.*

## How it works

**Float form (definition; training only, never trained).** Take the raw scores $z$ of the head. Fix a rival count $k$ in advance (1 or 8 in the rung-0 sheet) and let $\mathcal R_k$ be the $k$ highest-scoring non-target entries.

$$L=\frac1k\sum_{j\in\mathcal R_k}\max\big(0,\ 1-z_t+z_j\big).$$

The margin is fixed at 1. A row that already leads every rival by 1 or more contributes exactly zero gradient, and nothing is exponentiated or normalised in either direction.

**Integer form (definition; the live objective).** The writer ([discrete-learning](./discrete-learning.md)) already ranks by integer distances, so the hinge works on those. With $d$ the L1 token distance, $\mathrm{addr}$ the address distance to a cell, and $w=32$:

$$T=\max\big(0,\ w+d(\text{target})-d(\text{best rival token})\big),\qquad A=\max\big(0,\ 8+\mathrm{addr}(\text{target})-\mathrm{addr}(\text{best rival cell})\big),$$

$$\text{loss numerator}=8T+w\,A\qquad\text{over the common denominator }8w.$$

This is **objective A**. **Objective R** replaces $A$ by a rank term. Let $r$ be one plus the number of occupied cells that come before the target's cell in the order (address distance, cell number). Then

$$R=\max(0,\ r-K).$$

A crowded cell counts once, and $R=0$ exactly when the target's cell is among the $K$ kept cells ([admission](./admission.md)).

**Rules of the comparison (definition).** Which rows can count is fixed from the baseline. A missing comparison contributes zero, and the output reports how many rivals it had, so zero is never mistaken for learning. A batch with no eligible row is refused. Ties go to the smaller token or cell number.

**Consequence (proof, from the definition).** The weights 8 and $w=32$ are powers of two, so the numerator needs only adds, compares and shifts. Because the loss is a weighted sum, its total and the visible ranks can disagree: the pilot's token term fell while its address term rose by more.

**Rule check.** Compliant with D23 by construction: no exp, log-sum-exp, softplus, sigmoid or tanh. The older pooled-rank run used softplus on a log-ratio and is not compliant. It is the historical precursor only. No matrix multiplication (D30). The integer form already fits the integer serving contract (D11). The float form's export gap has never been measured, and under D37 it returns only as a new pre-registration with a fresh test-fitness review and an owner release.

## What has been tried

$K$ is the number of kept cells. "Gate" is a pre-registered pass line. "Bar (i)" is a top-1 count of 1,941 and "Bar (ii)" a top-8 count of 4,775 on the 14,287-row development panel.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-09-20 | Learned reader trained on a margin (earlier, constructed panel) | Precision +0.0987 against a bar of 0.15 | negative (that recipe and panel) | [reader result](../../archive/integration/relational-reader-result-2026-09-20.md) |
| 2026-09-24 | One learned score over admitted records, bounded margins, four arms | Complete correct answers 0 of 12 in every arm | negative (that panel) | [A4 result](../../archive/integration/integrated-attention-a4-result-2026-09-24.md) |
| 2026-10-10 | Strongest-wrong-rival pooled ranking on the older learner | Development replies 177 to 437 of 512 with no losses, but fresh rows scored 0 of 128 and the loss used softplus and softmax weighting | positive on development, then rejected (rule break) | [pooled rank](../../archive/labs/m2-pooled-rank-2026-10-10/README.md) |
| 2026-10-12 | Float hinge, 1 or 8 rivals, CPU and CUDA kernels | 938 library tests pass; CUDA parity 38 of 38; no training run | positive (construction only) | [margin objective](../../archive/labs/margin-objective-2026-10-12/README.md) |
| 2026-10-13 | Float rung-0 path repair; float rung-0 pre-registration | 18 of 18 finite controls pass; the registered run never executed | positive, then blocked | [complete path](../../archive/labs/rung0-complete-path-2026-10-13/README.md), [sheet](../harness/prep/m1-rank-margin-rung0-2026-10-13.md) |
| 2026-10-13 | Integer hinge in the discrete writer | 11 of 11 CUDA controls, including omitted-target and zero-rival cases | positive (construction only) | [discrete learning](../../archive/labs/discrete-geometric-learning-2026-10-13/README.md) |
| 2026-10-14 | Decomposition of the pilot loss | Token hinge sum $-7{,}996$ ($\times8=-63{,}968$); address hinge sum $+4{,}124$ ($\times32=+131{,}968$); total $+68{,}000$ while ranks improved | diagnostic stop | [diagnosis](../../archive/labs/a2-pilot-diagnosis-2026-10-14/README.md) |
| 2026-10-15 | Widen $K$ from 8 to 16 at evaluation | Admitted 2,400 against a gate of 4,894; the record closes only eval-time widening of a K=8-trained codebook and names the objective weights (8 times token plus 32 times address) as an open successor lever | negative (evaluation-time widening) | [Stage A sweep](../../archive/labs/rung0-stage-a-sweep-2026-10-15/README.md) |
| 2026-10-16 | Fixture runner for the six rank-hinge controls | Restart 5 passed all six, then halted at a timing count of 99 against 100 | diagnostic stop | [fixture runner](../../archive/labs/admission-rank-fixture-runner-2026-10-15/README.md) |
| 2026-10-18 | K16 qualification: start check, six controls, timing | Controls 6 of 6; untrained $K=16$ admits all 14,287 rows but only 16 cells are occupied; 260.08 and 260.10 ms per batch | positive (construction) | [K16 qualification](../labs/k16-rows-qualification-2026-10-18/README.md) |
| 2026-10-19 | K16-A and K16-R, two seeds, 16,404 batches, Route read | R: admitted 12,089 and 12,466, but 20 and 19 occupied cells, top-1 101 and 230. A: admitted 2,237 and 3,167, 209 and 192 cells, top-1 65 and 463. Bars (i) and (ii) missed by all four runs | negative (configuration); no KEEP | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |

**Reading the K16 result.** Objective R passes admission only because it herded the tokens into about 20 cells, so each row admitted about 3,400 of the 4,096 tokens. Objective A kept a healthy spread of cells but lost admission. Both runs used the 16-vector start. The result says nothing about hinge objectives that include an occupancy term.

## What has never been tested

- The float hinge as a trained model. No run has ever executed it.
- Rival sets other than the top $k$ by raw score (all rivals, random, hardest by margin), a margin other than 1, or a per-row adaptive margin.
- An occupancy-regularised rank objective, the registered next step after the K16 collapse.
- The hinge on a start with 4,096 distinct vectors, or combined with the admission-preserving acceptance rule.
- The hinge with a pointer head. The float sheet refuses a pointer today.
- Any bits-per-byte or likelihood measure. The hinge has no likelihood, so language quality under it has no agreed unit yet.

## Also known as

Margin hinge over declared rivals; `Objective::MarginHinge`; `8*token_hinge + width*address_hinge`; `NearestAddressMarginV1` (objective A); `OccupiedCellRankHingeV1` (objective R); the admission-rank objective; the D23 learning-law objective.

## Code

- [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): token margin, address margin, rank hinge R, loss terms and the overflow status.
- [`objective.rs`](../../crates/uor-r4-discrete/src/objective.rs): objectives A and R and their control scaffold. [`redirect_admission.rs`](../../crates/uor-r4-discrete/src/redirect_admission.rs): the K16 gates.
- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs) and [`cuda_stack.cu`](../../crates/uor-r4-training/src/cuda_stack.cu): the float form and its CUDA kernels (inactive).
- [`native_vector_writer.rs`](../../crates/uor-r4-training/src/native_vector_writer.rs): the float shared writer with token and address margin credit.

## Related mechanisms

[discrete-learning](./discrete-learning.md) runs the hinge as its integer loss and chooses the move. [admission](./admission.md) is the shortlist that objective R tries to protect. [address-cells](./address-cells.md) and [prime-zeta-codebook](./prime-zeta-codebook.md) change how many cells exist to rank. [flock-read](./flock-read.md) supplies the query vector being scored. [carrier-credit](./carrier-credit.md) used the same scoring head with omitted-target credit.
