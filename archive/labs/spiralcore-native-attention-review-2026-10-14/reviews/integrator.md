# Systems integration candidate: make the reconstruction program a native read object

Codex specialist role: visionary systems integrator. Source snapshot `88c413599`. This memo synthesizes the mathematics, networking, ML, physics and computer-systems reviews; it is not a claim of external-lab consensus. It proposes contracts for adversarial review rather than adopting a mechanism. No model computation, source changes, training, paid resources or new qualification claim occurred. Training remains held.

## The architecture the user is actually proposing

The strongest interpretation is a model operating over **executable descriptions of data**, where a description names immutable content, supplies lawful transformation/reconstruction instructions, and carries the geometric metadata needed to choose what to inspect. The same description can be stored locally, addressed across a network, moved through a geometric frame, and expanded when an operation needs its contents. Attention becomes the operation that decides which descriptions and relationships matter for the current query, and which parts must be reconstructed.

This is coherent. Having a shared instrument, decoder or dictionary is an enabling feature. The meaningful questions are what the model can compute directly on the descriptions, whether that computation preserves the information required for its task, and when the total cost beats eagerly expanding everything. Exact reconstruction and useful attention are separate properties that can be composed.

The full HTML provides more of this compatibility loop than the earlier assessment emphasized. Its active `NATIVE_63_ROUTE_GOLDEN_PCM_BLOCK32_V1` path constructs a score, generates a geometric route performance and uses a separate receiver to recover choices and ordered FBS data. The networking review traces the overriding implementation at HTML line 10861 onward and the shared instrument at roughly 10680–10714. The source and embedded author receipts establish what is implemented and reported; this review did not independently execute that waveform path. The separate Showcase supplies independently exercised shared-ancestry transfer. Their different compression targets and implementations should remain identifiable.

The proposal does **not** need to prove that every random input becomes a tiny standalone message. It needs useful workloads where shared structure, retained content and compositional operations make instructions substantially cheaper than repeatedly shipping or materializing their consequences. The existing cold/warm transport evidence demonstrates such conditional savings for its tested inputs. Whether the same pattern pays off inside the model is the new question.

## Five objects, one compatible system

The implementation should keep five types distinct:

1. **Exact content:** immutable bytes and ordered child descriptions. A decoder/schema identity gives the program a definite meaning.
2. **Occurrence:** where that content appeared in a particular document/version, with ordering, span and causal position. Repeated content can share storage while retaining repeated occurrences.
3. **Model symbol:** the pinned tokenizer's token ID and any byte-span mapping. This remains separate from an object's hash or address.
4. **Geometric carrier:** the learned root tuple, its frame/action history and model version. This is what native transport/read/output consume.
5. **Derived read state:** query-sufficient indexes, geometric summaries or temporary relational controller states, bound to the exact source, BinderPlan and current model/proposal state.

This distinction is a practical compatibility rule, not extra ceremony. A reference can preserve the exact object while its learned geometric interpretation changes. A single shared vector writer can unify native learned output without pretending that a byte digest, a token ID and a geometric coordinate are the same identity.

The current token carrier is not a reversible compression program. `discrete_encode` in `crates/uor-r4-discrete/src/discrete.cu:268–270` looks up a learned codebook; identities are separately retained in `src/lib.rs:116–124`. Its allowed persistent blocks do have an exact compact encoding: each is one of sixteen roots, so a root label needs four bits. At B4 the geometric payload could occupy two packed bytes instead of 128 expanded coordinate bytes. This is a representation-size derivation, excluding identities, schemas, indexes and tables. A V4096 token identity needs only twelve information bits; a sixteen-bit root tuple is not a compression of that identity, nor a guaranteed inverse of tokenization.

## A concrete compatibility diagram

```text
Original bytes
  -> exact reconstruction program + shared object store
  -> transport descriptor and missing dependencies
  -> verify decoder/schema/content identity
  -> pinned tokenizer / retained token-and-byte-span interpretation
  -> immutable occurrences + versioned geometric annotations
  -> [exact current read OR explicitly new relational read]
  -> selected occurrence/payload descriptions + lawful action tags
  -> materialize only required payloads, apply SpiralCore actions
  -> existing integer combine and shared vector writer
  -> emitted token IDs / exact referenced source spans
```

