# Shared learned language scheduling — #973

**PASS_SHARED_LANGUAGE_SCHEDULING.** One fit reaches 4,096/4,096 training answers and 1,536/1,536 development answers with exact occurrence paths. The same artifact and loop answer both direct and dependent language questions; all 96 intervention families complete. [Source-bound evidence](evidence/native_geometric_language_scheduling_973.json). Normal model `15baec48` remains retained, with no promotion. Final independent held-out evaluation is NOT_RUN.

## Implementation and learning

The [scheduler](../crates/uor-r4-core/src/native_geometric/dependent_language/scheduling.rs) extends the existing dependent_language module. It replaces the fixed second-read dispatch with one learned eight-row Read/Emit/Stop action table, and executes all three actions through the existing recurrent `execute_with_update` transition. The reader, learned one-word substitution, geometric word encoding, writer and cursor operator are frozen. No direct-versus-dependent mode or gold read count selects the serving path.

The prompt still supplies one or two question-mark-delimited clauses. The loop observes current-byte presence, whether the cursor is at the word boundary, and whether the actual selected content yields a usable next query and source. It preserves the distinction between a proposed substitution, admitted update and available route. No rejected route proves absent information; unsupported input and exhausted limits are failures.

The [learner](../crates/uor-r4-core/src/native_geometric/dependent_language/schedule_learning.rs) tries actual Emit, Stop and Read transitions, matching each emitted byte and final EOS to the answer. A successful suffix supplies action credit for the shared feature row. Conflicting action requirements are reported before fitting rather than majority-voted. This is output-constrained trajectory-derived action supervision, not joint gradients through the frozen reader/updater/writer. No intermediate answer, expected source, example kind or gold depth enters the search or runtime.

All 4,096 training examples have a successful trajectory; 27,328 search nodes total and at most 10 per example, below the 512-node limit. No action-row conflicts occur. Starting from all Stop entries, the learner changes rows 1 and 3 to Emit and row 7 to Read; row 0 stays Stop with explicit successful EOS evidence. Final table is `[2,0,2,0,2,2,2,1]`, where Emit=0, Read=1 and Stop=2. Every development feature row (0,1,3,7) was observed in training: this is integration and transfer of language trajectories, not unseen policy-feature or depth extrapolation.

## Data and actual behavior

The Rust [data adapter](../crates/uor-r4-core/src/native_geometric/dependent_language/schedule_data.rs) losslessly combines the preceding direct and dependent corpora. It retains all 2,048+2,048 training and 768+768 development examples, raw records, prompt bytes, answers and intervention families. It creates no new grammar and performs no model-based filtering. Each of the six kind/split cells has 256 examples. The lexical and joint names/verbs/answer strings remain absent from training; the syntactic split reuses vocabulary and recombines familiar active grammatical components. This development corpus was already used in prior operator development and is not an independent final holdout.

Actual joint-development trajectories:

| Raw prompt | Actual actions after initial selection | Final output | Total reads |
|---|---|---|---:|
| please tell me who did quietly guide felix? | Emit ×4, Stop | ruby + EOS | 1 |
| please tell me who did ruby quietly guide? please tell me who did they quietly trust? | Read, Emit ×5, Stop | clara + EOS | 2 |

The underlying contexts and complete states are preserved in sample-direct.json and sample-dependent.json. In the dependent intervention, changing only Ruby's guide from Felix to Dylan still switches the next trust source and final output from Clara to Alice. All existing active-final and inactive-final interventions pass through the same new loop. Frame continuity, silent Read behavior and incremented clause/read indices are checked; initial selection counts as read one. All 1,536 Full traces reproduce exactly after artifact reload.

