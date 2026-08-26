# How R⁴ is becoming a geometric language model, at undergraduate level

> The seminar version — real machinery, one-line definitions, honest
> caveats. Too dense? Start with [ELI5.md](ELI5.md). Want the full rigor?
> the [Geometric Causal Decoder Roadmap](../geometric_causal_decoder_plan.md).
> The retained table/graph compiler is specified separately in
> [TRANSFORMERLESS.md](../transformerless/TRANSFORMERLESS.md) and
> [PROOF.md](../transformerless/PROOF.md). Code map:
> `crates/uor-r4-core` (including its `transformerless` module),
> `crates/uor-r4-router`, and the root `tless_uor` witness module.

## The frame

The active project asks whether learned Riemannian geometry can replace causal
self-attention without losing free-running language:

```
prompt + prior tokens + persistent geometric memory
                  → bounded learned R⁴ neighborhood at each replaced layer
                  → residual / norm / MLP / LM head via uor-matmul
                  → next-token logits → write the completed turn back to memory
```

The first checkpoint is hybrid and experimental. “Transformerless” is earned
only after the product path makes zero source-attention calls and contains no
dense full-prefix Q·K matrix/softmax kernel; the replacement must use bounded,
intervention-qualified geometric support. That geometry may deliberately learn
to reproduce useful teacher-attention behavior. Multiplication-free is a later,
separate decision.

The sections below explain the historical table/graph system retained as a
comparator. Its measurements are not the current implementation order.

## transformerless: cross-compiling a behavior

**Compilation (historical TLA lane; multiplies allowed offline).** The
teacher is accessed through exactly two surfaces — the embedding table
`E ∈ ℝ^{V×D}` and a next-token logits oracle — so nothing about the
architecture (attention, norms, gating) can leak into the artifact.

```
E (32000×288 f32)                     corpus (150k teacher-labeled tokens)
      │                                          │
      ▼ residual vector quantizer                ▼ context encoder
 4 stages × 256 centroids,              window of 8, dyadic weights 2^(8-j)
 i8 fixed-point books                   (shifts), slot rotation j·17 mod 288
      │                                          │
      ▼                                          ▼
 token := 4 code bytes            bundle ∈ ℤ²⁸⁸ → signature s ∈ {0,1}²⁸⁸
 (87.2× smaller, 0.9692 cosine)        s_b = 1[bundle_b > θ_b], θ = train means
                                               │
                                               ▼ per stage: nearest of 256
                                                 binarized centroids (Hamming)
                                               ▼
                                     graded class code (4 bytes)
                                               │
                                     store: code prefix[..d] ↦ (token ↦ count)
                                     levels d = 0..4, B-tree
```

Two ideas carry the weight:

- **The signature is a coordinate system.** Bit *b* is a halfspace in
  context space; a *d*-bit prefix is a cell in a hyperplane arrangement.
  Two contexts are similar to degree *d* iff their prefixes agree — the
  same progressive-localization trick as LSH or iterated quantization, but
  here the bits themselves are the address. That's the "vector at each
  bit."
- **The store is a quantized backoff language model.** Prediction is
  *deepest populated class wins, argmax by count* — structurally Katz
  backoff, except "histories" aren't exact n-grams but cells of the context
  arrangement. Generalization is free: unseen contexts resolve to whatever
  cell they land in.

**Runtime.** Every arithmetic op is dispatched through an `OpKernel`
exposing `add / shl / xor / lt / table_u8 / table_i32` — multiplication is
absent from the *interface*, so the mul-free claim is by construction,
machine-checked by a source scan (P-4), and measured by the census:

```
per token:  add 59,598 │ xor 36,864 │ shift 22,734 │ cmp 1,322 │ table 54,999 │ mul 0
throughput: 77,342 tok/s   vs llama.cpp q8_0 344, f32 157, run.c 48 (same box, 1 thread)
size:       2.17 MB artifact (0.13 codes + 0.29 books + 0.04 sigs + 1.70 store)
            vs 25–94 MB weights
```

**Historical fixture snapshot.** These original rows are scoped to their named
artifact and population; they are not current product-quality numbers:

| design | mul-free? | agree w/ teacher argmax | WB bits/token |
|---|---|---|---|
| A-f32 (assignment multiplies) | no | 34.7% | 6.32 |
| **A-binary (shipped)** | **yes** | **31.7%** | **6.54** |
| B bit-prefix (no classes) | yes | 28.6% | 7.70 |

Teacher floor was 1.60 bits/token on that snapshot, so the table reported a
large capability gap. A historical fit suggested that additional store entries
moved the resolution knee at constant per-token compute. Later student-prefix
and product runs did not establish that this pointwise trend composes into
coherent generation; that negative result motivates the active decoder reset.

