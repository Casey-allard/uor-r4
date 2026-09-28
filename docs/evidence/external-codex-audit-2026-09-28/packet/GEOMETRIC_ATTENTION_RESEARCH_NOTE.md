# UOR-R4: a mechanism-level route to geometric attention

**Research synthesis, September 28, 2026.** Repository analysis pinned to `15d2ce3fac9f66ddcc945a8097e5c6216352c453`; live director board read at its 04:55 UTC revision. This document is a proposal, not an adopted decision, implemented UOR model, or capability promotion. It changes no frozen study, source, checkpoint, running job, or lab ownership.

## Executive decision

Keep the objective and existing integration work. Change the next mechanism question from “which geometric score improves this dense read?” to “which learned, finite read/write representation preserves task-relevant distinctions while reducing actual accesses?” A useful native GLM remains a credible research objective. Neither current repository evidence nor the computations here establish frontier-equivalent language capability on a laptop.

The recommended candidate is **typed group-relative addressed memory**, with gain-preserving codes, exact occurrence records, a learned relation kernel, and selective noninvertible writes. This is a synthesis of existing ideas. The repository already anticipates finite relation kernels, radial information, exact identities, and value transport. Novelty and advantage require a genuinely new, experimentally supported implementation result, not new terminology.

## 1. Latest source state and scientific implications

The current roadmap retains the recurrence-primary capacity-matched base comparison, the addressing contest, runtime certification and conversion fidelity work. Interfaces I1-I5 now bind checkpoint, serving bundle, session, addressed reads and dialogue data. The board reports owner approval of a prospective same-hard-artifact responsive-dialogue milestone.

The latest board also records a negative that supersedes the earlier exploratory reflection-pair pass: exact A5 tracking survives in the fresh three-seed replication, but one seed has text-loss cost +0.069 nats, outside its +0.05 gate. No lane type is retained in the stack under that experiment. Stage C is parked. These are scope-specific results, not a disproof of noncommutative geometry.

The full-exposure base result is not present in the retrieved PR #1437 body. Interim training scores are not the endpoint. Apply the existing rule once the source-bound endpoint arrives; do not substitute a reads-only transformer comparator for the recurrence-primary mission architecture.

The owner has explicitly transferred #1433 execution to Claude while preserving its frozen study and checkpoint. This supersedes the previous advisory restriction against an unauthorized takeover. The recorded execution queue and actual machine lock govern scheduling.

### The strongest unresolved attribution

PR #1438 replaced the existing score by a unit-normalized, signed-2I, nonlinear per-lane kernel. It retains full candidate access and existing values/copying, and is implemented in offline floating-point training/evaluation, not integrated integer serving. Its same-dose outcome is negative: NLL +0.019719, source completions 27 to 17, and paired cost 1.84x. The norm/capacity attribution arm was deferred.

Consequently, the result supports rejection of that parameterization at that dose. It does not isolate whether the loss arose from removed magnitudes, quantization collisions, changed initial scores, extra learned parameters, optimization, or the chosen relation function. The smooth branch differentiates through live key history despite detached hard caches; the code is not evidence that all write gradients were severed.

The roadmap's statements that geometry broadly failed and that capacity/exposure explain the prose deficit should be read as working hypotheses, not identified universal causes.

## 2. Mechanism distinctions

### Geometric coordinates versus geometric computation

For real quaternions, `Re(conj(q) * k) = dot(q,k)`. A fixed linear functional of all four relative coordinates is still a bilinear form. If upstream projections are unconstrained, this is not automatically more expressive than an ordinary projected dot reader.

An actual distinctive claim should involve a restricted parameter-sharing class, nonlinear signed relations, genuine selected addressing, or task-relevant actions on retained values. Replacing a multiplication instruction with a table without reducing access or total instruction work is a different engineering claim.

### Norms cannot be removed for free

