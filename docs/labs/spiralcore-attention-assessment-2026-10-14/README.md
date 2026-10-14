# SpiralCore, Cantor–Abraxas and the proposed attention head

Research assessment for Casey · Codex / Sol · 14 October 2026

**Owner-requested follow-up:** the [native reconstruction/attention review](../spiralcore-native-attention-review-2026-10-14/README.md) develops the compatibility-loop question that this first report underemphasized: attention over exact reconstruction descriptions, compression before tokenization, the full HTML's current geometric score path, and six specialist perspectives with fresh byte/data-structure evidence. This report's measurements remain preserved at their original scope.

**Disposition: retain the demonstrated transport and finite geometry; keep the new attention mechanism as an unqualified proposal. No model replacement is justified by the supplied evidence.** This is a completed source, mathematics and byte-codec review, not a model experiment or a TEST FITNESS verdict. Training remains subject to the existing owner-personal release gates. No model, optimizer, initialization, registered configuration or acceptance bar was changed.

The owner asked whether the six supplied artifacts form a coherent mechanism, how they connect to UOR-R4, whether their inexpensive transport and attention can work, and what adoption would require. The owner clarified that the attention note should be evaluated **as a proposal**. Its lack of an executable head is therefore an evidence boundary, not a reason to refuse the review.

Three parallel Codex reviewers examined mathematics/attention, transport/codec, and current implementation, followed by a source-reconciled synthesis. These are independent review assignments, not independent model replications or claims that Claude, Kimi or DeepSeek performed this review. Harness role-dispatch receipts record coordination roles; they do not establish that an external provider was invoked. The research-architect and project-audit procedures guided the assessment; current owner decisions supersede their older floating-point permissions.

## 1. Assessment

**The inexpensive transport is real within a specific, useful scope.** The showcase stores ordered byte ancestry at the receiving endpoint and sends short references plus missing nodes. We reproduced exact reconstruction, strong warm-cache reuse and small same-position updates. The source explicitly accounts for retained receiver state. A 254-byte root packet names a large stored object; it does not independently contain all that object's information.

**SpiralCore's finite geometric transport is also real, and UOR already uses it.** Its current eight-coordinate action is a signed permutation, executed as coordinate gathers and sign choices with finite action tables. Under a common frame change it preserves the relevant geometry exactly. This is a valid inexpensive primitive. It does not establish that every summary, route selection, admission decision or learned representation built around it preserves all information needed for language.

**The attention idea is plausible as a research direction, but the supplied recipe is incomplete and several advertised guarantees are incorrect.** Coupled states can implement relational computation. Nothing in the note establishes that phase alignment detects truth, that negative coupling generally creates 120-degree triads, that geometric orthogonality makes learned semantic roles independent, or that a logarithmic root frontier makes the full retained history and learning graph logarithmic. These are substantive gaps with exact counterexamples, not objections to a tolerance being off by 0.001.

**It is not established to be more correct or better than our current read.** Our existing Flock read has a concrete causal, integer implementation and finite qualification. Its language advantage is also unestablished: the registered pilot/completion arm uses Route, not Flock. A comparison between a defined implementation and an incomplete proposal cannot establish predictive superiority in either direction.

The constructive conclusion is to keep the layers distinct, reuse the exact carrier and writer, and specify a bounded geometric relational read only if its intended behavior answers an unresolved application question. The supplied evidence does not require a wholesale rewrite, adoption of all 240 roots, or another training run.

## 2. Four layers that can coexist

| Layer | Object and guarantee | What it does not establish |
|---|---|---|
| SpiralCore geometric action | A labelled finite action moves an 8D payload between defined frames; inverse and composition are explicit. | Semantic relevance, correct token selection, or lossless aggregation of many payloads. |
| Ternary FBS binder | Ordered children and base-three carries produce a small frontier over a retained tree. | That the summary vector contains every child's information, or that all retained storage is logarithmic. |
| Cantor–Abraxas byte transport | Content-addressed ancestry and optional shared dictionaries let the receiver reconstruct exact bytes when required state is present. | That a hash is an E8 carrier, or that the receiver can answer arbitrary semantic queries from a root without reading stored content. |
| Learned attention/read and provenance | A read chooses and combines relevant content; a trace records actual sources, actions and decisions. | Truth from coherence alone, or a causal explanation from a generated narrative. |

The word “ternary” occurs in several different roles: three-child branching, a base-three counter, three-valued gates, and possible coordinate alphabets. They are compatible concepts, but a theorem about one does not prove a property of the others. Likewise, the owner's “FSB” wording must resolve to the actual binder/store/action being discussed; the supplied implementations generally use **FBS**.

An integration can carry both a geometric payload and an exact content reference:

```text
token / occurrence / version identity
             │
             ├── integer E8 carrier ── SpiralCore action ── read ── shared writer
             │                               ▲
             └── ordered retained FBS children / exact backing records
                                             │
                     optional byte serialization + shared-store synchronization
```

This arrangement preserves identity and retrieval options without claiming that the hash and geometric vector are interchangeable.

## 3. What UOR-R4 actually uses now

