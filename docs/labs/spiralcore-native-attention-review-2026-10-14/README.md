# SpiralCore as a native reconstruction and attention substrate

Constructive, adversarial research review for Casey · Codex / Sol · 14 October 2026

**The architecture the owner describes is coherent and connects directly to the current system.** A compact, exact reconstruction description can be the object that a read selects, routes, transforms and selectively expands. The shared decoder, geometric operator tables and retained object store are useful parts of that architecture. Their existence is not a reason to dismiss its compression or native-computation potential.

The [first assessment](../spiralcore-attention-assessment-2026-10-14/README.md) underdeveloped this central question. It verified the separate byte-transfer Showcase and geometric actions, but did not explain the full HTML's current geometric score/instrument/reconstruction path or work through attention on the resulting descriptions. This report supplies that missing analysis. It preserves the earlier measurements and distinguishes an executable codec, a source-derived exact implementation opportunity, and a new attention hypothesis.

**No replacement head is adopted or rejected here.** The strongest new conclusions are: exact selective reconstruction is directly demonstrable; the present geometric carrier admits an exact compact encoding; the existing Flock comparator admits a small sufficient index under explicit conditions; and a relational head can operate on the same descriptors while adding a different kind of interaction. Whether that new law improves the application remains unmeasured. No training, model/reference execution, CUDA qualification, paid resources, or owner release occurred in this review.

Source snapshot: `88c413599b805373b307035ee722ae8cdd1efea3`. The original six attachment identities remain in the first report. This review uses the exact same supplied HTML, SHA-256 `770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8`, and Showcase `core.js`, SHA-256 `7ca3dcfb552ca6ff3fc8696734265bcd9b993aa8b8e6242984060ace8b35bc22`. All proposed model changes below are **unimplemented / NOT_RUN** unless explicitly identified as existing source or retained prior evidence.

## 1. What the compatibility loop actually establishes

The full HTML joins several working concepts: typed geometric identities, reversible address presentation, graph paths with provenance, replayable E8 operator labels, telecom codewords, and a reconstruction program interpreted by a shared instrument. Its local symbolic addresses have a legitimate role even though the source deliberately does not emit them as externally routed IP addresses. Its geometric operator path is a sequence of actual transformations, not just a visual line between nodes.

The active file recorder deserves particular attention. A later override at HTML lines 10861–10871 selects `NATIVE_63_ROUTE_GOLDEN_PCM_BLOCK32_V1`. The implementation around 10680–10724 maps bytes to choices among 63 lawful geometric routes, packs those choices, renders a musical route performance, and has a separately initialized receiver recover choices from the waveform. Ordered FBS cells retain the decoded digits, and inverse radix reconstructs the bytes. The instrument/schema identity and public framing tell the receiver how to interpret the score. The older gzip/base63 importer remains a separately supported path; it is not the current upload recorder.

This is a substantive implementation of the owner's “sheet music and orchestra” idea. The source contains author-provided software-reference waveform-recovery receipts and separately scoped browser evidence. Our new executions verify its **current score/packing/ordered-FBS byte primitives**, not a fresh full waveform run or physical acoustic channel. The [physics review](reviews/physics.md) names that boundary precisely.

The separate Showcase implements a complementary saving: retain exact content-addressed ancestry at the receiver, send the root description and missing nodes, and reuse what is already there. The first report independently reproduced cold, warm and edited-object transfers. These two demos share an architectural pattern while saving different things: a performance score avoids retaining its expanded waveform; a shared-ancestry transfer avoids resending known content. The current score is not advertised by its own source as smaller than every original file. This distinction identifies the benefit rather than negating either demonstration.

