# Mixed Add/Copy selection and exact operand references — #1140

## Decision and bounded behavior

**Retain 9ab64902 at bounded plain mixed Add/Copy scope.** CID blake3:9ab64902d4811f4e23e2119bdee74f16a6675eee7446e85e21666dfccfabea59; SHA256 14c76d1375eadec57cb34e9fff98c43b52f16926495c8493bc530c2e077e0de5; 12,562,902 bytes at .uor-models/native-typed-value-2026-09-05/mixed-operators/model.json. Preserve parent 866cb92d and all rejected candidates. The [evidence receipt](evidence/native_geometric_mixed_operators_1140.json) binds the actual sources, artifacts, reports, controls, selection and resources.

This experiment addresses plain numeric Add followed by Copy of either the latest result or the original total, while preserving the existing Add/Add path and learned stopping. Actual history first computes `13 + 4` and emits `17.\n`. The next query introduces three extra coins and requests the first Add. The four authored cases have these independent targets:

| Continuation request | Complete response target | Second operation and exact operand IDs |
|---|---|---|
| None | `20.\n` | Stop; no second write |
| `Copy the latest result.` | `20.\n20.\n` | Copy `[4,4]` |
| `Copy the original total.` | `20.\n17.\n` | Copy `[2,2]` |
| `Again.` | `20.\n23.\n` | Add `[3,4]` |

Record 2 is the original generated total, record 3 is the new literal, record 4 is the first current-response Add with ordered operands `[3,2]`, and record 5 is the requested continuation. These IDs are offline labels for this authored history; they are not constants in serving selection. The evaluator checks both pending decisions and committed derivations against the labels.

Construction contains three numeric groups under these four forms, twelve cases. Open evaluation uses two other numeric/name groups, eight cases. Stress uses reversed initial fact order and includes zero extra, eight cases. A separate fresh mode generates positive/signed cases with different names only when explicitly invoked; it passes 8/8 after recorded selection. No claim of held-out wording transfer follows from these finite forms.

The equal-value stress cases are important. The retained candidate emits the same `67.\n67.\n` text for Copy-latest, Copy-original and Add-again when the extra is zero, while selecting `[4,4]`, `[2,2]` and `[3,4]` respectively. Those recorded pending and committed identities distinguish the requested operations even when text alone cannot. The final candidate preserves that distinction in both pending and committed records.

## Observed parent bottleneck

The retained composed-output parent already supports repeated Add and formatted Add/Add. Its runtime proposals also contain Copy of any retained record. The first mixed diagnostic therefore tests actual selection before changing proposal support or dispatch.

Appending a Copy request changes the parent's first operation: it copies the original literal `13` instead of computing the requested first result `20`. The affected component is the shared derived-role router. Retained history makes this a derived-role context, so refining literal routing or numeric-versus-lexical admission would target the wrong component.

A separate representation gap affects the next-operation router. Its existing dictionary recognizes `add`, `again` and `original`, but does not recognize `copy` or `latest`. Same-position substitutions among unknown instruction words are erased from its lexical features. The intervention extends this learned dictionary and preserves the meaning of existing prime-coded features. It does not infer semantic distance from prime values.

## Native mechanism and artifact identity

[mixed_initial_training.rs](../crates/uor-r4-core/src/native_geometric/mixed_initial_training.rs) refines the existing derived-role router using exact first-operation and operand-ID labels. It preserves the frozen parent's ordered preferences, including NoOperation, on earlier contexts. Literal routing, admission, lexical emission, completion, tokenizer and geometry remain unchanged.

Preservation must cover the entire generated history. The role router is used at each intermediate response, not only the final query. Training therefore reconstructs every supplied history prefix with the frozen parent and collects its actual role preference before the corresponding next response. The current construction includes 436 such history boundaries, the earlier contextual panels and 745 preserved prompts. The complete prompt population is bound through one canonical `Vec<Document>` manifest receipt, retaining every ID and text while respecting the source-router receipt limit. The containing context receipts also bind the history turns from which the additional frames are derived.

[mixed_operators.rs](../crates/uor-r4-core/src/native_geometric/mixed_operators.rs) then continues the existing shared Stop/Copy/Add router. Its new targets name exact committed operand IDs. A Copy target names the same occurrence twice; an Add target names the supported ordered pair. Offline preparation rejects unavailable identities, unsupported actions, invalid operand order and overflowing arithmetic candidates. Serving keeps its existing bounded proposal enumeration, Copy-alias Add exclusion, arithmetic checks and three-operation cap.

The expanded dictionary is sorted and assigned its canonical primes. Because insertion can change an existing word's prime, the trainer remaps old kind-4 and kind-5 lexical code fields through exact old-word/new-word identity. It preserves their learned roots and all nonlexical fields during this migration. New features use the existing geometric learner. The role router grows from 667 to 679 codes and fits all 41 distinct frames; the operation router grows from 270 to 286 codes and its dictionary from 21 to 24 words. Both stages use angular mode, seed 1140, a 768-feature maximum, eight passes, 24 proposals and a 120-second fit ceiling. The experiment does not add a command parser, target-ID field in serving state, response template, provider or transformer backbone. Serving continues through bounded signed-H4 composition/ranking and exact typed integer/table execution without mathematical matrix products.