The main source audit is pinned to `23bd6128011e6903c62b9c966154e2f9da33a792`. Subsequent refreshes inspected main at `e1491635b`, including Antigravity's constructor PR #2473 and formatting follow-up #2474, and at `9048df01b` after its CUDA receipt PR #2477 merged during report delivery. Those additions are described separately below. The frozen completion runtime remains the qualified D5 source `33564b75e`; the relevant original runtime files and completion configurations are unchanged by those constructor additions.

| Stage | Current executed/implemented behavior | Source |
|---|---|---|
| Input | SHA-bound UORT token IDs, tokenizer and response mask; each target receives a preceding causal prefix with document/BOS clipping. | [Corpus preparation](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/corpus.rs#L257) |
| Shared carrier | One integer codebook of shape `[vocabulary, blocks, 8]` serves encoding, carried payloads, output candidates and targets. Occurrence/record/version identity remains separate. | [Identity types](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/lib.rs#L116), [encode](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/discrete.cu#L268) |
| Geometry | Six Clifford generators yield 15 bivector operators and a 64-element signed-action group. The discrete carrier uses one 16-root orbit of the scaled E8 anchor `[2,2,0,0,0,0,0,0]`. | [Operators](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-core/src/spiralcore_operator.rs#L358), [orbit construction](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/build.rs#L216) |
| FBS binder | Retains every leaf and three ordered child references per parent. Base-three carries organize the frontier; they do not delete the backing history. | [BinderPlan](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/metadata.rs#L26) |
| Route, selected by the registered arm | The final token supplies a query. Learned frame actions and fixed branch/port roots select one valid prefix leaf per block, possibly the query itself, then an accumulated SpiralCore action transports its payload. For a fixed query, frames and valid length, selected positions do not depend on earlier leaf contents. | [route_vector](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/discrete.cu#L41) |
| Flock, implemented alternative | Selects earlier atoms/binders by another block's frame L1/Hamming and recency, transports the separate payload into the query frame, sums and snaps to an orbit root, then combines levels through ternary gates. | [flock_vector and dispatch](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/discrete.cu#L95) |
| Shared writer | First/last root indices define 256 possible product cells. The writer admits nearest occupied cells, then ranks token identities inside them by full-vector L1 with deterministic ties. Target identity does not influence admission. | [Writer](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/discrete.cu#L208) |
| Learning | Exact token/address hinge objective; categorical proposals try current state plus 15 geometric moves, or three gate trits. Full read/writer evaluation is repeated; identity wins ties and apply requires strict improvement and matching epoch. | [Loss/proposals](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/discrete.cu#L248), [schedule](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/training.rs#L224) |

The registered shape is **V=4,096, four 8D blocks, width 32, context 192, batch 64, eight writer cells, Route**. Context length and vector width are separate quantities. The implementation bounds context at 243; the proposal's 100,000-token horizon is not available by changing a configuration flag. A full coordinate pass attempts 16,384 vector coordinates and 20 Route coordinates, ending at completed cursor 16,404. It is not a full corpus epoch. [Registered configuration](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/docs/labs/rung0-completion-release-2026-10-14/seed1-config.json).

There is **no Cantor–Abraxas/UMT/Golden Route/SIOPL runtime imported into this discrete model**. Its FBS is a retained ordered binder abstraction, not the complete marked external FBS object or the showcase's network store. The supplied JavaScript is not a model dependency. Historical floating paths elsewhere in the repository remain separate evidence; a scoped discrete-path audit is not a claim that every repository file contains no floating arithmetic.

The “same writer” is implemented within this prototype: the read and output candidates share the same codebook. Whole-application integration remains unfinished. A repository Rust/Cargo search found no dependency/import of `uor-r4-discrete` outside its own crate. [`Device::carry`](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/crates/uor-r4-discrete/src/device.rs#L698) is a bridge API, not evidence that the legacy compiler, exact memory store and answer emitter have all migrated to it.

### Current qualification and the fresh constructor change

The [completion release record](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/docs/labs/rung0-completion-release-2026-10-14/README.md) records Claude's executable `d4fbe26c…`, built from unchanged D5 on `vast-55939635`, one RTX 5090, nvcc 12.8.93, Rust 1.97.1 and `compute_120`. Its 25 finite controls passed; both plan checks returned `PLAN_METADATA_VALID` with owner release still required. These are inspected existing receipts, not a new pod visit or rerun by this assessment. Older overview text saying that this build is still unqualified is stale relative to that record.

During this audit, [#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473) added a standalone `prime-zeta-qualify` binary and `prime_zeta.rs`; [#2474](https://github.com/UOR-Foundation/uor-r4/pull/2474) formatted it. The constructor reports 4,096 distinct full vectors and 256 occupied cells at V4096/B4. Its construction uses explicit cell quotas and uniqueness assignment after geometric scoring, shared with the null controls; collision freedom is not established as an effect of prime phases alone. Primary seed labels 0/1/2 produce identical tables, demonstrating determinism rather than independent initializations.

The later [#2477 record and receipts](https://github.com/UOR-Foundation/uor-r4/blob/9048df01b/docs/labs/prime-zeta-construction-2026-10-14/README.md) now record **17 base CUDA controls plus six prime-zeta controls passing**, with `applied_updates: 0` and `corpus_rows_consumed: 0`, on executable `953f8f08…` from source `912a8a069`. All six retained receipt/log hashes match their manifest. One operational qualification matters: the job wrapper's final checksum command was given directory arguments and exited 1 after the passing control JSON; the log therefore does not support a claim that the entire wrapper exited successfully. This does not turn the recorded control results into a model negative. The candidate remains **separate from the unchanged frozen D5 initializer, with finite CUDA construction evidence but no trained benefit or runtime adoption**. This audit inspected those receipts without rerunning the model or touching that pod.

The [older mechanism brief](https://github.com/UOR-Foundation/uor-r4/blob/e1491635b/docs/mechanisms/prime-zeta-complex-routing.md#L46) still claims replacement of the active xorshift initialization and cites nonexistent `metadata.rs:344–367`. The original file ends at 210 and `discrete.cu:257–267` remains xorshift. The new constructor does not make that old runtime-replacement claim correct. This report records the discrepancy without changing Antigravity's owned implementation or research line.

## 4. Exact geometry: what is preserved

Write a signed action as `T(x)[i] = s[i] x[pi(i)]`, with `pi` a permutation and each sign ±1. Then:

```text
<T(x), T(y)> = <x, y>
||T(x)-T(y)||₂ = ||x-y||₂
||T(x)-T(y)||₁ = ||x-y||₁
coordinate_Hamming(T(x), T(y)) = coordinate_Hamming(x, y)
```

The inverse uses the inverse permutation and corresponding signs. These are algebraic consequences, not benchmark claims. Execution requires eight gathers/sign choices, not a dense multiply-accumulate. It is still a mathematical linear action; the important implementation property is its exact finite structure and cost. Applying **different** actions to two different vectors need not preserve their raw pairwise distance. Comparing in a common frame is essential.

The saved HTML's E8 construction has 112 integer roots and 128 half-integer roots. Under its particular 64-element action they form fifteen disjoint 16-root orbits: seven integer and eight half-integer. Its source explicitly distinguishes **15 operators** from **15 orbits** and refuses inter-orbit routing. Neither set automatically defines 15 semantic attention heads. The current model admits one orbit; using all 240 roots would require typed orbit identity and a designed cross-orbit policy, if cross-orbit movement is wanted. These source facts agree with the earlier [hosted-source audit](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/docs/labs/spiralcore-live-review-2026-10-09/README.md), which remains historical evidence at its own source identity. (A6: lines 782, 848, 996 and routing implementation around 2920–2956.)

The two orthogonal decagonal rings are the 20-root `H₂ ⊕ H₂` subset removed from H4's 120 vertices to form the 100-vertex Grand Antiprism. They are not two arbitrary partitions of the 100 remaining vertices. Orthogonal carrier planes support exact geometric separation, but semantic independence depends on the learned variables and downstream operators. The geometric construction is described in [Dechant's primary paper, section 7](https://arxiv.org/abs/2103.07817). (A6: 850, 3407–3432.)

The validation paper's 36-degree nearest-neighbor angle and recovery under a perturbation **strictly less than 18 degrees** have a legitimate nearest-root interpretation. At the boundary, a tie can occur. This does not prove that an arbitrary continuous state is preserved by snapping, or that the discarded distinction is irrelevant to language. Likewise, an icosian-to-E8 correspondence needs fixed basis/glue/Galois maps and an inverse witness; matching counts or a projected picture is insufficient. The saved HTML itself rejects a rank-four shadow as full E8.

Modulo-256 coding is compatible with exact finite state labels. Unrestricted coordinate reduction modulo 256 is not an isometry of the integer lattice: `x` and `x+256` become identical. A byte can name a root or action exactly when its table, range and version are bound. The numerical distance between byte labels is not automatically the geometry's distance.

## 5. What the showcase actually demonstrates

The supplied code splits bytes into padded 32-byte atoms, hashes domain-separated records with BLAKE2b-96, folds ordered triples, and retains all required ancestry. A receiver reconstructs from its own node map, validates metadata/padding and checks SHA-256. Missing records are staged before committing a successful reconstruction. Its root packet is `12 + 50 + 12R` bytes; separate metadata, requests, seed records and receipt frames are also counted. The `psi=1.618034` float header is a constant metadata field, not an oscillator. The source says it implements no H4 forwarding, Kuramoto/CHORDLOCK dynamics or physical network measurement. (A2: `core.js:85–102,166–249,313–321`; `transport.js:7–24,67–134`; `README.md:59–110`.)

All supplied suites passed: **6 core, 4 v1.1 and 10 showcase test groups**. An independent direct-codec audit also passed:

- exact base-three frontier/carry identities for N=1 through 1,000,000;
- 8,192 Tier-3 frame round trips and malformed-frame refusals;
- exact radix-capacity inequalities;
- a repeated-node topology counterexample;
- cold, warm, overwrite and prefix-insertion reconstruction on a separate byte fixture.

These are byte/tree tests, not model compute. Node was `v26.4.0`; the independent audit took **1.59 s wall** and **131,923,968 bytes maximum RSS**. Actual browser IndexedDB/UI, cross-machine networking, NIC throughput and physical oscillator behavior were not exercised. The embedded worker source was exercised under the supplied Node adapter. Optional Python launcher/build files were inspected but not run or adopted.

### Measured application-level exchanges

| Workload and case | Root packet | Complete counted exchange | Newly stored nodes | Serial sync rounds |
|---|---:|---:|---:|---:|
| Supplied 1 MiB, cold | 254 B | **62,579 B** | 2,245 | 1 |
| Supplied 1 MiB, warm repeat | 254 B | **438 B** | 0 | 0 |
| Supplied 1 MiB, one-byte overwrite | 254 B | **1,322 B** | 10 | 10 |
| Independent 128 KiB, cold | 182 B | **230,705 B** | 6,139 | 1 |
| Independent 128 KiB, warm repeat | 182 B | **367 B** | 0 | 0 |
| Independent 128 KiB, one-byte overwrite | 182 B | **1,083 B** | 8 | 8 |
| Independent 128 KiB + one-byte prefix insertion | 194 B | **230,768 B** | 6,140 | 1 |

Every reconstruction matched. Full counted exchange means actual serialized root/metadata/request/seed/receipt buffers across the local endpoint boundary. It excludes physical networking headers, browser message envelopes, comparison work and downloading the reconstructed file to a user. It is not a wire or production latency benchmark. See [independent receipts](evidence/independent-results.json) and [supplied-suite receipts](evidence/showcase_test_results.json).

The supplied 1 MiB fixture repeats only 512 distinct atoms. The independent fixture consists of 4,096 distinct SHA-256-derived atoms from fixed labels; it is reproducible novel content, not a theorem of randomness. The two update cases each start from a clone of the original receiver, preventing one from benefiting from the other. Fixed 32-byte alignment explains why a prefix insertion changes almost all later leaf IDs while a same-position overwrite has a small ancestry delta.

After the independent cold transfer, the retained serialized record store is **284,427 B** for a 131,072-byte input; JS/Map/browser overhead is additional. Direct deflate of that input is 131,118 B. The protocol therefore has a real novelty cost as well as a reuse benefit. Eight or ten serial repair rounds can matter on a real link even when byte traffic is small. No measured network latency advantage follows from these local byte counts.

The validation PDF's reported cold traffic and Python-VM timing are different evidence. Its original complete workload, runtime and logs were not supplied, so this report does not claim to reproduce its 23,618-byte cold result or its microsecond transit measurements. Nor does a different showcase result disprove those scoped measurements. The showcase is more explicit about full exchange accounting than a root-only number.

### The exact frontier theorem and its boundary

For the specified full ternary occurrence forest, let `s₃(N)` be the sum of the base-three digits of N. At each level the frontier contains zero, one or two roots. Insertion N causes `v₃(N)` carries, each replacing three roots by one:

```text
R(N) = s₃(N) ≤ 2(floor(log₃ N) + 1), for N > 0
R(N) - R(N-1) = 1 - 2v₃(N)
C(N) = (N - s₃(N))/2
retained node occurrences = N + C(N) = (3N - s₃(N))/2
```

The first three equalities follow by base-three induction/counting. The last is linear retained ancestry. At N=32,768 there are 16 frontier roots, 16,376 internal nodes and 49,144 node occurrences before deduplication. The root packet is consequently 254 B, while novel retained records can be much larger. Reconstructing N output bytes also requires at least N bytes of output work.

The paper's `β₀=s₃(N)` applies to the **unfolded occurrence forest**, not automatically the deduplicated store graph. Six identical atoms produce two frontier references to the same parent. The unique stored graph has one parent, one leaf and one connected component, while `s₃(6)=2`. This distinction was checked directly; it does not impair reconstruction.

A logarithmic-depth tree can still contain linearly many nodes. Backpropagation, replay or arbitrary historical queries do not become logarithmic merely because the top frontier is small. Exact information must live in backing children, a restricted sufficient statistic, or another accounted representation. A bounded summary of unrestricted sequences necessarily merges distinct histories; a later query about a differing position distinguishes them. This is a counting argument, independent of any preference for transformers.

### Compression and paper corrections

The 19-bit UMT address can name `120 × 4,096 = 491,520` agreed entries. An exact lookup is coherent if the dictionary and mapping are shared and the entries are uniquely assigned. It is not an injective code for arbitrary 4,096D continuous states. Storing 491,520 uncompressed 8,192-byte entries would require **4,026,531,840 bytes (3.75 GiB)**. A generative dictionary can trade storage for a construction rule; its rule and decode work still count. This agrees with source coding's dependence on the source and receiver information, rather than contradicting it. [Shannon's original paper](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf).

Tier-3 encoding satisfies `63^42 < 2^256 < 63^43`. Forty-three six-bit slots occupy 258 bits and a frame occupies 33 bytes for at most 32 payload bytes. It is a reversible alphabet/framing transform with **3.125% full-frame expansion**, not the entropy compressor. Deflate and shared-state reuse supply the measured reductions. The printed radix exponents are correct; an initial reviewer suspicion was withdrawn after exact verification.

Three corrections affect the validation PDF's claims:

1. **Page 3, equation (6):** `D_min ≥ 1 − 2^(−2R/d)` is inconsistent as the claimed minimum-distortion law: the right side grows with bit rate, although finer finite quantization of a bounded sphere can drive distortion toward zero. Its ≈0.641% value is also confused with the complementary ≈99.359% in the prose. The familiar Gaussian squared-error analogue uses `2^(−2R/d)` for normalized residual distortion, but the PDF specifies a uniform-sphere source; an exact spherical bound requires its own derivation. This report does not invent that missing theorem. The equation was visually checked in the PDF.
2. **Page 8, Table 5:** the row admitting 44 frames/1,408 decoded bytes in a 1,472-byte budget does not subtract the declared 62-byte header. Including that header allows `floor((1472−62)/33)=42` full frames and 1,344 decoded bytes, before any further framing.
3. **UDP scope:** 65,507 bytes is the maximum ordinary IPv4 UDP payload, not a guarantee of one unfragmented packet on a 1,500-byte path. Fragmentation and effective path MTU must be accounted for. [RFC 8085](https://www.rfc-editor.org/rfc/rfc8085).

These corrections do not invalidate the exact carry law or demonstrated shared-store transport. Hash collision resistance and final SHA checks remain engineering assumptions/checks, not a proof that a finite digest injectively represents arbitrary trees.

## 6. Attention proposal: what is coherent and what is missing

The strongest interpretation is a relational read with coupled states: related inputs influence one another, incompatible constraints may settle into differentiated configurations, and a bounded readout returns a carrier to the same writer. That is a legitimate family of mechanisms. The proposal does not yet define which object is a node, how tokens enter it, how values survive the interaction, or what finite computation produces the output.

At minimum it needs: node/state domains; input and payload maps; legal edge set and parameter sharing; exact update schedule; causal prefix/version rules; readout into the existing carrier; a compliant learning move/objective; and retained-state/cost bounds. Tokens, vocabulary types, H3 chambers, E8 roots, orbit labels and FBS summaries are different domains. Choosing one changes both complexity and representation.

### Potential descent is narrower than semantic correctness

For fixed symmetric couplings on an undirected graph, with common natural frequency removed, a conventional model is:

```text
dθᵢ/dt = Σⱼ Kᵢⱼ sin(θⱼ − θᵢ)
V = −½ Σᵢⱼ Kᵢⱼ cos(θᵢ − θⱼ)
dV/dt = −Σᵢ (dθᵢ/dt)² ≤ 0
```

The factor ½ avoids double-counting edges. A changed factor can be absorbed into time units; symmetry and the fixed-input/frequency assumptions cannot. For directed K the gradient sees its symmetrized part, and heterogeneous frequencies or forcing require a different argument. Energy descent does not prove global optimality, semantic truth or an efficient finite settling time.

An autoregressive implementation can relax a graph over the current prefix without future leakage. It must define when earlier states change and how training targets are versioned. Merely importing a symmetric physical metaphor does not establish that contract.

### Counterexamples to the proposed guarantees

| Claim in A1 | Exact issue | Constructive interpretation |
|---|---|---|
| Positive coupling implies zero-offset agreement. | With unequal natural frequencies, a positive-coupled pair need not lock; if it locks, its phase offset need not be zero. | Specify frequencies, forcing and the intended synchronization regime. |
| Negative coupling sends contradiction into 120-degree triads. | A negative-coupled pair minimizes its edge energy at 180 degrees. A triangle of three equal negative couplings can use 0/120/240 degrees because it is frustrated. No supplied map proves that contradictory content receives those couplings. | Treat graph-specific conflict states as a designed behavior to test. |
| PAS detects hallucination; PAS≥0.82 ensures lawful reasoning. | Any collection assigned identical phases has alignment 1, including false statements. The lawful equal-negative triangle has zero resultant and hence zero global alignment. | Specify grounded constraints and a relation/cluster-dependent diagnostic before giving alignment semantic authority. |
| Orthogonal rings give zero semantic interference. | The same bit can be encoded in two orthogonal axes; the carriers are orthogonal while the represented variables are perfectly dependent. A downstream joint operator can couple them. | Make channel separation an operator/readout contract; test task interference causally. |
| High kurtosis forces one clean semantic role. | Permuting vocabulary labels preserves moments while changing semantic assignments; a rare-token spike can be highly kurtotic without a coherent concept. | Use moments as a diagnostic, supported by interventions and task behavior. |
| Fifteen root subnets enforce grammar versus tone. | Root coordinates and orbit counts contain no supplied token-to-syntax/meaning theorem. | Treat role assignment as a hypothesis or explicit supervision, with matched controls. |
| Folding proves logarithmic training history and prevents OOM. | Small frontier and tree depth do not bound retained children, payloads or credit dependencies. Any finite hardware can exhaust a retained store. | Specify bounded storage, eviction, backing access and the task-sufficiency of summaries. |

PAS also needs a defined mean: circular mean is appropriate for phases; an arithmetic mean mishandles wraparound. The circular mean is undefined when the resultant is zero, requiring explicit behavior. These issues concern whether the instrument measures the intended phenomenon. Moving the threshold from 0.82 to another arbitrary number would not repair them.

The note's opening comparison also confuses **KV-cache storage**, ordinarily linear in sequence length for fixed width/layers, with quadratic all-pairs attention score/work. One cached key and value per token gives the linear count directly. This does not make long contexts free; it identifies the correct baseline quantity. [PagedAttention's primary analysis](https://arxiv.org/abs/2309.06180).

### Cost and hidden arithmetic

Calling parameters a coupling tensor neither proves nor disproves efficient geometry. A sparse fixed graph can evaluate exact local table transitions without dense matrix multiplication. An arbitrary all-pairs graph has quadratic edges; direct T-step simulation costs proportional to T times evaluated edges. Structured coupling might admit a cheaper algorithm, but its structure must be stated. A small fixed chamber graph bounds computation while moving the question to how many token distinctions its aggregated state retains.

The supplied HTML's oscillator demonstration uses floating sine/cosine and RK4 software loops. The showcase has no oscillator. No physical head, oscillator measurement setup, encoding/readout energy or end-to-end language benchmark is supplied. A physical substrate may eventually help; it does not erase input/output, settling, calibration and storage costs. The finite “−149.9 dB” assertion has no supplied experimental setup and is not the same statement as exact algebraic zero.

The proposal explicitly suggests AdamW/Muon and standard next-token cross-entropy. The conventional logits-based form introduces the normalized-exponential law prohibited by D23; the note specifies no alternative categorical law. Continuous downstream phases, gradients and those optimizers also conflict with D32. The [current zeta-plane exception](https://github.com/UOR-Foundation/uor-r4/blob/e1491635b/docs/integration/DECISIONS.md#L1593) permits only declared prime/zeta complex maps and explicitly keeps downstream routing, reads, writer and counterfactual learning integer/exact. It does not admit the proposed continuous recipe.

A discrete finite-phase/relational construction could be designed, using table actions and a well-founded integer energy or bounded transition rule. That would be a **new specified candidate inspired by the proposal**. It would need its own termination/tie/cycle analysis; continuous Kuramoto convergence results cannot simply be inherited after discretization.

### Related primary research, with limits

The [ROTATE paper](https://arxiv.org/abs/2604.06005) studies post-hoc rotations of existing transformer MLP weights in vocabulary space, with empirical interpretability and fidelity analysis. It supports investigating suitable statistical structure; it does not prove the proposed head's kurtosis/monosemanticity guarantee. The note's numbered references [1]–[25] have no supplied bibliography, so their identities cannot all be verified.

[Attention by Synchronization in Coupled Oscillator Networks](https://arxiv.org/html/2606.12059v1) supplies a more explicit research example: fixed query anchors and positive couplings yield a normalized aggregate at a fixed point under stated conditions. Its implementation uses learned linear maps, softplus and continuous dynamics, so it is not a directly compliant UOR replacement. It supports the plausibility of synchronization-based aggregation while illustrating the missing need for defined inputs, readout and convergence assumptions. No comparator from that paper was run here.

## 7. Our present limitations and measured costs

This assessment does not defend the current implementation by assuming it is already adequate.

Each current block has 16 possible roots, or four bits of root identity despite eight coordinate slots. Four independently varying blocks admit 65,536 tuples. The original initializer produced only **16 distinct full vectors for 4,096 tokens**, with 256 identities per pattern. Archived pilot diagnostics and the xorshift low-bit structure agree on this collision partition; changing its seed merely relabels it. The separately merged constructor is one candidate response, not a completed language result.

Route's fixed-query position selection can miss content-dependent distinctions. Flock makes selection content-dependent, but its finite frame states produce ties and its snapped consensus is many-to-one. A retained exact occurrence tree does not turn that lossy summary into a lossless context vector. The writer's eight-cell admission can exclude the target regardless of its later rank. These are distinct possible bottlenecks; the new attention proposal does not identify which one dominates useful answers.

Preserved [D2 measurements](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/docs/labs/discrete-driver-cost-2026-10-13/README.md#L120) used V4096/B4/C192/batch64 and three warmed synchronized API repetitions:

| API | Route median | Active Flock median |
|---|---:|---:|
| Forward, 64 target rows | 37.837521 ms | 40.576018 ms |
| Evaluate, 64 rows | 42.143551 ms | 44.765762 ms |
| Vector coordinate proposal | 3.903280515 s | 4.206217974 s |
| Read-parameter proposal | 3.959623663 s | 4.180529788 s |

These are prior fixed-batch API observations, not new measurements, decode rates, isolated transport timings or long-context scaling. The per-proposal cost includes repeated candidate evaluation and writer scans: a fully eligible 16-move proposal invokes the read/writer on baseline and trial for each move. Making the eight-coordinate action inexpensive does not remove those costs. Any whole-path efficiency claim must measure them.

The [seed-1 pilot](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/docs/labs/a2-pilot-2026-10-14/README.md) made 236 vector updates over 1,640 batches/104,960 masked-target exposures. On the 14,287-row DEV panel, top-1 rose 17→82 and top-8 173→1,169, below the masked-TRAIN count reference 1,226/4,060. Distinct full vectors grew 16→155 and occupied cells 16→124, while target admission fell 5,680→2,180. The exact loss increase was 68,000 numerator units: token contribution −63,968 plus address contribution +131,968. Rank and composite loss measure different aspects, so their opposite movements are arithmetically consistent. That decomposition alone does not prove a causal explanation for every changed row. [Preserved diagnosis](https://github.com/UOR-Foundation/uor-r4/blob/23bd6128011e6903c62b9c966154e2f9da33a792/docs/labs/a2-pilot-diagnosis-2026-10-14/README.md).

Route parameters had not yet been attempted in that pilot; Flock/gates were outside its arm; fresh transfer is NOT_RUN. These facts neither establish general language usefulness nor reject SpiralCore. The registered completion exists to answer its frozen question, and this review does not amend it.

## 8. The two thinking documents

**Lawful but Unselected** distinguishes legal configurations, reachable arrangements, functional success and what was realized. That is a useful discipline here: group closure establishes lawful actions; an orbit establishes reachability; neither establishes a successful language task. The outline explicitly recognizes combinatorial explosion and substrate dependence. It does not supply a complete efficient search or prove that the component inventory can express every desired function. Its formal realized-state notation is appropriate for a *successful* realized device; a general realized state can be reachable yet nonfunctional, as its own prose acknowledges. (A4: formal skeleton around 820–855 and scope guardrails around 1020–1035.)

**SIOPL** proposes typed source/claim/assumption/interpretation records, candidate and rejection events, state changes, and replay provenance. Its observed/directly-logged/inferred/post-hoc/unknown distinction is appropriate for UOR's existing artifact/epoch/identity receipts. It explicitly avoids requiring raw private reasoning transcripts. A useful first integration would record actual selected nodes/actions, tested alternatives, deterministic ties, source versions and missing records. A candidate never tried should remain untested, not acquire an invented reason for rejection. (A5: 129–144, 493–512.)

Neither observability nor a hash makes a statement true. Semantic parsing, contradiction detection and grounded constraint predicates remain substantial components. A trace can show which record and operator produced an answer; it does not automatically supply the semantic reason. Exhaustive alternative logging also has a cost, so provenance should be bounded and tied to actual events.

These documents provide useful standards for the research process and potential application observability. They do not fill the attention note's missing learning or truth-detection proof.

## 9. What should change, and what adoption would require

**No immediate model-code replacement is required by this review.** The immediate correction is to the claim boundary: cheap invertible action, stateful exact transport, bounded summary, semantic read and learned language result must be reported separately. The validation-paper arithmetic and the stale runtime-initializer description should be corrected in their owning sources. Those are specific observed defects, not an obligation to invent a model change.

The following roadmap is conditional. It preserves the current qualified implementation and separates questions so that an outcome can change a decision.

| Stage | Concrete deliverable and reuse | Discriminating check | Decision / owner boundary |
|---|---|---|---|
| 0. Evidence reconciliation | This report, input identities, byte receipts and explicit proposal claims. | Source/receipt checks and algebraic counterexamples; no model compute. | Keep exact transport; retain attention as a candidate. No training release or capability promotion. |
| 1. Complete the proposed read contract | Specify finite state/node domain, retained payload/identity, causal graph, action labels, finite update/termination rule, readout to the same codebook and exact learning moves. Reuse current action, binder, epoch and writer interfaces. | Work through duplicate payloads, conflicting updates, negative pair/triad, tied frames, empty/missing children and input order. Identify exactly which behavior current Route/Flock cannot provide. | A coherent discrete contract becomes implementable. If only continuous physical dynamics are intended, it remains a distinct hardware research proposal under current rules. |
| 2A. Optional state-transport integration | Rust serialization/synchronization of immutable typed records or snapshots, with dictionary/action/tokenizer versions and explicit backing-store identity. | Cold/warm/churn/eviction/reseed, same-position and shifted edits, corruption, duplicate identity and full reconstruction. Compare against ordinary content-addressed transfer on the same workload. Charge total storage, bytes, CPU/GPU boundary and round trips. | Adopt if a real application transfer/recovery need benefits. No attention replacement is needed for a storage-only gain. Content-defined chunking is justified only if insertion churn matters. |
| 2B. Optional discrete relational read | Add a source-bound read mode to schema/dispatch/snapshot lineage; use the same writer and full path. Keep frame and payload distinct. | Exact causal and transport checks, noncollapse witnesses, finite transition/cycle behavior, integer-kernel census and refusal behavior. Include a concrete query whose answer changes with relevant earlier content. | Construction qualification establishes implementation, not language superiority. Reuse existing Flock rather than creating a new mode if it already answers the specified behavior. |
| 3. Prospective learned comparison | One material native question: content-based read, relational state, or summary sufficiency. Fix tokenizer, corpus, writer, exposure and checkpoint accounting; declare initializer and read changes separately. | Train the mechanism in; compare admitted discrete controls under a FIT sheet. Measure mechanism-specific behavior, complete answers, admission/ranking, exact loss components, end-to-end cost and fresh transfer. Use appropriate seeds/rows and panel uncertainty. | Compute requires exact-source FIT and the applicable owner release. A configuration result does not close the geometric mechanism. No prohibited teacher/softmax comparator is needed. |
| 4. Shared application path | Connect the accepted carrier/read/writer to input compilation, versioned memory writes, retrieval, answer emission, save/reload and session continuation. | Source changes must affect generated answers; repeated equal values, reassertions, stale versions, order and missing backing data must behave explicitly. Measure actual decode and retained storage. | Promote only the demonstrated application behavior. Constructor or codec passes do not establish useful chat. |

The cheap review stage does not justify a guessed GPU budget or a 100,000-token promise. Before any later compute sheet, project complete preparation/build/controls/evaluation/retry cost from the actual candidate, use the existing D2 measurements only at their measured shape, and measure scaling rather than extrapolating it silently. Resource ownership/caps and D24's no-CPU-model-compute rule remain. No future run is authorized by this roadmap.

For every proposed check, answer the repository's four fitness questions: what is measured, whether that quantity exists at this construction stage, whether the instrument can resolve the bar, and whether this check belongs at this stage. Exact invariants can use exact equality; semantic/generalization claims need task evidence and uncertainty. PAS and arbitrary tolerances must not become a veto over an entire mechanism.

The most useful next design decision is **what additional information the proposed read retains and uses that existing Flock does not**. If the answer is only smaller serialized roots, it is a storage/transport improvement. If it is persistent relations, conflict structure or better query-dependent selection, those need an explicit discrete law and a task where the distinction changes an answer. That decision keeps SpiralCore central while avoiding a name change being mistaken for a new capability.

## 10. Reproducibility and evidence limits

The owner checkout and peers' source/jobs were preserved. The audit used an owned claimed worktree, Rust harness heartbeats and three native review assignments. The harness build was local tooling; no model initialization, forward, evaluation, optimizer, training, CUDA context, pod allocation or owner-release action ran. No new dependency was installed for the review. PDFKit extracted the PDFs; key equations were also visually checked. JavaScript was used only to exercise the supplied JavaScript byte codec, not to add a model implementation.

The measured harness build took 42.70 seconds with four jobs. Scratch/build/extraction material occupied approximately 549 MiB at the delivery check; the report and retained evidence occupied approximately 100 KiB. Free local space at that check was 45 GiB. The codec's 1.59-second run and RSS above are scoped measurements, not the total review's CPU or elapsed cost; no complete aggregate CPU accounting was collected for parallel document analysis. Model compute and paid pod use by this task were zero.

The retained [independent-codec.cjs](evidence/independent-codec.cjs) expects the unmodified ZIP's `core.js` and `transport.js` under a sibling `showcase/` directory. Reproduction: copy the script into a fresh directory, extract the ZIP there, and rename its extracted `SpiralCore_Showcase/` directory to `showcase/` beside the script; then run `node independent-codec.cjs`. Its result file is overwritten by that new run, so keep the preserved evidence separate. The original command was `/usr/bin/time -l node --max-old-space-size=128 local/transport-audit/independent-codec.cjs`. The [resource log](evidence/independent-resource.log) and [run log](evidence/independent-run.log) bind the reported observation. The three supplied test outputs are retained separately. Input source is identified below, not republished wholesale.

No claim is made here of browser production readiness, network security qualification, physical oscillator operation, universal compression, new language quality, new energy measurements, or a completed migration of the app. The original validation paper's full benchmark environment and the attention note's numbered bibliography were unavailable. A failed initial filename lookup was corrected; no successful result is inferred from that lookup or from an unavailable tool. The legacy model-time ledger path was absent and was not recreated or treated as a fresh resource allowance.

### Supplied-artifact manifest

All originals were supplied from `/Users/casey.allard/Downloads/`. Page references in this report refer to the original PDFs; JavaScript references refer to unmodified ZIP members; HTML/text line references refer to the supplied files. The attention note's internal instructions were read as proposal content only.

| ID | Original filename | SHA-256 |
|---|---|---|
| A1 | `To train attention heads in this architecture, you don't just throw massive floating-point matrices at backpropagation and hope they don't hallucinate..pdf` | `78f709022c40731700676c920dd9a6532cf83237f2e00703d7b9ef4feb545a0d` |
| A2 | `SpiralCore_Showcase_Local_App.zip` | `6399240e51ad5f5347390690ce45f460eda390bbe1d9c5e86dee0e8fd5c26754` |
| A3 | `SpiralCore_Cantor_Abraxas_End_to_End_Validation_Paper.pdf` | `ce1a88df14d6d2b14e57b3c96fadfb0f52cfcfe0816ee55b9fce618c0b057294` |
| A4 | `lawful_but_unselected_outline_v2.txt` | `4c61fc789e9773b5fe5ae0fc6a96f985074bca55cb357a0e7b061785391d1d4c` |
| A5 | `SIOPL_RFC_Draft_v0.1.txt` | `7a63158c32513de412f879217b84e8497ceb777f80f470a7ed362ead2837802e` |
| A6 | `view-source_https___spiralcore.netlify.app.html` | `770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8` |

The extracted transport `core.js` SHA-256 is `7ca3dcfb552ca6ff3fc8696734265bcd9b993aa8b8e6242984060ace8b35bc22`; `transport.js` is `1d254a75dfe497f91b302ada0d581eab3407c7ea8f3994920a58d9a2652dc0fe`. The independent script and retained receipts have their own [SHA256SUMS](evidence/SHA256SUMS.txt). These identify what was reviewed and run; a hash does not establish scientific correctness.
