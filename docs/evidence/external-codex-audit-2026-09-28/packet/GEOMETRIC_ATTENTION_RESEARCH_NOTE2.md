# UOR-R4: a research route to useful geometric attention

Date: 28 September 2026. Status: source-based diagnosis and proposed research, not an implemented model or a capability promotion.

## Executive conclusion

Retain the native recurrent programme, exact-memory work, Rust learning infrastructure, integer runtime and cross-lab integration plan. Change the attention research question from “which geometric score can replace the current dot score?” to “which task-trained, information-preserving geometric representation can be executed directly under the native contract at competitive total cost?”

A geometrically represented, table-executed attention score is constructively possible. A useful whole GLM remains an empirical learning and integration question. Frontier equivalence on a laptop is neither established by the current results nor disproved by the failed small experiments. Attention alone is not the complete solution: knowledge capacity, data, discourse learning, reasoning, memory access and the complete serving path remain necessary.

The strongest immediately actionable scientific issue is that the failed finite reader discarded each four-dimensional block's magnitude, replaced the scoring function, and introduced a randomly initialized nonlinear scorer together. Its triggered normalization control was deferred. The experiment validly rejects that submitted configuration at its frozen scope; it does not identify the cause or reject geometric attention generally. A new fit is not justified simply by this observation. First use a bounded, same-input decomposition to determine whether the missing information matters on the actual artifact.

## Source and execution boundaries

Current source reviewed: main `15d2ce3fac9f66ddcc945a8097e5c6216352c453`, with the live lab board version updated at 04:55 UTC and subsequent branch-publication verification. Live issue records supersede historical scheduling. The September 11 attachment is historical context, not a current policy or model snapshot.

The work combined live GitHub source and experiment records, Remote Desktop Commander inspection/publication, SciSpace and Consensus literature discovery, primary paper review, and explicit symbolic checks through Wolfram and SymPy. No independent model-agent panel completed this review. Codex's quota limitation remains recorded; no paid substitute was launched. No UOR training or model evaluation was performed for this note.

The mathematical checks are reproducible with `python verify_attention_math.py`, requiring SymPy. They establish the displayed algebra and explicit counterexamples, not a trained-model result. The original Wolfram evaluation separately returned zero for the quaternion identity residual and False for a possible reversal under the stated margin condition.

## 1. Roadmap diagnosis

### Retain

- D11's native, multiplier-free, transformerless serving contract. Offline Rust training can use floating point and matrix multiplication. Dense at-most-four-bit additive maps are explicitly interim, not completion of D5.
- One selected recurrent-primary base and one native runtime/frontend. Finish the frozen base comparison; do not replace it with a reads-only transformer under a geometric name.
- Exact event, payload, owner, version and occurrence identity where implemented. Learned selection and exact storage solve different problems.
- The existing training-to-hard-artifact bridge and fixed #1433 study. The live board now records owner transfer of its execution to Claude; older prohibitions on another lab resuming it are superseded by that explicit transfer.
- Equal-cost geometric versus ordinary indexing, including the complete query, write, traversal and payload-read costs.

### Correct

**Scope of geometric negatives.** ROADMAP's language about geometry failing as a continuous score or mixer is broader than the causal attribution supported by individual experiments. Preserve the negatives, but name the parameterization, representation, optimization conditions and tested role. Do not infer a universal theorem from those outcomes.

**Capacity attribution.** A small model trailing a larger reference does not, by itself, identify capacity as the sole cause. Objective, population, optimizer, architecture and exposure can interact. The capacity-matched comparison is valuable precisely because the attribution was previously unresolved.

**Base versus product.** The stack's code-loss gate can select a research base without qualifying conversation. Its code BPE and the dialogue line's #1017 tokenizer are different identities. Same vocabulary size is not token compatibility. Migration requires a tokenizer/data/export decision; old dialogue weights and panels do not transfer automatically.

**State-tracking scope.** The fresh reflection-pair replication achieves exact A5 tracking but fails the per-seed language-loss rule: seed 4 costs +0.0688 nats, above 0.05. No lane type is retained in the stack under the declared rule. Exact finite-state execution survives as a reusable result, not natural-language reasoning. Conditional use of an exact operator might eventually be useful, but requires learned invocation and a new justified study; it is not permission to relaunch the failed lane study.

**Admission versus relevance.** Preserving the dense reader's favorite events can preserve its mistakes. The oracle reranking result on five exposed failures identifies a local opportunity, not general learnability. Measure event admission, semantic ranking, value use and emitted answers independently.

