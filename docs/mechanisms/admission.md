# Admission (cell shortlist before the token is chosen)

**Status: ACTIVE.** It is the output step of the current model and the first measured binding limit, seen on rung 0 and for the top-8 bar (after K16, the cause of the weak ranking is open): the correct word often is not in the shortlist at all ([attention-state record](../labs/attention-state-2026-10-18/README.md)).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea (see [the charter](../../CHARTER.md), section 4.6). Unfamiliar project terms (K16, A8, admission, NOT_RUN) are in the [glossary](../start/glossary.md).

## In plain words

Before the model picks a word, it narrows its 4,096 possible words down to those stored in a few "drawers", the drawers nearest to where its read points. A word in a drawer that was not opened can never be chosen, however good the read was. So the number of correct words whose drawer gets opened sets a ceiling on everything upstream. This ceiling was the limit on rung 0; the later K16 runs show that a high admitted count alone does not give good ranking. In several runs, learning itself pushed correct words out of the opened drawers, so the number of reachable answers fell during training. The next registered run adds a guard against that.

## What it replaces in a transformer

The output head: a transformer scores every word in the vocabulary and normalises with a softmax. This model never scores all 4,096 words. It makes a hard shortlist of cells, then picks the nearest word inside it. There is no softmax and no normalised distribution.

