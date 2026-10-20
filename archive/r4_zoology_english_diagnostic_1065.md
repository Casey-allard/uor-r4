# Retained English construction diagnostic — #1065

## Frozen contract (2026-09-02; before inference)

Issue [#1065](https://github.com/UOR-Foundation/uor-r4/issues/1065) continues
#973 after the [#1063 English curriculum](r4_zoology_english_binding_1063.md).
It asks which behavioral failure profile is present in the retained final
model's construction answers. This is a diagnostic of an existing negative,
not a new capability or geometry qualification.

The sole model is #1063's final update-3,920 artifact, with 303,744 parameters,
two layers, one head, width 64 and an unchanged tied 4,096-token output head.
The prior construction result is exactly `2,396/8,192 = 29.248046875%`, NLL
`1.6081310920417309`. No optimizer, checkpoint/RNG state, development tensor
payload or native geometric frame payload is opened. Published JSON envelopes
include historical development statistics and artifact metadata; binding those
documents does not constitute a new development model decision.

The population is the exact 8,192 supported rows from the retained construction
container: 2,048 groups, each in order base-q0, base-q1, swapped-q0, swapped-q1.
The container also holds the old unknown rows; they are excluded before model
execution. No examples, vocabulary items, labels or worlds are newly generated.
Input length remains 41, selected readout position 40, batch size 256, eight CPU
threads and one inter-op thread. All vocabulary logits and returned causal
attention are included in reproduction. Labels and metadata stay outside the
model invocation.

Before interpreting a diagnostic, reproduce every field of #1063's saved
construction score, including the prediction, selected-logit and attention
digests and both NLL reductions. Any mismatch returns
`UNAVAILABLE_CONSTRUCTION_REPRODUCTION` and suppresses interpretation.

## Categories and denominators

Definitions: relative to each row's actual queried owner/object and its ordered
four facts, classify the predicted token as:

1. the target fact's location;
2. a same-owner, different-object fact's location;
3. a same-object, different-owner fact's location;
4. an unrelated fact's location;
5. a vocabulary location absent from the history;
6. `unknown`;
7. another vocabulary token.

Distinct history locations make these categories mutually exclusive. The q0
question has one fact of each of the first four categories. A q1 question has
one absent attribute-confound category and two unrelated facts. Report
category-eligible rows and eligible fact slots as well as raw counts; an absent
category has unavailable rates. Compare owner/object confounds on q0, where
their opportunities are equal. Also report unknown/out-of-history/other
predictions against all q0 errors, so a concentration within in-history errors
cannot silently stand for the whole failure.

Report all rows and strata by question type (same-owner or same-object question
pair), q0/q1, base/swapped history, and target displayed slot. Display target
exposure, selected-slot counts and correctness together. This separates lexical
role labels from their physical positions; a position association is not by
itself an internal causal explanation.

## Matched responses and next-action rule

There are 4,096 fixed-history question contrasts `(0,1)` and `(2,3)`, and 4,096
fixed-question location exchanges `(0,2)` and `(1,3)`. Every contrast changes
the correct target. Report prediction changes/invariance, both answers correct,
and changed predictions without both correct, overall and by question type.

Definition: when left and right targets are location IDs `a` and `b`, retain
those same two IDs in both logits and compute

```text
delta = [z_left(a) - z_left(b)] - [z_right(a) - z_right(b)]
```

Positive delta denotes a response in the desired direction, not successful
binding. Report its signed distribution, full-vocabulary absolute logit
differences, and each target's margin over the best other vocabulary token.
An unchanged top-1 can coexist with a useful sub-top-1 response.

The following strict-majority rules are descriptive follow-up selectors.
They are not capability thresholds, causal guarantees or authorization for a
new fit. A majority means `2 * numerator > denominator`, with an empty
denominator unavailable. All flags and denominators remain visible.

| Profile | Next focused investigation |
| --- | --- |
| A displayed slot receives most in-history selections, and an attribute confound receives most q0 in-history errors | Joint position-versus-attribute discrimination |
| Displayed-slot majority alone | Position-sensitive selection/readout |
| Same-owner confound majority | Object disambiguation |
| Same-object confound majority | Owner disambiguation |
| Neither concentration, but most matched question pairs keep the same prediction | Question-to-readout sensitivity |
| None of these profiles | Distributed binding errors |

## Binding, resources and named verification

The preparation directly binds the published/local #1063 preparation, fit,
result and replay envelopes, all 199 historical implementation files, the
retained model, construction container, vocabulary and manifest. It adds the
new diagnostic implementation and focused tests. It deliberately does not call
the old whole-data preparation validator, which would open development and
frame payloads. The construction-only loader independently parses supported
labels and verifies canonical group/variant metadata.

Resource ceiling: 300 seconds combined diagnostic plus fresh-process replay,
and 2 GiB maximum RSS per process. The cost basis is #1063's approximately
1.13-second final evaluation; parsing and descriptive reductions add work but
require no fit. All supported rows can contribute to the profile, so this
diagnostic can distinguish follow-up directions across the entire failure.
There is no hours-scale run or hidden admission sweep.

Named checks are the new semantic/classification/pair-margin/action tests,
construction access/binding checks, exact old-score reproduction, independent
fresh-process replay, and review of this implementation and its evidence.
Broad QA remains dormant. Required GitHub queue statuses acknowledge transport
only and are not tests or scientific PASS evidence. Source/model identities
are checked before and after execution. The first canonical quartet supplies
the fixed readable example; examples are not selected after viewing errors.

Prior attention positives #1059 and #1061 remain intact. #1063's construction
and development negatives remain intact. Geometry expansion, new training,
general English understanding, reasoning, chat readiness, softmax removal and
integer/table lowering do not follow from this diagnostic. #973 stays open;
#954 remains blocked.

## Reproduction commands

From `tools/r4-softmax-trainer`, use the locked offline environment. The
following paths identify the original local retained evidence; a relocated
reproduction needs a separately recorded preparation with its new roots.

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_english_diagnostic prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_english_diagnostic run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic
.venv/bin/python -m r4_softmax_trainer.zoology_english_diagnostic verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic
```

Each run/verify command is a separate process. Existing preparation, started
markers, results and replay files are immutable; these commands intentionally
refuse to overwrite them. The checked-in JSON evidence accompanies this record.

## Published preparation

The executable and contract were frozen at `58bd1559`. All 11 named focused
checks passed (six direct-binding/access, three analysis, two campaign), and
independent source review found no blocking issue. No fitted inference was
performed by those checks or that review.

The [preparation envelope](r4_zoology_english_diagnostic_1065_preparation.json)
binds 211 implementation files, including all 199 predecessor records:

- Preparation: `blake3:5679f721fa60c16601a4d3a8ca46397055b89d7769dfae1ca099a1e3f3fbe5a9`
- Implementation: `blake3:d22cde07d38be3e3042039a93c6cdc06c5f4c772c7ba5490905889215f24827b`
- Retained model: `blake3:a4eb5ef76c387ca6ebe9f185b1a5ad023c81291ce4cc9000bb5d23248aaef282`

The preparation and this section were published before construction inference.

## Executed result (2026-09-02)

Terminal: `CONSTRUCTION_DIAGNOSTIC_COMPLETE`; descriptive follow-up selector:
`QUESTION_READOUT`. The old construction score reproduced in all 13 recorded
fields, including every selected vocabulary logit, prediction, attention
digest, and both NLL reductions. Accuracy remains `2,396/8,192 = 29.2480%`;
this diagnostic performs no learning and does not revise #1063's negative.

The [raw result](r4_zoology_english_diagnostic_1065_result.json) and
[fresh-process replay](r4_zoology_english_diagnostic_1065_replay.json) record
identical complete evidence. Run/replay PIDs were `9725` and `9734`. Recorded
execution times were `1.770481625 s` and `1.663344875 s`, or
`3.433826875 s` combined. Peak RSS was `832,045,056 bytes = 0.774902344 GiB`,
below the frozen 2-GiB ceiling. Runtime was Python 3.12.14, PyTorch 2.7.1,
CPU Apple Accelerate, eight threads and one inter-op thread.

### Actual answer categories

| Predicted category | Count across 8,192 supported rows |
| --- | ---: |
| Correct target | 2,396 |
| Same-owner, wrong-object fact | 1,443 |
| Same-object, wrong-owner fact | 1,425 |
| Unrelated fact location | 1,641 |
| `unknown` | 1,287 |
| Location absent from the history | 0 |
| Other vocabulary output | 0 |

Thus `6,905/8,192 = 84.2896%` of answers selected a location present in the
history; the remaining `15.7104%` answered `unknown`. This shows a restricted
output pattern on this population, not reliable owner-object binding or
general lexical understanding.

For q0, where each confound has exactly one opportunity, the 2,253 in-history
errors divide into same-owner `841/2,253 = 37.3280%`, same-object
`834/2,253 = 37.0173%`, and unrelated `578/2,253 = 25.6547%`. Neither
attribute-confound category has a majority. An additional 641 q0 answers are
`unknown`, constituting `641/2,894 = 22.1493%` of all q0 errors. In the crossed
q1 strata, the missing same-object confound for same-owner questions and the
missing same-owner confound for same-object questions both have zero eligible
rows/facts and unavailable rates. The two unrelated facts are counted as two
opportunities, not one.

The crossed q0 strata still contain structure: in same-owner question groups,
the same-owner confound contributes `601/1,138 = 52.8120%` of in-history
errors; in same-object groups, the same-object confound contributes
`585/1,115 = 52.4664%`. These type-specific concentrations balance in the
predeclared pooled q0 selector. Absence of a universal owner/object majority
does not mean absence of attribute effects.

### Displayed positions

Displayed positions below are numbered 1–4 for readability; raw JSON uses
zero-based slots. Each slot has exactly 2,048 target exposures.

| Displayed fact | Selections among 6,905 in-history answers | Selection share | Correct when target is there |
| --- | ---: | ---: | ---: |
| 1 | 1,917 | 27.7625% | 680 / 2,048 |
| 2 | 1,704 | 24.6778% | 599 / 2,048 |
| 3 | 1,773 | 25.6770% | 592 / 2,048 |
| 4 | 1,511 | 21.8827% | 525 / 2,048 |

There is a modest positional association; there is no displayed-slot majority.
The selector therefore does not prioritize a single-position explanation.

### Responses to the question and the facts

| Matched comparison | Pairs | Answer changed | Both answers correct |
| --- | ---: | ---: | ---: |
| Question changes; facts fixed | 4,096 | 122 (2.9785%) | 20 (0.4883%) |
| Object changes in question; owner fixed | 2,048 | 33 (1.6113%) | 6 (0.2930%) |
| Owner changes in question; object fixed | 2,048 | 89 (4.3457%) | 14 (0.6836%) |
| Queried locations exchanged; question fixed | 4,096 | 1,134 (27.6855%) | 247 (6.0303%) |

The dominant observation is `3,974/4,096 = 97.0215%` question-pair top-1
invariance despite different correct answers. Raw logits are not completely
constant: their full-vocabulary mean absolute change is `0.013731617`, maximum
`0.253464222`. However, the fixed-target question contrast is positive in
2,040 pairs and negative in 2,056, with mean `0.0000143708894` and median
`-0.0000784397125`. The change has no consistently helpful target direction
under this summary. It is not merely a strong correct-direction signal hidden
under an unchanged winner.

Location exchanges cause larger changes, with full-vocabulary mean absolute
difference `0.032178366` and maximum `1.790657043`, yet only 247 pairs have both
answers correct. Their target contrast is positive in 2,053 and negative in
2,043. Question and location contrast means are algebraically related by the
quartet design and are **not independent corroborating measurements**.

The fixed first quartet illustrates the behavior. Its history places Leon's
key in the basket, Liam's key in the cabinet, Mila's coin in the locker, and
Liam's toy in the trunk. Asking about Liam's key or toy, before or after
exchanging their cabinet/trunk assignments, yields `basket` in all four rows.
The required answers are respectively `cabinet`, `trunk`, `trunk`, `cabinet`.

### Interpretation and next recommendation

The predeclared `QUESTION_READOUT` branch follows because neither the position
nor attribute-confound majority flag fires, while question invariance does.
This is a behavioral localization, not proof of an internal bottleneck. It does
not distinguish absent query information from information lost or outweighed
later in the computation.

The next recommendation is one controlled **readout-placement learning
experiment**: move the selected answer readout from the constant colon at
position 40 to the queried object word at position 37. Keep the ordinary
two-layer attention cell, lexical construction rows, labels, seed, optimizer
and fixed dose matched. That exposes one query attribute directly at the
readout, with the queried owner already in its causal prefix, resembling the
direct query-key access in the successful MQAR cell. This is a concrete
hypothesis to test, not a demonstrated repair. It requires a separate frozen
contract; this issue performs zero such training. Any new transfer claim must
use a separately frozen, previously unexamined development population rather
than treating #1063's disclosed development rows as fresh evidence.

The sole change would be the supervised answer position `40 -> 37` in a fresh
seed-123 fit, not an inference-time move of the already-trained weights.
Position 37's literal next input token is `?`; its proposed output target is
an explicit supervised answer label. Report both question types separately:
direct object access may assist object discrimination while owner distinction
must still be learned. If the fixed construction fit succeeds, evaluate the
single final artifact on the new frozen development population. If it still
misses, reject readout placement as sufficient and retain that result without
extra dose. This proposal changes neither geometry nor the ordinary attention
cell.

Preserve the original artifact and all previous attention positives. Geometry
expansion remains deferred until the language query/readout path learns this
binding task. Broad generalization, geometric superiority, reasoning, chat
readiness and integer/table lowering remain unestablished.

### Retained identities

- Result: `blake3:65b23631b10fe62b215411932cd9fe45f76b43d6b8503d0f2e74dc3d256c9b61`
- Evidence: `blake3:45ee741e2262afbe9e7909efbd8f3139f924fedea462a96b08af577fc54bb988`
- Replay: `blake3:7222a680c300552ab097ce184500c90c0e44ede8248c4c3f752aa09f4232c0ca`
- Supported tensor population: `blake3:eb9b8000be8e0ced9877e13c49ef243cf0b4492021cb80bab6247cd3c3ce2be5`
- Unchanged model state: `blake3:79f2d4fcb3b185cc6e65a3bf403585bc3cba2416000c128feac82c3dde32804a`

The local evidence directory is
`/Users/casey.allard/uor-r4/.uor-models/research/issue-1065-zoology-english-diagnostic`.
The original #1063 model, checkpoint and data remain in their source directory.
Optimizer updates, new development model decisions, development payload reads,
checkpoint/optimizer/RNG-state reads, native frame payload reads, new data rows
and geometry changes are all zero for this diagnostic. No generation ran.

Independent evidence review recomputed the three new canonical envelopes,
evidence hash, all 13 source-score comparisons, 219 permitted file records,
category denominators, paired summaries, state/runtime/resource bindings and
the predeclared focus. No blocking findings remained. The reviewer ran no
fitted model or additional diagnostic pass.
