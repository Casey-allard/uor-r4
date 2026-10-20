# DeepSeek execution: restore learned read confidence and test useful influence

**Executed by PR #1333.** The [principal review](reader-confidence-review-2026-09-21.md) retains fit expressivity but corrects fresh criterion/control scope. The [read-conditioned emission prompt](deepseek-read-conditioned-state-step-2026-09-21.md) supersedes this historical task specification.

You are a research contributor to the UOR-R4 Geometric Language Model. Implement and evaluate the next information-preserving influence interface, deliver through a protected PR, and synchronize the project. Use your own mathematical and engineering judgment. Challenge the proposed representation if a smaller, sounder construction answers the same question, explain the choice before opening new final outcomes, and retain all unfavorable evidence. This is substantive model work, not another documentation-only audit or an instruction to tune the old policy indefinitely.

## Recover authority, context and the exact parent

Refresh origin/main. Read AGENTS.md, README, project-track.md, current-state.md, model-direction-2026-09.md, DECISIONS.md D0-b/D1/D2 and the execution policy. Read the following completely:

1. [Principal review completing PR #1332](policy-obstruction-review-2026-09-21.md) and its [independent evidence audit](../evidence/policy-obstruction-principal-review-2026-09-21.json).
2. [Feasibility result with corrections](policy-feasibility-result-2026-09-21.md), [design](policy-feasibility-design-2026-09-21.md), and the prior [objective review](policy-objective-review-2026-09-21.md) where needed to distinguish historical populations.
3. [Mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) and [structural/Hopf direction](structural-memory-hopf-direction-2026-09-20.md), including current and retired reusable components. Historical execution pointers are dated evidence, not active instructions.

Inspect actual Rust in `learner/relational.rs`, `learner/policy_feasibility.rs` and `src/bin/competitive-reader.rs`, plus their actual consumers and artifact formats. Use the knowledge store and scoped history for missing context, primary research for consequential choices, and independent agents for bounded useful reviews. Avoid repeating an entire architecture audit without a reason. Verify actual PR #1332 merge/tree and reviewed source; its original submitted head was `24c874ac4e600478f0a78b632c9c2c779eaa765c`, and the principal corrections are later commits in that PR.

Preserve `/Users/casey.allard/uor-r4` and other agents' worktrees. Use an isolated full worktree and codex/ branch. Refresh issues #820/#973/#1139/#962/#963/#964 and any project items; assignment denotes current work. Preserve sealed report roots, all unique research and negative candidates.

Rust training/preparation may use floating point and matmul offline. Serving follows D0-b: bounded <=4-bit additive/shift/table learned maps and geometric routing, with no floating-point numerical serving or hidden transformer/provider. Score accumulators and identity addresses need not be four-bit weights. Name the executed kernel boundary and any unfinished whole-path compliance. Do not revive the historical blanket ban on mathematical linear maps.

## What is settled, and what is not

The unchanged five-bit, 32-address action class is exhausted as a research question on these development counts. Fit has 197 present and 23 absent queries. The retained contextual H4 parent emits 149 correct answers and reads seven absent queries; the translated acceptance requires >=145 correct and <=7 absent reads. An independent integer dynamic program gives maxima **74 H4 / 71 categorical**, even with every bucket free. Reaching 145 requires at least 16/15 absent reads. Operational fallback restrictions give the same bounds. NoRead, weak and one-nat actions produce zero present correct emissions in these saved counts; eight nats is needed for those answers.

This is a collision in the coarse influence observation. It is neither a global H4/R4 capacity limit nor an objective-only problem. Do not repeat the same 32-address solve, enlarge margins to hide this large loss, or infer a new geometric state requirement from it. The text-only optimum being negative does not prove any joint text/present combination feasible.

Principal code repairs fix fixed-objective offsets and tolerance consistency in the generic solver, saturated action-loss identity, and reporting of tune loss when no policy was selected. The independent count certificate does not depend on its floating-point optimizer. Both feasibility roots are sealed; attempt1 is superseded, not unsealed. Their panels and generations replay prior exposed data. Original executed-runner source correspondence remains unresolved and its module inventory was incomplete; preserve that limitation. No new held-out capability, general prose, physical-energy advantage or model promotion was established.

## Concrete objective: preserve the existing decision information

The retained scored reader already computes whether its best source/action beats NoRead. The influence table lost this information. Restore it at the causal inference boundary and learn how strongly to use an accepted source while preserving the ability to reproduce the useful parent.

Use the parent's actual learned integer advantage:

```
D = max_(candidate, strength) parent.strength_score(candidate, relation, strength, parent_bucket)
    - parent.noread_score(parent_bucket)
```

Use a widened signed difference; `D>0` permits the scored read, `D<=0` abstains. It is a learned score, not a probability or automatically calibrated log-odds. Preserve exact sequence/absolute occurrence, payload position, version and relation encoding. No target, correctness, authored domain/role, coverage label or future token may enter the observation or action.

For the pinned `relational` and `relational_ctx` H4 artifacts, the audit verified equal roots, source weights/ranks, bias, sb and NoRead intercept. All16 contextual rows uniquely maximize the eight-nat strength, and that offset is candidate-independent. On the identical ordered candidate pool, `ungated_top_source` therefore agrees with the parent source. Both preserve the first candidate on ties. The parent's `bucket_of` includes the newest-source bit and its own single-candidate margin convention; substituting the utility bucket or bare source_score minus NoRead breaks the witness.

Thus the concrete witness for this pair is:

```
empty pool -> NoRead
otherwise choose the same top source and compute D using the actual parent bucket
D <= 0 -> NoRead
D > 0 -> eight-nat read
```

Before fitting, verify exact source/action/strength/integer-logit parity against the independently loaded parent on causal development prefixes, including ties, empty pools, one candidate and multiple candidates. This is a cheap implementation prerequisite, not a new model-quality claim. If artifact equality has changed, derive and test the full actual parent decision rather than assuming the witness persists.

The proposed minimal operator is the existing five-bit address plus `D>0`, at most 64 entries, with the same four actions. Include the witnessed parent rule as an explicit candidate and fallback on unsupported entries. It already establishes construction expressivity; it does not establish a text-safe policy. A compact equivalent policy that retains this witness is welcome. A different fixed quantization of D needs a specific development argument, declared capacity/cost and a single selected design before final evaluation. Do not simultaneously change ranker, candidate admission, ring lifetime, local model and confidence representation. Do not turn this into an unrestricted confidence-feature sweep.

## Prospective design and joint learning

Write a concise design before extraction/fitting. Pin observation semantics, parent/witness, data partitions, selection rule, support/fallback, objectives/margins, solver allowance, fresh populations and resource limits. Prefer direct counterfactual next-token loss with actual integer dose/tie/saturation behavior. For fixed local payload probability p and conditional selected-source correctness r, the offline ideal boost is `logit(r)-logit(p)`; this motivates reliability plus local plausibility but does not turn D into r. Within varying-p cells, average individual costs, not a cost at mean p. Select finite actions by their actual counterfactual costs, not nearest-value rounding of the continuous optimum.

Preserve a feasible construction witness while minimizing complete-stream text loss, with all of these jointly measured:

- Present emitted answers at least parent minus the prospectively translated small count margin (historically at most2/121; report exact counts/rates).
- Present CE no worse than parent plus the declared margin (historical +0.05 bits/query).
- Absent read rate/count and loss no worse than parent.
- Text harm screen historically +0.05 bits/token versus local; separately require and report actual useful text gain for a positive influence claim.

Report both feasible preservation and improvement. A near miss can be retained and interpreted with magnitude/interval/cost; do not retroactively change an acceptance criterion to declare success. If a practical margin change is justified, state it before final inspection and preserve the old decision. Do not demand mathematically perfect scalar gating before developing a separately evidenced missing operator.

Choose a bounded appropriate optimizer. Reuse the corrected solver only if its64-entry behavior fits the projected budget; a parent-incumbent local/discrete optimizer with honestly unproven optimality may be more useful than exhaustive 4^64 search. You are not required to prove a global optimum to evaluate a selected feasible learner. Keep the construction-preserving parent incumbent, deterministic tie rules, independent saved-stat replay and measured bounds/status. Timeout is unresolved optimization, not information impossibility. Do not implement a general optimization framework or run an unbounded mixture-weight campaign.

## Matched controls and sufficient evidence

Keep local/NoRead, retained contextual H4 parent, fixed one-nat source reader, old coarse policy as an exposed diagnostic, the new H4 operator and a matched ordinary categorical control. The current categorical source lacks an equivalent contextual confidence head. Fit a matched confidence/controller path from the same development supervision, capacity and allowance if claiming geometry-dependent benefit. Otherwise explicitly label the engineering comparison asymmetric and withhold a geometry-advantage claim. Do not give H4 a privileged hand-authored semantic gate or supply categorical labels to serving.

Reuse frozen local logits and source observations; collect each alternative action once, rather than a full forward pass per optimizer candidate. Save compact reusable per-position or equivalent fully reconstructible statistics with:

- Immutable observation/config/parent identities, sequence/document/window boundaries, selected occurrence/payload positions and parent bucket/D/sign.
- Actual effective integer boosts, CE change, emitted token/correctness and read indicator for every action, parent outcomes and empty-pool local outcomes.
- Offline stratum labels, complete-stream denominators, distinct-document/sequence support and per-document/per-sequence sums for paired uncertainty.
- Fit **and tune** counterfactual arrays used for selection/constraints, not only a fit bucket aggregate. These must reconstruct fitted policy outcomes and every decision criterion without rerunning the model.

Check the loss identity against actual changed logits using the effective saturated boost; counts/argmax must obey served tie rules. Distinguish candidate-event averages from full-stream text loss. If tune constraints inform learning, call tune development and keep final separate. Previously exposed final documents/constructions are development/regression data now. Hash each document and verify availability/separation before split slicing; do not call recycled documents fresh. Choose an honest additional pinned local population within resources. Disclose prior-local-model corpus exposure even when reader-held-out. No large download is needed.

## Integrate the remaining interface repairs into this run

Implement the new causal interface through one target-free inference boundary used by evaluation, generation, interventions and timing. Verify teacher-forced parity without injecting supervised records. The expected-manifest consumer must bind actual source files including new modules, actual executable, tokenizer/local model, reader parameters, observation configuration, fit input boundaries/references and training config. Hash files after the final source write and before execution; record the actual checkout revision plus dirty source snapshot if applicable, not hard-coded stale base revisions. Keep report sealing separate from predictive dependency validation.

Use the verified loader's returned predictor for every served arm. Exercise drifted config, swapped valid policy and incorrect input/dependency expectations through that consumer. The previous loader checks are real but their digest omitted boundaries/references/config/local dependencies, and one fixed-strength arm discarded its returned predictor. Complete that exact boundary; do not claim an entire artifact framework from hash equality alone.

Repair changed-source accounting in the substantive run: exclude the deliberately edited occurrence when counting changed neighbors; count changed/added/removed neighbors, original-source still-admitted versus lost-admission, and NoRead separately. Keep enabled/disabled × original/changed conditions. Targets/strata are reporting metadata only. Preserve valid historical intervention evidence with its original scope.

## One selected evaluation and a useful stop

After development selection, freeze source/config/artifact and evaluate the selected operator once on fresh documents and construction sequences with paired cluster uncertainty. Include actual generated complete responses for present, absent, conflicting, multi-turn and ordinary prose prompts. Generated prefixes can change subsequent source pools; teacher-forced optimization alone cannot qualify rollout. Report parent/new/categorical/fixed/local outputs and causal changed-source/read-disabled effects. NoRead everywhere fails useful answers; reading everywhere fails absence even with small doses. Copying an answer does not establish derived reasoning or sustained prose.

If no useful joint candidate is obtained, retain the parent-preserving interface and diagnose the failure with concrete observations: does D-sign still merge cases needing different influence, does reliable retrieval lack an appropriate derived output, or is the local generator itself inadequate? End this bounded influence integration after its selected result. Do not automatically open another final draw, add a second feature sweep, or hold composition hostage to endlessly refined gating. A scoped negative with a useful preserved component and a concrete next operator is a complete research contribution.

The wider sequence is information-preserving relational attention → learned structural correspondence/persistence when older scope requires it → dependent Read/Update/Emit/Stop and derived outputs absent from all source payloads → broader prose/conversation/executed Rust → qualified scale/energy. These are linked responsibilities, not a compulsory ladder. Reuse exact occurrence/version memory, signed H4/Spin, directed relations, scalar compatibility, retained Hopf fiber and typed paired-H4/icosian primitives where an implemented need exists. S7/spinors, normalized E8 and harmonic coefficient banks remain conditional representation tools with equal-bit controls and explicit assignment/collision/cost models. They do not give free orthogonal syntax or infinite lossless memory. No GR, supersymmetry or twin-prime search is justified without a derived useful operator.

## Resources, preservation and delivery

Read the latest [ledger](resource-ledger-2026-09-19.md) and live `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`. Principal review reconciled the prior prose/live discrepancy and its 14,500 ms component rounding error; never reset those charges. Refresh physical free space and complete wall-time/CPU/threads/RAM/new-temporary-retained-storage/checkpoint projections **before** build, preparation, extraction, fitting, evaluation or retries. Standing local allowance extensions are already owner-authorized: record reason, increment and new cumulative limit before using one; do not ask the same approval again. No paid/external compute is authorized.

After safe cleanup the review observed 38.26 GB free, versus a 36.766 GB reserve plus 128 MiB stop margin; final check may differ. It removed only inactive Rust debug incremental intermediates, preserving executables/dependencies and every unique artifact/research root. Reuse valid builds; set `CARGO_INCREMENTAL=0` and bounded compiler workers when that avoids renewed bloat. Additional paper allowance is not physical free space. Never delete models, source, research, negative candidates, owner work or downloads to force a build to fit. Stop/checkpoint before the configured limits.

Claim report roots exclusively immediately after validation and before loading models; never append to sealed roots or reuse a failed root. Retain exact executed source/config/data/artifacts, measured charges and report verification. Compile/exercise the changed Rust path with meaningful tests for actual causal boundaries, tie/saturation, parent parity, serialization/rejection and the generated behavior. Run fmt/check and named focused tests; no blanket expensive suite. Run `python3 scripts/check_claim_wording.py` on claim changes. Queue compatibility acknowledgements are not tests, short noisy timings are not energy savings, and physical energy stays UNAVAILABLE without measurement.

Update design/result, README, project-track/current-state/CONTINUE/EVIDENCE/model-direction/PROJECT_MAP and any active mechanism/prompt pointers made stale. Update all six owning issues and project items if present, keep broad acceptance open, use References rather than Closes for partial work. Commit named paths, push a protected PR, attach it, verify actual merge and reviewed-tree equality, and preserve the owner's checkout. Import compact revision-pinned knowledge with source links and verify retrieval. End with your choices/disagreements, exact scopes/results/failures, generated behavior, useful retained pieces, source/artifact/merge receipts, resources/free space and one evidence-supported next action.