**Event sparsity versus parameter sparsity.** A sparse event index does not eliminate full vocabulary-head or dense projection reads. Report the remaining whole-token memory traffic.

## 2. What the failed finite reader actually does

The pinned `geometric_read.rs` already implements hard forward execution with a smooth straight-through backward path and language training. It is not missing a gradient in principle, nor does it need a newly invented relative-quaternion MLP: it already has one.

The relevant transformation is:

1. Partition a 64-coordinate query/key into sixteen four-dimensional blocks.
2. Normalize every query and key block to unit length.
3. Quantize each to one of 120 signed binary-icosahedral roots.
4. Form the exact relative group element `inverse(query_code) * key_code`.
5. Score each relative quaternion with a learned 4→8→1 tanh network and sum the block scores.

The magnitude discarded at step 2 is not supplied to this scorer. Antipodes are correctly kept distinct. Hard-code usage across all 120 elements does not establish preservation of useful distinctions.

The executed screen reports:

| Item | Same-dose baseline | Finite reader |
|---|---:|---:|
| Comparison-tail read NLL | 1.984752805 | 2.004471395 |
| Complete authored source answers | 27/32 | 17/32 |
| Sampled acceptable prose | 0/5 | 0/5 |

The initial scoring disturbance is approximately +0.4 nats. The cost ratio is 1.84, above the declared 1.5 threshold. The result is HARM as registered. The norm-controlled dot arm's trigger fired but the arm was deferred under shared resource accounting. Those facts justify a narrower attribution question, not retrospective promotion or an automatic repair fit.

## 3. A concrete information-loss obstruction

Write a nonzero four-dimensional query and key as real quaternions:

\[
q=r_q u,\qquad k=r_k v,\qquad \|u\|=\|v\|=1.
\]

Hamilton multiplication gives

\[
\boxed{q^\top k=r_qr_k\operatorname{Re}(u^{-1}v).}
\]

This follows because the real component of \(\bar q k\) is the Euclidean dot product. The identity was checked by symbolic expansion independently in Wolfram and SymPy.

A unit-only encoding cannot recover the discarded gains. Let \(e=(1,0,0,0)\) and query \(q=e\). Compare two cases with the same candidate identities, positions and other metadata:

- Case A: \(k_1=e, k_2=2e\), so dot scores are \((1,2)\).
- Case B: \(k_1=2e, k_2=e\), so dot scores are \((2,1)\).

After unit normalization the two encoded cases are identical, but their unique correct dot-product maximizers differ. No deterministic downstream score that only sees those unit encodings and unchanged metadata can reproduce both rankings. For the implementation, restrict the witness to nonzero magnitudes above its normalization threshold; floating boundary/tie artifacts are not the information source.

This proves a representational limitation. It does **not** prove that UOR's measured ten-answer regression was caused by norm loss. Training might learn to represent some lost distinctions in other coordinates. The actual artifact must decide whether such adaptation occurred or whether the discarded magnitudes were useful.

Magnitude also controls attention distribution sharpness. Preserving rank alone is insufficient: logits \((1,0)\) and \((10,0)\) have identical ordering but first-token softmax mass approximately 0.73106 and 0.99995. A reader can retain top-1 while substantially changing its value mixture or copying distribution.

## 4. Proposed mechanism: task-trained gain–shape geometric attention

This is a proposed synthesis of established ingredients, not a claim of global novelty. Its distinguishing research target is the conjunction of task-useful selection, information-preserving geometric coding, and one native hard executable inside the existing recurrent model.

### 4.1 Preserve exact payloads; compress retrieval coordinates

A memory event should retain an exact event/occurrence identity, its payload or span reference, and supported owner/version/provenance fields. Its geometric key is a lossy retrieval representation, not the canonical identity or a replacement for the fact.

A suggested logical record is:

```text
Event {
  exact_occurrence_id,
  payload_or_span_reference,
  supported_owner_relation_version_metadata,
  geometric_key_blocks,
  magnitude_codes
}
```

If owner or relation is inferred from text, it remains a learned and fallible decision. Do not supply answer-bearing oracle metadata at inference or hide parsing errors behind exact storage.

### 4.2 Start with a controlled geometric re-expression

For each block, retain a direction code \(g_b\) and magnitude code \(a_b\). The simplest baseline score is

\[
s_0(q,k_j)=\frac{1}{\sqrt d}\sum_b
\widehat r_{q,b}\widehat r_{j,b}
\operatorname{Re}(g_{q,b}^{-1}g_{j,b})
+\mathrm{age}(t-j),
\]

