# Frozen-state readout adjudication (M1 Part A) — September 24, 2026 UTC

Executes **Part A** of the owner-adopted [`escape-bigram-class-plan-2026-09-24.md`](escape-bigram-class-plan-2026-09-24.md)
under [`direction-decision-2026-09-24.md`](direction-decision-2026-09-24.md). It is a **read-only, float-only
diagnostic on the artifact's own recorded served states** — no training, no served-arithmetic change, no artifact
promotion. It decides whether the local deficit is **readout/feature-limited** or **state/horizon-limited**.

Base `f6f0d3ac`. Worktree `…/uor-r4-worktrees/escape-bigram-20260924`, branch `codex/escape-bigram-20260924`.
Evidence root `/Users/casey.allard/uor-r4-investigations/escape-bigram-20260924` (`partA-run1…run6` sealed; `run6`
authoritative). Artifact `warm-decay-candidate.tlx` `69e8b88b…`.

## Method

The runner mode `--readout-adjudication` reuses the merged probe machinery (`record_served_states`, `ProbeSet`,
`fit_float_probe`/`fit_ridge_readout`) and fits readouts on the exact `h` the served readout receives, **document-held-
out** (16 fit / 8 evaluation of the 24 development documents; 60,257 fit positions, 1,649 held-out). Instrument
control: the artifact-initialised probe at epoch 0 reproduces the artifact's own K-class readout on the held-out
positions to **2.7e-15**; the base-convention assertion passes.

## Result (`partA-run6`, 288.3 s, held-out bits/target)

| arm | bits/target | top-1 |
|---|---|---|
| `A` artifact K-class readout | 5.7555 | 0.2304 |
| `A` full-vocabulary | 5.8357 | — |
| `U` unigram | 8.5690 | 0.0340 |
| `C` tuned order-2 count prior | 4.8041 | 0.3863 |
| (i) converged float linear (`P_softmax`) | **5.7048** | 0.2492 |
| (i′) closed-form ridge (`P_ridge`) | 7.7411 | 0.1710 |
| (ii) rank-K quadratic sketch (K*=64) | 7.7023 | 0.1789 |
| (iii) 2-layer MLP (best of 64/128) | 6.9112 | 0.1292 |
| **(v) count-augmented linear (decision arm)** | **4.7819** | **0.3875** |

Document-cluster paired intervals (bits(a)−bits(b); negative = a better):

- **(v) − C = −0.0223 [−0.0295, −0.0149]** → the state adds **+0.022 bits** over the tuned count table.
- (i) − A = −0.0507 [−0.0016, +0.1037] → the converged float readout is at least as good as the artifact readout
  (ordering reproduces the merged state probe; the merged −0.5423 was on 24 clusters and 5.5× the fit states).
- (iii) − (i) = **+1.2064 [0.9447, 1.4618]** → the MLP is **worse** than the converged linear readout.
- (ii) − (i) = +1.9976 [1.7560, 2.2311]; (i) − C = +0.9006 [0.7872, 1.0159].

**Decision rule.** `branch1-literal` false; **`branch1-substantive` true** (`|gain(v over C)| = 0.022 ≤ 0.05`);
`branch2` false (no nonlinear readout gain); `branch3` true (cited depth ladder: lag-2 recovery 74.86 % ≥ 60 %).

## Decision

**The state's local information is subsumed by a servable order-2 count table (0.022 bits), and no readout work —
linear, quadratic or MLP — is justified on this artifact.** The lever is the **state/horizon**, i.e. Part B. This
corroborates the merged count-blend negative (`+0.0049 [−0.0262, +0.0405]` tune-frozen) and the direction decision's
diagnosis, and it does **not** revive the addressed local `(prev,cur)` read.

## Caveats (recorded)

- Only 8 held-out documents; arm (i)'s gain over `A` is not significant at that cluster count.
- Arm (ii) is a **declared-seed closed-form quadratic sketch** (it inherits the closed-form ridge underfit); a truly
  learned bilinear head is not tested. Arm (iii) is Adam-trained and still loses to (i), so no nonlinear readout gain
  is measured **on this state**.
- `A` may have been trained on corpus content overlapping the evaluation documents, so it is not a document-held-out
  comparator; the arm-vs-arm and arm-vs-`C` comparisons are the primary evidence.
- The fit design (features, class count, param counts, λ grid, label convention, base assertion) is recorded in the
  sealed `receipt.json`.

## Limits

A feature-form measurement on one artifact's recorded states. **No** language, capability, reasoning, geometric, or
energy claim. It does not test a served readout, and it does not establish that no readout can help — only that none
of the tested readouts of this state adds materially over a servable count table.

## Next

**Part B** — the gate × capacity arm on a synthetic induction panel where the count control is at chance
([plan](escape-bigram-class-plan-2026-09-24.md) §3). The owner's "vectors and Hamiltonians" lead
([project-track.md](project-track.md) owner lead, 2026-09-23) attaches here as a **learned parameterisation of the
state/gate**, to be tested against a matched ordinary parameterisation at equal parameters — its implemented fixed-Q8
transport form already ties an ordinary signed-permutation control (1,024/1,024) and is not evidence of advantage.
