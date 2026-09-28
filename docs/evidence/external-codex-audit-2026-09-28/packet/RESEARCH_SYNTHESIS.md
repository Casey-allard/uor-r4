# UOR-R4: mechanism diagnosis and a route toward geometric attention

**Research note, 28 September 2026.** Advisory research, not a replacement roadmap, implementation authorization, or model promotion.

## Verdict

Retain the native geometric-language-model objective and the Rust learning/runtime foundation. Rethink the attention representation, what each experiment identifies, and the relationship between geometric transport and mutable memory. The current programme can develop a useful native recurrent language model; neither this review nor its source evidence establishes that its current mechanisms will produce frontier-equivalent laptop capability.

The highest-value immediate finding is a **radial-information confound in two investigated mechanisms**. The failed finite-group reader normalizes every four-dimensional query/key block. The current local addressing implementation stores fixed unit-root codes without a per-key gain, while learned k-means centroids can retain magnitude. These experiments are valid comparisons of their complete parameterizations. They do not isolate geometry from information loss.

The proposed route is **task-trained, norm-preserving geometric memory attention**: preserve exact occurrence and version bindings; retain enough gain and directional information; learn query-dependent relational selection; compile the learned finite operations into bounded integer lookup; use non-invertible addressed writes alongside reversible geometric transport. The ingredients have substantial prior art. The project-specific integration is a research hypothesis, not a demonstrated globally novel algorithm.

## 1. Sources, authority and executed scope