with the existing NoRead branch kept separately and unchanged. The hats denote quantized gains. This equals dot scoring for the represented vectors before table/output rounding, not for arbitrary original vectors.

Initialize from this known function instead of simultaneously discarding gain and introducing a random scorer. Measure its zero-update discrepancy. Only then decide whether a learned residual relation score is justified. Any residual must satisfy the adopted low-bit learned-operator contract, rather than becoming an unrestricted floating-derived learned lookup that silently changes policy.

The binary icosahedral group provides exact relative composition for the H4 candidate. E8 roots are a different candidate codebook, not a group with the same multiplication law. Compare H4 against appropriate learned/ordinary codebooks at matched total budget rather than assuming four dimensions are optimal.

### 4.3 Make the complete numerical computation native

A fixed table can store

\[
T[a,b,g]=Q_{\mathrm{score}}\left(r_a r_b\operatorname{Re}(g)/\sqrt d\right),
\]

where the gains, group and score quantizer are explicitly bound to the artifact. Runtime scoring then uses exact group-code composition, lookup and accumulation. No runtime trigonometry or floating-point product is needed for this **score subproblem**.

Illustration only: sixteen magnitude bins on each side and 120 relative group elements give 30,720 entries, or 61,440 bytes at signed 16 bits. That excludes group tables, classification, gains, metadata, candidates, values and all model weights. Padding each 120-element row to a 128-element power-of-two stride would make that illustrative score table 65,536 bytes instead. Neither sixteen bins nor signed-16 scores are qualified; range, saturation and error must be selected from the actual training population. Common scales permit sharing; per-head scales or learned residuals change storage.

The input projections, normalization/classification, score scaling, integer softmax, normalization of masses, weighted value transport, copy mixture and sampler must all obey the same serving boundary. An integer lookup replacing just the score does not make the model compliant. Dense projections and output-head access remain explicit interim costs.

If symmetric query quantization causes most error, an asymmetric variant can keep a higher-resolution integer query while retaining compressed keys. Its query-to-codebook calculation must itself use permitted arithmetic and be included in cost. Full-precision query matmul hidden in table construction is not acceptable.

### 4.4 Train for useful selection, not only vector reconstruction

Retain language learning and complete-prefix dialogue supervision. Add a bounded, source-grounded ranking objective only where support labels can be obtained lawfully during training:

\[
\mathcal L=
\mathcal L_{\mathrm{language}}
+\lambda\mathcal L_{\mathrm{support\ ranking}}
+\mu\mathcal L_{\mathrm{hard\ bridge}}.
\]

One possible support-ranking term is a margin loss

\[
\max\{0,m-(\widehat s_+-\widehat s_-)\},
\]

where the positive event actually supports the requested answer and the negative is a causally available, confusable event. Examples include the same owner with an outdated value, the same value belonging to someone else, subject/object reversal, an irrelevant recent mention, and an absent relation. Counterfactual answer labels must change when the underlying fact changes. Values, names, syntax and distances should be held out in combinations, not merely reshuffled inside memorized templates.

Teacher distributions can preserve useful existing behavior, but an incorrect teacher ranking must not be the only objective for a repair. Exact-copy answers need exact payload retention; soft mixtures of linguistic state are a separate operation. Preserve first/current/previous-record semantics, including same-value reassertions.

The hard code and selection path must be exposed during learning. The existing straight-through estimator is a biased surrogate, not a proof that gradients optimize discrete decisions. Verify actual hard-decision movement and completed responses. Account for gradients to omitted candidates; hard top-k without an admission-learning mechanism can leave excluded useful events untrained. A bounded exploration or soft-admission training surrogate is a design choice to compare, not an implicit serving fallback.

### 4.5 Use decision error budgets

Let true scores be \(s_j\), approximations \(\widehat s_j\), and \(\max_j|s_j-\widehat s_j|\le\epsilon\). If the true top-two margin is \(\Delta>2\epsilon\), the argmax is unchanged. The proof is immediate: the winning score can decrease by at most \(\epsilon\), and a competitor can increase by at most \(\epsilon\).

A useful vector-error decomposition is

\[
|q^\top k-\widehat q^\top\widehat k|
\le \|q\|\,\|k-\widehat k\|
+\|\widehat k\|\,\|q-\widehat q\|.
\]

Sum the block bounds and add table/accumulation rounding error. Use this to diagnose which queries need better gain or direction precision. Average vector reconstruction error and full codebook occupancy can conceal a few answer-changing near-ties.

