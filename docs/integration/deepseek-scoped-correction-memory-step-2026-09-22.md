# DeepSeek next milestone: correction-aware conversation memory across unfamiliar entities

September 22 UTC, 2026. Principal architecture decision after the [ordinary-form review](ordinary-form-binding-review-2026-09-22.md). This brief supersedes the preceding brief's scheduling after its argument-binding milestone; it does not erase earlier artifacts or broaden their claims. References #973, #1139, #962, #963, #964 and programme #820.

## Your role and the outcome

You are a capable autonomous research contributor. Codex remains the principal mathematics/ML/systems architect and reviewer; you own substantive design, implementation, diagnostics and complete delivery of the following useful capability. Use first principles, evidence and relevant research to challenge the proposed implementation when a better one is justified. Do not stop at a plan, compiling helpers or a single successful fixture.

Build **one loaded native conversation-memory path** that learns to ingest an ordinary assertion about an unfamiliar entity, answer from it, accept an explicit correction, answer the appropriate current/historical question, retain independent scopes, and survive a real save/reload. Dependent references must continue through the same owned causal read/continue/emit engine. This is a complete observe → query → correct → query → restart milestone, not an omnibus request to add Q8, prose and every geometric representation in the same run.

The project goal remains the UOR-R4 Geometric Language Model: a local transformerless model for conversation/memory and coding/reasoning, ultimately competitive capability and lower energy on consumer hardware. Rust offline training may use floating point and matrix multiplication. The final serving target is owner D0-b: learned linear-map coefficients at most four bits, executed through permitted additions/subtractions/shifts/lookups, without multiplier instructions or floating/transcendental arithmetic in the declared kernel, with geometric routing/shared typed operators preferred. Indices, record IDs and accumulators may be wider; coefficient width is not a four-bit limit on all state. Existing experimental wide-score or table-construction paths are not thereby qualified. No hidden dense transformer/provider, Python model, grammar interpreter substituting for learned language, or gold serving annotations.

## Recover context without losing the live branch

1. Refresh origin/main and live PR #1344. The returned ordinary-form submission was `5d0a770c`; the principal source correction is recorded in the linked checks receipt. Verify actual merged source/tree, not a PR title, queue acknowledgement or old chat. If merge is pending, use the exact corrected reviewed head with its full parent history; do not rebase away review/source dependencies. Preserve the owner's original checkout and unique worktrees.
2. Read this entire brief and the entire [principal review](ordinary-form-binding-review-2026-09-22.md), then AGENTS.md, DECISIONS.md D0-b/D1/D2, execution policy, canonical project-track, current state, model direction and project map. Read the relevant result/audit/check receipts rather than treating their headline as the whole finding.
3. Recover project knowledge with literal searches for `ordinary-form`, `scoped correction`, `same-value`, `initial previous current`, `query participation`, `grounded session` and `Q8`. Follow source-linked records to actual current code/artifacts. If MCP access is unavailable, the local `uor-knowledge` CLI and repository sources remain available; document retrieval gaps. Do not mistake a stale memory snapshot for current authority.
4. Inspect the concrete donor mechanisms listed below. Reuse the architecture audit for wider navigation; do not spend the run repeating a broad census or importing every older model artifact.
5. Read and project against the live shared time/storage ledger before expensive work. Check actual free space before builds. Necessary local allowance extensions are already authorized when recorded prospectively; preserve prior charges and the physical reserve plus128MiB stop margin. No paid/external compute. No arbitrary short timer or fixed retry quota constrains thoughtful diagnosis.

## What we now know

The ordinary binder learns a joint hypothesis `(subject span, cue span, cue role, optional object span)` with singleton background scores and cue-sided argument roles. The tested bounded authored forms include interrogatives, possessor assertions, object-first assertions, trailing nonanswer text and redirects. Exact lexical byte identity joins different BPE forms; surface extents still own source provenance and emission.

