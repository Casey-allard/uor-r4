# Query-object answer readout with the matched English dose — #1067

## Frozen contract (2026-09-02; before fitting)

Issue [#1067](https://github.com/UOR-Foundation/uor-r4/issues/1067) continues
#973 after the [#1065 construction diagnostic](r4_zoology_english_diagnostic_1065.md).
The retained English model reproduced `2,396/8,192 = 29.2480%` construction
accuracy but kept the same answer in `3,974/4,096` question contrasts. Only
`20/4,096` question pairs were both correct: `6/2,048` same-owner pairs and
`14/2,048` same-object pairs. That established a behavioral question/readout
failure profile, not an internal cause.

This experiment asks whether changing the supervised readout placement alone
is sufficient at the same fixed learning dose. The candidate gathers the
answer hidden state at the queried object, position **37**, instead of the
constant colon, position **40**. The owner is at position 35, before possessive
36 and object 37. The literal next input token after the object is `?`; the
supervised target is an explicit location/unknown answer label. A positive
result would qualify this answer-readout interface, not ordinary answer-at-colon
generation or general next-token language modeling.

The full 41-token input and entire model forward computation remain intact.
Positions 38–40 are still materialized but cannot affect the selected causal
position 37. This keeps dropout tensor shapes and the training RNG sequence
matched. The intervention changes the selected position, its direct lexical
input and its available causal prefix together; it does not isolate colon
identity alone.

## Exact matched construction and learning clock

The construction and vocabulary containers are copied byte-for-byte from
#1063. All 10,240 inputs, targets, group IDs, variant IDs and question types
remain identical. Only the loader's returned positions tensor changes from
all 40 to all 37. No original artifact is rewritten.

The fresh seed-123 model retains the released Zoology attention cell: two
layers, one head, width 64, 4,096-token tied vocabulary head, maximum position
120, source normalization/dropout/residual settings and 303,744 parameters.
It loads no earlier model or optimizer checkpoint. The prior trainer is copied
with only descriptive/schema/issue identities changed; a whole-module AST
comparison verifies that the numerical, RNG, sampler, scheduling, admission,
resume and export code is unchanged.

The fixed dose remains batch 512, AdamW learning rate
`0.00046415888336127773`, weight decay `0.1`, one cosine schedule with 64
blocks, each of 196 updates. The first 2,352 updates sample supported
construction; the following 1,568 sample all five variants. Exactly 3,920
updates yield 2,007,040 answer presentations. The final work ledger must match
the retained sampler trajectory: 1,846,452 supported and 160,588 unknown
presentations. No evaluation or additional iterator/model construction is
inserted into fitting. The first eight real updates count toward the dose and
supply the unchanged admission observation.

Every supported answer fact is available before position 37. All relevant
query words are available by that position. Thus all 8,192 supported decisions
can respond to the intervention; there is no route-reachability ceiling that
excludes most of the measured population.

## New development population and access

Before fitting, freeze 256 new groups at seed `10672`: 1,024 supported rows and
256 absent-binding rows. Keep the same lexical map, grammar, held-out
owner-object partition, two question types, five counterfactual variants,
fact-slot balance and answer-location balance. Every new base, location-swapped
and absent-binding world must be disjoint from every historical development
world, every construction world and other accepted new groups. World identity
uses sorted owner/object/location facts, independently of their display order.
Exact input overlap is also checked.

Historical development is regenerated using the unchanged #1063 generator and
seed; its canonical bytes must reproduce the published original CID. Its
retained payload is never opened. Rejection during new-world generation keeps
the preassigned target slots and locations fixed, preserving balance. Build
time independently audits labels, missing bindings, all held-out pairs,
counterfactuals and exclusions without running a model.

Default preparation validation and training loaders read only the manifest,
vocabulary and copied construction payload. They neither load nor regenerate
development. The new development payload is opened for model scoring only if
the final construction score reaches the frozen fit threshold. It measures
fresh combinations within this familiar controlled task, not transfer to an
unseen task or unrestricted English.

## Empirical criteria and divergent actions

The fixed final update-3,920 artifact is the sole evaluated checkpoint. Report
construction accuracy/NLL, all #1065 error categories and matched question/
location-swap responses, including both question types and changes against the
retained baseline. A partial gain remains evidence even when the fit threshold
is missed.

| Criterion | Frozen count |
| --- | ---: |
| Supported construction accuracy at least 99% | 8,111 / 8,192 |
| New supported development accuracy at least 95% | 973 / 1,024 |
| All four answers correct in at least 90% of new groups | 231 / 256 |
| Complete same-owner groups at least 90% | 116 / 128 |
| Complete same-object groups at least 90% | 116 / 128 |
| Correct new missing-binding answers at least 95% | 244 / 256 |

1. **Construction miss:** return `QUERY_OBJECT_READOUT_CONSTRUCTION_MISS`,
   preserve the final model and any partial gains, and leave new development
   unscored. Readout placement is insufficient under this fixed dose. Do not
   add updates or retry; the next design must address joint query binding.
2. **Construction fits, development misses:** return
   `QUERY_OBJECT_READOUT_FRESH_TRANSFER_MISS`. Preserve construction learning
   and address transfer to the new combinations. Report supported, quartet,
   question-type and unknown results separately.
3. **All ordinary criteria pass:** return
   `QUERY_OBJECT_READOUT_FRESH_BINDING_PASSED`; the next separate step is
   unchanged-adapter R4 inference preservation on this artifact.
4. **Admission, numerical or resource interruption:** retain the exact partial
   checkpoint/trajectory and report its actual limit. Never represent an
   incomplete fit as a scientific negative or restart its admission clock.

This experiment performs no R4 adapter or native-frame execution. More
geometry, generation, broader language claims, reasoning, chat readiness,
softmax removal and integer/table lowering do not follow automatically.
The earlier #1059/#1061 attention positives and #1063/#1065 evidence remain
intact; #973 stays open and #954 blocked.

## Resources and named verification

Use the original 1,800-second combined fit/evaluation/replay and 4-GiB RSS cap,
eight CPU Apple Accelerate threads, one inter-op thread, 1.25 admission
projection factor, and 60-second evaluation allowance. #1063 completed fit,
evaluation and replay in 249.37 seconds at 0.486 GiB; retaining the full trunk
retains the work shape. This is one minutes-scale fit, with no hardware sweep,
new seed, extra dose or learning-rate search.

Named checks: copied-data/readout identity and stage-specific data access;
fresh-development semantic/exclusion/balance checks; whole-trainer clock
equivalence; construction-failure suppression of development scoring; both
question-type decision thresholds; preservation of partial gains; final
example serialization; immutable model/source bindings; and exact complete
evidence replay in a different process. Independent review covers source,
evidence and current claims. Broad QA remains dormant. Required queue statuses
are transport acknowledgements, not tests or scientific PASS evidence.

## Commands

From `tools/r4-softmax-trainer`, use the locked offline environment:

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
.venv/bin/python -m r4_softmax_trainer.zoology_query_readout verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1067-zoology-query-readout
```

Preparation, results and replay are immutable. A fit may resume only the same
bound checkpoint within the original cumulative budget. Verification performs
no training and compares full final evidence, including prediction/logit/
attention identities, construction diagnostics and any admitted development.

## Published preparation

Executable and contract freeze: `24a8f3cd`. Eight focused checks passed (three
data, one whole-trainer equivalence, four campaign/serialization checks).
Independent source review found no remaining blocking issue. No fitted model
was initialized or executed by those checks or review.

The [preparation envelope](r4_zoology_query_readout_1067_preparation.json) binds
223 implementation files, including the unchanged predecessor closure. The
actual preparation audit verified all 10,240 original construction labels,
all 1,280 fresh development labels, 768 fresh variant worlds, zero historical
or construction world/input overlap, all 64 held-out owner-object pairs,
balanced supported target locations and relevant fact slots. The copied
construction and vocabulary CIDs remain exactly those of #1063.

- Preparation: `blake3:b6df21b50fe67696910e8e01ae2aa590c9e9b6aebf15b090cc11b98edc5a82d3`
- Implementation: `blake3:9d368e194b2fad64aebadaf87ca71b4927134b13c33bb949c0e7cc0ca6e81b48`
- Dataset manifest: `blake3:da7be1fba1bb02d3891f4c08840cdd368828b28a2e0a19119f08ca8aa44a5f28`
- Dataset tree: `blake3:0ce48d3f9266377013543af1eae00fe445b40b90ca29ed73085e732c08725bd5`
- Fresh development: `blake3:5f20f7f5f9e079e73886bd885cb88dc0948effc73a58d2a111f387ae5604b780`
- Unchanged construction: `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`

These bindings and the decision contract were published before the first
training update. The fresh development population had zero model decisions
at publication.


## Measured result (2026-09-02; after the frozen fit)

**Terminal: `QUERY_OBJECT_READOUT_CONSTRUCTION_MISS`, with a substantial partial
improvement.** The single admitted fit completed all 3,920 updates. Its final
artifact scored **3,735/8,192 = 45.5933%**, compared with the retained
2,396/8,192 = 29.2480%: **1,339 additional correct answers, +16.3452 percentage
points**. Construction NLL fell from 1.6081310920 to 1.3790687658 nats
(delta −0.2290623263). The frozen 8,111-correct construction criterion was
missed. No additional updates or alternative checkpoints were evaluated.

Raw evidence: [fit and complete work ledger](r4_zoology_query_readout_1067_fit.json),
[result and all diagnostics](r4_zoology_query_readout_1067_result.json), and
[independent-process replay](r4_zoology_query_readout_1067_replay.json).

### What changed behaviorally

Each question pair holds all four displayed facts fixed and changes one query
attribute. Same-owner pairs change the object; same-object pairs change the
owner. A changed answer alone is not a correct binding, so both-correct counts
are reported beside responsiveness.

| Construction measurement | Colon readout, #1063/#1065 | Query-object readout, #1067 |
| --- | ---: | ---: |
| Supported answers correct / 8,192 | 2,396 (29.2480%) | 3,735 (45.5933%) |
| Object-changing question pairs: changed / 2,048 | 33 (1.6113%) | 1,413 (68.9941%) |
| Object-changing question pairs: both correct / 2,048 | 6 (0.2930%) | 447 (21.8262%) |
| Owner-changing question pairs: changed / 2,048 | 89 (4.3457%) | 193 (9.4238%) |
| Owner-changing question pairs: both correct / 2,048 | 14 (0.6836%) | 47 (2.2949%) |
| All question pairs: both correct / 4,096 | 20 (0.4883%) | 494 (12.0605%) |
| All question pairs: invariant / 4,096 | 3,974 (97.0215%) | 2,490 (60.7910%) |
| Fixed-question location-swap pairs: both correct / 4,096 | 247 (6.0303%) | 1,216 (29.6875%) |

Correct individual answers increased in both question types: same-owner from
1,213 to 2,029 of 4,096; same-object from 1,183 to 1,706 of 4,096. Nevertheless,
owner changes still leave 1,855/2,048 = 90.5762% of predictions unchanged.
The mean fixed-target question-logit contrast is 1.8402354482 for object
changes and 0.0283599555 for owner changes. These are descriptive logits,
not calibrated capability scores. The quartet design makes the corresponding
mean question and location-swap contrasts algebraically linked; they are not
independent corroboration.

The frozen descriptive focus is now **`OWNER_DISAMBIGUATION`**. Among the
4,096 q0 rows, where both attribute confounds are equally available, there
are 2,306 errors, including 2,131 in-history errors. Of those in-history errors,
1,439 (67.5270%) choose the same object with the wrong owner, 386 choose the
same owner with the wrong object, and 306 choose the unrelated fact. The other
175 errors answer unknown. This asymmetry supports the next design choice;
it does not identify an internal causal failure.

Across all 8,192 supported answers, output categories are target 3,735,
same-owner confound 733, same-object confound 2,285, unrelated fact 1,109,
and unknown 330. No answer selects an absent location or another vocabulary
ID. Thus 7,862/8,192 = 95.9717% select a location actually in the history.

**Position bias remains material.** The four displayed slots receive 960,
3,158, 1,063 and 2,681 in-history selections, respectively, despite exactly
2,048 target exposures per slot. No single slot exceeds the diagnostic's
50% flag, but human-numbered slots **2 and 4 together account for
5,839/7,862 = 74.2686%** of in-history outputs. The false single-slot flag
therefore does not establish position independence. Correct answers by target
slot are 413, 1,486, 485 and 1,351 of 2,048 each.

A fixed first-quartet example makes the partial gain concrete. The history is
“leon put the key in the basket. liam put the key in the cabinet. mila put the
coin in the locker. liam put the toy in the trunk.” The new model answers
Liam's key with **locker** (wrong) and Liam's toy with **trunk** (correct).
After swapping the locations of Liam's key and toy, it still answers the key
with locker, but correctly changes the toy answer to **cabinet**. The retained
colon-readout baseline answered basket in all four cases. This is one fixed
illustration of the aggregate result, not a selected capability demonstration.

### Work, access and exact replay

The fit used 2,352 supported-phase and 1,568 mixed-phase updates at batch 512:
2,007,040 presentations, comprising exactly **1,846,452 supported and 160,588
unknown** presentations. This matches #1063's complete work ledger. Full
41-token forward tensors, seed, sampler/dropout random sequence and source
training implementation remained matched. The same eight-thread Apple
Accelerate CPU configuration used Python 3.12.14 and PyTorch 2.7.1.

The first eight updates passed admission: mean step 0.081619 seconds,
projected remaining total 459.12 seconds, and peak RSS 490,897,408 bytes.
Actual fit time was 286.5381 seconds; final evaluation 1.8947 seconds and replay
1.8782 seconds. The combined **290.3110 seconds (4.84 minutes)** and peak
**833,601,536 bytes (0.7764 GiB)** stayed within the 1,800-second/4-GiB caps.
Fit peak RSS was 519,667,712 bytes (0.4840 GiB).

The evaluation process was 11644 and the replay process 11650. Replay reports
`exact_replay: true`, `fresh_process: true` and zero optimizer updates. It
reproduces the entire evidence envelope, including full-head selected logits,
predictions, attention, construction diagnostics, comparison, access counts
and decision. Learned state is identical before and after evaluation, and
future-position attention is zero. This is evaluation replay of the retained
final artifact, not an independently repeated training run.

Fresh development is **`NOT_RUN_CONSTRUCTION_MISS`, zero model decisions**.
Its generated labels and exclusions were audited during prefit preparation;
the tensor was not opened for fitting or model scoring. Historical development
payload reads, prior-model/checkpoint reads, evaluation optimizer/checkpoint/RNG
reads, native-frame reads and geometry changes are zero. Training included
unknown targets, but this supported-only construction evaluation does not
establish absent-binding accuracy. Fresh transfer remains unmeasured.
R4 is **`NOT_RUN_SEPARATE_INFERENCE_STEP`**. Generation was not run.

The eight named focused checks passed before source freeze. Independent review
of source and preparation preceded fitting; a separate evidence review found
no discrepancy in the measured envelopes, work ledger, replay, access counts
or claim boundaries. Broad workspace tests, BDD,
WASM, fuzz, audit and release QA remain `NOT_RUN`; required queue statuses
acknowledge transport only.

| Retained identity | CID |
| --- | --- |
| Fit | `blake3:ec2dea07ef3b2eaf3d6532830c0434c33935f72df982b16929ecc6fc48be08e8` |
| Model file | `blake3:9386849d191b038803ae30b267f2fbf654cb077f48d5f3ddc92137c60875dd98` |
| Learned state | `blake3:feeabb398a0ca2f799bcd4274c607dc8527f39619ac3cca39be8727a27f1e005` |
| Result | `blake3:c6dfcb3a856963ab4493c3d26bf729f6d9cad70147316ef2b9b62e87c3116369` |
| Complete evidence | `blake3:e1ae84b84d6ebedaf6a49afd43378c0975ec6fe1597ee246b00cc60e2191fb79` |
| Replay | `blake3:98c799c7844e36d68b56c6948824c1dacb53fb36e9f38bc7c052ec6fe0873fac` |

### Decision and next recommendation

The predeclared action is **`RETAIN_GAINS_AND_REDESIGN_JOINT_QUERY_BINDING`**.
Placement alone is insufficient at the matched dose, but the 16.3452-point
accuracy gain and improved object-dependent selection are retained evidence.
This result is specific to one matched seed, learning dose, construction
population and explicit answer-readout interface. It does not establish a
seed-robust improvement, fresh-combination transfer, or general English.

The next recommendation is **one separately frozen joint-query encoding fit**:
keep supervised position 37 and add the owner token embedding from position 35
to the queried-object embedding at position 37, before the unchanged source
embedding dropout and attention trunk. In symbols, use
`x37 = E(token37) + P37 + E(token35)`; other positions are unchanged. This gives
both query attributes a direct path while retaining the object component.
The owner was already in the causal prefix, so this proposal changes its
accessibility rather than repairing a demonstrated absence.

Use fresh initialization, the same four-fact construction rows/labels, seed,
optimizer, full tensor shapes, 3,920-update dose and resource caps. The proposed
fixed residual introduces no parameters, input labels or additional random
draws. Compare with this retained #1067 model and report both question types,
correct matched changes and displayed-position behavior separately. Freeze the
exact source, decision criteria and a new unscored development population
before that separate fit. Owner-changing both-correct improvement is the
proposed primary decision, with preservation of overall accuracy and
object-changing both-correct behavior required alongside it; freeze numerical
effect thresholds in that next contract. The next issue must decide whether
joint access is sufficient at the fixed dose before a broader curriculum or
model change. This is an
explicit lexical-query interface, not unrestricted next-token generation.
No further fit was performed in #1067.

The earlier #1063 held-out negative and #1065 diagnostic remain unchanged.
The #1059 11,900/12,000 = 99.1667% preservation and #1061 identical ordinary/
coherent-R4 8,071/8,192 = 98.5229% result remain intact, including the latter's
86.2061-percentage-point loss under transport control. More geometry remains
deferred. #973 stays open and #954 stays blocked; general English, reasoning,
chat readiness, H4 superiority and softmax removal remain unestablished.
