# Mathematics adversarial review of the integrated candidate

Reviewed `integrator.md`, `physics.md` and `parent-challenges.md` against the first-pass proofs and current source at `88c413599`. No computation or implementation was run. This review authorizes neither model changes nor compute.

**Assessment:** the integrated architecture is coherent as a research candidate. Its exact compressed-state path has explicit sufficient conditions; its relational head remains a specific family of possible new laws, not yet an executable design. The evidence supports investigating this composition. It does not support declaring superior attention, universal compressed-domain speedups or automatic semantic correctness. The gaps below are contracts to resolve in a finite design card, not grounds for closing the mechanism.

## Resolved: exact descriptions may be native model objects

There is no mathematical obstacle to attention operating on reconstruction descriptions. With a fixed exact decoder D, any query F has a representation-level implementation F∘D. Efficient direct operation requires more: an exact compositional statistic or a query algorithm which visits only the required dependencies. The integrator states this distinction correctly. Its five object types avoid conflating exact bytes, token identities, occurrence identities, geometric coordinates and learned derived state.

The strongest constructive evidence remains the source-specific query-sufficiency theorem for the current Flock comparator. That makes the review more than a generic analogy: an actual read in this repository admits a smaller exact selection index. It does not prove that current selection quality is adequate or that the proposed interaction law has already been supplied.

## Required clarification: exact 16K scope and trace parity

The theorem is valid only for the current order:

    frame L1, frame coordinate-Hamming, last occurrence descending, node ID ascending.

Every same-frame candidate has identical first two keys for every query, so a candidate behind K same-frame records cannot enter the global top K. It is not a universal attention compression theorem. In particular, a candidate-candidate relation can make an old discarded candidate relevant, invalidating the dominance argument. A new law must admit its own pool or establish a new sufficient statistic.

The fixed conditions are the actual BinderPlan, eligibility cut, K, geometry schema and effective model state, including virtual VectorBlock proposal overlays. The integrator includes these. Gate-only changes do not alter current selection. Exact payload reconstruction must reproduce current frame conventions, sums, canonical root order and ties.

**An additional necessary instrument field:** current Flock records total eligible candidates in its trace. `discrete.cu:166` initializes `eligible`; line 169 increments it for every eligible node; line 183 writes it into the level trace. A shortlist by itself preserves selected candidates and output but loses this count. Preserve an exact mergeable eligible count per level or per frame alongside the K-lists before claiming full read/trace/instrument parity. Similarly, preserve original BinderPlan node IDs rather than substituting content hashes. Counts combine over disjoint occurrences, not deduplicated content identities.

The current parent frame is inherited from its first child at `discrete.cu:148`; the integrator's citation to line 144 should be corrected. Frame-index construction therefore need not eagerly compute snapped parent payloads. However, computing a selected high-level payload can still require most descendants. At C192, K8 permits a 128-record shortlist, and many higher levels already have fewer candidates. No measured speedup follows from the bound.

## Required architecture fork: storage tree versus model tree

The Showcase atomizes bytes into fixed 32-byte pieces (`core.js:166–183`). Current model `BinderPlan` atomizes token occurrences (`metadata.rs:31–60`). These are not the same tree. A token can span byte blocks; an alternative byte compression program can change the storage graph without changing decoded content.

Three defensible contracts are available:

1. Keep the model's canonical token-occurrence BinderPlan distinct from the transport/storage DAG. Exact byte references and span mappings connect them. Current read parity then has a definite target.
2. Define a content-invariant compositional read and prove its output is unchanged across admitted equivalent reconstruction programs.
3. Intentionally make reconstruction-program structure part of the learned input. Output may then depend on the program's structure; this is a new representation whose transfer across encodings must be evaluated, not advertised as automatically content invariant.

The integrator mentions encoding invariance, but this concrete byte-tree/token-tree distinction should be explicit in the final report. Shared three-child branching does not alone give compatible node semantics. This is a manageable interface choice and does not undermine the shared reconstruction-program idea.

## Geometry: exact transport is not automatic aggregate equivariance

The implemented signed actions are exact and invertible. A lazily transformed descriptor is valid if its decoder explicitly satisfies

    D(Map(g,z)) = g·D(z),

and composition uses the actual action table and decoder convention. The arbitrary-byte two-nibble code provides a direct exact example: compress bytes, map each nibble to a canonical orbit root, transport with a known action, invert that action, decode nibbles, then decompress. This is exact geometric symbol transport; the root labels alone do not endow raw byte numerical distance with semantic meaning.

The existing snapping rule breaks general equivariance at ties. `snap(0)` chooses the first root r₀, while a nontrivial group element moving r₀ leaves zero fixed. Hence `snap(g·0) ≠ g·snap(0)` for such g. The integrator correctly retains a canonical frame/tie rule instead of promising frame independence.

Relatedly, a root labels a point, not necessarily a unique action. Independent canonical BFS paths between endpoint roots do not themselves prove path-composition/cocycle identities or conjugacy under a common frame change. To claim frame-covariant composition, retain full frame/action representatives or prove the chosen connection law. Merely using an exact action at each step is enough for lawful local transport, but not for every stronger global invariance. The present candidate need not require that stronger property if its canonical conventions are explicit.

## C6 versus G64: the proposed distinction is valid but narrow

