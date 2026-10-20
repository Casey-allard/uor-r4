# Principal review of PR #1323: useful control, then learn the relation

Reviewed September 21, 2026 against merge `29f680fb5c80d9d00af9f9f2fb1e4c3e6205303d`. PR #1323's head and merge trees both equal `a6aac28b08b1ab12c87731fe2716632e7ede8f42`. This is source review and analysis of retained outcomes, not a new model fit or inference run. The [saved-data evidence](../evidence/contextual-utility-principal-review-2026-09-21.json) binds inputs, methods and limitations. All 23 manifest-listed files in `contextual-utility-2` independently match their byte sizes and BLAKE3 hashes; there are no unlisted files. Preserve both attempts and every older artifact unchanged.

## Decision

**Retain the contextual utility controller. Next, make the existing signed H4 relation learn useful source selection under matched optimization and honest document holdout.** The [next execution prompt](deepseek-relational-learning-step-2026-09-21.md) integrates the necessary instrument repairs into this constructive step. It does not schedule another full repair-only campaign. A larger role-pair table is an optional diagnostic if the learning experiment exposes a concrete need; the last run does not establish missing representational capacity.

The controller's primary improvement is real. Contextual minus frozen global strength is **−0.2576439268 bits/candidate position**, with the original sequence-cluster interval **[−0.3270884384, −0.1829678711]** reproduced exactly from 1,813 positions in 140 sequences. Retain the historical implemented decision `positive=true` for that comparison. It is component progress, not an integrated language-model promotion.

The gain reaches the relational questions themselves:

| Present-answer final queries, n=119 | Global H4 | Contextual H4 | Categorical, global utility |
| --- | ---: | ---: | ---: |
| Hard CE, bits/query | 12.54534 | **9.08857** | 8.25501 |
| Correct selected payloads | 11 | **49** | 58 |
| Correct emitted next tokens | 8 | **35** | 47 |

The newly calculated present-query difference is −3.45677 bits/query, interval [−4.39276, −2.49967]. This is **post-hoc diagnostic evidence**, not an additional preregistered success. On 21 absent final queries the controller reads 10 times versus 5 and worsens loss by +0.15382 bits/query, interval [+0.01288, +0.32926]. Absence handling still needs work. The categorical arm has the best aggregate point estimate; contextual minus categorical is +0.06727 bits/position with interval [−0.03265, +0.16383], so this comparison does not establish categorical superiority. A categorical-plus-contextual arm is still missing.

## Corrections that change the next decision

Source references below are to `crates/uor-r4-core/src/bin/competitive-reader.rs` at the pinned merge unless stated otherwise.

1. **The 12.8× diagnostic is for the global arm.** Lines 1360–1420 call `relational.choose`, then charge NoRead's missing gain to the purported ranking residual. They do not decompose the contextual arm. The best admitted action cannot measure admission failures outside that pool. Thus neither “strength largely solved” nor “ranking/admission proven dominant” follows. Keep those raw numbers as historical outputs with this correction.
2. **Correctness was undercounted by occurrence identity.** Lines 1387–1407 compare to the first target-bearing candidate. Duplicate correct payloads must count as correct predictions; exact occurrence truth is a separate metric. From saved decisions, global covered success is **355/646 = 54.95%**, contextual **410/646 = 63.47%**. The old 25.2% and top-rank 46.3% do not establish the proposed selection gap; corrected top-rank attribution is NOT_RUN.
3. **The reader did not receive a document-separated text evaluation.** Text fitting at 769–794 and evaluation at 1667–1746 use the same first windows of the same eight Dev documents. Only 178 candidate-bearing text positions accompany 2,969 synthetic fit positions. The +0.218875 bits/token harm against local, versus +0.244319 for global, is a training-regression result on 488 positions. This remains useful but cannot qualify transfer. Historical exposure by the frozen local model is a separate disclosure; it does not prevent holding documents out from the new reader.
4. **Optimization has an untested gap.** Lines 894–906 fit scalar parameters, refine descriptors, then quantize without refitting the scalars after changing descriptors. Two coordinate passes do not establish that the present representation is exhausted. Alternating a bounded scalar refit with discrete descriptor refinement is the first useful intervention. The soft expected-action training objective and hard exported policy must be measured separately.
5. **A frozen table does not freeze its function when the ranker changes.** In `learner/relational.rs:338–403`, the ranker's winner and margin determine the utility bucket. Changing source scores changes both bucket input and competition with NoRead even if `ctx` bytes remain fixed. Fit the same controller procedure to each source arm for the integrated comparison. A narrowly frozen-input attribution would need the old scorer too, with its cost counted.
6. **Controls have narrower scope than the headline.** Most shared-path, future and query reconstruction checks use global H4, not the new contextual arm. The altered-source probe changes the first target-bearing occurrence and modifies recent query context in 405/646 cases. It is not a paired selected-source intervention. Zero-table behavioral parity and independent artifact equality are valid narrower checks. Next evaluate the actual independently reloaded contextual policy through the shared causal path.
7. **Artifact binding is incomplete.** The data digest at 851–861 covers abbreviated synthetic records, omitting natural-text fit inputs and full candidate/payload observations; configuration also omits contextual fit details. Bind complete inputs/splits/tokenizer/parent/geometry/seed/optimization/export semantics in the next artifact or cryptographically bound manifest. A source hash alone is insufficient.