Both categorical and H4-hybrid arms achieved48/48 development,40/40 exposed regression and24/24 final-composition answers. The final panel first appeared in historical root7; root8 is a byte-identical worlds/rows replay for source binding. It shares known vocabulary/forms and all question strings with training, so it is not new-entity or novel-syntax qualification. The principal corrected execution is also exposed; see its own receipt for matched control and intervention counts.

The principal fixes enforce16-bit feature-token bounds, accumulate structured cue scores in i64 without premature clipping, strengthen independent decode/update checks, preserve all non-object text during source edits, and add an exact lexical-membership comparator beside the preserved historical token comparator. Preserve those fixes. Do not return to a clone-and-patch loader, stale text/alignment, target-conditioned serving or report-root reuse.

The H4 hybrid has learned potential weights but its49 descriptor codes remain initialized. Its strict exact-hit code search starts at96/96 and cannot accept an improvement. Zero moves is ceiling censoring. Use the categorical binder as the primary interface for this milestone, retain H4's artifact, and avoid another uninformative sweep. You may revisit representation if a concrete transfer, compactness, interference or inference-cost problem supplies a discriminating objective.

## The next obstruction, from source rather than speculation

The current read step admits matching lexical subject/goal clauses and selects by joint parse score. The fixture guarantees one matching fact. A stale assertion can win against a correction simply because it parses more confidently. Adding a constant c(x) to every hypothesis score for one clause leaves that clause's parse unchanged but changes cross-clause ranking. Therefore syntactic score cannot define source authority or current version.

The immutable document binding also makes an update a different runtime rather than a supported persistent operation. We need learned interpretation feeding exact versioned storage, with one stable read view per answer.

## Proposed design; improve it when justified

Separate three responsibilities explicitly:

- **Learned observation/intent:** infer entity/value extents, relation type, assertion/correction/conflict intent and current/previous/initial request intent from actual text and permitted causal context. You may extend the joint structure, share argument features, mask lexical identities during fitting, or use a small learned typed action table if that improves transfer. The final unseen entities and values must not receive fitted semantic entries or a host dictionary assigning their role.
- **Exact versioned memory:** address records by `(scope, entity lexical key, relation type)`. Retain exact record ID, predecessor, source/version, owned payload and update action. Version order, equality, serialization and predecessor traversal are deterministic infrastructure; there is no need to approximate them with geometry or learn arithmetic monotonicity. Use injective typed/length-delimited encoding, not delimiter-concatenated fields that can collide.
- **Causal read and complete emission:** select eligible records under one pinned read view, then reuse learned dependent continuation and exact captured output. Keep source selection, scope/version eligibility and generated answer correctness separately observable.

Host-authenticated user/project scope is acceptable input. A host semantic parser, supplied correction flag, expected temporal intent or preselected fact is not acceptable evidence of learned behavior. If you initially use an explicit scope handle, state that boundary and do not claim natural-language scope inference.

Choose the smallest principled update semantics that handles the requested examples. Before fitting, write down:

1. When an assertion introduces a record, when an explicit correction supersedes it, and how a contradictory bare assertion is represented.
2. Whether `previous` means previous assertion or previous distinct value. Preserve the distinction; do not silently deduplicate same-value reassertions and lose history.
3. How current, previous and initial requests behave on missing history, conflict, reassertion and explicit correction. Keep an independently authored typed oracle outside serving.
4. For a historical dependent request, which relation's history is selected, and whether later hops resolve at that selected record's revision or the request's pinned read view. Do not silently apply `previous` independently at every hop.
5. How source ownership and pinned views behave if an update arrives between dependent hops or after a payload is captured.
6. What happens at declared capacity: preserve/pin required evidence, or return a typed resource/lifetime result. An unfollowed link is not evidence that the source was evicted.

Start with one local writer and deterministic committed revisions; do not build a distributed replication protocol. A pinned read view should prevent a result assembled from incompatible pre/post-correction versions. A new query sees the new committed state; an in-flight read follows its declared view. Keep origin liveness separate from captured payload validity.

