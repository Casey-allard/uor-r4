# UOR-R4: geometric-attention research synthesis

**Date:** 28 September 2026. **Pinned roadmap/source:** `15d2ce3fac9f66ddcc945a8097e5c6216352c453`. Later live issue observations are separately identified. This is an advisory research result, not a changed roadmap, a model promotion, or an authorization to interrupt another lab.

## Verdict

Continue the GLM programme, but change the geometric-attention research method. The retained Rust learner, exact occurrence memory, integer execution, and fixed experimental contracts are useful foundations. They do not yet establish a path to frontier-equivalent language. The central missing evidence is that a learned geometric representation retains the distinctions needed by a task, can select them at bounded cost, and improves or preserves useful responses through the complete hard runtime.

Do not interpret a failed score replacement, a finite automaton, or a small matched-model loss comparison as deciding the entire GLM hypothesis. Conversely, do not preserve a failed implementation merely because its mathematical vocabulary matches the mission.

## Live corrections

The roadmap at the pinned revision has explicit base/export/session/admission/dialogue interfaces. The base model decision remains conditional on the full-exposure comparison; PR1437 remains draft and its cycle-5 memory arms are reported unrun. The code BPE versus dialogue tokenizer integration remains unresolved. Retargeting a training recipe to a new architecture is not loading its incompatible weights.

The fresh reflection-pair replication reported on issue973 at 04:43:31 UTC fails its original per-seed text-loss rule: one seed costs +0.0688 nats against a limit of +0.05. Its A5 tracking succeeds, but no lane type is retained in the stack. Exact finite-state execution survives as a synthetic result. The owner has transferred execution of the frozen dialogue-code-choice study to Claude because the Codex allowance is exhausted; that transfer does not change the study.

The finite 2I reader in PR1438 is a valid scoped negative. It normalizes away the magnitude of each four-coordinate block, snaps direction to 120 roots, and introduces a learned nonlinear score using a biased straight-through surrogate. Its matched loss, complete-answer and cost failures remain. The triggered norm/capacity control was not executed, so the outcome does not isolate which of those changes caused the failure.

## What counts as geometric attention

A useful definition for this programme is a learned read operation in which a declared geometric representation determines candidate access, compatibility, or value transformation; its complete numerical execution obeys the native runtime contract; and its contribution is established against an equally informed ordinary comparator at matched cost.

Three questions remain distinct: geometry's effect on useful prediction, geometry's effect on representation/access cost, and conformance of the implementation. Success at one does not establish the others. A prime/hash identity is not a semantic metric; exact provenance can remain valuable without carrying semantic distance.

## Mathematical boundaries

### 1. Scalar quaternion compatibility can be ordinary dot compatibility

For real quaternion coordinates,

`Re(conj(q) * k) = dot(q, k)`.

This is an identity, not a new predictive mechanism. The repository's September26 attention design already states it, as well as the limitations of linear relative scores and endpoint-only value transport. The Wolfram symbolic checks performed during this review also return zero for the identity's difference.

### 2. Removing magnitude can reverse a correct ranking even on exact roots

Take `q=(1,0,0,0)`, `kA=(1/4,0,0,0)`, and `kB=(1,1,1,1)`.

Their dot scores are `1/4` and `1`; keyB wins. Normalizing each key gives scores `1` and `1/2`; keyA wins. Both directions are actual 2I elements, so this reversal requires no angular quantization error. Retaining gains `1/4` and `2` restores the original scores.

This proves representational loss in that construction, not that norm loss caused PR1438's measured regression. Norm-explicit quantization is established prior work, and the repository already warned about this control.

### 3. Extra products inside a closed group do not increase resolution

If `g1,g2` are in `G`, then `g1*g2` is still in `G`. Encoding two elements and retaining only their product cannot yield more than `|G|` distinguishable directions. Refinement must preserve additional information, use a genuinely larger codebook, or leave the original group through additional representatives. HQMQ explicitly makes the analogous distinction for its 24-element group.

### 4. Pure group state is not the right primitive for every semantic update

A group action is invertible. Overwriting a current value maps distinct previous values to the same result and is not invertible. Therefore group transport alone cannot implement all update/reset/forgetting semantics. Reuse explicit versioned writes and gated/non-invertible operations; do not force them into a rotation claim. Endpoint transport also factorizes into changes of frame unless additional path/edge structure is genuinely represented.

## Proposed mechanism: learned asymmetric group-orbit codebooks

This is a project-specific research synthesis, not a claim of a globally new algorithm. Orbit-generated quaternion quantization, asymmetric attention indexing, output-aware quantization, and counterexample-guided refinement all have prior literature. The untested contribution is their integration into this native recurrent model, with its task distinctions and complete runtime constraints.

Let `G=2I`, and let `a_i^Q` and `a_j^K` be small sets of unit representatives. They need not be in G and may differ for queries and keys. For the first candidate keep these representatives fixed; learn the assignments through the existing low-bit-servable encoders. Do not conceal high-precision learned weights in a supposedly fixed table. Encode a four-coordinate block as

