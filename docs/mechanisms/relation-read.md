# Relation read (finite relational head)

**Status: ACTIVE.** It is built, passes 9 of 9 finite CUDA controls, and has trained twice. Both training runs were stopped by loss of admission before the read could be judged, so there is no verdict yet ([A8 dose record](../labs/a8-dose-2026-10-19/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

Much of language works by copying. When you meet a word you saw earlier, the next word is often the one that followed it last time. The [Flock read](./flock-read.md) finds earlier positions that look like the current one, but it cannot say "take the word right after that earlier copy." Relation adds that ability. It lets the read follow simple links between positions: the word right after, a sibling in the same phrase, the same word again. Each kind of link is controlled by one three-position dial: off, boost or penalise. With every dial off, the model behaves exactly like Flock. The dials have been trained twice, but never long enough to judge them.

## What it replaces in a transformer

The scoring half of attention, which decides which earlier positions matter. In a transformer, copying the next word is done by "induction heads" that learn a query-key match pattern. Here the same habit is a bounded, exact rerank by link type. There is no learned score table and no softmax.

![The sequence x A B y z A drawn as leaves of a small tree. The earlier A is lit with match score 8. A plus-one successor dial sends a scored arrow one leaf to the right, so B also scores 8 and jumps to the top of the ranked list. A toggle sets all dials to zero and the ranking falls back to the Flock order.](../figures/mechanisms/relation-read.svg)

*The copy-the-next-word read on "x A B y z A": the dial for "successor" passes the earlier A's score on to B.*

## How it works

Relation is Flock plus a reranker. The pool is every causally eligible node of the same ternary tree that [Route](./route-read.md) and Flock use. Five link types come from retained structure only:

| # | Link type | What it connects |
|---|---|---|
| 0 | parent-child | a node and its tree children |
| 1 | sibling | neighbouring children of one parent |
| 2 | sequential successor | leaf $j+1$ follows leaf $j$ (the copy-successor primitive) |
| 3 | same token | two positions holding the same token id |
| 4 | same frame | two nodes sharing an orbit-root frame |

**Definition.** For a query with frame root $f_q$, each node $i$ starts with a match score $m_i=8-d_{L1}(f_q,f_i)\in\{0,4,8\}$. Each (link type $e$, tree level) has a learned trit $g[e,\ell]\in\{-1,0,+1\}$, which makes 25 dials for five levels. For sweeps $t=1,\dots,S$ ($1\le S\le5$):

$$r_i^{(t)}=r_i^{(t-1)}+\sum_{e=0}^{4}g[e,\ell(i)]\sum_{j\in\mathcal{N}_e(i)}s_j^{(t-1)},\qquad s^{(0)}=m,\ \ s^{(t-1)}=r^{(t-1)}\ (t\ge2)$$

Here $\mathcal{N}_e(i)$ are node $i$'s neighbours under link type $e$. Candidates are ranked by $r_i^{(S)}$ first, then by the Flock keys (frame distance, Hamming distance, recency, node id). The top $K\le8$ feed the unchanged Flock transport, consensus and gated add.

**Worked example.** In "x A B y z A" the query is the final A. The earlier A has the same frame, so $m=8$. Set the level-0 successor trit to $+1$ with one sweep. B receives $r=+8$ and outranks every candidate with $r=0$, whatever that candidate's own frame distance. B's payload is carried in. That is the copy-successor read.

**Two facts that follow from the code (proof).**
- With every trit at zero, all scores are zero and the ranking is bit-identical to Flock. This is the rollback identity, and it is the control used in training.
- Scores are integers bounded by $8\cdot(1+8)^5=472{,}392$, far below $2^{31}$, so plain integer addition suffices. No multiplier, no float, no clamp.

**Why Route and Flock cannot do this (proof, from reading the kernel).** Both rank each node by its own frame and carry that node's own payload. The successor's frame has no fixed relation to the query's frame, so neither read can promote a node because its predecessor matched. The design record makes this argument in full.

**The target is large (measurement).** On the pinned development panel (14,287 eligible rows, context 192), the query token recurs earlier in 8,648 rows, and the target is the word after the latest earlier copy in **3,847 rows (26.9%)**. This is the copy-successor sub-panel, called $P_{succ}$ here. Its distance buckets 1-8, 9-32, 33-96 and 97-192 hold 50, 2,872, 900 and 25 rows.

**Learning and the bar.** The trits reuse the existing gate parameter kind and are learned by counterfactual moves ([discrete-learning](./discrete-learning.md)) on a schedule where every eighth batch is a read batch. To count, a trained Relation must beat a context-free successor table (called T2), which scores **6,300 of 14,287** top-1 in a host re-score, while the trained Route read scored 149.

**Rule check.** Hard top-K by integer keys (no softmax, D23); no matrix multiplication (D30); integer compare, gather, add and subtract (D11). No D11 export or export gap exists for a trained Relation artifact.

## What has been tried

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| Sept 2026 | Precursor: one-read typed attention over four records, authored panel | 64/64 development answers; controls fail; not the discrete head | positive | [#973 record](../../archive/native_geometric_relational_attention_973.md) |
| 2026-10-14 | Model-free statistics and an adversarial review of four attention proposals | A bounded exact-relation reranker is the smallest falsifier; read parameters got 20 of 16,404 proposals | inconclusive | [independent review](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) |
| 2026-10-15 | `ReadSpec::Relation` built in the discrete crate | Source only; GPU qualification later passed 9/9 | inconclusive | [construction](../../archive/labs/readspec-relation-m1-construction-2026-10-15/README.md) |
| 2026-10-17 | Interleaved schedule; six paid pod attempts at compiler checks | 0 model updates; blocked by tooling | blocked | [learning path](../../archive/labs/native-relation-learning-path-2026-10-17/README.md) |
| 2026-10-17 | Zero-training preflight | 9/9 controls pass; starting $P_{succ}$ admission 2,034 of 3,847 | positive | [preflight](../../archive/labs/attention-preflight-2026-10-17/README.md) |
| 2026-10-17 | Host re-score of saved checkpoints | T2 table 6,300 versus trained read 149; admission K 8 to 16 did not help (149 to 138) | inconclusive | [host diagnostics](../../archive/labs/host-diagnostics-2026-10-17/README.md) |
| 2026-10-18 | Dose 1 on the ordinary path | Owner stopped it for speed at 904 and 1,065 batches; not graded | diagnostic-stop | [strategic stop](../labs/attention-dose1-strategic-stop-2026-10-18/README.md) |
| 2026-10-18 | Row-parallel kernel against the ordinary path | Identical losses, 3.843 s versus 0.0691 s (55.6 times faster) | positive | [rows parity](../labs/counterfactual-rows-parity-2026-10-18/README.md) |
| 2026-10-18 | Dose 1 (row-parallel), Relation against a trit-locked control, 16-vector codebook, stopped at 4,096 batches | $P_{succ}$ admission 2,034 to 225 (Relation) and 205 (control). Trits attempted/sensitive/accepted: 275/140/94 against 275/141/0 | diagnostic-stop | [Q1 diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) |
| 2026-10-19 | Zero-update test of three codebook placements | Admitted rows: address-structured 1,351, uniform 127, shuffled 476 of 3,847 | positive | [address cells](../labs/address-cells-preflight-2026-10-19/README.md) |
| 2026-10-19 | Injective codebook as the start | 127 of 3,847 against a gate of 500; run not dispatched | diagnostic-stop | [injective preflight](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) |
| 2026-10-19 | A8 dose: Relation against the control | At batch 4,096, Relation: 1,351 to 551, stopped there under its half-start rule. Control: 1,351 to 857, then ran the full dose. Trits 275/124/87 | diagnostic-stop | [A8 dose](../labs/a8-dose-2026-10-19/README.md) |
| 2026-10-19 | Admission-preserving rerun, pre-registered | Passed its fitness review; nothing run | inconclusive | [admission-preserving](../labs/admission-preserving-2026-10-19/README.md) |

**Reading these honestly.** In the A8 dose, the Relation arm fell faster, which coincides with 87 accepted relation-trit updates. That is co-occurrence, not attribution. Both arms lost admission to learning moves, and the sheet named this risk before the run. Under the project's diagnostic-stop rule (D31) and the rule that a negative closes a configuration, never a mechanism (D22), these are diagnostic stops: not negatives, and not a KEEP. The fresh transfer slice has never been opened.

## What has never been tested

- A full-dose Relation arm paired against its control with admission intact.
- Whether the level-0 successor trit is ever learned, and what it buys on the 3,847-row sub-panel.
- Relation against the T2 table and against Route in the matched discriminator.
- More than one sweep, the other four link types, and any level above 0.
- Whether the interleaved schedule gives the read enough proposals.
- Relation with admission-preserving acceptance, at K=16, or on a fully injective address-structured codebook together.
- Cost and tokens per second at serving shape.

## What comes next

1. **Admission-preserving A8 rerun.** A guard audits a fixed anchor of 3,847 training rows every 64 batches and rejects any interval that erodes admission (margin 24). It stops as a no-op if the rejection rate exceeds 0.50. The sheet is FIT; the controller and verifier merged (`UOR-Foundation/uor-r4#2782`, `#2783`); finite qualification, project-manager dispatch and an owner-typed release are outstanding.
2. **Stage M2 discriminator.** A matched interleaved Relation against Route, with a zero-trit ablation. Still held and awaiting a fitness review.
3. **Float extension (hypothesis).** A float-trained version of the 25 trits (D37) only after a Relation KEEP, with an integer export and a reported export gap.

Open questions: whether the guard rejects so many intervals that learning does nothing; whether copying is better delivered by the read or by a successor table beside it; and which of the 25 dials matter beyond level 0.

## Also known as

`ReadSpec::Relation`; finite relational head; bounded exact-relation reranker; D34 reranker-first; copy-successor or induction read; relation trits; the Dose 1 and A8 Relation arms. Some older notes call the row-parallel run "Dose 2" because its attempt id contains `dose2-rows`. This page follows the audit and calls it Dose 1.

## Code

- [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): the Relation branch inside `flock_vector` (lines 164-212) and the comparator at line 236.
- [`lib.rs`](../../crates/uor-r4-discrete/src/lib.rs): `ReadSpec::Relation { kept, sweeps }`, `RELATION_EDGE_TYPES = 5`.
- [`relation_qualification.rs`](../../crates/uor-r4-discrete/src/relation_qualification.rs): the 9 controls. [`training.rs`](../../crates/uor-r4-discrete/src/training.rs): the interleaved schedule and the frozen-zero control.
- [`attention_learning.rs`](../../crates/uor-r4-discrete/src/attention_learning.rs), [`attention_preflight.rs`](../../crates/uor-r4-discrete/src/attention_preflight.rs), [`attention_transfer.rs`](../../crates/uor-r4-discrete/src/attention_transfer.rs), [`attention_audited.rs`](../../crates/uor-r4-discrete/src/attention_audited.rs).
- Design record: [`finite-relational-head.md`](../../archive/mechanisms/finite-relational-head.md).

## Related mechanisms

[flock-read](./flock-read.md): relation-off equals Flock. [route-read](./route-read.md): cannot express copy-successor. [admission](./admission.md): the ceiling that stopped both doses. [address-cells](./address-cells.md): the codebook change that restored starting admission. [prime-zeta-codebook](./prime-zeta-codebook.md): the injective constructor. [object-store](./object-store.md): the companion D34 line, not attention. [discrete-learning](./discrete-learning.md): the move law and the speedup that train the trits.