Take q=(1,0,0,0), A=(4,3,0,0), B=(6,8,0,0). Dot scores are 4 and 6, whereas unit-direction scores are 0.8 and 0.6. Ranking reverses. This is a mathematical counterexample to universal rank preservation by normalization, not a diagnosis of the specific UOR checkpoint.

Store or reconstruct a bounded gain channel, for example a quantized dyadic exponent, separately from signed direction. Measure total code bits, gain clipping, and query/key approximation under the actual task distribution. An exact real-valued gain is an attribution oracle, not an equal-bit deployment comparator.

### Rotations cannot overwrite by themselves

Every group action is invertible. A memory operation mapping both old value A and old value B to new value C is not injective. Pure rotation of the same complete state therefore cannot implement this overwrite. Reversibility can be restored by retaining the old information elsewhere, but then that history has a storage cost.

Use geometry for compatibility, relative orientation, and transport. Use explicit learned write/erase/revision policies for memory lifecycle. Current-state views and exact retained history should be distinguished. This does not require a new symbolic rule for every sentence.

### Fixed group closure is not unlimited precision

Multiplying any number of elements of a fixed group G and retaining only the product yields at most |G| states. `G G = G`. Extra factors from the same 120-element carrier do not create a finer directional codebook when collapsed into one code.

A genuine refinement must retain additional information: a residual code, a tuple that is not folded away, a larger carrier, or exact records. Arbitrary E8 roots must not be assumed to inherit the binary-icosahedral group multiplication table.

## 3. Proposed architecture

Use the selected native model and existing read interface. Do not start another complete learner.

```
exact token/occurrence tape
        |
learned contextual query, key, write and role features
        |
signed group codes + bounded gains + optional refinements
        |
query-relative addressed candidates, with measured selection cost
        |
relation-aware ranking + selected latent value operation
        |
existing shared state update and emit/copy/stop
        |
exact identity/version-preserving record updates
```

Keep two representations distinct:

- **Exact records:** occurrence, raw token/span payload, record identity, predecessor/version and explicit provenance. Entity and role assignment are learned interpretations, not guaranteed merely by assigning a record ID. The initial test can use the existing occurrence tape instead of first building a general semantic parser.
- **Approximate learned read representation:** finite group direction, quantized gain, query type, and bounded latent features. These select evidence but do not replace the evidence's exact identity.

A possible score family is

`s(t,i) = sum_l K_l(g_q,l^-1 g_i,l, gain_q,l, gain_i,l, type_t, type_i) + age/version terms`.

This is a specification family, not permission to materialize its entire Cartesian product. Gains should be factored, types few/shared, and every additional table charged. Learned numerical coefficients must respect the adopted legal-code contract. A calibrated gain-aware scalar kernel provides a reference; a nonlinear signed relation residual initially set to zero is a separable hypothesis.

The query should address offsets directly: for a small learned/selected support S, candidate cells are `g_q d` for d in S. Computing scores over all historical events and retaining eight at the end is not an eight-event retrieval cost. The support-selection and cell-decoding costs must be charged too.

Collisions are expected. Retain exact bounded postings or pointer records for ambiguous buckets, and declare overflow, refinement and NoRead behavior. Failure to visit a bucket does not establish absence of its records. Preserving a correct record does not establish selecting it correctly.

Any value transport acts on specified latent channels. Do not rotate or average a token ID and present it as exact copied content. Endpoint homomorphic transport alone can factor into a frame change; a claim about language-conditioned operations needs its own matched evidence.

No expert mixture or independent expert network is needed for this proposal. Address selection and shared typed operations remain within the intended architecture.

## 4. Exact finite-group read reduction

### Claim and assumptions

Let G be finite; let rho be a linear representation of G on a value space V; let the retained eligible records have keys k_i in G and values v_i in V. Let kappa:G -> nonnegative scalars have support S. All records in one bucket must share the same query-dependent kernel coefficient and eligibility. Define

`N(q) = sum_i kappa(q^-1 k_i) rho(q^-1 k_i) v_i`

