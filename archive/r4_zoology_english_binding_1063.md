# English supplied-context binding with the qualified attention cell — #1063

## Frozen contract (2026-09-02; before fitting)

Issue [#1063](https://github.com/UOR-Foundation/uor-r4/issues/1063) is a child of
#973 and follows the [exact-data R4 preservation result](r4_zoology_exact_coherent_inference_1061.md).
The preceding checkpoints remain immutable. Their vocabulary IDs represent
MQAR symbols; this curriculum initializes a fresh seed-123 copy of the same
Zoology source cell with an explicit English lexical map. It does not remap or
continue either earlier checkpoint.

The architecture remains two layers, one attention head, width 64, vocabulary
4,096 and positional capacity 120, with the source dropout, normalization,
residuals, double initialization and tied full-vocabulary head. Exactly 52 IDs
represent the declared words, punctuation, BOS and `unknown`; all 4,096 IDs
remain eligible outputs. An unused ID counts as a wrong answer. There is no
teacher, external provider, pretrained tokenizer, role-tagged model input,
candidate filtering or answer parser in model execution. The serialization
parser is solely a preparation instrument.

Each 41-token input has four facts such as `mara put the key in the drawer.`
and a question such as `where is mara's key? answer:`. The model predicts one
location word or `unknown` at position 40. The label is never appended to its
input. This is a controlled English-form binding task, not unrestricted English
understanding or open-ended generation.

Each world contains `(A,x), (A,y), (B,x), (C,z)` with three distinct owners,
three distinct objects and four distinct locations. Same-owner and same-object
distractors force the central question to use both lexical attributes. A
four-row group asks two questions of a history and of the same history with
their locations exchanged. Half of groups contrast questions sharing an owner;
half contrast questions sharing an object. Question words, fact positions and
location targets are explicitly balanced.

The fifth row exchanges only the object words in logical facts zero and three:
`(A,z), (A,y), (B,x), (C,x)`. The original `(A,x)` question now has no answer
fact, while its owner and object still appear elsewhere. The token multiset,
length, fact count, sentence order and locations remain identical. Its target
is `unknown`. Removing a whole sentence would supply an unintended shortcut.

Construction uses 2,048 groups: 8,192 supported rows and 2,048 unknown rows.
Development uses 256 groups: 1,024 supported rows and 256 unknown rows. Seeds
are 10631 and 10632. Owner-object pairs satisfying
`(owner_index + object_index) % 4 == 0` are excluded from every construction
fact and question, including absent-binding variants, and reserved for
development. All 192 construction and 64 development pairs occur, with zero
pair or canonical-world overlap. All lexical items occur in construction.
Each location has exactly 1,024 construction targets and 128 development
targets. Within each question type, both relevant-fact positions are balanced.

## Fixed learning clock and resource decision

One fresh ordinary-model fit uses batch 512, AdamW learning rate
`0.00046415888336127773`, weight decay `0.1` and the source defaults. It runs
2,352 supported-only updates, then 1,568 mixed updates, with one cosine schedule
stepped every 196 updates and `T_max=64` such blocks. The complete fixed dose is
3,920 updates and 2,007,040 answer presentations: 1,204,224 in the first phase
and 802,816 in the second. Actual supported/unknown counts in the partial final
mixed traversal are recorded. Neither development scores nor checkpoint
selection alter this dose.

The first eight actual updates supply one time/memory admission observation;
their optimizer, sampler and RNG states are retained and their updates count
toward the dose. Remaining time is projected from their mean step duration
with a 1.25 safety factor plus 60 seconds for final evaluation and replay.
The whole campaign has a 1,800-second and 4-GiB ceiling, with one CPU process,
eight intra-op threads and one inter-op thread. Checkpoints preserve cumulative
time and state every 16 updates and at phase/block boundaries. An ambiguous
interrupted admission update stops unavailable rather than repeating timing.

The prior cost basis is #1057: 3,920 updates at sequence 120, vocabulary 4,096
and eight queries took 1,096.9868 seconds. The new sequence is 41 with one
query; the prior conservative fit/evaluation/replay estimate is approximately
1,413.5 seconds. This estimate is superseded by the actual eight-update
admission observation. No hardware matrix, learning-rate sweep, new seed,
post-reveal retry or extra dose is authorized.

Reachability is explicit: every supported input contains its unique answer
fact, all queries follow their evidence, and the two histories require
different answers. Every absent-binding input lacks exactly the queried pair.
The preparation instrument independently parses the decoded input IDs to
verify all labels, split exclusions, pairings, vocabulary coverage and balance
before any model update.

## Empirical criteria and divergent actions

The fixed final artifact is the sole scored checkpoint. Report construction,
supported development, complete counterfactual groups and unknown behavior
separately:

| Criterion | Required count |
| --- | ---: |
| Construction supported accuracy at least 99% | 8,111 / 8,192 |
| Development supported accuracy at least 95% | 973 / 1,024 |
| All four answers correct in at least 90% of groups | 231 / 256 |
| Complete same-owner groups at least 90% | 116 / 128 |
| Complete same-object groups at least 90% | 116 / 128 |
| Development unknown accuracy at least 95% | 244 / 256 |

Only if all ordinary-language criteria pass does this campaign install the
unchanged #1059/#1061 inference-only R4 adapter on the final artifact. Require
identical top-1 IDs for all 1,280 development answers, maximum selected
full-vocabulary logit difference at most `0.005`, attention difference at most
`1e-5`, NLL difference at most `1e-5`, unchanged learned/tied weights and zero
future contributions. The native 8,192-token map and 120-frame bundle remain
unchanged; no new geometry is introduced.

After preserved inference, the inconsistent source-frame transport control
must use equal causal support and work. A supported-accuracy loss of at least
50 percentage points is the separate strong-sensitivity criterion. Its result
does not erase any established plain/R4 preservation. Fresh-process replay
must reproduce final-artifact behavior, logits and audits exactly without
retraining or reading optimizer/checkpoint RNG state.

A positive recommends a separately frozen broader language/context
application. A construction miss retains the artifact and redirects to
learning this lexical recipe. A construction fit with development miss
redirects to compositional transfer. A missing-binding miss isolates that
behavior. A language pass with R4 disagreement isolates integration without
retraining. Resource interruption retains the available checkpoint/artifact
and reports unavailable; it does not authorize a forced continuation.

This task does not establish general English understanding, reasoning,
factual correctness, chat or release readiness, H4 superiority, softmax
removal, recurrence, integer/table lowering or completion of #973/#954.

## Activated implementation checks

Three focused data checks cover the serialized oracle and split, bag-preserved
absence and lexical round trip, and construction-only training access with
corrupt-label rejection. Five focused decision checks cover unknown masking,
question ignoring, matched causal-control work, carried resource budgets and
post-load artifact mutation. Independent review covers the fixed optimizer
clock, sampler/checkpoint continuity, label boundary, immutable lineage and
conditional claims. All eight checks passed before fitting. Broad workspace,
BDD, no_std, native/wasm, kappa, legacy campaign and release QA remain dormant.
The protected queue supplies transport acknowledgements only.

## Reproduction entry points

From `tools/r4-softmax-trainer`, the locked Python environment runs:

```text
python -m r4_softmax_trainer.zoology_english_binding prepare ROOT --frames-root FRAMES
python -m r4_softmax_trainer.zoology_english_binding fit ROOT
python -m r4_softmax_trainer.zoology_english_binding run ROOT
python -m r4_softmax_trainer.zoology_english_binding verify ROOT
```

`prepare` requires a new evidence root. `fit` resumes only the same frozen
trajectory and returns immutable completed results. `run` scores only a
complete fixed-dose fit. `verify` must execute in a fresh process and carries
the fitting and evaluation time into its budget. New execution is not an
authorization to overwrite or tune this frozen campaign.

The implementation, exact dataset, predecessor evidence and native frames are
bound in the preparation envelope. The following append-only entries will
record the preparation identity, actual dose/outcome and protected delivery.

## Preparation published before fitting (2026-09-02)

Implementation freeze: `25ab0ab2`. Independent source review approved the
serialized-data boundary, fixed learning clock, admission/resume behavior,
conditional R4 comparison and exact replay. The review's fitted-artifact
rehash finding was fixed before the freeze and covered by the fifth decision
check. No optimizer update or fitted-model evaluation had run at publication.

- Preparation: `blake3:c926e16516ef0f1d8242dc0af39a04be46cb082bb6c76590bc73f2717e027ca8`.
- Implementation closure (199 files): `blake3:da90e0ecb9a7fe82d9eaccc3f0c38ae597036d5de5e68539b13746c21388fbef`.
- Dataset tree: `blake3:173e2275b014f87fc5ebb06a443e3aba6d44f8c2017c42574d6070e71352d144`.
- Dataset manifest: `blake3:fbf3c3d6b5694dea16b2d5c1f5e4fb5d198b339b36a80b0dab91d4714ce04d7d`.
- Construction bytes (3,768,792): `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`.
- Development bytes (471,496): `blake3:5e43c75f4728cde1d5ed776e5c2afd0873baade00602915555ff6dc5ba156890`.
- Vocabulary bytes (65,283): `blake3:aa001c3a4369ad2f8bb3596a316270bd72b736f927158ee04403116b430c649d`.
- Unchanged native frame bundle: `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.

The [raw preparation envelope](r4_zoology_english_binding_1063_preparation.json)
contains the complete file inventory, lexical/data audit, fixed decisions and
learning/resource policy. Preparation validates development semantics; the
optimizer receives no development rows or decisions.

## Fixed-dose result and exact replay (2026-09-02)

The complete frozen fit finished with `FIT_COMPLETE`; the scientific terminal
is **`ENGLISH_BINDING_CONSTRUCTION_MISS`**. The optimizer made exactly 3,920
updates and 2,007,040 single-answer presentations, comprising 1,846,452
supported and 160,588 unknown presentations. It received zero development
decisions. The final artifact was retained without a checkpoint substitution,
extra dose or new seed.

| Final-artifact measurement | Observed | Frozen requirement |
| --- | ---: | ---: |
| Construction supported answers | 2,396 / 8,192 = 29.2480% | 8,111 / 8,192 |
| Development supported answers | 218 / 1,024 = 21.2891% | 973 / 1,024 |
| Complete four-answer groups | 0 / 256 | 231 / 256 |
| Complete same-owner groups | 0 / 128 | 116 / 128 |
| Complete same-object groups | 0 / 128 | 116 / 128 |
| Development unknown answers | 37 / 256 = 14.4531% | 244 / 256 |

Construction supported NLL was `1.6081310920417309`. Development supported
NLL was `1.6934579554363154`, unknown NLL `1.5795006966218352`, and pooled NLL
`1.6706665277481079`. The pooled 255/1,280 accuracy is not a substitute for the
separate supported and unknown results above.

The question/history contrasts localize the visible behavioral failure:
changing the question while retaining the same facts changed the prediction
in only `12/512 = 2.34375%` of comparisons. Changing the paired location
assignments with a fixed question changed it in `106/512 = 20.703125%`.
Changed predictions are diagnostic counts, not necessarily correct answers;
none of the 256 groups had all four answers correct. The swapped history
retained the old answer 109/512 times, and the missing-binding history retained
the original answer 57/256 times. The raw result retains all grouped target
and prediction IDs and the complete quartet correctness-pattern histogram.

The first frozen example illustrates the problem without selecting a favorable
case. Its facts include `omar put the shoe in the pouch.` and
`omar put the coin in the crate.`, plus `nora put the shoe in the cabinet.` and
an unrelated distractor. Asking about Omar's shoe should produce `pouch`;
asking about his coin should produce `crate`. The model produced `cabinet`
for both, and kept doing so after their locations were swapped and after
Omar's shoe binding was removed. It did not reliably use the requested pair.

Because the language criteria missed, coherent R4 inference and its
inconsistent-transport control are **`NOT_RUN_ENGLISH_BINDING_MISS`**. This is
the predeclared conditional action. The previous #1059/#1061 preservation and
transport-sensitivity results remain intact; #1057's original
`NOT_RUN_PRIMARY_MISS` control remains historical. No geometry expansion,
generation, table/integer lowering or further learning ran.

The eight-update admission passed at mean `0.07179862499981482 s/update`, with
`411.09527624909447 s` projected remaining including the evaluation allowance.
Actual fit time was `247.1106636249997 s`; final scoring took
`1.1293503750002856 s` and fresh-process replay `1.1256050000001778 s`.
The combined measured time was `249.36561916699975 s`, with maximum peak RSS
`521,437,184 bytes = 0.485626220703125 GiB`. Execution used one CPU process,
eight Apple Accelerate threads and one inter-op thread. No hardware or runtime
limit explains this terminal.

Exact fresh-process replay passed for the complete final-artifact evidence,
including predictions, full-vocabulary logit and attention digests, metrics,
conditional decisions and examples. Scoring/replay made zero optimizer
updates, preserved learned tensor identity and produced zero future attention
weights. Replay did not load optimizer/checkpoint RNG state or prior models.

- Fit CID: `blake3:7c857e5b8a1506cdab8db7d858428cb78639e10fb419b51396192d3e8aa90a79`.
- Final artifact (1,217,024 bytes): `blake3:a4eb5ef76c387ca6ebe9f185b1a5ad023c81291ce4cc9000bb5d23248aaef282`.
- Learned tensor state: `blake3:79f2d4fcb3b185cc6e65a3bf403585bc3cba2416000c128feac82c3dde32804a`.
- Result CID: `blake3:aaca100c5c2b8abfb126937523c5cce44bb7e6ca2eb8d48260f42e9281606e0f`.
- Exact evidence CID: `blake3:50a8cedfaad543dbc6e974d3eb56c9fabad7dc93d7ccf1c3a19cb64e27927ecb`.
- Replay CID: `blake3:dd5984c22d507faa1e2cea0f9b0c8051fbd3ec923cf53c896768e62708295e02`.

Raw committed records: [fit](r4_zoology_english_binding_1063_fit.json),
[result](r4_zoology_english_binding_1063_result.json), and
[replay](r4_zoology_english_binding_1063_replay.json). The local evidence root is
`/Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding`;
it retains the exact data, final model, optimizer/RNG/sampler checkpoint and
execution logs. Code/data/decision bindings did not change after preparation.

## Interpretation and next action

The miss is already present on construction examples. It therefore cannot be
explained solely by held-out combinations, and it is not a test of whether
ordinary causal attention or coherent R4 transport exists. This fixed lexical
recipe did not learn the needed owner-object binding.

The source comparison identifies additional computations introduced by the
English surface. Accepted MQAR puts a single query-key token directly at the
supervised readout position, with adjacent key/value facts. Here the question
owner is at position 35, object at 37 and readout at the constant colon at 40;
each fact separates its owner, object and location. In evaluation mode, the
first-layer query at 40 begins from the same colon-plus-position embedding in
every row. The model must gather the question, compose both lexical attributes
and associate the separated fact fields before copying the location. The
source results did not establish learning of those added compositions. This
is a code-level distinction, not proof that two layers cannot perform them.

The supported phase also has 1,204,224 supervised queries, versus 8,000,000 in
#1050's passing source run. Equal optimizer updates do not equate supervision
or task difficulty. No conclusion about sufficient training dose or an
architectural capacity ceiling follows from this one run.

The next recommendation is one separately frozen, construction-only
behavioral diagnostic on the retained final artifact, with zero training:
classify selected locations as target, same-owner distractor, same-object
distractor, unrelated fact or outside-history/unknown; then stratify the
question and location-swap responses by question type and fact slot. That can
distinguish weak use of owner, object or question information from a fixed
position preference before selecting a narrowly targeted learning/readout
change. The current construction log contains aggregates/digests only, so this
diagnostic requires a new bounded inference contract; it has not been run or
silently added to #1063. Additional geometry is deferred.

For precision, the contract's balance wording refers to question-pair types,
location targets and relevant fact slots. Individual owner/object occurrence
counts are recorded in the preparation audit; they are not claimed identical.
#973 remains open and #954 remains blocked.
