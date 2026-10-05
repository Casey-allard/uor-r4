# Conditional span learning improves capture; frame and reader transfer remain — October 5

Removing unused-branch span credit produces substantial bounded capture gains. It does not solve unfamiliar utterance classification or the frozen reader. The next implementation is source-only ordered geometric frame composition; actual natural cue/query source-binding learning is an independent reader dependency.

## Executed repair and scope

Source `13d10d222166a43f39623ec0588c2e02c4efac85`, Linux x86 CPU executable SHA256 `f3b49b78afbf4c2a0059c10736853ef365f43dcf798f652fc07bc67d75d94afc`. Explicit `--span-objective conditional-write --curriculum crossed-2`. Span loss and development selection condition on labelled assert/update tasks and average span CE over write rows. Act/relation credit remains on all rows. No predicted-act loss gating, runtime gold span, supplied answer or selected record is added. Runtime prediction, artifact layout and span decoding are unchanged; legacy all-row replay remains default.

The combined intervention changes span credit, normalization and checkpoint selection. Same-checkpoint artifacts preserve byte-identical act/relation arrays across all 30 old/new checkpoint pairs, so their changed actions are span/fallback effects. Selected-model comparisons can also reflect different chosen checkpoints. No separate gradient-versus-selection causal ablation is asserted.

Both local H4 orientations use three real paired head seeds 1001/1002/1003, the same frozen carrier/donors, 512 weighted training rows, 128 development rows, 64 updates and learning rate .03. Earliest strict minimum native conditional development CE includes step0. Training/development/factor sources and labels are exactly the prior curriculum. New fresh48 rows and three episodes are frozen before fit, with exposed-source/value exclusions; fresh never selects. Neither a transformer score nor open-domain knowledge gates this experiment. These are six readouts on one encoder, not six chat-model lineages.

## Selected actual outcomes

| Arm / seed | Step | Dev writes /64 | New fresh writes /32 | Dev exact store /12 | Dev selected reply /12 | Dev all-bank reply /12 |
|---|---:|---:|---:|---:|---:|---:|
| relative /1001 |32|61|1|0|0|1|
| product /1001 |32|62|1|11|7|5|
| relative /1002 |16|54|1|5|4|5|
| product /1002 |32|60|1|11|7|5|
| relative /1003 |16|46|2|6|4|5|
| product /1003 |16|57|2|11|7|5|

Actual predicted actions drive writes, original statements remain source cues and actual natural questions drive reads. Store, selected-record consumption and all-bank generation are separate results. Product store gains do not imply a reader improvement: 75 old/new executions with identical reader inputs reproduce identical generated IDs and full available traces. All-bank gains and regressions caused by changed compilation remain retained.

Across matched selected factors, familiar wording/new values improves48/384→205/384 and repetition8/96→65/96. These pool measurements of fixed sources across six heads; they are not independent linguistic samples. The local span diagnostic's supervised conflicts decrease100→4 as predicted. This is useful learning evidence and does not establish geometric superiority.

## The remaining failures are different mechanisms

Familiar-wording/new-value factors retain82 correct-start end truncations,38 start-only and9 combined emitted boundary failures. Repetitions retain19 end truncations. Conditional complete-interval credit and learned capture remain justified alternatives for these continuation errors.

Novel-wording/known-value factors reach247/384 exact writes, with only3 emitted boundary failures and62 exact-span outputs having wrong act/relation. Another72 outputs are NONE/query; none is a no-positive-span fallback. A span-only intervention cannot repair their frame classification. On new fresh,135/192 write evaluations decline through NONE act/relation versus6 no-positive-span fallbacks; all8 fresh questions fail in each head. These final reasons do not identify which raw head failed. Raw fresh head scores are NOT_RUN; do not infer them from fallback actions. Old and new fresh panels differ and their rates are not compared.