Initial-role and transition supervision are explicit offline stages. History and the complete first response are freely generated and checked. For new mixed examples, the trainer also requires exactly one committed first write with the labelled action and ordered operand IDs; matching a numeric string alone is insufficient. Where a next operation is supervised, preparation executes that operation on actual records and lets the unchanged completion path generate its bytes; expected response strings are never observed as generated tokens. Complete model-selected evaluation is separate from these training frames.

The checkpoint callback receives a refreshed, validated artifact after the initial-role stage, before the transition fit. It saves that artifact and its fit report so a later failure does not discard the initial-stage evidence. The final outer witness stores the complete previous operation block and the previous role router. Restoring both must reconstruct the full retained parent before current CID/address validation succeeds. Dictionaries, other routers, operator limits and earlier witnesses are therefore preserved at their declared boundaries. Older artifacts omit the optional field; a new artifact requires the updated runtime.

`MixedInitialDisabled` restores the previous role router, `MixedTransitionDisabled` restores the previous operation block, and `MixedOperatorsDisabled` restores both. The older `ComposedOutputDisabled` control also uses the old dictionary with its older prime-coded router. Restoring only an old router under a newly assigned dictionary would change the control's meaning.

## Rejection margin and candidate exclusion

An ordinary-response gain is insufficient when an unavailable requested operation causes an unrelated substitution. The `7e0f5780` candidate passes the ordinary retained panels but changes an earlier latest-intermediate-exclusion response from `43.\n` followed by Stop to `43.\n40.\n`, where the second operation copies an unrelated earlier total. Both fail the unmodified two-Add target, but the new fallback weakens abstention when the required Add proposal is unavailable.

The current training correction adds a generic companion to each positive transition frame. It removes all labeled operation alternatives, preserves the original state and feature values, retains every incorrect alternative, and labels Stop. Together with the original positive frame, this teaches the ordering **correct available operation > Stop > incorrect substitutes**. No control-specific feature or serving exception is added. Stop frames and original positive targets remain part of construction.

This is declared rejection-margin supervision. Reused candidate-removal cases are now intervention-trained construction, not independent evaluation evidence. The latest record remains in state and features under the existing proposal-exclusion control; this is not memory erasure. The final fit adds 50 rejection companions to the 455 ordinary transition frames. After deduplication, all 70 distinct frames fit with zero hinge, without reaching the time ceiling. Ordinary preservation and separate transfer results are reported below.

## Preserved failures and comparison limits

The following material remains part of the evidence history:

- The first fit stops before producing a final artifact after its role update changes a historical intermediate response from `22` to `4`. Its complete **72.005 model seconds** remain charged. Adding intermediate history boundaries addresses that missing preservation scope.
- Candidate `c24ab204` passes twelve new construction, eight open, eight stress and 254 transition-replay cases, but changes retained literal case `literal-fit/0/true/0` into a repeated `13` response. The main panel is 662/663. It is rejected; the complete already-open 103-case literal panel is added as Stop construction.
- Candidate `7e0f5780` restores ordinary 663-case and earlier-panel behavior, but exhibits the unrelated-Copy fallback under intermediate candidate exclusion described above. It is superseded by the qualified rejection-margin candidate and remains preserved.

Exact control-output comparison must remain separate from aggregate correctness. Eight already-wrong current-query removal rows change their actual wrong outputs without losing previously correct responses. That is neither exact preservation of those rows nor evidence of a new correct capability. Do not claim that all fourteen historical result/control reports remain byte- and identity-identical. Final comparisons must retain the individual changed rows, the controls' intended targets and the distinction between correct-response regression and changed behavior on an already-failed intervention.

Raw candidates, stopped-fit logs, source snapshots, intermediate artifacts and comparisons remain under `/Users/casey.allard/Documents/Codex/2026-09-08/uor-r4-mixed-operators`. The superseded partial-comparison checker retained misleading true equality flags while collecting differences; its explicit superseded receipt excludes those flags from the result. The final comparison checks every previously correct row and records each changed negative.

The first rejection-margin candidate, a107f3e5, passed the ordinary panels and restored the intermediate-exclusion control, but the API and direct runtime both returned 36 instead of 17 on a new independent sum after formatted 20/23 composition. It is rejected. The complete 42-case prior composed panel was added as freely generated history followed by the original independent sum. Candidate 66f3d262 then passed all 65 API checks but regressed three of the six historical numeric Copy cases to Unknown. It is also rejected. Adding the complete six-case qualified numeric panel with actual history produces the retained candidate. Final construction uses 12 new examples, 447 preservation contexts, 745 prompts and 436 derived history boundaries. Expected history text is checked after generation, never injected.

