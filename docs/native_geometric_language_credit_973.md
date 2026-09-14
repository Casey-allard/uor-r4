# Shared final-output reader and update credit — #973

**PASS_SHARED_LANGUAGE_OUTPUT_CREDIT.** Four fixed training blocks reach 4,096/4,096 training answers, then 1,536/1,536 development answers and exact occurrence paths, with 96 complete intervention families. [Source-bound evidence](evidence/native_geometric_language_credit_973.json). The final rules equal the preceding artifact and the two-block bootstrap: **no incremental accuracy gain from the mixed-credit blocks was measured**. Normal model `15baec48` stays retained; no promotion. Independent final holdout is NOT_RUN.

## Implementation and credit

The Rust [learner](../crates/uor-r4-core/src/native_geometric/dependent_language/joint_learning.rs) clears reader compatibility and query-update rules, then fits four discrete blocks under the retained learned scheduler and byte/EOS writer. Direct-reader bootstrap is explicit. Later blocks execute the actual shared [language loop](../crates/uor-r4-core/src/native_geometric/dependent_language/scheduling.rs) after each forced occurrence or substitution and score final answer plus EOS. Expected source paths, intermediate answers and example kind do not supply training credit.

Reader probes hold the occurrence across its whole span; both reader and updater probes must actually fire. Query formation, future source availability, scheduling and output are recomputed. Later mixed sites are discovered from actual baseline execution. Early candidates that directly emit the final answer remain positive, even when their authored source path is wrong. Path correctness is evaluated separately. Shared monotone-conjunction rules must select uniquely without accepting incompatible candidates; training does not discard these shortcuts using oracle paths.

Every block saves candidate suffix tokens, exhaustion, committed queries, selected occurrences, proposed rules and complete training results before requiring coverage and rowwise nonregression. Reader search uses 4,047 one-to-four-literal conjunctions over 18 features; updater search 92 one-to-three-literal conjunctions over 8 features; each operator admits at most 8 rules. This is explicit direct bootstrap and discrete block-coordinate final-output credit, not cold end-to-end or joint-gradient training. The serving call uses no probe and retains its normal behavior.

| Block | Correct training answers before → after | Covered sites | Reader rules | Updater rules |
|---|---:|---:|---|---|
| Direct reader bootstrap | 0 → 2,048 | 2,048/2,048 |1282,68098|empty|
| Updater bootstrap |2,048 →4,096|2,048/2,048|1282,68098|3|
| Mixed reader credit |4,096 →4,096|6,144/6,144|1282,68098|3|
| Updater refresh |4,096 →4,096|2,048/2,048|1282,68098|3|

The mixed block contains 5,120 positive early candidate choices requiring downstream processing and 2,560 output-compatible direct shortcuts. These are candidate counts, not independent examples. All 4 blocks preserve every previously correct training answer. The common credit path is exercised, but its final rules and development answers equal the bootstrap; its incremental training advantage is not demonstrated. The scheduling table stays [2,0,2,0,2,2,2,1] and the writer artifact remains identical.

## Actual behavior and limits

The same existing Rust corpus is used unchanged: 4,096 training and 1,536 development rows. Every direct/dependent × lexical/construction-combination/joint cell passes 256/256 answers and paths. All96 sixteen-row intervention families complete. This corpus was already used during previous operator development; it is not an independent holdout or new grammar qualification. All 1,536 Full trajectories reproduce after artifact reload, and state continuity and silent Read increments pass.

Clearing the reader yields 0/1,536 answers. Clearing the updater preserves768/768 direct answers and loses all 768 dependent answers. PolicyDisabled, ReadDisabled, AlwaysRead, CursorDisabled and StopDisabled each yield 0. ContinuationDisabled, EmitInsteadOfRead, UpdateDisabled, PayloadReversed and update-position ScorerDisabled preserve all direct answers and lose all dependent answers. ExactIdentity and FeedbackDisabled each pass 1,536/1,536, so no new geometric metric or emitted-feedback advantage follows.

The earlier scoped adapters retain 13,248 old outputs,512 recurrent traces,640 ordered controls,1,408 language controls,128 old language rows through relative selection,9,984 relative controls,9,984 dependent controls and19,968 previous scheduling rows, compared individually including relevant source paths/actions/state observations. One new and36 retained focused tests pass. Release compilation and frozen source/executable bind the report. CI names are compatibility acknowledgements, not tests.

Serving remains bounded geometric routing and shared transitions with integer/table operations; no transformer, serving matrix product or teacher/provider was added. Signed H4 prefix geometry and Hamming structural comparisons retain their existing word-binding role. Fixed encoding, explicit question boundaries and one-word substitution remain limitations. This result does not establish general prose, arbitrary discourse reference, learned encoding, geometric semantic distance, depth extrapolation or complete-path energy savings.

## Evidence, resources and next action

Candidate SHA256: 62da0bfaddb37452a5ec7eb1aa220fbda7b33c79ef3dbcd4da1dcb11293e7a95. Parent SHA256: 65c28a3ae06148bf7defd0acdc6c1b25928f35febc1f964520309831e209ed22. Local report root: /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-credit-1/attempt-1;25 sealed files. Frozen sources/executable: /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-credit-1/source-freeze.json and source/. All 47 previous sealed roots/1,196 files, predecessor source/executable, normal model and both original dirty checkouts verify unchanged. Full projection, allowance extension, command/resource receipts and precise restart remain in the established project-local handoff. No external compute, paid cost or deletion.

Complete model/build/fit/controls charge 381962/2100000 ms; shared 127376801/132950000 ms; parent 7626228/9810000 ms. Peak sampled process-tree RSS 2976006144 bytes; step storage high water 216502272/402653184 bytes. Full receipts in resource-final.json; model allowance 2,100,000 ms, RAM 6 GiB, storage 384 MiB including 128 MiB margin, two build threads and one model process. Parent model allowance was extended 900,000 ms to 9,810,000 before execution under standing authorization; shared ceiling 132,950,000 ms stayed unchanged. No storage extension.

**Next:** Evaluate a bounded third dependent language read using the unchanged fitted reader, updater and scheduler before considering any refit. The existing Frame and shared transition already advance clause/query state and permit four reads; only the two-clause admission limit needs a scoped extension. Freeze a separate Rust depth-transfer corpus with first/middle/terminal/inactive source interventions and a second-update-specific control. Retain all one/two-read panels and current byte/word limits. No depth-specific learned table, new parser, old-fit replay or broad audit.
