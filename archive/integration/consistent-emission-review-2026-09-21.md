# Principal review: coherent geometric computation after PR #1336

September 21, 2026. Reviewed submitted source `ce8d9340c0f5a22de97319decf8bc18dcba4d46d`, its two sealed attempts, predecessor PR #1335 and the current/retired mechanism inventories. The [saved-data audit](../evidence/consistent-emission-principal-review-2026-09-21.json) and [reconstruction script](../evidence/consistent-emission-principal-review-2026-09-21.py) distinguish independently reproducible quantities from runner-reported results. Original artifacts and reports are preserved. [Next execution brief](deepseek-geometric-computation-step-2026-09-21.md).

## Principal judgment

The programme remains a plausible research direction, but it has not yet established a useful integrated language model or uniquely geometric attention. The recent sequence has repeatedly confused component learning defects with representational limits. Repeating a small fit under another name, or moving to a harder task before explaining the present failure, would not resolve that problem.

The architectural correction is to distinguish **exact evidence, learned contextual addressing, shared relational computation, and lexical emission**. Their interfaces must compose, but one 120-state descriptor need not encode every token identity, relation, scope and output association. A finite state has finite information; a more elaborate mathematical name cannot restore distinctions that were discarded. Conversely, a learned table is an appropriate mechanism for arbitrary finite lexical associations. Its success is a reusable component, not a reason to abandon geometric computation.

Codex retains principal responsibility for holistic architecture, evidence interpretation and roadmap decisions. DeepSeek is a capable research contributor with authority to choose and revise implementation, learning and diagnostic approaches within that direction. It should pursue coherent milestones and justified follow-ons, with explicit hypotheses and resource updates, rather than optimize its behavior for an artificial one-run or fixed-time-window rule. Returned runs receive a new principal assessment against the entire programme, including internal negatives and external research.

## What the latest run establishes

| Arm | Development /180 | Tune /120 | Final /120 | Final pairs both /60 |
| --- | ---: | ---: | ---: | ---: |
| Local, ReadDisabled, UpdateDisabled | 0 | 0 | 0 | 0 |
| Development constant | 32 | 13 | 15 | 0 |
| H4 update plus learned residual | 50 | 36 | 25 | 1 |
| C120 update plus learned residual | 42 | 25 | 20 | 4 |
| Learned selected-value lookup | 164 | 112 | 108 | 54 |

Both seven-file seals verify with no unlisted files; delivered source, executable and artifact hashes match the submitted experiment. The independently reconstructed lookup is correct on every correctly selected payload: 164/180, 112/120 and 108/120. Final exact-source counts are 108/120 as reported by saved flags. Delivered value maps really are injective over the eight values; The runner reports seven accepted H4 map changes and three C120 changes. These are useful repairs and retained evidence.

The H4/C120 aggregates remain reported measurements: the saved rows omit their emitted IDs, updated states, post-update logits and actual occurrence references, so those aggregates cannot be independently rebuilt from the rows alone. Even allowing all 12 wrong-source rows to be correct emissions, H4 can only have 13–25 correct emissions among 108 correct-source rows. Selection cannot explain the bulk of the gap.

All eight value-to-answer associations recur across development, tune and final. Prefixes differ, but this is familiar-association performance in new contexts, not held-out compositional transfer. The final seed was first evaluated in attempt 1 and repeated with identical artifacts/rows/predictions in the report-only attempt 2. There is no evidence of model selection between those attempts; this is one draw plus a replay, not two independent confirmations.

Three loaded-artifact three-token autoregressive traces **did run**. First answers are 1/3 correct; continuations are `edia-f`, `edia-f`, and `](..`. Useful prose and text integration remain unqualified. The result's former `NOT_RUN` generation statement is corrected. Historical screen failure remains unchanged.

## Why the expressivity diagnosis is not established

1. **Hard support differed during learning and serving.** The submitted learner ternarized all 4,096 output rows during fitting, then retained only 64 by latent norm. H4 loss rises from 9.321115 to 12.868782 bits at this projection, a 3.547667-bit penalty; C120 rises 9.625529 to 12.896118, a 3.270589-bit penalty. These were already ternary forwards, so `float_after_gradient_fit` is a misleading label. Coefficient alphabet consistency is insufficient when row support changes.
2. **The final output refit was omitted.** The runner fitted W, refined W, then changed value/transport maps and evaluated the old W. The required output → maps → output stage did not execute. A function that resets W to zero is also not an incumbent-preserving continuation.
3. **Objectives changed between stages.** Calibrated CE trained W; an unbounded average margin plus a finite collision penalty selected maps. A stage can improve its own surrogate while worsening the deployed predictor. A large finite coefficient is not a hard priority.
4. **The lookup comparison changes the emission interface.** It emits the learned categorical answer directly; H4 adds a bounded residual against hostile local logits. All 420 saved local target margins are negative. The lookup is a valuable task-level comparator, but its advantage does not isolate geometry from optimization and integration.
5. **The task cannot establish the desired structural advantage.** For arbitrary `y=f(selected_value)`, an eight-entry development-fitted dictionary is the natural sufficient mechanism. Requiring geometry to beat it was a new, poorly motivated condition, not the previous principal prompt's requirement to report that comparison. Do not change the recorded failed outcome. Prospectively separate useful component learning from evidence of geometric advantage.

