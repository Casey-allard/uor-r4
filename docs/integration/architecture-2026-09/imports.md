# UOR-R4 imported mechanisms and contributor reconciliation

Source review, 2026-09-07 UTC. Reviewer scope: actual pinned dependency surfaces, tracked imports, prior exact source receipts, and selected fresh contributor call chains. No model build, model inference, Lean/GAP checking or upstream benchmark executed. Source inspection establishes definitions/call paths; numerical and capability results below remain attributed to their original receipts. Read AGENTS.md and current-state.md before review. Active owner clarification permits geometric address/page lookup and offline Rust training matmul while prohibiting serving matrix products. Shared typed operators are the current priority, with expert gates retained as a conditional future option.

Workspace W = `/Users/casey.allard/.codex/worktrees/r4-typed-value/uor-r4`. All W-relative references below denote real files in this worktree. Dependency and cached source paths are given explicitly. Current Rust model is `d59070c2` per W/docs/integration/current-state.md; this report supplies import decisions, not a new model result.

## Source closure and reading coverage

- W/Cargo.toml:66–68 and patch section105–106 pin uor-addr `165b51e3e2113ee5d032730cde709335d4fe9b60`, UOR-Framework foundation/SDK/verify `51c01382200b0179d6640b07e9c8119364ab69a1`. The patch unifies Rust type identity across direct and transitive imports.
- W/crates/uor-r4-model-source/Cargo.toml:28–31 pins uor-matmul `b13c98449948174f590e337c4dc25dfc394a07d0`. graph-cli also imports its core. Their actual local Cargo git checkouts were read.
- W/Cargo.lock:1851–1909 locks all five uor-prism crates at crates.io 0.4.0. Actual registry source, rather than the similarly named old tracked tree, was read for the numerical and tensor kernels.
- `git ls-files uor_standards` reports 1,121 tracked files. W/Cargo.toml:26 excludes this legacy tree. Its Framework/addr/Prism sources remain research/history, not the dependency versions being executed. Do not delete them, count them as extra active engines, or edit them expecting the active dependency to change.
- Existing knowledge already has complete tree inventories for NEMESIS (205 files), W33 (29,843), GoldSnnail (301), HELM (10,510); these are discovery counts, not claims every proof/source line was read. The prior ecosystem survey inventories 547 public repositories but flags most personal repositories DISCOVERY_ONLY. W/docs/integration/uor-source-audit.md, external-research-audit.md, nemesis-w33-relevance.md, afflom-ecosystem-followup.md preserve the scope.
- This review re-read named actual dependency implementation sections, entire selected GoldSnnail routing/chat/compression/attention functions, actual cached W33 constructor/CAS/path-copy/operator checks, the three-page NEMESIS carrying report text, and current repo integration/provenance/results. It does not claim an exhaustive proof audit of the enormous contributor archives.

## UOR-ADDR: canonical identity and typed composition, already useful

Actual checkout: `/Users/casey.allard/.cargo/git/checkouts/uor-addr-52e08702e451f8fd/165b51e`.

Read `crates/uor-addr/src/lib.rs`, `composition/canonicalize.rs`; prior exact source receipt supplies json/value.rs, ring/mod.rs and format modules. The library offers JSON JCS+NFC, CBOR, ASN.1, XML subset, S-expression, ring and code-module canonicalization; feature-gated GGUF/ONNX structural carriers; pluggable hash axes; source-polymorphic Inline/Borrowed/Stream carriers; format-independent resolver plumbing; typed witnesses. Existing R4 uses JSON addressing for manifests and CBOR for graph/TLA boundaries. This is an active integration, not hypothetical.

Concrete composition laws at this pin:

- G2 (`canonicalize_g2`, line109) lexicographically sorts two full label byte sequences, concatenating min then max. Commutativity is implemented. It does not preserve operand order and is not an associative arbitrary sequence fold.
- F4 uses min(raw digest, bitwise complement), explicitly a byte-level involution quotient.
- E6 tags a class from first digest byte modulo9; it is a classification over identity bytes, not a semantic distance.
- E7/E8 continue the declared canonical composition family; their algebraic naming does not establish an R4 language encoding or a coordinate-preserving E8→H4 conversion.
- `check_axis` rejects mismatched digest axes. NAF SHA256 and internal BLAKE3 are not interchangeable.

