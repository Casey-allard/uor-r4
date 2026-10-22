# Flock read (starling / murmuration neighbour read)

**Status: ACTIVE.** The integer Flock kernel passes its finite CUDA construction controls and runs inside every Relation and control arm. But no run has ever isolated what the Flock read itself contributes ([attention-state record](../labs/attention-state-2026-10-18/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

A starling in a flock does not watch the whole flock. It watches about seven nearest neighbours and turns toward their average heading. The Flock read works the same way. At each step the model picks a handful of earlier positions that sit closest to the current one, rotates what each one knows into the current position's point of view, and takes their consensus. Then it nudges its own state toward that consensus. Nothing is normalised, no scores are exponentiated, and no weights are multiplied. Whether this beats simply reading one position is still untested on real text.

## What it replaces in a transformer

Attention, both halves: choosing which earlier positions to read, and mixing what was read into the current state. A transformer weights every earlier position with a softmax and sums them. Flock keeps a few by hard integer ranking and adds their rotated consensus with a three-valued dial.

![The 600-cell drawn as 120 vertices. One query token sits at its home vertex with the 12 adjacent vertices outlined. Its seven nearest earlier tokens inside that neighbourhood are highlighted, each with an arrow showing its vector rotated into the query's frame. The arrows sum into one consensus vector, and the query vector bends toward it.](../figures/mechanisms/flock-read.svg)

*The flock read on the 600-cell: a query point, its seven nearest neighbours, their rotation into the query's frame, and the consensus they produce. The live integer version uses 16 frame roots in place of the 600-cell (inset).*

## How it works

**The float-era picture (inactive evidence).** The 600-cell is a four-dimensional solid with 120 vertices, and each vertex has 12 neighbours. Each token was given a home vertex. A query read only the tokens stored at its own vertex and the 12 adjacent ones, kept the nearest seven, and moved their values into its own frame using exact icosian transport. Those experiments ran in floating point, with the selection on the host and softmax helpers during training.

**The live integer form.** Per 8-dimensional block, context sits in a ternary tree whose leaves are earlier positions ([route-read](./route-read.md) uses the same tree). Each interior node holds the snapped sum of its three transported children. For each tree level $\ell$ there is one learned gate $g_\ell\in\{-1,0,+1\}$.

1. **Rank (Definition).** Eligible nodes at level $\ell$ end strictly before the query. Keep the best $K$ ($1\le K\le 8$) under the lexicographic key $\big(d_{L1}(f_n,f_q),\ d_{H}(f_n,f_q),\ -\mathrm{last}(n),\ n\big)$. Here $f$ is a node's frame root among the 16 roots of one E8 orbit and $d_H$ is coordinate Hamming distance.
2. **Carry.** Each kept payload $p_n$ is moved into the query's frame by the precomputed transport action between the two roots: $c_n=\mathrm{BFS}[f_n\to f_q]\,(p_n)$.
3. **Consensus.** $m_\ell=\operatorname{snap}\big(\sum_{n\in\text{kept}}c_n\big)$, the nearest of the 16 roots.
4. **Gated add.** $v\leftarrow v+g_\ell\,m_\ell$. Trit $+1$ adds the consensus root, $-1$ subtracts it, $0$ skips the level. The output is $v$ snapped back to a root.

This is built as an integer form of the Vicsek/Kuramoto alignment rule $\Delta=K_h(\text{mean}-v_t)$ with $K_h\in\{-1,0,+1\}$: align with the local flock, with no weights, no normalisation and no division. That correspondence is the design intent; the page makes no equivalence claim.

**Rule check.** Hard rank by integer keys, so no softmax (D23). No matrix multiplication (D30). The kernel uses gather, compare, add and subtract, and the snap is built by repeated addition (D11). No D11 export of a trained Flock model has been built or measured.

**Cost (hypothesis).** The tree variant should read in about $O(\log n)$ steps. That has not been measured in the integer form.

## What has been tried

Almost all evidence below is from the **float era**, on a synthetic 512-to-1,536-token recall task or a small language-model recipe, with host-side selection and softmax helpers. It is inactive evidence under the no-softmax rule (D23). It shows what that recipe did, and it cannot judge a hinge-credited integer flock. "Recall" means the share of queries whose stored value was read back correctly.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-10-10 | Four flock/rank-table reads in a 19.93M-parameter stack, 2 seeds | All four missed the 0.01 BPB bar. Softmax 0.8772 BPB; fixed rank +0.0398; Hamming rank +0.2585; learned rank table +0.0185; wide flock +0.0172 | negative | [softmax-free read](../../archive/labs/softmax-free-read-2026-10-10/README.md) |
| 2026-10-11 | Rung 1 (from scratch), rung 3 (content gate), diagnostics, and SoftSort | Recall 0.009 and 0.010 against 0.617 for softmax. With the source forced into a fixed slot, recall was 1.000. SoftSort was stopped by the owner | negative | [rung 1](../../archive/labs/starling-read-2026-10-11/README.md), [content](../../archive/labs/starling-content-2026-10-11/README.md), [pivot](../../archive/labs/starling-pivot-2026-10-11/README.md), [SoftSort](../../archive/labs/softsort-s-2026-10-11/README.md) |
| 2026-10-11 | Rung 2: dense helper annealed to zero, 3 seeds | Source in the kept seven rose from 0.099 to 0.727 then fell to 0.556; recall 0.011 | negative | [anneal](../../archive/labs/starling-anneal-2026-10-11/README.md) |
| 2026-10-12 | Training-only softmax surrogate; distillation and sweep | Recall 0.141 (bar 0.28); distillation 0.188 to 0.380; heavy weight kept the source more often but broke the readout (0.128) | negative / inconclusive | [surrogate](../../archive/labs/starling-surrogate-2026-10-12/README.md), [sweep](../../archive/labs/starling-distill-sweep-2026-10-12/README.md) |
| 2026-10-12 | Light-distillation read, 3,600 steps | Mean recall 0.780, 5 of 5 seeds above 0.30. Training budget was unequal to softmax | positive | [recall](../../archive/labs/starling-recall-2026-10-12/README.md) |
| 2026-10-12 | Equal budget against softmax, ring store of the last 4 keys per cell | At 512 positions: softmax 0.991, flock 0.831 (bar 0.841). At 1,536: 0.999 against 0.241, because old sources are evicted | negative | [context](../../archive/labs/starling-context-2026-10-12/README.md) |
| 2026-10-12 | Persistent base-3 tree replaces the ring, 1,536 context | Source found 0.99, 0.68, 0.90 of the time; 6 to 36 candidates read per query; recall 0.329 against softmax 0.999. Retention fixed, readout limits | positive | [tree](../../archive/labs/starling-tree-2026-10-12/README.md) |
| 2026-10-12 | Readout distillation; dense bootstrap teacher | Mean recall 0.377 versus 0.329; one seed reached 0.964 at 68.9 mean candidates. The teacher run needed a dense model, so it is inactive | negative / inconclusive | [readout](../../archive/labs/starling-readout-2026-10-12/README.md), [bootstrap](../../archive/labs/starling-bootstrap-2026-10-12/README.md) |
| 2026-10-13 | Lawful native read (rung 0b) and the integer Flock and Gate kernels | 16/16 finite controls pass for rung 0b; 17/17 for the integer kernels; 25/25 in the driver-cost run. No training | positive (construction only) | [rung 0b](../../archive/labs/starling-rung0b-source-2026-10-13/README.md), [integration](../../archive/labs/discrete-flock-writer-integration-2026-10-13/README.md), [driver cost](../../archive/labs/discrete-driver-cost-2026-10-13/README.md) |
| 2026-10-18 | Dose 1: the Flock-equivalent control (relation dials locked at zero), 16-vector codebook | Admission on the copy-successor panel fell 2,034 to 205 of 3,847; the read could not be judged | diagnostic-stop | [Q1 diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) |
| 2026-10-19 | A8 dose: the same control for the full 16,404 batches | 301 accepted vector updates, about 0.12 s per batch; admission fell 1,351 to 857 of 3,847 by batch 4,096 (a 37% loss); fresh slice sealed | diagnostic-stop | [A8 dose](../labs/a8-dose-2026-10-19/README.md) |

**Reading these honestly.** The M4 stack and the starling rungs were softmax-tuned, short-context and host-side. The ring-store collapse is eviction, which the tree fixed. The two October control runs lost admission to learning moves first ([admission](./admission.md)), so the read was never the thing being measured. SoftSort was closed by owner decision, not by a measured negative. D22 applies: these close configurations, not the mechanism.

## What has never been tested

- Flock as a stand-alone integer read trained on the corpus and compared with [Route](./route-read.md). No arm tests the transported-consensus read against Route.
- A read-off versus read-on ablation of Flock inside the control arm.
- The lawful no-softmax decisive run (in-context recall on copyable development targets).
- Flock on a codebook whose addresses are distinct and stay admitted.
- K and M at long context, and the sink and local-window variants in integer form.
- A float-trained coupling magnitude, rank-consistent soft-to-hard selection, and the tree read in integer form.
- A chirality sign on the coupling.
- Served tokens per second of a flock read against a scored read at long context.

## A fair test needs

*From the earlier brief, [archive/mechanisms/starling-read.md](../../archive/mechanisms/starling-read.md).*

- A training signal for the addresses: soft-to-hard annealing works (rung 2).
- A readout that keeps the right source's information: under the murmuration read this is the transported flock mean, scored by flock coherence and the source-in-flock rate.
- Recall measured by distance bucket (so eviction is distinguishable).
- More than one read layer before any multi-hop claim.

## What comes next

The mechanism is ACTIVE, so this is a plan, not a reopening.

1. **Coupling sheet (C1).** Train a float coupling strength per head and level over the fixed integer Flock topology. Export it to a trit or small integer. Include zero-coupling and random-coupling controls, 2 seeds or 200 fresh rows, and a reported export gap. The re-derived pre-registration sheet ([`m4-flock-coupling-prereg-DRAFT-2026-10-22.md`](../harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md), superseding [`m4-c1-flock-coupling-DRAFT-2026-10-18.md`](../harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md); [lab record](../labs/m4-flock-coupling-prereg/README.md)) has all finite controls (`C0`, `X1`–`X12`, `F1`–`F7`) verified on the host and awaits PM/owner registration of the 5 open items on [`#2860`](https://github.com/UOR-Foundation/uor-r4/issues/2860#issuecomment-6352562539).
2. **Soft-to-hard selection (C2).** Use a rank-consistent hinge and a linear anneal, with no temperature exponential, ending at the exact integer rank. Score it by how often the source is in the flock, and by rank-1-when-kept by distance bucket.
3. **Tree read (C3).** Use the tree for $O(\log n)$ reads, only after C1 shows the coupling learns, and with tokens per second reported first.

Any retry of the float starling or rank-table forms needs a fresh fitness review, an owner release, a separately scanned float module, a D11 integer export and no softmax surrogate.

## Also known as

`ReadSpec::Flock`; starling read; murmuration read; `starling_native` (rung 0b); flock or rank-table reads; softmax-free read; seven nearest neighbours on the 600-cell.

## Code

- [`crates/uor-r4-discrete/src/discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): kernel `flock_vector` (lines 125-269) and `snap` (102-110).
- [`lib.rs`](../../crates/uor-r4-discrete/src/lib.rs): `ReadSpec::Flock { kept }`, the gate trit parameter, and `ReadInitialization::ActiveLeafV1`, which sets gates to +1 so the read is not causally dead at the start.
- [`flock_qualification.rs`](../../crates/uor-r4-discrete/src/flock_qualification.rs): six finite CUDA controls.
- [`flock_coupling_controls.rs`](../../crates/uor-r4-discrete/tests/flock_coupling_controls.rs): seven finite host controls verifying `C0`, `X1`–`X12`, `F1`–`F7`, `F5b`, and `F5c` against `build.rs` `ROOTS`/`BFS`.
- Inactive evidence: `crates/uor-r4-training/src/starling_native.{rs,cu}`, `flock.rs`, `geometric_stack.rs`. The served integer selector for the older lookup stack is `crates/uor-r4-integer/src/stack/flock.rs`.

## Related mechanisms

[route-read](./route-read.md): Flock replaces Route's single leaf with a transported consensus. [relation-read](./relation-read.md): Flock plus a reranker; with all relation dials at zero it is bit-identical to Flock. [admission](./admission.md): every Flock-equivalent run was limited by admission first. [rank-margin](./rank-margin.md): the hinge that replaced the softmax surrogate. [spiralcore-fbs](./spiralcore-fbs.md): the transport table and tree. [hopf-observation](./hopf-observation.md): considered for neighbour keys. [exact-memory](./exact-memory.md): the persistent tree is an exact addressed store of the past.
