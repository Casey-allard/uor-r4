# #961 — S0: canonical lexical ingestion and hierarchical route-state serialization

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-27T00:39:05Z
- **Updated:** 2026-08-27T04:04:00Z
- **Closed:** 2026-08-27T04:02:02Z
- **Labels:** enhancement
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/961
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Create the one canonical, reversible path from pinned text/corpus bytes into route-native state and back into text units. This is the active entry stage for the geometric-intelligence programme under #820.

The serving artifact stores geometry as addressable state: the route is the location. It does not smuggle a learned vector model into the runtime.

## Normative architecture

1. **Lexical codec, not κ, segments language.** `C_lex` owns byte normalization, lexical boundaries, unknown-unit behavior, vocabulary/version identity, and reversible lexical decoding. Its exact input contract is corpus/source text plus declared metadata.
2. **κ is identity and serialization integrity.** `kappa(X)` addresses the canonical schema/provenance-bound byte envelope. Equal κ values mean equal canonical envelopes; digest distance has no semantic or geometric meaning. κ is not a tokenizer.
3. **Route records are first-class.** Registered prime atoms, semiprime transition experts, ordered multiplicity-preserving prime n-lets, the finite revisioned zeta-ordinate grid, S3/R4 state, S2 Hopf observation, retained fiber/torsion, and exact Z[φ] golden radial coefficients are serialized rather than reconstructed from an opaque embedding.
4. **Typed least-cost chart adapters are serialized.** The project adapters are Euclidean `√2`, complex/discrete `2i`, and Riemannian interval `[0,2]`. They are named coordinate/scale conventions, not a claim that the mathematical domains or values are literally equal. Every selected adapter binds direction/orientation, exactness or error bound, conversion cost, inverse/fidelity witness, and deterministic selection rule.
5. **The Euler/Hopf boundary is typed.** The project seam is `e^(iπ) + π^0 =_bridge 0^0`. `continuous-null` preserves the complex cancellation as `0`; `discrete-empty-product` phase-shifts/retypes the seam as the discrete identity `1`. `=_bridge` is a domain-transition operator, not ordinary equality. For the Hopf observation `S3 → S2 ⊂ R3`, `sin = ±1, cos = 0` is the signed active-magnitude class and `sin = 0, cos = 1` is the null/identity class. At `cos = 0`, tangent is not numerically divided; a typed quarter-turn phase shift preserves sign and fiber/torsion.
6. **The paired-H4/E8 construction is load-bearing.** Use the icosian ring—quaternions over Z[φ] with the declared Turyn norm—as the E8 lattice as a Z-module. Each 8-dimensional block is a canonical golden-coupled pair of R4 points; the 600-cell `H4 ⊕ φH4` folding is the fixed polytope presentation. Project shorthand **`E8 = H4 × H4`** names this paired construction; the normative implementation is the icosian golden-coupled `H4 ⊕ φH4` folding, not a literal Cartesian-product identity or an identification of R4 with E8. Capability remains measured, not assumed.
7. **Hierarchy is serialized incrementally.** Canonical local state commits to current, previous, and last-two routes; sentence commits to ordered lexical-route identities; paragraph commits to ordered sentence identities; conversation commits to ordered turn/paragraph identities; global commits to a versioned bounded knowledge snapshot. Parents carry bounded geometric summaries and ordered child identities, never a requirement to rescan the whole corpus.
8. **Unseen histories retain exact identity plus overlapping geometry.** Every hierarchy node binds exact κ identities alongside bounded harmonic/trajectory summaries: session hypersphere vector, winding/window state, projection energy, shared-prime factor summary, cosine-resonance profile, and accumulated Hopf phase. These summaries overlap across adjacent windows so #952 can route an unseen history without pretending it was an exact recall hit.
9. **Manifest binding is explicit.** `manifest.json` binds the codec, prime registry, zeta grid, route schema, typed chart/Euler/Hopf profile, icosian profile, hierarchy roots, shared spin/torsion equivalence indexes, content blobs, harmonic/trajectory profiles, and every referenced κ. Like spin states remain jointly addressable through canonical shared-class indexes without duplicating the underlying payload.