## Reuse the project's work

Read `crates/uor-r4-core/src/native_geometric/learner/observed_text_session.rs` and its runner adapter first. Reuse the loaded model boundary and actual session rather than adding a parallel host loop.

Then inspect:

- `native_geometric/relation.rs`: exact record IDs, predecessor links, current directory, learned update action and conflict representation. Its owner-only key, old learned artifacts and fixed capacity are not automatically appropriate for scoped relation addresses.
- `native_geometric/durable_memory.rs`: scope/checkpoint envelopes and historical durable use. Do not adopt its host-fact fallback, one-word assumptions or host pronoun behavior as proof of learned ingestion. Check exact scope-key encoding before reuse.
- `native_geometric/dependent_language/query_participation.rs` and `scheduling.rs`: required query evidence versus replaceable content, source lifetime and continuation. Reuse responsibilities, not old authored grammar rules.
- `native_geometric/learner/grounded_session.rs`, the later relational-session source and their principal reviews: owned SourceLease and Q8 factorization for the next consumed-computation milestone. A compute state that never affects a later read or answer is not integrated reasoning.
- The retained E/S local predictor and prior reader integration negatives when designing future language integration. No need to load/score all historical artifacts during this step.

The source-linked architecture/mathematics/import audit and geometric-attention synthesis hold the wider toolbox: exact prime/UOR identity, ordered n-lets, fixed zeta phases, R4/S3/H4 transport, signed orientation, retained Hopf fiber and the coupled icosian/E8 bridge. Keep identity separate from semantic proximity. A correction is generally not reversible group transport.

## Useful development and final evaluation

Build readable development interactions, with diagnostics available during fitting. New entity/value transfer belongs inside the same useful multi-turn lifecycle, not a separate indefinite campaign.

Cover the interacting cases with enough counterexamples to distinguish the intended mechanism:

- Questions and one declared nonasserting input must produce NoWrite; an office correction must leave the same entity's project unchanged. This is bounded ingestion evidence, not a requirement to solve general quotation/discourse.
- Same request before/after an explicit correction, including a corrected intermediate dependent link and a corrected terminal value.
- Same entity/relation bytes in two scopes with different values; unrelated-scope updates must not change the answer within retained capacity.
- Current/previous/initial behavior with same-value reassertion and multiple distinct corrections; document the chosen historical meaning.
- A syntactically high-scoring stale fact competing with the eligible current version, and reordered record storage. Authority must not depend on sentence confidence or physical vector order.
- Multiword/shared-prefix and cue-overlapping entity/value strings absent from fitting, including actual different BPE boundaries. Report lexical and structural novelty separately. You may keep familiar forms/relations initially.
- Read view pinned across an intervening update, owned capture across replacement, save/reload between operations and during an answer. A fresh process/real disk reload should demonstrate restart behavior, not only cloning an object.
- Missing required facts, conflict, cycle and explicit capacity/eviction outcomes. NoRead/UpdateDisabled and same-source changes should reveal whether output depends on the intended evidence/update.

Keep current primary controls and preserve row-level regressions. Add the most informative matched controls, such as unscoped storage, latest-physical-record selection, history-disabled or update-disabled behavior, on the same observed inputs and byte/capacity boundary. Do not demand every conceivable ablation. A deterministic exact-storage reference diagnoses bookkeeping; it is not proof of learned intent. A control denied needed identity information is not a fair learned-versus-geometric comparison.

After selecting the design and training recipe, create one genuinely new final population with unseen entity/value strings and held-out interaction combinations. Record novelty intersections and exact model/tokenizer/data/source identities. If a final failure informs a design change, retain it as exposed and draw a new final after that change. Replaying an identical final to fix provenance is a replay, not another independent win.

Judge complete emitted answers, source/version ownership and actual continued behavior. Do not substitute parse accuracy, a final internal label, state mutation, nonempty output, a loss curve, or CI acknowledgements. A promising near miss may justify a prospective practical criterion change; preserve the original criterion and result instead of retroactively passing it.