`qhat = rhoQ * c(gQ) * a_i^Q`

`khat = rhoK * c(gK) * a_j^K`.

Then

`dot(qhat,khat) = rhoQ*rhoK*Re(conj(a_i^Q) * c(inverse(gQ)*gK) * a_j^K)`.

The score therefore uses a table `A[i,j,r]`, where `r=inverse(gQ)*gK`, rather than an arbitrary table over every pair of expanded codewords.

For four representatives on each side and 120 group elements, there are up to 480 codes per side. A full pair table has 230,400 entries; the relative-group table has 1,920. At hypothetical int32 precision these are 921,600 and 7,680 bytes, or 8,192 bytes after padding the relative-code dimension to 128. This is a directional-score-table comparison only. It is not a 120-fold model speedup or a comparison with the best ordinary compressed implementation.

This parameter sharing imposes simultaneous left-G invariance. The encoders must learn a representation for which that restriction is useful; language does not come with a demonstrated physical icosahedral symmetry. For a learned nonlinear relative kernel, this restriction still permits more than rank-four coordinate scores: `F(r)=1` at identity and zero otherwise gives an exact rank-120 identity matrix over one orbit. That expressivity example is not an advantage over arbitrary high-dimensional learned embeddings and is not evidence of learnability.

### Execution and representation obligations

A real numerical serving path must include query/key code selection, gains, index access, scores, value mixing, normalization and sampling. No inference-time FP32 query-table construction is allowed. NoMAD's original query-dependent FP32 preprocessing is specifically not a compliant substitute.

For dyadic gains the gain product can be implemented by exponent addition and bounded shifts, with explicit rounding and overflow rules. Other gain grids require a bound product table or another declared integer construction. Fixed mathematical tables and wider activation/state integers are not additional high-precision learned weight maps; learned residual maps still obey the declared low-bit contract.

A fixed rational representative can support shift/add classification: scale its four-coordinate transform to an integer numerator, compare common-scale scores, and reuse the H4 classifier. The existing helper needs appropriate score-return and range checks; this is a design possibility, not an implemented or instruction-audited encoder. Learned arbitrary representatives require separately qualified coefficient coding/classification. The four-orbit codebook is not itself a closed 480-element group, and its arbitrary coordinates are not all integral icosians. Exact group-label composition and approximate scored coordinates must remain distinct claims.

Do not replace event records with independent per-lane histograms. Memories `{(0,0),(1,1)}` and `{(0,1),(1,0)}` have identical marginals but different bindings. Geometric codes can route to a record; its identity, owner, scope, version and payload must remain available to the actual consumer.

## Executed checks and results

These runs used this session's Linux analysis container, not the M1 training slot. They loaded no UOR weights or language corpus. The coordinate ordering is independently constructed, not the repository's canonical ID order.

`finite_group_checks.py` checked 120 exact roots over Q(phi), 14,400 products/relative-dot identities, and 1,728,000 simultaneous-left-action combinations. A fixed Q16 gain/relative lookup was checked at 921,600 root/gain combinations. All assertions passed. Exact group operations do not make the Q16 scores exact: the declared per-table-entry rounding bound is `1/(2*65536)`.

`orbit_codebook_checks.py` fixed four rational representatives before running: `(1,0,0,0)`, `(3/5,4/5,0,0)`, `(1/3,2/3,2/3,0)`, `(2/3,1/3,2/3,0)`. It found 480 exactly distinct directions, independently checked all 1,920 reduced table entries with rational/golden arithmetic, and cross-checked 230,400 full pair scores numerically. Maximum discrepancy was `4.44e-16`. A separate Wolfram polynomial check of the factorization returned zero.

A fixed synthetic diagnostic used 128 queries, 32 events per query, and 16 four-coordinate blocks. Seed was20260928; it is not a language dataset.

| Configuration | Top-1 retained out of128 |
|---|---:|
| Unquantized directions, magnitude discarded |39|
| Unquantized directions, eight dyadic gains |94|
| One120-code orbit, exact diagnostic gains |67|
| One120-code orbit, eight dyadic gains |61|
| Four fixed orbits/480 codes, exact diagnostic gains |95|
| Four fixed orbits/480 codes, eight dyadic gains |81|
| Ordinary random antipodal480 codes, exact diagnostic gains |88|
| Ordinary random antipodal480 codes, eight dyadic gains |76|

Exact diagnostic gains are not a fixed-bit serving scheme. Four orbits use more directional bits than one. The random comparator is not a fitted or optimized ordinary quantizer. The outcome supports investigating expanded resolution, but establishes neither geometric superiority nor acceptable preservation. Increasing codebook size after inspecting the first failure was a new explicitly recorded construction, not part of a pre-registered language comparison.

### Margin certificates: correct mathematics, not yet useful acceleration

The dot-score error satisfies

`|dot(q,k)-dot(qhat,khat)| <= ||q-qhat||*||k|| + ||qhat||*||k-khat||`.