Smaller material corrections: the fresh contextual policy takes **827 eight-nat actions, 138 one-nat actions, 848 NoReads and zero weakest actions**. The result's bucket narrative omitted the existing strength bias. Sixteen `[i32;4]` rows use **256 resident bytes of entries**, plus Vec metadata/capacity; 64 bytes is the serialized table payload. Retained attempt 2 timing is 1355.66/1360.07/1384.66 µs for local/global/contextual, but its probe admits **zero candidates**: active-reader cost and physical energy remain UNAVAILABLE. The opportunity JSON's `*_bits` names contain nats. The two-sweep `ctx_fit` is a measured surrogate improvement, not a global optimizer or demonstrated hard-loss optimum.

The original design's ranking-dominance falsifier and the result's softened interpretation must remain visible. The diagnostic does not establish that falsifier's premise. A useful primary gain should be retained without retroactively rewriting criteria or claiming the controller solves all integration problems.

## A concrete signed-geometry capacity witness

The constructed task has fourteen disjoint symmetric partner pairs. The 120 unit roots form the binary icosahedral group, containing sixty antipodal pairs. Choose fourteen distinct antipodal classes, excluding `{+1,−1}`, and assign

`Q(a_i)=g_i`, `Q(b_i)=−g_i`.

Then both partner directions satisfy `Q(a_i)^−1 Q(b_i)=Q(b_i)^−1 Q(a_i)=−1`. Unrelated pairs do not, because their antipodal classes differ. Assign non-role tokens identity. A bounded selector witness sets `rank[−1]=+7`, other ranks −7, the exact-key feature weight +7, other feature weights and bias zero, all strength biases zero, `ctx=0`, and NoRead threshold 7. A same-key partner scores 14; a wrong-key partner scores at most 7; a same-key nonpartner scores 0. With strict read-over-NoRead comparison, this separates the authored present/absent final-query cases without a bigger table, assuming their intended source is admitted.

This is an **algebraic source-selection and NoRead expressivity witness**, not learned performance, a solution to natural language, or permission to insert fixture labels into serving. With tied strength biases, the present tie order selects the weakest boost; the witness does not establish that the answer beats the frozen local vocabulary logits. The next run should check it cheaply against the exact table and then fit without gold partner assignments. General asymmetric or overlapping semantic relations can require more structure; this witness concerns the present fixed symmetric fixture only.

This is also an immediate connection to the owner's spin proposal: `q` and `−q` represent the same SO(3) rotation but distinct elements of signed S3/Spin(3). Identifying them would erase this available relation code. It is classical discrete group computation, not quantum hardware. The common-left invariant `q^−1 k` supplies the relation; the antipodal witness is not itself a general Hopf/S7 memory.

A raw 120×120 directed table has 14,400 entries versus 120 relative-rank entries: **120×** as many scores, or 7,200 versus 60 bytes if packed at four bits, before maps/metadata. Small absolute size can make a diagnostic reasonable, but it changes sharing and inductive bias. A token-pair table at vocabulary 4,096 has 16,777,216 entries; a fixture-role-only table relies on an authored vocabulary partition. None is automatically a better learner. Fresh keys/payloads within the same learned role pairs do not establish transfer to unknown partner rules. Arbitrary unseen pairings require observable evidence.

