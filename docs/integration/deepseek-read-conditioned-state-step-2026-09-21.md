# DeepSeek execution: learned read-conditioned geometric emission

You are a research contributor to the UOR-R4 Geometric Language Model. Execute the next constructive step: characterize the frozen confidence operator through actual generation, then implement one bounded learned Read -> geometric Update -> shared Emit primitive. Preserve useful components and unfavorable evidence. Use your mathematical/engineering judgment; an equivalent smaller factorization is welcome if it answers the same causal question. Explain substantive choices before opening final outcomes. Do not reduce this to another gate-feature sweep or a documentation-only plan.

## Recover exact context and authority

Refresh origin/main and verify PR #1333's actual merge and reviewed tree, not only its subject. Its initial submitted head was 51470ea99fb32ba7060e2657c1b661e95fd7844d; the principal corrections are later commits in the same PR. Read AGENTS.md, README, project-track/current-state/model-direction, DECISIONS.md D0-b/D1/D2 and the stable execution policy. Read these completely:

- [Principal confidence review](reader-confidence-review-2026-09-21.md) and [independent audit](../evidence/reader-confidence-principal-review-2026-09-21.json).
- [Confidence result with correction](reader-confidence-result-2026-09-21.md), its [design](reader-confidence-design-2026-09-21.md), and the prior [interface review](policy-obstruction-review-2026-09-21.md).
- [Current/retired mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) and [structural/Hopf scope](structural-memory-hopf-direction-2026-09-20.md).

Use project knowledge search/get_source and scoped history to close gaps. Inspect actual donor source before reuse. Prefer relevant primary research and bounded independent reviewers over repeating a broad audit. Preserve the owner checkout and other worktrees; use an isolated full worktree/codex branch. Refresh issues #820/#973/#1139/#962/#963/#964 and any project items. Rust preparation/training may use floats and matrix multiplication; serving follows D0-b's bounded <=4-bit additive/shift/table operators and geometric priority. No hidden transformer or provider responses, no Python model implementation. Historical workflow wording does not override owner decisions.

## What the last run established

The confidence sign restores a distinction omitted by the old 32-address class. Both new 64-address policies meet their development-fit constraints. Zero mismatches were reported on 2,969 nonempty fit positions comparing parent selected index/strength with the hand-built D-sign rule. Full independent-loaded prediction/logit parity, empty/tie-case coverage and confidence generation were not measured.

Both final policies fail joint preservation against their own parents. H4 has 71/117 correct answers versus 73, categorical 79 versus 81; both satisfy the two-count margin. Present CE increases +0.159790/+0.152981 bits/query, exceeding+.05. Absence reads rise 11 versus 10 and 7 versus 6. Text deltas remain positive, +0.037882/+0.020116; tune text already exceeded the+.05 screen at +0.061316/+0.061381. Whole fresh-construction emissions fall 178 versus 364 and 184 versus 378. Retain the fitted expressivity gain, but do not claim general preservation from the selected-query counts.

The three report roots are sealed and all declared source/artifact hashes verify against the submission. They repeat the same tables/data/outputs; the one new acceptance population was first exposed in attempt 1 and replayed in 2/3. All 36 old Dev documents are now exposed to reader evaluation. Local-prior exposure remains separate. Confidence generation/interventions/timing are NOT_RUN; old panels do not qualify new arms. Confidence fit identity and saved per-position/tune statistics remain incomplete. Principal fail-closed and metadata repairs prevent missing confidence artifacts from silently becoming local-only runs; they do not supply the missing behavior evidence.

The proposed "add a text/query role bit" successor is not selected. The residual harm is real, but its exclusive cause is not established. A supervised corpus/stratum flag must never enter serving. The sign is not a calibrated probability, and the optimum applies only to the measured operational class/support/fallback.

## Why change the operation

Current `competitive-reader::predict_next` only adds a nonnegative boost to the selected payload logit c. For other tokens v,w, `p'(v)/p'(w)=p(v)/p(w)`. If gold y!=c, `delta NLL=log(1+p(c)*(exp(a)-1))>=0`. Its argmax can only remain the local winner or become c. If a required answer is absent from all source payloads and differs from the local winner, no refinement of this scalar-copy table can solve it.

