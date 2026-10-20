# How the model learns

Most language models learn by working out a smooth slope for every number inside them and nudging all the numbers a little. This model does not. It takes one entry of its table, tries every legal turn that entry can make, counts the mistakes after each turn using whole numbers, and keeps the best turn only if it beats leaving the entry alone. The method is exact, cheap per step and runs on the GPU. So far it has moved some counts but has not produced better language, and every new training run needs the owner's personal release.

![A loop for one training batch. A batch of 64 example rows feeds a baseline pass, then 16 parallel lanes (the current state plus 15 rotations) each score every row. A chooser lights the lowest integer loss, with ties going to staying put. An apply step commits only a strict improvement, and a validate step re-checks the stored table on the GPU. A side panel shows the same loop scoring K coordinates per launch (BVG-K) and committing them as one block only if the joint loss beats the best single move. A second side panel shows the offline float path as a separate module that must end in an export-gap check.](../figures/model/discrete-learning-loop.svg)

*One batch of discrete learning: score every legal move on 64 rows, choose the lowest integer loss, commit only a strict gain, then verify the stored result.*

Labels follow [`docs/formal_vocabulary.md`](../formal_vocabulary.md): **proof** is derived from definitions or code, **measurement** is a recorded run, **hypothesis** is untested. The mechanism page, [discrete-learning](../mechanisms/discrete-learning.md), is the short form of this page and carries the full run history.

## How it works

**What is learned.** The model holds three kinds of *coordinate*. A *vector block* is one of the four 8-number blocks of one token's vector. A *route frame* is one of 64 signed-axis rotations of 8-dimensional space, one per routing stage. A *gate* is a switch that takes the values $-1$, $0$ or $+1$. A vector block is always one of 16 roots of an E8 orbit, so it has 16 legal states, counting "stay as it is" ([architecture](./architecture.md)).

**The learning step (definition).** Training visits one coordinate $\theta$ per batch. A batch is 64 rows of training text, and which rows can count is fixed by the baseline. For each candidate move $m$ among the identity and 15 rotations, the machine replays the read and the writer and adds up the integer [rank-margin loss](../mechanisms/rank-margin.md):

$$L(m)=\sum_{x\in\mathcal R}\ell\big(x;\ \theta\leftarrow m\big),\qquad m\in\{\mathrm{id},m_1,\dots,m_{15}\}.$$

It chooses $m^*=\arg\min_m L(m)$. Ties go to the identity, then to the lowest resulting port number, then to the lowest move index. It applies $m^*$ only if $L(m^*)<L(\mathrm{id})$, and then re-checks the stored table on the GPU. Nothing is sampled, and no floating-point number appears in the step.

**Consequence (proof, from the definition).** Every committed move strictly lowers the loss on its own batch, and the new state is always a legal state. The rule says nothing about rows outside the batch. A move that helps its 64 rows can push other targets out of the shortlist, and that is the failure behind the early stops in the table below.

