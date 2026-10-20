# Discrete geometric learning (exact-move counterfactual training)

**Status: ACTIVE.** It is the learning rule of the only live model path, `crates/uor-r4-discrete`. It has trained a short pilot, a full K=8 dose, the four K16 runs (runs with a shortlist of K=16 cells; see the [glossary](../start/glossary.md)) and the Relation runs that stopped at the first checkpoint. None has shown a language gain or beaten its frozen control, and training stays held until the owner releases each run ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

The model is a big table of small whole-number vectors, one per word, plus a few routing choices. To teach it, the machine picks one table entry and tries every allowed rotation of it, 16 options counting "leave it alone". For each option it replays a batch of training examples and counts the mistakes with whole-number arithmetic. It keeps the best option only if that is strictly better than leaving the entry alone. There are no gradients, no decimals and no matrix multiplication. Every step stays inside the values the finished model can hold. So far the method has moved some counts but has not produced better language.

## What it replaces in a transformer

The optimiser. A transformer is trained by backpropagation: it computes a smooth slope for every weight and nudges all of them a little. Here nothing is smooth. Learning is a search over a short list of legal moves for one coordinate at a time, scored by a replay of real examples.

![A loop drawn around one table entry. Step 1, counterfactual: the entry's 16 legal states (the current state plus 15 rotations) are laid out as a 4 by 4 grid of cells, each with an integer loss bar below it. Step 2, choose: the lowest bar is lit, with ties going to staying put. Step 3, apply: the entry moves to the lit state only if its bar is strictly lower than the stay-put bar. Step 4, validate: the stored table is re-checked on the device. An arrow returns to step 1 for the next coordinate.](../figures/mechanisms/discrete-learning.svg)

*One learning step on one table entry: try all 16 legal states, pick the cheapest, commit only on strict improvement, then check the stored result.*

## How it works

**State (definition).** A shared integer codebook maps each of the $V=4096$ tokens to $B=4$ blocks of 8 integers. Each block is one of 16 *roots* of a single orbit of the E8 lattice (a fixed set of 16 integer vectors; not the full set of 240). The model also holds, for each routing stage, one of 64 signed-axis rotations of 8-dimensional space (the SpiralCore group), and optional gates that take the values $-1$, $0$ or $+1$. Each of these is a *coordinate*: a vector block, a route frame or a gate.

**Read and write (definition).** A [read](./route-read.md) turns the token window into an integer vector $q$. A shared writer gives each token a *cell* from the root numbers of its first and last block ($16\times16=256$ cells), keeps the $K$ nearest occupied cells to $q$ by L1 distance ([admission](./admission.md)), and picks the nearest token inside them. The same table encodes input, carries memory and ranks output.

**Loss (definition).** The loss is the integer [rank-margin hinge](./rank-margin.md). Per row, with $T$ the token hinge and $A$ the address hinge:

$$\ell = \frac{8\,T + w\,A}{8w},\qquad w = 8B = 32 \text{ in the lineage of record.}$$

**Learning step (definition).** For one coordinate $\theta$ and a batch of rows $\mathcal R$ whose eligibility is fixed by the baseline:

$$L(m)=\sum_{x\in\mathcal R}\ell\big(x;\ \theta\leftarrow m\big),\qquad m\in\{\mathrm{id},m_1,\dots,m_{15}\}.$$

The 15 non-identity moves are bivector rotations. The step picks $m^*=\arg\min_m L(m)$. Ties go to the identity, then to the lowest resulting port number, then to the lowest enumeration index. It applies $m^*$ only if $L(m^*)<L(\mathrm{id})$, then re-validates the stored table on the GPU. Nothing is sampled, and no floating-point number appears anywhere in the step.

**Consequence (proof, from the definition).** Every committed move strictly lowers the loss *on its own batch*, and the new state is always a legal state. The pilot's receipt audit confirmed this on every accepted move (**measurement**, [pilot record](../../archive/labs/a2-pilot-2026-10-14/README.md)). The step says nothing about rows outside the batch. A move that helps its batch could push other targets out of the shortlist; that is a hypothesis consistent with the Q1 stops below, and causal attribution was not made.

**Speed-ups.** The row-parallel kernel (`rows_v1`) computes the baseline once per row and each candidate once per row and lane, instead of replaying every row serially per candidate. It reproduces the serial losses on every checked case (**measurement**: Route read 3.843 s to 0.0691 s per proposal, 55.6 times faster, 64 rows; Route-mode index coverage was widened after review). BVG-K (batched verified coordinates) scores up to 64 coordinates per launch and commits a block only if its joint loss beats the best single move.

**Rule check.** No softmax, exp, softplus, sigmoid or tanh (D23), and no matrix multiplication (D30): the choice is an integer minimum, distances are integer L1 and Hamming, and moves are table lookups. The snapshot is integer, but it has not been exported or audited against the D11 serving form, no serving engine for it exists and no export gap or tokens per second has been measured. D37 allows offline float training; that would need a separate scanned module and a reported export gap.

## What has been tried

$P_{succ}$ is the 3,847-row copy-successor sub-panel (see [relation-read](./relation-read.md)). "Q1" is the checkpoint at 4,096 batches. "Admitted" counts rows whose target cell is in the shortlist. A "construction control" tests that code does what it says; it is not a learning result.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-09-12 to 10-04 | Five earlier learners on the pre-discrete engine (one-use credit, tied sampler, native capture and route fits) | 0 of 8 replacements improved loss; complete construction 79 to 67; 244 to 83 of 256; about 15,650 to 15,700 coefficients moved and the parent kept | negative (older engine) | [credit](../../archive/native_geometric_credit_assignment_973.md), [tied](../../archive/native_geometric_tied_learning_973.md), [held credit](../../archive/integration/geometric-native-held-credit-2026-10-02.md), [route fit](../../archive/integration/geometric-native-route-fit-2026-10-04.md) |
| 2026-10-13 | 11 CUDA construction controls on the first discrete source; then 17 with the Flock read and gates added | All pass; a virtual improvement 640 to 544 is witnessed; no move applied | positive (construction only) | [discrete learning](../../archive/labs/discrete-geometric-learning-2026-10-13/README.md), [flock writer](../../archive/labs/discrete-flock-writer-integration-2026-10-13/README.md) |
| 2026-10-14 | Pilot: seed 1, 1,640 batches, vector blocks only | Top-8 hits 173 to 1,169, top-1 17 to 82, yet loss rose 13,816,032 to 13,884,032; 236 accepted updates; below the count-table reference | inconclusive | [pilot](../../archive/labs/a2-pilot-2026-10-14/README.md) |
| 2026-10-14 | Diagnosis of the pilot | Start codebook has only 16 distinct vectors; admitted targets fell 5,680 to 2,180 while ranks inside the list improved | diagnostic stop | [diagnosis](../../archive/labs/a2-pilot-diagnosis-2026-10-14/README.md) |
| 2026-10-15 | Full 16,404-batch dose at $K=8$, then widening $K$ to 16 at evaluation | Admitted 1,920 at $K=8$ and 2,400 at $K=16$, against a gate of 4,894 | negative (evaluation-time widening only) | [Stage A sweep](../../archive/labs/rung0-stage-a-sweep-2026-10-15/README.md) |
| 2026-10-14 to 10-17 | Driver cost (25 controls), finite counterfactual controls, cost breakdown, GPU baseline reuse | 25 of 25 controls pass; one proposal 3.903 s against a 37.8 ms forward pass; repeated baseline work is 46.9 to 47.2% of lane cycles; baseline reuse matched on 32 pairs, timing not run | positive (construction) | [driver cost](../../archive/labs/discrete-driver-cost-2026-10-13/README.md), [decomposition](../../archive/labs/counterfactual-decomposition-run-2026-10-16/README.md), [reuse](../../archive/labs/counterfactual-baseline-reuse-finite-pass-2026-10-17/README.md) |
| 2026-10-18 | Row-parallel kernel against the serial one | Identical losses; Route 3.843 s to 0.0691 s (55.6 times), Relation gate 4.144 s to 0.0728 s (56.9 times) | positive | [rows parity](../labs/counterfactual-rows-parity-2026-10-18/README.md) |
| 2026-10-18 | Relation run on the slow path, then on the fast path to Q1 | Slow path stopped by the owner at 904 and 1,065 batches. Fast path: $P_{succ}$ admission 2,034 to 225 (Relation) and 205 (frozen control) of 3,847 | blocked, then diagnostic stop | [strategic stop](../labs/attention-dose1-strategic-stop-2026-10-18/README.md), [Q1 diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) |
| 2026-10-19 | K16: two objectives, two seeds, 16,404 batches each | Occupancy collapse in one objective, admission loss in the other; no run reached the top-1 bar of 1,941 | negative (configuration) | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |
| 2026-10-19 | A8 dose: address-structured cells, Relation against frozen control | $P_{succ}$ 1,351 to 551 (Relation) and 857 (control) at Q1; control then finished 16,404 batches in 1,967 s (0.12 s per batch) | diagnostic stop | [A8 dose](../labs/a8-dose-2026-10-19/README.md) |
| 2026-10-19 | Injective start and BVG-K block-apply qualifier | BVG-K block-apply qualifier passes on fixtures (259 ms at $K=16$, about 8 times); the injective start fails its start check (127 of 3,847 against 500), so no run was sent | diagnostic stop | [injective pre-flight](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md), [BVG-K scoring](../labs/bvgk-rowsk-scoring-2026-10-18/README.md) |
| 2026-10-19 | Admission-preserving controller (rejects any interval that shrinks admission) | Source and 198 host tests pass; real GPU qualification and any learning result not run | inconclusive | [controller](../labs/admission-anchor-controller-2026-10-19/README.md) |

**What the negatives do and do not say.** Under D22 a negative closes one configuration, never the mechanism ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)). The pilot diagnosis found that a fixed 8-cell access budget reached fewer target cells (5,680 to 2,180) while the address hinge rose, and that per-move causal attribution was unobserved. Separately, it found a start with only 16 distinct vectors: a coupled-index initialisation defect, not an E8 capacity result. A later census found K16 training had broken much of that aliasing (ceiling about 9,000 rows) while top-1 stayed at 65 and 463, so aliasing does not explain the K16 ranking failure ([K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md)). For the Q1 stops, the admission pathway is a source-linked hypothesis, not a causal attribution or a mechanism verdict: acceptance looks only at the batch's own loss, so an accepted move could eject targets from their cells (the A8 record states this as a risk before measurement, and the K16 outcome keeps ranking capacity, the read and the schedule open). The owner stopped Dose 1 on the slow path before Q1; that is not a result.

