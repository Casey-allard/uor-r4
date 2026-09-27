# Selected dialogue child: native conversion and retention

The selected complete-prefix child now exports, reloads and generates through
the native integer development profile. Its hard conversion loses useful
relations that the same continuous child retained. Preserve this implementation
and diagnostic artifact; do not promote the converted model. The observed
losses justify one prospectively selected response-aware code-choice repair,
with useful-output retention as its decision boundary. No repair fit has run.

The [portable evidence packet](../evidence/dialogue-child-native-observation-2026-09-27.json)
contains source and artifact identities, all 174 actual replies with exact IDs,
prompts, bytes, stops and histories, the common-trace summary, build and execution
receipts, and independent source, numerical and whole-output reviews. Original
per-selection records and per-position traces remain in the bound sealed reports.
This is open development evidence, not fresh held-out qualification.

## The child and its executable path

The selected `dialogue-study-full_prefix-1` checkpoint completed 1,024 local
updates and 1,165,549 response/EOS target visits. Its model-file SHA-256 is
`98aca5ab14a9edab58dcd2d74d71e74d3904c27e1ce3c126fb66b670cc2baa1c`;
its recomputed parameter fingerprint is
`1eef006a29767f06fd07d29e03f4e9cdf254a6a8a0562c5c8a019b8c0e6bb1d6`.
The R1d ancestor completed 2,237 updates in the preceding historical training run.
Conversion introduces zero new model or optimizer updates.

Source `03bfaaf5f3bbd71827433966651bdc117b469114` adds the typed
`DialogueChildArtifact` loader and `dialogue-child-integer-bridge` example.
The loader verifies the sealed fit/final checkpoint and actual child arrays,
then joins saved clocks, protocol, tokenizer, corpus manifest and schedule.
It checks the optimizer-file identity without restoring Adam, and verifies
the recorded data joins without rescanning the corpus. A separate conversion
schema preserves the historical exporter and validator. Provenance paths do
not cause source-checkpoint, optimizer or corpus access during serving.

The bundle identity is
`55b3fa54a424e46681bfde5cd257ff0da51e0f560fef550fa6d84d07f698562a`.
It remains width 576, read-vector width 64, context 256, full causal access,
Quaternion transport and Dot reading, with vocabulary 4,096. Read-vector width
is not an access limit. Dense coefficient access remains; this is not D5
selected access or a change to the accepted width-256 artifacts.

```sh
dialogue-child-integer-bridge CHILD_REPORT_ROOT CORPUS_MANIFEST TOKENIZER TABLE_ROOT NEW_CONVERSION_ROOT
uor-r4-integer pack-dialogue576 NEW_CONVERSION_ROOT/packed TABLE_ROOT TOKENIZER NEW_BUNDLE_ROOT
dialogue-integer-observe --child CHILD_REPORT_ROOT CORPUS_MANIFEST TOKENIZER NEW_CONVERSION_ROOT/packed NEW_BUNDLE_ROOT REQUESTS CHILD_RESPONSES NEW_OBSERVATION_ROOT
```

Every output root must be new. The observer's explicit `--child` mode uses this
child's own saved responses. The historical positional invocation still selects
the historical parent. Current FF must reproduce the saved prompt IDs, selected
IDs, bytes and stops; all actual outputs are retained before a mismatch fails.

## Numerical localization and actual replies

FF reproduces all 58 saved child replies exactly. FF, QF, QQ and integer then
consume the same 3,914 saved prediction positions per form: 1,508 assistant
targets and 2,406 other targets. These targets are the saved child's generated
choices, not independently correct answers or corpus evaluation labels.
FF uses continuous parameters/interfaces; QF replaces parameters with their
decoded packed values; QQ also quantizes interfaces; integer executes the
native arithmetic. The trace records raw probability half-L1 differences.

| Adjacent comparison | Mean half-L1 | Greedy differences / 3,914 | Assistant differences / 1,508 |
|---|---:|---:|---:|
| FF → QF | 0.1624243395 | 909 | 237 |
| QF → QQ | 0.0010222333 | 3 | 0 |
| QQ → integer | 0.0008871466 | 4 | 1 |

Saved assistant-target NLL is 1.035923796 / 1.303490419 / 1.303527429 /
1.303479592 across those four forms. The largest adjacent change is parameter
conversion, including its recurrent propagation. This does not isolate a
particular tensor, clipping, or a unique fraction attributable to four-bit
matrix rounding. It is already visible at initial BOS before stored-history
reading; a new attention mechanism is not required to explain that perturbation.

Separate FF, QQ and integer generations each cover all 38 requests/58 turns
using their own actual histories. QF free generation was not added. Principal
and independent reviewers read every reply and retain the five prior subjective
baseline disagreements instead of turning label counts into acceptance scores.

