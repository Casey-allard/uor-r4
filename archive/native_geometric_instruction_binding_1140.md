# Contextual instruction binding — #1140

## Retained behavior and scope

Retain `blake3:ccbe7f2a5038b0707a26b8a85d8b67ee274fd82b2a7f1b8afd7121ad2ada46ad` at bounded contextual instruction scope. The [evidence receipt](evidence/native_geometric_instruction_binding_1140.json) binds source, actual construction, preserved and rejected artifacts, complete responses, controls and cumulative resources. Parent `0f0f5fe4` and all earlier material remain preserved.

Given `suri has 13 coins. orin has 4 coins.` and the tested sum question, ordinary requests now retain the correct Add and produce complete output:

| Request placement | Actual response |
|---|---|
| `In a sentence, ` before the question | `17 is 4 plus 13.\n` |
| ` Explain in a sentence.` after the question | `17 is 4 plus 13.\n` |
| `In Rust, ` before the question | `17 == 4 + 13\n` |
| ` Write a Rust equality.` after the question | `17 == 4 + 13\n` |
| No formatting request | `17.\n` |

There are twelve authored request/placement forms: one plain, six sentence and five Rust forms. Construction contains three numeric groups in both fact orders, 72 examples. Open evaluation uses two new numeric groups, 24 examples; another 24 cases change names or reverse fact order. All pass on the retained candidate. Restoring both parent routers leaves only 8/24 open responses correct, losing the sixteen repaired responses. This is a joint intervention on the two refined routers, not an isolated measurement of either router's contribution.

These results establish transfer over the specified wording, numbers, names and order. They do not establish arbitrary instruction following, general prose, broad reasoning, Rust program synthesis, alpha or energy advantage. Requested operand reordering is not a learned capability here: the offline targets require exact source IDs `[1, 0]`, following the existing derivation/emission convention.

## Observed cause and implementation

Parent diagnostics showed that all required values were retained, but longer instruction phrases disturbed upstream selection or numeric admission. Prefix cases sometimes copied 4; suffix cases sometimes returned Unknown. The bounded newest-first query representation includes lexical position and provenance features, so added words change context even when the arithmetic facts are unchanged.

[instruction_binding_training.rs](../crates/uor-r4-core/src/native_geometric/instruction_binding_training.rs) continues the existing literal-selection and joint-admission routers offline. No serving feature law, dictionary, tokenizer, lexical emitter, operator-transition parameter or geometric identity changes. Literal selection has 640 feature codes, adding 21; admission has 476, adding 348. New codes use the existing prime/provenance feature families and learned signed-H4 roots. The actual artifact binds configuration, receipts and learned parameters. Training uses 72 declared ordered Add targets and 745 already-open parent contexts, with the original parent's ordered preferences/admission as preservation targets. The shared learner retains its 768-feature ceiling, eight passes, 24 proposals, seed 1139 and 120-second per-router ceiling.

Serving still performs the existing bounded signed-H4 composition/ranking and typed integer/table execution. It adds no keyword-to-answer branch, sentence template, arithmetic parser, provider or matrix product. The existing emitter selects bytes and exact committed operand reads. Prime identities are not semantic distances. Fixed zeta identities, exact geometry and paired-H4 material remain preserved; this experiment does not isolate their individual predictive contribution.

[instruction_binding.rs](../crates/uor-r4-core/src/native_geometric/instruction_binding.rs) stores the previous two routers as an outer witness. Restoring them reconstructs and validates the complete accepted parent before checking current identity. The matched disabled control uses those exact previous routers. Older artifacts omit the new optional field and remain loadable; the new artifact requires the updated runtime. Actual API checks distinguish frozen-parent rejection from current-artifact identity rejection and reject unknown witness fields.

## Rejected candidates and preservation

The first candidate `db6c7781` passed 36 construction cases but used weaker commutative Add preservation targets; its `features_added` report field also incorrectly counted feature-expansion frames. Preserve it as superseded and unqualified. The corrected trainer preserves ordered IDs and reports both quantities separately.

Candidate `f17dd654` passed 36 construction and 24 open cases and retained all 72 prior lexical outputs, but failed five of 24 changed-context cases when fact order reversed. Adding both orders produced `f76521ab`: 72/72 construction, 24/24 open and 24/24 stress, but eleven earlier preservation cases failed. Six were Rust/city outputs, four were initial chain-generation responses, and one Where query changed an expected abstention into arithmetic. All 663 main retained answers still passed; that aggregate did not justify retention.