In the latest text panel,939/976 next targets are absent from admitted payloads. This does not prove those positions have useful retrievable context. It does prove why a copy-only action cannot use relevant retrieved information to improve an uncopied target. We need an operation that can change relative scores among nonpayload tokens.

## First, close the frozen-artifact boundary cheaply

Load the actual retained H4/categorical confidence artifacts through the verified consumer. Run a compact prospective development set of present, absent, conflicting and ordinary-text complete-response prompts through the shared target-free predictor. Include the scored parent, local, current confidence and ReadDisabled, changed-source and UpdateDisabled where applicable. Save complete emitted tokens and selected exact occurrences, not just a first answer. Confirm confidence policy dispatch after reload and integer logits. Explicitly cover empty pools, one/many candidates, first-candidate source ties and read/NoRead ties with focused fixtures.

Do this without refitting or a full three-harness replay. It is completion of missing evidence, not another fresh final claim. A narrow optional strict parent NoRead veto is a diagnostic comparator: on identical prefixes it bounds reads by the same parent if parity holds. It does not automatically improve CE or generated behavior. Do not optimize another confidence threshold/table in this step. If the frozen path is broken, repair that interface before dependent work; if its outputs are weak but mechanically correct, preserve the negative and proceed with the new primitive rather than demanding general prose first.

## Implement one read-conditioned update and shared emission

Freeze the local E+S reference, retained source descriptors/ranking, candidate admission, ring/candidate bounds and exact occurrence/version semantics. Select the frozen reader/admission rule prospectively (confidence or the parent-preserving witness), use that same rule in all update controls, and explain the tradeoff. Do not quietly train a new gate alongside the update.

A concrete compact sketch is:

```
q0 = learned query root from causal prefix
r  = directed relation of query and selected source
v  = learned descriptor of the selected actual payload
q1 = q0 * T_theta(r) * V_theta(v)       // exact signed-H4 products
z1 = z_local + u_theta(q1) - u_theta(q0)
NoRead or UpdateDisabled: q1 = q0 and residual = 0 exactly
```

The two small shared transport maps avoid a 120^3 table. Freezing selector descriptors/ranking does not force the new value descriptor to reuse a coarse old palette: permit one separate bounded learned value-code map if needed, artifact-bound and capacity-matched in the categorical arm. Check development input-signature collisions before diagnosing optimization failure. Retain q0 and the owned exact source/payload reference separately; one 120-state root does not losslessly store all context. Preserve the multiplication order, signed orientation and declared frame convention. Do not claim equivariance merely because group products occur: derive how query/source/value transform if making that claim. A different comparably bounded finite operator or equivalent shared residual is allowed with a prospective mathematical/byte-cost argument. Do not silently substitute hash distance for geometry.

Inspect `learner/query_read.rs` learned root maps and `row_scores`/`residual_scores`; `dependent_attention/runtime.rs::update_query` and its controls; existing owned-reference and typed emission/scheduling modules. Reuse the learned boundary and shared output-credit idea rather than importing a tiny authored grammar, fixed two-byte keys or an explicit gold schedule. QueryHard's output rows are additive maps with full-vocabulary costs; report row construction, materialized bytes, cache footprint and per-token accesses. No vocabulary-sized expert per source, hidden dense attention/MLP or unexplained precomputed transformer.

Start with one read, one update and one shared emission residual. Learn the smallest effective transport and, if required, one small shared low-bit residual readout under final-token/output loss. A no-op initialization must reproduce the local baseline and retain a route to learning. Identity transport plus a constant/zero readout can deadlock both components. Use, for example, identity transport with a nonconstant seeded readout and discrete transport search, or nontrivial transport with an initially zero readout; verify finite parameter sensitivity once on development. Avoid saturation-order changes breaking exact zero residual. Training may be floating point, but assess/export the actual hard integer path, bind learned parameters and use widened arithmetic or explicit checked bounds where necessary. The update state is an intermediate computation; longer-lived structural memory is not automatically introduced.