The old collision counts 36/20/28 include wrong-read contradictions; every conflicting pair includes a wrong selected value. The saved injective maps are a valid artifact fact, but this counter does not prove all actual features are distinct. Inspect the served feature `R(q0*T[r]*V[v])-R(q0)` across relations and source-correct examples when diagnosing representation. Distinct state indices can still produce identical differences.

## Repairs delivered with this review

One deterministic top-row ternary projection is now shared by every hard training forward and export. Training logits use the actual integer residual routine, including saturation. The latent gradient through sign and row admission is explicitly a surrogate; hard-forward equality does not imply unbiased or successful optimization.

The output fitter starts from its incumbent, retains the best actual served CE, and discrete coefficient refinement uses that same calibrated CE. Map search prioritizes observed incompatible distinct-payload aliases lexicographically, then CE; contradictory labels on the same selected value are separated from value-code merging. This is still not a universal injectivity theorem.

The actual runner calls output fitting both before and after map search, consumes the final returned output in export/evaluation, checks the post-map starting loss and non-worsening refit, and records stage losses and output hashes. Focused checks cover nonzero >64-row projection/reload equality, target admission, changed-map refit consumption and incumbent retention. The [check receipt](../evidence/consistent-emission-principal-checks-2026-09-21.json) records executed validation. **No new complete model fit or fresh capability evaluation is claimed by these source repairs.**

The continuing evidence obligation is compact, sufficient per-position output: actual references, states, predictions, loss/margins and action parity. It should come from the shared predictor, not an independent reporting implementation. Repair it in the next substantive run; do not build a new general audit framework.

## Systemic investigation: prevent the recurring boundary failures

A separate investigator compared PR #1328 through #1336. The repeated failure lies in experiment orchestration: fit/serve thresholds diverged (#1328), new artifacts were bypassed by generation (#1333), prefix/target/categorical boundaries were wrong (#1334), arithmetic and retained coefficients diverged (#1335), and sparse support/refit stages diverged (#1336). This is evidence about those execution paths, not a finding that group mathematics caused the failures.

The corrective pattern is concrete: one shared hard numerical contract; one complete learning lifecycle returning the actual final predictor and stage losses; one prediction record feeding both rows and aggregates; and atomic expected-before/read-after updates of the existing resource JSON. The first two are repaired here. Complete sufficient prediction rows and transactional accounting in the next substantive run without constructing another framework. A small adversarial consumer check is more valuable than parameter equality or an additional checklist. If these shared boundaries agree and the model still fails, optimization/data/representation becomes the legitimate research question.

The principal's own handoffs must also improve: specify the scientific objective and causal discriminator, give DeepSeek responsibility for the complete lifecycle and room to adapt, then review its actual outcome against the full programme. Repeated tiny repair assignments are not the goal. Necessary corrections should be completed when discovered, with honest exposure/resource updates, rather than deferred merely to preserve a historical prompt.

## Coherent programme direction and reuse

| Function | Reusable mechanisms | Present evidence and next responsibility |
| --- | --- | --- |
| Preserve evidence | Prime/UOR identity, ordered n-lets, owned occurrence references, versions, leases/commit, historical links | Exact identity and scoped memory behavior exist. Keep bytes/version ownership outside lossy descriptors; rejected links do not prove absence |
| Find relevant context | Occurrence reader, relative signed-H4 descriptors, source routing, Hamming proposals, scalar compatibility/NoRead | Authored selection works substantially better than emission. Candidate availability, selection, abstention and useful integration remain separate |
| Interpret and compute | Finite 2I products/inverses, typed shared operators, selected-context query updates | Algebra is verified, language advantage is not. Train actual content-and-query-dependent transformations and novel combinations |
| Express a result | Learned lexical lookup, low-bit shared decoder, copy operator, byte/EOS donors | The lookup solves familiar eight-way lexical association given the right source. Reuse it where appropriate; shared contextual emission and general prose remain missing |
| Decide what to do next | Historical Read/Emit/Stop, dependent reads and trajectory credit | Finite authored results supply mechanisms, not transferable BPE artifacts. Learn the next read from the previous result and terminate without gold hop counts |
| Retain structure | Role/scope banks, required-evidence/replacement state, exact versions; sparse overwrite reduction of delta memory | Addresses the subject-versus-filler idea. Invoke when structural alias/eviction is a limiting cause, and train occurrence roles rather than inject grammar |
| Enrich relation features | Signed spin/phase, relative Hopf observations, retained fiber, finite harmonic bases; conditional paired-H4/E8/S7 | Choose the smallest representation that preserves a witnessed distinction or tests a specific generalization hypothesis. These are available research tools, not mandatory milestones |
| Deliver useful language | Existing local E/S prior, TinyStories/prose learning infrastructure, shared contextual operators, source-separated text and Rust tasks | Three model families remain distinct; transfer an operator/interface, not incompatible artifacts. Integrate useful conversation/memory and executed reasoning/code on one path before claiming alpha |

