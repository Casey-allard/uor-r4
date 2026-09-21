# Geometric attention: mechanism reuse and missing connections

**Current sequencing after PR #1338:** retain lexical decoding and advance [shared learned transitions with owned result/phase and complete response continuation](deepseek-shared-transition-continuation-step-2026-09-21.md). The [principal review](derived-state-decoder-review-2026-09-21.md) corrects missing supervised operation domains and the unsupported free-cell diagnosis. Three latent product corners constrain a fourth; unknown lexical decoding can still weaken identification. Reuse primitive actions from observed sequences, distinguish reversible H4 transport from noninvertible Read/Emit/Stop and memory replacement, and keep exact payload ownership outside compressed geometry. Relative H4/Spin/Hopf, finite spectral features and structural role/scope retention remain concrete supporting mechanisms; paired-H4/E8/S7 fields remain conditional. Codex retains principal leadership, DeepSeek substantive research autonomy. Dated next-task statements below are historical.

September 20, 2026; source basis `fbf542aa4c1f7ed336cd028c6b7b3502cba4118c`. This is a source/literature synthesis, not a new model result. The [canonical plan](project-track.md#structural-memory-and-geometric-representation-follow-up) owns sequencing; [current state](current-state.md) owns measured outcomes. The [correcting occurrence-reader audit](occurrence-reader-audit-2026-09-20.md) corrects the evidence for the immediate successor. The owner subsequently reconfirmed that the mathematical bridges should guide that successor, not merely be parked behind calibration. **The selected primary experiment is learned contextual query/source descriptions and directed H4 reads**, with a repaired/calibrated copy reader as comparator and loss-based integration in both arms. [Single execution prompt](deepseek-competitive-reader-step-2026-09-20.md).

## Architectural decision

**Active priority update:** exact-current-token admission and the existing two-neighbor equality observations are structural limitations, independent of the output boost. Test learned causal contextual descriptors and directed finite relationships now, drawing on `source_routing.rs` and `relational_attention/runtime.rs`. Use the old shallow reader with competent loss calibration as a baseline. If nonidentical-cue relations require broader proposals, choose one causal bounded pool shared by the compared selectors and isolate its coverage contribution. This advances the reduced Hopf/spin relation bridge without assuming general S7 or wave memory is already warranted.

Build geometric attention as **bounded query-conditioned access to exact memory, with learned scalar compatibility, directed finite geometric interpretation, and shared read/update/emit operators**. Keep three distinct kinds of information:

- Exact token/span, occurrence, source version and committed payload identity.
- Learned contextual geometry: ordered roots, relative action and any demonstrably useful retained phase.
- Structural state: scope, role, required evidence, legal replacement and retention/eviction status.

A wave or harmonic summary may eventually help the second or third function. It must not silently replace exact payload memory. Nor does a single compressed state supply all three: one 120-valued root has only `log2(120) ≈ 6.91` bits. The earlier shared-core diagnostic already found an empirical floor of at least 168 errors in 660 next-ASCII decisions when four ordered roots were collapsed to one code. That is a specific representation collision result, not a claim that geometric states cannot learn language.

A minimal proposed interface is:

```
A_t      = bounded_admission(query, exact_memory, scope)
r_i      = directed_relation(query.geometry, record_i.geometry, phases, scope)
s_i      = learned_scalar_compatibility(query, r_i, exact/contextual features)
a_t      = choose(NoRead, (record_i, copy_strength), or later typed Read)
h_next   = shared_update(h_t, selected_context, transported_relation)
z_next   = frozen_local_logits + bounded_selected_payload_contribution
```

The last two operations become a learned silent read/refine cycle when composition is needed. Sparse retrieval alone does not establish attention: candidate availability, query dependence, useful selection, causal use and integrated generation each need evidence. No dense query/key matrix is necessary for this interface; whether it learns sufficient language behavior remains open.

## Mechanism inventory and disposition

Paths in this table are relative to the repository. The [engine](architecture-2026-09/engines.md), [mathematics](architecture-2026-09/mathematics.md), [imports](architecture-2026-09/imports.md) and [September 13 attention](geometric-attention-research-2026-09/README.md) inventories provide the extended source map. Reuse is at the named operator/interface level; artifacts and tokenizers across model families are not interchangeable.

| Family / source | Connection to geometric attention | Evidence boundary and disposition |
| --- | --- | --- |
| Frozen E + S query-only prior; `learner/occurrence.rs` | Preserve local full-vocabulary emission while episodic memory supplies selected context | Local prediction gain is retained; repetitive generation remains. Static local prediction is different from dynamic binding |
| New exact-occurrence reader; `learner/occurrence.rs`, `bin/occurrence-reader.rs` | Causal 128-token ring, bounded 24-candidate admission, learned Read/NoRead, exact successor | Constructed selection gain; raw ranking/abstention/copy-strength failure. Current geometry is eight-class equality; see correcting audit |
| Exact ring/postings/local transport; `native_geometric/memory_runtime.rs` | Identity plus local source/query transforms; reject overwritten sequence references | Reuse mechanics. Historical `/4` response composition outperformed `/5`; first-token gains did not imply complete answers or Rust success |
| Owned objects/leases/commit; `addressed_attention/objects.rs` | Separate selected source, owned payload, action preparation and actual output commitment | 256 occurrences, 64-byte leases, eight results are scoped limits; bytes are not BPE tokens. Some snapshots allocate |
| Versioned history; `historical_read.rs`, `dependent_read.rs` | Assertions/corrections, exact predecessor identity and conflict validation | Rejecting a link does not prove absence or eviction. Retain same-value reassertion × initial/previous/current counterexamples |
| Hamming refinement; `hamming_refinement/{metric,runtime}.rs` | XOR/popcount proposals plus selected four-root context changes the next query | Exact single-root angular ranking correspondence; fixture policy authored. Up to 256+8 records scanned per hop; not a sublinear router |
| Learned relational attention; `relational_attention/runtime.rs` | Ordered relative roots and phase differences feed a shared learned scalar scorer | 64/64 constructed development, 32/32 changed-source; supplied four records/two-byte keys/one-byte values. Phase-disabled and diagonal-only also pass, so their unique benefit is unproven |
| Source routing; `native_geometric/source_routing.rs` | Ordered H4 accumulation, relative group elements, learned landmarks and explicit NO_SOURCE | Stronger geometric donor than equality of slots; authored input/artifact interface needs a BPE adaptation |
| Ordered correspondence; `dependent_language/occurrence.rs` | Injective occurrence witnesses distinguish repeated names and phrase order | 384/384 new rows with retained traces after repairing adjacency. ExactIdentity matched geometry; supplied boundaries and grammar remain |
| Learned source/query roles; `occurrence_role.rs`, `query_participation.rs` | Keep role, required evidence and replacement masks separate | Useful learned interfaces; unknown-neighbor keys alias incompatible cases. More fitting cannot separate identical observations |
| Contextual-role repair; `dependent_language/contextual_role.rs` | Identifies which scope/role distinctions were missing | The 2,304/2,304 panel uses explicit auxiliary/verb lists and clause rules. Reuse distinctions as learning targets, not as a learned language parser |
| Read/Emit/Stop; `dependent_language/scheduling.rs` | Silent read, updated query, committed emission and learned termination | 1,536/1,536 authored mixed answers; explicit `?` clauses. Reuse shared trajectory credit, not the grammar scaffolding |
| Hamming policy/LUT4; `hamming_policy/`, shared-core diagnostics | Typed query/extent/delta/control interfaces and nonlinear shared operators | Initialized 0/24 complete answers is not a trained predecessor. Avoid a giant random LUT circuit or collapsing useful context before learning |
| VSA/HDC; `learner/vsa_codes.rs`, `vsa/context_engine.rs` | Lossy semantic cue/sketch alongside exact source identity | Learned-root vocabulary has at most 120 codes. Transition bundling deduplicates codes. Common XOR on both keys/queries cancels in Hamming distance; arbitrary hash bits are not semantic distance |
| Angular/prime windows/pages; `prime_route_geometric_attention.rs`, `uor-r4-graph-runtime/src/route_attention.rs` | Conditional coverage-first shortlist and packed bounded selection | Constant candidate count does not remove window/page work. Coarse/fine VSA source recall is currently low; Q30 hierarchy may multiply |
| Prime identities, ordered n-lets, fixed zeta | Exact keys and structured phase features for shared relations | Commutative semiprime products lose order. Fixed phases need no RH proof; twin-prime spacing provides no semantic channel theorem |
| Exact H4 spin/phase; `bounded_global_exact_spin_attention.rs` | Table composition/inverse with separately retained fiber/torsion | Reusable finite substrate; signed state and SO(3) rotation equivalence must remain distinct |
| Hopf/base/fiber/JEPA; `hopf_metric.rs`, `learner/jepa_trainer.rs` | Observation plus retained fiber, contextual predictive training auxiliary | Older optional prose path exists; Q30 helpers multiply/divide. Predicting a next-token embedding is not predicting useful memory access |
| Gated delta retention; `geometric_gated_delta_retention.rs` | Selective overwrite as a model for addressed role/scope updates | Compiler-side floating matrix implementation is not the serving design; one-hot reduction motivates a finite sparse operator |
| SpiralCore signed operators and v68 diagnostics | Finite typed composition; distinguish missing, rejected, capped and stale routes | Chart transport/semantics explicitly unestablished. Raw octonion products require parenthesization; implemented signed linear operators compose associatively. Do not identify operator composition with octonion-label multiplication |
| UOR-ADDR/Framework/Prism | Bind identity, scope, operator and source versions; bounded graph metadata | Intersection nerves/Betti summaries are not learned semantic topology. No full graph/mesh engine required before a demonstrated composition need |
| GoldSnnail, NEMESIS, W33, HELM/dense R4/Spin | Selected arenas/opcodes, counterexamples, reference gauge tests | Existing audits preserve orientation-insensitive norm attention, Clifford/probe defects and scoped negatives. Do not revive whole engines or assume source compliance |
| TLA/R4G1 frozen graph runtime | Packed deterministic execution, fixed scratch, routing contracts | A separately scoped certified execution substrate; not a learned language model or proof of the new whole path |

## Connections that justify new experiments

### 1. Exact identity plus learned geometry prevents two opposite losses

A hash can preserve identity while having no semantic neighborhood. A shared geometric root can represent a neighborhood while merging distinct tokens. Store the exact reference and learn the descriptor separately. VSA sketches can propose candidates; exact references determine copied content and publication. This aligns with the finite associative capacity discussed in the [VSA review](https://arxiv.org/abs/2106.05268) and the conditional relation between attention and [Sparse Distributed Memory](https://arxiv.org/abs/2111.05498). Neither paper proves the repository's routing advantage.

**Test:** change an older binding while holding the recent suffix fixed; include tokens sharing a root and unseen payload assignments. Measure source availability before ranking. Do not allocate 4,096-bit sketches per occurrence without comparing compact shared-code alternatives.

### 2. Selective overwrite connects structural persistence to existing delta-memory research

For matrix memory written mathematically as `S' = S + (v - S k) k^T`, a one-hot key `k=e_j` changes only column `j`, replacing it with `v`. This algebraic reduction supplies a sparse write primitive without implementing the dense expression at serving. The unresolved learning problem is which slot, scope and version to write. Explicit role/scope banks make persistence depend on structural lifetime as well as age.

Recent [Sparse Delta Memory, July 2026](https://arxiv.org/abs/2607.07386) separates large memory capacity from sparse reads/writes; [HOLA, July 2026](https://arxiv.org/abs/2607.02303) combines compressed recurrent state with a bounded exact cache. They support investigating this decomposition, not importing their kernels or their quality claims into UOR-R4. [Engram](https://arxiv.org/abs/2601.07372) concerns static conditional lookup, a useful separate role from episodic occurrence memory.

**Test:** equal total bytes and training exposure; FIFO/shared bank versus learned role/scope retention under distractors, corrections, nested subjects and repeated names. A surviving pointer into an overwritten ring is not retained memory: pin/copy the payload or expire it explicitly. A bank identifier without scope collapses multiple active subjects.

### 3. Scalar compatibility and directed transport solve different problems

A scalar `Phi_theta(query, record)` can decide relevance or copy utility. It need not discard the relative element used for interpretation. For common-left frame changes, `r_ij = g_i^-1 g_j` is invariant; `g_i g_j^-1` instead has common-right invariance. Declare the action convention, signed state and phase semantics. A learned scalar field can be a finite integer table or bounded low-bit function; it does not imply gravity, a physical potential or supersymmetry. [Gauge-equivariant learning](https://proceedings.mlr.press/v97/cohen19d.html) supplies the general distinction between changing frames and transforming directional quantities.

**Test:** identical candidate support/payloads with directed relation versus scalar distance, equal-information categorical code and reversed relation controls. The old relational attention result makes this a concrete donor. Do not force sixteen all-pairs relations if diagonal/local relations suffice. Pure frame edges telescope around loops; nonzero holonomy is not obtained merely by renaming them transport.

### 4. Exact reads become attention for reasoning through shared state updates

The existing Hamming refinement and Read/Emit/Stop work can connect one useful read to a later query. This is more consequential than adding a higher-dimensional score to one-shot copying. A learned scheduler should choose whether to read, emit, stop or report a scoped unresolved state; it should not receive the gold hop count. A copied source, a computed result and an emitted token remain distinct objects.

**Test:** changing only the first retrieved relation must change the later read and answer; reverse relation order and include missing/ambiguous continuations. Use novel relation combinations and actual completed outputs. Transfer the owned-reference/commit discipline, not authored clause parsing. Include a derived output absent from all source payloads when claiming movement beyond copying.

### 5. A contextual training objective can connect JEPA to useful queries

The older JEPA path predicts token geometry. An optional auxiliary could instead predict contextual relations, useful source distinctions or an operator's effect, alongside actual next-token loss. [data2vec](https://proceedings.mlr.press/v162/baevski22a.html) is a primary reference for contextual targets. The inference query still uses only the causal prefix; future information is allowed only in declared training labels, not served features.

**Test:** token loss alone versus one auxiliary at matched capacity/dose; judge unfamiliar useful reads and generation, not latent loss alone. Charge target construction and quantization. Do not open a representation-learning campaign before a measured collision or learning failure motivates it.

### 6. Page routing and caching become useful after access quality is measured

Combine exact-key/recency candidates with geometric proposals only if source coverage or access work requires it. Measure recall, ranking conditional on recall, page touches and total latency independently. Reuse packed selectors and immutable pages; do not store every pairwise relation. At 4,096 nodes, all unordered eight-byte edges already cost roughly 64 MiB, before any payload; frame-derived edges can be reconstructed lazily.

For immutable ordered group prefixes `G_t=g_1...g_t`, a range product is `G_(l-1)^-1 G_r`. A commutative Fenwick update cannot be assumed correct for noncommutative H4; an order-correct mutable tree is conditional future work. No Fenwick implementation was found in the searched source. Cache keys for learned relevance must include query/model/source version and metric; content hash alone is insufficient.

## Where the new geometric ideas fit

The detailed [Hopf/spin/field note](structural-memory-hopf-direction-2026-09-20.md) supplies the mathematical corrections. The useful experimental order is finite relative H4/retained phase, then a specific representation comparison if those distinctions prove insufficient.

- **H4 and quantum-spin mathematics:** the binary icosahedral group gives finite unit-quaternion actions. Preserve signed spinor state where meaningful; `q` and `-q` represent the same SO(3) rotation but not necessarily the same encoded state. Relative phase may carry a learned relation. This is classical finite computation, not quantum hardware or a quantum speedup.
- **Hopf:** the old loop is `S3 -> (S2 base, retained S1 fiber/chart) -> S3`, with finite-precision/chart limitations. Quaternionic Hopf is `S3 fiber -> S7 total -> S4 base`. For `(a,b)=(g,h)/sqrt(2)`, its base `(2 a conjugate(b), |a|^2-|b|^2)` reduces to `(g h^-1,0)`, using the common-right action convention. Thus an existing relative quaternion lookup is already a cheap special case; demonstrate a need for unequal magnitudes or additional retained fiber before adopting general S7 state. A free pair of quaternions is not automatically the fixed golden/Galois-coupled icosian construction.
- **E8 and R8:** Euclidean eight-space is R8; normalization gives radius plus direction on S7 for nonzero vectors. The E8 lattice's 240 minimal roots give a finite codebook on that sphere after scaling, not 240 orthogonal channels or all of S7. Preserve radius/zero/identity where needed. A normalized finite codebook needs a declared closed update or projection rule.
- **Harmonic fields:** a function `F_t(x)=sum_alpha c_(t,alpha)Y_alpha(x)` is a coefficient bank, not one eight-coordinate point. Continuous orthogonality is an integral identity; sampled/quantized codes require a measured Gram matrix and mode-preserving writes. Multiple subjects in the same mode still collide. An indexed coefficient read is cheap only when mode assignment and its update were actually computed. Use such fields first as bounded context summaries or selection features while exact payloads remain addressable.
- **Symmetry breaking:** useful as a learned distinction between otherwise aliased contextual roles, with a matched generic-code comparator. The research question is what information is preserved and generalized. Twin-prime phases are optional basis controls; neither twin primes nor supersymmetry currently supplies a necessary attention operator. No GR/SUSY engine is a roadmap milestone.

The older [multi-resonance sieve audit](../multi_resonance_attention_sieve_audit_973.md) proposes positive unnormalized weights `w_i = epsilon + |A(q,k_i)|^2`, with `alpha_i = w_i / sum_j w_j` before aggregation. Pointwise positivity does not by itself establish a positive-semidefinite kernel. It remains conditional on a task requiring multi-source aggregation. Replacing the exponential does not remove all-prefix quadratic work, and finite feature expansions/normalization have real costs. Existing gauge/Lorentz negatives retain their exact scope; they do not prohibit a new finite operator with an independently stated test.

## Research controls and boundaries

The immediate relational-reader step should learn contextual descriptions and directed compatibility, together with *which source*, *whether to read* and *how much to influence output*. Calibration is a comparator/integration component, not the sole successor. [Pointer Sentinel Mixture Models](https://arxiv.org/abs/1609.07843) and [Pointer-Generator Networks](https://arxiv.org/abs/1704.04368) motivate separating copy from general emission; their dense architectures are not adopted. The current fixed 16-nat boost makes source mistakes extremely expensive. Offline loss-aligned learning may export a small finite set of additive/shift actions under D0-b.

Progress follows observed bottlenecks, not an obligation to run every row in this document. Freeze prospective usefulness/cost comparisons before fresh evaluation. A useful near miss may justify a revised practical threshold with a written reason; do not relabel the old result or relax identity, causality and evidence correctness. Preserve successful subcomponents and negative artifacts. A failed 8-class selector is not a falsification of every H4 mechanism; a successful geometric fixture is not general language.

## Review coverage

Three independent read-only audits covered current evidence/instruments, historical native memory/composition, and imported/research mechanisms. The repository inventory had 14,928 tracked paths before PR #1314; broad historical inventories and targeted source sections were reused rather than claiming a line-by-line audit of every file. UOR Knowledge supplied historical discovery records; available GitNexus snapshots dated September 3 and were not treated as current-source evidence. Current source and retained receipts control claims. TLA, W33, GoldSnnail, NEMESIS and dense reference engines were covered through their existing audits, not re-executed. The recalled softmax-tree source remains unresolved.

A further source caveat is preserved for any future VSA reuse: the exact random-hyperplane expectation `E[d_H]=D*theta/pi` assumes an appropriate isotropic distribution. Bounded uniform-integer cube normals do not justify that exact law, and finite integer cancellation need not have probability zero. Measure the actual finite code table; this does not require repairing a dormant path now.

No build, fit, model-forward campaign, deletion, paid compute or model promotion accompanies this synthesis. New role/scope, geometric transport, S7 and harmonic variants remain NOT_RUN in the current BPE path. Research sources were checked September 20, 2026; recent papers are external evidence for design hypotheses, not validations of this implementation.