The networking connection also has primary precedent. SCHC sends rule identifiers and residual values when both endpoints share a context, and can reconstruct fields that were not transmitted. That supports the central engineering pattern without implying SpiralCore implements the SCHC wire protocol. [RFC 8724, section 7](https://www.rfc-editor.org/rfc/rfc8724.html#section-7). The supplied Bell pair assignments were checked against ITU-T Q.320/Q.151; the documentation address conventions were checked against RFC 3849/RFC 5952. See the [networking review](reviews/network.md) for exact mappings, tests and remaining conformance boundaries. We do not make a blanket IEEE-conformance or novelty claim, and neither is a prerequisite for investigating the model architecture.

## 2. Are our tokens already compressed into vectors?

**They are encoded into learned geometric states, with exact identities retained separately. They are not currently encoded by the supplied reversible byte codec.** `discrete_encode` copies a learned codebook row. The carrier also retains token, occurrence, record and version identity and the codebook epoch. Two token IDs may share one geometric state; the retained pilot diagnosis documents substantial initialization collisions. The vector alone therefore cannot generally reconstruct the original token or source bytes. [Encoding](https://github.com/UOR-Foundation/uor-r4/blob/88c413599b805373b307035ee722ae8cdd1efea3/crates/uor-r4-discrete/src/discrete.cu#L268), [identity types](https://github.com/UOR-Foundation/uor-r4/blob/88c413599b805373b307035ee722ae8cdd1efea3/crates/uor-r4-discrete/src/lib.rs#L116), [retained diagnosis](../a2-pilot-diagnosis-2026-10-14/README.md).

The FBS binder retains ordered children and leaf identities. Flock's parent payload is a transported sum snapped to one orbit root. That payload is a useful candidate summary; the backing children preserve information it does not contain. Exact backing storage and learned summaries are compatible parts of one model.

There is nevertheless a substantial exact representation opportunity **inside our current geometry**. Every persistent 8D block is one of 16 roots in the admitted orbit. A fixed root-table label therefore needs four bits. At B4:

| Geometric payload only | Current coordinate representation | Possible exact root-label representation |
|---|---:|---:|
| One carried vector | 32 i32 coordinates = 128 bytes | Four 4-bit labels = 2 bytes |
| V4096 codebook | 524,288 bytes | 8,192 bytes |

These are derived payload sizes, not a measured checkpoint, memory, speed or energy reduction. Identity, version, indexes, decoder tables, alignment and temporary sums are excluded. A V4096 token ID itself has twelve information bits, so a sixteen-bit root tuple does not compress token identity. Packing preserves the *existing geometric state*; it does not make the learned map injective. [Orbit construction](https://github.com/UOR-Foundation/uor-r4/blob/88c413599b805373b307035ee722ae8cdd1efea3/crates/uor-r4-discrete/build.rs#L216), [codebook storage](https://github.com/UOR-Foundation/uor-r4/blob/88c413599b805373b307035ee722ae8cdd1efea3/crates/uor-r4-discrete/src/device.rs#L271).

Transport could operate directly on these labels through a `64 × 16` finite action table. L1 and coordinate-Hamming costs could use `16 × 16` tables derived from the actual roots. Bitwise Hamming between label numbers is **not** the geometric distance. Selected roots need expansion only for the existing small integer sums and canonical snap. A state label also cannot replace an action identity: distinct actions can agree on one frame and differ on another payload. This is an operation-preserving implementation proposal, not a new dense coefficient map or a measured speedup.

## 3. Can we compress before tokenization?

**Yes.** There are three compatible but materially different ways to do it.

1. **Transport/storage first:** losslessly encode the original bytes, send the descriptor and missing dependencies, reconstruct identical bytes, then use the identical pinned tokenizer. If `D(C(x), S) = x`, then `T(D(C(x), S)) = T(x)` for the same deterministic tokenizer/configuration. This can save storage or transfer without changing the model.
2. **Keep source compressed while retaining token interpretation:** tokenize once at ingestion, retain token/occurrence-to-byte-span bindings, and keep exact source objects compressed. Attention can read geometric descriptors; only selected quotations or missing content need byte reconstruction. An incremental tokenizer is possible only after its actual boundary/state contract is established. Arbitrary 32-byte chunks do not necessarily end at token or UTF-8 boundaries.
3. **Make reconstruction operations native model inputs:** let the model consume typed reference, literal, concatenation, repetition or action descriptions. This is the ambitious version of the proposal. It needs a learned interpretation and an explicit policy for different programs that reconstruct the same text. It can still feed the same native carrier/writer, but it is a new model representation, not an automatic substitution into the existing tokenizer.

There is also a direct mathematical bridge from arbitrary byte symbols to the admitted geometry: bind the sixteen roots to nibble labels, encode a byte as two roots, apply a known SpiralCore action, then invert the action and decode the nibbles. This carries bytes exactly in geometric symbols. The symbol code alone uses the same eight information bits per byte; real compression or shared-object coding comes first. The nibble labeling does not automatically make byte numerical distance or semantic relevance equal to E8 distance. The [mathematics review](reviews/math.md) gives the exact scope.

Established compressed-data algorithms show that querying a reconstruction description without fully expanding it is a valid computational strategy. Grammar-compressed random-access structures support character and substring access while retaining a compact grammar; this is relevant precedent, not a claim that our attention law inherits their complexity. [Bille et al., Random Access to Grammar-Compressed Strings](https://www2.compute.dtu.dk/~phbi/files/publications/2011ratgcsC.pdf).

## 4. What we tested ourselves in this review

The tests below exercise byte formats and data structures. No tokenization, model forward, attention reference, learning, oscillator-head simulation or GPU work ran. Node adapters execute the original supplied JavaScript primitives rather than rewriting them in Rust; the new selective byte reader is dependency-free Rust. Each evidence directory states its reproduction command, source hashes, measured scope and exclusions.

| Fresh execution | Observed result | What it establishes |
|---|---|---|
| Original HTML address/radix functions | 65,536 symbolic-prefix round trips; 10,000 address cases; four formatting examples; 291 radix round trips; ten malformed refusals; PASS | Exact behavior of the inspected presentation/radix primitives, not physical routing or whole-standard conformance |
| Current golden score/packing and source FBS functions | 353 byte round trips; 5,460 carry/frontier checks; 256 exact radix-bound checks; all 63 packed choices; seven malformed refusals; PASS | Current codec primitives preserve tested bytes through ordered FBS; waveform and head execution remain NOT_RUN |
| Rust selective reader over actual Showcase-generated records | 813 range cases across unique, repeated, UTF-8 and empty fixtures; integrity/order/refusal controls; PASS | Exact selective byte access to the retained ternary structure |

The primary agent reran both HTML-primitive scripts and the Rust byte-reader experiment; the result JSON/text matched the specialists' outputs byte-for-byte. This is a reproduction within the same lab, not an independent external-lab replication. The parent Rust build took 1.91 seconds and peaked at 145,457,152 bytes resident memory; the reader invocation took 0.14 seconds and peaked at 3,325,952 bytes, including fixture loading and checks. These are single local instrument observations, not performance comparisons. [Address/radix evidence](evidence/network/README.md), [current golden-codec evidence](evidence/physics/README.md), [Rust byte-DAG evidence](evidence/cs/README.md).

The selective-read result is particularly relevant to the proposed architecture. From a 32,768-byte object containing 1,532 stored nodes, a 23-byte requested range required two leaf occurrences and eight authenticated nodes, representing 384 serialized node-record bytes. An unrelated missing leaf did not block that range; it did block full reconstruction. A repeated 7,776-byte object occupied eight unique node records, 380 bytes before manifest/store overhead, and still reconstructed all 243 ordered leaf occurrences. The unique-byte fixture occupied 70,972 node-record bytes, so deduplication did not save storage there.

The Rust harness initially loads the serialized fixture into a resident map. The 384-byte count is the records accessed by the range operation, **not** total process I/O, resident memory or an observed network transfer. Source construction, loading and map overhead remain costs. The range reader verifies traversed BLAKE2b-96 nodes against the pinned structure and compares returned bytes with the fixture; it does not independently check the whole-object SHA-256 for a partial read. These limits do not diminish the demonstrated operation: requested content can be reconstructed without expanding unrelated content.

For current golden-codec packing, a 32-byte cell produces 43 six-bit choices. An isolated cell occupies 33 packed bytes; the actual 4,096-byte upload page concatenates 128 cells before packing and occupies 4,128 packed bytes, excluding headers and score JSON. This explains how the score describes a much larger generated performance while not claiming source-byte compression from that radix encoding alone.

## 5. A precise bridge to attention we already implement

The current Flock selector ranks each eligible node by:

```text
(frame L1 distance, frame coordinate-Hamming distance,
 last occurrence descending, node ID ascending)
```

Payload distance and BFS route length are not extra ranking terms in this implementation. Payload transport occurs after selection. Fix the block, level, actual BinderPlan/window, causal cut and complete model/proposal state. Within any one frame root, the first two keys are identical for every query. Retain only the best K eligible occurrences for each of the sixteen frame roots.

**This preserves the current global top K exactly.** Any discarded candidate already has K candidates of its own frame ahead of it under the complete order. It cannot enter the global first K. The union has at most `16K` records, and top-K truncation under the same order is mergeable across child sets. The [proof and cache contract](reviews/math.md) are source-derived and hand-checked; no packed/indexed model implementation or CUDA parity run is claimed. [Actual comparator](https://github.com/UOR-Foundation/uor-r4/blob/88c413599b805373b307035ee722ae8cdd1efea3/crates/uor-r4-discrete/src/discrete.cu#L165).

This is a concrete answer to whether attention can operate on compact descriptions. For the query family actually defined by Flock, a compact index is sufficient. The full exact history remains available behind it. Moreover, a parent's frame inherits its leftmost child's frame, so selection metadata need not wait for every parent's transported/snapped payload to be constructed. Selected payloads could be materialized lazily.

The conditions matter to correctness and cost: keep repeated occurrences distinct; include eligible interior nodes, not only frontier roots; bind the actual grouping and causal cut; preserve original node IDs, tie rules and total eligible counts for trace parity; and track hypothetical VectorBlock overlays as well as committed epochs. A proposal can change many occurrences before an epoch increments. Gate-only proposals may reuse selections; frame/payload dependencies determine other reuse. Writer occupancy/admission must refresh when a proposal changes endpoint roots. A selected high-level payload can still require most of the context.

The **byte-storage tree and model tree are different objects**. Showcase groups 32-byte atoms; the present BinderPlan groups token occurrences. Exact current-read parity should retain the canonical token BinderPlan and connect it to storage through span/occurrence mappings. If a new head instead uses the compression program's tree as model input, rechunking can change its behavior even when decoded content is identical. It then needs an explicit encoding-invariance claim or an intentionally program-sensitive representation and transfer test.

At the present C192 shape and K8, `16K` is at most 128 candidates; many higher levels already contain fewer. This does not establish a large speedup at today's dimensions. The writer's work, proposal repetitions and cold/changed-model index costs remain. It is an exact implementation opportunity, not proof that present Flock provides sufficient semantic attention.

## 6. Where the proposed attention head can add something new

The potentially valuable idea is **query-conditioned interaction among candidate descriptions during selection**. In present Flock, a candidate's rank comes from its own frame distance and fixed tie keys; Flock already combines selected payloads afterward. A relational head could change the rank itself because another candidate provides support, conflict, role binding or version information, even while the query-to-candidate metric stays fixed. That is a meaningful additional behavior to investigate.

The head would read descriptors and geometric states. Its learned “weights” could be finite shared relation choices, action/phase offsets and ternary controls. The current model already learns root choices, Route action choices or Flock gates rather than requiring conventional dense weight matrices. The compressed data is the input representation; the learned controls determine how it is read.

A concrete candidate boundary is:

```text
Exact compressed object store + separate causal occurrences
                  |
     Versioned geometric descriptors / compact root states
                  |
       Explicit bounded candidate and sparse relation graph
                  |
   Query/input-anchored finite relational controller (new law)
                  |
     Selected exact references + declared action identities
                  |
   Existing SpiralCore payload transport -> integer combine
                  |
             Existing shared vector writer
```

The immutable evidence must remain available while temporary controller states interact. Otherwise a controller could “resolve” a contradiction by changing both sides until they agree. Graph construction must have an explicit bounded algorithm and obtain its information from the actual causal input, with no answer-aware relation oracle or concealed all-pairs computation. The current Flock-specific `16K` index is **not** automatically sufficient for this new law; an older discarded candidate could matter through a relation. The first design can use all eligible nodes in the already bounded current context, or declare a different admission law honestly.

Two finite realizations are coherent:

- **Existing geometric states:** use current root/action labels and exact geometric cost tables on sparse typed edges. This is closely compatible with the carrier, but it is a finite geometric interaction law rather than a claim to reproduce continuous Kuramoto dynamics.
- **A separately typed six-state phase controller:** if signed coupling and 120-degree frustrated triads are central, twice cosine at the six phase differences is exactly `[2, 1, -1, -2, -1, 1]`. Ternary edge signs select sign/zero operations on these integer costs. SpiralCore still transports the unchanged payloads. The current 64-element action group cannot contain a nontrivial order-three cycle, so the phase variable cannot simply be renamed as one of those actions. Six phases also do not provide a 90-degree I/Q state.

These are alternatives for a design card, not silently adopted model changes. The group-order statement concerns a faithful cyclic phase action; it does not prohibit three distinct states or three-node frustration in a G64-based law. An energy that observes only an action's image of one anchor distinguishes only sixteen root images, so the design must choose root states or specify terms that distinguish full actions. Phase-offset edges need a consistent orientation, with inverse offset on reversal when representing the same edge.

An exact finite table does not inherit every continuous oscillator theorem. A strict integer-energy decrease under sequential updates prevents accepted cycles when the energy is bounded below; it does not prove a global optimum or useful language behavior. For M ternary C6 edges, edge energy range is at most 4M, plus unary ranges; this bounds accepted unit drops, not rejected trials, graph construction or writer work. A declared move/sweep bound makes runtime finite even before local settling. The learned controls must improve the **outer task objective through the actual writer**. The energy law, deterministic reference/action readout and task objective must each be specified; the real learning schedule must demonstrably attempt the new parameter slots. Lower inner energy or stronger phase agreement is not itself evidence of relevance or truth.

Related primary work establishes that learned oscillator attention can be constructed under specific dynamics and readouts. Its continuous projections/couplings and normalization differ from the proposed discrete UOR law, so it is conceptual support rather than a ready-compatible implementation or performance result for us. [Pasqualetti and Guo, Attention by Synchronization in Coupled Oscillator Networks](https://arxiv.org/abs/2606.12059).

The application-facing question is specific: can interactions recover evidence or relations that independent frame ranking misses, while preserving exact references and producing better grounded output at acceptable total cost? Current evidence does not show Flock has failed this task; the registered retained pilot used Route. It also does not show the new head is better. A fair assessment keeps both possibilities open.

## 7. Expert discussion and adversarial resolution

Six native Codex specialist assignments covered mathematics, networking/telecom, machine learning, physics, computer science and systems integration. The first three exchanged source findings; physics and computer science challenged the integrator's candidate; mathematics and ML then reviewed the integrated proposal. These are distinct review roles using the available agents, not six credentialed humans, different model providers, or external-lab endorsements. Advisory harness dispatch receipts do not establish a DeepSeek invocation.

The [integrated candidate](reviews/integrator.md) incorporates both [mathematical](reviews/math-cross-review.md) and [ML](reviews/ml-cross-review.md) adversarial passes. They found substantive interface requirements rather than a reason to rule out the architecture:

| Challenge | Resolution in this assessment |
|---|---|
| Does compression alone preserve every semantic query? | Exact backing data preserves access; efficient direct reads need a query-sufficient index or a deliberately new learned law. |
| Can the existing Flock shortlist restrict any replacement? | No. Its proof is comparator-specific; a relational head needs its own explicit causal pool. |
| Does an identical reconstructed file imply identical attention? | Only with a preserved canonical model tree or an established encoding-invariance contract. |
| Does an exact output imply complete parity? | Preserve trace counts, node/occurrence identities, virtual overlays and writer occupancy as well as output. |
| Is the new head already doing more than Flock? | Its proposed novelty is relation-dependent selection, not the existence of payload combination. This needs a construction control and a specified readout. |
| Does a smaller synchronization energy establish useful attention? | No. Keep energy, actual selection and outer writer/task loss separate; preserve relevant opposed evidence. |
| Could compressed caches leak future data? | Pin causal summaries, dictionary/index state and relation generation; vary the future suffix/target while holding the prefix fixed to test isolation. |
| Is literal 120-degree phase action already present? | Not as a faithful cyclic subgroup of G64. A separately typed finite phase controller is a coherent option, with its own law and limits. |

All six reviews support the architectural possibility. Their agreement is an expert-role synthesis within one agent system, not empirical validation of a head. The remaining design choices are stated in the roadmap; this assessment is complete and creates no unwritten waiting review or FIT gate for another lab.

The conceptual attachments help organize this work. *Lawful but Unselected* distinguishes a lawful configuration from a reachable, functional or realized one: we should neither reject a coherent arrangement for not yet being built nor declare it useful because it is lawful. SIOPL supplies a useful provenance vocabulary for actual object IDs, selection decisions, transformations, versions and repair events. That observability can expose mistakes; it does not make phase coherence a truth oracle.

## 8. What would change, and an evidence-dependent roadmap

There is no demonstrated need to replace SpiralCore, the shared writer, the current learner or the active completion configuration as a result of this review. The changes below are concrete options tied to distinct questions. Exact packing/index work and the new relational head can be reviewed separately; one is not a compulsory detour before the other.

| Step | Concrete deliverable | Evidence that would justify its next step |
|---|---|---|
| Specify the common read boundary | Typed exact object, occurrence, tokenizer/span binding, geometric state, action, model/proposal identity and missing-state behavior | The complete reconstruction/read/write diagram has explicit meanings and no lost identities |
| Build the exact representation option, if pursued | Root-label carrier and/or versioned sufficient Flock index, same writer and same read law | Exact GPU action/read/writer/proposal parity under FIT; then measured payload, cache and full-path costs |
| Specify the new head independently | Chosen finite state alphabet, causal candidate pool, sparse edge generation, input anchors, actual readout, learning slots and resource bounds | A design review can identify the intended new interaction and its failure cases; no claim relies on a PAS threshold or semantic labels assigned by geometry alone |
| Qualify its construction | Causal isolation, relation sensitivity, deterministic ties, bounded updates, immutable evidence, transport and schema/missing-state controls | The head demonstrably implements its declared law; failures localize instrument, fixture or implementation defects |
| Evaluate a trained-in configuration only after the applicable FIT and owner release | Frozen data/masks/writer lineage where appropriate; admitted-target, conditional-rank, separated-loss, grounding/fresh-transfer and full-cost reports | The outcome distinguishes admission, interaction, learning and output failures rather than rejecting geometry as a category |
| Extend only where evidence warrants | Instruction-native input alphabet, more E8 orbits, distributed fetch or physical oscillator hardware | The extension addresses a measured capability or resource limit rather than being assumed necessary for the core idea |

An appropriate relational construction control holds the query and query-to-candidate metrics fixed, changes another candidate or relation, and checks whether the declared interaction changes selection. It must also show how those relations are obtained without hidden answer hints. Matched relation-on/off controls and suffix/target isolation help distinguish genuine interaction from a leaky graph builder. A fixture proves that behavior only; useful language requires learning and fresh task evidence. Exact parity is a meaningful bar for claims of unchanged computation. Quality claims need the standing panel/noise and mechanism-own interpretation, not an arbitrary closeness-to-consensus number.

Total cost must include initial construction, shared-state seeding, candidate/edge generation, proposal invalidation, payload expansion, controller updates, writer work and output reconstruction. Warm lookup, cold loading and changed-model operation should be measured separately. A storage reduction remains a valid result even if latency does not improve; a faster read is not useful if it removes necessary evidence. Neither outcome automatically closes the larger mechanism.

The resulting assessment is more constructive than the first report: **there is an exact, native bridge from reconstruction instructions to today's geometry and writer, and there is a distinct relational-attention hypothesis worth specifying on that bridge.** The byte/data-structure evidence and conditional proofs support that statement now. Predictive superiority, end-to-end savings, arbitrary long-context behavior and a chosen replacement head remain unestablished. Training stays held, and the current completion work and other labs' source remain unchanged.
