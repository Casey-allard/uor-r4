# W(3,3) geometry qualification — measurement record and verdict (#845)

- **Issue:** #845 — "research/#826-C: qualify W(3,3) against Hamming, binary, VSA, and spectral
  baselines" (item C of S4 tracker #826, programme #820).
- **Date:** 2026-08-22. **Records are append-only.**
- **Frozen contract:** [`docs/w33_geometry_qualification_spec_845.md`](w33_geometry_qualification_spec_845.md)
  (§5/§5-A statistics pre-registered before this run), measured under Amendment A2 of
  [`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) §12.
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md).
- **Execution scope:** offline / reference geometry evaluated through fixed production-planning
  readouts; certifier-instrument / off-serving-path. Nothing here is deployed-serving evidence,
  and no production code changed anywhere in #845.

---

## 1. Verdict — NO GEOMETRIC ADVANTAGE

**Empirical Criterion (S4 item C outcome). Status: Empirical.**

> Under the frozen equal-budget, equal-byte comparison, the pinned W(3,3) mapping provides **no
> measurable planning advantage** on either pre-registered axis. On the budget axis it does
> strictly **more** search work than the incumbent non-geometric ordering in every cell where
> reduction is structurally possible (0 of 12 reduction/no-regression conjunction cells beyond the
> five degenerate ones). On the correctness axis it fails all nine primary tightened-budget cells —
> and sits at the level of its **own randomized and scrambled controls**, while two non-geometric
> orderings (goal-distance, Hamming/popcount) hold correct-outcome rate 1.0000 in the same cells.
> The pre-registered negative branch applies: W(3,3) stays in exploratory/reference scope, the
> production planner remains the lowered non-geometric baseline, and this is a **successful
> falsification**, not an incomplete measurement.

The programme falsifier this verdict discharges (completion plan, global falsifiers): *"if a
geometry arm cannot beat Hamming/binary/VSA/spectral controls under equal bytes, candidates, and
operation budgets, keep it out of the production runtime."* It is kept out.

## 2. What was measured

Both axes ran on 2026-08-22 in one deterministic pass (`w33_measurement_845.rs`, ignored grid;
single-threaded, teacher-free, fixture-free, ~50 s): n = **512** held-out joint-split instances
per cell, the frozen #844 identities with Amendment A1+A2, the deployed capacities, and the arm
roster of spec §4/§4-A — the W(3,3) geometry arm, eight non-geometric ordering controls
(goal-distance incumbent, Hamming/popcount, learned-table-codes, VSA-binding, spectral-embedding,
random-embedding, isomorphic-relabel, phase-permuted), the `bounded-breadth-first` baseline, and
the four #843 nulls, all under one `PlanBudget` per cell, all in arm mode on the parity-proven
reference skeleton.

## 3. A2(a) — the budget axis (12 separating cells, frozen budget)

Geometry holds correct-outcome **1.0000 in all 12 cells** (the mapping does no harm at the frozen
budget). It never reduces work: the bar (strongest perfect non-geometric ordering) is
`table-guided-beam` in every cell.

| cell | class | bar mean exp | geometry mean exp | paired r̄ | LB(95%) | reading |
|---|---|---|---|---|---|---|
| graph-navigation H=1 | no-regression | 1.0 | 1.0 | 0.0000 | 0.0000 | PASS (degenerate) |
| graph-navigation H=2 | no-regression | 2.1 | 2.6 | −0.2461 | −0.2735 | fail |
| graph-navigation H=4 | reduction | 12.8 | 17.6 | −0.3704 | −0.3877 | fail |
| graph-navigation H=8 | reduction | 88.5 | 99.9 | −0.1276 | −0.1342 | fail |
| symbolic-transformation H=1 | no-regression | 1.0 | 1.0 | 0.0000 | 0.0000 | PASS (degenerate) |
| constraint-satisfaction H=1 | no-regression | 1.0 | 1.0 | 0.0000 | 0.0000 | PASS (degenerate) |
| constraint-satisfaction H=8 | reduction | 84.8 | 96.2 | −0.1355 | −0.1424 | fail |
| multi-hop-evidence H=1 | no-regression | 1.0 | 1.0 | 0.0000 | 0.0000 | PASS (degenerate) |
| multi-hop-evidence H=2 | no-regression | 2.1 | 2.6 | −0.2461 | −0.2735 | fail |
| multi-hop-evidence H=4 | reduction | 12.8 | 17.6 | −0.3704 | −0.3877 | fail |
| multi-hop-evidence H=8 | reduction | 88.5 | 99.9 | −0.1276 | −0.1342 | fail |
| counterfactual-intervention H=1 | no-regression | 1.0 | 1.0 | 0.0000 | 0.0000 | PASS (degenerate) |

**Conjunction: FAIL (5/12; the five passes are the structurally degenerate H = 1 cells; Holm:
0/12).** The §5-A classification confirmed its pre-registered expectation exactly: the five H = 1
cells are no-regression cells; the H ≥ 2 cells carry real headroom (up to 0.91) — headroom the
geometry ordering not only failed to take but *spent against*, doing 12–37% more work than the
goal-distance beam.

## 4. A2(b) — the correctness axis (9 primary + 9 secondary cells)

The bar (strongest of baseline, nulls, and non-geometric ordering controls, all under the cell's
budget) is an *informed non-geometric ordering* in every primary cell — never a null.

| cell | kind | geometry | bar | LB(95%) vs bar | reading |
|---|---|---|---|---|---|
| graph-navigation frontier-16 | PRIMARY | 0.6230 | hamming-popcount 1.0000 | −0.4122 | fail |
| graph-navigation frontier-8 | PRIMARY | 0.2656 | table-guided-beam 1.0000 | −0.7665 | fail |
| graph-navigation frontier-4 | PRIMARY | 0.0742 | table-guided-beam 1.0000 | −0.9448 | fail |
| constraint-satisfaction frontier-16 | PRIMARY | 0.6426 | hamming-popcount 1.0000 | −0.3923 | fail |
| constraint-satisfaction frontier-8 | PRIMARY | 0.2617 | table-guided-beam 1.0000 | −0.7702 | fail |
| constraint-satisfaction frontier-4 | PRIMARY | 0.1387 | table-guided-beam 0.9961 | −0.8828 | fail |
| multi-hop-evidence frontier-16 | PRIMARY | 0.6230 | hamming-popcount 1.0000 | −0.4122 | fail |
| multi-hop-evidence frontier-8 | PRIMARY | 0.2656 | table-guided-beam 1.0000 | −0.7665 | fail |
| multi-hop-evidence frontier-4 | PRIMARY | 0.0742 | table-guided-beam 1.0000 | −0.9448 | fail |

**Conjunction: FAIL (0/9; Holm 0/9).** All nine secondary cells (the expansion ladder, where the
baseline is fully collapsed and the bar degenerates to `direct-continuation`, and the TIGHT frozen
H = 16 cells) also fail, 0/9 — at H = 16 geometry reaches 0.9707–0.9824 while Hamming holds
1.0000.

## 5. Attribution — where the signal actually is, and is not

The per-arm table at graph-navigation H = 8 frontier-16 (constraint-satisfaction and
multi-hop-evidence read the same within noise):

| arm | rate | | arm | rate |
|---|---|---|---|---|
| bounded-breadth-first (baseline) | 0.5391 | | **w33-geometry** | **0.6230** |
| table-guided-beam | 1.0000 | | random-embedding | 0.6641 |
| hamming-popcount | 1.0000 | | isomorphic-relabel | 0.6445 |
| learned-table-codes | 0.8945 | | phase-permuted | 0.7090 |
| spectral-embedding | 0.8965 | | vsa-binding | 0.7871 |

Three attributions follow, each pre-registered as a reading rule before the run:

1. **The geometry arm sits at the level of its own randomized and scrambled controls** — random
   tables of the same shape and bytes (0.6641), the alignment-destroying relabel (0.6445), and the
   phase-swapped variant (0.7090) all match or slightly exceed it. Whatever the W(3,3) tables
   contribute over FIFO is what *any* table of that shape contributes: the generic
   informed-ordering effect, not quadrangle structure.
2. **The small gain over the baseline attributes to "any ordering", not geometry** (the §5
   attribution rule): every ordering arm beats FIFO's 0.5391; the non-geometric informed ones beat
   it to saturation.
3. **The pinned phase convention is mildly anti-aligned on these tasks** — the phase-swapped
   control outperforms the true phase (0.7090 vs 0.6230). A sign flip would not rescue the arm:
   0.7090 is still far below the 1.0000 non-geometric bars. Recorded, not optimized against.

The mechanism is not mysterious: μ quantizes slot values through mod-9 digit residues, and the
wrap-around of residue classes destroys the monotone relationship between slot distance and
task-step distance that the goal-distance and Hamming orderings exploit directly. A 40-point
incidence geometry over wrapped digit classes carries almost no information about how many grid
steps remain.

## 6. Verification behind the numbers

- **Counter-exact parity (Structural).** The reference skeleton reproduces the deployed planner —
  outcome, plan, every `PlanCounters` field — for both parity retention rules over a
  3-family × 3-horizon × 4-budget × 32-seed grid (>1,000 episodes per rule), machine-checked in
  `w33_ordering_harness_845.rs` before this measurement ran.
- **Equal budgets, equal bytes (Structural).** One `PlanBudget` per cell for every arm and null;
  every control's auxiliary tables at or under the geometry arm's 3,200 bytes, audited by test.
- **Controls fire and fail (Structural).** Determinism and separation asserted per arm; the
  relabel scramble asserted non-automorphic; the A2(a) classifier and both pass rules asserted to
  fire in both directions on synthetic cells; the A2(b) bar asserted able to fire.
- **Reproduction.** Deterministic, seedless beyond pinned constants:
  `cargo test -p uor-r4-graph-certify --release --test w33_measurement_845 -- --ignored --nocapture`.

## 7. Boundaries — what this verdict does and does not say

It says exactly: **the pinned μ mapping of W(3,3) into the #844 planning benchmark provides no
equal-budget advantage on the measured tasks, horizons, and budgets.** It does not refute W(3,3)
as mathematics; it does not touch the f64 geometric router's separately measured content-retrieval
results (MRR 0.88+, #486/#490/#502), which are outside the P-4 kernel by design; and it licenses
no claim about other mappings. **Re-entry condition:** a new mapping Definition (not mod-9 digit
quantization) with a pre-registered alignment rationale, a fresh design contract and probe, under
the same frozen controls and budgets. Absent that, W(3,3) remains a visualization/reference
construct and no geometry enters the production runtime.

## 8. Downstream

- **#846 (S4 item D)** inherits a fully-dispositioned item C: certification proceeds against the
  lowered non-geometric baseline (RF-33 `bounded-breadth-first`) on the sealed partitions #843
  never opened; the S4 promotion verdict remains #846's to make, with the geometry-gain clause of
  the #826 gate resolved negatively by this record.
- The #824 free-running-coherence and #823 calibrated-confidence boundaries stand untouched.
- `CONFORMANCE.md` is unchanged by #845 (no new RF id; reference/certifier scope only).
