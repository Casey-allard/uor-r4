# Mathematics review: native reconstruction descriptors as attention objects

Source inspected: UOR-R4 `88c413599`, supplied Showcase `core.js`/README, supplied attention-note extraction and conceptual text documents. This is a source and mathematical review. No model/reference execution, training, pod work or architecture mutation was performed. Theorems below are hand-derived; their proposed implementations are NOT_RUN.

## Strongest coherent interpretation

The user's idea is mathematically coherent: an immutable reconstruction program together with a versioned shared decoder/store is an exact representation of its object. A model may operate on that representation without first expanding the object. This is a real architecture choice, not an information-theory violation. Shared prior structure is precisely what makes a short instruction useful. The question is which operations admit a cheaper implementation on those instructions, and which information/ordering/frame contracts those operations need.

There are two particularly concrete routes:

1. **Exact compressed-domain operations:** store compositional query indices on the immutable ternary DAG. A read uses these indices and returns selected exact payload references/carriers. For the current Flock comparator, a small exact index can be proved sufficient; see below.
2. **New descriptor-native relational read:** define a finite geometric state and sparse relation law on the DAG's nodes, use exact action tags and immutable payload references, and read its output through the existing writer. This could be useful even when it does not reproduce Flock. Its task value remains empirical; it should not be dismissed because it is not an implementation of conventional attention.

These can coexist. The descriptor identifies and reconstructs the object; its geometric fields support selection and lawful movement; its learned relation law decides which information matters for the current query.

## What “already compressed into vectors” means in this source

Current encoding is codebook lookup, `discrete.cu:268–270`. Exact token/occurrence/record/version identity is explicitly separate (`lib.rs:116–124`). The codebook's allowed root set has 16 members (`build.rs:216–237`), and each block stores eight integer coordinates. A root label therefore exactly represents a **known orbit vector** in four bits, provided the root table/schema is fixed. Four such blocks could be represented by 16 bits rather than materialized 32 integer coordinates. That is a possible exact storage representation of the carrier, not a claim that four roots uniquely encode the original arbitrary data or every token.

The current learned map is not constrained to be injective: encode performs no inverse/collision enforcement. Flock deliberately sums and snaps child payloads (`discrete.cu:148–153`) and selected payloads (`185–198`); those summaries are not invertible descriptions of all children. Exact ancestry is separately retained in `BinderPlan` (`metadata.rs:31–60`, `87–99`). The Showcase, in contrast, retains bytes and ordered child identities (`core.js:166–191`) and reconstructs them (`207–249`). Therefore the meaningful integration is to carry an exact descriptor/identity **with** its geometric state, not to assume the existing learned vector is already the exact decoder program.

This distinction supports the user's architecture rather than arguing against it: the exact descriptor could supply a capability the current carrier alone lacks.

## Factorization theorem: when can a query run on a compressed description?

Fix a decoder/store version S, an object domain X, and an encoder C_S: X → Z. For a deterministic query F_q: X → Y, an exact compressed-domain query H_q satisfying

    H_q(C_S(x)) = F_q(x)

exists **if and only if**

    C_S(x) = C_S(x')  implies  F_q(x) = F_q(x').

Proof: necessity follows by applying the same H_q to equal descriptors. For sufficiency, define H_q(z) to be F_q(x) for any x in the fiber C_S^-1(z); the implication makes the choice well-defined. This criterion applies to an entire query family by requiring it for every allowed q, model parameter state and causal cut.

If D_S(C_S(x)) = x, the criterion always holds: H_q = F_q ∘ D_S is an exact implementation. The remaining problem is efficiency. An implementation that expands everything is the existence proof, not the desired optimization.

If encoding adds novel nodes to S, the full exact state is (descriptor, shared S, missing-node delta). Treating the short descriptor alone as the representation would omit a dependency; counting that dependency is not an objection to the design. The Showcase actually implements this dependency and refuses missing nodes (`core.js:222–224`).

A summary σ(x) need not be lossless for all queries to be exact for a useful family. For example, a count is sufficient for a count query; a fixed-state automaton's block transition is sufficient for that automaton; an ordered list of top candidates may suffice for a particular selection law. A learned geometric summary can be useful without being sufficient for arbitrary future queries, provided backing children remain available and its read is evaluated as the operator actually defined.

## Exact compositional computation is the bridge

For an ordered three-child node with decoded value x₁x₂x₃, suppose a statistic σ has an exact composition rule

    σ(x₁x₂x₃) = combine(σ(x₁), σ(x₂), σ(x₃)).

Then a DAG can cache σ once per distinct content-and-model-version node and combine parent summaries without expanding byte leaves. Queries of the form F_q(x) = h(q, σ(x)) operate natively on this representation. Examples include exact counts, finite-state transition maps, fixed query-family candidate indices, and typed relations with a bounded interface. Whether a proposed semantic operation has such a rule is an explicit design question, not something compression guarantees or forbids.