## r4: geometric routing + a formal witness substrate

Two layers with very different levels of rigor — worth keeping apart.

**App layer (heuristic geometry).** No learned weights anywhere:

```
word w  ──► next prime p(w)          (Gödel-ish numbering)
            v(p)_i = sin(ln p · γ_i), γ = first 512 nontrivial ζ zeros
            → a deterministic spectral embedding, zero training

sentence ─► Π p(w_i)  (unique factorization = a content ID you can factor)

state h ∈ ℝ⁵¹², route = argmax over 16 windows of ‖h_window‖·(1+bias)
window ↦ log-spaced scale [10⁴, 10⁶]

state metrics: σ_q (deviation from uniform), σ_kl,
               λ = −ln(1−σ_q),  deficit angle θ_d = π − λ  (conical deficit),
               κ = max weight
```

The Hopf part treats the normalized state as a point on S³ and reads off
the fibration S³ → S² with S¹ fibers:

```
χ = asin ρ₂,  θ₁ = atan2(b,a),  θ₂ = atan2(d,c)
δ = θ₁−θ₂,    α = (θ₁+θ₂)/2
phase transport:  α' = α + ½λ·cos(2χ)·δ      (a connection on the bundle)
```

Generation is a Markov chain over the indexed corpus with candidates
scored `ln p_trans + gravity·cos(v_word, h) − penalty·freq`, and the state
evolves by EMA on the sphere: `h ← normalize(γh + (1−γ)v_word)`.
Trajectory metrics (stratum, winding number, commutator curvature
`(d_E − d_H)/(d_E + d_H)`, D₈ monodromy) are computed per step.

**Framework layer (the rigorous part).** The vendored UOR substrate
computes over the ring R_n = ℤ/2ⁿℤ, where the critical identity
`neg(bnot x) = succ x` makes negation and complement generate the dihedral
group D_{2^n} — with ring/Hamming derivatives, Witt levels, and a
ψ-pipeline that lifts bytes through a Čech nerve, (co)homology, a
Postnikov tower, to k-invariants. The payoff operationally: every route
goes through the axis/PrismModel machinery,

```
640-byte input ──► R4Axis (window argmax, metrics)
                   ──► run_route ──► Grounded certificate:
                       fingerprint, σ, d_Δ, χ(nerve), residual, stratum,
                       + derivation trace that replays hash-free
```

so a route is not a float, it's a *proof object* with a re-verifiable
derivation.

## The integration: one R4 library, internally layered

```
                    uor-r4-wasm-router (facade + server)
        ┌───────────────────────────────┬────────────────┐
        ▼                               ▼                ▼
  uor-r4-core                    uor-r4-router   tless_uor
  (R4 math + integrated          (engine + its  (R4 UOR surface:
   transformerless compiler/     witness layer) TlessAxis, addressing,
   runtime/teacher adapters)                     Grounded)
```

Three concrete joins:

1. **Addressing.** The TLA3 artifact and each store entry are
   content-addressed (deterministic CBOR + blake3 κ-labels).
   transformerless's "store persistence as κ-keyed content" open item is
   closed by r4's addressing layer.
2. **Witnessing.** `TlessAxis` takes a 16-byte window, returns a 31-byte
   record: token, resolution depth, class code, evidence count, and the
   five census counters — *no multiply field exists*, mirroring the
   kernel. `UorTlessModel` mints a `Grounded` per prediction; replay
   re-certifies bit-identically.
3. **Living store.** `POST /api/tless/index` folds text in as new
   evidence (the store κ change is the attestation), `/api/tless/generate`
   emits per-step witnesses, and deleting an entry returns its
   pre-removal κ — attribution and unlearning as data structure
   operations, not policy.

## Critical assessment (the part seminars are for)

- the historical TLA runtime is **strong where it is measured and silent where
  it is not**: its operation-set and reproducibility claims are scoped to exact
  artifacts and checks; the 31.7% row above is an older fixture result, not a
  coherence or current-production claim.
- r4's app layer is **heuristic** — hardcoded eigenvalues,
  `sha256 mod 500` QIMC primes, a 3/8 law stated mostly in demo HTML. Its
  framework layer is the genuine math; the witness machinery doesn't
  validate the heuristics, it certifies *that they ran*.
- The intellectual fit is real though: both systems insist that **a claim
  without a witness isn't a claim** — transformerless prices encodings by
  measured residual, r4 prices executions by replayable derivation. That's
  the coherent thesis of the merged repo.