The argmax guarantee requires that the relevant candidates are admitted. It does not certify omitted events, truth of the underlying ranking, the softmax mixture or a whole generated response. A variable-precision or additional-probe policy can be explored within a declared hard budget; it must report unresolved cases when the budget cannot certify a decision. Do not claim worst-case constant-time exact retrieval for arbitrary memories.

## 5. A single-step bound connecting admission, quantization and value use

This section is an elementary deduction, not a model-quality result.

Let a fixed full read have softmax weights \(p_j\) and values \(v_j\) with \(\|v_j\|\le R\). Let the admitted set have omitted mass \(\eta<1\). Renormalize the original weights over that set. The resulting output differs from the full output by at most \(2R\eta\), because the full output is a convex combination of admitted and omitted conditional means.

Within the admitted set, suppose logit error is at most \(\epsilon\). The approximate/exact probability ratio is between \(e^{-2\epsilon}\) and \(e^{2\epsilon}\), since each exponent and the normalizing sum change by at most a factor \(e^{\epsilon}\). Hence a conservative L1 distribution bound is \(\min\{2,e^{2\epsilon}-1\}\).

If each retained value is additionally approximated with error at most \(\epsilon_v\), a one-step output bound is

\[
\|o-\widehat o\|
\le 2R\eta + R\min\{2,e^{2\epsilon}-1\}+\epsilon_v.
\]

This connects three separately measurable losses: missing support/mass, inaccurate scores, and inaccurate values. It requires fixed inputs and the stated bounds, and it does not identify whether the full model's answer was correct. Recurrent propagation and free-running generation can amplify perturbations. The bound is a diagnostic and a candidate optimization target, not a universal long-conversation certificate.

## 6. Three bounded research moves

### Move A: localize information loss without another fit

Use the retained base and available fixed query/key/value traces. Compare, on identical positions:

1. Original dot scores.
2. Per-block unit-normalized dot scores, without finite coding or a learned new scorer.
3. Finite direction-only cosine scores.
4. Finite gain–shape scores at a declared equal-cost representation.

Preserve age bias, NoRead, admissible history, values and emission conditions. Compare score margins, attention distributions, value outputs and known source decisions. A limited saved-model generation comparison is separate from same-input trace analysis. Record clipping, zero blocks and near-ties.

**Decision:** if normalization or finite coding explains a material, predeclared portion of the relevant decision changes, test the corresponding repair. If it does not, stop treating norm restoration as the explanation and pursue ranking/representation learning. If the original ranking already loses the required relation, score fidelity alone cannot solve that case. This is a narrower continuation of a deferred causal question, not an unregistered repeat of the HARM fit.

### Move B: one utility-trained geometric reader and a matched ordinary control

Only after Move A supplies a justified representation and initialization, insert the gain–shape read into the selected existing learner. Match data, tokenizer, parameter/access budget, magnitude information, ordinary comparator tuning effort and training exposure. Use hard-forward execution and explicit bridge checks. Select endpoints, seeds, error bars and resource limits prospectively; do not improvise them from an early win.

Primary success is improved or retained completed conditional responses and support selection without unacceptable language regression. Use natural text plus held-out entity/role/version contrasts. A better geometric codebook should beat or match the strongest ordinary alternative at comparable **total** cost. If both improve equally, retain the useful learning result without claiming geometric advantage. If neither improves, the address representation or training signal remains insufficient. Do not add a fresh dose automatically.

### Move C: one artifact, native execution and total cost

Export/reload the selected reader with the chosen base, common tokenizer and dialogue protocol. Run the already-approved product milestone through the actual frontend, not a synthetic replacement. Include context exhaustion, save/reload, new facts, corrections, contradictory or absent facts, instruction following and complete replies.

Measure source-bound release instruction coverage, per-token parameter/event traffic, query/index/write costs, peak memory, complete generation latency and physical energy under matched outputs/quality. Keep the existing <=0.02-nat bridge rule where applicable; do not mistake it for a conversation-quality gate. Preserve the unfavorable historical energy result and its different-checkpoint/instrumentation caveats.

Resource design must reuse existing traces/builds, project the local cost and use the shared queue. No new model fit, infrastructure deployment, external compute or package installation is authorized by this note.

## 7. Primary-literature map and novelty boundary