| Requested behavior | Same continuous child | QQ and native integer |
|---|---|---|
| Cat-name recall | Names Momo, with a poor repetitive tail | Ends at “The cat is named”; name is lost |
| Favorite-color recall | Answers green despite later blue, then repeats | Favorite-color relation is lost; integer “greenhouse” is not an answer |
| Job recall | Repeats teacher in malformed wording | “The job is nearby” confuses occupation with school location |
| Birthday / instrument | Partial July / piano answers | Those values disappear from final answers |
| France's capital | Paris plus an unnecessary sentence | Correct, cleaner Paris answer |
| Sister's location | Tokyo with an unfinished repetitive tail | Tokyo relation remains, still repetitive |
| Car color | Blue car with a short cycle | Blue-car relation remains and ends at EOS |
| Name / favorite food | Alex / pizza with repetition | Alex remains but completion worsens; pizza remains cued in a malformed predicate |

The remaining factual, instruction and underspecified requests still expose
substantial failures. A changed greeting is a judgment-sensitive regression;
it is not the decisive reason for the repair. Neither general chat nor coherent
prose is established. The blue-car retention differs from the historical R1d
conversion and must not be carried over from that ancestor's result.

QQ and integer match selected IDs on 52/58 turns, prompt IDs on 56/58 and stops
on 57/58. The first color-turn divergence changes the next two prompts, so those
later outputs are not predictions on identical histories. All 58 native history
and pending-token accounts agree: 38 initial boundaries, 11 model-EOS boundaries
and 9 explicit caller-EOS boundaries. Greedy selection leaves the sampler cursor
unchanged. State comparisons are confined to the common-input traces.

## Engineering checks and cost

Thirteen focused release checks pass: two child-lineage checks, seven retained
width-576/bundle checks, and four observer checks. Both training examples and
the integer binary compile offline; direct Rust formatting and diff checks
pass. Independent source review covers lineage validation, dispatch and the
loaded artifact path. These checks establish engineering behavior, not language.

Three build preparations are retained. The first exited before any job because
the receipt glob also matched a specification filename. The second stopped its
own compiler group at a sampled 3,231,301,632 bytes, just beyond the projected
3 GiB limit; no test or model ran. Before the successful third attempt, the
recorded local projection reduced Cargo concurrency from two jobs to one and
increased build RAM by 1 GiB to 4 GiB under standing owner authorization. Other
time, storage and reserve limits stayed fixed. The successful attempt took
362.956 seconds; all three preparations total 399.962 seconds.

The completed conversion, packaging and observation supervisor took
56.245 seconds, with maximum child RSS 205,946,880 bytes. Its process components
were 2.059 / 1.034 / 52.679 seconds; these are nested costs, not extra cumulative
charges. The observer executed 25,928 model steps: 15,656 common-trace steps and
10,272 generation steps. No new backward or optimizer updates ran.

On identical trace inputs, recorded model-step time was FF 2.823 seconds,
QF 2.702, QQ 3.624 and integer 12.593. This single instrumented pass establishes
no native speed advantage and measures no energy. Free generation has unequal
work: FF/QQ replay prefixes while integer retains its session, with
3,914 / 3,832 / 2,526 calls respectively. Do not infer an equal-workload speed
ratio from those totals. Preparation, review and delivery are charged once in
the packet's cumulative ledger snapshot; process timings are breakdowns only.
At 13:26:12 UTC, this increment charged 2,518,488 ms of complete elapsed work,
bringing owned elapsed to 46,388,080 / 64,800,000 ms and the shared cumulative
ledger to 694,543,660 / the verified 722,400,000 ms allowance. The ledger's
foreign-reported 744,000,000 ms limit was preserved but not adopted. Physical
free space was 30,149,734,400 bytes internally and 172,491,235,328 bytes on the
owner SSD. The 128 MiB stop margin remains. Subsequent delivery/reporting time
continues from this cursor and is not claimed inside this snapshot.

## Decision and next implementation

The declared skip-repair condition is not met: Momo and green are concrete lost
relations, with further damage to partial answers. Keep nearest-hard as the
same-child baseline and implement one response-aware choice of legal neighboring
codes using the complete-prefix training population. Keep the continuous child,
its exact exported grids, serving arithmetic and evaluator identities fixed.
The existing development replies are for observation, not new training labels.
Select the dose and complete resource projection before fitting.

A repair must improve useful complete replies against nearest-hard and preserve
the continuous child's demonstrated relations, while checking all other outputs
for tradeoffs. Lower NLL, more greedy matches, changed stop counts or correct
keywords alone do not qualify it. Failure preserves the candidate and motivates
a different causal implementation; it does not authorize repeated dose/scale
sweeps. No final held-out set has been opened.

The [prior width-256 learned-rounding result](learned-rounding-result-2026-09-25.md)
improved numerical retention but still lost complete source answers. Existing calibration already searches three dyadic
row scales; the current result is not proof that clipping is the cause.
The optional [AdaRound reference](https://proceedings.mlr.press/v119/nagel20a.html)
supports data-aware relaxed rounding with a layerwise objective. Using the
project's complete-trajectory supervised response objective is a separate
choice, not a demonstrated dialogue result from that paper.
[GPTQ](https://arxiv.org/abs/2210.17323) supplies a one-shot approximate
second-order alternative; its transformer/GPU results do not establish native
recurrent dialogue retention. Claude's new GPTQ source has no trained result
packet in the reviewed head, and its grouped codec/separate head differ from
this path. Neither external method is adopted by reputation alone.

References #973 and #820. Protected delivery remains separate from model
promotion; the broader geometric attention, useful inference, prose, chat and
reasoning programme remains open.