For a finite-state tokenizer/transducer, a block can conditionally be represented by its map from incoming parser state to outgoing state and emitted output rope; these maps compose. This is an exact pre-tokenization route **if the actual tokenizer admits the stipulated finite state/boundary contract**. It is not established for our present tokenizer merely by proposing it. The universally exact, conservative route remains transport compressed bytes → decode → run the identical pinned tokenizer. Feeding compressed framing bytes to the old tokenizer instead defines a different sequence/model interface.

## A source-specific exact theorem for the existing Flock read

At fixed block b, level ℓ, causal prefix, BinderPlan and model state, let A be precisely the eligible node occurrences in the current kernel. The eligibility test is `level[node] == ℓ` and `last[node] + 1 < length` (`discrete.cu:165–168`), so every included node ends strictly before the query token. Each candidate has one of 16 frame roots. The comparator (`170–181`) is exactly

    (L1(query_frame, frame), coordinate_Hamming(query_frame, frame),
     last_occurrence DESCENDING, node_id ASCENDING).

There is **no** route-length or payload-distance term. The source transports selected payloads only afterward (`185–190`). The gate does not enter selection; it affects whether/how the result contributes (`160–164`, `192–194`).

Let K be the configured kept count, 1 ≤ K ≤ 8 (`82`, `120–122`). For each frame root r, retain the first K eligible records in A with that frame, ordered by `(last DESC, node_id ASC)`. Call this set I_r. Include exact occurrence identity and the payload or its exact versioned reference in every record. Let I be the union of the 16 I_r lists.

**Theorem.** For every query frame, the first K candidates in A under the actual Flock comparator are exactly the first K candidates in I. Thus I contains at most 16K records.

**Proof.** Take any discarded candidate a with frame r. It was discarded only because K candidates of the same frame precede it in `(last DESC, node_id ASC)`. All K have the same first two comparator components as a for every query frame, and beat a on the remaining exact tie key. Consequently at least K candidates precede a in the full order, so a cannot enter the global first K. Removing all such discarded candidates cannot alter the first K. If fewer than K candidates exist, none needed for the result is discarded. The source's deterministic node-ID tie breaker makes the order total within the defined candidate set.

Identical payload bytes at two positions remain **two candidate occurrences**. Deduplication of immutable data must not remove their occurrence keys or multiplicity. The retained list includes earlier binder nodes at level ℓ, not just current frontier nodes: the current Flock source scans every node of that level.

**Mergeability.** Under the fixed per-frame total order, let T_K keep its first K records. Then

    T_K(A ∪ B) = T_K(T_K(A) ∪ T_K(B)),

with union interpreted over unique occurrence identities, not unique payload bytes. Every element discarded locally has K better elements in its own input and therefore cannot enter the global first K. This gives exact ternary subtree combination. Each internal node contributes its own record at its own level; its children's candidate indices contribute their retained descendants. Relative occurrence offsets may be used only if composition restores the identical global ordering and canonical tie identity.

**Implementation contract.** Cache identity must bind:

- immutable source/root and exact BinderPlan/window span;
- causal eligibility cut;
- geometry/root-table and summary-schema identities;
- block, level and K/capacity;
- the parameter state which determines candidate frames and payloads;
- exact occurrence and version identities, including deterministic ties.

An epoch number alone is insufficient during current virtual `VectorBlock` proposals: `table_block`/`input_block` (`24–33`) apply a hypothetical selected-token/block action without committing an epoch. A cache must include that overlay identity or invalidate/update affected occurrences and ancestors. Gate-only proposals can reuse frame/candidate/payload indices because current gates affect only readout. Window sliding may re-anchor the ternary grouping, so a causal end position alone does not identify the BinderPlan.

**Cost scope.** A warm exact index replaces selection's per-level scan over all eligible nodes with at most 16K candidates. It also permits selected payload materialization only when needed, if their cached/root descriptors are valid. Constructing the index first still processes its dependencies; retaining indices at many DAG nodes consumes storage; model changes may invalidate extensive dependencies. The existing writer's vocabulary scan (`229–246`) and proposal repetitions remain. This theorem proves an exact way to compute **the same current selection law** more compactly. It does not prove that this law supplies the missing relevance/attention, that end-to-end inference is logarithmic, or that a speedup has been measured.

## Geometry can be carried as reconstruction instructions

There is also a fully exact way to transport arbitrary pre-tokenization bytes through this actual finite geometry. Fix the canonical enumeration of the 16 orbit roots r₀,…,r₁₅. Encode a byte b as the two roots `(r[b >> 4], r[b & 15])`. Transport both by an allowed invertible action g. The receiver applies g⁻¹, looks up the two canonical root labels and reconstructs `(high << 4) | low`. Thus arbitrary bytes can be carried in genuine geometric symbols, with no assertion that arbitrary coordinate bytes themselves lie on E8. If each root is packed into four bits, the symbol code alone uses eight bits per input byte, plus action/schema framing: it is exact encoding rather than entropy compression. Applying a lossless byte compressor or shared-store instruction codec first, then this geometric symbol code, gives the user's proposed compression-before-tokenization loop. Decompressing before the pinned tokenizer returns identical token input. Root distance in this arbitrary nibble assignment is not thereby a learned semantic distance, and coordinate learning must not silently change the exact byte decoder convention.