The source store and read metadata can remain compressed throughout most of this loop. A selected payload need not expand into all original bytes to supply a root vector. Conversely, an application asking for an exact quotation or original file can follow backing references and reconstruct bytes. Those two requests have different decoder outputs but compatible identities.

For a first integration, ordinary RAM/storage and an ordinary transport envelope suffice. Requiring a new Internet routing stack, analog device or audio channel would unnecessarily tie the model question to deployment choices. The geometric addresses and action semantics remain native to the application.

## Compression before tokenization: three genuinely different contracts

**Exact transport first.** Compress the original bytes, move/store the reconstruction program, then reconstruct identical bytes and apply the identical tokenizer. This is directly compatible with the present language input. It can save transfer/storage without changing the attention law. Whether it saves end-to-end time is a measurement question.

**Retain token interpretation while keeping backing bytes compressed.** At ingestion, tokenize once and retain token IDs and byte spans alongside the exact compressed object. Later queries can select compact token/geometric descriptions, and only source quotations or missing values require byte reconstruction. This avoids assuming that arbitrary byte chunks coincide with tokens. An incremental compressed-domain tokenizer is also possible, but only with the actual tokenizer's boundary/parser-state contract; it is not implied by the ternary tree.

**Learn over reconstruction operations.** Treat typed `Literal`, `Reference`, `Concat`, `Repeat` or `ApplyAction` descriptions as input objects. This is the most ambitious version of the user's idea. It could expose reusable compositional structure directly, but it is a new input representation and a new learned mapping to the shared writer. It is not a free substitution of compressed bytes into the old tokenizer. Different programs can reconstruct the same content, so the experiment must specify canonicalization or the exact output invariances it promises.

In particular, the Showcase's storage atoms are 32-byte pieces, while the current BinderPlan's atoms are token occurrences. Sharing a ternary branching rule does not make them the same tree. For current-read parity, retain the canonical token-occurrence BinderPlan and its node IDs separately from the byte storage DAG, connected by exact spans/references. If storage chunk edges instead become the new head's semantic edges, changed chunking can change its behavior; that is a deliberately different input law unless content invariance is established.

None of these routes requires searching for an optimal compression program merely to begin. A fixed exact codec can provide the substrate first. Learning better reconstruction programs is a separate hypothesis.

## Path A: an exact implementation of today's read on compact descriptions

The mathematics and ML specialists identified a specific bridge to the existing Flock law. At a fixed payload block, level, causal cut and model state, Flock ranks candidates by frame L1, frame Hamming, descending recency and node ID (`discrete.cu:165–181`). Within one of the sixteen frame roots, the first two keys are identical for every query. Therefore the best K occurrences per frame root contain the original global top K for every query frame. Their union contains at most 16K records, and these per-frame lists compose exactly across child lists.

This is an exact query index, not a claim that arbitrary context information compresses into 16K records. It preserves occurrences, original BinderPlan node IDs and their payload references. Complete instrument parity also needs exact eligible-candidate counts alongside the lists: current Flock increments that count for every eligible occurrence and records it in the level trace (`discrete.cu:166–183`). Counts combine over disjoint occurrences, even when content is deduplicated. The computer-systems specialist adds an important source fact: a parent's frame is the frame of its leftmost child (`discrete.cu:148`), so constructing this selection index need not eagerly compute every snapped parent payload.

Selected payloads can be constructed lazily. Allowed actions can operate on packed roots through a fixed 64×16 action table, and distances through fixed 16×16 tables derived from the admitted coordinate roots. Hamming distance between four-bit label encodings is not the coordinate Hamming distance or L1 metric; the existing root ordering and tie rules must survive packing. A root ID names a state, whereas an action ID names a transformation. Distinct actions that agree on one frame can act differently on another payload, so a state label cannot replace a required cumulative action. Only the selected combination needs the existing bounded integer sums and canonical snap. The same writer still decides admission and token ranking.

There are material limits. A selected high-level payload may depend on almost the entire context; lazy materialization is not an unconditional sublinear guarantee. At the present C192 shape and K8, the maximum shortlist is 128 records, while some levels already contain fewer candidates. Index construction, reuse, table overhead and payload work must be measured separately. A cache must bind the actual BinderPlan and virtual proposal overlay, not only the committed epoch. A VectorBlock proposal may change every occurrence of the affected token and many ancestors. Dependency-specific invalidation can be better than discarding everything, but it needs proof and measurement.

