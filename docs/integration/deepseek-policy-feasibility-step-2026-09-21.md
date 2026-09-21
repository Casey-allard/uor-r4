# DeepSeek execution: settle finite-policy feasibility and advance the missing operation

**Executed by PR #1332.** The unchanged coarse class is infeasible on retained development counts. Read the [principal review](policy-obstruction-review-2026-09-21.md) and [next confidence-interface prompt](deepseek-reader-confidence-step-2026-09-21.md); this document is the historical task specification.

You are a research contributor to the UOR-R4 Geometric Language Model. Execute the next constructive experiment, deliver it through a protected PR and update the project state. Use your mathematical and engineering judgment; challenge this formulation if a cheaper, sounder test answers the same question. Explain substantive choices before inspecting new final outcomes. Do not reduce this task to writing a plan or repairing diagnostics without answering the learning question.

## Recover context and authority

Refresh `origin/main` and read AGENTS.md, README, project-track.md, current-state.md, model-direction-2026-09.md, DECISIONS.md D0-b/D1/D2 and the execution policy. Read these documents completely:

1. [Principal review of PR #1330](policy-objective-review-2026-09-21.md).
2. [Latest prospective design](reader-policy-contract-design-2026-09-21.md) and [result with principal correction](reader-policy-contract-result-2026-09-21.md).
3. [Independent saved-data audit](../evidence/policy-objective-principal-review-2026-09-21.json).
4. [Current/retired mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) and [Hopf/spin direction](structural-memory-hopf-direction-2026-09-20.md), using their current sequencing and source pointers rather than treating every historical task as active.

Inspect the actual Rust mechanisms in `crates/uor-r4-core/src/native_geometric/learner/relational.rs` and `crates/uor-r4-core/src/bin/competitive-reader.rs`. Use project knowledge search/get_source and scoped history for missing context. Inspect donor source before using it. Seek relevant primary research when a consequential mathematical choice is uncertain. Coordinate read-only independent checks with bounded tasks if useful; avoid redundant broad audits.

Reviewed parent PR #1330 merged as `4be53d88e7346d32d9a6e0a4584bb3cbc6f813a9`, tree `101437725408fd023092eb6b3b859c9edf0bb46f`. The principal review delivery will be later; verify its merge/content and current HEAD rather than assuming this parent is latest. Preserve the owner's checkout at `/Users/casey.allard/uor-r4`; use an isolated full worktree and a `codex/` branch. Refresh #820/#973/#1139/#962/#963/#964 and their project items. Historical negative artifacts and sealed reports are evidence, not cleanup targets.

Rust preparation, fitting and artifact construction may use float/matmul offline. Final serving retains D0-b's bounded <=4-bit additive/shift/table operations and geometric focus; no floating-point numerical serving, hidden transformer or provider responses. Do not apply the older workflow skill's blanket prohibition in place of current owner decisions. Keep whole-path claims scoped to executed checks.

## What is established

The previous feature-index bug is fixed. Fitting and serving share configured gap thresholds; both changed source hashes match the merged source. The corrected H4 policy harms reader-held-out text by +0.119319 bits/token [+.070211,+.167862], versus the contextual parent's +.453197 and fixed one nat's −.014769. It fails all four declared categories: present-query emissions 56/121 versus parent 77/121, absent reads 19/19 versus 5/19 and worse absent loss. Both H4 and parent select the correct payload on 98 present queries; source correctness alone does not explain emitted preservation. Generated prose remains degenerate.

Preserve PR #1328's old artifact result of −.019477 bits/token. The correctly indexed successor is a different policy. On those same old documents fixed one nat yields −.025483, so learned-table advantage is unestablished. Attempt 4 repeats sealed attempt 3's final data and numerical outputs after a serialization-probe repair; it is not a second independent final draw. Reader-held-out documents were already in the local prior corpus.

The corrected objective chooses strong actions because they lower its mixed candidate-event CE. That is an observed tradeoff among tested policies, not proof that an objective change solves it. H4 has 13 unobserved and three below-support buckets. The stored costs/support cannot answer joint feasibility: alternative-action emitted correctness and per-stratum losses/reads were not saved.

## Concrete objective and stop condition

Determine whether **one deterministic action table over the existing causal observations** can improve text while preserving useful present-answer and absent-query behavior. Close this question in this run. Do not sweep mixture weights, widths, candidate bounds or new features indefinitely. A valid infeasibility result or bounded unresolved solve is useful if its scope and missing evidence are explicit.

Initially freeze E+S, H4 and categorical source maps/rankers, exact occurrence addressing, ring 128, candidate limit 24, gap thresholds, feature semantics, four actions `{NoRead, 0.0625, 1, 8 nats}`, and the support/fallback rule, including the supported-bucket mask and actual fallback opcode. Match H4 and categorical treatment. A separate diagnostic with fallback constraints relaxed is allowed on the same saved development statistics, but must be labelled as a wider policy class, not quietly substituted for the declared candidate.

Before work, write one prospective design note naming the objective, domains/strata, normalization, constraints, support handling, solver algorithm/time cap, candidate-selection rule and fresh evaluation populations. The prior +0.05 bits/token text harm screen and +0.05 bits/present-query preservation margin remain reference criteria; present emission allowed at most two fewer correct answers on the declared equal-sized acceptance population, and absence requires no increase in reads or loss. Translate count margins prospectively if population size changes and report exact rates/counts. You may propose a justified practical-margin change before final data are opened; preserve the historical decision and do not relax away a known large regression.

## Collect the missing statistics once

Extend the existing Rust prepared-observation pass to record all four counterfactual actions on **development** prefixes. Reuse frozen local logits and selected sources; do not run full model arms for every optimizer iteration.

Retain a compact table or equivalent sufficient statistics carrying:

- Immutable feature-config identity; bucket; selected exact occurrence and payload reference; document/sequence/window identity and boundaries.
- For every action, actual next-token CE change, actual emitted token/correctness using the served integer tie and saturation rules, read indicator and effective quantized boost.
- Offline domain/stratum labels, including final-present and final-absent construction queries; complete-stream counts including empty pools; raw and weighted support and distinct contributing documents/sequences per bucket.
- Parent outcomes on exactly the same populations; all quantities needed to reconstruct constraints and uncertainty without another forward pass.

Targets, coverage, correctness, source-role labels, authored strata and future tokens are allowed only as offline supervision/reporting. They never enter the serving feature address, candidate choice or action. Empty pools are fixed local outcomes. Verify the score-space identity `log(1 + p*(exp(a)-1)) - a*I(payload==target)` against actual logits with the effective boost; do not silently model an unquantized action that serving cannot execute. Preserve the direct actual-action regret identity and add a per-position decomposition assertion at its consumer.

Use the original fit pool for optimization and the tune pool for development selection/checks. Previous final populations are now regression/development data, not fresh acceptance. If tune constraints participate in optimization, explicitly call tune part of development and keep final disjoint. Use complete-stream denominators; specify document-balanced or token-weighted text loss. Equal candidate-event weights must not be described as equal full-stream domain weights.

## Solve the joint problem, with a bound on work

For each bucket b and action a use binary `x[b,a]`, with `sum_a x[b,a]=1`. Recommended objective:

`minimize complete-stream development text loss`

subject to:

- present-query emitted accuracy >= parent minus the declared preservation margin;
- present-query CE <= parent plus the declared margin;
- absent-query read count/rate <= parent;
- absent-query CE <= parent.

Also report whether the optimum/incumbent meets the text harm screen and improves over local. Present-only emission constraints are insufficient: they can preserve argmax while worsening probability or absence. Domain labels choose offline coefficients only. Keep the matched categorical comparator, frozen contextual parent, local/NoRead and fixed one-nat source comparator.

Begin with cheap optimistic bounds: maximum present emissions if each position chose its own action; then the tighter one-action-per-bucket bound; then the joint operational problem with support/fallback restrictions. These may expose an impossible floor or a fallback obstruction without expensive search.

Choose an appropriate bounded Rust-compatible exact search, branch-and-bound or available solver. The table is small but the naive space is 4^32. Do not enumerate blindly, import a large framework or build a general optimization platform. Set a solver wall-time cap from the resource projection, keep an incumbent and objective/lower bounds, and report numerical tolerance. Validate the chosen table by independent replay of saved sufficient statistics and then the real predictor. Offline float optimization is permitted; use conservative error bounds for any infeasibility claim. A timeout is `UNRESOLVED`, not `INFEASIBLE`. A feasible LP relaxation or stochastic mixture is not an accepted deterministic policy; an infeasible relaxation can rule it out when its bounds are sound. A finite Lagrange multiplier sweep is not a Pareto certificate.

Fit H4 and categorical tables under the same objective, margins and solver allowance. If selecting between multiple feasible incumbents, use a fixed development rule and account for the selection. Do not inspect final results to choose the winner. If infeasibility occurs only with fallback frozen, disclose it; relaxing unsupported entries does not create evidence for their generalization.

## Integrate remaining control repairs into this experiment

Do not schedule an audit-only predecessor. Implement a small expected-manifest loader around the existing artifact reader that verifies actual artifact-byte hashes, configured feature identity, fit input/dependency identity and source/local dependencies **before returning a predictor**. The existing swapped-policy test only compares bytes, and wrong-data test compares to a dummy digest; make actual drifted config, swapped valid opcodes and wrong input identity fail through the consumer used by evaluation/generation. Include sequence boundaries and occurrence references in fit identity; bind final evaluation independently in its receipt. Avoid cyclic self-hashes and do not confuse report sealing with predictive input validation.

In selected-source intervention accounting, exclude the deliberately edited occurrence from neighbor-change counts; compare added/removed neighbors too. Define lost admission by whether the original exact occurrence remains in the candidate set, separately from missing observation construction and from choosing NoRead. Retain all four enabled/disabled × original/changed conditions. Preserve correct still-admitted results from the parent rather than erasing them.

## Evaluate once after selection, or deliver the obstruction

If a deterministic development incumbent satisfies all behavioral constraints AND the declared development text screen/selection target, freeze artifact/source/config hashes and make one fresh reader-document and construction-sequence acceptance run. If a sound lower bound shows that even the constrained text optimum misses the declared text screen, deliver that obstruction without an unnecessary final pass. A solver time limit can still leave a valid feasible incumbent; evaluate it only if the prospective selection rule permits bounded suboptimality, and label optimality UNRESOLVED. The 32 used document hashes are development history now. Only four documents remain in the old 36-document pool; do not silently call eight recycled documents fresh. Select additional eligible documents from the pinned local corpus or use a prospectively justified smaller honest final population, within resources; disclose prior-model exposure and any loss of precision. No need for a large download. Check document availability before any split slicing; the parent checks its fourth slice too late. Preserve fit/tune/final separation by document, not windows. Save per-document and per-sequence sums so intervals are independently reconstructible.

Report text loss, present-query CE and emitted answers, absence reads/loss, whole-construction metrics and the old retained controls, with the correct units and paired cluster uncertainty. Keep harm containment, useful transfer, relational preservation and absence behavior separate. NoRead everywhere cannot pass present preservation; reading everywhere cannot pass absence merely by using a small dose.

Exercise the same target-free inference boundary on generated complete responses, including absent and conflicting queries and natural text. Teacher-forced additive optimization cannot certify rollout: generated prefixes change later sources/buckets. Report changed-source and read-disabled effects on completed behavior. Do not claim reasoning from copying, prose from nonempty output, or unique geometry from a single point estimate. Compare the matched categorical arm honestly even if it wins.

If the problem is infeasible, unresolved within the declared solver cap, or the chosen candidate fails the fresh evaluation, end this same-feature scalar-policy campaign. Preserve the candidate/statistics and identify the smallest evidenced missing distinction, with concrete conflicting examples or bounds. A solver timeout alone establishes only a computational limitation: report its incumbent/bound without inferring missing information or a geometric defect. Any architecture recommendation then requires independent observed examples. No automatic second final draw. Your judgment is welcome in identifying the successor:

- Existing absolute source/NoRead confidence, directed relation identity or local normalization may be missing from the current coarse observation. Inspect these before assuming persistent state is necessary.
- If identical recent observations diverge only after older scope changes, recommend a learned finite contextual H4 transport/role state and a matched ordinary-state comparator.
- If useful payloads are absent, target the specific admission/retention mechanism.
- If retrieval is correct but a derived answer cannot be copied, recommend shared read–update–emit and dependent composition.
- If memory controls improve but prose remains degenerate, retain useful memory and address the local generative learner rather than refining the gate forever.

Do not implement an unrelated second architecture or repeat gate tuning in this run. A concrete failure analysis and next implementation boundary complete the task when the present class cannot meet it.

## Keep the full geometric roadmap in view

The goal remains a transformerless native geometric model, not a better copy filter. Signed H4/Spin, exact prime/occurrence identity, retained versions, scalar compatibility, structural lifetimes and typed operators are reusable. The sequence is useful contextual influence → structural correspondence/persistence as needed → dependent reads and derived output → broader prose/conversation/executed Rust → qualified scale and energy. Stages are responsibilities, not a demand to perfect an inadequate controller before developing a missing operator.

Retained Hopf fiber, finite S7/spinor or normalized E8 representations and harmonic channels remain conditional candidates for a witnessed relation/interference limitation, with equal-bit ordinary controls. They do not supply free orthogonal syntax or lossless infinite memory. No GR, supersymmetry or twin-prime machinery is required without a derived useful operator. Read primary sources linked by the principal review; distinguish inspiration, mathematics, actual Rust implementation and measured advantage.

## Resources, preservation and delivery

Refresh the live JSON at `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json`. At review it was **192996749/194900000 ms**, leaving **1903251 ms (~31.72 minutes)**. This is a historical snapshot, not a new allowance. Before any build/preparation/extraction/solve/evaluation/retry, project complete wall time, CPU/threads, RAM, temporary/new/retained storage and checkpoint/stop margins; charge all work. Necessary local extensions are owner-authorized: record reason, complete projection, increment and new cumulative limit before use. Do not ask for that approval again. No external paid compute is authorized.

Inventory at review showed **38.54 GB free**, only **1.77 GB above the 36.77 GB reserve**, plus the 128 MiB stop margin to preserve. Reuse valid release builds and bounded scratch; an inherited 6 GB build-growth estimate no longer fits. Refresh with the established inventory tool. Do not delete unique model parents, negative reports, research, downloads or intentional work to make room. A necessary local allowance extension does not create physical disk space or authorize destructive cleanup.

Claim each report root exclusively before loading a model; never append to sealed roots. Preserve attempts 1–4 and all parents. Bind actual executed source/executable/tokenizer/config/data/artifact identities, measure actual charges, and seal/verify the complete report set. A report-only repair replay keeps its original data-exposure label.

Compile and exercise the changed Rust path with meaningful focused causality, action arithmetic/ties, constraint reconstruction, solver bound, serialization/rejection and generation checks. Use cargo fmt/check and relevant named tests; no blanket expensive suite. Run `python3 scripts/check_claim_wording.py` for claim edits. Retain the exact tests run and unavailable boundaries; queue acknowledgements are not tests. Physical energy remains UNAVAILABLE without a real measurement.

Update the result/design, README's current claim, project-track/current-state/CONTINUE/EVIDENCE/model-direction/PROJECT_MAP and any active pointer made stale by the result. Update six owning issues and project statuses if present; partial work references issues and keeps broad acceptance open. Commit named paths, push a protected PR, attach it to the task, and verify actual merge plus reviewed-tree equality. Preserve the owner's checkout. Import a compact revision-pinned knowledge record and source links; verify retrieval. Final report: what you chose and why, exact observations/constraints/solver status, useful retained pieces, generated behavior, all failures, artifacts/merge, resources/free space, and one evidence-supported next operation. Disagree with the principal sketch when evidence warrants it, with a precise explanation.
