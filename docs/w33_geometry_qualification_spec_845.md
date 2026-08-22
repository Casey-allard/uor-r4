# W(3,3) geometry qualification — design contract (#845)

- **Issue:** #845 — "research/#826-C: qualify W(3,3) against Hamming, binary, VSA, and spectral
  baselines" (item C of S4 tracker #826, programme #820).
- **Status:** frozen design contract (increment 1 of #845). Amendments append; nothing here is
  rewritten.
- **Provenance:** the 2026-08-22 maintainer decision on #845 (its decision comment is the source of
  record) and Amendment A2 of the benchmark constitution
  ([`compositional_planning_spec_844.md`](compositional_planning_spec_844.md) §12).
- **Claim language:** normative per [`docs/formal_vocabulary.md`](formal_vocabulary.md).
- **Execution scope:** offline / reference geometry evaluated through fixed production-planning
  readouts; the measuring harness is certifier-instrument / off-serving-path. Nothing in this issue
  is deployed-serving evidence, and no production crate changes. "Reasoning" in any public phrase
  stays limited to the exact typed state-transition/planning tasks measured.

---

## 1. Decision record — how this issue became executable

Three recorded facts precede this contract; none of them is revisited below.

1. **The #859 edge is converted, not ignored.** F0 (#859) stays held and unstarted. In its place:
   the W(3,3) object every geometry arm uses is pinned by the **Definitions in §2–§3 of this
   contract** (formal-vocabulary scope), and a `PROMOTE FOR LOWERING` verdict does **not** by
   itself authorize production lowering — the separate lowering issue it would open must
   re-acquire the exact #859 theorem/library pin before any production geometry lands. Formal
   backing gates *lowering*; it never substitutes for *measurement* (this issue's own rule).
2. **The zero-ceiling finding.** The lowered baseline (RF-33 `bounded-breadth-first`) is at
   correct-outcome rate 1.0000 in all 20 frozen cells, so no arm can clear δ_min over it on the
   frozen primary statistic anywhere on the frozen grid. Amendment A2 (#844 spec §12) is the
   sanctioned repair: a **budget axis** on the 12 separating cells and a **probe-admitted
   correctness axis** on tightened-budget cells, with every frozen value standing.
3. **The binding cheap instrument ran first and passed.** The failure-surface probe
   (`geometry_probe_845.rs`, 2026-08-22: 70 cells, 33.5 s) found 18 admissible cells and froze
   A2(b)'s nine primary cells; the greedy-solvable families admitted zero cells at every setting.
   Probe numbers are recorded in §6 and in A2; the instrument ships with increment 2.

## 2. The normative W(3,3) — reference definitions

Everything in this section is **Definition** scope (reference mathematics). No semantic usefulness,
activation, or empirical superiority is asserted by any of it; §5 is where such claims would be
earned or refused.

**Definition (the symplectic quadrangle W(3), here called W(3,3)).** Let V = GF(3)⁴ carry the
non-degenerate alternating bilinear form

    ⟨u, v⟩ = u₀v₁ − u₁v₀ + u₂v₃ − u₃v₂  (mod 3).

The **points** of W(3) are the 40 projective points of PG(3,3); the **lines** are the 40 totally
isotropic projective lines of the form. Two distinct points p = [u], q = [v] are **collinear**
exactly when ⟨u, v⟩ = 0 (the span is then totally isotropic because the form is alternating). This
is the generalized quadrangle of order (3,3): every line carries 4 points, every point lies on 4
lines, and for a point P not on a line L exactly one point of L is collinear with P.

**Definition (canonical representative).** Every projective point has exactly one representative
whose first nonzero coordinate is 1; all point-indexed tables below are indexed by these 40
canonical representatives in lexicographic order. This convention is part of the definition.

**Definition (collinearity distance d_W).** d_W(p, q) ∈ {0, 1, 2}: 0 iff p = q, 1 iff p, q are
collinear and distinct, 2 otherwise. The collinearity graph is strongly regular with parameters
(40, 12, 2, 4), so d_W is total with diameter 2.

**Definition (phase φ).** For canonical representatives u_p, u_q: φ(p, q) = ⟨u_p, u_q⟩ ∈ GF(3).
φ(p, q) = 0 iff p = q or p, q are collinear; φ(p, q) = −φ(q, p) (mod 3). The value depends on the
pinned form and the canonical-representative convention; that dependence is deliberate and is what
the adversarial phase-permutation control (§4) exercises.

**Definition (the 96-vertex canvas, reconciled by disambiguation).** The dashboard's "96-vertex
W(3,3) phase field" (`index.html`, `w33Count = 96`) renders 96 points on a Euclidean sphere at
angles θ = 2πi/96, φ = π((i mod 12)/12) − π/2, with per-vertex activity keyed to a serving expert
count. It has no incidence structure, no GF(3) content, and neither 40, 80 (the point count and the
incidence-graph order of W(3)), nor any other invariant of the quadrangle. It is a rendering motif
and is **non-normative for this issue**: the operative W(3,3) for every geometry arm is the
Definition above. This resolves the entry-gate inconsistency recorded in the issue body — the
inconsistency was between a visualization label and the mathematical object, and it is resolved by
naming which one binds. (Consistent with the mechanism-boundaries table of the completion plan,
which already records the canvas as not a normative construction of the 40-point/40-line
quadrangle.)

## 3. The mapping μ and its readout semantics

**Definition (state digitization μ).** A planner state is a `SlotVec` s = (s₀, s₁) of i16 slots
(#843 typed surface). Let aⱼ = sⱼ mod 9 taken in {0,…,8} (`rem_euclid`), and write aⱼ in base 3 as
(aⱼ mod 3, ⌊aⱼ/3⌋). Then

    v(s) = (a₀ mod 3, ⌊a₀/3⌋, a₁ mod 3, ⌊a₁/3⌋) ∈ GF(3)⁴,

and μ(s) is the projective class of v(s) when v(s) ≠ 0, and the basepoint [1:0:0:0] when
v(s) = 0. μ is total and deterministic; it reads only slot values, never entity names, operator
vocabularies, or goal templates, so it is invariant under every surface axis by construction.

**Definition (surjectivity and fibers — μ is a quantization).** As (a₀, a₁) ranges over the 81
residue pairs, v(s) covers all of GF(3)⁴, so μ is onto the 40 points; each point's fiber is a
union of residue classes of period 9 per slot. μ deliberately identifies states — it is a
geometric bucketing, not an encoding. The **readout** of a point is its fiber: geometry never
replaces a state, it only *orders* candidate successors, and the planner's states remain the exact
`SlotVec` states throughout (the "replace missing state/action evidence with geometry" non-goal is
honored structurally, not by promise).

**Definition (goal and candidate scoring).** For a task with goal-region center g and a candidate
successor state s′, the geometry score is the triple

    G(s′, g) = ( d_W(μ(s′), μ(g)), t(φ(μ(s′), μ(g))), canonical effect order ),

ordered lexicographically ascending, where t is the pinned order 0 < 1 < 2 on GF(3). A geometry-
guided search expands candidates in G-order under exactly the same bounded search skeleton,
budgets, and counters as every control arm (§4).

**Definition (integer realization; projected lowering).** d_W and φ are 40 × 40 tables (one byte
per entry; 3,200 bytes total). μ needs per-slot residues mod 9, realizable without division or
multiplication: a 256-entry table reduces each byte, and a 9 × 9 × 4 composition table combines
byte residues (16-bit slot = high byte contribution (r_hi · 256 mod 9 = r_hi · 4 mod 9, a 9-entry
table) added to r_lo, then one 81-entry digit-split table). Total fixed tables under 4 KiB,
touched by table reads, adds, and compares only. **Assumption (recorded):** this projection shows a
P-4-legal lowering *exists*; this issue implements the reference arms offline and does not lower
anything — lowering is the separate issue a positive verdict would open, under the §1 Lean-pin
condition.

## 4. Arms, controls, and the equal-budget / equal-byte harness

**Definition (the search skeleton).** One reference bounded best-first search with a pluggable
candidate-ordering functional, mirroring the deployed planner's budget accounting exactly
(`PlanBudget` fields; `PlanCounters` semantics). Increment 3 asserts counter-parity: with the
no-ordering (FIFO) functional the skeleton reproduces the deployed `bounded-breadth-first`
episode outcomes and counters on the probe grid. Every arm below is this one skeleton plus an
ordering; nothing else differs.

**The geometry arm.** G-ordering per §3, from the fixed d_W/φ tables.

**Non-geometric ordering controls** (each byte-matched to the geometry tables' exact byte count —
padded with dead bytes or truncated by construction — and each fitted, where fitted at all, on the
fitting half only):

- **Hamming/popcount:** ascending `popcount(bits(s′) XOR bits(g))` on the raw 32-bit slot pair.
- **Learned binary codes:** a per-residue-class k-bit code fitted deterministically from the
  fitting half's observed goal-displacement statistics; ascending code Hamming distance.
- **VSA/binding:** slot-role hypervectors bound by XOR and bundled by majority, fitted on the
  fitting half; ascending Hamming distance of bound state/goal vectors.
- **Spectral:** an offline f64 Laplacian eigen-embedding of the induced rule graph (compiler-side
  f64 is in scope for fitting only), quantized to integer tables; ascending embedded distance.
- **Random embedding:** a seed-pinned random 40-point table of the same shape as d_W/φ.
- **Isomorphic relabel:** the geometry tables conjugated by a pinned nontrivial permutation of the
  40 points arising from a relabeling of the underlying states — if G-order's value survives with
  the mapping's semantic alignment destroyed, the gain was not the claimed geometry.
- **Adversarial phase permutation:** d_W kept, φ replaced by a pinned non-identity permutation of
  its values — isolates whether the phase refinement specifically carries signal.
- **No-geometry baselines:** `bounded-breadth-first` (the lowered RF-33 arm, the bar of A2) and
  `table-guided-beam` (support ordering, the recorded equal-scoring alternative).

**Nulls.** The four #843 nulls unchanged (retrieval-only, direct-continuation,
memorized-trajectory, shuffled-state), fitted and measured exactly as in the #843 harness.

The exact fitted-control constants (k, hypervector width, eigen-count, seeds) are pinned in
increment 3 as an appended section (§4-A) before any measurement runs; their fitting data access
(fitting half only), determinism, byte parity, and non-degeneracy obligations are frozen here.
A control unable to fire or unable to fail voids the cell (`NOT TRIGGERED`), never a pass.

### 4-A. Pinned control constants (appended in increment 3, before any measurement)

Implementation: `crates/uor-r4-graph-certify/tests/support/{ordering,arms,episode}.rs`; every
constant below is asserted by `w33_ordering_harness_845.rs` before increment 4 runs.

- **The seam, and its two modes.** The reference skeleton is the deployed layered search with one
  seam: the ordering score. **Parity mode** applies it to frontier retention only and is
  machine-checked per episode — outcome, plan, and every `PlanCounters` field — against the
  deployed planner for both retention rules (FIFO ≡ breadth-first; goal-distance ≡ table-guided
  beam) over a 3-family × 3-horizon × 4-budget × 32-seed grid. **Arm mode** (the §3 "ordering
  functional for beam/best-first expansion") additionally expands the retained layer in descending
  score order (stable sort; equal scores keep the canonical arrival order), which is what lets an
  ordering reduce expansions by reaching the goal-generating expansion sooner — layers before the
  goal layer are swept in full regardless of order, so retention quality and last-layer order are
  exactly what an ordering can influence. Every measured arm and ordering control runs in arm mode
  under identical budget accounting; parity mode anchors that accounting to the deployed planner.
  Scoring work is outside `PlanCounters` (the deployed beam's goal-distance evaluations are
  uncounted there too); each arm reports auxiliary lookups and table bytes separately.
- **Byte budget** = the geometry arm's 3,200 bytes (two 40 × 40 tables); the audit asserts every
  control at or under it. Seeded tables use splitmix64.
- **w33-geometry:** retention −(4·d_W + t(φ)); 3,200 B; μ per §3.
- **hamming-popcount:** −popcount(raw 32-bit slot pair XOR goal); 0 B.
- **learned-table-codes:** 40 × 40 u8 mean remaining-gold-steps between mapped classes (255 =
  unseen) + 40 row-median-threshold 16-bit codes; retention −(32·table + code-Hamming); 1,680 B;
  fitted on fitting-half gold paths only. Deliberately the same 40-class quantization as the
  geometry arm: it isolates whether the quadrangle *structure*, not any table over μ's classes,
  carries the signal.
- **vsa-binding:** two roles × nine residues × 128-bit fillers, seed `0x845a_0001`, XOR binding;
  −Hamming of bound state/goal hypervectors; 288 B.
- **spectral-embedding:** Laplacian of the fitting-half class-transition graph; fixed-sweep cyclic
  Jacobi (32 sweeps, fixed pair order — deterministic by construction); kernel vector skipped;
  8 dimensions quantized ×1024 to i16; −L1; 640 B. f64 in fitting only (compiler/certifier scope).
- **random-embedding:** two 40 × 40 tables with values in {0, 1, 2}, seed `0x845a_0002`, scored
  with the geometry functional; 3,200 B — matched in bytes, shape, and range; devoid of structure.
- **isomorphic-relabel:** the true geometry tables conjugated by a pinned Fisher–Yates scramble of
  the 40 points, seed `0x845a_0003`, asserted **not** to be a collinearity automorphism (a
  symplectomorphism would leave d_W invariant and make the control vacuous); 3,200 B.
- **phase-permuted:** true d_W, φ values swapped 1 ↔ 2; 3,200 B.
- **table-guided-beam (incumbent):** −goal-distance, 0 B — the deployed beam's exact signal.

## 5. Statistics, generalization controls, and the verdict space

**Empirical Criterion (A2(a), the budget axis — co-primary). Status: pre-registered.** On the 12
separating cells at frozen terms: the geometry arm must (i) hold correct-outcome rate exactly equal
to the baseline's 1.0000 in every cell, and (ii) show a paired one-sided 95% lower bound of at
least **ρ_min = 0.10** relative reduction in expansions against the **strongest non-geometric
ordering control** that also holds 1.0000, intersection-union over the 12 cells, Holm reported
alongside. Candidates and table reads are co-reported; a correctness regression anywhere voids the
axis for the regressing arm.

### 5-A. A2(a) per-cell reading (appended in increment 3, before any measurement)

**The structural fact.** An episode that finds an L-step plan expands at least L states (the root
and each intermediate), and a horizon-1 episode expands exactly one state whatever the retention
rule — declines included (the root is expanded, the next layer is empty or unreached). Every arm
therefore spends *identically* one expansion in every H = 1 cell, the relative reduction is zero by
arithmetic identity, and a ρ_min bar can never be cleared there by any mechanism. A criterion a
correct arm cannot pass is as broken as one a wrong arm cannot fail — the #844 §11.6 shape, on the
budget axis.

**Definition (the per-cell reading; the 12-cell conjunction and every frozen value stand).**
Classify each A2(a) cell from *non-geometry* data alone (the bar arm's measured expansions against
the per-instance gold-length floor, so no geometry result is consulted): a cell whose bar arm's
mean relative headroom over the floor is at least ρ_min is a **reduction cell**, read exactly as
frozen (paired one-sided 95% LB of relative expansion reduction ≥ ρ_min = 0.10); a cell with less
structural headroom than ρ_min is a **no-regression cell**, read as: correctness exactly equal AND
the paired LB of relative reduction ≥ 0 (identical work reads as the degenerate LB = 0, which
passes; regression fails). Expected classification, to be confirmed by the run: the five H = 1
cells are no-regression cells; the seven H ≥ 2 cells are reduction cells. The intersection-union
conjunction still ranges over all 12 cells; Holm is still reported alongside.

**Empirical Criterion (A2(b), the correctness axis — co-primary). Status: pre-registered.** On the
nine frozen primary cells (#844 spec §12): the geometry arm's paired one-sided 95% lower bound over
the **bar arm** (the strongest of baseline, nulls, and non-geometric ordering controls in that
cell, all under the cell's budget) must clear **δ_min = 0.05** in every cell (intersection-union;
Holm alongside). The nine secondary cells are measured and reported with identical rigor and gate
nothing.

**Generalization and attribution controls (both axes).** Held-out evaluation is the joint split
throughout. Any positive cell is re-read under: surface relabeling (invariance expected — μ never
sees labels; verified, not assumed), the isomorphic-relabel and phase-permutation controls
(alignment-specificity), topology transfer (the held-out effect-set halves, as in #843), and the
attribution ladder — if a non-geometric ordering control matches the geometry arm within the
confidence bound in a cell, that cell attributes to *informed ordering generally* and does not
count toward the geometry claim. Trajectory stability, if reported at all, is a diagnostic and
never a gate.

**Definition (verdict space).**

- `PROMOTE FOR LOWERING` — A2(b) primary passes **and** A2(a) passes. Opens a separate packed-
  lowering issue carrying the §1 Lean-pin obligation and explicit P-4/witness obligations.
- `REVISE` — exactly one axis passes; record which, and what would change.
- `NO GEOMETRIC ADVANTAGE` — neither axis passes with all instruments non-vacuous. W(3,3) stays
  exploratory/reference; the strongest simpler planner stands. This is a successful falsification,
  not a failure of the issue.
- `NOT TRIGGERED` — an instrument or control failed non-vacuity and the affected axis could not be
  read; recorded per axis, never converted into a pass.

## 6. Run contract (probe-verified, 2026-08-22)

- **Metric / current value:** A2(a) — expansions at equal correctness on the 12 separating cells
  (baseline means at frozen budget: H=1 ≈ 1.0, H=2 ≈ 2.8–3.1, H=4 ≈ 17.5–19.8, H=8 ≈ 98.3–108.4
  per instance). A2(b) — correct-outcome rate on the nine primary cells (bars 0.7422–0.8340).
- **Reachability ceiling:** A2(b) headroom per primary cell = 1 − bar = 0.1660 (graph-navigation,
  multi-hop-evidence) and 0.2578 (constraint-satisfaction) — 3.3× and 5.2× δ_min; no primary cell
  is TIGHT. A2(a) ceiling: the H=8 baseline spends ≈ 100 expansions on goals of depth exactly 8, so
  an ordering can reach the same outcomes on a fraction of the wavefront; ρ_min = 0.10 sits far
  inside that envelope while H=1–2 cells (1–3 expansions) contribute near-zero reduction and are
  retained in the conjunction deliberately: the claim must survive the cells where reduction is
  structurally scarce or the conjunction reading would overstate it. If that conjunction is the
  binding obstacle, the recorded outcome is `REVISE` with the per-horizon decomposition, not a
  silently narrowed cell set.
- **Pinned identities:** benchmark = #844 constitution + A1 + A2 at this repository revision;
  splits = joint (fit low half / hold out high half on every axis); baseline = RF-33
  `bounded-breadth-first`; budgets = frozen `PlanBudget` and the A2(b) named tightenings; seeds =
  the deterministic seed walk of the #843 harness; all arm/control tables content-pinned in
  increments 2–3; harness = `geometry_probe_845.rs` (probe) plus the increment-3 measurement
  harness; reports CID-bound in the increment-4 record.
- **Nulls / falsifiers:** the four #843 nulls; random-embedding, isomorphic-relabel, and
  phase-permutation controls; equal-candidate/equal-operation budget parity asserted by counters.
- **Binding cheap instrument + verdict:** the failure-surface probe (this document §1.3) — **ran
  first, PASSED**: 18 admissible cells; A2(b) triggered; both non-vacuity gates green. Increment 3
  re-runs the probe's saturated-cell reproduction check before the measurement grid.
- **Exit rule:** §5 verdict space, pre-declared.
- **If positive:** open the packed-lowering issue (Lean-pin + P-4/witness obligations); #846
  certifies against sealed partitions.
- **If negative:** publish `NO GEOMETRIC ADVANTAGE`, keep W(3,3) exploratory/reference, proceed to
  #846 with the non-geometric baseline. The next action differs from the positive branch in both
  artifact and successor issue, so the run has decision value in both directions.
- **Cost estimate:** the probe measured 70 cells in 33.5 s single-threaded, teacher-free; the full
  measurement grid (≈ 21 gating cells × ≈ 12 arms/controls/nulls) projects to minutes, not hours;
  no shared-compute contention; peak RSS well under one workspace test binary.

## 7. Conformance, compatibility, and documentation

- Research/reference only: no production format, runtime, or dependency changes; no new RF id in
  this issue (RF-08/12/13/27/28 are context, and existing reference IDs are not deployed-planning
  evidence); `CONFORMANCE.md` untouched; `model/ledger.toml` route-fit entries unchanged.
- All code lands as certifier-instrument tests in `crates/uor-r4-graph-certify/tests/`; the four
  local gates plus the applicable ladder run per increment; claim wording is CI-gated.
- The measurement record will be `docs/w33_geometry_qualification_845.md` (append-only), and
  claim-changing results reconcile README/RESEARCH/plan summaries per the repository rule.

## 8. Increments

1. **This design freeze** (docs only): this contract, Amendment A2, the plan mirror entry.
2. **Mapping and instruments:** W(3,3) tables + μ + round-trip/isomorphism/metamorphic tests +
   the probe instrument shipped as `geometry_probe_845.rs` with its non-vacuity gates.
3. **Arms and harness:** the ordering skeleton with counter-parity assertion, all controls with
   pinned constants (appended here as §4-A), byte-parity audit.
4. **Measurement and verdict:** both axes, the record, tracker/source-of-record updates, and the
   #845 closure with its DoD evidence.