This path could make today's read cheaper or smaller. It does not add the missing relational capability the user suspects, and its implementation is **not a prerequisite** for testing a new head. The new head can first use the current expanded carrier.

## Path B: a genuinely different attention law over the descriptions

The possible new contribution is **candidate-candidate interaction in selection**. Today's Flock order is determined independently by each candidate's frame distance and tie keys. Flock already transports and combines candidate payloads and constructs ternary consensus; interaction in aggregation is not new here. A relational head can instead change a candidate's priority because another candidate supplies support, conflict, role binding or version context, even while the query and that candidate's frame stay fixed. This is a concrete distinction worth testing.

A minimal candidate has this contract:

- Freeze the exact causal candidate pool, source content, occurrences and input geometry for the query. Initially use all eligible nodes in the existing bounded BinderPlan, or explicitly declare any narrower admission law. C192 has 286 retained nodes before causal/per-level exclusions, from `(3C - digit_sum_base3(C))/2`. The current 16K Flock index is **not** automatically sufficient for a new relational law; an older candidate it removes could become relevant through a relationship.
- Construct a sparse graph from explicit parent/child links, query anchors and a bounded declared set of relation links. Specify the edge-generation algorithm as well as its output degree, so it does not hide an all-pairs search. The graph cannot receive answer labels or future content as relation hints.
- Preserve immutable evidence frames/payloads. Keep temporary controller states separate, with query and input-derived anchors. Otherwise the controller could erase a contradiction simply by changing everything until it agrees.
- Update finite states through exact table/gather/add/compare operations under a deterministic schedule. Strict decreases in a bounded integer energy prohibit accepted cycles. A separate move/sweep limit bounds runtime even when the process has not reached a local optimum.
- Produce typed relation outcomes or an ordered selection of exact candidate references. Use unchanged SpiralCore payload transport, the declared integer combination and the existing writer initially, so the causal change is identifiable.

The learned controls are finite choices: which declared relation type attracts, repels or ignores; which action/phase offset a structural role prefers; how an input frame initializes or anchors controller state; and how relation outcomes affect candidate priority. Their learning must use the actual outer task objective through the shared writer, with the current exact finite proposal protocol or an explicitly reviewed extension. The admitted learning dose must actually attempt the new coupling/readout slots; an arm that never updates them is not a trained-in test of the new head. Specify three contracts separately: internal energy and updates; deterministic final reference/action readout; and the outer shared-writer objective. Decreasing an internal synchronization energy is not evidence of improved language loss, admission, truth or usefulness.

### Two honest finite realizations

**Existing geometric state.** Use root/action labels from the current SpiralCore group and integer geometric table costs. This fits the present carrier closely and tests finite geometric consistency. It should be called that; it does not reproduce every claim about continuous oscillator phases. If an action-state energy observes only the image of one anchor root, its effective state is the sixteen-root quotient, not sixty-four distinguishable states. A full-action controller needs additional action/payload-dependent observations, or should explicitly use root-state variables.

**A typed C6 phase controller.** If the note's signed coupling and 120-degree frustrated triads are the essential hypothesis, use a separate six-state phase variable. Twice the cosine at its six equally spaced phases is exactly the integer table `[2,1,-1,-2,-1,1]`. Ternary edge signs select attraction, repulsion or no relation, with sign/zero operations and table lookup. The controller returns references/actions; SpiralCore continues to transport the original geometric payload. The current transport group has 64 elements and cannot contain a nontrivial order-three subgroup, so identifying its states directly with a faithful 120-degree phase cycle would be incorrect. C6 also has no 90-degree state: it does not establish the note's I/Q orthogonality claim.

The order-three statement is narrow: it does not prohibit three-way distinctions or frustrated three-node relations over G64 labels. For C6, a term such as `-J C[p_i-p_j-delta]` also needs an oriented-edge convention: store/evaluate the ordered edge once, or negate its offset when reversing it. Each proposed state move must compare the true affected total energy, including all incident and unary terms. With ternary signs and no extra edge magnitudes, each pair term lies in `[-2,2]`, giving a total pair-energy range at most `4M` for M edges, plus unary ranges. That conditionally bounds accepted strict decreases; rejected trials and graph construction still need the separate work limit.

Neither realization is currently selected or qualified. A root-to-phase assignment is a learned controller parameter, not an isometry or an automatic semantic map. Retaining the original frames and exact descriptors limits what that small phase alphabet can erase, but usefulness still needs an actual readout and task test.