## Serving boundary

- Source-model weights may be used only by an offline teacher to generate labels or comparisons. They are not ingested as serving knowledge and are never a runtime dependency.
- The serving path has no transformer, MoE, dense-matmul, source-attention, Ollama, hosted-provider, or equivalent generative fallback.
- Routine testing is dormant. Only the smallest decision-bearing probe below is authorized; no broad suite, corpus-scale compile, or long run belongs in S0.

## Blockers and order

- **Status: ACTIVE.** This is the only newly assigned implementation stage.
- #958 closed through PR #960 at merge commit `02781ab8ad94a05234a30c7aa100a881c7ba2b70`; its retained artifacts are the bounded input to this schema, and S0 must name exactly what #952 consumes.
- S0 completion blocks #952 (recursive attention). The later inference, correctness, reasoning, product, optimization, formal, and release stages remain unassigned.
- #949 is closed as the superseded hybrid tracker. #958 is closed as the retained GI-0 foundation under #820. S0 is unblocked and remains the sole active implementation stage.

## Smallest decision-bearing probe

Round-trip one fixed two-turn, two-paragraph fixture containing a repeated lexical unit and one unknown-unit case:

1. ingest bytes;
2. produce route atoms, semiprime/n-let records, spin/torsion plus typed chart/Euler/Hopf and icosian records, exact identities, overlapping harmonic/trajectory summaries, and all five hierarchy roots;
3. serialize, reload, and serialize again;
4. require byte-identical output, identical κ values, exact lexical reconstruction where defined, and one shared-class lookup that reaches every matching spin state.

If any identity, hierarchy, inverse-witness, trajectory-summary, or lexical round-trip check fails, stop and revise S0. Do not expand the corpus.

## Definition of done

- The schema, chart-adapter/phase tags, and canonical ordering are versioned and documented.
- The lexical codec is reversible on its declared domain and fail-closed elsewhere.
- Every manifest reference is validated transitively.
- State updates are incremental and bounded by changed nodes.
- Exact recall identity and geometric unseen-history summaries are distinguishable in traces.
- The probe emits a compact coverage/identity witness and a truthful PASS/FAIL/NOT_RUN verdict.
- #952 is updated with the exact artifact and hierarchy contract it consumes.

## Comments (1)

### comment 5434219905 — 2026-08-27T04:04:00Z — Casey-allard

## Closure evidence

PR #966 merged through the protected queue at `ae85874a621005ca96f264cece4c24e18ecc225b`; live `main` has the identical reviewed tree from `81058deb70b75db0d1b7fd935715a857423f58de`.

The one authorized fixed probe completed with `PASS_REVERSIBLE_STATE_PLUMBING_ONLY`: canonical bytes and referenced κ values were identical after reload/re-encode, lexical reconstruction was exact, the unknown unit rejected before mutation, every hierarchy reference resolved transitively, shared-class lookup reached 3/3 matching routes, maximum changed nodes was 2, every causal cursor state exposed numeric geometry, and the final seven-level cursor trace equaled the terminal trace.

Artifact κ: `blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd`. Canonical byte CID: `blake3:55ee35e2301e7c262c6acab2de0c8d007f5ba7d4ad41086a15185e63d9471969` (137222 bytes). Embedded frozen schema-2 κ: `blake3:e874ceedebb05846976dc06ce039bc930f03371760a71ec741d71940685c6b37`.

Focused Clippy, CLI compilation, formatting, claim wording, and diff checks were green. Per this issue, no workspace tests, BDD, doctests, no_std, audit, fuzz, Gate C, model evaluation, corpus compilation, or WASM certification were run locally. Required GitHub statuses were queue transport only and are not test evidence.

The exact merged A1 consumer contract is recorded on #952: https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434217921

This closes reversible state plumbing only. Attention, coherent generation, correctness, and reasoning remain unestablished.


---
_Comment/review ids in this file: 5434219905_

