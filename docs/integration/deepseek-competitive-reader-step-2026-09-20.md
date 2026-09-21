# DeepSeek next task: make geometric attention discriminate competing sources

> **Completed as PR #1321; historical instructions.** Read the [principal correction](competitive-reader-review-2026-09-20.md) and execute the [contextual-utility successor](deepseek-contextual-utility-step-2026-09-20.md). Do not rerun this prompt as the active task.

This is the single successor to completed PR #1319. You are a research contributor: choose the smallest justified representation, optimizer and fitting dose, explain deviations, and retain useful mechanisms even when a larger hypothesis is negative. The primary outcome is an implemented and exercised **query-dependent single read on competing plausible sources**, through one causal inference function shared by evaluation and generation. Incorporate the named corrections into that constructive task; do not finish with a parity-only replay or another audit.

## Recover the actual project

Refresh origin/main, live issues #973/#820 and relevant PRs, worktree state, artifact identities, the shared model ledger and storage receipts. Use an isolated full worktree and preserve owner and other active checkouts. Read AGENTS.md, DECISIONS.md D0-b/D1/D2/D3, current-state and canonical project-track, then read these completely:

1. [Principal review of PR #1319](relational-reader-review-2026-09-20.md).
2. [Current/retired mechanism and mathematical synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md).

Read the executed runner/module and original design/result as evidence, not as instructions that override this successor. PR #1319 head `ae1e0fbd` merged as `bfa13a91`, with equal trees. Its saved prototype is worth preserving. Obtain additional history and primary research as useful; inspect actual donor source before adoption. The goal remains learned transformerless geometric language, conversation/memory and coding/reasoning on consumer hardware. Offline Rust training may use float/matmul; D0-b serving permits bounded <=4-bit additive/shift/table maps without numerical-kernel multipliers or served float/transcendentals. No hidden teacher responses, authored serving parser or transformer backbone.

## What is established, and what must change

The relational prototype improved synthetic candidate-position hard-action CE from the exact reader's -1.082268 to -1.499668 bits relative to frozen local, and correct reads from 176 to 199. Preserve that scoped result. The +0.098712 interval is covered **decision success**, not the conditional-precision difference. Its -0.217206-nat interval measures a soft action-policy expectation, not deterministic serving loss. The -0.417401-bit hard-action advantage has no paired interval yet. Natural text was neither fitted nor scored for CE in that run.

The categorical model was trained with `(7*code[q]+code[k])%120` and evaluated at H4 addresses; its reported loss is invalid for attribution. Only geometry got learned-code refinement. Every intended geometric answer came from a paired-role bank while distractors came from a solo bank, admitting a source-class shortcut. Q is a static preceding-token lookup. Positive strength is globally 8 nats because the action score factorizes. Generation uses an earlier candidate-bearing query, and two outputs are empty. These are the reasons to improve the experiment, not reasons to discard H4 or return to a local-only model.

## Build one shared target-free inference boundary

Create or factor a small current-prefix read/predict operation used by teacher-forced evaluation, actual autoregressive generation, interventions and timing. It must:

- Operate on the actual last observed token and prefix, including prefixes with no candidate. Emit the local prediction in the latter case; do not silently stop generation.
- Require no target/sentinel future token or precomputed supervised `Pos` to make a decision. Compute losses/labels outside the inference function after choosing the action.
- Preserve actual session/absolute occurrence identity, selected source context and directed relation. Resolve the real observed payload reference, not an index fabricated by `cands_of`. Define write/read order and successor-availability conventions; demonstrate they match across entry points.
- Use the same declared relation encoding in training hard-forward, inference, export and reload for **every arm**. Bind mode, code map, exact table, parameters, configuration, tokenizer and actual observation/data identities. A portable generic comparator may use an explicit finite lookup rather than forbidden arithmetic in its declared serving kernel.
- Separate local scoring from ring/admission/scoring work so the cost baseline truly bypasses reader work. Count actual predictions and all scanned records; serialized bytes are not resident bytes.

Repair the independent S-query-only check using the retained scorer's defined absence behavior. Compare the predictor at the changed future token's predecessor as well as earlier positions; require unchanged score/action/reference/payload, including no-candidate cases. Export tests must distinguish categorical and H4 indices on inputs where they differ and verify each arm, rather than reloading one already-wrong function. An altered-source test records both selected payload and actually emitted output. Repair the CI sign check to use the upper bound for a negative-loss interval. These are focused correctness tests, not a broad historical test campaign.

Fix descriptor refinement: deduplicate affected positions, include the current root/loss as incumbent, preserve ties and accept only strict improvements with a declared tolerance. Verify nonincrease of the exact full position objective, including a repeated-role example. Name initialization explicitly: old slot labels interpreted as group IDs versus actual `palette.elements[a_codes[t]]`. Changing that choice is a new configuration, not a silent repair of the old artifact. Exercise actual disk continuation if modifying/using the resumable fitter; do not repair unused legacy CPQK machinery.

First exercise a tiny shared-path fixture before fitting, so semantic generation/evaluation parity is caught early. Re-evaluate frozen old panels only as needed to preserve comparability and separate evaluator corrections from retraining. Old panels are inspected regressions; a new metric on them is not fresh validation. Do not overwrite `relational-reader-1`; its current root has an unlisted `summarize.py` in addition to the original four-file manifest. Preserve it and record current inventory honestly. Put corrections/derived files in new claimed roots and seal/verify them.

## Construct a discriminating learned-attention task

Use a compact fit/tune/new-evaluation construction in which **several plausible sources compete**:

- Include multiple sources with the same key and paired roles from different families; an incorrect paired source must be as plausible by source class as the correct one. Balance role membership, recency and placement against correctness.
- Hold the memory fixed and change the query so that a different stored value becomes correct. Also hold the recent query tail fixed while changing the relevant older binding/payload. Keep payload membership fixed in selected counterfactuals where that helps isolate query-dependent choice.
- Include absent answers, conflicting or revised bindings with a stated task rule, repeated keys, filler, and held-out payload/key/context combinations. Report how many examples actually require the proposed distinction.
- If claiming direction/order, use a task whose two directions or role orders have different answers; the old symmetric pair relation cannot prove this. If claiming nonidentical-key access, include supported cases and report admission coverage. Do not require every mathematical property to win a separate panel before useful attention can advance.
- Define answers independently of learned roots/codes. Do not add a gold source to admission or pass task-family/coverage/answer labels to the serving policy. Do not demand inference of a fresh arbitrary role-pair mapping with no training or prefix evidence; that mapping is unidentifiable.

Start with the 128-token ring and <=24 admitted candidates, keeping one shared pool for ranking comparisons. Inspect whether it supports the new examples before fitting; if the intended relation is excluded, one small justified causal proposal correction is allowed, with coverage/cost contribution measured separately. No ring/candidate sweep is requested. Most old examples admitted at most five candidates; label cost extrapolations accordingly.

Choose the simplest learned contextual descriptor that can express the cases. Existing one-token Q is an available baseline. If the same lexical cue takes different roles/scopes under identical existing observations, show that collision and introduce the smallest distinguishing causal context, possibly an ordered two-lane H4 description using neighboring observations. Learn the mapping and shared compatibility from the objective; don't insert a hard-coded role parser. You may reuse source_routing.rs, relational_attention/runtime.rs and hamming_refinement/runtime.rs for directed action and exact-reference interfaces, preserving their historical artifact/scaffolding distinctions.

## Comparators and objective

Keep frozen E plus S-query-only local emission. Compare local/NoRead, a competent exact-context reader, a properly trained/evaluated learned categorical or other small generic contextual alternative, and the H4 candidate. Add a cheap **query-blind source-only** baseline or intervention to expose the paired-bank shortcut. Choose one useful generic alternative rather than a zoo; state parameter/representation/optimizer differences and account for code learning in both arms when attributing a benefit to group structure. Equal table size with one learned code map and one frozen random map is insufficient. A useful geometric implementation can be retained without claiming unique-H4 superiority.

Judge the exported deterministic action with actual next-token loss and emitted behavior. Soft expected action loss remains a valid training surrogate/diagnostic, but report it separately. You may train against that surrogate, cost-sensitive finite actions, or another justified offline loss-aligned method. No new reward model or RL subsystem is needed for observed one-step costs.

The present `candidate_score + sb[a]` necessarily chooses a global nonzero amplitude. A global gain is a valid cheap baseline. If contextual strength is intended, introduce one small bounded interaction with causal candidate/query/local-score observations, e.g. an action-specific finite table or low-bit term. Compare its usefulness and account for bytes. Do not feed actual coverage/correctness to it. A raw local margin is not a calibrated probability, and rescaled quantized policy logits change the softmax distribution even when the hard argmax stays fixed.

Include a bounded source-separated natural-text fit/tune/regression component, with all-position losses and reader participation, so synthetic usefulness does not silently harm the language predictor. Reuse available pinned local text; declare repository overlap and do not call inspected docs a fresh external benchmark. Choose and report the mixture/weighting; keep newly held-out construction and natural-text evaluation separate from fitting. No large download or corpus scale-up is needed. If available data cannot support a fresh natural-text claim, state that clearly and use an honest development regression.

## Decide without an arbitrary near-miss treadmill

Before seeing the new evaluation, state one practical rule based primarily on **hard served predictive loss**, useful generated query/source counterfactual behavior and the declared natural-text/cost tradeoff. A reasonable formulation is a meaningful paired hard-loss improvement with a correctly signed interval and no material regression on the stated language population; choose the actual margins from fit/tune needs and available precision, not from new test outcomes. Report absolute quality as well as gains over the weak local predictor. Use sequence/document clusters appropriate to the data; one seed's data interval does not establish stability across training seeds.

Retain separate denominators for eligible positions, admitted support, covered positions, reads, correct payloads and emitted correctness. Treat unavailable sources and ambiguous compatible payloads honestly. All-NoRead is a baseline, not attention advancement. Correctness, causality, exact identity and truthful provenance are invariants; quality screens may change prospectively with rationale. The old 0.15 margin and `positive=false` remain historical. Do not rebrand a rerun of the old frozen panels as prospective evidence under a new rule.

This run should produce a functioning current-prefix reader and evidence of whether it uses the query to distinguish competing sources. A few bounded corrections are appropriate; stop repeated threshold/width tuning when the representation or data is the bottleneck. Preserve partial gains. If the H4 arm is useful but the generic alternative matches it, report that and keep the useful mechanism without a uniqueness claim. If a witnessed observation collision blocks learning, preserve the implementation and identify one concrete next representation change.

## Place in the larger roadmap

We remain in Stage 1: useful query-conditioned access. Stage 2 adds structural role/scope retention when needed; Stage 3 learns dependent reads and shared Read/Emit/Stop; Stage 4 develops broader language and executed Rust; Stage 5 qualifies useful scale and full machine cost. Profile costs throughout.

Finite H4 relations already exercise the reduced Hopf/spin idea. Retain the relative element and exact reference in the read result, providing an interface for the later query update. S7/E8 or mutable harmonic coefficient banks are conditional tools for a measured representation/retention limit, not required replacements for a broken evaluator. Existing direct slots and equal-memory alternatives remain useful controls. Do not add a full two-hop scheduler or novel-output operator in this run: dependent retrieval, scheduling and derived lexical computation are distinct capabilities and should enter with an attributable first-read foundation. This sequencing does not require unique H4 superiority before composition; it requires a working useful query-dependent read.

## Resources and delivery

At this review the live shared ledger is `182138565 / 191300000 ms`, about 152.7 minutes remaining. Refresh it; it is not a new allowance. A starting complete projection is <=90 minutes for preparation/build/fit/controls/evaluation/corrections/checkpoints, one model worker, <=8 GiB RSS, <=512 MiB new data/reports and <=2 GiB incremental build. Measure a small step before longer fitting. Respect the 128 MiB protected margin and existing machine reserve, and record actual complete debit once. Necessary local extensions are owner-authorized: record reason/increment/new limit before use. No paid/external compute or deletion of unique material follows from that authorization. Preserve all original artifacts and checkouts.

Compile/exercise the changed Rust path and meaningful focused arithmetic, causal, serialization and interface tests. Use one reviewable branch, stage named paths, push and deliver through a protected PR with References #973/#820 and relevant owning issues. Verify actual merge/tree content. Update README, current state, canonical plan, evidence, continuation and live issues/project items with measured scope; do not close broad capability issues. Record free space and any authorized cleanup separately. Queue acknowledgements are not test results.

Conclude with the chosen implementation and rationale; actual outputs; hard/soft loss and all relevant denominators; controls and unresolved limits; artifact/source/data identities; complete resource balance; delivery receipts; and one justified next action. Use the freedom above to make a scientific contribution, not to reproduce a broken control or force a positive label.
