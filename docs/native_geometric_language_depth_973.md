# Third dependent language read — #973

**PASS_THIRD_LANGUAGE_READ.** The unchanged learned artifact passes all 1,280 three-read answers and exact occurrence paths, including 64 complete intervention families. [Source-bound evidence](evidence/native_geometric_language_depth_973.json). No fitting, parameter edits or new depth-specific action table were used. This establishes bounded depth transfer under the existing sequential-question protocol. Normal model15baec48 stays retained and unpromoted; independent final holdout is NOT_RUN.

## Mechanism and test environment

The [shared runtime](../crates/uor-r4-core/src/native_geometric/dependent_language/scheduling.rs) now admits up to three question clauses instead of two, preserving the 256-byte prompt, 128-byte clause, 16-word and 16-byte-word bounds. Frame already advances clause/query state generically. The existing executor counts initial selection as read one and permits four reads; this panel uses three. The same reader, updater, scheduling table and byte/EOS writer perform two silent Read transitions before final emission. No broader runtime limits changed.

The separate Rust [depth corpus](../crates/uor-r4-core/src/native_geometric/dependent_language/depth_data.rs) contains 1,280 rows: eight name rotations × four relation orientations × two retained styles × four source rotations × five variants. There are 64 families of 20 rows, and 256 rows per variant. Styles are plain did/who and today…can quietly…/which person. Both names and grammatical components are familiar from previous development. Each reference denotes the latest selected value by the explicit protocol; this does not test arbitrary discourse coreference. Existing one/two-read data was unchanged.

Independent typed validation resolves the raw questions against raw facts and checks unique answers, exact paths, two intermediates, balanced roles/sources, source rotations and single-word interventions. The negative test rejects stale intermediate metadata and unrelated source edits. Only raw records and prompt enter inference. No model-based filtering or answer/path metadata enters runtime. Acceptance was written before artifact load and the report root was exclusively claimed.

## Actual trajectories and controls

Baseline records are ruby did guide felix; felix did trust clara; clara did trust dylan; dylan did trust alice. The questions ask Ruby's guide, then that value's trust relation, then its trust relation again. Actual selected values are Felix → Clara → Dylan; only Dylan plus EOS is emitted.

| Matched intervention | Actual selected values | Final output |
|---|---|---|
| Baseline |Felix → Clara → Dylan|dylan + EOS|
| First relation answer Felix → Clara |Clara → Dylan → Alice|alice + EOS|
| Middle relation answer Clara → Dylan |Felix → Dylan → Alice|alice + EOS|
| Active terminal answer Dylan → Bruno |Felix → Clara → Bruno|bruno + EOS|
| Unused tail answer Alice → Bruno |Felix → Clara → Dylan|dylan + EOS|

Every variant passes 256/256 answers and paths across source rotations, roles and styles. All full traces have two silent Reads, read counts 1→2→3, exact intermediate values, continuous Frames and emission only in the third clause. All 1,280 traces reproduce after artifact reload.

SecondUpdateDisabled keeps the current query instead of committing the second update. StaleSecondPayload substitutes the first selected value at the second update. Both controls preserve the complete first Read and the second incoming Frame/current source, then change the committed query on all 1,280 rows. Both yield 0/1,280 final answers and paths. Stale payload history exists only for the diagnostic control; normal serving retains no added history for it.

PolicyDisabled, ReadDisabled, ContinuationDisabled, AlwaysRead, EmitInsteadOfRead, UpdateDisabled, PayloadReversed, ScorerDisabled, CursorDisabled and StopDisabled also yield zero complete answers. AlwaysRead can visit the correct occurrences without producing the answer, demonstrating why path-only success is insufficient. ExactIdentity and FeedbackDisabled each retain 1,280/1,280 answers/paths; this panel adds no geometric metric or emitted-feedback advantage.

## Preservation, claims and resources

The candidate is byte-for-byte identical to its parent: SHA256 62da0bfaddb37452a5ec7eb1aa220fbda7b33c79ef3dbcd4da1dcb11293e7a95. Reader masks [1282,68098], updater [3], scheduler [2,0,2,0,2,2,2,1] and writer are unchanged. Frozen runtime/data/source and executable receipts bind the new evaluation separately from historical artifact training provenance.

Retained comparisons preserve 13,248 old outputs, 512 recurrent traces, 640 ordered controls, 1,408 language controls, 128 old-language rows through relative selection, 9,984 relative controls, 9,984 dependent controls, and two independent 19,968-row comparisons against preceding scheduling and output-credit reports. Relevant tokens, exhaustion, occurrences, actions, policy rows and final read counts are compared row by row. One new and 37 retained focused tests pass with optimized offline compilation. Queue status names remain compatibility acknowledgements, not tests.

All 48 prior sealed roots / 1221 files, predecessor frozen source/executable, normal model and original dirty checkouts verify unchanged. The new attempt has 13 sealed files. Local lineage and restart: /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-depth-1/, including source-freeze.json, attempt-1, verification.json and resource-final.json. No training or retry: one build and one evaluation. Model/build/data/controls charge 220541/1800000 ms; shared 127597342/132950000 ms; parent 7846769/9810000 ms. Peak sampled process-tree RSS 2425618432 bytes. Local projection: 6 GiB RAM, 256 MiB new storage including 128 MiB stop margin, two build threads and one model process. No allowance extension, paid compute or cleanup.

The runtime remains geometric routing/state/operators and integer/table execution without a transformer, serving matrix product or runtime provider. Signed H4/Hamming structural comparisons keep their existing role. Familiar lexical grammar, supplied question boundaries, one-word substitution and bounded copying remain limitations. No general prose, arbitrary reference resolution, semantic metric, energy or novel-grammar claim follows.

**Next:** Distinguish completed answers from unresolved required continuations. First run a small frozen-artifact diagnostic comparing a direct question, a valid dependent question, a source-only missing compatible continuation and a source-only conflicting continuation. Capture pending-clause state, update admission, next-route status, policy row and actual output/EOS. The source currently collapses final and unusable pending continuations into the same observation; premature completion is a source-level prediction, not a newly measured failure. If reproduced, separate pending-clause presence from continuation availability and preserve a typed unresolved outcome without claiming that rejected routes prove missing facts. Retain all successful one/two/three-read paths; do not add another depth ladder or broaden answer length before this completion distinction is reliable.
