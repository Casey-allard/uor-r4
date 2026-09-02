# Direct owner-plus-object query encoding at the matched dose — #1069

## Frozen contract (2026-09-02; before fitting)

Issue [#1069](https://github.com/UOR-Foundation/uor-r4/issues/1069) continues
#973 after [#1067's matched readout experiment](r4_zoology_query_readout_1067.md).
The query-object readout improved construction accuracy from 2,396/8,192
(29.2480%) to **3,735/8,192 (45.5933%)**. Object-changing question pairs were
both correct in 447/2,048 cases, while owner-changing pairs reached only
47/2,048. Wrong-owner/same-object choices accounted for 1,439/2,131 (67.5270%)
of q0 in-history errors. Displayed slots 2 and 4 received 74.2686% of in-history
selections despite balanced targets. These identify a remaining behavioral
asymmetry, not its internal cause.

This experiment changes one declared input-encoding operation. At the existing
query-object/readout position 37, use

```text
x37 = E(token37) + P37 + E(token35)
```

The source embedding otherwise returns `E(token_i) + P_i`. Add only the owner
**word** embedding from position 35, with fixed coefficient one; do not add its
position embedding. Apply this operation before the unchanged first-block
embedding dropout and attention trunk. All other positions remain unchanged.
The owner already lies in the causal prefix. This intervention changes direct
access, embedding magnitude and gradient paths together; it does not prove that
owner information was previously absent or isolate an internal mechanism.

The input retains all 41 tokens. Readout remains 37, whose literal next input
token is `?`; the supervised target is a location/unknown label. This is an
explicit joint-query answer interface, not ordinary answer-at-colon generation.

## Matched data, model and learning clock

Copy #1063's construction and vocabulary files byte-for-byte, as in #1067.
All 10,240 input/target/group/variant/type rows and their ordering remain
unchanged. The loader continues to return position 37 instead of the stored
historical position 40. Preserve two attention layers, one head, width 64,
4,096-token tied head, maximum position 120, source normalization/dropout,
and 303,744 parameters. The added connection introduces no parameters,
buffers, modules or random draws. Bind its policy in preparation and model
artifact metadata; the package loader must require that metadata and install
the same operation before inference. Weights alone do not encode this policy.

Use fresh seed-123 initialization, with no prior model/optimizer loading.
Preserve AdamW learning rate `0.00046415888336127773`, weight decay `0.1`,
batch 512, one cosine schedule over 64 blocks of 196 updates, and the original
2,352 supported-phase plus 1,568 mixed-phase updates. The only evaluated model
is the final update-3,920 artifact. Its exact ledger must match #1067:
**2,007,040 presentations = 1,846,452 supported + 160,588 unknown**.
Full forward/dropout tensor shapes and sampler/RNG ordering remain matched.
No intermediate scoring, added updates or checkpoint selection enters fitting.

Before training, freeze 256 new development groups at seed **10692**: 1,024
supported answers and 256 absent-binding answers. Retain the grammar, lexical
map, 64 held-out owner-object pairs, both question types, all five variants,
slot balance and location balance. Exclude every construction world and every
#1063/#1067 development world, including base, location-swapped and absent
variants independently of display order. Also exclude exact input overlaps
and reuse of a world across accepted new groups. Recreate historical
populations with the unchanged frozen generators and verify canonical CIDs;
never open their retained tensor payloads. Prefit preparation audits labels,
exclusions and balance. Fitting/default validation neither opens nor regenerates
development. The new tensor is opened for model scoring only after construction
passes the retained 99% gate.

## Empirical criteria and divergent decisions

The primary bounded improvement concerns owner selection with both answers
correct, not merely a change in the prediction. The fixed construction
thresholds are:

| Criterion | Required candidate count | Retained #1067 |
| --- | ---: | ---: |
| Owner-changing question pairs: both correct | at least 150 / 2,048 | 47 / 2,048 |
| Overall supported answers preserved | at least 3,735 / 8,192 | 3,735 / 8,192 |
| Object-changing question pairs preserved: both correct | at least 447 / 2,048 | 447 / 2,048 |

The owner threshold requires at least 103 additional correct pairs, or
**5.0293 percentage points**. This is a predeclared fixed-population effect
criterion, not statistical significance or full binding. The 2,048 owner pairs
are two related contrasts in each of 1,024 worlds (original and swapped
locations). Report the number of worlds with zero, one and two successful
question pairs; two corresponds to a complete quartet. Do not count these
contrasts as independent experimental units. Report both question types,
location swaps, confound categories, NLL, and per-slot behavior.

1. **All three behavior criteria pass; construction misses:**
   `JOINT_QUERY_PARTIAL_GAIN`. Retain the joint-query artifact as the improved
   construction baseline and use its recorded remaining-error/position profile
   to choose the next separate repair. Fresh development stays unscored.
2. **Either preservation floor misses:** `JOINT_QUERY_PRESERVATION_MISS`.
   Report gains and regressions separately. Reject this residual as the next
   improved baseline; retain #1067 and revise the binding learning recipe.
3. **Preservation passes, owner threshold misses:**
   `JOINT_QUERY_BELOW_DECLARED_OWNER_GAIN`. Preserve any smaller measured gain,
   but retain #1067 as the baseline and revise the binding learning recipe.
4. **Construction reaches at least 8,111/8,192:** score fresh development once
   on the sole final artifact. Full binding requires at least 973/1,024
   supported answers, 231/256 complete groups, 116/128 complete groups for each
   question type, and 244/256 unknown answers. Return
   `JOINT_QUERY_FRESH_TRANSFER_MISS` or `JOINT_QUERY_FRESH_BINDING_PASSED`.
   A miss preserves construction only; a full pass permits a separate unchanged
   R4 inference-preservation step. No R4 execution belongs to this issue.
5. **Interrupted fit:** retain the exact partial checkpoint/work and actual
   numerical/admission/resource limit. An incomplete fit is not a scientific
   negative and never receives a restarted budget.

Construction pass with behavior failure is an integrity inconsistency:
8,111 correct permits only 81 wrong rows, hence at least 2,048 − 81 = 1,967
both-correct pairs in each question type. No legitimate full-construction pass
can fail the lower owner or preservation floors.

The three behavior criteria are separate from full binding. In particular,
`behavior.passed=true` with a construction miss still has overall
`passed=false`. A partial gain never opens fresh development.

## Resources and focused verification

Retain the combined 1,800-second fit/evaluation/replay and 4-GiB RSS ceilings,
eight CPU Apple Accelerate threads, one inter-op thread, 1.25 admission
projection multiplier and 60-second evaluation allowance. The first eight
real updates count toward the 3,920 dose and supply admission. #1067 completed
in 290.31 seconds at 0.7764 GiB peak. One additional owner lookup/vector addition
retains the attention tensor shapes. All 8,192 supported decisions have the
owner, object and answer facts available by position 37; no reachability
restriction excludes most of the measured population.

Named checks cover adapter locality, causal source, gradients, unchanged
state/parameters/RNG and artifact reconstruction; full-trainer AST equivalence
apart from the declared adapter/metadata/identities; copied data and fresh
population semantics/exclusions/access; owner/preservation/development decisions;
world-level dependent-pair accounting; final artifact/state binding; and exact
fresh-process complete-evidence replay. Independent source/preparation/evidence
review is required. Broad workspace, BDD, WASM, fuzz, audit and release QA
remain dormant; protected queue statuses acknowledge transport only.

## Commands

From `tools/r4-softmax-trainer`, in the locked offline environment:

```bash
uv sync --frozen --offline
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query
.venv/bin/python -m r4_softmax_trainer.zoology_joint_query verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1069-zoology-joint-query
```

Preparation/result/replay files are immutable. Resume can only continue the
same bound checkpoint within the original cumulative clock. Evaluation/replay
perform no training and use the explicitly adapted artifact loader. Preserve
#1063/#1065/#1067 evidence and #1059/#1061 attention positives. More geometry,
generation, general English, reasoning, chat readiness, H4 superiority, softmax
removal and integer/table lowering remain deferred or unestablished. #973 stays
open and #954 blocked.


## Published preparation

Source and contract freeze: `b3d8dc60`. All **12 focused methods passed**:
three adapter/state/reconstruction checks, one whole-trainer AST comparison,
three data/exclusion/access checks, and five decision/interface/world-accounting
checks. Independent source review found no blocking issue. These checks used
synthetic inputs; no fitted artifact or historical development payload was read.

The [preparation envelope](r4_zoology_joint_query_1069_preparation.json) binds
**238 implementation files**, the original construction/vocabulary bytes,
the complete retained baseline and work ledger, and the declared owner residual.
Actual preparation audited all 10,240 construction and 1,280 new development
labels. Fresh development has 768 distinct base/swapped/absent worlds, zero
world/input overlap with construction or either historical development
population, all 64 held-out owner-object pairs, and balanced target locations
and relevant slots. Each historical development population has 1,280 rows and
768 worlds; the per-history audit records distinguish #1063 and #1067.
The inherited top-level `historical_development_rows` and
`historical_development_canonical_worlds` fields refer to the original #1063
population, not the union; use the per-history records for the two exclusions.

- Preparation: `blake3:e2709d32436f7979aeda795f2ec735d99932cdd1037e7e17321d72a7009ad7e1`
- Implementation: `blake3:e5436a8c0d8e2e92312721f54164b05269c5a6138a52f54e4574bb3fa7de1e2d`
- Dataset manifest: `blake3:3597f258750242bd1b9482e234b4ed939375d4a32c08c37a76d1a500cfe9e490`
- Dataset tree: `blake3:f85ed814f31be179fbe85d6bc08caedb323c742adba0b0a1660ffff330714119`
- Fresh development: `blake3:cbee180e7e37fe302d9cffd198d7af91ed42ed9851bbbf91e7ee4814d0e0a9b1`
- Unchanged construction: `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`

The frozen behavior counts are owner at least 150/2,048, overall at least
3,735/8,192, and object at least 447/2,048. Construction must separately reach
8,111/8,192 before any fresh development model decision. These identities and
criteria are published before the first optimizer update.


## Measured result (2026-09-02; after the fixed fit)

**Terminal: `JOINT_QUERY_PRESERVATION_MISS`.** The final artifact substantially
improves owner-changing answers and aggregate construction accuracy, but it
misses the required object-pair preservation floor. Keep the gain and the
regression as measured evidence; #1067 remains the accepted reference under
this frozen decision.

The single admitted fresh fit completed all **3,920 updates** and the exact
2,007,040-presentation ledger. Construction reached **4,118/8,192 = 50.2686%**,
up 383 correct answers / **4.6753 percentage points** from #1067. NLL fell from
1.3790687658 to **1.2404528148 nats** (delta −0.1386159509).

Raw evidence: [fit/work ledger](r4_zoology_joint_query_1069_fit.json),
[result and complete diagnostics](r4_zoology_joint_query_1069_result.json), and
[fresh-process replay](r4_zoology_joint_query_1069_replay.json).

### Primary decision and retained tradeoff

| Construction measurement | #1067 object readout | #1069 owner-plus-object | Frozen verdict |
| --- | ---: | ---: | --- |
| Correct supported answers / 8,192 | 3,735 (45.5933%) | 4,118 (50.2686%) | Overall preservation passes |
| Owner-changing question pairs: both correct / 2,048 | 47 (2.2949%) | 338 (16.5039%) | At least 150 required; passes |
| Object-changing question pairs: both correct / 2,048 | 447 (21.8262%) | 376 (18.3594%) | At least 447 required; misses |
| All question pairs: both correct / 4,096 | 494 (12.0605%) | 714 (17.4316%) | Descriptive gain |
| Fixed-question location-swap pairs: both correct / 4,096 | 1,216 (29.6875%) | 1,279 (31.2256%) | Descriptive gain |

Owner pairs gain **291 / +14.2090 percentage points**, exceeding the required
103-pair gain. Object pairs lose **71 / −3.4668 percentage points**. Thus
`owner_pair_gain=true`, `overall_preserved=true`, but
`object_pairs_preserved=false`, and both `behavior.passed` and full `passed`
are false. The object criterion concerns two related answers both being
correct. Individual accuracy in that question family actually rises from
2,029 to 2,083 of 4,096; individual owner-family accuracy rises from 1,706 to
2,035. The pair loss must not be mislabeled as lower individual object-family
accuracy or hidden by the aggregate improvement.

Changing the owner changes the prediction in 1,073/2,048 pairs (52.3926%),
compared with 193; changing the object does so in 1,139/2,048 (55.6152%),
compared with 1,413. The mean fixed-target question-logit contrasts are
2.2821559642 for owner changes and 2.5126506260 for object changes, versus
0.0283599555 and 1.8402354482 previously. These descriptive contrasts neither
establish correct binding nor identify an internal cause. Mean question and
location-swap contrasts remain algebraically linked by quartet construction.

The dependent-world accounting is:

| Question change | Worlds with 0 successful pairs | With 1 | With 2 / complete quartet |
| --- | ---: | ---: | ---: |
| Object | 743 | 186 | 95 |
| Owner | 774 | 162 | 88 |

Each row contains 1,024 worlds and two related question pairs per world;
186 + 2×95 = 376 and 162 + 2×88 = 338. No independence or significance claim
is made from the 2,048 pair denominator.

### The remaining position and confound profile

**Accuracy is highly dependent on fact position.** Targets are balanced at
2,048 per displayed slot:

| Target fact slot (human numbering) | #1067 correct / 2,048 | #1069 correct / 2,048 | #1069 accuracy |
| --- | ---: | ---: | ---: |
| 1 | 413 | 735 | 35.8887% |
| 2 | 1,486 | 738 | 36.0352% |
| 3 | 485 | 604 | 29.4922% |
| 4 | 1,351 | 2,041 | 99.6582% |

The fourth slot gains 690 correct answers while the other slots collectively
lose 307. Their weighted contributions are +8.4229 and −3.7476 percentage
points, respectively, yielding the +4.6753-point total. Accuracy over the first
three slots falls from 2,384/6,144 = 38.8021% to 2,077/6,144 = 33.8053%.
This is an arithmetic decomposition, not an internal causal explanation.
The fourth-slot result is a useful restricted positive; it is not general
retrieval across positions.

In-history output selections by slot are 1,800, 1,827, 1,495 and 2,791.
The largest share is only 35.2711%, so the frozen single-slot majority flag
is false despite the extreme target-conditioned accuracy disparity. The
frozen descriptive label is `DISTRIBUTED_BINDING`; this label likewise does
not establish position independence or the absence of confounds.

Among 4,096 q0 rows with equal confound availability, 2,096 errors include
1,946 in-history errors: wrong-owner/same-object 798 (41.0072%),
same-owner/wrong-object 742 (38.1295%), and unrelated 406 (20.8633%).
Another 150 q0 errors answer unknown. Across all supported rows the categories
are target 4,118, same-object confound 1,284, same-owner confound 1,187,
unrelated 1,324, and unknown 279. In-history selections are
7,913/8,192 = 96.5942%; no absent-location or other-vocabulary outputs occur.

The fixed first-quartet example answers both “Liam's key” and “Liam's toy”
with the fourth fact's **trunk**, then answers both with **cabinet** after
swapping Liam's key/toy locations. Only the toy answers are correct. The prior
#1067 model also got the toy answers correct, while answering both key queries
with locker. This fixed example illustrates why following a location change
alone does not establish joint query binding.

### Resources, access and replay

Fit time was **277.167708 seconds**, evaluation
**1.800316 seconds**, and replay **7.923750 seconds**:
**286.891776 seconds (4.78 minutes)** combined. Peak RSS
was **843,513,856 bytes (0.785583 GiB)**; fit peak was
516,587,520 bytes. Eight Apple Accelerate CPU threads, one inter-op
thread, Python 3.12.14 and PyTorch 2.7.1 match the predecessor configuration.
The first eight updates passed the same admission policy; their mean step was
0.081161 seconds, projected remaining total 456.88 seconds, and observed peak
493,797,376 bytes. The full run stayed within the 1,800-second/4-GiB limits.

The exact ledger remains 2,352 supported-phase and 1,568 mixed-phase updates,
1,846,452 supported and 160,588 unknown presentations. No extra dose or
alternative checkpoint was evaluated. The final weights retain the original
state layout, with the owner-residual execution policy explicitly bound in
artifact metadata and applied by the checked loader.

Evaluation process **13611** and replay process **13869** reproduce the
complete evidence exactly (`exact_replay=true`, `fresh_process=true`). The
learned state is identical before and after evaluation, future attention is
zero, and evaluation/replay perform zero optimizer updates. This is replay
of the final artifact, not a second training run.

Construction remains below 8,111/8,192. Fresh development therefore has **zero
model decisions**, `NOT_RUN_CONSTRUCTION_MISS`. Its labels/exclusions were
audited before fitting; it was not opened for fitting or model scoring.
Historical development payload, prior model/checkpoint, evaluation checkpoint/
optimizer/RNG, and native-frame reads are zero. Geometry changes are zero.
Unknown targets were used in training, but this supported-only construction
score does not measure absent-binding accuracy. R4 remains
`NOT_RUN_SEPARATE_INFERENCE_STEP`; generation was not run.

All 12 named focused methods passed before source freeze. Independent source
and preparation reviews passed. Broad workspace, BDD, WASM, fuzz, audit and
release QA remain `NOT_RUN`; required queue statuses are transport only.

| Retained identity | CID |
| --- | --- |
| Fit | `blake3:ff7dd732a914ea7377095d25cfac8a57f039c4c5185550cf4f6bf9c38654825b` |
| Model file | `blake3:e68fe47e415229f0b1f48cdc8d9b82fe577b5e422a720524429a2906fa0fdd2f` |
| Learned state | `blake3:0822f7e9ac710030e74d933547d67a2af3f3eed099f5f1a77775893bfabb3897` |
| Result | `blake3:bc1066eb0e9bbf08304ab296ca0c1681b7e8af4b0ea9026945ebef83c7fb9d53` |
| Complete evidence | `blake3:72004d0146c51ed30dcf093073d1c381a36a329f78a7b622068fff4e59ccd3f7` |
| Replay | `blake3:6a6ad3ce5ef9e9541fec4994006b39d7a15ce52432a1af3cff351f0b9d96fcf2` |


### Decision and next recommendation

The frozen action is **`RETAIN_1067_BASELINE_AND_REVISE_BINDING_LEARNING_RECIPE`**.
The owner residual supplies useful owner-selection and aggregate gains in this
single matched setting, while failing object-pair preservation and leaving a
large position dependence. Retain the #1069 artifact and all of those findings;
do not adopt it as an improved replacement under the declared conditions.
The 99.6582% fourth-slot result is a restricted construction positive. Neither
it nor the overall 50.2686% establishes fresh transfer or complete binding.

The next recommendation is **one separately frozen fit with deterministic
cyclic fact-order augmentation**, using the plain #1067 query-object readout
and no owner residual. Keep all four facts, all 41 tokens, query/readout
position 37, original labels, model size, fresh seed, optimizer and the
3,920-update budget. Rotate the four intact eight-token fact blocks through
four offsets across successive full training traversals; leave BOS and the
question suffix unchanged. Use the same offset for all variants of a world
within a traversal. Derive the schedule from the existing phase/update count,
without extra random draws, and record actual exposure counts, including the
partial final traversal.

The facts have distinct owner-object keys, so these cyclic rotations preserve
answer and absence semantics. The original population balances target slots
across different worlds, but each particular world's display order was fixed.
This proposed recipe teaches the same binding at different positions while
retaining the actual four-fact problem and its valid absence controls. It is
more directly motivated by the measured position profile than an easier
replacement task. It does not establish that augmentation will solve binding.

Freeze the exact effect, preservation and worst-slot criteria before that
separate run. Compare the final candidate and retained #1067 reference on
canonical construction and all four cyclic rotations. Require improvement in
both owner- and object-changing **both-correct** behavior together with an
explicit worst-slot requirement; answer-change frequency or average accuracy
alone is insufficient. The rotation-zero case is canonical construction, not
an additional independent population. These four rotations do not cover all
24 possible fact orders. Full-binding and fresh-development gates remain
separate, and any new transfer claim requires newly frozen unscored worlds.
No augmentation fit or additional diagnostic model run occurred in #1069.

Independent evidence review found no discrepancy in the envelopes, adapted
artifact, work ledger, replay, access counts or measured arithmetic. The
historical #1063/#1065/#1067 records remain unchanged. #1059 retains
11,900/12,000 = 99.1667% preservation. #1061 retains ordinary and coherent R4
both at 8,071/8,192 = 98.5229%, with all 8,192 predictions identical and an
86.2061-percentage-point loss under transport control. More geometry remains
deferred. General English, reasoning, chat readiness, H4 superiority, softmax
removal and integer/table lowering remain unestablished. #973 stays open and
#954 blocked.
