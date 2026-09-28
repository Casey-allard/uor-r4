# D5 addressing contest: result (T2, Lab 2 OpenCode)

September 28, 2026. References #973 under #820. **Predeclared plan frozen at
`docs/integration/addressing-contest-plan-2026-09-27.md`; the frozen decision rule is the
director's §4.2 rule. This run adjudicates it; the instrument never adjudicates.**
Preserves D11 (native, multiplier-free, transformerless serving). No training, no model
fitting, no serving-default change.

## What ran

Parent `fit-quaternion-6/checkpoint-final` (step 15,672, continuous, Full admission). The
offline `addressing_arms.rs` codebook/PQ-ADC module and `examples/joint-addressing-contest.rs`
(scored every arm in one pass over the Full-run states), plus a thin in-crate
`joint-evaluate-addressing` that runs the **existing** fixed-weight policy evaluation
(`evaluate_loaded`, same weights, whole prefix) with the contest admission arms. Report
roots under `/Volumes/UOR-Workspace/uor-r4-lab/opencode-addressing-contest/` (`full-2` for
the one-pass metrics; `restricted-r2-*` for the restricted reads).

## Outcome A — fidelity to the dense reader (selected events, mass, output)

Comparison tail (233,472 targets; s=16):

| arm | recall@1 | recall@3 | captured mass | Read NLL | ΔNLL vs full | correct/233,472 |
|---|---|---|---|---|---|---|
| full (dense) | — | — | 1.0 | 1.996490 | — | 122,864 |
| oracle q·k (ceiling) | 0.9930 | 0.9755 | 0.8899 | 2.003056 | +0.006566 | 122,569 |
| **k-means-120 (best ordinary)** | **0.9813** | **0.9541** | 0.8796 | 2.003692 | +0.007201 | 122,487 |
| **h4-rht** | 0.9696 | 0.9376 | 0.8709 | 2.005804 | +0.009313 | 122,498 |
| h4 | 0.9681 | 0.9345 | 0.8664 | — | — | — |
| e8-rht | 0.9450 | 0.9032 | 0.8541 | 2.008573 | +0.012083 | 122,408 |
| sign4-rht | 0.8977 | 0.8418 | 0.8287 | 2.014547 | +0.018056 | 122,247 |
| recent-16 | 0.4452 | 0.4147 | 0.3492 | 2.291023 | +0.294533 | 113,991 |

Long-range slice (75,799 positions whose dense top-1 event is ≥32 tokens back, s=16):
h4-rht **0.9880** vs k-means-120 **0.9940** (0.6 pt behind). NoRead-selected subset
(51,680 positions, NoRead mass < 0.5) reported alongside; all three denominators are in the
report and are not conflated.

## Outcome B — retention and correct use of the required entity/relation/version

On the five condition-A distractor rows the required entity has **dense_qk_rank = 1** (it is
the top event by true query·key content) but **dense_rank 2–3** under the dense reader, and
the fixed codebooks **admit it** (h4/h4-rht/e8/sign4-rht rank it 2–3, `admitted_top16=true`),
while `recent-s` does not. The earlier oracle re-rank (published in the read-localization
unit) changes the five exposed rows from **0/5 to 5/5** by swapping the entity's read mass
with the top-1 distractor (S), with the matched control C at 0/5. **That is evidence about
ranking in those five cases, not evidence that the entity was absent, and no general
learned ranking repair is demonstrated.** An index that faithfully reproduces the dense
reader's wrong preference has not repaired recall — the fixed content index correctly
surfaces the entity, which is precisely why its recall is high but its output still differs
from a corrected reader.

## Frozen gate outcome

A geometric arm qualifies only if it is within **1 point of recall@s** and **0.005 nats** of
the best ordinary arm at the same s, **with a cheaper multiplier-free decode**.

- nats (s=16): h4-rht +0.009313 vs k-means-120 +0.007201 → **0.0021 nats behind** → inside 0.005.
- recall (s=16, comparison tail): 0.9696 vs 0.9813 → **1.17 points behind** → outside 1 point
  (within the margin only on the ≥32-token long-range slice, 0.6 pt).
- decode: h4 and k-means-120 have **identical declared arithmetic** (7,936 table reads, 6,016
  adds, 14 bytes/event) → **not cheaper** (the fixed codebook's only structural saving — no
  learned codebook to store/fetch — is not priced by this accounting).

**Outcome: NOT QUALIFIED.** Per the frozen rule this index configuration is retired for
serving; this is a negative for this configuration at T=256 and these bit rates, **not** a
disproof of geometric mechanisms generally. No seeds, dose or new mechanism are added after
this endpoint.

H4 and E8 decode to unit-norm roots, and k-means to raw centroids, with no gain channel in
any arm (`addressing_arms.rs` lines 713-725). The geometric-versus-ordinary contrast is
therefore confounded by magnitude, and D3's decision moves to the gain-controlled
follow-up (ruling 11).

## Complete query cost (declared; no postings/multi-probe in this configuration)

The reported `decode_work` covers the query LUT and the admitted-`s` decode. The full scan
is derived from the per-event accounting and the fixed block layout (mean 128 candidates per
position, max 255):

| component (per query) | h4-rht / k-means-120 | e8-rht | sign4-rht | recent-16 |
|---|---|---|---|---|
| tables constructed (LUT) | 7,680 reads + 5,760 adds | 15,360 reads + 13,440 adds | 448 reads + 14,672 adds | 0 |
| traversal (full scan, 128 cand.) | 2,048 reads + 2,048 adds | 1,024 reads + 1,024 adds | 2,048 reads + 2,048 adds | 0 |
| entries/bytes fetched (128 cand.) | 1,792 B | 1,024 B | 1,792 B | 0 |
| index bytes/event | 14 | 8 | 14 | 0 |

**Dense work remaining outside the index (unchanged by every arm):** the model's read/state
projections, value mixing and the 4096-token vocabulary head remain per-token dense; the
parent has ~1.68 M dense parameters. Every previous event is scored from its codes (a
compressed exhaustive scan: 128 candidates on average, 255 at most). Only the number of
*retained* events whose values are read falls to s. Inspected and retained events are
reported separately, and this run claims no sparse-index cost. This is **not** D5
completion, and no full-parameter sparsity is claimed.

## Consumer interface (I4, per the board)

`admit` (the fixed codebook event index: `AddressingArm::encode` → `Codes`, PQ-ADC
`query_lut`/`score`, top-s) and `rank` (the same scores as a ranking) are exposed and
measured separately in `crates/uor-r4-training/src/addressing_arms.rs` (serde, deterministic,
`multiplies = 0`). The exact H4 decoder is **#1435** (Codex's exact signed H4 integer
classifier): reuse it for the H4 codebook's serving decode; this lab does **not** revive the
parked finite-H4 read-score experiment. The `AddressingArm`/`Codes`/`Lut` interface is handed
to T3 (Anti-Gravity) as the I4 seam.

## Blocking question answered

The remaining blocker is **learnable relational ranking**, not admission: the correct entity
is admitted and content-ranked first, but the dense reader's learned age/recency prior ranks
a more recent distractor above it and emission follows the read. A geometric content index
reproduces — rather than repairs — that ranking.

## Cost

Model compute (Full pass): tune 2.7 s, comparison 43.8 s; k-means fit 16.5 s (harness); arm
scoring 88.8 s (harness); restricted reads recorded per root. Reported separately from
orchestration. Charged once with the work unit; ledger reconciled in one line in
`current-state.md`.