`Z(q) = sum_i kappa(q^-1 k_i)`.

For each g define `M(g)=sum_{i:k_i=g} v_i` and `C(g)=#{i:k_i=g}`. Then

`N(q) = sum_{d in S} kappa(d) rho(d) M(qd)`

`Z(q) = sum_{d in S} kappa(d) C(qd)`.

When Z(q)>0, the normalized reader is R(q)=N(q)/Z(q). When Z(q)=0, return the declared NoRead state.

### Proof

Each event belongs to exactly one relative class d=q^-1 k_i, equivalently k_i=qd. Regroup the finite sum by that class. Since its coefficient and rho(d) are identical for the whole class and rho(d) is linear, factor them outside the within-class sum. Applying the same grouping to the denominator gives the count formula. No limit, approximation or statistical assumption is used.

The reduction holds for any finite group and valid representation. It is not a newly claimed theorem of geometric intelligence. It identifies a lawful execution transformation relevant to this proposal and closely related to established quantized-attention aggregation.

### Arithmetic realization and limits

A permutation representation makes rho(d) an index permutation. If kappa(d) uses bounded powers of two, the tested numerator and denominator need table reads, permutations, shifts and additions. That does not implement a complete no-float model: code generation, support selection, probability normalization, overflow, output emission and compiler instruction coverage still require implementation and validation.

For fixed support size S and value dimension D, the reduction's **read stage** costs O(|S|D) independently of the number of events. This excludes write cost, index construction, exact postings, learned encoders, multiple lane/type/gain partitions, and the vocabulary output head. It cannot be reported as whole-model constant cost.

Query-specific age/eligibility/version distinctions invalidate a naive common bucket unless represented by extra partitions, maintained summaries, or exact selected records. Pooling lane marginals does not preserve arbitrary cross-lane conjunctions.

### Frame statement

For common left change h, `(hq)^-1(hk)=q^-1 k`. This is a common-frame identity. It does not prove invariance under independent local frame changes or identify a semantic gauge symmetry of natural language. Quantization at zero and ties requires its own policy and does not inherit continuous equivariance automatically.

Furthermore,

`sum_i a_i rho(q^-1 k_i)v_i = rho(q)^-1 sum_i a_i rho(k_i)v_i`.

Pure endpoint transport can therefore be algebraically factored. Its usefulness may be computational or representational, but it is not by itself a new reasoning mechanism or nontrivial holonomy.

## 5. Executed probe and counterexample

`algebra_probe.py` independently constructs SL(2,F5) in the container, without a mapping to UOR's group IDs. It uses exact finite-field products and bounded integer value accumulators. It verifies:

- 120 elements and all 14,400 products;
- all 1,728,000 associativity cases;
- all 1,728,000 regular-representation composition cases;
- all 1,728,000 common-left-frame cases;
- 1,200 direct-versus-addressed numerator/denominator comparisons, two seeds, event counts 0/1/32/256/1024, eight support elements;
- 485 zero-denominator cases retained as NoRead;
- 256 insert/evict steps of a 32-event rolling window against fresh recomputation.

This is algebra/implementation evidence, not a learned GLM, native serving test, benchmark against dense attention quality, or M1 energy measurement. Normalization was not executed. Counts and sums were bounded well within int64 for the tested inputs only. The recorded runtime is a test-harness timing, not a model throughput result.

### Decisive binding obstruction

History A: Alice -> blue; Bob -> green.
History B: Alice -> green; Bob -> blue.

If both owners map to the same bucket and only the sum of blue/green value vectors and the count is kept, the two histories produce identical summaries. The correct answer to “What belongs to Alice?” differs. Any deterministic decoder of only the identical summaries gives the same result for both, so at least one answer must be wrong. Randomization cannot make both conditionally correct with certainty either.

The computation constructs this collision explicitly. The implication is not that all aggregate memory fails. It is that a summary may replace records only for a clearly defined class of queries for which it retains sufficient information. Source-specific bindings need exact records or a demonstrated binding-sensitive representation.