The final fit adds 82 already-open initial/literal contexts from the affected parent panels, for 745 preservation documents. It repairs those eleven failures while preserving the new gains. No serving exception was added. All failed artifacts, fit-source freezes, complete responses and charges remain in the local evidence directory.

The current evaluator independently requires exact ordered source IDs in new responses. Prior lexical preservation additionally requires the baseline artifact to equal the frozen parent and compares exact saved text, EOS, read IDs, action, value and ordered operand IDs/values. A commutatively equal sum cannot hide changed visible operand order.

## Executed validation

| Check | Executed result |
|---|---|
| Construction / open / changed-name and mention-order cases | 72/72 / 24/24 / 24/24 |
| Both refined routers restored to parent | 8/24 open; sixteen repaired responses lost |
| Main retained answers and all earlier panels | 663/663; all recorded preservation panels pass |
| Exact old lexical output preservation | 72/72: prior construction 18, open 18, name/order 12 and former fresh 24 |
| Earlier operator transitions | 24/24; both dependency controls retain 16 single-operation answers and lose eight second operations |
| Earlier current-query / harder histories | 48/48 / 16/16; removing the original result retains 32 independent answers and changes all 16 dependent answers |
| Earlier lexical emitter/read/learned-transform controls | Each retains six plain answers and loses twelve formatted answers |
| Actual native Rust API | 38/38 checks, including ordinary prefix/suffix forms, checkpoint import, subsequent independent sum and artifact tampering |
| Focused runtime checks | Six read-state and six operator tests, native source guard and formatting pass |
| Separate post-selection numeric panel | 24/24 with ordered operands, complete output and EOS |
| Actual generated Rust equalities | 30 compile and evaluate: ten open, ten changed-context, ten fresh |

The actual generated Rust equalities are embedded unchanged in an `assert!` harness, compiled and executed locally. The surrounding program is a test harness, not model-generated program synthesis. The separate fresh panel uses two positive/negative numeric groups under the same finite request forms, after recorded selection; it does not test unseen instruction wording. No refit follows that panel.

The actual retained artifact passes eight positive/negative development cases with zero allocations in ingestion, begin-response, predict and observe; 136 checkpoint positions and exactly one typed write per response are checked. Cold loading took 18.785 seconds. Scoped warm predict/observe median was 0.114 ms and maximum 0.200 ms; encoding, loading, session creation, checkpointing and reporting are excluded. Bounded scan/counter limitations remain, and no complete-path latency, comparative performance or energy advantage is established. Queue compatibility acknowledgements execute no tests.

## Artifact, resources and next step

Local retained artifact: `.uor-models/native-typed-value-2026-09-05/instruction-binding/model.json`, 12,305,599 bytes, SHA256 `1ee9a8f6e537c2c5da4f49e76bbe136ff827b4818a1f615236ba0004caffdeb5`. It adds 303,474 bytes over parent `0f0f5fe4`. Raw evidence, source freezes and candidates remain at `/Users/casey.allard/Documents/Codex/2026-09-08/uor-r4-instruction-binding`; these local artifacts are not distributed by a fresh clone.

The pre-use projection reserved 1,200 model seconds and 1,800 engineering command seconds, one model process/two threads, 512 context tokens, 96 output tokens, 4 GiB model RSS, 6 GiB build RSS and 1 GiB new storage. Before later checks, unused phase allowances were reassigned to preservation; the total cycle and cumulative ceilings did not increase. All four fits, failed preservation, replays, API, runtime checks and fresh evaluation are charged. This cycle used **886.106 model seconds** and **408.225 engineering command seconds**. Small document/receipt/Git commands are outside the engineering-command stopwatch; no model execution is omitted.

Cumulative model use is **9,047.086 / 10,350.000 seconds**, with the prior 196.620-second reservation retained and 1,106.294 seconds available after it. Conservative storage is **31,593,533,440 / 34,863,214,592 bytes**, including the retained copy and a 4 MiB final metadata reserve. The 134,217,728-byte stop margin remains. Sampled peak process-tree RSS was 1,607,680,000 bytes for model commands and 2,598,174,720 for engineering commands. No unique material was deleted and no paid/external compute was used.

Next: combine the retained within-response operator transitions with formatted emission. Check exact intermediate and operand identities, learned continuation/stopping, current-query boundaries and previous abstentions. The two-operation and formatted-response panels currently establish separate behaviors on one artifact; their combined output is NOT_RUN. New-artifact browser/WASM/HTTP is NOT_RUN. #1140 remains open; #973 owns broad language learning.