Repository baseline: `15d2ce3fac9f66ddcc945a8097e5c6216352c453` (PR #1449, merged 04:48:28 UTC). Live lab-board snapshot: 04:55 UTC. Read-only laptop/source observations continued through 05:26 UTC. The owner checkout was not edited; no fit, model evaluation, build, checkpoint conversion, or paid model review was started by this investigation.

Current owner decisions outrank older instructions: the prospective product gate is approved; Claude is authorized to execute the unchanged #1433 study because Codex is out of tokens. Its frozen checkpoint and evaluations remain unchanged. At the observed boundary Claude held the six-thread B1 closure slot. This note does not change that queue.

Evidence roles:

- Repository reports establish what the labs reported and which gates they applied. This session did not reproduce their model runs.
- Source inspection establishes representation and interface facts at named revisions.
- `probe.py` and `probe-results.json` contain this session's independent exact algebra and synthetic quantization computation. They use no project data or checkpoint, and their root ordering is not UOR artifact compatible.
- `wolfram-check.wl` records a separately evaluated symbolic Hamilton-product check. It returned zero polynomial differences for the three stated identities.
- Primary literature supplies prior mechanisms and external measured evidence, not UOR conformance.
- No independent multi-agent expert panel completed. The preceding session's Codex attempts failed at the usage limit. The present synthesis uses source review, primary research, mathematical reasoning, and independent symbolic computation, not fictitious reviewer votes.

### Repository sources

[R1] ROADMAP at the pinned main revision:
https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/ROADMAP.md

[R2] Live board, observation 04:55 UTC:
https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5860922002

[R3] Finite geometric reader implementation:
https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/crates/uor-r4-training/src/geometric_read.rs

[R4] Finite geometric reader result and deferred attribution control:
https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/docs/integration/geometric-read-result-2026-09-27.md

[R5] Fresh B1 replication, 04:43 UTC:
https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863591275

[R6] Local OpenCode source, not published at observation: commit `924ffb2d77f0248158e647a59006cc1c2885b2d5`, `crates/uor-r4-training/src/addressing_arms.rs`. Git blob `656e39b40bc78bc11182d76ee173e867c3b4977d`; source SHA-256 `a4dc5e08e84007ab3d3d0b80a92cda0c028912c262f610c3c921a98194e64cf9`. Inspected through read-only `git show`, not inferred from another document. `local-source-observation.md` records the relevant excerpts and retrieval commands.

## 2. Roadmap diagnosis

### Preserve the negative results without inflating their scope

The finite reader used sixteen four-coordinate lanes, normalized signed 2I codes, exact relative group composition, and a learned 4→8→1 score per lane. It already preserves the full signed relative element, not just scalar distance. Calling a new full-relative-element scorer novel would rediscover that implementation.

Its 1,024-update screen was a valid negative: Read NLL worsened by 0.019719 nats, complete source answers fell from 27 to 17, and reported paired training cost was 1.84x. The norm-controlled attribution arm was deferred. The result therefore does not identify whether the harm came from gain removal, coarse coding, initialization/optimization, scorer capacity, or their combination. Preserve HARM; do not replace it with a retrospective pass. [R3–R4]

The fresh reflection-pair replication also failed its frozen gate: seed 4 incurred +0.0688 nats against the 0.05 cap. Nevertheless all three new seeds learned exact A5 tracking and passed the finite certificate. This is failure of the specified language-cost tradeoff, not failure of finite-state algebra. No lane type is currently retained in the stack; Stage C is parked. Do not relaunch those studies under a new name. [R2,R5]

The roadmap's statement that geometry has failed as a continuous score or mixer is wider than the measured configurations. Its explanation of the prose deficit as capacity/exposure is also not a complete causal identification. Parameter count, objective, data exposure, context conditioning, and mechanism have not all been isolated by one factorial comparison. The existing mainline decision still stands at its declared scope. [R1]

### Admission, ranking, value use and writing are separate mechanisms

An admission index answers which records are inspected. A reader answers which admitted records matter. A value path determines how their contents affect output. A write path determines which associations will exist later. Improving one does not qualify the others.

The current T2 plan already uses asymmetric scoring: quantized keys and an exact query. Recommending asymmetric scoring alone would duplicate its work. Its complete equal-cost contest remains useful, but should not be interpreted as a pure test of geometric versus ordinary structure. [R6]

At that local revision, `Codes` contains only block IDs. H4/E8 reconstruction returns fixed unit roots. K-means fits raw block vectors and reconstructs learned centroids, which can have different norms. Thus the fixed-root and learned-code arms differ in available radial information. At equal current storage this is a legitimate practical quantizer comparison. To identify the role of geometry, a subsequent controlled comparison must give both gain-shape families the same gain budget, charge all added bytes and operations, and distinguish that new comparison from the frozen original.

### Sparse event access does not finish the runtime architecture

R3/D5 requires selected learned-parameter access, not only fewer event reads. A model can have an excellent event index and still touch its entire projection/vocabulary store every token. Query construction, write construction, value aggregation, emission, normalization, sampling, and state maintenance all belong in the final cost and numerical contract. [R1]

## 3. Mathematical findings

### 3.1 Ordinary dot attention already contains geometry

For real four-vectors interpreted as quaternions,

`q · k = Re(conjugate(q) k)`.

For nonzero vectors, write `q=a u`, `k=b v` with nonnegative gains and unit quaternions. Then

`q · k = a b Re(u^-1 v)`.

Consequently, relative quaternion composition can represent the ordinary score, but discarding `a` and `b` changes it. The identity and common-left-frame invariance were checked symbolically and through the finite group. These are established algebra, not novelty claims.

Counterexample: `q=(1,0,0,0)`, `k1=(1,0,0,0)`, `k2=(1.5,0,1.5,1.5)`. Dot ranks k2 above k1: 1.5 > 1. Unit-only scoring ranks k1 above k2: 1 > 1/sqrt(3). More fitting cannot reconstruct an omitted norm from a fixed code that does not carry it. A newly trained encoder could reorganize its representation, which is a different question.

### 3.2 Coordinate transport can be a placebo

Let local value `v_j^local = rho(g_j^-1) v_j^global` and use pure frame transport `rho(g_t^-1 g_j)`. Mapping the read back gives

`rho(g_t) rho(g_t^-1 g_j) rho(g_j^-1) v_j^global = v_j^global`.

This coordinate change alone adds no new operation. A task-dependent relation action or genuinely different learned state is needed, and its contribution must be tested against an equally informed ordinary operation. Common-left-frame invariance is an algebraic guarantee; semantic equivalence of sentences is learned and separately tested.

### 3.3 Mutable memory needs more than invertible group actions

Overwriting Alex's color with green maps both a blue state and a red state to the same current state. Such an operation is non-injective. A group action is invertible, so pure group transport cannot implement that overwrite on the same state space. Retaining complete version history in a larger state removes the contradiction, but consumes memory.

Use reversible group operations for transport/composition and explicit addressed writes, versioning and bounded forgetting for memory. The complete transition family need only be closed under composition, not consist exclusively of invertible rotations. This does not reject geometry; it assigns different operations their proper jobs.

### 3.4 Exact bindings must survive compression

The records `{Alex→blue, Bea→red}` and `{Alex→red, Bea→blue}` have identical owner and value marginals. Independent owner/value summaries therefore cannot answer the same query correctly in both cases. Joint occurrence bindings, or an equivalent representation of the conjunction, are required. Geometric summaries should not silently erase exact identity/version distinctions.

## 4. What the executed probe establishes

The independently constructed 2I table used exact coefficient pairs `(a+b*phi)/2`, with `phi^2=phi+1`. It checked 14,400 products, all 1,728,000 associativity triples, 240 inverses, 14,400 scalar-inner-product identities and all 1,728,000 common-left-frame cases. The scalar real part takes nine values; the full relative state takes 120. This is a finite arithmetic check, not proof that learned semantics belong to that group.

For each of 4, 16 and 64 dimensions, the seeded synthetic retrieval test used 128 queries against 255 keys. Each four-dimensional block had random isotropic direction and log-uniform gain in [0.5,2]. Keys and queries were quantized to 120 directions, optionally retaining a 16-bin gain. This distribution is not the project's learned distribution.

| Dimension | Unit-only top-1 agreement | Gain-preserving top-1 agreement | Exact-score refinements, mean / 255 |
|---|---:|---:|---:|
| 4 | 8.59% | 44.53% | 22.08 |
| 16 | 15.63% | 46.88% | 59.23 |
| 64 | 22.66% | 45.31% | 218.16 |

The gain-preserving representation uses extra bits. These are not equal-rate or quality-matched wins. They demonstrate the possible information value of gain and the remaining inadequacy of this coarse codebook.

The refinement used the valid bound

`|q·k - qhat·khat| <= sum_b (||qhat_b|| eps_kb + ||khat_b|| eps_qb + eps_qb eps_kb)`.

Every refined top-1 matched the original reference, but all candidate bounds were constructed and the refinements used floating-point reference scores. At dimension 64, about 86% of keys still required refinement. This is **not an efficient sublinear index or D11 implementation**. Do not implement this bound-based design as the presumed solution without evidence of useful pruning on real queries.

## 5. Relevant prior work

[P1] Dai et al., *Norm-Explicit Quantization*, AAAI 2020, https://ojs.aaai.org/index.php/AAAI/article/view/5333 . Explicitly separates norm and direction for maximum-inner-product search. Supports testing radial loss; does not establish UOR's cause.

[P2] Guo et al., *Accelerating Large-Scale Inference with Anisotropic Vector Quantization*, ICML 2020, https://proceedings.mlr.press/v119/guo20h.html . Optimizes errors relevant to inner-product retrieval rather than relying only on reconstruction error. Motivates query/task-aware coding and an ordinary comparator.

[P3] Lingle, *Transformer-VQ*, ICLR 2024, arXiv:2309.16354v2, https://arxiv.org/abs/2309.16354v2 . Quantized keys and caching permit linear-time dense attention. Quantized attention is therefore not new; its transformer architecture is not adopted for serving.

[P4] Basu et al., *Equivariant Mesh Attention Networks*, TMLR 2022, arXiv:2205.10662v2, https://arxiv.org/abs/2205.10662v2 . Shows meaningful equivariant attention for known mesh symmetries. Natural-language role changes are not automatically the same symmetry.

[P5] *DeltaProduct: Improving State-Tracking in Linear RNNs via Householder Products*, arXiv:2502.10297v7, https://arxiv.org/html/2502.10297v7 . Separates richer state transitions from purely diagonal recurrence; its update mechanisms include non-invertible behavior. It is an architectural comparison, not an already compliant D11 runtime.

[P6] Cabannes et al., *Sparse Delta Memory*, arXiv:2607.07386v1, https://arxiv.org/html/2607.07386v1 . Uses sparse reads and writes into larger explicit memory, including a learned initial state. This is particularly relevant to combining episodic and parameter memory. Its learned projections and floating arithmetic need substantial adaptation; do not import its numerical-performance claims into UOR.

[P7] Zhu et al., *Scalable MatMul-free Language Modeling*, arXiv:2406.02528v7, https://arxiv.org/abs/2406.02528v7 . Supports investigating language models without standard matrix-multiplication mechanisms at substantial scale. Its meaning of matmul-free is not proof of UOR's stricter opcode and sparse-access contracts.

[P8] van den Dool et al., *AVQ-Attention*, arXiv:2607.12789v1, https://arxiv.org/html/2607.12789v1 . Allocates finer codebook capacity where attention concentrates. Its demonstrated domains/implementation are not a native integer language model. Useful as an adaptive-refinement idea, not a priority claim.

This was a bounded search across primary papers, general web discovery and SciSpace. No claim of exhaustive novelty clearance is made. Consensus access was declined and not retried. Abstract-only inspection was sufficient only for high-level statements; no uninspected theorem is used as a proof dependency.

## 6. Proposed mechanism and its falsifiers

### Preserve the working model before learning a different operator

Represent each learned key block as `(gain_bin, signed_group_code, optional_residual)` while preserving separate exact event/owner/version/payload IDs. For a query, initially retain the higher-precision path as a diagnostic comparator; eventually quantize it under the same hard-serving contract.

A baseline-preserving family is

`delta_tjl = inverse(g_tl) * g_jl`,

`score_tj = sum_l a_tl b_jl Re(delta_tjl) / sqrt(d) + native_bias_tj + Delta_theta(delta_tj, causal_context)`.

Start the residual at zero. The geometric quantizer introduces a separately measured error; do not claim an exactly unchanged reader unless a witness establishes that. Learn only after determining whether the coding error is tolerable. Preserve age bias, causal mask and NoRead behavior in the baseline comparison.

A subsequent relation-conditioned variant may use `(inverse(g_tl)*g_jl)*r_tl` and a discrete relation action on geometric value features. Relation codes must be produced from causal text/state by the learner, not supplied by a privileged runtime parser. Such conditioning may also be represented by an ordinary query encoder. The ordinary control must receive the same context, parameter budget, objectives and exposure.

### Compile small operations, not a dense model hidden in a table

Group products/inverses use existing pinned tables and classifiers. A shared score table with 16 query gains, 16 key gains and 120 relation states occupies 61,440 bytes at two bytes per derived score; a 128-padded relation dimension gives 65,536 bytes. This is a design calculation, not measured serving footprint. Learned free coefficients must still satisfy the adopted low-bit rule; fixed arithmetic tables or derived scores are not unrestricted high-precision learned weights.

Charge per-lane/per-head table replication, query encoding, gain computation, residuals, index construction, value mixing, output projection, sampling and memory updates. A small relation table alone does not make the whole model small or multiplier-free.

### Couple learning to the task, not merely the dense reader

Use language loss together with training-only relational contrasts: role reversal should change the answer; valid paraphrases should preserve it; a negated transfer should not mutate ownership; current and prior versions should remain distinguishable. Optional distillation preserves the retained model on general text but must not force reproduction of known erroneous reads. Auxiliary targets are training supervision, never serving inputs.

The proposal has three separate falsifiers: gain-shape coding may fail at equal rate; the conditioned geometric operation may not beat an equally informed ordinary control; and any quality gain may disappear after hard compilation or cost accounting. A failure at one stage stops that candidate, not the entire field of geometry.

## 7. Three bounded next research units

### Unit A: real-key information-loss diagnosis (OpenCode, read-only first)

Reuse the existing parent, tune/development separation, query/key access and contest harness. Do not rerun the model merely to generate another identical dump. After the frozen contest, compare its unit-root representation with explicitly gain-preserving fixed and learned gain-shape representations at matched total bytes. Include original keys, norm-only perturbation and asymmetric queries. Record dense-score fidelity and true entity/version retention separately.

Success means finding a representation/cost point that closes a material hard-reader deficit and changes the next design decision. Failure means no useful equal-rate operating point; park the codebook/rate, not invent another unbounded sweep. This evaluation does not prove training causality or language generalization.

### Unit B: one task-trained reader adapter (Claude, only after Unit A supports it)

Attach the candidate to the selected mainline model; do not start a new learner. Freeze source/data/tokenizer, training dose and matched baseline before fitting. Use baseline-preserving initialization, real context gradients, and one declared relational counterfactual objective. Give the ordinary control identical information and supervision. Use held-out composition/paraphrase and record identity, not only fresh names or copied payloads.

Success requires the prospectively fixed language non-regression and complete-response/relational improvement criteria. Failure parks the candidate. Do not reinterpret current B1/Stage C gates or automatically extend the dose. Value transport is a subsequent ablation only if the base adapter warrants it, not another simultaneous confound.

### Unit C: hard-path and selected-access integration (Anti-Gravity)

Compile the accepted adapter through the existing classifier/tables and one mission runtime. Bind exact group ordering, tokenizer, code grids, numerical ranges and source/compiler/binary identity. Compare common-input predictions and complete generated replies before claiming retention. Measure query+index+read+write+emission traffic and cost; separate event sparsity from parameter sparsity.

Success means the accepted quality survives the actual hard path inside its declared cost and numerical contract. Failure localizes conversion or cost and preserves the result. Whole-model energy requires suitable sensors and matched output usefulness; the prior whole-system estimate is not a causal kernel benchmark.

## Final recommendation

Do not abandon the geometric objective and do not expand the current failed parameterizations indiscriminately. The immediate scientific move is to separate **useful information preserved**, **relations learned**, **operations compiled**, and **work avoided**. Geometric attention is worth pursuing as a learned addressing-and-memory operation. Whether it provides a better quality/cost tradeoff than strong ordinary mechanisms remains an empirical question this process can actually answer.

### Numerical qualification of the refinement probe

The score-error inequality is derived in exact real arithmetic. Its test and the branch-and-bound stopping comparison use float64 and a 1e-12 tolerance, not formally outward-rounded interval arithmetic. All observed rankings matched the reference; the executable is not a machine-checked numerical certificate.