The implemented action is an exact signed permutation (`discrete.cu:19–21`). Let D be a decoder whose leaves are geometric payloads, and let g act on each leaf. Extend the descriptor grammar with a typed `Map(g, z)` node whose meaning is

    D(Map(g, z)) = g · D(z).

Then applying an action need only append/compose an action tag until a read demands a payload. `Map(g, Map(h,z))` can reduce to `Map(gh,z)` using the exact composition table. Ordered concatenation commutes with a pointwise action. This is a genuine lazy transport mechanism: move the instructions and reconstruct transformed values at the endpoint.

This is not an assertion that BLAKE hashes themselves transform geometrically. A descriptor with an action tag has its own typed identity; the original raw-byte digest and the new reconstruction-program identity must not be confused. It also does not follow that transforming carrier coordinates transforms arbitrary original bytes. That requires an explicit byte/carrier decoder intertwining the two actions. For arbitrary bytes the safe semantics are that payload bytes remain exact and unchanged while their frame/address metadata moves.

For exact frame comparison, payload and frame are separate types. Current Flock already follows that distinction: frame block selection (`106–107`), query frame/payload (`133–138`), source-to-query BFS action (`185–188`). Preserve the actual group/action identity when composing transformations; a root label alone need not identify the transformation which reached it. Independent canonical endpoint BFS paths are not, by their construction alone, a proof that all endpoint-path compositions satisfy a cocycle law. Store cumulative actions or explicitly prove the required path convention.

One important boundary: the current `snap` chooses the first root on ties (`95–104`). It is therefore not equivariant under every nontrivial common group action. Hand-derived counterexample: `snap(0)` selects the first root r₀; a group element moving r₀ exists because its orbit has 16 roots, but g·0 = 0, hence `snap(g·0)=r₀ ≠ g·r₀`. This does not invalidate signed-action transport. It says aggregation has a canonical frame/tie convention that must be retained. A proposed compressed read can operate consistently in that canonical frame or carry the convention covariantly; it should not claim all intermediate snaps are frame-independent.

## How the proposed resonance idea can fit without importing floats/matmul

The conceptual transfer worth considering is **relational compatibility dynamics**, not a literal import of the note's gradient/CE/continuous equations. A compliant candidate could use finite action/phase labels g_i on selected immutable descriptor nodes, a fixed sparse typed edge set, and integer relation energies φ_e. The payload remains immutable while routing/phase labels change. The energy is a sum of bounded table terms, and candidate changes use permitted finite geometric moves.

If each sequential accepted move lowers an integer global energy by at least one and that energy has a lower bound, there can be at most E_initial − E_min accepted moves before termination. This is a direct descent proof. It requires that each move compare the actual affected global terms and that simultaneous moves not silently invalidate the comparison. It proves termination at a state with no accepted improving move, not a global optimum or semantic correctness. A finite iteration budget guarantees cost but may stop before even that local condition.

This can act as a new read/reranker over reconstruction descriptors and return payloads to the existing writer. Its learned weights can be bounded typed relation choices or action tables rather than a dense per-token coefficient grid. A finite table is not automatically a disguised matrix multiplication: the function represented and actual gather/add/compare execution must be audited. Conversely, renaming a full all-pairs learned coupling array does not supply sparse cost. A sparse graph must be specified.

Whether the useful dynamics should be phases, group actions, constraints or tree messages is an open design question. No mathematical result above requires arbitrary numerical acceptance thresholds, continuous floating point, or a conventional transformer baseline.

## Questions for the other specialists

- **ML:** Which missed linguistic relation is the descriptor-native read meant to recover that current frame-distance/recency selection cannot? Does the proposed query admit an exact sufficient index, or is it intentionally a new approximate task operator? Preserve distinct transport correctness and task usefulness.
- **Systems:** Can the exact 16K index be maintained across the actual virtual proposal schedule without rebuilding most nodes? Measure dependency invalidation, not just warm serving lookup. Can payload root IDs stay packed until the existing writer boundary?
- **Networking:** Bind decoder/schema/model versions and occurrence identities in the reconstruction protocol. A warm byte cache does not automatically imply a warm learned-query cache, and the reverse need not hold either.
- **Physics:** Which physical coupling can implement the desired finite relation, with input/readout and synchronization costs included? If the law is implemented digitally, what is its actual bounded edge/update count?
- **Integrator:** Keep the compression benefit and attention novelty separable. An exact Flock index is a low-ambiguity bridge; a finite relational descriptor read is a separate candidate for better attention. Both reuse SpiralCore rather than replacing its core idea.

## Assessment

There is a defensible architecture here. The most concrete newly established connection is that current Flock selection already admits an exact compact, mergeable query index on the ternary structure. The more ambitious reconstruction-program/relational-head architecture is also mathematically coherent, provided its query and state contracts are explicit. Existing source establishes exact bytes, exact finite actions and a concrete current comparator; it does not yet establish the cache implementation, its measured benefit or the new head's language behavior. None of those absences warrants rejecting the idea before a fair, specified construction test.