The immediate milestone is a reliable learned **Read → Transform → Emit** primitive, keeping the association task as a regression. Correct its remaining learning confounds, then choose between the repaired geometric emitter, a justified relative-feature variant, or a learned lexical decoder attached to geometric computation. Do not spend repeated cycles making geometry beat a dictionary on arbitrary labels. Continue to dependent Read/Update/Read/Emit/Stop when the selected primitive is useful; a decoder donor may remove the lexicalization obstacle without resolving or hiding the old residual's negative.

For composition, change older evidence while preserving the query tail, and change query operation while preserving evidence. Hold out combinations of familiar primitives. Answers absent from source payloads alone do not establish context use: PR #1334's suffix-only fixture already demonstrated that trap. H4-defined synthetic labels can test implementation, but are not independent evidence that H4 is the best language geometry. Compare equal-information learners with both operation and value inputs; do not blind the lookup arm to the query.

Structural memory can support this milestone immediately if scope, ownership or eviction is the observed obstruction; it need not wait for an arbitrary stage number. More general S7 state or a harmonic field needs a specified write/read law, extra preserved distinction, and quality/cost comparison. One point on S7 is not a field of harmonic coefficients, orthogonality does not make token writes interference-free, and the 240 E8 roots are not 240 orthogonal registers. Exact payloads remain independently owned.

## New literature and mathematical bridge

[Neural Networks Provably Learn Spectral Representations for Group Composition, v2 July 2026](https://arxiv.org/abs/2606.02993) is directly relevant to the owner's harmonic intuition. Its general finite-group specialization theorem concerns projected gradient flow on an approximate small-logit full-population objective in a quadratic two-layer network. Prediction analysis requires additional conditions; automatic population coverage is proved under restricted Abelian assumptions. Held-out generalization remains open there. It does not validate sparse ternary STE, this H4 learner or laptop cost.

**Engineering hypothesis:** replace the random 120×16 state embedding with a selected quantized real matrix-coefficient basis of 2I, compiled offline into the same finite lookup footprint. Existing product/inverse tables perform transitions; no representation-matrix multiplication is needed at serving. Compare random, spectral and learned features if feature aliases/conditioning or recombination motivate the experiment. Check quantized Gram/rank, incompatible-output aliases, signed `q/-q` distinctions, generated pairs and actual cost. Characters alone collapse conjugacy classes and may erase directed information; include appropriate matrix coefficients and real/quaternionic consistency. Orthogonality is a full-group averaging identity, not zero interference under arbitrary token populations. This finite spectral bridge is a cheaper first hypothesis than a general S7 wave, and is **NOT_RUN**.

[Lake and Baroni's SCAN study](https://proceedings.mlr.press/v80/lake18a.html) motivates testing familiar primitives in unfamiliar compositions rather than only drawing new contexts. [When does compositional structure yield compositional generalization?](https://proceedings.iclr.cc/paper_files/paper/2025/hash/572a6f16ec44f794fb3e0f8a310acbc6-Abstract-Conference.html) reinforces that compositional representations alone do not ensure transfer; training statistics and shortcuts matter. These sources motivate instruments, not importing their architectures.

The [historical mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md), [geometry scope note](structural-memory-hopf-direction-2026-09-20.md), source audits, saved failures and prior principal reviews remain the broader context. This review does not claim a new line-by-line audit or reexecution of every historical engine.

## Resources and delivery

DeepSeek's already-applied six-million-ms allowance is retained; its missing 4,400,000-ms debit is reconciled exactly once, yielding 212,857,254 /216,900,000 ms before principal work. The prose's uninterrupted-storage-compliance claim is corrected: the handoff describes a recovered build-time reserve breach. A time allowance creates no physical disk. Record complete revised local projections and update live JSON before an authorized extension; preserve the physical reserve and 128 MiB stop margin. No additional owner permission is needed for that already-authorized class of necessary local increase.

The principal audit preserves both attempts and the owner checkout. Live issue bodies, current state, the canonical plan and front doors are synchronized; experimental source delivery is not model promotion. The resource ledger/check receipt owns final debits and disk measurements. Paid/external compute remains unauthorized. Full-path serving compliance, useful general prose/reasoning and energy savings remain unestablished.