## Research synthesis and the larger roadmap

The primary literature supports separating the questions, rather than treating a new manifold as a universal answer:

- [Altabaa and Lafferty, ICML 2025](https://arxiv.org/abs/2405.16727) explicitly separate relational and sensory information. The useful connection is to separate exact payload identity, learned relation and action utility. Their dense Transformer architecture is not a serving design adopted here.
- [Gated Delta Networks, ICLR 2025](https://arxiv.org/abs/2412.06464) combines selective forgetting with targeted writes. It motivates testing those as distinct functions at the structural-persistence stage, with our own bounded integer implementation and matched retention controls. It does not establish that continuous delta updates satisfy D0-b.
- [Scalable MatMul-free Language Modeling, revised July 2025](https://arxiv.org/abs/2406.02528) is a relevant ternary/additive systems comparator. Its quality and efficiency measurements do not transfer to UOR-R4 or prove a geometric advantage.
- [Learning to Forget Attention, February 2026 preprint](https://arxiv.org/abs/2602.12204) provides a recent hypothesis about temporary retrieval versus consolidated behavior. Its claims motivate evaluation; they are not accepted evidence for this model or a reason to add a new engine now.

The [existing mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) remains the reuse map. The five stages are:

1. **Useful contextual access and influence — current.** Learn source-sensitive signed relations and calibrated NoRead/strength, test contextual categorical controls, and show reader-level held-out text behavior. Reuse exact occurrence memory and frozen local E+S query-only. Keep retained historical bounded selection/composition results at their own artifact scope.
2. **Structural persistence.** Learn role/scope routing and event-driven retention beyond chronological decay, with exact payload lifetimes, overwrite/absence controls and equal-byte explicit-bank comparators. Retained Hopf fiber or phase is a candidate when it preserves a distinction actually lost by projection. A single static token descriptor is not a contextual syntactic-role model.
3. **Dependent reads and shared composition.** Reuse existing typed operators and learned scheduling after useful reads survive controls. Require a dependent second read and derived output absent from all source payloads; copying alone is insufficient. Reconcile into the same artifact rather than stitching incompatible prior successes.
4. **Language, conversation and executed Rust.** Source-separated sustained prose, corrections across turns, multiple predicates and actual compiled/tested programs. These do not follow from a synthetic relation win.
5. **Qualified scale and delivery.** Measure complete-path optimized latency, resident state, memory traffic and energy at useful quality on the named laptop. Cost checks occur throughout; broad efficiency claims wait for complete evidence.

S7 and E8 are available representation candidates at a measured aliasing or retention boundary. Normalize nonzero R8 vectors to S7 only while accounting for discarded radius; E8 roots are a finite subset, not the full sphere. Preserve the project's specific coupled icosian construction. Spherical harmonic orthogonality is an inner-product property, not automatic immunity to finite-precision mixing, shared normalization or bad role assignment. `G2/SU(3)` is S6, not S7. Hopf projection loses fiber unless retained; the quaternionic fibration has S3 fiber, S7 total space and S4 base. Scalar utility is already productive in the current controller. Twin primes, supersymmetry and physical spin dynamics have no demonstrated additional predictive role here; a proposed use needs a finite operator and distinguishing test.

## Delivery and resource scope

This review changes documentation and saved-data analysis only. No Rust build, training or inference was run; prior focused-test results belong to PR #1323. The shared ledger remains **186736749/194900000 ms**, **8163251 ms (~136.05 minutes)** remaining at review. A 90-minute projection did fit the previous 106-minute balance: the extension's stated arithmetic rationale is corrected, while preserving its authorized recorded limit and all charges.

Storage inventory: **43,057,942,528 bytes free (43.06 GB / 40.10 GiB)** against a 36,766,079,385-byte reserve. No deletion was made. Models are a lower-bound inventory where three sealed paths remain unreadable; no permissions were changed. Refresh resources before the next run and record any necessary standing-authorized local extension prospectively.