Add score-table and any other numerical errors. If candidate a's lower score bound exceeds every other candidate's upper bound, its top-1 identity is certified for that reference score. The bound follows by writing the difference as `(q-qhat) dot k + qhat dot (k-khat)` and applying Cauchy-Schwarz. A global retrieval certificate also needs bounds for unvisited cells. It does not prove semantic truth or preserve the full soft attention output.

The first off-grid diagnostic had zero bound violations over4096 scores, but zero certified queries. An on-grid separated example certified non-vacuously. Therefore these loose bounds are not justified as a production pruning mechanism at the tested coarse precision. Tighter data-dependent bounds/refinement remain a hypothesis, not a recommendation to add a certification framework before useful learning.

## The research process worth changing

Train and refine codes for the decisions they must preserve, not just Euclidean reconstruction or code usage. A minimal decision trace distinguishes: relevant occurrence unavailable, available but scored incorrectly, correct read but incorrect value/action, and correct latent result but incorrect emission. Each class needs a different intervention.

For exact identical **complete served decision inputs**, conflicting required answers cannot be fixed by tuning a downstream deterministic scorer. If input class z contains labels with counts n(z,y), the best deterministic empirical accuracy on those classes is bounded by `sum_z max_y n(z,y)/N`. This bound is not valid if the diagnostic omits side information that the real consumer sees. Add resolution or the missing relation/version information only after establishing such an alias or another causal representation error. Otherwise repair ranking, learning or emission rather than widening every code.

A conditional learning objective can combine natural-language loss, context-grounded relation/ranking loss, hard/soft preservation loss, and measured access cost. Grounded labels must be available only in training/evaluation, not supplied by an oracle at runtime. Separate faithful imitation of an existing reader from correcting its mistakes. Query and key encoders can differ; Saap provides an adjacent attention-indexing precedent, not a complete implementation under D11.

## Three bounded next routes

1. **Frozen-reader representation screen.** Reuse OpenCode's query/key exposure and original evaluator. Compare original score, magnitude removal alone, directional coding with restored gains, expanded orbits, and an equally budgeted ordinary codebook. Preserve tokenizer, parent, candidate set, age, NoRead, values/copy and decoding. Score both dense-reference fidelity and actual relation/version use. No model fit initially. If magnitude or resolution is not causal at this scope, do not revive PR1438 by assumption.
2. **One learned hard-read candidate.** Only if the screen identifies a useful representation/cost opportunity, modify the selected base's existing encoder/read interface. Use informed initial scores, a bounded nonlinear residual if justified, actual hard-forward code use, matched ordinary opportunity, and language/context credit. Preserve the existing frozen studies; this is a separately declared follow-on, not a third learner. Failure of whole replies or the declared cost/retention contract parks the candidate without an automatic dose extension.
3. **One same-artifact native execution comparison.** Convert the selected candidate, preserve source/model/tokenizer/codec identity, audit actual numerical code, run the same full replies, and measure complete M1 traffic/latency/RSS/energy. Candidate admission must exclude work, not merely rank a densely scanned store. Event sparsity does not remove the dense output head or parameter maps. The original T2 1-point recall/0.005-nat comparison remains scoped and must not be relaxed after inspection.

Claude should own the causal study/base decision and the authorized frozen #1433 continuation. OpenCode should supply the existing query/key/admission instrumentation and strong ordinary index controls. Anti-Gravity should price and implement only the selected numerical contract, rather than optimize a moving proposal before language preservation is known.

## Novelty and feasibility

This review does not establish worldwide novelty. HQMQ already uses group-generated quaternion codebooks; NEQ separates norm and direction; Saap uses asymmetric assignment; group-equivariant attention is established; CEG4N already combines counterexamples and quantization/equivalence checks. The relative-orbit lookup factorization follows from quaternion algebra. The candidate contribution would be a demonstrated, learned, task-conditioned native language read with explicit identity preservation and a measured quality/cost frontier.

A practical transformerless geometric model remains a credible research objective. Frontier-equivalent capability remains unestablished and requires evidence about learning scale, knowledge, robustness, context, reasoning and complete-task cost. Laptop inference and laptop-only training are different constraints. No new external or paid compute is authorized by this note.

## Scope of this session

Live GitHub/source and the authorized laptop were read. Primary literature was retrieved through web and research tools. Symbolic and exact/synthetic mathematical checks executed. No independent model-review panel completed in this turn; the previous Codex attempts were usage-blocked and are not counted as reviews. No repository file, active job, checkpoint, frozen gate or GitHub issue was changed. No geometric language model was trained or promoted.

## Source navigation

R1–R5 and P1–P11 are defined with stable URLs and explicit scope in `SOURCES.json`. Roadmap/track findings use R1–R3/R5; the prior mathematical-design cautions use R4; norm/orbit/asymmetry precedents use P1/P2/P4; the FP32 preprocessing caution uses P3; refinement precedents use P8. New numerical statements are reproduced by the two delivered scripts and their exact result JSON, not by an external paper.
