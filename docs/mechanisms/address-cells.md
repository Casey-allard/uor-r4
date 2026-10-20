# Address-structured cells

**Status: ACTIVE.** This is the starting table of the live attention line. At zero training it lifted the number of next-word targets the reader can reach from 127 to 1,351 of 3,847. Training then eroded that gain, and the first learning run stopped at its first checkpoint. No paired verdict exists, and the admission-preserving controller built to fix the erosion has not yet run on a GPU ([A8 dose](../labs/a8-dose-2026-10-19/README.md), [pre-flight](../labs/address-cells-preflight-2026-10-19/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

Picture a 16 by 16 board of 256 cells. Each of the 4,096 words gets a home cell, and every cell holds 16 words. The trick is where words go. A word is placed next to the cells of the words that usually come just before it in the training text. Only about 1,350 words are placed this way; the other 2,749 of 4,096 have no placed predecessor and are placed by the prime-zeta fallback (rule 4 below). When the model has just read a word, the reader looks only at a few nearby cells. If likely next words live in those cells, the right answer is at least in view. This is a table made before training, and it changes nothing about how the reader works. It only decides what the reader can see.

## What it replaces in a transformer

The token embedding table, in the part of its job that decides which tokens the next-word step can consider. A transformer's output step scores every word in the vocabulary. This model scores only the words in the cells it opens ([admission](./admission.md)), so where each word lives limits what can be chosen.

![A 16 by 16 board of cells with 16 dots each. Heavy training bigram arrows run from a previous word's cell to its likely next words. Three board states for the random-injective table, the shuffled table and the address-structured table show the arrows growing shorter, with the eight opened cells outlined and a counter of reachable targets reading 127, 476 and 1,351 of 3,847.](../figures/mechanisms/address-cells.svg)

*The same board with three placements. With the address-structured table the likely next words sit near the last word's cell, so more of them fall inside the eight opened cells.*

## How it works

**Cells.** A word's *writer cell* is the pair (root of block 0, root of the last block) on a 16-root E8 orbit, the same $16\times16=256$ grid as the [prime-zeta codebook](./prime-zeta-codebook.md). All 4,096 full vectors stay distinct.

**What the reader can see (Definition).** The reader admits the $K=8$ occupied cells nearest its output $q$, by L1 distance on those two root blocks, and can rank only the words inside them. Placement therefore decides visibility.

**Building the table (Definition).** This is host data preparation from the training split only.

1. Count integer bigrams $s\to t$ at the positions where the model must answer, with $s$ the last context word.
2. Take words by falling frequency (ties to the lower id).
3. Put each word in the cell $c$ that minimises
$$\sum_{s\ \text{placed}} \mathrm{count}(s\to t)\; L_1\big(\mathrm{cell}(s),\,c\big)$$
   subject to a quota of 16 words per cell.
4. A word with no placed predecessor goes to its prime-zeta cell if free, else the nearest cell with room. Inner blocks get distinct root pairs by rank in the cell.
5. A recount requires 4,096 distinct vectors with every cell at quota.

**The control.** The *shuffled* table (S8) runs the same greedy search with predecessor identities relabelled by a seeded permutation. It keeps frequency, quota and total mass but destroys the relation, so it separates "words placed near their predecessors" from "words packed neatly".

**A stated gap (proof, by reading the construction).** The reader's real centre is the last word plus the consensus of the context read, not the last word's cell alone. Placement only partly predicts admission for that reason. The gap was reported, not tuned away.

**Rule check.** The construction is integer counting and integer L1 sums, with no floating point, so there is no export gap. No softmax, no squashing, no matrix multiplication (D23, D30). Host work is data preparation only; no model quantity is computed there (D24).

## What has been tried

Every number in the last four rows is a count of reachable target words on the development panel. None is an accuracy, loss or reply quality.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 14 Oct | Pilot on the random start (16 distinct vectors), 1,640 batches | Top-8 hits rose (173 to 1,169 of 14,287) but reachable targets fell (5,680 to 2,180) and total loss rose | inconclusive | [pilot](../../archive/labs/a2-pilot-2026-10-14/README.md), [diagnosis](../../archive/labs/a2-pilot-diagnosis-2026-10-14/README.md) |
| 18 Oct | First relation-versus-control dose on the 16-vector start | Both arms hit the admission-collapse gate at 4,096 batches (2,034 to 225 and 205). Registered stop, not a negative | diagnostic stop | [Q1 diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) |
| 19 Oct | Two objectives (an address margin, K16-A, and an occupied-cell hinge, K16-R), 2 seeds, 16,404 batches | K16-R passed admission only by collapsing to about 20 cells. K16-A missed the gate. No keep | negative, for those objectives | [outcome call](../labs/k16-outcome-call-2026-10-19/README.md) |
| 19 Oct | Prime-zeta table at K=8, zero updates | 127 of 3,847 reachable against a gate of 500; full panel 427 of 14,287 | diagnostic stop | [pre-flight](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) |
| 19 Oct | Host export of the address-structured and shuffled tables, rebuilt by a Rust binary and an independent Python verifier | Both injective, 256 cells at quota 16. Mean training-predecessor L1 per target: 9.22 (address-structured), 12.32 (shuffled), 13.96 (prime-zeta) | positive | [codebook](../labs/address-cells-codebook-2026-10-19/README.md), [verifier](../labs/address-cells-consumer-verifier-2026-10-19/README.md) |
| 19 Oct | Five-arm zero-update pre-flight on one GPU, bars fixed beforehand | Reachable of 3,847: address-structured 1,351; K=16 version 2,178; shuffled 476; prime-zeta 127; K=16 prime-zeta 246. Paired against prime-zeta: 1,302 rows gained, 78 lost (net 1,224 against a floor of 74.3). Paired against shuffled: net 875 against 81.0. Cost about $0.17 | positive for the reach gates; the comparison against an unpinned initializer was NOT_RUN (post-merge correction) | [pre-flight](../labs/address-cells-preflight-2026-10-19/README.md) |
| 19 Oct | Learned-attention dose, relation read against a frozen-zero control, seed 1, K=8, 16,404 batches planned | Relation fell to 551 at the first checkpoint and stopped (threshold 675.5). Control fell to 857, ending at 797. Relation accepted 87 relation trits (its three-valued link dials). Cost about $1.9. No paired verdict; fresh slice sealed | diagnostic stop | [A8 dose](../labs/a8-dose-2026-10-19/README.md), [status](../labs/a8-current-status-scope-2026-10-19/README.md) |
| 19 Oct | Admission-preserving controller: reject any interval that drops reachability below an anchor of 3,847 training rows (margin 24, floor 500) | Source merged, 198 host tests pass. Real GPU build, qualification, timing and speed-up `NOT_RUN` | blocked | [controller](../labs/admission-anchor-controller-2026-10-19/README.md), [source record](../labs/admission-preserving-2026-10-19/README.md) |

**Reading these honestly.** Placement works at zero training: it moved reachability ten-fold at the same $K$ and the same board, and the shuffled control shows the predecessor relation, not neat packing, explains most of it. That is evidence the reader can now *see* its targets, not that it can learn to pick them. The dose then lost reachability in both arms (control down 37%, relation down 59%), consistent with accepted learning moves carrying words out of their placed cells. That is co-occurrence, not attribution. The earlier failures all ran on a start with 16 distinct vectors, so they close that start, not attention. The record's own risk section predicted the erosion and called it a diagnostic stop.

## What has never been tested

- Whether the reader can learn on a well-placed table at all. No run has produced a paired development verdict.
- Any accuracy, ranking, bits-per-byte or reply quality for this table.
- Placement that also uses successor edges. Version 1 lets only already-placed predecessors pull a word.
- Placement keyed on the real read centre instead of the last word's cell.
- The compact 32-cell form, $K\approx34$ or more, and partly injective tables.
- More than one shuffle seed, and training from the K=16 bracket arms.
- The admission-preserving rule in a real GPU run, and a finer, cell-level census of which moves eject which words.
- Other shapes, such as 16,384 words on 512 cells (planned after the dose).

## What comes next

The mechanism is ACTIVE, so this is a plan, not a reopening. First comes the GPU qualification of the merged controller, then the admission-preserving rerun of the learned dose. Two risks are on the record. The rule could turn into a no-op if it rejects more than half the intervals, or three full intervals in a row. And $K=8$ may be too small as occupancy grows. A relation arm must also beat the successor-table comparator (top-1 on 6,300 of 14,287 development rows) to count as learning.

## Also known as

A8 (`address_structured_v1`); S8 (the shuffled control); P8, P16 and A16 (uniform and K=16 bracket arms); injective codebook initialisation; 256 by 16 cells; predecessor-structured placement. Not to be confused with the October 4 "observable geometric observation cells", which are a different, older mechanism that shares the word "cells".

## Code

- [`address_cells.rs`](../../crates/uor-r4-discrete/src/address_cells.rs): bigram counts, the greedy build with quota and recount, the shuffled control.
- [`address-cells-codebook.rs`](../../crates/uor-r4-discrete/src/bin/address-cells-codebook.rs): the exporter. It reads the training split only and never the development panel.
- [`attention_preflight.rs`](../../crates/uor-r4-discrete/src/attention_preflight.rs): the zero-update battery.
- [`attention_learning.rs`](../../crates/uor-r4-discrete/src/attention_learning.rs): the relation-versus-frozen-zero dose with its first-checkpoint stop rule.
- [`attention_anchor.rs`](../../crates/uor-r4-discrete/src/attention_anchor.rs) and [`attention_audited.rs`](../../crates/uor-r4-discrete/src/attention_audited.rs): the admission-preserving controller.
- [`coverage_census.rs`](../../crates/uor-r4-discrete/src/coverage_census.rs): alias ceiling, tie mass and crowding.
- Pre-registered sheets: [pre-flight](../harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md) and [training dose](../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md).

## Related mechanisms

[prime-zeta-codebook](./prime-zeta-codebook.md): parent and baseline; its cell is the fallback. [admission](./admission.md): defines the gates that decide whether this table is any use. [relation-read](./relation-read.md): the arm the dose was meant to test. [flock-read](./flock-read.md): its consensus shifts the read centre away from the last word's cell. [route-read](./route-read.md): the read used in the two-objective redirect. [rank-margin](./rank-margin.md): the occupied-cell hinge that collapsed occupancy, the failure this table avoids by fixing the table instead of the objective. [discrete-learning](./discrete-learning.md): the shared writer whose accepted moves erode the placement. [spiralcore-fbs](./spiralcore-fbs.md): source of the E8 orbit roots.