Use now: bind learned artifact/state/operator/page manifests, semantic type/role/order, exact source closure and format version; deduplicate immutable identical pages; verify restoration. Keep identity separate from learned placement and from content relevance. A digest is not a tokenizer, meaning vector or language score. JSON normalization may change raw bytes, so preserve lexical bytes/spans in separate typed fields.

Best bridge: the current model already reconstructs parent artifacts and commits selected values. Give new selected operator/page records typed, versioned UOR identities at that existing boundary, without replacing the runtime or adding another address implementation.

Primary source: https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/composition/canonicalize.rs

## Framework and Prism: typed execution and some finite operations; ontology names are not all numerical engines

Actual Framework checkout: `/Users/casey.allard/.cargo/git/checkouts/uor-framework-5fc0e124af70bb49/51c0138`.
Actual Prism registry root: `/Users/casey.allard/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` with `uor-prism*-0.4.0`.

Framework has generated ontology traits/handles/records, constrained shape declarations, typed operation axes, resolver tuples, sealing of Grounded/Certified states, execution/trace verification and finite structural primitives. R4's existing W/src/tless_uor.rs:1368–1417 implements a concrete PrismModel and routes one TlessAxis invocation through run_route; the trace test at1877 calls verify_trace. This certifies the declared wrapper's structural/replay contract, not linguistic correctness of arbitrary output.

Important distinction for Hopf: `foundation/src/kernel/convergence.rs` defines ConvergenceLevel/HopfFiber traits, Null sentinels, resolver-backed handles/records and constants for R/C/H/O dimensions1/2/4/8, associated spheres and names such as memory/agency/self. Those names are ontology metadata. They are not implementations of memory learning, agency, consciousness or a Hopf attention algorithm. The project has separate actual Hopf/R4 operators; the bridge must identify which data each API carries.

Concrete Prism operators inspected:

- `uor-prism-numerics-0.4.0/src/ring.rs`: caller-buffer addition `a[i] ^ b[i]`, multiplication `a[i] & b[i]`. Algebra is a product of independent bits over GF(2). This is neither wrapping Z/256Z multiplication/addition nor polynomial GF(256). Example1 XOR1=0 whereas1+1 mod256=2. Useful for masks/finite Boolean fields when explicitly typed; cannot silently back modular numeric operators.
- `uor-prism-tensor-0.4.0/src/tensor.rs:133–178`: CpuI8MatmulSquare uses row/column/k loops and `acc += a*b`, saturates output. It is a matrix product despite being typed and integer. Excluded from final serving under owner's clarification; potentially training/reference only.
- Standard library also exposes BigInt modular, fixed-point, prime-field, hash/commitment/signature, activation and shape axes. Availability of a standard-library operator does not require including it in model serving. The facade contains a one-time-pad reference FHE axis; it is not a ready production FHE inference solution.

### Vector calculus and triangulations in the imports

Actual `foundation/src/kernel/op.rs:7012–7065` DC1/DC2/DC3/DC4 entries are static IRIs for ring/Hamming derivative identities. `docs/content/concepts/differential-calculus.md` states ∂R f(x)=f(succ(x))−f(x), ∂H f(x)=f(bnot(x))−f(x), Jacobian/curvature and curvature-weighted selection proposals. Actual `lean4/UOR/Individuals/Op.lean:1845–1908` defines ontology Identity records with `verifiedAtLevel := #[]`, validity fields none. These are data declarations, not proofs of a general learned field or executable autodiff. The analytical conformance fixture is RDF/SHACL vocabulary validation.