**Coverage (proof, by arithmetic).** One pass is 16,404 batches of 64 rows, which is 1,049,856 target rows. The training split has 2,425,925 targets, so one pass sees 43.3% of them. Each coordinate is visited about once per pass, with about 64 rows of evidence (chief architect's review §3.4).

### The GPU kernels

All model compute is GPU-only; a host fallback is a defect to fix ([charter](../../CHARTER.md) §3.5). The kernels live in [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu) and its generated siblings. The build checks the emitted GPU code and rejects any floating-point type, tensor-core instruction or approximate exponent (`crates/uor-r4-discrete/build.rs`, PTX admission audit).

1. **Ordinary counterfactual.** One block of 16 threads, one thread per move. Each thread walks every row in turn and recomputes the baseline. A route proposal took 3.903 s against 37.8 ms for a plain forward pass, which projected to about 17.8 hours for one 16,404-batch pass (**measurement** and projection, unverified, [driver cost](../../archive/labs/discrete-driver-cost-2026-10-13/README.md)). A later diagnostic put this kernel at 99.99% of a proposal ([sub-phase diagnostic](../../archive/labs/propose-subphase-diagnostic-run-2026-10-16/README.md)).
2. **Row-parallel counterfactual (`rows_v1`).** It computes the baseline once per row and each virtual move once per (row, lane), then a reducer replays the ordinary loop over those records in row order. So 64 rows by 16 lanes run as 1,024 threads. It is opt-in, and it matched the ordinary kernel's losses on every finite case checked (72, 108 and 288 cases in the three tables of the [rows parity](../labs/counterfactual-rows-parity-2026-10-18/README.md) record; 1,085 in all across the attempt files, per the [documentation reconciliation](../labs/documentation-reconciliation-2026-10-18/README.md)).
3. **Speed (measurement).** At vocabulary 4096, 4 blocks, context 192, $K=8$, 64 rows, on one RTX 5090, the median of five proposals:

| Read and coordinate | Ordinary | Row-parallel | Ratio |
|---|---|---|---|
| Route, vector block | 3.843 s | 0.0691 s | 55.6 times |
| Relation, gate | 4.144 s | 0.0728 s | 56.9 times |

This is the "56 times" figure. The timing is non-gating and was not qualified (`qualified=false`). The A8 control arm later completed 16,404 batches in 1,967 s, or 0.12 s per batch ([A8 dose](../labs/a8-dose-2026-10-19/README.md)). It used a different read, so treat that as a rough size of the gain.

### BVG-K: several coordinates per launch

BVG-K means *batched verified coordinates*, with $K$ the batch size. One row-parallel launch occupies at most 8 of an RTX 5090's 170 multiprocessors, so the card sits mostly idle. BVG-K scores $K$ coordinates in one launch (1 to 64 are supported), chooses the best move for each, and commits the whole block only if its *joint* loss beats the best single move.
- **Scoring (measurement, quoted).** About 16 times faster at $K=16$ than scoring one coordinate at a time, from the first qualifier ([BVG-K scoring](../labs/bvgk-rowsk-scoring-2026-10-18/README.md)).
- **Block apply (measurement, on fixtures).** 259 ms at $K=16$ with joint verification, about 8 times faster than 16 single proposals at roughly 130 ms each ([qualifier](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md)).
- **As a learning law, never run.** BVG-K has passed construction tests only. It is the matched discrete competitor that any float method must beat.

### Float training, offline only (D37)

On 18 October the owner revised the discrete-only rule: **floating point is legal in offline training**. A float forward pass, gradients and an optimizer may be used. Three things still hold ([D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)):

- **The bans on expensive operations stand.** No softmax or other squashing function, no dense matrix multiplication, GPU only, and the path must stay geometric.
- **Float lives in a separate module.** It gets its own scanned entry, added with its pre-registration. It never goes in the discrete crate or in `crates/uor-r4-training`.
- **The integer artifact is the boundary.** Every float candidate reports its *export gap*, the loss in quality when its weights become the integer file. A gap above the two-seed noise band disqualifies it, and a float score alone qualifies nothing (D39 item 5).

No float sheet is registered. The order set in D39 item 4 is the read-code capacity capture (W2), then BVG-K (W3), then a single float-credit sheet only if W2 points at the learning law and W3 fails. The chief architect expects the export gap to be structurally non-zero, because the table entries are exact roots with no continuous scale (**hypothesis**, `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md` §3.3).

### Holds and pre-registration

Training is **held**. The global training hold, the M2 hold and Stage B all stay in force, and a review that finds a plan fit releases nothing ([charter](../../CHARTER.md) §5.1; D37 item 6).

- **Pre-registration.** Before any GPU training or evaluation, and before any run a milestone claim rests on, a one-page sheet states the bar, the noise, the controls, the seeds, the cost and what each outcome decides. A non-author reviews it for `TEST FITNESS: FIT`. A fresh-context subagent counts as non-author. Builds, finite kernel tests and host checks need no sheet.
- **Release.** One typed owner release per sheet covers all its arms, seeds and stop rules. Casey types it personally with `UOR_LAB` unset. A lab stages the commands and verifies the records, and never types one itself (D40).
- **Compute.** Pods go through `scripts/pod/uor-pod`: at most 4 pods and $8 an hour across all labs. A healthy run is never cancelled for a self-set budget.
- **Bar.** The M1 closing bar is integer top-1 through the D11 export, on each of two seeds, beating the unigram table (1,226 of 14,287) and the successor table (6,300 of 14,287) on the same panel, with tokens per second reported (charter §4.8). The earlier 0.90 bits-per-byte bar was replaced for M1, since the hinge gives no likelihood.

## Where it stands

No training run on this path has produced a KEEP. The table below lists six campaigns since 13 October: the pilot, the Stage A sweep, the Q1 diagnosis, the four K16 runs, the A8 dose and the controller run.

| Date | Run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-10-14 | Pilot, seed 1, 1,640 batches, vector blocks only | Top-8 hits 173 to 1,169, top-1 17 to 82, yet loss 13,816,032 to 13,884,032; 236 of 1,640 batches accepted | inconclusive | [pilot](../../archive/labs/a2-pilot-2026-10-14/README.md) |
| 2026-10-15 | Same lineage to 16,404 batches, $K=8$ | Top-1 149, top-8 747 of 14,287; 1,920 targets admitted against a gate of 4,894 | negative | [Stage A sweep](../../archive/labs/rung0-stage-a-sweep-2026-10-15/README.md) |
| 2026-10-18 | Relation and control to the 4,096-batch checkpoint | Copy-successor admission 2,034 to 225 (Relation) and 205 (control) of 3,847 | diagnostic stop | [Q1 diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) |
| 2026-10-19 | K16: two objectives, two seeds, 16,404 batches | Objective A: top-1 65 and 463, accepted 407 and 410; objective R collapsed to 19 or 20 occupied cells | negative (configuration) | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |
| 2026-10-19 | A8 cells, Relation against frozen control | Admission 1,351 to 551 (Relation) and 857 (control) at the checkpoint | diagnostic stop | [A8 dose](../labs/a8-dose-2026-10-19/README.md) |
| 2026-10-19 | Admission-preserving controller | 198 host tests pass; GPU qualification and any learning result `NOT_RUN` | inconclusive | [controller](../labs/admission-anchor-controller-2026-10-19/README.md) |

**What the runs say.** Only 0.8% to 2.5% of batches in the K16 runs were accepted (407, 130, 410 and 129 of 16,404), against 14.4% in the pilot. A negative closes a configuration, not the mechanism (D22). The next decisive step, W2, is already reviewed fit and queued. It separates a representation wall from a learning-law wall (renormalization audit record).

**Never tested.** Route-frame and gate learning at dose; a full corpus pass; BVG-K or the admission-preserving rule as a GPU training run; all 240 E8 roots; any generated text or tokens per second from a trained discrete model; fresh-slice transfer (sealed); any float credit sheet.

## Go deeper

- Pipeline: [the model, end to end](./architecture.md).
- Mechanism pages: [discrete-learning](../mechanisms/discrete-learning.md), [rank-margin](../mechanisms/rank-margin.md), [admission](../mechanisms/admission.md), [legal-construction](../mechanisms/legal-construction.md) (the older float attempt at the same goal).
- Code: [`device.rs`](../../crates/uor-r4-discrete/src/device.rs), [`training.rs`](../../crates/uor-r4-discrete/src/training.rs), [`rows_source.rs`](../../crates/uor-r4-discrete/src/rows_source.rs), [`bvgk_source.rs`](../../crates/uor-r4-discrete/src/bvgk_source.rs).
- Audit: the [renormalization audit record](../labs/repo-renormalization-2026-10-19/README.md) and the [discrete-learning mechanism page](../mechanisms/discrete-learning.md).
