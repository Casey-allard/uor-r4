# Design: Learned Per-Dimension Weighting for Signature Assignment

> **Historical design input.** This learned weighting proposal belongs to a
> preserved compiler lane and is not part of the final route-native serving
> architecture. See the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

*Issue #310. Status: DESIGN FOR REVIEW — exploratory sketch; no code
authorized by this document. Decision points marked ⚑ are open.
Claim language: Definitions, Objectives, and Empirical Criteria only.*

## Provenance (what motivated this, stated plainly)

The immediate stimulus was `sanity/renegade` (crates.io `renegade-ml`),
a zero-config KNN library by Ian Clarke. It is **not** a
quantization/LM project: no k-means, no codebooks, no integer kernels —
its entire compute path is f64. Three of its ideas transfer by analogy
to this repo's compilation and assignment paths; nothing transfers
directly. This note sketches each transfer and states what would have
to be true for it to be worth building.

## Item 1 — per-dimension importance weighting (the main sketch)

### The gap it addresses

Both shipped assignment metrics treat all 288 signature dimensions
uniformly:

- Sign-Hamming: `popcount((s XOR p) AND m)` — every surviving bit costs 1.
- Shift-add dot (TLA6): each centroid value is one packed power of two
  (`DOT_TERMS = 1`), so dimension importance is whatever the RVQ
  centroids happened to encode — there is no learned, per-dimension
  discriminative signal.

Renegade's transferable idea: learn a per-dimension weighting from
data, and — this is the discipline that matters — **keep it only if a
held-out criterion improves**, otherwise fall back to uniform. Their
mechanism (isotonic regression into an "effect space") is f64 and does
not transfer; the weighted-distance outcome does.

### Definition (weighted masked Hamming)

Partition the D dimensions into b weight classes (b ∈ {1, 2, 4} ⚑),
class j carrying a power-of-two weight 2^e_j and a 36-byte mask m_j:

```
d_w(s, p) = Σ_j  popcount((s XOR p) AND m_j) << e_j
```

- b = 1, e_1 = 0, m_1 = today's mask ⇒ exactly today's distance.
  The design is a strict generalization of the shipped path, the same
  property that made Design G cheap to review (#243).
- Kernel op census per byte per class: one xor, one and, one popcount
  table-read, one add; per class one shift. All contract §2 ops; no new
  operation classes. Op-count growth ≤ b× table-reads/adds on the
  Hamming path — inside the ≤ 2× budget frame already set in
  docs/graded_signature_address_design.md.
- Storage: b masks + b exponents per class per stage
  (b · 36 B · 256 · 4 ≈ 37 KB at b = 1, 147 KB at b = 4) — small next
  to the ~590 KB dot tables.

### Learning the weights (compile time; contract §4 unrestricted)

Candidate importance signals, all deterministic and content-addressable:

1. **Threshold-margin reliability** — per dimension, the distribution of
   |bundle − threshold| over the training sample. Dimensions whose
   values pile up near the threshold flip sign on noise; downweight
   them. Cheap: one pass over the same 6,000-sample context set the
   compiler already draws.
2. **Mutual information** between dimension d's sign bit and the f32
   control's stage-1 class label — directly measures "how much does
   this bit drive the assignment we are trying to approximate."

⚑ which signal; ⚑ whether exponents come from ranking the signal into
b quantile buckets (simplest, platform-stable) vs. a continuous map
quantized by IEEE exponent extraction (precedent: compiler.rs's
libm-free `e = ((r.to_bits() >> 23) − 127)`).

### Alternative framing (⚑ pick by measurement, not taste)

Design G's thermometer encoding already makes Hamming equal L1 on
graded values. Per-dimension importance is then *per-dimension ladder
spacing* — dense thresholds where the dimension matters, sparse where
it doesn't. If Phase A of #243 lands G, importance weighting may be
strictly better expressed there (learned per-dimension ladders) than as
a parallel weight-class mechanism on the binary path. Do not build both
blind: add decomposition rows first.

### Phase A rows (same harness discipline as #243)

| Row | Isolates |
|---|---|
| A-W(b), b ∈ {2, 4} | importance weighting on the binary signature |
| A-G-ladder | per-dimension learned ladders inside Design G |
| A-binary (existing) | uniform baseline |
| A-f32 (existing) | the ceiling |

### Empirical Criterion

On the #244 harness at certification-era conditions: the winning
weighting row must beat A-binary (28.2/31.9) by a margin that closes a
visible fraction of the A-f32 gap (32.1/36.4) — ⚑ threshold, suggested
≥ 25% of the gap, weaker than #243's "half the gap" because weighting
composes with G/R rather than replacing them — at unchanged operation
classes and ≤ b× Hamming-path op counts. Anything less ships nothing:
the rows stand as the recorded negative result, exactly as #243's exit
rule provides.

## Item 2 — exact indexing for compile-time assignment

Renegade's VP-tree gives it 87–347× over brute-force search. Our
situation is different and the lesson narrows accordingly:

- **Runtime: do nothing.** The serving scan is 4 stages × 256 classes ×
  288 ops — exhaustive is already trivially fast, allocation-free, and
  branch-predictable. The graph runtime's routing shortlist
  (`popcount((sig AND mask)) ≤ threshold`) is the correct indexing
  structure for region scale; it already exists.
- **Compile time: profile before building.** The k-means E-step is
  exhaustive O(n·k·D) — fine at n = 10,000 / k = 256, a real cost if
  the graph compiler's region budget grows. If profiling shows the
  E-step dominating, the transfer candidate is **Hamerly/Elkan
  triangle-inequality pruning**, chosen specifically because it is
  *exact*: assignments are identical to unpruned Lloyd's, so artifact
  bytes are unchanged and the determinism invariant is untouched. It
  also composes with the existing pattern in `induction.rs`
  (rayon-parallel E-step, ordered f64 reductions).
- **Explicitly excluded:** approximate nearest neighbors (VP-trees,
  LSH, HNSW) anywhere in the compile output path. Approximation changes
  which centroid wins near boundaries — a silent change to the compiled
  objective, not just its cost. Exactness of the E-step is the
  invariant; speed is the variable.

## Item 3 — validation discipline (k0 and update-store triggers)

Renegade picks K by leave-one-out CV over k ∈ [1, √n] and retrains on
explicit growth triggers (metric at 50% growth, index at 20%). Two
transfers:

1. **Region-count validation.** k0 is entropy-justified today. Add a
   held-out comparison — k0/2, k0, 2k0 scored on a content-addressed
   held-out split (partition by sample ID, plan §4.1) — as extra
   certifier-side rows. No runtime change; either confirms the entropy
   rule or replaces a heuristic with a measurement.
2. **Update-store invalidation triggers.** The update store's refresh
   behavior is heuristic; growth-/drift-triggered invalidation with
   explicit, tested thresholds is the renegade-shaped improvement.
   ⚑ this needs its own investigation of `tless_uor.rs` before any
   commitment — recorded here as a direction, not a plan.

## Explicit non-goals

No floating point, multiply, or divide in the runtime kernel under any
item. No approximate indexing in compiler outputs. No changes to
κ-label semantics. No third `ScoreQ` definition. No runtime format bump
authorized by this document — Item 1 ships, if at all, behind the
artifact-era discipline already used for TLA6 dot tables.

## Sign-off

- Casey: ____   - Ari: ____   - Alex: ____

⚑ decisions: weight signal · b · exponent quantization · W-vs-ladder
framing · gap-closure threshold · update-store investigation scope