The strengthened mixed evaluator also checks a following independent sum after every correct Full response, requiring fresh literal occurrence IDs in both pending and committed derivations. The final fit includes 505 transition frames before deduplication, including 50 generic rejection companions. All 41 role and 70 transition frames fit with zero hinge and neither stage hits its time limit. A report-parent directory error consumed 21.251 seconds without running preservation cases; its setup-failure receipt and charge remain separate from model evidence.

## Final verification and resources

| Final check | Result |
|---|---|
| Construction, open and stress complete outputs with exact IDs | 12/12, 8/8 and 8/8, including exact fresh-record independent turns |
| Rejection-margin fit and candidate-exclusion behavior | 70/70 distinct frames; open 8/8 targeted exclusion outcomes, preserving available Copy-original and stopping when latest is required |
| Main 663 responses and earlier preservation panels | 663/663 and every earlier preservation panel; 254/254 transition replay |
| Prior exact lexical/composed outputs and scoped controls | 72/72 saved lexical outputs; composed open/stress/former-fresh 28/28 each; all earlier correct rows retained, 13/14 reports exactly equal, 8 already-wrong current-query control rows explicitly changed |
| Actual native API, subsequent independent turns and artifact tampering | 65/65 |
| Mechanical migration/identity/overflow tests and source guard | Five mixed, six lexical-read and six operation tests; integer-kernel source guard and formatting pass |
| Actual artifact checkpoint and allocation checks | Mixed, previous composed and previous lexical checks pass; zero allocations in measured ingestion/begin/predict/observe paths |
| Separate post-selection fresh evaluation | 8/8 numeric/name transfers; 8/8 declared candidate-exclusion outcomes; no subsequent fit |
| Generated Rust expression preservation | 54 unchanged generated equalities compile and execute; these are expressions, not synthesized programs |

The actual mixed allocation check covers 28 checkpoint positions over two cases and real histories; the prior composed and lexical checks cover 72 and 136 positions. Warm predict/observe timing excludes load, encoding, session creation, BOS/end-response, checkpoint/JSON/report work, so it establishes no complete-path or energy advantage. The caller-facing API still allocates outside these measured paths.

The evaluator freely generates history, changes only the declared control for an intervention, and then ingests the identical query. It checks complete text, EOS, ordered pending writes, committed values/derivations and operation counts. Per-token verification compares repeated predictions, restored predictions and committed completion/value state. Captured numeric and lexical query/source data remain unchanged during the response. One- and two-operation Stop targets terminate below the three-operation safeguard.

This cycle used **2739.786 model seconds** and **1037.308 engineering-command seconds**, including all failed fits, rejected candidates and repeated preservation checks. Cumulative model use is **12707.541 / 13350.000 seconds**. The pre-use increments were 900 seconds, then 600 seconds and another 600 seconds under standing local authorization; the cycle allowance was extended from 1,800 to 3,000 model seconds before use. Retain the 196.620-second reservation, leaving 445.839 seconds after it. Conservative storage is **32,580,792,320 / 34,863,214,592 bytes**, including the retained artifact copy and 8 MiB final metadata reserve, with the 134,217,728-byte stop margin preserved. The projection keeps one model process, two threads, 512 context tokens, 96 output tokens, 4 GiB model RAM, 6 GiB build RAM and 1 GiB new storage. Sampled peak model/build process-tree RSS is 1,899,954,176 / 2,447,785,984 bytes. Small documentation/receipt/Git commands are outside the engineering-command stopwatch. No unique material was deleted and no paid external compute was used. During this cycle the owner separately requested disk cleanup: inactive debug incremental/object/archive/metadata caches were removed after active-use and type checks, while source, executables, dynamic libraries, release outputs, models and Downloads were preserved. About 20.18 GB was recovered, leaving 45.32 GB free at the cleanup receipt. Inherited storage charges and observed positive target growth were retained without refunds. Queue compatibility acknowledgements execute no tests.

## Limits and next source-directed step

The intended result is plain Add→Copy of the original/latest result and retained Add→Add, with exact occurrence references and stopping after one or two operations. It does not establish Copy→Add, three-operation learned stopping, arbitrary instruction placement, general prose, generalized reasoning, Rust program synthesis, alpha or complete-path energy advantage. Formatted Copy and new-artifact browser/WASM/HTTP remain NOT_RUN for this artifact.

Next, run an actual mixed-operation formatting diagnostic before choosing another implementation. [lexical_emission.rs](../crates/uor-r4-core/src/native_geometric/lexical_emission.rs) currently omits the anchor's action from its emission features. After a numeral, Add and Copy therefore begin with the same token-context features for the same query. A Copy also repeats one operand ID, which the current role function maps only to its first-operand role; a distinct second-operand role is unavailable. The existing Add-shaped suffix is not evidence of correct unary Copy emission.

If actual generation confirms that bottleneck, the smallest supported successor is shared action-conditioned emission from the committed typed anchor, preserving the current exact-read semantics and all accepted Add formatting. It should choose lexical output geometrically from the operation and state, with matched action/read controls, rather than introducing another response-family template. Broader language learning remains with #973; #1140 remains open.