## Freedom and stopping decisions

You may change training credit, bounded state organization, generic candidate support or fit dose when evidence identifies a bottleneck. Explain the observation, reduced-form prediction and falsifier before a material change. Use the necessary local time/storage extension authorization rather than abandoning useful work at an arbitrary timer. Stop/checkpoint at actual configured ceilings; repair storage reserve first rather than merely acknowledging a breach after computation.

If unseen-entity transfer fails, localize whether the missing distinction is observation, admission, exact identity, intent or version eligibility. Address that same capability. Do not hard-code the held-out names, silently widen a word exception list, or reopen a solved codebook sweep. If a smaller factorization would solve the deficiency more cleanly, choose it and report why.

Do not make Q8 consumption a mandatory dependency of this run. Once the scoped correction lifecycle works, the next substantive step is one learned/composed geometric result that is consumed by subsequent inference. Broader E/S prose, conversation expression and executed Rust follow through the same path; complete runtime/energy qualification follows a useful integrated artifact. If integration exposes an immediate small reusable Q8 need, you may investigate it, but keep acceptance of memory and computation separately measurable.

## Research to use critically

The principal review checked SLOG (lexical versus structural generalization), LongMemEval (updates/temporal reasoning/abstention), EverMemBench v3 (role attribution/evolving versions), and official repeatable-read documentation (consistent read views). Inspect primary sources when adopting an idea. Their architecture/results do not prove UOR-R4 behavior and do not authorize importing a dense backbone or launching a large unrelated benchmark.

Two seemingly relevant 2026 directionality/structural-generalization preprints, arXiv2607.02307 and2604.26157, are withdrawn for an evaluation-metric mismatch. Do not cite their reported wins as support. Verify current status for new papers.

Harmonic/scalar fields remain an alternative basis for learned role/compatibility functions; signed H4/Hopf fiber may preserve a witnessed orientation alias; E8/S7 may help a demonstrated state-sharing problem. Exact scoped banks already express structural persistence. No dimension ladder, physics analogy or more scalar features can recover information the serving representation has erased.

## Resources, validation and delivery

Read the live model ledger and current storage inventory. Project full context preparation, builds, fitting, diagnostics, controls, final evaluation, checkpoint/report and delivery costs; two compiler workers and one model worker are a reasonable current baseline, but justify changes against the machine. Preserve the recorded physical reserve and128MiB margin. Reuse valid release caches and preserved executables. Do not delete a delivered binary and rely on rebuilding it later. Never delete sealed attempts or unique material to manufacture headroom; use reviewed regenerable cache cleanup.

Use Rust for preparation/training/inference. Focus tests on meaningful arithmetic, representation, lifetime, serialization and causal interfaces. Run actual loaded generation and saved-state continuation. Run format/touched-package checks and the claim-wording script when changing capability prose. The five compatibility check names do not run local tests for you.

Create every report root exclusively before loading models; seal and verify the complete file set, keep derived inputs outside sealed roots, retain unsuccessful attempts and source/executable hashes. Account for all retries and delivery. No paid/external compute or new secrets are authorized.

Commit named paths, push and deliver through the existing/new appropriate protected PR after refreshing branch state. Do not direct-push main or bypass protection. Update current-state, canonical plan, README/ROADMAP/PROJECT_MAP/CONTINUE where their active statements change, the evidence index, original-result qualifications, resource ledger and the owning live issues. Keep broad acceptance issues open unless their full criteria are met. Update knowledge with source-linked records and distinguish model evidence from delivery status. Confirm actual merge/tree content before saying merged; pending protected delivery remains pending.

Your final report should lead with the useful behavior achieved or the precise remaining obstruction, explain the mechanism and any justified deviation, present matched complete-answer evidence and scope limitations, state which geometric contribution is actually measured, and give exact delivery/resource/artifact pointers. Recommend the next architecture decision in the context of the full project.