| Control | Direct exact / 768 | Dependent exact / 768 |
|---|---:|---:|
| Initialization | 0 | 0 |
| Full | 768 | 768 |
| PolicyDisabled | 0 | 0 |
| ReadDisabled | 0 | 0 |
| ContinuationDisabled | 768 | 0 |
| AlwaysRead | 0 | 0 |
| EmitInsteadOfRead | 768 | 0 |
| UpdateDisabled | 768 | 0 |
| PayloadReversed | 768 | 0 |
| ScorerDisabled | 768 | 0 |
| CursorDisabled | 0 | 0 |
| StopDisabled | 0 | 0 |
| ExactIdentity | 768 | 768 |
| FeedbackDisabled | 768 | 768 |

Suppressing the learned extra-read decision, continuation feature, query update, retrieved payload or update-position scorer retains direct answers and removes every dependent success. Policy/read/cursor/stop suppression and unconditional Read remove every complete answer. AlwaysRead can visit the right source occurrences while still failing to emit the answer, so correct paths alone do not qualify behavior. StopDisabled retains the older executor's stop-suppression semantics; its trace records the requested action, not a normal scheduled Read.

ExactIdentity and FeedbackDisabled remain fully correct. The result establishes no geometric metric advantage or necessity of emitted-byte feedback. Signed H4 prefix geometry and structural Hamming comparisons retain their existing word-binding role; canonical occurrence identity remains separate. No new zeta/paired-H4 contribution or general semantic geometry is measured. Punctuation, a usable-continuation feature and the frozen relative-match/substitution predicates remain material scaffolding. There is no learned clause parser, general planning, general prose or new grammar-family claim.

## Preservation, resources and remaining work

Prior adapters retain 13,248 older outputs, 512 recurrent traces, 640 ordered controls, 1,408 language controls, 128 old language rows through relative binding, 9,984 relative controls and 9,984 dependent controls. Those replay the earlier scoped paths and controls; the new shared loop separately qualifies all 1,536 direct/dependent Full rows. Two new focused tests plus 34 retained focused tests pass, along with optimized compilation, formatting, claim wording and diff checks. One build, one fit and one sealed report; no retries or older fits.

Source review caught two integration issues before compilation: shared Query::push visibility and pending-prefix exhaustion during repeated emission. The shared helper now has crate visibility and the loop checks the existing 128-byte pending-prefix limit before Emit. Frozen prior source/executable and all 46 earlier sealed roots / 1,183 files verify unchanged, as do both original dirty checkouts and the normal model.

Candidate SHA256 `65c28a3ae06148bf7defd0acdc6c1b25928f35febc1f964520309831e209ed22`; parent `b44de7fa42ddb1e2728d4e2d950bc91d1ad2320f85fcb67a615b057f3c3612e2`. Full source/artifact identities and complete report file hashes are in the evidence JSON. All local originals remain under `.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-scheduling-1/`, including source-freeze, report, review, verification, projection, resources, protected delivery and restart notes.

Build/preparation/search/fit/controls charge: 183,792 / 1,800,000 ms. Shared cumulative 126,994,839 / 132,950,000 ms; parent 7,244,266 / 8,910,000 ms. Existing allowances suffice, with no extension, paid compute or cleanup. The projection includes two hours wall, 6 GiB RAM, 384 MiB new storage and a 128 MiB stop margin, two build threads and one model process. These are complete local experiment costs, not an inference latency or energy benchmark. Bounded vectors/traces allocate, and word geometry is recomputed. Inference uses no transformer, mathematical matrix product or external provider. CI's compatibility acknowledgements are not tests.

**Next:** Give the reader compatibility and query-update operators shared final-output credit inside this same language loop, using the learned scheduler as a warm start. Begin with a bounded mixed direct/dependent language task and actual suffix re-execution after each changed early selection; expose training conflicts before fitting and preserve all current source interventions and prior outputs. Keep supplied punctuation and fixed encoding explicit. This addresses frozen primitive learning rather than adding another scheduling-only mechanism or a broader metric panel. Do not restart the broad audit, old fits, parked repair or V3–V7.