Every learned fresh episode has0/12 store answers and0 native reader calls: learned fresh reader behavior is **NOT_RUN**. The reference instrument independently compiles/stores12/12 on both episode splits. Its fresh selected reader starts correctly on all12 unique queries but completes0/12:8 later wrong-Copy errors,3 premature Period and1 premature Stop. Full-bank reference also completes0/12, with9 wrong-record first tokens despite target support. Those same frozen-reader cases repeat across the compiler runs and are not72 independent trials.

The development ordering control isolates source binding. The same job query, mariner job value, carpenter home value and assertion cues produce `mariner.` forward and `carpenter.` reversed. The correct target's individual Copy score increases21,466,591→25,165,825, while the distractor increases14,680,065→32,753,392 and wins. The target is present; ranking is defective. Terminal calibration is not the default repair for this first-token routing error.

## Next native implementation

Preserve conditional local heads as research parents and controls. Add source-only ordered prefix/frame composition using exact H4 products over the actual word observations, with one bounded carrier group for act/relation and span. Compare prefix-to-word geometric transport against a same-slot readout of the same prefix information. Match masks, parameter counts, data, initialization and supervision; report differing table-operation cost. Prefix state is updated incrementally, with no repeated full-prefix encoding, runtime floating point, dense transformer, gold payload mask or forced span closure. Exact bytes remain separate from finite geometric observations.

This adds missing ordered frame context rather than repeating frozen local heads. If that information does not improve classification, train the geometric frame transitions themselves using the existing offline learning/bridge machinery; do not repeat dose/seed tuning or retire the geometry family. Conditional interval likelihood is a separate narrower option, not a mandatory gate before frame work.

In parallel, reuse actual native all-source marginal loss to learn natural cue/query geometric observations and source-binding interactions. Correct admission, first-token record choice, within-record progression and termination remain distinct. The fresh reference changes wording and values together, so a raw-query-only cause is unproven. Preserve the ordered-prefix/source-end donors and evaluate complete own-prefix answers/EOS. GroundedSession integration remains separately coordinated on#1552; shared session files are unchanged. General prose, reasoning, long-distance attention, full chat and laptop energy remain open.

## Checks, cost, interruption and retention

Actual13 compiler-library tests,10 driver/curriculum tests and release pass. Six valid completed fits are independently admitted with133,922 saved-artifact checks and0 errors: input identity, three genuine initialized seeds, paired initializations, all checkpoints, earliest native selection, export packing, row flags/counts, positive controls and actual store/native output receipts. Reviews do not rerun BPE, native encoding, gradients/optimizer or BLAKE3. Actual drivers seal and verify report roots. Metal unavailable; GPU unused.

The initial product1003 attempt stopped at0.322188s, exit−15, due the aggregate512MiB report cap before producing a model. It is retained INCOMPLETE and excluded. A prospective report-only512→768MiB correction permitted a distinct retry root; none of the five completed fits was repeated. Source/data/objective/dose/seed/selection and whole-card limits stay fixed.

Ten worker receipts total511.822838s: build/test475.760985s, model/evaluation including interruption36.061853s. Maximum sampled process-tree RSS2,921,885,696B, maximum recorded owned storage9,965,305,856B. Same120-minute card includes preparation/review/return/delivery; cumulative ledger1,285,035,028/1,285,800,000ms. CPU48/RSS8GiB/new-owned1GiB/owned10GiB/free4GiB+128MiB. Preparation's separate local full worktree1.22GiB is recorded; no local Cargo build.

Complete returned416-file set757,758,064B matches SHA256/size inventory. Inner archive38,675,295B SHA256 `e0693b9c8139c08f1cba68303fae8334f8dd85ffe564756c0ed71d418210f925` is retained under approved `/workspace/codex-uor-r4-20261004` and local research `~/uor-r4-local/workspace/research/geometric-conditional-span-20261005`; cloud receipt is in machine evidence. All six models, earlier parents, negatives and the interrupted attempt remain preserved.