## 6. Three bounded research steps

These are proposed successor work, not retroactive changes to frozen acceptance criteria. Do not launch another heavy fit while another lab owns the slot.

### Step 1: a frozen-parent representation diagnostic

Question: which useful read distinctions are lost before the nonlinear relation learner is even trained?

Reuse current captured queries/keys and actual read/output observations where available. Compare original dot, norm-normalized dot, hard direction-only coding, and gain-aware coding on identical positions. An exact-gain arm is an attribution oracle, not a deployment win. Use a held-fixed bit allocation or clearly report unequal bits.

Inspect score margins, dense top-event retention, correct-source availability/ranking, downstream distribution change, and whole-response effects. Do not equate agreement with a wrong reference ranking with correct answering. No neural fit or new decoder is required for the first diagnostic.

Decision: if gain retention cannot preserve useful distinctions at the target cost, diagnose the remaining collision/bit-budget problem before fitting a nonlinear score. If it does, carry a calibrated finite baseline to the learning step. Stop this diagnostic when it distinguishes the identified alternatives; do not convert it into an unconstrained codebook sweep.

### Step 2: task-trained relation selection on the same base

Question: does a nonlinear, signed, typed relation kernel improve actual evidence use beyond an equally informed ordinary finite reader?

Keep the selected base, occurrence tape, objective population, context, budget and evaluator fixed. Begin with calibrated hard-forward legal codes; use a disclosed biased surrogate for the encoder. Add a zero-initialized relation residual or one bounded action, not new scoring, memory, recurrence and decoder all at once. Observe actual gradients through past writes as well as query formation.

Use an equal-information, comparable-bit/cost ordinary reader. A consistent relabeling of the group table is an isomorphism, not an ablation. A genuine comparison changes the operation class and trains both alternatives, with controlled parameter count and sufficient optimization opportunity.

Couple next-token/response learning to declared source-selection supervision or counterfactual contexts, rather than labels inferred from the model's own wrong ranking. Runtime entity/role/source decisions must come from text; oracle record labels are training/evaluation-only. Use withheld names, paraphrases, role reversal, same-value reassertions, corrected values, and intervening irrelevant turns. A small authored set is a diagnostic, not the full capability gate.

Freeze gains/thresholds/resources before fitting. Promote only if the predeclared useful-response benefit and retained-language requirements are met. Lower NLL, active codes, exact algebra or an isolated correct noun are not substitutes. If only the ordinary reader works, keep that functionality and retain the geometric negative at its proper scope.

### Step 3: compile the same accepted operator into real selected access

Question: can the accepted reader retain its behavior while actually fetching less information per token?

Start with existing legal integer tables and I2/I4 interfaces. Select relative support, address cells directly, preserve exact source identity through admitted records and report all collision/fallback costs. Compare dense admission and addressed admission using unchanged accepted weights and inference policy.

Audit actual emitted code after the known matcher repair, normalization, overflow, model/session reload and real complete outputs. Measure bytes and instructions for encoding, support selection, cell/posting traversal, value reads, updates, output head and sampling. Sparse event access does not finish D5 if dense learned parameter scans remain.

On a quiet named laptop, qualify elapsed cost and physical energy at comparable output quality. If refinement routinely scans everything, report it and stop claiming the selected-access advantage; do not hide the worst case behind an average top-k size.

## 7. Optional decision-preserving refinement

For q=qhat+eq and k=khat+ek, the triangle and Cauchy-Schwarz inequalities give

`|q.k - qhat.khat| <= ||eq|| ||khat|| + ||qhat|| ||ek|| + ||eq|| ||ek||`.

A candidate winner is certified for this reference score only when its lower bound exceeds every competing upper bound, including bounds on unvisited cells. An ambiguous comparison can fetch a bounded residual or exact record. The residual bit/storage/access cost must be charged.