![A 16 by 16 grid of drawers with 4,096 tokens drawn as dots. A query star lands in the grid and the 8 nearest occupied drawers are lit. The target word is shown outside the lit set. Three small copies of the grid, at the start and after 4,096 batches in two training arms, show the target's drawer drifting out of the lit set, with the count of reachable targets beside each.](../figures/mechanisms/admission.svg)

*Admission: only words in the lit drawers can be chosen. In the A8 control run, the number of copy-successor targets in lit drawers fell from 1,351 to 857 of 3,847 within 4,096 batches.*

## How it works

The writer takes the read's integer vector $q$ ($B=4$ blocks of 8 numbers) and picks a token in two steps.

**Definition (admission).** Each token $t$ has a codebook vector. Its cell is the pair $c(t)=(\rho_{\text{first}}(t),\rho_{\text{last}}(t))$ of orbit-root labels of its first and last block, so there are $16\times16=256$ cells. The address distance from $q$ to a cell is the L1 distance of $q$'s first block to the cell's first root plus the same for the last block. The $K$ nearest *occupied* cells are admitted, ties by cell number. $K$ is 8 in the lineage of record and at most 16.

**Definition (ranking).** Among tokens in admitted cells, the nearest by total L1 over all blocks wins. Ties go to the lowest token id.

**Consequence (proof).** A target outside the admitted cells cannot be predicted whatever the read does. So top-1 hits $\le$ top-8 hits $\le$ $N_{\text{adm}}$, the number of rows whose target cell is admitted. Rung 0 shows the force of this. Its top-1 bar was 1,941 rows, but only 1,920 targets were admitted at the end, so the bar was arithmetically unreachable.

**Training hinge.** Learning uses an integer hinge: $8\times$ the token margin plus $32\times$ a cell term. The two cell terms are objective A (the margin to the nearest rival cell) and objective R (how many occupied cells outrank the target beyond $K$). The weights are applied by shift-and-add, with no multiplier ([rank-margin](./rank-margin.md)).

**Why learning erodes it.** Codebook moves change a token's cell. A move that improves the loss can eject a target from the shortlist. The admission-preserving rule audits a fixed anchor panel every 64 batches and rejects any interval that shrinks admission.

**Rule check.** Hard top-K and a hinge over declared rivals: no softmax (D23). Sums of absolute differences, no matrix multiplication (D30). Integer only (D11), but no served export of an admission artifact has been measured.

## What has been tried

Terms: $P_{succ}$ is the 3,847-row copy-successor sub-panel (see [relation-read](./relation-read.md)). Q1 and Q4 are the checkpoints at 4,096 and 16,404 batches. The full panel has 14,287 rows. "Gate" is a pre-registered pass line.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 2026-10-14 | Seed-1 pilot, 1,640 batches (and its loss decomposition) | Top-1 17 to 82, top-8 173 to 1,169; loss rose slightly. Token loss fell 63,968 but cell loss rose 131,968; 16 distinct vectors grew to 155 | inconclusive | [pilot](../../archive/labs/a2-pilot-2026-10-14/README.md), [diagnosis](../../archive/labs/a2-pilot-diagnosis-2026-10-14/README.md) |
| 2026-10-15 | Full rung-0 dose, $K=8$, 16,404 batches | Top-1 149 (bar 1,941); top-8 747 (bar 4,775). Admitted fell from 5,680 at the start to 1,920 | negative (admission-limited) | [rung-0 prep](../../archive/labs/rung0-admission-prep-2026-10-15/README.md), [redirect sheet](../../archive/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md) |
| 2026-10-15 | Widening $K$ from 8 to 16 at evaluation time on four saved checkpoints | Admitted at the end: 1,920 at $K=8$, 2,400 at $K=16$ (gate 4,894). Earlier checkpoints reached higher ceilings | negative | [Stage A sweep](../../archive/labs/rung0-stage-a-sweep-2026-10-15/README.md) |
| 2026-10-17 | Host re-score, $K=8$ against $K=16$ on the 1,920 admitted rows | Top-1 149 to 138; raising $K$ alone did not help | inconclusive | [host diagnostics](../../archive/labs/host-diagnostics-2026-10-17/README.md) |
| 2026-10-17 | Zero-training preflight on the seeded codebook | Start: $P_{succ}$ 2,034 of 3,847; full panel 7,500 of 14,287 | positive | [preflight](../../archive/labs/attention-preflight-2026-10-17/README.md) |
| 2026-10-18 | Codebook coverage census | The start codebook had 16 distinct vectors for 4,096 tokens (256 each); 391 of 16,384 attempted moves were accepted by the end | inconclusive | [alias census](../labs/course-correction-2026-10-18/alias-census-REPORT.md) |
| 2026-10-18 | Dose 1 with Relation, stopped at Q1 | $P_{succ}$ admission 2,034 to 225 (Relation), 205 (control) | diagnostic-stop | [Q1 diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) |
| 2026-10-19 | K16-A and K16-R: two objectives x two seeds, Route fixed | Admitted: A 2,237 and 3,167; R 12,089 and 12,466 (gate 4,894). Occupied cells: A 209 and 192; R 20 and 19. Top-1: A 65 and 463; R 101 and 230 (bar 1,941) | negative (configuration) | [K16 outcome](../labs/k16-outcome-call-2026-10-19/README.md) |
| 2026-10-19 | Injective codebook with uniform cell spread | Aliasing fixed, but $P_{succ}$ admission was 127 of 3,847 against a gate of 500; not dispatched | diagnostic-stop | [injective preflight](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md) |
| 2026-10-19 | Zero-update test of placing each token's cell near its predecessors' cells | $P_{succ}$ admitted: address-structured 1,351, uniform 127, shuffled 476 of 3,847 (3.3% to 35.1%); at $K=16$, 2,178 | positive | [address cells](../labs/address-cells-preflight-2026-10-19/README.md) |
| 2026-10-19 | A8 dose, Relation arm and trit-locked control | $P_{succ}$ at 4,096 batches: 1,351 to 857 (control, 37% lost) and 551 (Relation, 59% lost) | diagnostic-stop | [A8 dose](../labs/a8-dose-2026-10-19/README.md) |
| 2026-10-19 | Admission-preserving rerun, pre-registered | Passed its fitness review; nothing run | inconclusive | [admission-preserving](../labs/admission-preserving-2026-10-19/README.md) |
| 2026-10-26 | CSR cell-bucketed CUDA writer (`m1-faster-writer`, `#2840`): 256-cell CSR inverted index (`bucket.cu`) qualified on `1× RTX 5090` | `0 / 14,287` mismatches on `A8` (`top1 = 133`) and `PrimeZeta` (`top1 = 38`) `DEV` panels and `0` mismatches across `V ∈ {64, 256, 4096, 16384}`; per-row comparisons `39,270` $\to$ `384` (`102.26×` `IdOrder`, `81,533 tok/s`) and `272` (`144.37×` `ServingPruned`, `106,648 tok/s` vs `1,790 tok/s` full scan) at `V = 4,096`, and `150,243` $\to$ `765` (`196.15×` `IdOrder`) / `592` (`253.50×` `ServingPruned`) at `V = 16,384` | positive | [m1-faster-writer](../labs/m1-faster-writer/README.md) |

**The K16-R pass is hollow.** Objective R collapsed the codebook onto about 20 occupied cells. Each row then admitted about 3,400 of the 4,096 tokens, so "admission" was nearly pass-through. The registered occupancy check fired on both seeds, and the call was no KEEP. Objective A kept 192 to 209 cells but lost admission. Both are negatives for these exact configurations, and D22 says no mechanism family is closed by them. Widening $K$ at evaluation time closes only that route for that codebook; lineages trained with $K>8$ remain open.

## What has never been tested

- Admission-preserving acceptance during learning (registered and fit, never run).
- A lineage trained at $K>8$ with distinct, address-structured cells, to full dose.
- Ranking once admission is intact. No run has had admission intact and then succeeded or failed at ranking.
- An address that uses more than the first and last block of $q$ (full-vector, interior-block or cell-envelope).
- An occupancy-regularised objective that does not collapse to about 20 cells.
- Fresh-slice transfer of any admission result. The sealed 2,000-row slice has never been opened.
- Whether admission capacity trained in at $K=8$ can reach the 4,894 gate.

## What comes next

1. **Admission-preserving A8 rerun** with the 64-batch anchor guard. The controller is merged (`UOR-Foundation/uor-r4#2782`). Finite qualification and an owner-typed release are outstanding.
2. **Occupancy-regularised or load-balanced rank objective**, the registered route after the K16 collapse.
3. **A finer address metric** if admission stays capped at $K=16$.
4. **$K>8$ on an injective, address-structured codebook.**

Open questions: why learning moves targets out of the cells that predecessor placement chose, and how fast; whether the guard's 24-row margin rejects so much that learning becomes a no-op; whether 256 cells are too coarse; and when ranking, not admission, becomes the bottleneck.

## Also known as

Kept cells or $K$ shortlist; target admission; Q1 admission collapse; the B3_init and G-Cap gates (starting and capacity checks on reachable targets); $P_{succ}$ admission; G-Adm and G-Occ (the admission and occupancy gates); the admission-preserving guard.

## Code

- [`discrete.cu`](../../crates/uor-r4-discrete/src/discrete.cu): `address_distance` (line 274), `token_cell` (286), `cell_precedes` (301) and the writer (307-345).
- [`objective.rs`](../../crates/uor-r4-discrete/src/objective.rs): objectives A and R. [`coverage_census.rs`](../../crates/uor-r4-discrete/src/coverage_census.rs), [`address_cells.rs`](../../crates/uor-r4-discrete/src/address_cells.rs), [`prime_zeta.rs`](../../crates/uor-r4-discrete/src/prime_zeta.rs): codebook census and placement.
- [`attention_anchor.rs`](../../crates/uor-r4-discrete/src/attention_anchor.rs), [`attention_audit.rs`](../../crates/uor-r4-discrete/src/attention_audit.rs), [`attention_audited.rs`](../../crates/uor-r4-discrete/src/attention_audited.rs): the guard.
- [`rows_device.rs`](../../crates/uor-r4-discrete/src/rows_device.rs): the row-parallel scoring, about 56 times faster, that made the K16 and A8 doses affordable.
- [`bucket_source.rs`](../../crates/uor-r4-discrete/src/bucket_source.rs), [`writer_bucket_device.rs`](../../crates/uor-r4-discrete/src/writer_bucket_device.rs), [`writer_bucket_qualification.rs`](../../crates/uor-r4-discrete/src/writer_bucket_qualification.rs), and [`address_cells.rs`](../../crates/uor-r4-discrete/src/address_cells.rs) (`WriterCellIndex`): the 256-cell CSR inverted index writer (`m1-faster-writer`, `#2840`), cutting per-row comparisons by `102×`–`253×` (`45×`–`123×` direct writer `tokens/s` speedup) with `0` parity mismatches against the full-vocabulary scan.

## Related mechanisms

[address-cells](./address-cells.md) sets which cell each token lives in. [prime-zeta-codebook](./prime-zeta-codebook.md) is the injective initialiser. [route-read](./route-read.md) is the read whose output admission consumes. [relation-read](./relation-read.md) and [flock-read](./flock-read.md) had doses stopped by admission loss. [rank-margin](./rank-margin.md) is the hinge family. [discrete-learning](./discrete-learning.md) is the move law that trains the table.