The physics specialist's strongest correction is that disagreement can be useful evidence. An anti-aligned candidate may contain the exact contradiction the query needs. Therefore a universal “keep the most synchronized” readout would be an inadequate default. The readout must be query/role conditioned; supportive and contradictory evidence should not be blindly averaged into one consensus. The first implementation could use relation states only to rerank, keeping payloads immutable, while separately reporting each selected item's relation/provenance.

## What makes this more than a renamed Flock head?

A construction control must hold the query-to-candidate metrics fixed, change only another candidate or a declared relation, and show a corresponding justified change in selection. If selection cannot change, the interaction has not demonstrated its proposed new capability. Sensitivity alone does not establish correctness: a head that responds to every irrelevant neighbor would satisfy a bare change test. The task must independently specify which evidence matters, with matched relevant-change and irrelevant-change controls. If selection changes only because an answer-aware graph builder supplied the right relation, the result demonstrates that helper, not learned attention.

Useful controls include same-content/different-occurrence multiplicity, a relevant contradiction, an irrelevant extra node, a missing relation, a missing candidate, and a conflicting version. They test the declared mechanism, not unrestricted language understanding. A later language assessment must obtain relationships from the actual model/input path rather than supplied task labels.

Synthetic edge-on/edge-off controls can establish a kernel's response to a declared relation. They do not establish that the model infers that relation from text. For the selection-interaction comparison, hold candidate membership/order, query metrics and immutable payloads fixed where possible; an end-to-end input-derived relation generator then requires its own causal test.

Candidate admission, relational ranking, payload combination and writer admission remain distinct potential bottlenecks. A reranker cannot recover evidence absent from its pool, and a successful read cannot emit a target token excluded by the writer. The retained pilot's admission/ranking divergence makes this separation necessary; it does not establish that Flock has failed or that the proposed replacement is better.

## Encoding, frame and resource invariants

Content-equivalent compression programs may have different chunk boundaries and dictionary references. If the head is promised to depend only on content and its occurrences, canonicalization or a proven/evaluated encoding-invariance contract is needed. Implementation-dependent cache cost is allowed to vary; that does not by itself justify output changing. If program structure is intentionally part of model input, say so and test transfer across encodings rather than claiming content invariance.

Current signed actions are exact and invertible, but the existing snap's canonical tie order is not universally frame-equivariant. Preserve a declared canonical frame and tie law; do not infer that every aggregation is coordinate-independent merely because transport is exact. Keep cumulative actions or an established path convention when composing deferred transforms.

Every read needs a cost contract covering candidate/edge construction, index maintenance, virtual-proposal invalidation, dependent-node fetch, decompression, controller updates, payload construction and the writer. A short descriptor and a small controller state do not account for those terms automatically. Cold, warm and changed-model cases can legitimately have different costs.

Causal isolation applies to all derived inputs, including whole-document compressed summaries, dictionary statistics, cached graphs and relation generators. With model state fixed, altering a suffix after the causal cut or changing a held-out target must not alter the current input-side read. Retaining future bytes in an exact store does not authorize their summaries to enter the model early. Cache identities can change to preserve integrity without making their arbitrary hashes new semantic features. When a virtual proposal changes an endpoint root, refresh the writer's occupancy/admission view as well as read caches; otherwise unchanged-writer parity would be false.

Missing nodes, mismatched decoder schemas and stale learned summaries need typed repair/refusal. They are infrastructure states, not zero vectors, model negatives or fabricated evidence. Exact byte/state parity tests have meaningful equality requirements; language-quality decisions need the registered panel/noise and mechanism-own criteria, not an arbitrary synchronization threshold.

## Outcome-dependent roadmap for review