This is an optional later optimization, not a tested UOR feature or a prerequisite for the first learning study. It preserves a chosen numerical decision, not factual correctness or whole-distribution equivalence. Without useful cell bounds it may require O(N) inspection; a strict budget must report uncertainty or a declared fallback rather than pretend to certify unseen records.

## 8. Literature and overlap

The following primary sources were inspected during this research. Their claims do not establish UOR compliance or success.

| Source | Useful implication | Boundary |
|---|---|---|
| Transformer-VQ, arXiv:2309.16354 | Quantized keys admit cached aggregate attention formulations. | Aggregation is prior art; the architecture is a transformer. |
| Norm-Explicit Quantization, arXiv:1911.04654 | Norm information can matter substantially to maximum-inner-product retrieval. | Does not identify the cause of #1438's negative. |
| Online Vector Quantized Attention, arXiv:2602.03922v3 | In the studied setting, online key/value dictionaries improve on static VQ, using sparse updates. | Equal-norm/isotropic assumptions enter its derivation; model scale and optimized implementation are limited; no D11 proof. |
| Sparse Delta Memory, arXiv:2607.07386v1 | Sparse reads and writes grow explicit recurrent memory without proportionate dense-state work; includes controlled language/recall evidence. | Uses continuous projections/gating and hybrid local attention; no automatic multiplier-free or laptop-frontier result. |
| DeltaProduct, arXiv:2502.10297 | Householder products provide structured noncommutative recurrent transitions with state-tracking evidence. | Does not override UOR's failed integrated-lane experiment or solve semantic interpretation. |
| T-MAC, arXiv:2407.00088 | Grouped low-bit table computation is a relevant implementation donor. | Mathematical dense maps and whole-model traffic remain separate from opcode avoidance. |
| AVQ-Attention, arXiv:2607.12789v1 | Adaptive code refinement allocates effort where it matters to attention. | Vision and GPU-oriented evidence, not UOR language or exact integer qualification. |

Primary references:
- https://arxiv.org/abs/2309.16354
- https://arxiv.org/abs/1911.04654
- https://arxiv.org/html/2602.03922v3
- https://arxiv.org/html/2607.07386v1
- https://arxiv.org/abs/2502.10297
- https://arxiv.org/abs/2407.00088
- https://arxiv.org/html/2607.12789v1

Repository references, with the analysis pin:
- https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/ROADMAP.md
- https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/docs/integration/fourth-lab-geometric-attention-2026-09-26.md
- https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/crates/uor-r4-training/src/geometric_read.rs
- https://github.com/UOR-Foundation/uor-r4/pull/1438
- https://github.com/UOR-Foundation/uor-r4/pull/1437
- https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5860922002 (mutable; used 04:55 UTC revision)

## 9. Novelty, verification and delivery status

This work establishes the displayed finite-sum reduction by derivation, verifies an independent bounded implementation, and exhibits explicit information-loss obstructions. It does not establish that the proposed learned model works. The existing repository design already contains many of the same mathematical cautions and candidate mechanisms.

A defensible research contribution would be the specific learned, task-useful, source-preserving finite operator and its verified low-traffic native realization, demonstrated against appropriate ordinary baselines. Calling its components new before that evidence would overstate both novelty and progress.

Primary-source retrieval, symbolic Wolfram checks and self-reviewed computations were used. No separate researcher panel completed; prior Codex specialist attempts in this conversation were quota-blocked. This packet contains no new trained checkpoint and makes no claim to have surveyed every archive item or all global prior art.

The Visualize direct-mode specification was not exposed by the available tools in this session. No inline app widget is claimed to have rendered.

## 10. Reproduction

Requires Python and NumPy already installed in the analysis environment. This is research-side code, not a proposed repository runtime dependency.

```sh
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 python algebra_probe.py
```

`algebra_probe_results.json` records domain, cases, seeds, versions, runtime, source hash and limitations. A repeat changes its timestamp and runtime; scientific fields should agree. `verification.json` records the repeat comparison. `MANIFEST.json` binds the delivered files.