| Source | Relevant contribution | Boundary for UOR |
|---|---|---|
| Norm-Explicit Quantization, arXiv:1911.04654v2 | Magnitude and directional errors have different effects in inner-product retrieval. | Does not establish that UOR's answer regression was caused by norms. |
| RepCONC, arXiv:2110.05789v1 | Joint representation and quantizer learning for task-effective compact retrieval. | Dense retrieval evidence, not a multiplier-free language model. |
| Spotlight Attention, arXiv:2508.19740v4 | Query/key asymmetry and ranking-trained nonlinear hashing for attention retrieval. | Uses a transformer teacher and dense-score relevance; may imitate its errors. |
| LOOKAT, arXiv:2601.10155v1 | Explicit product-quantized key attention with query-conditioned tables. | Small prototype; table construction still uses full-precision query/codebook products. Its claim that ranking alone suffices for softmax is not valid in general. |
| Scalable MatMul-free Language Modeling, arXiv:2406.02528v7 | Recurrent language learning with ternary maps at substantially larger scale. | Elementwise products and dense parameter access remain; hardware and training do not establish UOR's D11/D5 or frontier parity. |
| DeltaProduct, arXiv:2502.10297v3 | Richer noncommutative recurrent updates through products of Householder transformations. | A strong geometric ordinary control, not evidence of a quaternion-specific advantage. |
| BASED, ICML 2024 | Explicit recall/throughput tradeoffs in recurrent/linear-attention language modelling. | Does not supply lossless arbitrary memory in a fixed small state. |
| T-MAC, arXiv:2407.00088 | Grouped lookup computation for efficient low-bit maps. | An arithmetic implementation method, not sparse parameter access or learned semantics. |

Search scope: live project source/history, SciSpace and Consensus discovery, arXiv and primary publication pages, through the available September 28, 2026 sources. The proposed combination has substantial prior art. No patent search or exhaustive global novelty search was conducted. The credible research contribution would be a demonstrated integration of useful learned geometric access, exact identity-aware memory, native hard execution and competitive total cost, not claiming invention of product quantization, quaternion algebra or lookup attention.

## 8. Feasibility judgment

**Constructively feasible:** a native geometric score and bounded selected read. The finite table construction shows how its mathematical score can be represented without runtime floating-point arithmetic or hardware multiplication, within explicitly chosen quantization ranges.

**Plausible, not demonstrated here:** a useful transformerless GLM combining learned language, exact episodic memory, geometric access and integer execution. External recurrent/low-bit work supports taking this route seriously, without qualifying this implementation.

**Unestablished:** frontier-equivalent breadth, reasoning and knowledge on the target laptop at favorable energy. No current result is a scaling law or end-to-end demonstration at that objective. Running inference on a laptop and training a frontier model entirely on the laptop are distinct requirements; the former does not logically require the latter. No new paid or external training is proposed without separate authorization.

Geometric attention is a necessary research target for this architecture, not a substitute for data, learning capacity, instruction training, long-context semantics or strong evaluation. The roadmap should measure those remaining gaps explicitly rather than promise that a successful index completes them.

## 9. Codex branch publication: completed

At the owner's request, inspected `codex/h4-integer-classifier-20260927` in `/Users/casey.allard/.codex/worktrees/h4-integer-classifier/uor-r4`. The worktree was clean and one commit ahead of origin, not uncommitted.

Published the pre-existing commit unchanged:

```text
12ae89e6338fd4a2e87b6e1de7afc8ad82966d1b
  -> 852c1c67ed7036589f2d9f0a6ffa9ff917858b5a
```

The normal push succeeded. `git ls-remote` and GitHub PR #1435 both returned the new exact head. A follow-up local status at 05:39:35 UTC was clean and synchronized. `git diff --check HEAD^ HEAD` passed. No source edits, rebase, force push, merge, new build or model execution occurred. The PR remains draft; current integration conflicts and applicable checks are not resolved merely by publication. The parked table consumer was not revived. Publication comment: #1435, comment 5864053985.

## Source locators

- Roadmap: https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/ROADMAP.md
- Live board: https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5860922002
- Fresh B1 replication: https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863591275
- Reader source: https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/crates/uor-r4-training/src/geometric_read.rs
- Reader result: https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/docs/integration/geometric-read-result-2026-09-27.md
- Branch publication: https://github.com/UOR-Foundation/uor-r4/pull/1435#issuecomment-5864053985
- NEQ: https://arxiv.org/html/1911.04654v2
- RepCONC: https://arxiv.org/html/2110.05789v1
- Spotlight: https://arxiv.org/html/2508.19740v4
- LOOKAT: https://arxiv.org/html/2601.10155v1
- MatMul-free LM: https://arxiv.org/html/2406.02528v7
- DeltaProduct: https://arxiv.org/html/2502.10297v3
- BASED: https://proceedings.mlr.press/v235/arora24a.html
- T-MAC: https://arxiv.org/abs/2407.00088
