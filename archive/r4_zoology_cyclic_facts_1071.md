# Cyclic fact-order learning at the matched dose — #1071

## Frozen contract (2026-09-02; before fitting or model scoring)

Issue [#1071](https://github.com/UOR-Foundation/uor-r4/issues/1071) continues
#973 after [#1069](r4_zoology_joint_query_1069.md). The owner residual raised
aggregate construction accuracy to 50.2686%, but failed object-pair preservation
and reached 99.6582% at target slot 4 versus 29.4922–36.0352% elsewhere.
The retained reference is the plain [#1067 query-object readout](r4_zoology_query_readout_1067.md).
This experiment teaches the same bindings across positions before changing
geometry. One fresh fit is authorized; no extra dose or retuning follows.

## Intervention, exact dose and reachability

After the unchanged sampler returns each batch, reshape input tokens 1–32
into four eight-token fact blocks and **right-rotate** the fact axis by:

```text
supported phase: floor(completed_updates / 16) modulo 4
mixed phase:     floor((completed_updates - 2352) / 20) modulo 4
```

`completed_updates` is the zero-based count before the next update. The offset
resets to zero at update 2,352. Each full supported traversal has 16 batches;
each mixed traversal has 20. All variants of a world share the same offset
within a traversal. Clone-and-roll also executes at offset zero, so the first
eight real updates measure the same transform path used later. Rotation uses
no random draws and changes no labels or metadata. Original construction and
vocabulary bytes stay identical. BOS token 0, query suffix 33–40, all 41 input
tokens and supervised position 37 stay intact. Each world has distinct
owner-object keys; order changes preserve answer and absence semantics.

Use the plain source model: two attention layers, one head, width 64, tied
4,096-token head, maximum position 120 and 303,744 parameters. No owner residual.
Preserve seed 123, fresh initialization, source normalization/dropout, AdamW
learning rate 0.00046415888336127773, weight decay 0.1, batch 512, and the
original cosine clock. The sole evaluated candidate is update 3,920:
2,352 supported plus 1,568 mixed updates, **2,007,040 presentations =
1,846,452 supported + 160,588 unknown**. Sampler/RNG and dropout tensor shapes
remain matched; the sampled token values are rotated. No prior learned state
or optimizer is loaded into fitting.

Expected total presentations by offset are **507,904 / 507,904 / 501,760 /
489,472**. Supported-phase updates are 592 / 592 / 592 / 576; mixed-phase
updates are 400 / 400 / 388 / 380. The mixed phase ends after 78 full traversals
plus eight batches at offset 2. Derive supported/unknown exposure from the
actual existing counters, including the partial tail, rather than assuming a
20% unknown fraction. The derived ledger adds no checkpoint state.

The readout is an explicit supervised answer at the query object; its literal
next input token is `?`. This does not establish ordinary generated answers.
All 8,192 supported construction decisions have the four facts available in
the causal prefix. Canonical paired headroom is 2,001 owner and 1,601 object
pairs, above the declared 205-pair gain. No geometric expansion is needed to
reach this population.

## Matched comparisons and divergent decisions

After the fit, score the retained #1067 model and candidate on the same 8,192
supported rows at offsets 0, 1, 2 and 3. Offset zero **is** canonical order,
not a fifth population. Require the reference's entire canonical score and
diagnostic to reproduce the published #1067 evidence exactly before interpreting
the comparison. Retained reference weights are hashed during preparation and
loaded only for final scoring/replay; default fitting validation does not open
them. The checked plain-model loader binds artifact and learned-state identities.

In **every rotation**, adoption requires:

- For each question type, both-correct pairs at least
  `min(same-rotation reference + 205, 2048)`. The untruncated gain is
  10.009765625 percentage points. Flag ceiling-limited cases explicitly.
- Each of four target slots at least **1,024/2,048 = 50%** correct.
- Overall supported accuracy and each individual question-family accuracy
  preserved against that matching reference. Any paired, individual-family or
  overall regression is reported as a tradeoff and keeps #1067 as reference.

Report per-order NLL, paired question/location-swap behavior, confounds, slots
and fixed first-quartet examples. Two question pairs per world and four cyclic
views are correlated observations. Report worlds with zero/one/two successful
question pairs per order, and world counts with zero through four completely
correct rotations. No independence or significance claim follows. Four cyclic
orders cover only four of the 24 possible fact permutations.

1. Behavior passes but any construction view is below 8,111/8,192:
   `CYCLIC_FACTS_PARTIAL_GAIN`. Retain the improved augmentation recipe and
   address its recorded remaining errors separately. Development stays unscored.
2. Gain/slot criteria miss without regression:
   `CYCLIC_FACTS_BELOW_DECLARED_GAIN_OR_SLOT_FLOOR`. Preserve partial progress,
   retain #1067, and revise the binding learning recipe.
3. Any family/overall regression: `CYCLIC_FACTS_PRESERVATION_MISS`.
   Preserve measured tradeoffs and retain #1067.
4. **Both behavior and all four construction >=8,111/8,192 pass**: open fresh
   development for the final candidate in all four cyclic orders. Require per
   order 973/1,024 supported, 231/256 complete groups, 116/128 complete groups
   for each question type, and 244/256 unknown answers. Return
   `CYCLIC_FACTS_FRESH_TRANSFER_MISS` or `CYCLIC_FACTS_FRESH_BINDING_PASSED`.
   A transfer miss preserves construction; all pass permits a separate unchanged
   R4 inference-preservation step.
5. Incomplete/resource/integrity outcomes are explicit and do not become
   scientific negatives. Retain the exact partial checkpoint and cumulative
   clock; no restarted budget or replacement fit.

A 99% construction pass can legitimately miss the relative behavior criterion
against an unusually strong rotated reference. In that case return
`NOT_RUN_BEHAVIOR_MISS` for development; it is not an integrity contradiction.
A construction miss uses `NOT_RUN_CONSTRUCTION_MISS`. Both cases have zero
fresh-development model decisions.

## Fresh population and access

Before fitting, freeze seed **10712**, 256 fresh groups with all five variants:
1,024 supported and 256 unknown answers, original grammar/lexical map,
64 held-out owner-object pairs and balanced question types, slots and locations.
Exclude construction and **#1063/#1067/#1069** development base,
location-swapped and absent-binding canonical worlds, regardless of display
order, plus exact input overlaps. Reject world reuse across new groups.

Recreate historical populations using their unchanged generators and verify
canonical tensor CIDs against published records; never open retained historical
development payloads. Preparation audits labels, exclusions and balance.
Fitting/default validation neither opens nor regenerates development. Record
per-history and union counts explicitly. Fresh transfer, if admitted, still
concerns familiar grammar and held-out combinations rather than general English.

## Resources, source review and commands

Keep the combined **1,800-second / 4-GiB** ceiling over fit, eight construction
views, conditional development and exact fresh-process replay. Use eight Apple
Accelerate CPU threads and one inter-op thread. Preserve the first-eight-real-
update admission rule, 1.25 projection multiplier and 60-second evaluation
allowance. The prior 277-second fit and 1.8-second single view motivate a
sub-six-minute estimate; the cumulative cap remains binding.

Focused verification covers fact/causal semantics, RNG, phase boundaries,
actual rotation exposure, whole-trainer AST equivalence except declared changes,
data copying/exclusions/access, matched decisions, both development gates,
correlated-world accounting, canonical reference reproduction, immutable model
and state binding, and exact fresh-process complete-evidence replay. Independent
source, preparation and evidence reviews are required. Broad workspace, BDD,
WASM, fuzz, audit and release QA remain dormant; queue statuses acknowledge
transport only.

From `tools/r4-softmax-trainer` in the locked environment:

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding \
  --reference-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts
.venv/bin/python -m r4_softmax_trainer.zoology_cyclic_facts verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1071-zoology-cyclic-facts
```

Preparation/result/replay files are immutable. Publish preparation before the
first update. Append final evidence, reconcile six current mirrors and native
trackers, deliver through the protected PR path, then close only #1071.
Historical records remain intact. #1059 retains 11,900/12,000 = 99.1667%
preservation; #1061 retains ordinary/coherent R4 both at 8,071/8,192 = 98.5229%,
all predictions identical and an 86.2061-point transport-control loss.
More geometry, native-frame work, generation, general English, H4 superiority,
softmax removal, reasoning and chat readiness remain deferred or unestablished.
#973 stays open and #954 blocked.


## Published preparation

Source freeze: `98ea9564`. All **15 focused methods passed** in 8.451 seconds:
three rotation/ledger checks, one whole-trainer AST check, three data/exclusion/
access checks, two reference-binding checks, and six comparison/reveal/world-
accounting checks. Ruff and claim wording passed. Independent source reviews
found no blockers. No fitted model had been scored or trained.

The [preparation envelope](r4_zoology_cyclic_facts_1071_preparation.json) binds
254 implementation files, original construction/vocabulary bytes and the plain
#1067 comparison artifact. Preparation hashes that artifact without loading model
tensors. Its complete published baseline and exact dose are retained.

The audited new population has 1,280 rows and 768 distinct base/swapped/absent
worlds, 64 held-out owner-object pairs, and balanced slots/locations/question
types. Each of the three historical populations has 1,280 rows and 768 worlds;
their union contains 3,840 inputs and 2,304 worlds. Including construction,
exclusion covers 14,080 distinct inputs and 8,448 canonical worlds. Every
new-development overlap count is zero. The default validation path does not
open or regenerate development.

- Preparation: `blake3:92ffef681fe6bd4cfc6532bd811496b6e9c4d052b978c172d3a5ca9bcef8ef05`
- Implementation: `blake3:4693fca24e67aa4bab43ddfc71ee144cb3771c7fca32abea5cb55cdb4356b761`
- Dataset manifest: `blake3:6eeb4831d9558dbbb0ac1235b314fb47e1412a561aad429d2af56acd81dc0a54`
- Dataset tree: `blake3:4856c4365443f562803880a14013569c8ce7c944eda4a174d4b1d8f50f97da1c`
- Fresh development: `blake3:31dca3ff6327c75bd5208758dcd748661002c8bb109346d8c2d4a3b414c02ed9`
- Retained reference model: `blake3:9386849d191b038803ae30b267f2fbf654cb077f48d5f3ddc92137c60875dd98`

These identities, the capped 205-pair gains, each-slot 50% floor, family/overall
preservation and both development-admission gates are published before the
first optimizer update.


## Measured result (2026-09-02; after the fixed fit)

**Terminal: `CYCLIC_FACTS_PRESERVATION_MISS`.** The single augmented fit
regresses against #1067 in every cyclic order, for both question families and
aggregate accuracy. Every declared behavior criterion misses. Retain #1067
as the reference and preserve this negative result; no additional fit or dose
was run. More uniform low accuracy is not successful binding.

Raw evidence: [fit and rotation ledger](r4_zoology_cyclic_facts_1071_fit.json),
[matched construction evidence](r4_zoology_cyclic_facts_1071_result.json), and
[exact fresh-process replay](r4_zoology_cyclic_facts_1071_replay.json).

### Matched construction results

The retained reference's canonical full score **and complete diagnostic**
reproduce #1067 exactly. The new candidate has the same plain model architecture,
seed and exact 3,920-update work ledger; its intervention is the frozen fact
rotation schedule.

| Right-rotation offset | #1067 correct / 8,192 | Augmented correct / 8,192 | Accuracy change | NLL reference → candidate |
| --- | ---: | ---: | ---: | ---: |
| 0 | 3,735 (45.5933%) | 1,702 (20.7764%) | -24.8169 points | 1.3790687658 → 1.6585848257 |
| 1 | 3,354 (40.9424%) | 1,721 (21.0083%) | -19.9341 points | 1.4482103437 → 1.6577429585 |
| 2 | 3,487 (42.5659%) | 1,705 (20.8130%) | -21.7529 points | 1.4342498891 → 1.6589015834 |
| 3 | 3,347 (40.8569%) | 1,711 (20.8862%) | -19.9707 points | 1.4479931369 → 1.6581562310 |

Canonical accuracy loses **2,033 correct answers / 24.8169 percentage points**.
The candidate scores 20.7764–21.0083% across orders, versus 40.8569–45.5933%
for the reference. Canonical NLL worsens by 0.2795160599 nats. No threshold
was adjusted after these results.

| Rotation | Owner both-correct: reference → candidate / 2,048 | Required owner | Object both-correct: reference → candidate / 2,048 | Required object |
| --- | ---: | ---: | ---: | ---: |
| 0 | 47 → 11 | 252 | 447 → 20 | 652 |
| 1 | 23 → 18 | 228 | 329 → 23 | 534 |
| 2 | 35 → 17 | 240 | 392 → 24 | 597 |
| 3 | 33 → 13 | 238 | 352 → 30 | 557 |

None of the required counts is ceiling-limited. Individual family accuracy
also regresses in every order: canonical owner answers fall 1,706 → 868 /
4,096 and object answers 2,029 → 834 / 4,096. Canonical all-question pairs
both correct fall 494 → 31 / 4,096. Location-swap results and complete
per-family comparisons remain in the raw evidence.

### Position, absence and world-level findings

| Rotation | Slot 1 correct / 2,048 | Slot 2 | Slot 3 | Slot 4 | Worst-slot accuracy |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 394 | 402 | 454 | 452 | 19.2383% |
| 1 | 413 | 414 | 425 | 469 | 20.1660% |
| 2 | 418 | 416 | 419 | 452 | 20.3125% |
| 3 | 392 | 444 | 434 | 441 | 19.1406% |

All 16 candidate slot cells are below the 50% floor. Across them, accuracy is
19.1406–22.9004%. Their narrower spread is descriptive: it accompanies broad
loss of correct answers and does not establish useful position-independent
binding.

Canonical supported rows produce **2,647 unknown answers / 8,192 = 32.3120%**,
up from 330 / 8,192 = 4.0283%. Other candidate outputs are 1,702 correct,
1,154 wrong-owner/same-object, 1,103 same-owner/wrong-object and 1,586 unrelated
fact locations. No absent-location or other-vocabulary answer is emitted.
These are supported-only scores: they do not measure correct absence handling.
The candidate's unknown counts by rotation are 2,647 / 2,647 / 2,687 / 2,667.

Canonical question-pair predictions remain unchanged in 3,173 / 4,096 =
77.4658% of contrasts, versus 2,490 / 4,096 = 60.7910% for #1067. The
frozen descriptive focus is `QUESTION_READOUT`; this is not a causal diagnosis.
Owner-changing prediction frequency increases 193 → 356, 176 → 354,
182 → 366 and 218 → 375 / 2,048 across the four orders. Both-correct counts
fall in each order, so this responsiveness is retained as a limited descriptive
effect and does not meet the binding criterion.

The fixed first quartet answers trunk/trunk/unknown/trunk against
cabinet/trunk/trunk/cabinet, yielding only one correct answer.

The candidate has **zero complete four-answer quartets in every rotation**.
Each question family's per-order world bins are therefore
`[1024 - both_correct_pairs, both_correct_pairs, 0]`: the small pair successes
never complete both related pairs from one world. Across four orders all
2,048 candidate worlds have zero complete rotations. The reference has
1,724 / 191 / 128 / 5 / 0 worlds with zero / one / two / three / four complete
rotations. Thus neither model has a world correct in all four orders.
These are correlated views of 2,048 worlds, not independent experimental trials.

### Work, resources, access and replay

The final fit retains exactly 2,352 supported plus 1,568 mixed updates and
2,007,040 presentations, including 1,846,452 supported and 160,588 unknown.
Observed per-offset exposure is:

| Offset | Updates | Total presentations | Supported | Unknown |
| --- | ---: | ---: | ---: | ---: |
| 0 | 992 | 507,904 | 466,944 | 40,960 |
| 1 | 992 | 507,904 | 466,944 | 40,960 |
| 2 | 980 | 501,760 | 462,004 | 39,756 |
| 3 | 956 | 489,472 | 450,560 | 38,912 |

The final partial mixed traversal contributes 4,096 presentations, of which
844 are unknown; counts are measured rather than a nominal 20% assumption.
No additional parameters, random draws, updates or checkpoint choices occurred.

Fit took **292.891308 seconds**, eight-view evaluation
**10.391308 seconds**, and fresh replay
**12.306646 seconds**: **315.589262 seconds
(5.26 minutes)** combined. Peak RSS was
**843,907,072 bytes
(0.785950 GiB)**. Eight Apple
Accelerate CPU threads and one inter-op thread stayed within the original
1,800-second / 4-GiB limits. Admission passed on the first eight real updates
with mean 0.07420137 seconds/step and 422.8447 projected remaining seconds
including the evaluation allowance.

Evaluation process **21178** and replay process **21188** reproduce
the complete evidence exactly. Both candidate and reference learned states are
unchanged before/after scoring. Future attention is zero. Each evaluation
process loads the pinned reference and final candidate once, with zero optimizer
updates and zero checkpoint/optimizer/RNG reads. Fitting loads no prior model.
Historical development payload and native-frame reads are zero.

Fresh development remains **`NOT_RUN_CONSTRUCTION_MISS`**, with **zero model
decisions**; none of the construction views approaches 8,111/8,192 and behavior
also fails. Preparation audited its labels/exclusions, but neither fitting nor
model scoring opened it. R4 is `NOT_RUN_SEPARATE_INFERENCE_STEP`; geometry
changes and generation are zero. The single evaluated candidate is retained.

All 15 focused methods, source review and actual preparation review passed
before launch. Broad QA stayed `NOT_RUN`; queue statuses acknowledge transport
only. Evidence verification does not convert this scientific negative into a
binding success.

| Retained identity | CID |
| --- | --- |
| Fit | `blake3:7ec8c4208338940b5d06fb91a6f54e67f47445097f982714a8980bcf8c372e94` |
| Model file | `blake3:069295e312b576b0a26c45a98e352f73c21dccbd345164863a41a7e45c405d65` |
| Learned state | `blake3:78d3413c7a473a378310484ef3bca7c539a121ef99c38f2daccf79e1afb818a4` |
| Result | `blake3:b5ab27771843d347d9188d4541b46c34bc4ab1d860d956387b8726a100f513be` |
| Complete evidence | `blake3:0bf0667367e24b09f3e1e44e8ae952dcaca1870baa708d1e48e1b3887cb36ff3` |
| Replay | `blake3:08d671be8d4e7db36860493d59c85e0ddec2bfab2259b94c628c6d23436685de` |


### Decision and next recommendation

The frozen action is **`RETAIN_1067_REFERENCE_AND_REVISE_BINDING_LEARNING_RECIPE`**.
The tested cyclic-augmentation recipe is rejected as an improved replacement
at this dose. Preserve its artifact and measured effects, including increased
owner responsiveness without paired correctness. The outcome neither proves
that augmentation can never work nor establishes why this fit failed. No extra
training, dose, alternative checkpoint, or new diagnostic model run follows
within #1071.

The next recommendation is **one separately frozen fact-level learned Q/K/V
binding prototype** on the same four-fact lexical task. Use fixed grammar roles
to form an explicit compound key and query:

```text
q   = Wq [E(query_owner); E(query_object)]
k_i = Wk [E(fact_owner_i); E(fact_object_i)]
v_i = Wv E(fact_location_i)
```

Learn `Wq` and `Wk` independently; share `Wk` and `Wv` across all four facts,
with no fact-slot embedding. Use ordinary softmax over the four fact keys plus
one learned null key/value for missing bindings, then the full 4,096-token
output head. Freeze the output projection, normalization, null construction,
initialization, parameter count and entire optimizer contract before fitting.
Retain a fresh initialization and the same bounded presentation/update dose;
record any architecture/parameter difference explicitly. No exact-key equality
matcher, target-dependent attention mask, correct-location routing or output
candidate restriction belongs in this prototype.

This intervention tests composition on both sides of attention: the owner's
and object's roles in the question, and their joint binding inside each fact.
A shared fact encoder avoids giving each displayed slot its own learned identity.
The current evidence motivates this explicit path; it does not establish the
internal cause of the previous failures. Another increase in geometry or a
continuation of the failed fit would not answer this representation question.

There is a simple **existence argument**, not a trained-result guarantee:
separate owner/object feature subspaces can score a full two-attribute match
above either crossed distractor; a null score between those levels can represent
absence. The future model must learn its weights—do not insert that solution.
Keep all four facts, crossed owner/object distractors, location-swap controls and
missing-binding examples. Verify that the role extractor uses only the causal
input and that value/binding interventions change the answer path. Freeze the
specific controls and resource ceiling with the prototype, not after results.

Keep construction, all four cyclic views, absent-binding behavior and fresh
transfer distinct. Any new transfer run needs freshly frozen worlds excluding
construction and all four prior development populations, including #1071's
unscored set. Qualification permits a separate unchanged-R4 preservation test;
a miss preserves the evidence and rejects this exact prototype/dose. The scope
of a future positive is **learned compound binding through an explicit structured
interface**. Fixed grammar roles are supplied; learned English parsing, general
language understanding and geometric superiority would remain unestablished.
No such prototype or model run occurred in #1071.

Independent evidence and measured-record reviews found no discrepancy. The
historical #1063/#1065/#1067/#1069 records remain intact. #1059/#1061 qualified
attention positives remain unchanged. More geometry stays deferred; general
English, reasoning, chat readiness, H4 superiority, softmax removal and
integer/table lowering remain unestablished. #973 stays open and #954 blocked.