A real finite topology routine exists: `foundation/src/enforcement.rs:7836–8011`, `primitive_simplicial_nerve_betti[_in]`. It expands registered recursive constraints, caps input at8constraints/8sites, computes support intersections, enumerates edges and triangles, forms boundary1/boundary2 and computes ranks modulo1,000,000,007. It emits a Betti vector for this implemented 2-dimensional complex; no boundary3 is used. This is constraint topology, not a general R4 Delaunay/mesh triangulator or trained manifold router. Its broad source comment about total unimodularity must not be promoted to a proof for arbitrary simplicial boundaries; actual finite domain matters. No imported curl/Hodge/semantic mesh execution was established by this review.

Use now: explicit domain/shape/operation identities, bounded typed failures, replayable finite operators. Use finite calculus to define a task-specific geometric transition only when its state map and behavior objective are stated; do not mistake vocabulary coverage for solved language reasoning.

## uor-matmul: useful offline contraction implementation, not a loophole in serving purity

Actual checkout `/Users/casey.allard/.cargo/git/checkouts/uor-matmul-b081c6ed648478b5/b13c984`.

Read `crates/uor-matmul/src/slice.rs:270–449`, `uor-matmul-gemm/src/driver.rs:125–185`, `coded.rs:1–120` and entrypoint/census references. W/crates/uor-r4-model-source/src/geometric_training.rs:1518 calls `uor_matmul::slice::gemm_float`.

There are distinct paths:

1. Dense generic GEMM traverses i,j,k and applies exact accumulator MAC, optional panels and one encode/rounding per output.
2. Coded streaming decodes weights then contracts; the source explicitly states it issues the same m*k*n products plus decode cost. It saves residency, not mathematical work.
3. Tabulated coded weights orient blocks along the reduction and use table read+add per code after table preparation. Operand projection, preparation, memory and dynamic contraction remain. The object being computed is still C=A*B.
4. Float facade treats float bitpatterns as exact dyadic codes, uses Atlas-octet contraction and exact accumulation followed by final rounding. This differs from conventional sequential FMA/BLAS rounding. Caller-owned packed-code offers alter reuse and preparation traffic; no arbitrary matrix disappears merely by changing scalar instruction implementation.

Offline Rust training/reference is expressly permitted to use these operations. Final inference must not call these mathematical matrix products, even a no-multiply-instruction tabulated variant. The useful transferable idea is finite coded operator execution with honest cold/warm accounting and typed exact semantics; do not import the GEMM call into serving.

Prior source audit found upstream `3cc5882f210667f9ac00fd8c02c5b5957b493f5d` seven commits newer than pin, only manifests/tests/policy/docs/CI changed, numerical source unchanged in that prior comparison. No repin performed here, no fresh parity or timing claimed.

Primary source: https://github.com/UOR-Foundation/uor-matmul/blob/b13c98449948174f590e337c4dc25dfc394a07d0/crates/uor-matmul/src/slice.rs

## NAF and GNAF: an actual interchange slice plus inert formal research

W/crates/uor-r4-naf/src/lib.rs implements `uor-naf/1-draft.6`: arbitrary-precision signed integer NAF normalization/encoding/decoding; tensor value/shape and integer/tensor address-binding; explicit malformed, unresolved, unsupported, unadmitted, commitment failure, resource refusal and policy-out-of-domain taxonomy. It uses SHA256-only typed NafLabel while R4 internally uses BLAKE3. Spec draft is part of each manifest preimage: future spec-version changes require identity regeneration.

State/operator domains, Atlas-word adapter, uor-naf-plan, execution and optimality capabilities are explicitly unsupported/deferred (source lines1–9, decode_artifact706–726). Claims vocabulary exists in claims.rs; W/docs/gnaf_integration_653.md says it is not wired throughout live graph-certify/API results. Thus NAF is ready to carry selected integer/tensor values with proper identity, not a hidden operator-training or reasoning engine.