1. **Specify the shared descriptor/read boundary.** Freeze exact types, decoder/model/schema identities, occurrence order, causal cuts, cache dependencies and resource counters. Keep the current native writer. This resolves what is meant by reconstructing bytes versus token IDs versus geometric payloads.
2. **Qualify any exact representation claims independently.** A transport-only prototype can show byte/token parity; a packed-carrier or indexed-Flock implementation needs exact action/read/writer/proposal parity and its appropriate GPU construction qualification. If storage shrinks but latency does not improve, retain only the demonstrated benefit or keep the representation optional. No architectural mandate follows from the size arithmetic.
3. **Give the new head its own finite design card.** Choose which law is being tested, its state alphabet, graph construction, candidate admission, outer learning slots and readout. Do not require all packing/index optimizations first. Construction controls establish relational sensitivity, deterministic bounded execution, causal isolation and faithful payload transport. A flat or trivial consensus result calls for diagnosing the specified law, not closing geometric attention as a category.
4. **Only after FIT and the owner's applicable release, evaluate a trained-in arm.** Pin the data, target masks, writer and lineage where causal comparison permits. Report admitted targets, conditional ranks, separated exact loss terms, grounded reply behavior, fresh transfer and complete cost. Compare against the relevant existing geometric read, without a prohibited comparator. Inner-energy improvement alone is not KEEP evidence.
5. **Expand the instruction-native model only when evidence identifies a need.** A new reconstruction-opcode alphabet, full multi-orbit E8 representation, cross-device networking or physical oscillator hardware are separable extensions. Advance the one that addresses an observed bottleneck or capability gap; none is required to make the initial hypothesis fair.

The coherent proposal is therefore an exact reconstructible substrate plus a separately learned query operation. Source and mathematical evidence already support a practical connection to today's native writer. The anticipated attention advantage is a testable hypothesis about joint relations, not something proved or disproved by the codec alone. Exact reconstruction establishes that a query can always be implemented by decoding first; an efficient direct query still requires its own compositional summary or algorithm. The review does not establish end-to-end sublinear cost, superior attention, adoption or FIT.

## Cross-specialist challenges incorporated

- Mathematics supplied the query-factorization criterion, exact 16K Flock proof, occurrence/version requirements and snap-equivariance counterexample.
- Networking corrected the active HTML recorder to the golden-route implementation and separated exact application mappings from blanket standards conformance. It supplied the versioned immutable read protocol and independently tested address/radix primitives.
- ML separated existing learned carrier encoding from reversible compression, and required outer task learning, writer-admission accounting and trained-in evaluation.
- Physics required immutable evidence anchors, distinguished relational interaction from independent ranking, identified the order-three/G64 incompatibility, and proposed the exact C6 alternative without importing a 90-degree claim.
- Computer systems refined invalidation to actual frame/payload dependencies, observed that parent frames are inherited from the leftmost leaf, retained worst-case selected-subtree expansion, and rejected using the Flock-specific shortlist as a universal new-head prefilter.
- Parent adversarial questions additionally constrained present-shape cost claims, candidate admission, encoding invariance, causal comparison and complete cost accounting.
- The mathematical adversarial pass added eligible-count trace parity, the byte-storage/token-model-tree distinction, oriented C6 offsets, the narrow order-three claim, the action-stabilizer quotient and the explicit accepted-move energy-range bound. All are incorporated above.
- The ML adversarial pass scoped novelty to selection, separated inner energy/readout/outer loss, required actual learning-slot exposure, added suffix/target isolation across compressed metadata, distinguished synthetic supplied relations from input-derived relations, and required writer admission refresh on endpoint moves. All are incorporated above.

## Final resolution after adversarial discussion

The six perspectives support the architectural coherence of attention over exact reconstruction descriptions. They also agree on an implementable compatibility boundary with the present native carrier and writer. The exact Flock index gives a source-specific example of a query that can operate on a compact sufficient description; the geometric action tables give an exact way to move compact carrier states without reconstructing their full coordinate arrays. These are derived claims, not measured implementation speedups.

The review does not select C6 over G64, replace Flock, or declare the supplied attention proposal correct as an executable model. It identifies a fair next artifact: a finite design card that chooses the candidate domain, edge generator, state alphabet, oriented relation energy, anchoring, update budget, deterministic readout and actual outer learning slots. Its first useful controls must distinguish relevant relational selection from irrelevant sensitivity, prevent answer/future leakage, and retain exact transport and writer behavior. That is a bounded construction question connected to the user's hypothesis, rather than a demand that the whole system already be implemented before considering it.

The architectural assessment is complete at this scope: **the user's proposed connection is coherent and sufficiently concrete to investigate; current evidence establishes neither better attention nor a reason to rule it out.** No new source integration, FIT verdict, compute permission or adoption decision is implied. Packing, compressed transport and the new head remain separable choices, so a failure or lack of benefit in one does not manufacture a negative result for the others.
