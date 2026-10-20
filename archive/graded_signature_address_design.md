# Design: f32 Residual-VQ Store + Mul-Free Graded-Signature Query Assignment

> **Historical design input.** This proposal is retained for comparison and
> does not define the current prime-route/harmonic address or work order. See
> the [Geometric Intelligence Programme](geometric_intelligence_programme.md).

*Issue #243. Status: DESIGN FOR REVIEW — decision points marked ⚑ are open
until Casey / Ari / Alex sign off on this document (review on the PR).
Claim language: Definitions, Objectives, and Empirical Criteria only.*

## Objective

Close the measured assignment-quality gap between the shipped mul-free
path and its own f32 control (#230: 3.9pp top-1, 4.5pp agreement at
certification-era parity conditions), without violating the runtime
contract (no multiply/divide/float in the deployed kernel, P-4
machine-checked), and treating every representational choice as
**address-space design** (docs/addr_prism_correspondence.md): the
quantization scheme decides collision structure, cacheability, and what
"near" means for #263's region objects and any future distributed serving.

## The gap, mechanically

The f32 ablation (`certify.rs`) L2-normalizes the thresholded bundle and
runs true residual VQ: per stage, nearest centroid by Euclidean distance,
then **subtract the centroid and re-quantize the residual**. The shipped
path thresholds once to a 288-bit sign signature and reuses it unchanged
at every stage — no residual update, no normalization, no magnitude.
Three information losses, in decreasing measured importance (Objective:
confirm ordering empirically in Phase B):

1. **No residuals** — later stages re-answer the question stage 1 already
   answered instead of refining what stage 1 got wrong.
2. **No magnitude** — a dimension barely over threshold and one far over
   it are the same bit.
3. **No normalization** — bundle norm varies with window content; sign
   thresholds absorb some of this but the f32 control shows headroom.

## Key enabler (settled, not ⚑)

`INFERENCE_OPERATION_CONTRACT.md` §4: compilation is explicitly
unrestricted. The store may be **built** with full f32 residual-VQ
assignment. Only the **query side** must remain in the integer kernel.
The design problem is therefore: a mul-free query-side approximation of
the compile-side assignment function, with a measured approximation gap.

## Candidate designs (benchmark both; ⚑ pick by measurement, not taste)

### Design R — integer residual subtraction
*(Risk note from code review: R's blast radius exceeds the query kernel —
stage-k class signatures (`art.class_sigs`) currently live in one
signature space and must be retrained in residual space, and `ctx_cb`
integer copies must be serialized (today marked certifier-side-only).
Higher ceiling, bigger change. Design G, by contrast, is a strict
generalization of the shipped path — b=1 is today's signature — with
width-variant Hamming kernels as its main engineering. Neither fails by
inspection; the decomposition rows decide.)*
Ship quantized-integer copies of the per-stage centroids (`ctx_cb`
exists compiler-side today, never serialized). Query side, per stage:
Hamming-assign as today → subtract the winning centroid's integer copy
from the bundle (add/sub, in-kernel) → re-derive the next stage's
signature from the residual with per-stage thresholds. Recovers loss #1
directly; artifact grows by `STAGES × K × D` integer entries (⚑ i8 vs
i16 quantization of centroid copies — measure fidelity vs size).

### Design G — multi-bit graded signatures
Quantize per-dimension **magnitude** into b bits (b ∈ {2, 3, 4} ⚑)
against per-stage threshold ladders derived offline (normalization folds
into the ladder, addressing loss #2 and #3 without any runtime divide).
Distance = popcount over a thermometer/unary encoding (b bits per dim →
288·b-bit signature; Hamming on thermometer codes equals L1 on the
graded values — order-preserving by construction, which is exactly the
property the relational layer wants; see the correspondence doc).
Composes with Design R (graded signature *of the residual*).

### Address-space consequences (review these as interface changes)
- Signature width changes (288 → 288·b bits) change ROUT prototype/mask
  layout (#247 consumes this) and region-key derivation (#263).
- Thermometer-graded Hamming = L1 keeps neighborhoods meaningful under
  truncation — the "chop bits for locale" property lives HERE, in the
  relational layer, not in κ-labels (per the #258 investigation).
- Determinism: threshold-ladder derivation must be content-addressed
  into the artifact (κ-pinned) and platform-independent — interacts
  with #265; until D2 lands, ladders are part of the macOS-pinned
  baseline with an era note.

## Phasing

- **Phase A (certifier-side, no runtime change): decompose before
  designing.** The 3.9pp gap was measured by an ablation that changes
  three things at once (normalization, Euclidean assignment, residuals)
  — the existing evidence cannot attribute the gap among the three
  losses. Phase A therefore adds *decomposition rows* first, then the
  candidate rows, all in one certify run (same harness discipline as
  #244's rows):

  | Row | Isolates |
  |---|---|
  | A-norm-only (normalize → sign-bits, no residuals) | loss #3 alone |
  | A-resid-only (residual VQ in raw space, no normalization) | loss #1 alone |
  | A-G(b) for b ∈ {2,3,4} (graded signature, no residuals) | loss #2 (+#3 via ladder) |
  | A-R (integer residual + re-threshold, no normalization) | Design R as buildable |
  | A-R∘G(b) (graded signature of the residual) | composition |
  | A-f32 (existing) | all three jointly — the ceiling |

  **Exit rule:** if a single-loss row recovers ≥ ~70% of the A-f32 gap,
  the corresponding minimal fix is the design (e.g. norm-only winning ⇒
  a threshold-ladder/scaling fix, and both R and G as drafted are
  over-engineering — an explicitly acceptable outcome). Otherwise pick
  the best buildable row (A-R / A-G / A-R∘G) by the Empirical Criterion
  below. Either way the full table is posted on #243 before Phase B.
- **Phase B (kernel):** implement the winning query path in the integer
  kernel behind an artifact-versioned format bump (R4G1 era note);
  extend the P-4 source scan to the new code; equality-witness the
  kernel form against the plain form per certification run (existing
  discipline).
- **Phase C (adoption):** store construction flips to f32 residual-VQ
  (+ #244's store-shape decision, whichever won); re-pin κ baselines
  with era notes; BASELINE.md gains the new table.

## Empirical Criterion (the DoD's teeth)

On the #244 harness (same corpus, same run): the winning design must
recover ≥ half the A-f32 gap — ≥ 30.1% top-1 and ≥ 34.1% agreement at
certification-era conditions — at unchanged runtime op census (no new
operation classes; op-count growth ⚑ budget: ≤ 2× table-reads/adds per
token) and store keys within the #244-decided envelope. Anything less
ships nothing: the certify rows remain as the recorded negative result.

## Explicit non-goals

No floating point in the runtime kernel under any design. No changes to
κ-label semantics (identity layer untouched). No dependency on ordered
or invertible digests (see #258 and the #230 roadmap addendum).

## Sign-off

- Casey: **approved 2026-07-29** (Phase A authorized as exploratory
  measurement; ⚑ design decisions deliberately deferred to Phase A
  results)   - Ari: ____   - Alex: ____
⚑ decisions resolved: R-vs-G-vs-R∘G *(deferred to Phase A)* · b
*(deferred)* · centroid int width *(deferred)* · op-budget *(deferred)*

Phase A baseline is fixed: the #244 matrix (A-binary 28.2/31.9 ·
A-f32 ceiling 32.1/36.4 · A-single+query-beam 28.2/31.9 at 134,733
keys), reproduced bit-for-bit across three runs on 2026-07-28/29.