W/proofs/wasm-gemm-gnaf was subtree-imported from `afflom/WASM-GEMM-GNAF` at `171652cd95c0b8e8620f76151b7e0c485e30ccfc`; provenance and both licenses retained. Later separately inspected upstream reference is `917306fd2b5a397ab02c5d38918fb8620fcc5ae0`. W/docs/gnaf_import_provenance.md and gnaf_integration_653.md catalogue every proof family. Actual `WasmGemmGnaf/Theorems/Status.lean` retains WorkloadIncomplete: released compiler/semantics/coverage/cost/lower-bound obligations are open, released global-optimality theorem absent. Some generic existence/conditional cost/closure lemmas exist; no proof transfers automatically to R4 or uor-matmul. No Lean execution this review.

Use: typed result/value boundaries now, selected incremental-equals-clean-rebuild or exact operation witness discipline later at an existing consumer. Avoid a new proof registry/harness before model work needs it.

## NEMESIS: state/transition/primitive design criteria, not an implemented model

Pinned research source `markrnd87-cmd/NEMESIS-Theory@0d106967843c2c96477cf3e57aeff213e7db1c97`.
Cached root `/Users/casey.allard/.local/share/uor-r4/knowledge/audits/2026-09-03/research/snapshots/markrnd87-cmd__NEMESIS-Theory`; extracted carrying report also `/tmp/uor-architecture-audit/nemesis-scs.txt`.

The existing complete inventory is205files, predominantly165PDF+36DOCX, no standalone executable/formal project. Three-page Structure Carrying Substrates report was re-read. It names (1) bijective state representation for the declared domain, (2) transition fidelity, (3) native primitive interpretation. These are excellent questions for the UOR-R4 lowering contract: what is retained, what exactly does the table action compute, how do we decode its result?

The report asserts O(d) queries, one-time O(N) compilation and optimal energy, but provides no concrete compiled learned model, population-size bound or measured laptop implementation that establishes those assertions. Other earlier reviewed PDFs contain placeholders and proposed geometric quotients whose group action remains unspecified. No source claim about all arbitrary neural artifacts becoming parameter-count-independent lookup should enter the architecture as an established theorem. NEMESIS's source archive has no established reuse license at inspected pin; cite and derive a fresh precise specification, preserve attribution.

Useful immediately: define E:semantic-state→typed-geometric-state, D on reachable values, and T_operator with D(T_operator(E(s)))=specified-update(s) for finite deployed operators. Learning must choose useful representations/operations; algebraic preservation alone does not choose what language means.

## W33: exact finite operators and persistent pages, with preserved negative model evidence

Source `wilcompute/W33-Theory@5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`, MIT. Cached sources under the research snapshot root; direct reread `/tmp/uor-architecture-audit/w33_runtime.py`, `w33_operators.g` (same pinned excerpts from previous exact receipt).

Concrete executable reference definitions:

- F3^4 projective normalization,40points/40lines, symplectic incidence; Geometry.route picks direct adjacency or canonical common neighbor, at most two graph edges.
-160point-line chambers; panel operators P and L change one incidence coordinate. GAP witness constructs matrices from incidence and tests P²=3I+2P, L²=3I+2L, ordered braid/commutator/projector/spectrum identities. Matrices here are offline finite witnesses; each sparse incidence action could separately lower to explicit finite table/permutation actions if useful, but this is not yet a learned R4 encoder.
- MicroVM finite ISA HP0..2/HL0..2, ADD,RECV,YIELD,HALT, bounded fuel, actual addressed execution.
- ContentStore canonical JSON+SHA256 DAG; `state_at` descends one radix40 address, `_rewrite_at` copies ancestor path, immutable siblings/shared identical subtrees remain reused. This is a real design reference for deduplicated pages/checkpoints. Billions of uniform logical leaves via a handful of shared nodes are representational sharing, not billions of independent computations.

Missing bridge: F3 incidence points and rational160-dimensional chamber operators are not current R4/H4/Spin language states. Need explicit state subspace and encode/decode/action correspondence. A48-dimensional observable image cannot injectively carry unrestricted64-dimensional learned values. Deterministic panel selectors are not individually the aggregate P/L operators; normalization by60 cannot be inverted modulo256. These exact cautions are preserved in W/docs/integration/nemesis-w33-relevance.md.