The existing action group has order 64. Consequently it has no nontrivial subgroup or element of order three. This rules out identifying a **faithful cyclic 120-degree phase rotation** with an element/subgroup of that group. It does not rule out selecting three distinct states, constructing a frustrated three-node relation, or representing three-way distinctions using G64 labels. Keep the subgroup claim at its actual scope.

A separately typed C6 controller is a coherent finite exact mechanism. The six-entry integer table `[2,1,-1,-2,-1,1]` exactly represents twice cosine at the six allowed phase differences. Using it to evaluate a finite sparse edge law requires no runtime cosine, float or normalized exponential. Its output can select exact descriptors while G64 actions transport their unchanged payloads. This composition does not assert that C6 is a subgroup of G64 or an isometric relabeling of E8.

The physics memo's equal-negative-triangle minimum is correct under its stated three-edge law. Scaled energy is the sum of pairwise doubled cosines. The identity

    2Σ(i<j) cos(θᵢ−θⱼ) = |Σᵢ exp(iθᵢ)|² − 3

gives lower bound −3, attained by C6 states `(0,2,4)`. This is a mathematical derivation of a finite construction, not a model test or a language claim. A negative pair alone instead prefers opposition. Six phases do not include 90 degrees, so the note's independent I/Q claim is outside this realization.

**Edge-orientation requirement:** for a term `−J C[p_i−p_j−δ_type]`, either store and evaluate an oriented edge once, or reverse the sign of δ when reversing that edge. Arbitrarily duplicating offsets in both directions can define a different energy. The update must compare the true total affected edge/unary terms. This is essential for the descent argument, especially when directed semantic relations are represented by oriented energy terms. It is not an obstacle: a discrete energy can encode a directed role through an ordered pair potential, provided the actual total potential is what updates evaluate.

If a G64-state energy uses only the image of a single anchor root, different actions in the anchor's stabilizer are indistinguishable to that energy. A 64-label controller does not automatically provide 64 effective distinctions. Use root-state variables when that is the intended quotient, or define the additional payload/action-dependent terms that make full action labels meaningful. This should be settled in the design card rather than assumed from the label count.

## Sparse relation energy: what must be designed before an executable test

The integrated candidate appropriately separates immutable evidence from mutable controller state. Otherwise all nodes could agree by erasing the original distinctions. It also separates inner energy from the outer task loss through the writer. Both are necessary.

The remaining design decisions are substantive:

- the candidate pool and causal admission algorithm;
- which edges come from actual structural/occurrence/version facts and which require learned inference;
- the finite state, relation parameters, permitted moves and ordered update schedule;
- unary anchoring from observed input/query geometry;
- how final relations alter selection, including retrieval of useful contradictory evidence;
- how new learnable parameters enter the actual shared-writer proposal/evaluation path.

Parent/child links are observed structure. They are not, by themselves, evidence of semantic support or contradiction. Hand-authored relation labels can qualify a finite kernel's response to those labels, but cannot establish that the model derives those labels from text. The eventual end-task experiment must use the actual input-side edge rule, without answer-aware graph construction.

Keeping query-to-candidate distances fixed while changing another candidate is a useful interaction control. A changed output proves sensitivity to that change only; its *correctness* must follow from a predeclared task with independently specified relevant evidence. A mechanism which changes on every irrelevant neighbor would also pass a bare sensitivity test. Include relevant change and irrelevant-change controls, plus missing-candidate and contradictory-evidence cases. This is already mostly anticipated by the integrator; the final report should retain that causal scope.

## Boundedness proof and cost boundary

Strict sequential decreases of a lower-bounded integer total energy prevent accepted cycles. With unit-or-larger decreases, accepted moves are bounded by `E_initial−E_min`. For C6 ternary pair couplings without additional magnitudes, each edge term lies in `[−2,2]`; M edges contribute energy range at most 4M, plus the summed unary ranges. This is a useful explicit conditional bound.

It does not bound rejected trials or say the energy reaches a global minimum. Declare a sweep/move cap, deterministic tie behavior and cap-hit output. Simultaneous local moves need noninteracting edges or a checked joint acceptance rule. Computing sparse neighbors must itself be bounded; a low-degree output graph can still hide an all-pairs graph-builder search.

Neither descriptor size nor finite alphabet size accounts for graph construction, cache invalidation, reconstruction, controller trials, writer scans or the outer learning proposals. The candidate is potentially useful if that complete cost buys a material read capability or saves repeated work; no existing evidence measures that proposition.

## Final resolved and unresolved list

**Resolved by source or derivation:** exact reconstruction is a valid native representation; exact operations can factor through sufficient summaries; current Flock admits the ≤16K selection index; arbitrary bytes can be geometrically encoded with a pinned inverse; finite C6 relational energy is compatible with exact integer execution; strict sequential integer descent has a conditional termination bound.

**Required corrections/clarifications for the synthesis:** add eligible-count metadata for instrument parity; correct parent-frame citation to 148; distinguish byte storage DAG from token model tree; state C6 edge orientation; keep G64's order-three limitation narrow; do not claim 64 effective distinctions from an anchor-only energy.

**Unresolved empirical/design questions:** efficient implementation and invalidation cost; a complete non-oracle graph/readout/learning law; task advantage over the existing geometric read; compressed-domain output invariance when promised; complete latency/memory/energy cost. These support a specified next construction review. They do not justify either adoption or rejection of the proposed attention mechanism now.
