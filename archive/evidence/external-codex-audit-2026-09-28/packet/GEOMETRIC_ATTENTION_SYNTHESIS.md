# UOR-R4: roadmap diagnosis and a geometric-attention research route

**Date:** 2026-09-28. **Evidence status:** research synthesis, exact finite checks, symbolic checks, and synthetic numerical diagnostics. No new language model was trained, no serving artifact was promoted, and no repository source or other lab job was changed.

**Repository source pin:** `15d2ce3fac9f66ddcc945a8097e5c6216352c453` (#1449). **Latest board read:** revision 2026-09-28 04:55:34 UTC. **Additional local T2 observation:** unpublished clean source `b0f70c637290cc694f78218bc53b7c336f671fa2`, inspected at approximately 05:20–05:23 UTC. Local evidence and a GitHub source snapshot are not interchangeable.

## 1. Verdict

A useful native geometric language model remains a reasonable research objective. The present evidence does not establish a path from the retained small models to frontier-equivalent laptop capability. The arithmetic constraints do not supply semantics, knowledge, or a scaling law by themselves.

The recommended change is not a new backbone. It is to connect **the representation that learns relevance, the geometric relation used to score it, and the addresses used to fetch it**. Preserve the working Rust learning, exact event identity, integer runtime, and comparison infrastructure. Rethink irreversible unit normalization, abrupt replacement of an already learned score, and calling a compressed full scan a sparse index.

Proposed research hypothesis:

> A task-trained finite-group read can use the same gain-preserving relational score to produce candidate addresses and rank retained evidence. This may give geometry an actual computational role instead of adding a geometric score to an otherwise dense read.

This is a candidate synthesis, not a priority claim or a demonstrated language improvement. Existing group kernels, vector quantization, residual refinement, and threshold retrieval are substantial prior art.

## 2. Current roadmap: what is supported

The current board reports that the fresh reflection-pair replication also failed its pre-registered language-cost gate: exact A5 tracking in all three seeds, but +0.069 nats text loss in seed 4. No lane type is retained in the stack; finite-group state remains Stage A evidence and Stage C is parked. This does not refute geometric addressing or all non-commutative state mechanisms.

The capacity-matched recurrence-primary base comparison remains undecided in the last board read. Its intermediate values are not its final acceptance result. The existing within-0.03-nat rule remains in force. The owner has transferred execution of #1433 to Claude because Codex is quota-blocked; the fit and endpoint evaluations are unchanged. This supersedes the previous no-takeover advice only at that explicitly authorized execution boundary.

The finite reader in #1438 was a substantive learned geometric operator, not geometry added only to telemetry. It used sixteen signed 2I code lanes, all four coordinates of each relative quaternion, a nonlinear 4→8→1 score per lane, and hard-forward/smooth-backward learning through the existing language graph. It replaced the learned dot score on an existing checkpoint, normalized every four-coordinate query and key lane, and introduced 784 parameters. At its fixed dose it worsened comparison NLL by 0.019719, reduced source answers from the dose-matched control's 27/32 to 17/32, and exceeded its offline training cost gate. The norm/capacity attribution control remained unrun. That is a negative result for this configuration, not a clean test of non-commutativity independently of norm loss, quantization, initialization, and adaptation.

The original September 26 design explicitly anticipated norm loss and the missing control. Restoring that causal question is not the discovery of a previously unknown flaw or authorization for an automatic repair-fit.

### New local T2 finding

At local `b0f70c6`, `joint-addressing-contest.rs:338–344` builds a query table, scores every supplied event code, and sorts the scores. Lines 831–833 supply the complete previous-event slice. However, `decode_spec` at lines 366–395 charges per-event score work multiplied by the retained size `s`, rather than by the inspected candidate count; the report definition repeats that estimate.

Thus the current implementation is a **compressed full scan followed by selection**, not yet an implementation of sublinear candidate access. Compression can still reduce traffic versus full keys and its recall measurements remain useful. The stated decode estimate cannot be used as the executed query cost. At 255 prior events and s=16, the per-event score term uses 255 events, not 16; this is not a claim of 15.9× total runtime because query-table construction, sorting, and other work have separate costs.

The local `full-2` JSON, SHA-256 `e277fb7d198308472e19d32e1278002c14a3112baa167c148374e88b4e6398fa`, contains 232,560 all-event positions and 51,680 selected-read positions. Illustrative s=16 top-event retention is 0.96814 H4, 0.96958 rotated H4, and 0.98127 kmeans120 seed 1 on the full population. On selected-read positions these are 0.98355, 0.98905, and 0.99621. These are reported admission diagnostics, not my rerun, a completed cost qualification, or a final adoption decision. The relevant denominator must be resolved under the study's rule before interpreting its gate.

## 3. What current external research changes

- **Transformer-VQ** (Lingle, 2023) shows that quantized-key attention has useful caching/algebraic formulations. It remains a transformer and does not satisfy D11 merely by replacing keys with codes.
- **Online Vector Quantized Attention** (Alonso, Figliolia, Millidge, 2026) argues that static pretrained key dictionaries can fail recall, then updates key/value dictionaries online with sparse updates. It separates state size from update cost. Its actual equations still include dense query–dictionary products; it is not a drop-in multiplier-free runtime. Averaged value dictionaries also cannot replace exact occurrence/version records without losing those distinctions.
- **AVQ-Attention** (2026) refines important parent codes using prelearned child codes. Adaptive precision is therefore not a new idea here. Its demonstrated domain is vision; language quality and M1 execution remain unproven for UOR.
- **Norm-Explicit Quantization** and **ScaNN's anisotropic quantization** show why vector reconstruction error and inner-product ranking fidelity are different objectives. They motivate preserving gain and weighting errors by actual query utility, not claiming gain is automatically a semantic confidence variable.
- **DeltaProduct** uses products of generalized Householder transformations. An ordinary reflection-pair control is still geometric mathematics. The scientifically meaningful distinction is the operation class, parameter sharing, learning behavior, and cost, not geometry versus non-geometry as a binary label.
- **Scalable MatMul-free Language Modeling** demonstrates language modeling at up to 2.7B parameters, but retains elementwise products. It supports investigating non-transformer low-bit models; it does not prove D11's stricter no-multiplier contract or frontier-equivalent quality on an M1.
- **Product-key memory** supplies a precedent for large parameter capacity with selected access. It does not turn sparse event attention into sparse access to every model parameter.
- **Fagin–Lotem–Naor threshold retrieval** supplies the relevant retrieval principle for combining additive factor scores. Its assumptions about sorted/random access matter; it does not guarantee every query touches few records.
- **T-MAC** is relevant to grouped lookup execution, but its measured energy results cannot be inherited by UOR's different shapes and model path.

## 4. Executed mathematical and numerical results

### Exact geometric carrier

`geometric_attention_probe.py` independently constructs the 120 binary-icosahedral roots over `(Z + Z phi)/2`, phi²=phi+1. This is not a replacement for the repository's canonical ordering. It checked all 14,400 products, all 1,728,000 associativity triples, and all 1,728,000 shared-left-frame relative-code triples. These finite assertions passed.

For unit h, relative codes satisfy `(h q)^-1(h k)=q^-1 k`. A common right action instead conjugates the relative result. Consequently, an arbitrary full-relative-coordinate score is not automatically invariant under every choice of frame. A zero-to-identity quantizer cannot be equivariant under the transitive group action; ties and neutral states need explicit treatment.

For quaternions, `Re(conj(q) k)=q·k`. Wolfram's symbolic expansion independently returned zero for this difference and for the two stated frame identities. An initial self-referential variable definition failed; the corrected symbolic computation returned the identities. This does not establish language relevance or a new inductive bias. The repository already notes this equivalence.

Taking products of more elements of the same group does not create a finer codebook: the result remains one of the same 120 elements. Ordered products matter, but increased sequence length is not increased single-lane representational resolution. Additional independent lanes, gains, or residual information are different constructions and must be charged.

### A precise norm-loss counterexample

There is a stronger fixed-encoder obstruction before considering any particular nonlinear score. Positive rescalings of a nonzero block have the same nearest unit-root code. Thus k and 2k produce identical group codes, while q·k and q·(2k) differ whenever q·k is nonzero. No function of those group codes alone can reproduce every donor dot score. This applies to a full nonlinear relative-coordinate score, not just to cosine. Adaptation can potentially re-encode information into directions, but that is additional learning rather than a behavior-preserving conversion.

Let phi=(1+sqrt(5))/2, q=(1,0,0,0), k_A=(phi,1,phi-1,0), and k_B=(1,0,0,0). Then ||k_A||=2, ||k_B||=1, and k_A/2 is already a 2I root.

Original scores: q·k_A=phi>1=q·k_B.

After unit normalization: q·(k_A/2)=phi/2<1=q·k_B.

No angular code-rounding error is needed for this reversal. This proves that the representation change can discard a donor ranking distinction. It does not prove that this was the dominant cause of #1438's measured failure, which used a learned nonlinear score rather than cosine alone.

### Synthetic diagnostic, not a language result

The reproducible probe uses 192 queries, 128 keys, 16 four-coordinate blocks, Gaussian directions with variable lognormal gains, and seed 20260928. It compares top-one ranking agreement with the original full dot scorer:

| Representation | Matching top-one query rankings |
|---|---:|
| Unit 2I codes, magnitudes discarded | 25/192 (13.02%) |
| 2I codes plus exact real gains, an oracle | 121/192 (63.02%) |
| 2I codes plus four-bit dyadic gain exponents | 112/192 (58.33%) |

These are not equal-storage alternatives. The exact-gain oracle is not a finite-bit serving design. The distribution is not a saved UOR query/key distribution and should not be compared numerically to T2's asymmetric-key results.

A conservative score-error certificate for q=qhat+e_q and k_i=khat_i+e_i is

`B_i = ||e_q|| ||khat_i|| + ||qhat|| ||e_i|| + ||e_q|| ||e_i||`.

It follows by expanding the dot product and applying Cauchy–Schwarz. A winner a is certified relative to this dot scorer when `shat_a-B_a > max_{i!=a}(shat_i+B_i)`.

The important negative: the coarse gain+shape representation certified **0/192** queries. Extra coordinate residual grids increased certificate coverage, eventually to 164/192 at step 1/64, with no false certificate, but used more precision and still scanned every candidate. This is not evidence for cheap certified attention. It shows exactly where an appealing theoretical bound is too loose to serve as the proposed efficiency argument.

## 5. Proposed mechanism: compile the learned score into its own addresses

### 5.1 Keep information channels distinct

Represent each query/key block as

`x_l = a_l c(g_l) + r_l`, with `g_l in 2I`.

The group code carries direction and signed relational structure; a bounded gain code preserves selected magnitude information; a bounded residual code preserves decision-critical detail where needed. Exact token/occurrence/owner/version identity remains outside this approximate representation. An address collision must not merge two distinct records.

Start from the current accepted model path. Gain and residual precision are selected from actual score/output sensitivity, with ordinary quantizers at the same total bits and traffic as controls. Do not begin by assuming four dimensions or H4 is universally optimal. No arbitrary increase in bits is an efficiency win.

### 5.2 Separate conversion from new learned behavior

For a coarse geometric conversion of a donor dot score, the block term is

`a_q a_k Re(c(g_q^-1 g_k))`.

A finite table of gain bins and relative codes can compute a quantized version through lookup/add/shift. This is a conversion of an ordinary mathematical bilinear score, not a new attention principle. Products of runtime values, normalization, encoding, and value accumulation still need explicit integer implementations and bounds.

Then introduce a zero-started learned correction only under a separate study:

`s(q,k) = sum_l [B_l(a_q,l,a_k,l,g_q,l^-1 g_k,l) + Delta_l(g_q,l^-1 g_k,l)] + age + context_bias`.

Keep NoRead, exact causal masks, the value/copy interface and the original score scale aligned. Do not add a new recurrence, new output head, admission pruning and a new parser to the same comparison.

The failed reader already has a nonlinear relative score and a straight-through gradient. The proposed change is not 'add learning.' It is to measure information loss first, warm-start a calibrated score, train through the intended hard representation, and separately test whether the learned relational correction improves actual output. A straight-through estimator remains biased.

### 5.3 The useful algebraic coupling

For a single factor with fixed gain/context bins, write

`s(g_q,g_k)=F(g_q^-1 g_k)`.

Sort the finite relative offsets delta by F(delta) offline. For a query code g_q, the corresponding key-code order is simply

`g_k = g_q delta`.

Proof: `g_q^-1(g_q delta)=delta`. Thus a learned preference over relations can be compiled into the order of addresses to inspect. The query need not rescore every event to discover those single-factor code cells. The experiment `group_address_probe.py` verified this ordering identity over all 120 queries and 120 key codes with distinct score values.

Maintain postings of exact event IDs under coded cells. If a code cell contains several records, selection still preserves their separate occurrence/version identities. Gain bins condition which relation list is used. Any additional context selector would need an existing shared typed-state interpretation; this proposal does not authorize MoE or a new expert gate. Any contextual correction that varies inside a cell must have a valid bound or be evaluated later; it cannot be omitted from correctness claims.

This construction is related to finite-group correlation and existing inverted-index retrieval. The proposed UOR contribution would be its successfully learned language representation and complete native implementation, not the elementary group identity itself.

### 5.4 Multiple factors require real retrieval logic

Taking a fixed number of best cells independently from each factor is not safe. Three records with factor scores A=(6,6), B=(10,0), C=(0,10) have A as the best total score, but the union of the two per-factor top-one lists excludes A.

A threshold procedure may instead combine ordered factor streams. When all unseen objects have factor scores at most u_l, their summed score is at most sum_l u_l, plus valid bounds on the other terms. Already-seen objects need complete scores. This can stop early on favorable distributions; the worst case still touches all candidates.

Geometric index creation, posting intersections, coding, table construction, sorting, metadata, and fallback/refinement traffic must be included in the access cost. A fixed runtime cap cannot simultaneously guarantee exact arbitrary top-k retrieval unless the representation/index supplies the needed certificate. Budget exhaustion must be reported as approximation, not hidden by an uncharged dense fallback.

### 5.5 Top-one preservation is not sufficient for soft attention

Suppose the omitted m events have scores at most U and every value has norm at most Vmax. Let Z_S be the unnormalized mass of retained events, including the intended NoRead entry. Then omitted mass is at most m exp(U), and

`eta <= m exp(U)/(Z_S + m exp(U))`.

The complete read is a convex combination of the retained and omitted normalized reads. Therefore

`||read_full - read_retained|| <= 2 Vmax eta`.

This is an elementary bound on the attention-vector error under the stated assumptions. The second probe checked 128 random instances with no violations, using all actual scores to construct U; it did not demonstrate a cheap way to produce U. A serving implementation would also budget score-table, probability, and accumulation rounding. This bound does not guarantee the next token or a truthful answer.

### 5.6 What not to infer

The group law does not tell the model which statement answers a question. Prime IDs remain identities, not semantic coordinates. Hopf or other projections cannot erase task-relevant information and regain it merely by adding a scalar feature. Endpoint value transport by a representation can factor into a change of frame; it is not automatically a novel reasoning mechanism. Parameter sparsity of the query encoder and output head remains separate from event-attention sparsity.

## 6. Three bounded research moves

### A. Attribute representation damage on the current donor

Use the existing query/key accessor and evaluator, after the responsible lab and slot permit. Compare unchanged donor scores, continuous unit normalization, unit H4, gain-preserving H4, and a same-bit learned/PQ codebook on exactly the same saved prefixes. First use evaluation-only evidence; do not start a repair-fit by default.

Measure score distributions, near-tie margins, key/query gains, true-source retention, NoRead, captured mass, output distributions and generated-answer changes. Keep tuning/dev separate from held-out acceptance. If unit normalization accounts for little observed damage, reject gain loss as the leading explanation for this donor. If even adequate information-preserving encoding cannot retain behavior, do not add a learned relation term on top of that failure.

**Durable output:** a source-bound attribution table and a decision about the smallest justified intervention.

### B. Train one calibrated finite read and compile the same score to addresses

Reuse the chosen mainline learner and the existing geometric-read module. First establish the intended hard score path without new semantic behavior; then compare one learned relative correction against a matched ordinary quantized reader with the same exposure and an explicit total-bit/cost accounting. Use source-sensitive language and relational tasks, not only code occupancy or vector reconstruction.

Offline donor distillation is permitted as a declared initial calibration, not hidden serving or the sole correctness target. Wrong donor preferences must remain distinguishable from correct entity/relation use. Query-specific retrieval order comes from the same score tables used at evaluation. Benchmark an ordinary code-pair or learned-codebook alternative with honest differences in parameter sharing and footprint.

**Success:** a prospective language/retention gate plus useful access reduction on the same artifact. **Failure:** no repeat dose by default; preserve whether representation, learning, or retrieval cost failed.

### C. Qualify the integrated native path

On the same frozen artifact, include integer encoding, all query/candidate work, NoRead, value/copy emission, recurrent update, and output projection. Run exact-history output comparisons and a repaired compiled instruction audit. Report actual bytes touched and complete-request latency/RSS, followed by a quiet-machine, same-quality cost comparison when available.

A faster table microkernel, full-scan recall result, or lower numerical loss alone cannot pass this route. Do not call the result frontier-equivalent; progress through the already approved responsive-dialogue milestone, then the same-model reasoning/coding and broader evaluations.

## 7. Ownership and research handoff

Claude owns the mainline decision, the authorized #1433 completion, and the model-learning hypothesis. OpenCode owns honest admission-cost accounting, representation diagnostics, and score-derived indexing. Anti-Gravity owns the native encoder/lookup/value path and whole-artifact measurement. These proposals are advisory; they do not amend current frozen studies or authorize a new competing heavy job.

**Best next research prompt:**

> On the selected retained learner, use existing source and sealed evidence to determine whether the finite reader's main loss is introduced by unit normalization, coarse coding, or the score replacement. Preserve the full evaluator and artifacts. Do not fit a new model until that attribution changes the decision. Then propose one gain-preserving hard reader whose learned relative score can also generate candidate addresses, with matched ordinary coding and explicit inspected-versus-retained event cost. Stop the route if the representation cannot preserve useful answers or if candidate lookup still needs an uncharged full scan.

## 8. Verification, scope and files

- `geometric_attention_probe.py/json`: exact independent group checks, norm counterexample, synthetic quantization and conservative score certificates.
- `group_address_probe.py/json`: score/address inversion, a counterexample to naive factor shortlisting, and bounded soft-read checks.
- `local_t2_observation.json`: local source and report identities, cited ranges, population and illustrative reported metrics.
- `sources.json`: primary source locators, exact versions where inspected, and applicability limits.
- `architecture.mmd`: editable diagram source for the proposed, unimplemented path.
- No new human panel or successful independent Codex specialist review is claimed. The subscription-backed specialist route was quota-blocked in the prior audit and the live board confirms that limitation. This review uses current source, primary research, SciSpace discovery, Wolfram symbolic evaluation, and separately checked computations. Consensus retrieval was rate-limited. The requested Visualize app-block interface was not exposed by the session's tools, so no working embedded app-block is claimed.
- The computations are research-side Python and may use floating-point matrix operations. They are neither a new Python model dependency nor an audit of an M1 serving binary.
- No fresh model fit, held-out language evaluation, energy benchmark or complete review of all 16,057 repository files was performed. Exact algebra, finite assertions, measured donor behavior, and proposed language benefit remain separate.

**Bottom line:** retain the programme, but make the representation, learned relevance, and physical memory-access pattern one coupled design. The research target is not simply another geometric score. It is a learned geometric read that selects the right evidence, can address that evidence directly, and preserves its useful behavior in the actual integer runtime.

## Source register

- [repo-roadmap](https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/ROADMAP.md): Pinned programme contract, tracks and decision rules; labels do not replace measurements.
- [repo-board](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5860922002): Board revision 2026-09-28T04:55:34Z; later work may differ.
- [finite-reader-pr](https://github.com/UOR-Foundation/uor-r4/pull/1438): Reported fixed-dose negative and unrun conditional norm/capacity control; not independently rerun.
- [finite-reader-source](https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/crates/uor-r4-training/src/geometric_read.rs): Actual normalization, hard codes, full-coordinate nonlinear score and straight-through learning.
- [prior-geometric-design](https://github.com/UOR-Foundation/uor-r4/blob/15d2ce3fac9f66ddcc945a8097e5c6216352c453/docs/integration/fourth-lab-geometric-attention-2026-09-26.md): Existing norm-control warning, bilinear/frame equivalences and relative-score proposal. These concepts are not new to this session.
- [norm-explicit](https://arxiv.org/abs/1911.04654): Norm-Explicit Quantization: Improving Vector Quantization for Maximum Inner Product Search; ranking-specific motivation, not UOR causal proof.
- [scann](https://arxiv.org/abs/1908.10396): Accelerating Large-Scale Inference with Anisotropic Vector Quantization; task-score distortion rather than plain reconstruction.
- [transformer-vq](https://arxiv.org/abs/2309.16354): Transformer-VQ: Linear-Time Transformers via Vector Quantization; still a transformer, not D11 deployment.
- [ovqa](https://arxiv.org/html/2602.03922v2): Online Vector Quantized Attention, inspected v2; sparse online memory updates, but dense products remain. Inspect equations rather than treating the name as a contract.
- [avqa](https://arxiv.org/html/2607.12789v1): AVQ-Attention: Adaptive Vector-Quantized Attention, inspected v1; adaptive refinement antecedent, vision results not language/M1 certification.
- [deltaproduct](https://arxiv.org/abs/2502.10297): DeltaProduct: Improving State-Tracking in Linear RNNs via Householder Products; noncommutative geometric transforms, not a UOR serving witness.
- [matmul-free](https://arxiv.org/html/2406.02528v7): Scalable MatMul-free Language Modeling, inspected v7; up to 2.7B parameters, retains elementwise products and does not demonstrate frontier equivalence on M1.
- [product-key-memory](https://arxiv.org/abs/1907.05242): Large Memory Layers with Product Keys; selected parameter capacity antecedent, original transformer setting not adopted as runtime.
- [threshold](https://research.ibm.com/publications/optimal-aggregation-algorithms-for-middleware): Fagin, Lotem, Naor, 2001; monotone-score threshold retrieval with access-model assumptions. No universal sublinear claim.
- [threshold-author](https://www.wisdom.weizmann.ac.il/~naor/PAPERS/middle_agg.html): Author abstract describing sorted-list/random-access model; no full theorem-application claim beyond the elementary bound derived here.
- [tmac](https://arxiv.org/abs/2407.00088): T-MAC low-bit lookup kernels; its efficiency numbers do not transfer to UOR without measurement.