Actual R4 result to preserve: W/docs/w33_geometry_qualification_845.md measured the pinned mod9 mapping under equal3,200-byte and work budgets; no geometry advantage. No work-reduction win in separating cells, all9primary tightened-budget correctness cells fail, randomized/permuted controls match or exceed the geometry. That rejects this mapping, not the finite mathematics. Reusing the same mod9 score is repeating a completed negative.

RH caution from actual cached upstream RH_PROOF_STATUS.json: graph-Ihara RH statements are separate finite graph claims; `zeta_W_equals_zeta` is OPEN and `Riemann_zeta_RH` explicitly OPEN. Finite graph spectra do not identify classical Riemann zeta zeros. Fixed numerical zeta phases in R4 need no proof of global RH to be usable constants.

Best reuse order: persistent page/DAG ideas when context scale requires them; exact ordered finite operators only when a learned-task encoding is defined. Do not import the whole archive or its physical interpretation.

## GoldSnnail (owner confirmed this is “goldworm”): broad experimental Rust system, not an available full replacement LM

Historical audited pin `unicornd47-afk/GoldSnnail@e8e0f303aa956759343cc14177068dba9ba027bd`. Live GitHub refresh on this review finds publish `a8b8affda6e3427d8d8005fc6c8f56d22a95bb41`, dated2026-09-04; compare reports one commit ahead, only README changed9lines. All inspected Rust files therefore match the old exact source pin. MIT source license. No build/test or upstream benchmark run here.

Actual source coverage: cached lib.rs, Cargo.toml, StateArena/AVX2, quaternion/Poincare, QLIF, token_engine, attention, test_chat, architecture doc, result JSONs and Phase2status. Fresh primary source fetched/read at a8b8affd: routing/moa.rs, routing/shd_ccp.rs, chat/mod.rs, chat/world_chat.rs, chat/thought_chain.rs, compression.rs, world_model.rs. Full301-file inventory and181Rust paths were reviewed by family; not every181Rust file line-audited. Newly retained excerpts in `/tmp/uor-imports-extra/`.

| Family | Actual mechanism | Decision under owner constraints |
|---|---|---|
| Substrate/StateArena | Parallel flat membrane/recovery/threshold/refractory arrays, indices, pre-sized spike buffers; WeightMatrix still dense; AVX2 is x86-specific float/FMA | Reuse ownership/layout ideas only if profiling calls for it; not AVX2 on M1 or dense arithmetic |
| QLIF/SNN | Per-neuron Euler integration, refractory timers, adaptation, quaternion rotation using trig/float | A real dynamic system, but no demonstrated conversion to current finite table LM and no replacement language result |
| LongLongMoA | Flat token×expert float scores, top2 scan, sub-expert indices, expert-load statistics and optional softmax | Explicitly excluded learned expert/sparse gating, despite flat-index implementation |
| QuaternionAttention | softmax(norm(q*conj(k))) followed by weighted values; per-query score/weight Vec allocations, no causal-mask argument | Do not port. Quaternion norm multiplicativity removes orientation sensitivity for equal-norm keys; “in_place” does not remove intermediate allocations |
| Compression | latent×input learned/provisional projection over active spike norms, tanh, hyperbolic delta threshold, allocates latents | Matrix product/projection and magnitude-only phase readout; not the desired phase-preserving serving operator |
| SHD-CCP |12-byte event records with16-bit source delta,32-bit destination; no actual RLE in displayed implementation despite header description | A communication codec only. Not exact unrestricted IDs: initial source65536 narrows delta to0; out-of-order−1 decodes as+65535. Require bounded domain and strict decoder before reuse; not a model mechanism |
| Semantic lexicon | Hand-authored small German classes/coordinates, role/grammar tokens, noise, compound reward, grammar templates | Fixed lexicon/scaffolding; not evidence of learned general prose or Rust |
| WorldChat | Averages known token coordinates, predicts latent state, full nearest-token lookup | Loses within-sentence order by construction; repeats a known class of UOR representation problem |
| WorldModel | h=tanh(Wih*x+Whh*h), Who*h tangent then exp_map; simple output-weight update | A conventional dense recurrent numerical model in geometry; serving matrix products excluded |
| ReasoningEngine | Fixed decompose/recall/associate/synthesize steps, nearest neighbors or transitional generation, hand-coded confidence constants; `_conv` unused | An interpretable orchestration shell, not validated multi-step reasoning. Tests establish nonempty chain, not correctness |
| ARC solver/vision/audio/reward/harness | Actual finite DSL/program search and modality encoders are inventoried; reports cover narrow classifications/ARC tasks | Retain donors for exact operator/program structure, not claimed frontier language capability |