A second dependent read/learned scheduler remains the next responsibility after this primitive demonstrates a useful causal transformation. Do not combine an entirely new two-hop scheduler, retention mechanism, readout architecture and representation scale sweep in the first test. Your judgment may eliminate unnecessary scaffolding, but keep the causal contribution separable.

## Design a reduced test that the old operator cannot solve

Before fitting, define one small task family in which the selected context is relevant and the required next token or derived result is absent from every admitted source payload. Also retain a decisive subset where the frozen local argmax is wrong. Hold recent suffixes constant while older relevant content changes the required nonpayload output. Reuse appropriate language/typed-operation data generators if their assumptions are explicit; ordinary contextual agreement or a bounded derived transformation can be an instrument, not a claim of general language reasoning.

Verify source coverage and the old operator ceiling on development examples before investing in training. An oracle-source version may diagnose whether learning or selection is failing, but cannot replace end-to-end serving or qualify success. Keep gold targets, domain labels, role annotations and expected operations in offline supervision/reporting only. The model receives the actual prefix and exact selected content. Do not hardcode a word/domain parser, a target-specific emission table disguised as a gate, a gold operation ID or an expected next query.

Split by relationship/content combinations or families so that memorized token-to-label remapping cannot qualify compositional transfer. Vary distractors, source positions/order and literal payload identity; use more than one surface form if claiming linguistic transfer. Record exact held-out criteria and the falsifying alternative before final outcomes. A bounded authored success establishes a reusable read-conditioned transformation, not broad prose or reasoning.

## Learn and evaluate with matched controls

Use final prediction/output loss on the actual hard serving path, with a documented offline optimizer or discrete search. Keep update/operator capacity, training exposure and optimizer effort comparable for H4 and a genuinely matched ordinary categorical-state control. Give both the same source information, admission and shared-emitter capacity. Reuse the exact geometric identity tables; geometry's architectural priority is not evidence of a comparative gain. A consistent relabeling of H4 elements and its multiplication table preserves an isomorphic group; it is not the ordinary nongeometric comparator.

Required informative comparisons:

- Local/NoRead and retained scalar-copy reader.
- New H4 read-conditioned update plus shared emission.
- Matched categorical update/emission.
- UpdateDisabled/no-op, ReadDisabled and changed selected payload/source; recompute downstream state and generation after interventions.
- When needed, a declared source-correct diagnostic separated from normal end-to-end results.

Measure whether retrieved evidence causally changes the relative logits of two nonpayload tokens, whether it changes the right emitted answer, and whether the complete generated response remains coherent for the scoped task. Report source availability, source selection, update effect and final output separately. A zero residual/all-NoRead collapse cannot qualify because the local-wrong subset must improve; source-correct teacher forcing alone cannot qualify actual generated behavior.

Retain the old present/absent/text controls against each arm's own named parent, with all count AND CE criteria and whole-stream outcomes. Do not use H4's weaker absence baseline for categorical. Treat ordinary text as a harm/generalization screen and the noncopy task as the new capability test; state that scope explicitly. Existing practical margins can change prospectively with a reason and effect-size/uncertainty analysis, never by relabeling an old failure. A small count miss need not destroy a useful component, but all failed metrics must remain visible.

Use open development data for learning/selection. Predeclare the tune selection rule and react to tune failure before spending a fresh final. Freeze one candidate per declared comparison, then make one actual held-out draw. The old 36 Dev documents and previous construction populations are exposed; select eligible additional pinned documents or a modest new corpus slice with honest source separation and hashes, disclosing local-prior exposure. Do not repeatedly run a full harness merely to fix metadata; separate report repair from prediction/fitting.

