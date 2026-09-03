# Learned compound fact binding — #1073

## Frozen contract (2026-09-02; before fitting or model scoring)

Issue [#1073](https://github.com/UOR-Foundation/uor-r4/issues/1073) continues
#973 after [#1071](r4_zoology_cyclic_facts_1071.md). Cyclic training failed
preservation: supported accuracy was 20.7764–21.0083% across four orders,
versus the retained #1067 reference's 40.8569–45.5933%. This experiment changes
the binding representation in one fresh fit at the same update dose. It gives
ordinary learned attention explicit owner/object/location roles from the fixed
grammar. It does not learn an English parser or expand geometry.

## Model definition and information boundary

**Definition.** With embedding width 64 and four facts, the model is:

```text
q   = Wq LN0_128(concat(E(query_owner), E(query_object)))
k_i = Wk LN0_128(concat(E(fact_owner_i), E(fact_object_i)))
v_i = Wv LN0_64(E(fact_location_i))
a   = softmax(q [k_1,k_2,k_3,k_4,null_key]^T / 8)
h   = affine_LN64(Wout(a [v_1,v_2,v_3,v_4,null_value]))
logits = h E^T
```

Q and K are independently learned; all four projection matrices are bias-free.
The same K/V encoders serve every fact. LN0 is affine-free LayerNorm, epsilon
1e-5; the output LayerNorm has learned gain/bias. One shared 4,096-by-64 lexical
embedding also supplies the full vocabulary head. Every row includes learned
null key/value vectors. There are no fact-position embeddings, dropout, direct
query residual, vocabulary-head bias, equality matcher, oracle mask, absence flag,
answer-dependent routing or candidate-vocabulary restriction.

Exact parameter count is **286,976**: embedding/head 262,144; Q/K 16,384;
V/output 8,192; output LayerNorm 128; null key/value 128. This is 16,768 fewer
parameters than #1067. Seed 123 initializes E, Wq, Wk, Wv, Wout, null key and
null value, in that order, with seven independent normal tensor draws at
standard deviation 0.02. Constructors consume no implicit random draws.
Output LayerNorm gain starts at one and bias at zero.

The fixed grammar extracts owner positions 1/9/17/25, object positions
4/12/20/28, location positions 7/15/23/31, and query owner/object at 35/37.
The supervised readout remains 37 in a 41-token input. Filler and future
positions 38–40 are never embedded. Targets enter only training loss or
post-forward scoring. Output is one full 4,096-token distribution and the
actual rectangular attention tensor `[batch,1,1,5]`. Answer supervision at
the query object is not ordinary free generation; its literal next token is `?`.

## Fixed dose, reference and reachability

Fit only the original #1063 canonical construction bytes. No augmentation or
previous learned state. Preserve the #1067 trainer's AdamW learning rate
0.00046415888336127773, weight decay 0.1, batch 512, original 64-block cosine
clock with 196 updates per block, sampler, admission and checkpoint lifecycle.
The sole candidate is update **3,920 = 2,352 supported + 1,568 mixed**,
or **2,007,040 presentations**. The new architecture changes RNG consumption;
the exact #1067 sampling trajectory and supported/unknown counts are not
asserted. Report the actual ledger. The mixed phase comprises 78 full
10,240-row traversals and eight batches; each full traversal has 2,048 unknown
rows and the partial tail can contain 0–2,048 unknown rows.

Retain the recorded #1067 four-order scores in #1071's published result.
Their canonical score and diagnostic already reproduced #1067 exactly.
Bind that evidence and the 254-file #1071 source snapshot; load no reference
weights and perform no new reference scoring. This is a changed structured
architecture at a matched dose, not an isolated attention-mechanism comparison.

All 8,192 supported construction questions expose the required four facts
before readout. The canonical reference leaves 4,457 supported errors,
2,001 owner-pair and 1,601 object-pair gains available; neither information
reachability nor pair ceilings preclude the declared gains. The 2,048 absent
questions can select a learned null. This establishes opportunity, not expected
success. The first eight real training updates are the resource admission
instrument; no additional fit or tuning follows a miss.

## Qualification and causal control

**Empirical criteria.** Evaluate all 10,240 construction rows in each of four
right-rotated fact orders (offsets 0, 1, 2, 3; zero is canonical). Each order
must reach:

- Supported: **8,111/8,192** correct.
- Missing binding: **1,946/2,048** correct UNKNOWN answers.
- Complete four-supported-question groups: **973/1,024 in each question
  family**, owner and object. The family criterion is separate: concentrating
  the 81 errors permitted by the supported threshold can leave only 943
  complete groups in one family.

All 10,240 top-1 predictions must be identical across orders, and the maximum
absolute difference over every full-head logit from canonical order must be
at most **1e-4**. Report the actual maximum, changed-prediction counts and
attention differences after undoing fact rotation with null fixed. There is
no additional cross-order attention tolerance gate. Four orders are correlated
views and cover four of 24 fact permutations.

Only after construction and order qualification, run one causal intervention
in all four orders: **right-cycle the four projected fact values by one**,
keeping Q, K, attention and null value fixed. Destination key j now carries
the original location value at `(j - 1) modulo 4`. Replacement labels are
derived by the scorer from the original target's unique fact slot and that
fixed inverse index. They are never supplied to the model or intervention.
In each order require:

- Original supported correctness drops by at least **4,096/8,192 = 50pp**.
- Reassigned locations are correct on **7,783/8,192** supported questions.
- Missing-binding correctness remains at least **1,946/2,048**.
- Actual attention weights are exactly unchanged from that same order.

Report changed UNKNOWN predictions separately; threshold preservation does not
assert their exact identity. Distinct fact locations make replacement labels
different from original labels. The replacement criterion already implies a
large original-answer loss; the explicit drop criterion remains binding.

## Fresh development and divergent decisions

Freeze development seed **10732**, 256 groups, 1,024 supported and 256 unknown
rows, 64 held-out owner-object pairs and the same grammar/vocabulary. Exclude
construction and #1063/#1067/#1069/#1071 development canonical base, swapped
and absent worlds, exact inputs, and reuse across new groups. Recreate older
populations using unchanged generators and verify their published CIDs; do not
open retained historical payloads. Preparation audits labels and exclusions.
Default validation and fitting neither open nor regenerate development.

Development opens only after construction, order and causal control all pass.
Each of the four development orders must reach 973/1,024 supported, 231/256
complete groups, 116/128 complete groups in each question family, and 244/256
unknown answers. This measures new combinations in familiar grammar.

1. Order criterion fails: `COMPOUND_BINDING_ORDER_MISS`; investigate the
   declared permutation interface before claiming binding qualification.
2. Construction misses: use the inherited matching-order #1067 comparison
   only to interpret partial progress. Each family needs both-correct pairs
   at least `min(reference + 205, 2048)`, each target slot 1,024/2,048, and
   preserved overall, individual-family and paired accuracy. A regression is
   `COMPOUND_BINDING_PRESERVATION_MISS`; otherwise return partial gain or
   below-declared-gain/slot-floor. Partial gains are retained, but development
   remains unscored and full construction qualification is unresolved.
3. Construction/order pass but control fails:
   `COMPOUND_BINDING_CONTROL_MISS`; retain the fit and investigate value
   dependence; development remains unscored.
4. Construction/order/control pass but fresh development misses:
   `COMPOUND_BINDING_FRESH_TRANSFER_MISS`; retain structured construction
   binding and causal evidence, address fresh-combination transfer separately.
5. All pass: `COMPOUND_BINDING_FRESH_PASSED`; proceed next to a separately
   frozen **unchanged-R4 inference-preservation** experiment.
6. Incomplete/resource/integrity outcomes are explicit, with the partial state
   and cumulative clock retained; no replacement fit or restarted budget.

This issue performs no R4 execution, frame expansion or free generation.
#973 stays open and #954 remains blocked.

## Resources, review and reproducibility

The cumulative fit/evaluation/replay ceiling is **1,800 seconds and 4 GiB RSS**,
with eight Apple Accelerate CPU threads and one inter-op thread. Preserve the
first-eight-real-update admission projection, multiplier 1.25 and 60-second
evaluation allowance. #1071 took 292.89 seconds to fit and 315.59 seconds for
the complete path; the new architecture has no measured runtime estimate
before its admission updates. The cap is binding for every decision branch.

Focused checks cover exact initialization/RNG, parameter count, causal fields,
label independence, all 24 synthetic fact permutations, value-control isolation,
gradient paths, artifact reload, whole-trainer AST equivalence except declared
model/metadata changes, copied data and exclusions, no-development access,
recorded lineage, rectangular scoring and all decision boundaries. Independent
model/contract and campaign source reviews found no blocker. All 18 focused
tests passed (10.296 seconds); Ruff and claim wording checks passed. Preparation and
final evidence require separate review. Broad workspace, BDD, WASM, fuzz,
audit and release QA remain dormant; queue statuses acknowledge transport only.

From `tools/r4-softmax-trainer` in the frozen offline environment:

```bash
.venv/bin/python -m r4_softmax_trainer.zoology_compound_binding prepare \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding \
  --source-root /Users/casey.allard/uor-r4/.uor-models/research/issue-1063-zoology-english-binding
.venv/bin/python -m r4_softmax_trainer.zoology_compound_binding fit \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding
.venv/bin/python -m r4_softmax_trainer.zoology_compound_binding run \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding
.venv/bin/python -m r4_softmax_trainer.zoology_compound_binding verify \
  /Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding
```

Publish the immutable preparation and source commit before the first update.
Append outcomes and reconcile the current mirrors and native trackers after
exact fresh-process full-evidence replay. Close only #1073 after protected merge.


## Published preparation (2026-09-02; zero fitted updates)

Source freeze: `041d52cfaaffd944b4717e32e148063185569ae3`. The
[preparation envelope](r4_zoology_compound_binding_1073_preparation.json) binds:

- preparation: `blake3:360b6c6bb63f4040d0baff06c9e56b3038111587bdf234681e6f4d1cdc89d038`.
- 270-file implementation: `blake3:28519bb41f02590b6a358dda935f4a3cfd0afead1a1e87777385c3e270533495`.
- dataset manifest: `blake3:574d667e61b70e32c39b26d43547d5aeb29e92f16260fa840ddd4eda30c4e694`.
- dataset tree: `blake3:c150f19e02caaa7537e2fd244ebd3dae27f6e561c8869b69c91b93351044a6da`.
- construction.safetensors: `blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2`.
- development.safetensors: `blake3:0e343a7448098ea2a22d850d4d9fb31d75c55090807f4b2f00fa20531a422335`.
- vocabulary.json: `blake3:aa001c3a4369ad2f8bb3596a316270bd72b736f927158ee04403116b430c649d`.

The five excluded populations contain 15,360 unique input rows and 9,216
canonical worlds. Fresh development overlaps neither set. Original construction
and vocabulary bytes match #1063. Preparation audits the 256 new groups and
768 new worlds; fitting/default validation will not open or regenerate them.

## Outcome (2026-09-02): compound binding and fresh transfer passed

The single final candidate returned **`COMPOUND_BINDING_FRESH_PASSED`**.
All frozen construction, order, value-control and fresh-development criteria
passed. Publication preceded fitting in commit `9e4675a0` and
[issue comment 5518436017](https://github.com/UOR-Foundation/uor-r4/issues/1073#issuecomment-5518436017).
Independent preparation review validated the published identities with guards
rejecting development, model and checkpoint payload reads before launch.

| Measurement | Result in each of the four orders |
|---|---:|
| Supported construction | **8,192/8,192 = 100%** |
| Missing-binding construction | **2,048/2,048 = 100%** |
| Complete construction quartets, owner-changing | **1,024/1,024** |
| Complete construction quartets, object-changing | **1,024/1,024** |
| Supported fresh development | **1,024/1,024 = 100%** |
| Missing-binding fresh development | **256/256 = 100%** |
| Complete development groups | **256/256**, including **128/128 per family** |

Canonical supported NLL was **0.0005899063353**; aggregate development NLL
was **0.0007402848845**. Every construction slot reached 100%, and all
2,048 paired questions in each family were both correct. The recorded #1067
supported reference remains 3,735 / 3,354 / 3,487 / 3,347 correct by order.
Its history remains intact; #1073 is the qualified structured-binding
candidate for the next preservation step.

Across all 10,240 construction rows, every prediction was identical under
four cyclic orders. Maximum absolute full-head logit differences from
canonical were **0 / 5.7220458984375e-6 / 4.76837158203125e-6 /
4.76837158203125e-6**, below the frozen 1e-4 threshold. Maximum aligned
attention difference was 2.384185791015625e-7. Thus top-1 was exact across
orders; floating logits were tolerance-equivalent, not byte-identical.

The value intervention passed in every order. Cycling only four projected
fact values reduced original supported correctness **8,192 → 0** and produced
the reassigned location on **8,192/8,192** questions. All 2,048 unknown answers
remained correct with zero changed predictions. Actual attention tensors were
exactly identical to the corresponding unmodified order. This is causal
evidence that decoded supported answers follow the attached location values
under fixed queries, keys and attention.

Mean canonical null attention was **0.5463976382 on supported questions** and
**0.9125146940 on unknown questions**. Success therefore does not establish
winner-take-all selection of a fact or an argmax-null absence rule. The learned
weighted mixture and output decoder matter; a geometric adapter must preserve
the entire four-fact-plus-null mixture.

Construction and control qualification admitted the already frozen fresh
population. Each order scored all 1,280 rows; the four views total 5,120 model
decisions, not 5,120 independent examples. Every same-history question change
and same-question location swap changed the supported answer correctly.
None of the absent cases retained the former location. The first frozen
example quartet answers Ada's comb/hat as **trunk/pouch**, then **pouch/trunk**
after the location swap, and **unknown** after the owner-object binding is
removed. These are selected supervised readouts at token 37, not generated
sentences.

The fresh result concerns the **64 held-out owner-object combinations in
familiar grammar and vocabulary**. It does not establish arbitrary English
parsing, reasoning, free generation or chat readiness. The supplied role
interface and architecture changed together; this does not isolate a geometric
advantage or support geometry expansion.

## Actual dose, resources and replay

The final artifact completed exactly **3,920 optimizer updates** and
**2,007,040 presentations = 1,846,495 supported + 160,545 unknown**.
The 78 full mixed traversals account for 159,744 unknown presentations;
the final eight batches contain **801 unknown and 3,295 supported**.
No old model, development or model frame was read during fitting. There was
one seed, one fit and one final candidate, with no extra dose or intermediate
evaluation.

First-eight-update admission passed: measured mean step time
0.02177829175 seconds, projected remaining total 166.49585 seconds including
the 60-second evaluation allowance. Actual times were:

- Fit: **49.903205583 seconds**.
- Final evaluation, including conditional control/development: **7.145526833 seconds**.
- Exact fresh-process full-evidence replay: **6.750702708 seconds**.
- Cumulative: **63.799435374 seconds**, peak **1,421,606,912 bytes =
  1.323974609375 GiB**, within 1,800 seconds / 4 GiB.

The evaluator process was **23350** and replay process **23385**. The full
evidence CID reproduced exactly, including all logits/attention/prediction
identities, diagnostics, controls, examples and development metrics. Learned
state was identical before/after evaluation and bound to the exported model.
Evaluation/replay performed zero optimizer updates, checkpoint/optimizer/RNG
payload reads, reference model loads, historical development payload reads,
label arguments to model forwarding, geometry changes or native-frame reads.
R4 remains `NOT_RUN_SEPARATE_INFERENCE_STEP`.

The reviewed source passes 18 focused checks; the whole-trainer AST comparison
confirms only the declared model/metadata changes. Broad workspace, BDD, WASM,
fuzz, audit and release QA were not run. The protected queue's statuses concern
transport only and are not additional scientific evidence.

Independent final review verified eight self-bound evidence envelopes, all 270
frozen implementation files, the published preparation, model bytes and the
learned-state identity reconstructed directly from Safetensors bytes. It
recomputed the dose, every decision criterion and resource totals, with no
additional model scoring. No blocking evidence finding remained.

| Immutable evidence | CID |
|---|---|
| [Fit](r4_zoology_compound_binding_1073_fit.json) | `blake3:9c6bc25f9bcfa8279fbba6acf15d0bf1279653652c46222473fcc23fb95daf84` |
| Model, 1,148,672 bytes | `blake3:9c055cc6ea09548bf960e37288276535b30515b94a50a96aa929b5e55afea3c4` |
| Learned state | `blake3:abbdbcaafc2d9eb36543ce75fbb0101b6788119d80a6ed9c017bb9d06fbeac59` |
| [Result](r4_zoology_compound_binding_1073_result.json) | `blake3:1f3c5bee5ebd0e8e34f9f1a5fa03d514b397928638fd66deaf64b8abf7946041` |
| Complete evidence | `blake3:6ffc6c755401c601b0ff29adf139bd00dce93e4c0a650699b7e2dc126118121d` |
| [Fresh-process replay](r4_zoology_compound_binding_1073_replay.json) | `blake3:9500d279e228eb3fff646a537fcbbbf861aafc216cd7e7ad750444381c4a17f2` |

Retained local root:
`/Users/casey.allard/uor-r4/.uor-models/research/issue-1073-zoology-compound-binding`.
Source/optimizer/RNG checkpoint, weights, data and raw envelopes remain there.

## Next decision

Proceed to one separately frozen **unchanged-R4 inference-preservation**
experiment on this exact learned artifact. Adapt the existing coherent R4
mapping to the rectangular four-fact-plus-null attention interface and compare
ordinary attention, coherent R4 transport and a predeclared broken-transport
control on the same supported, absent and fresh-combination decisions. Preserve
the complete softmax weighted mixture, null entry, full vocabulary decoder,
learned parameters and causal role interface. Freeze the adapter, controls,
preservation tolerances and resource ceiling before scoring. No further fit
or geometry expansion is warranted by this result.

A preservation pass would carry the demonstrated binding behavior into R4;
a miss would identify a transport/adapter preservation failure to resolve
before broader language work.
The older #1059 and #1061 results remain scoped to their respective models.
They do not substitute for preserving this new compound-binding model.
#973 stays open and #954 remains blocked.