## What has never been tested

- Route-frame and gate learning in the pilot (only vector blocks 0 to 1,639 moved).
- A collision-free start together with a wide enough shortlist ($K$ of about 34 or more, or cells placed by address structure).
- The admission-preserving rule actually running on a GPU.
- Frequent-token revisits and a full corpus pass (one 16,404-coordinate pass uses 1,049,856 of 2,425,925 training targets).
- The BVG-K learning law as a training run, and Flock reads trained in at dose.
- All 240 E8 roots, the icosian and quaternion adapters, and $\mathbb{Z}[\varphi]$ pairs.
- Any generated text, reply grade, bits per byte or tokens per second from a trained discrete model.
- An integer serving engine for the snapshot, and fresh-slice transfer for any run (the fresh slice is sealed and unopened).
- A float credit sheet on the same geometry (D37 allows it; none is registered).

## Also known as

SpiralCore/FBS discrete learning; counterfactual-choose-apply-validate training; the D32 learning law; the `rows_v1` row-parallel path; BVG-K; the discrete driver, Dose 1, Dose 2 and the rung-0 pilot and completion; `Device::propose` and `apply`.

## Code

- [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): forward pass, 16-lane counterfactual, choose, apply, validate, shared writer.
- [`device.rs`](../../crates/uor-r4-discrete/src/device.rs) and [`training.rs`](../../crates/uor-r4-discrete/src/training.rs): `propose`, `apply`, the corpus driver and the training hold.
- [`rows_source.rs`](../../crates/uor-r4-discrete/src/rows_source.rs) and [`bvgk_source.rs`](../../crates/uor-r4-discrete/src/bvgk_source.rs): row-parallel counterfactual and BVG-K.
- [`attention_audited.rs`](../../crates/uor-r4-discrete/src/attention_audited.rs): the admission-preserving controller. [`spiralcore_operator.rs`](../../crates/uor-r4-core/src/spiralcore_operator.rs): the 64-element group.

## Related mechanisms

[rank-margin](./rank-margin.md) is the loss each step minimises. [route-read](./route-read.md), [flock-read](./flock-read.md) and [relation-read](./relation-read.md) supply frames and gates as coordinates. [admission](./admission.md) is the shortlist whose erosion drives most stops. [prime-zeta-codebook](./prime-zeta-codebook.md) and [address-cells](./address-cells.md) replace the 16-vector start. [spiralcore-fbs](./spiralcore-fbs.md) supplies the group and tree. [legal-construction](./legal-construction.md) is the older float attempt at the same goal. [carrier-credit](./carrier-credit.md) contributed the shared token table.