Store compact per-position or equivalently reconstructible data: document/sequence boundaries, actual source/payload references, q0/r/v/q1, parent/config identities, logits or sufficient outcome differences, update/action and count/CE outcomes, complete-stream denominators and per-document/per-sequence sums. Preserve fit AND tune statistics needed for the chosen objective; save decision criteria evaluated on the actual new arms. Targets remain offline metadata. Use paired cluster uncertainty with units and small-sample limits; four documents do not support precise broad-population claims.

## Artifact, causality and stop rules

Bind actual executed source files (including every new module), executable, tokenizer/local parent, source reader, update/emitter parameters, semantics version, training configuration, fit documents and sequence/occurrence boundaries. Keep final evaluation identity separate from fit identity. Recompute source hashes after the last code write and before execution; record actual checkout plus dirty-source identities. Use the verified loader's returned predictor in every evaluation, generation and timing path. Failed/missing/mismatching artifacts must stop, never silently select a local comparator.

Build causal generation on the same observed-prefix boundary. No supervised Obs, coverage label, artificial candidate index, future token or authored stratum may affect inference. Preserve exact ownership when a selected slot is updated or evicted. A report seal certifies its file set, not model quality or complete causal dependency closure.

Stop after this bounded primitive and selected evaluation. If it fails, preserve the candidate and diagnose whether the limitation is admission, source reliability, update expressivity, learning, shared emission or unsupported generalization from concrete traces. A failed fit is not proof that all geometric states cannot do it. Do not automatically add dimensions, reopen dose tuning or demand a broad theorem campaign. If it succeeds, recommend dependent Read -> Update -> Read/Emit with learned stopping as the next integration; require derived outputs rather than only another copied payload when claiming reasoning.

## Keep the whole roadmap and resources coherent

The goal remains native geometric language computation with useful conversation/memory and Rust coding/reasoning on consumer hardware. Signed H4/Spin, exact prime/occurrence identity, typed paired-H4/icosian geometry, retained Hopf fiber, scalar compatibility and shared typed operators remain the reusable mechanism set. Structural lifetimes/role state address witnessed older-context aliases; S7/E8/harmonic fields require equal-capacity experiments and explicit mode assignment/interference/cost. They are not free orthogonal syntax or infinite lossless memory. This update test uses an existing geometric bridge to expand what a read can compute.

Read the live JSON and latest [resource ledger](resource-ledger-2026-09-19.md), not a historical allowance in this prompt. Principal review reconciled the stale prior JSON and 299800 ms component undercharge; checks add their own measured debit. Necessary local extensions are already owner-authorized: record full projection, reason, increment and updated cumulative limit BEFORE use. Include all preparation/build/extraction/fit/controls/evaluation/retries/checkpoint/delivery work. No paid/external compute.

Refresh physical storage. Review cleanup recovered 4.358 GB observed free space from only inactive incremental intermediates; about 39.78 GB remained before principal checks. The 36766079385-byte reserve plus 128 MiB stop margin still applies. Use `CARGO_INCREMENTAL=0`, shared valid artifacts/builds, bounded compiler threads and bounded scratch. A paper storage extension does not create free space. Never delete unique source, models, sealed evidence, research, owner/other-agent work or downloads to fit a build. Stop/checkpoint before limits, rather than record an overrun afterward.

Claim report roots exclusively before model load; preserve every attempt and never mutate sealed roots. Compile/exercise changed Rust with focused arithmetic, causality, no-op/disabled parity, serialization/rejection and generated-behavior tests. Run fmt/check and the claim-wording gate; no blanket suite. Queue compatibility statuses are not tests. Energy remains UNAVAILABLE until physical measurement; row lookup alone does not prove whole-path efficiency.

Deliver through a protected PR, stage named paths, attach the PR and verify actual merge plus reviewed-tree equality. Update README, canonical plan, current state, CONTINUE/EVIDENCE/model-direction/PROJECT_MAP, active mechanism pointers, design/result and all six owning issues/project items if present. Broad acceptance remains open. Import compact revision-pinned knowledge and verify retrieval. End with your choices/disagreements, actual causal behavior, every failed criterion, retained components, source/artifact/merge receipts, resources/free space and one evidence-supported next operation.