Prior reported results retained: Phase2 DSL at1/400 ARC evaluation (exact0.25%, document rounds0.2%) and NO-GO, ~16/400 training; short-response grammatical rate0.3333, avalanche0.05 and criticality false in supplied JSON. Other audio percentages are unrelated tasks. No selected speed ratio should be used as cost-equivalent useful LM throughput. The new README-only commit does not change these mechanisms.

Useful import: low-overhead state ownership; finite program opcode interfaces when developing operator composition; explicit update/restore patterns. No wholesale model, MoA, geometry attention or dense world-model import is justified.

Primary sources at current verified head:
- https://github.com/unicornd47-afk/GoldSnnail/blob/a8b8affda6e3427d8d8005fc6c8f56d22a95bb41/src/routing/moa.rs
- https://github.com/unicornd47-afk/GoldSnnail/blob/a8b8affda6e3427d8d8005fc6c8f56d22a95bb41/src/attention.rs
- https://github.com/unicornd47-afk/GoldSnnail/blob/a8b8affda6e3427d8d8005fc6c8f56d22a95bb41/src/world_model.rs
- https://github.com/unicornd47-afk/GoldSnnail/blob/a8b8affda6e3427d8d8005fc6c8f56d22a95bb41/src/chat/thought_chain.rs

## SpiralCore provenance and reusable exact action

W/crates/uor-r4-core/src/spiralcore_operator.rs binds local v63 reference SHA256 `3f8e6a98186999cca6c55ea42cd8b496935837c2987379f39e8e659b56360215`. It implements oriented octonion basis/Fano products, left/right signed8×8 conventions, fifteen bivectors and finite64-state Cl(0,6) composition/inverse tables. Prime sextet `[5,7,11,13,17,19]` and semiprime plane correspondence are R4's explicit adapter, not something SpiralCore itself supplied. W/docs/adr/0003-fixed-zeta-prime-route-attention.md:298–337 records original static source audit and later exact Rust reproduction; W/docs/prime_route_attention_qualification_958.md and focused test file preserve finite evidence.

September 12 additive update: the owner supplied [SpiralCore v68 and Atomic Ternary FBS revision 3.1](../../../research/spiralcore-v68/README.md). Exact originals, hashes and scoped attention/address/Bell/trace consultation are preserved there. This is a separate research revision; the v63 Rust adapter and its claims remain unchanged. The inspected v68 routing is deterministic, and no learned shared-state credit rule was found. Its identity/route/observable distinction informs the current [credit-assignment trace](../../native_geometric_credit_assignment_973.md).


Current statuses are `IMPLEMENTED_CONTROL_ONLY`, `OPTIONAL_CONTROL_PENDING`, chart transport `NOT_ESTABLISHED`.64-state accumulation is useful ordered finite operator/holonomy state but necessarily collides many histories; cannot replace retained occurrence identities. It does not equate8D E8 with4D R4/H4 or Hopf base. A small signed-permutation/table transition could satisfy no-serving-matrix-products when its semantics are specified and actually lowered; dense8×8 matrix application is not necessary for a signed permutation. Chart rollover and the input/value/operator bridge remain required.

## Other ecosystem mechanisms already discovered, avoid duplicate rediscovery

