# Design P: Complex Discrete Phase-Clock Addressing (Cyclic Graded Codes on the Zeta Torus)

> **Historical design input.** Relevant phase ideas may be reused only through
> an explicit current route contract; this document itself does not authorize
> work or establish attention. See the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

*Issue #276. Status: DESIGN FOR REVIEW — decision points marked ⚑ are open
until Casey / Ari / Alex sign off on this document (review on the PR).
Claim language: Definitions, Objectives, and Empirical Criteria only.*

## Objective

Give the address layer a **phase channel**: a discrete, mul-free context
coordinate built from the lineage's zeta-seeded frequencies, on which
"near" is cyclic distance and next-token transport is an exact digit
rotation. Design P is the cyclic sibling of Design G
(docs/graded_signature_address_design.md): G grades **magnitude on a
line**, P grades **phase on a circle**. Like every representational
choice in this repo, it is address-space design
(docs/addr_prism_correspondence.md): the bucket structure decides
collision geometry, what "near" means for #263's region objects, and
what a distributed serving tier could shard on.

## Where P sits after the Phase A/B decision rows (2026-07-29, #243)

The round-2 metric-isolation rows revised Phase A's attribution: the
dominant loss is the **assignment metric** (A-dot-only 30.4/34.2 with
no residual updates; sign-Hamming + residuals collapses to the floor),
and Phase B pivots to a shift-add dot approximation. P does not compete
with that fix —
it contributes a channel the sign signature **discards entirely** (the
imaginary half of every zeta clock), which no R/G row can recover.
Objective, not assumption: whether that channel carries context-
discriminative signal at certify scale is exactly what the P row
measures. The prime-router evidence is consistent but not sufficient:
the mod12 signal-preservation tables (research report §6; recorded in
docs/prime_router_geometric_context_evidence.md, PR #275) show linear
decodability resolving to 1.0 on a discrete phase coordinate at
trajectory end — on router-lineage trajectories, not on the graph
compiler's bundles.

## Definitions

- **Analytic completion.** Per frequency γᵢ, the lineage vector
  component `sin(ln(p)·γᵢ)` is the real part of a clock; the completed
  signal stores the pair `(cos(ln(p)·γᵢ), sin(ln(p)·γᵢ))` — i.e.
  complex discrete L2, one phasor per frequency. Compile-side only;
  floats are unrestricted there (INFERENCE_OPERATION_CONTRACT.md §4).
- **Clock digit.** Phase θᵢ = atan2(sin, cos) quantized into b buckets:
  cᵢ ∈ Z_b. A context address is the digit vector (c₁, …, c_F) — a
  point on the torus Z_b^F.
- **Cyclic distance.** Per digit d(x, y) = min(|x−y|, b−|x−y|) (Lee
  metric), summed over F. Satisfies the triangle inequality, so the
  #277 VP-tree applies unchanged.
- **Transport.** The per-token phase step is a rotation: digit
  increment δᵢ mod b per frequency. On the completed signal this is
  exact; on real-parts-only it is undefined — this is the mechanical
  content of "completing the analytic signal".
- **Capacity (Definition, not Guarantee).** A single b-clock addresses
  b states; the torus addresses b^F. The 4D continuous embedding's
  collision collapse (32k tokens × contexts in R⁴) is the negative
  capacity datum this design responds to; top-M membership over 288-bit
  signatures is the positive one (k-NN under Hamming already works
  where the space is large enough).

## Runtime realization (mul-free by construction)

Two candidate encodings, ⚑ pick by measurement:

1. **Cyclic thermometer (two-arc unary).** Encode digit c as b bits with
   a contiguous arc of ⌈b/2⌉ ones starting at c (circular). Hamming
   distance between two such codes = 2·Lee distance (order-preserving).
   Distance machinery is then **identical to Design G's popcount path**
   — same kernels, same P-4 scan surface, width F·b bits.
2. **Digit LUT.** Pack digits (b ≤ 16 → 4 bits each) and resolve per-
   digit Lee distance through a b×b table read. Smaller signatures,
   new (but table-read-only) kernel shape.

Transport is integer digit addition mod b in either encoding. No
multiply, divide, or float anywhere on the query side; the contract §4
runtime op census must be unchanged in the measured rows.

⚑ b ∈ {8, 12, 16}: 12 is the empirically privileged modulus in the
prime-router tables; 8 and 16 are the power-of-two controls.
⚑ F: the 16-window zeta design-matrix frequencies (#246 re-port) are
the natural candidate set; a stride-sampled subset is the cheap control.

## Address-space consequences (review as interface changes)

- Signature width becomes F·b bits (encoding 1) or 4·F bits (encoding
  2) — ROUT prototype/mask layout interplay is the same class of change
  Design G already declares (#247 consumes widths; #263 derives region
  keys). If G and P both land, widths compose additively (channels
  concatenate; ⚑ joint vs separate membership scans).
- Phase-bucket boundaries are compile-time constants derived from the
  frequency set: content-addressed into the artifact, κ-pinned,
  platform-sensitivity noted — until D2 (#265) lands they are part of
  the macOS-pinned baseline with an era note, exactly like G's ladders.
- Sessions: #247's signature lane can carry transported phase state
  without new plumbing — P gives that lane a discrete, exact-transport
  coordinate instead of a float trajectory.

## Measurement (the DoD's teeth)

P joins the **same** decomposition harness as R and G (certify.rs Phase
A rows; no separate harness). ⚑ The one genuinely open mapping question:
where do per-context phases come from in the harness?

- **Lane (b) — transport-composed (primary):** zeta-seeded token
  phasors composed over the context window by per-token transport;
  this exercises the mechanism P actually claims (exact rotation
  composition) and needs nothing from #246.
- **Lane (a) — design-matrix projection:** phases of the #246
  re-ported 16-window design-matrix projection of the bundle; runs
  only if/after #246 lands.

Rows: P(b) for the chosen b set, and P∘G (phase digits concatenated
with graded magnitude) as the composition row. Baselines and criterion
are the standing ones — measured against the A row that is shipped at
run time (post-#243-decision), at unchanged op census:
**Empirical Criterion: ≥ 30.1% top-1 and ≥ 34.1% agreement**, with the
#244 store-key envelope. Exit rule: if every P row measures below the
best shipped-family row on both metrics, P closes as a recorded
negative result and the phase channel is retired from address-space
candidates (the #247 session-lane use is unaffected — it is a state
channel, not an address claim).

## Sequencing

1. The #243 A-R-retrained decision run is in flight; its outcome sets
   the baseline the P rows must beat. P work does not start before that
   decision is recorded on #243.
2. Lane (b) rows are certifier-side only and follow the batch-flow
   amendment (rows land as code, measurement runs backgrounded, table
   posted on #276 before any kernel work).
3. #246 unlocks lane (a); #277's VP-tree gains a second metric the day
   any P row ships (Lee distance is triangle-inequality-clean).

## Explicit non-goals

No floating point or multiplies in the runtime kernel under either
encoding. No change to κ-label semantics (identity layer untouched).
No capacity Guarantee — b^F is a Definition about address counts, not a
claim about usable discrimination; only the harness rows can license
the latter. Not a residual-VQ replacement: R fixes how the existing
magnitude channel is quantized; P adds a channel — they compose or
compete only through measurement.