W/docs/integration/uor-source-audit.md and afflom-ecosystem-followup.md retain original selected-source hashes/links:

- `hologram` immutable archive/store and deterministic operation-address memo keys; ordered operands matter, raw content identity differs from derivation identity. Good page lifecycle reference, no arbitrary whole-model constant-time claim.
- `kappa-registry` verified content/intake and store service; later artifact distribution/provenance, not semantic ranking.
- `atlas-12288` page-byte classifier/packing: R96 is b%96 and loses information (0 and96 collide); Lean/C packing differ outside page0..47. Rust calls minimal C rather than exported Lean runtime. Useful finite carrier only under stated domain; not address→language decoder.
- `UOR-Atlas-UTQC` exact Cartan/Gram constructions and modeled use cases, separate source with explicit assumptions; no measured quantum advantage.
- `afflom/matmul` at reviewed pin is template metadata, distinct from actual uor-matmul.
- LexLean controlled language→Lean/document/verification tools; lean4-prod extraction/codegen has bounded Nat/Int and primitive-specific correspondence obligations. Useful later for one selected finite operator proof, not an English semantic parser or proof of generated code correctness by naming alone.
- F1's RH/Hodge-index positivity stays open in the prior source audit. This review did not rerun or newly audit every F1 source; reuse its exact ledger and inspect a concrete lemma before adoption.
- HELM contains actual causal dense QKV/MLP softmax transformer references. It is a training/reference donor, excluded as serving architecture; its retained behavior remains separate.

## Recommendation to the architecture lead

There is no contributor import that supplies the missing general language learner while meeting the owner's serving constraints. The strongest immediately useful combination is already mostly in-house: exact prime/occurrence/value identity; learned placement in typed finite geometric state; an address/page selection rule with explicit absence; ordered bounded operators and causal writes; a trained non-dense token readout. Use UOR-ADDR/Framework for verifiable identity and finite-operation contracts, SpiralCore/H4 for candidate finite state/action algebras, and graph/W33-style pages for immutable storage when context scale demands it. Keep zeta/Hopf/chirality channels where they preserve implemented information; teach and measure their effect rather than forcing every named mechanism into a score.

The latest numeric-versus-word/source-admission bottleneck remains a supported concrete improvement because it concerns operator choice on already reachable information. But architecture review should first distinguish learned geometric placement/address evaluation from the explicitly excluded top-k expert gating. A selected finite arithmetic/copy/compare/composition operator is not an MoE network merely because its dispatch is learned; its source representation, semantics, work and parameter ownership must make this distinction concrete.

For a larger next step, build on the current causal state/value path with one jointly trained typed source/action selector and a small compositional finite operator vocabulary, rather than adding a parallel donor model. Demonstrate new complete language/coding behavior from committed derived state and emitted text; measure whole-path work/storage on M1. This is a plausible route toward the objective, not evidence that frontier capability is attainable at any particular resource level.

## Subsequent owner clarification and vector-bundle scope

After the initial report, the owner clarified that expert gates may eventually be necessary. Accordingly, every earlier “excluded” classification of learned expert gating or GoldSnnail LongLongMoA in this report means **outside the currently prioritized geometric/shared-operator design**, not a permanent ban on considering experts. The current review does not adopt expert banks. A later proposal must name the actual missing capability, describe its gate/operator ownership and compare full cost and generated behavior with the geometric shared-operator path. The serving prohibition on mathematical matrix products remains unchanged.

The owner also highlighted fibers and vector bundles. These are relevant to retaining local orientation and comparing transported states: declare the base space, the value carried in each fiber, local trivializations/sections and transition/connection maps. A Hopf fibration is a principal-bundle structure; it should not automatically be called a vector bundle without defining the associated vector representation. Framework's Hopf ontology supplies typed vocabulary; R4's actual transport code must supply the action. Signed finite permutations or exact transition-table actions are compatible candidates for serving. A learned useful state/operator bridge and measured language behavior remain necessary; the names “fiber” and “bundle” alone supply neither.
