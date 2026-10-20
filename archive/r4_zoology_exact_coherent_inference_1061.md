# Exact-data coherent R4 inference (#1061)

**Authority:** [#1061](https://github.com/UOR-Foundation/uor-r4/issues/1061), native child of #973.

The user authorized the next step recommended after #1059: reuse its working inference adapter on #1057's preserved final block-40 model and original development population. Parent: #973. Prerequisites: closed #1057 and #1059. Start from protected main `83e1e90d9fae8e92457e32f16223996f5389f4ed`.

## Deliverable

Add a small sibling inference entry point that binds the exact #1057 source/model/data and reuses the unchanged #1059 frame loader, inner-attention adapter, model loader, scorer, numerical comparison, CPU policy and resource/replay helpers. Compare all 8,192 development query decisions through plain and coherent R4 paths without fitting or selecting another checkpoint. Deliver frozen preparation, result, exact fresh-process replay, concise current programme summaries and protected closure.

## Frozen source and mechanism

- #1057 result: `blake3:35b1cedfd51385bf98277a4527b1ce05f5dd3b93fffe125a5ea28c2a34b6387c`.
- Final block-40 model: `blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445`, 1,217,024 bytes; state `blake3:f2a67ec0cc7ac44f586b815da43efabcc81d444b1bab9954b5536c37cb96ff90`.
- Primary exact-data file: `blake3:96f154042f0fd920c7f6f3b1b650a6ce20f11c401f9ae0c81734f47ae231b7f1`, retained under #1053's data root. Read only `test_inputs`, `test_positions`, `test_targets` tensor values. Hashing may cover the immutable container. No training tensors, checkpoint/optimizer state, evaluation RNG or physical binding-control tensors are loaded.
- Width 64, two layers, one head, vocabulary 4,096, sequence length 120, eight queries per row; all learned embeddings/positions, QKV/output biases, normalization, residuals, identity mixer, softmax and tied head remain unchanged. Source attribution remains HazyResearch/Zoology at `de4e258784224e09909c257ff3ea040f089ed660`.
- Reuse #1059's complete native 8,192-token frame map and sidecar unchanged. Its frame-map coverage includes all 4,096 model tokens; do not slice, rederive, expand or retune geometry. Revalidate the bound bytes/native witnesses, but do not rebuild/export native geometry or rerun its historical tests.
- Reuse `R4ZoologyInference` unchanged. Apply complete eval mode after installation, disable gradients, and pass targets only to the scorer. No module-global monkeypatching or edits to historical CID-bound source.

## Predeclared inference decision

Evaluate canonical development rows 0..1023 in two batches of 512, with identical boundaries in plain, R4 and conditional control. The historical evaluation shuffled its two batches: record new canonical output digests, not an invented match to its shuffled logits CID. Correct-count reproduction is the historical comparison.

Primary preservation requires:

1. Plain reproduces exactly 8,071 correct of 8,192 decisions (98.52294921875%).
2. R4 preserves every one of the 8,192 plain top-1 IDs. Maximum logit absolute difference <=0.005, maximum attention-weight difference <=1e-5, and mean NLL difference <=1e-5 nats. These are unchanged #1059 engineering tolerances, not a new 99% training threshold.
3. Source/model/implementation/frame bindings and learned-state/tied-head identities remain unchanged. Causal source support, zero future attention weight and complete vocabulary coverage hold. Fresh-process replay reproduces the complete inference evidence.

Only after primary preservation passes, run one same-artifact/same-support inconsistent-transport control. As in #1059, source K/V encodings remain in their true frames while transport uses the existing HELM causal-prefix cyclic source-frame permutation `(s+1) mod (q+1)`. Require complete decisions, zero future attention weight/read through transported support, and matching work before interpreting recall loss. A >=50 percentage-point drop establishes strong sensitivity to deliberately inconsistent transport on this population. A weak or invalid control retains a primary preservation positive and leads to review of that intervention only.

This control belongs to the new issue. #1057's physical binding-permutation control remains historically `NOT_RUN_PRIMARY_MISS`; no threshold/result is retroactively changed. The population is previously observed assignment-disjoint development, not a new sealed evaluation.

If preservation and control pass, retain the exact-data R4 path and recommend a narrowly scoped supplied-context language-binding application under #973. If preservation misses, retain both checkpoints and localize only the new data/config/inference wiring or precision discrepancy. Integrity/resource interruption is incomplete, not a model negative. No new training, third continuation, parameter sweep, geometry expansion, English generation, correctness/reasoning, product chat, softmax replacement, exact/table lowering or release follows automatically. #954 remains blocked.

## Resources and minimal checks

All 8,192 query decisions traverse the existing adapter, so preservation is fully observable. Causal pair work is 14,868,480 per two-layer pass, versus #1059's 12,480,000 (1.1914x); selected vocabulary-logit volume is about 0.3413x. #1059 completed run plus replay in 27.49 seconds. A 1.25 safety factor on the larger work ratio projects about 41 seconds. Reuse four CPU/Apple Accelerate threads, one inter-op thread, one process, no CUDA/MPS. Keep the same 900-second combined run/replay and 4 GiB peak-RSS caps. Finite batch progress and create-once start markers prevent silent repetition or budget renewal.

Activate only: new development/config binding checks and minimal decision/accounting checks; unchanged source/frame-byte validation; the matched primary and conditional control; one fresh-process replay; scoped formatting/claim-wording/diff checks; independent changed-code and evidence review. Reuse all #1059 native/adapter/scorer evidence. No old source reproduction, checkpoint audit, native build, workspace/BDD/WASM/fuzz/audit/conformance campaign or CPU timing matrix. Queue compatibility checks remain delivery acknowledgements.

Commit and publish the implementation, preparation and run binding before the first fitted-model forward. Append results without rewriting history.

## Definition of done

- [ ] Live parent/prerequisite relationships and assignment verified.
- [ ] Exact source/data and unchanged #1059 adapter/frame implementation bound.
- [ ] Focused new wiring checks and independent prereveal review pass.
- [ ] Matched inference, control status, resources/work and fresh-process replay recorded.
- [ ] Original artifacts and historical modules remain unchanged; zero optimizer updates.
- [ ] Current direction updated; protected PR merged; issue and parent outcome recorded.

## Pre-inference freeze — 2026-09-02

The implementation at `42d5eb34` and exact predecessor bindings are complete.
Six new focused checks passed: three development/root/domain/binding checks and
three preservation/control/accounting checks. Scoped Ruff formatting/lint,
claim wording, diff hygiene, and independent source review passed. Existing
#1059 adapter/scorer/native evidence was reused; no inherited tests, native
build/export, fitted model forward, or training ran during preparation.

The [frozen preparation](r4_zoology_exact_coherent_inference_1061_preparation.json)
binds 188 implementation files, including all 153 original #1059 files and
all 54 #1057 trainer records (overlap deduplicated), the three #1059 evidence
documents, and the new package/tests. The source model and dataset identities
revalidated unchanged. The model and data roots remain distinct.

- Preparation CID: `blake3:c8e97664f7feab8c83ad15d298620da675bc3f156a9b0dcfcfa98ac69fad6c35`.
- Implementation tree CID: `blake3:d9272cbf0fd01535cb6b1aa6c6baaee3bdc59a312b0ea66674ccf307328431e6`.
- Source model CID: `blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445`.
- Native frame bundle CID: `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.

Primary comparison, conditional transport control, and fresh-process replay are
`NOT_RUN` at this freeze. The original thresholds and 900-second combined /
4 GiB / four CPU-thread policy remain unchanged.

## Completed exact-data inference and replay — 2026-09-02

**Terminal: `EXACT_DATA_R4_PRESERVED`.** The implementation frozen at
`42d5eb34`, with preparation committed at `6d75643f`, ran without modification.
The sole matched primary and its conditional transport control passed their
separately declared criteria. Fresh-process replay reproduced the complete
inference evidence exactly.

| Measure | Plain | Coherent R4 | Inconsistent source transport |
| --- | ---: | ---: | ---: |
| Correct / query decisions | 8,071 / 8,192 | 8,071 / 8,192 | 1,009 / 8,192 |
| Recall | 98.52294921875% | 98.52294921875% | 12.31689453125% |
| Mean NLL (nats) | 0.0914115384221077 | 0.0914115309715271 | 8.528378009796143 |
| Top-1 changes versus plain | 0 | 0 | 7,156 |

Every one of the 8,192 original/R4 predictions was identical, including the
same 121 incorrect decisions. Plain reproduced the exact historical
8,071-correct count. Maximum selected-logit difference was
`2.5272369384765625e-05` against `0.005`; maximum
attention-weight difference was `2.2351741790771484e-06`
against `1e-5`; mean-NLL absolute difference was
`7.450580596923828e-09` against `1e-5`. All frozen
preservation criteria passed. There was no new 99% training threshold.

The new inconsistent-transport control lost `86.2060546875`
percentage points, exceeding its separately frozen 50-point sensitivity
criterion. Complete decisions, zero future attention weights, causal transport
support, and matched work all passed. This establishes sensitivity to
inconsistent transport on the previously observed exact-data development
population. It does not establish H4 superiority or new sealed generalization.

### Exact state, work, and access boundaries

The original final block-40 model retained all learned tensor bytes and the tied
head. Model-state CID remained `blake3:f2a67ec0cc7ac44f586b815da43efabcc81d444b1bab9954b5536c37cb96ff90`.
V4096/T120, eight query positions per row, learned positions, normalization,
residuals, QKV/output biases and softmax stayed unchanged. There were no
optimizer updates or checkpoint/optimizer/evaluation-RNG reads. The loader
opened only three development tensor values in their original #1053 container;
training tensor values and historical physical-binding-control tensors were
not loaded. No original source artifact or historical implementation was
rewritten. Labels reached only the scorer.

Both native and control arms admitted `14,868,480` causal
attention pairs, transported `237,895,680` key blocks and
the same number of value blocks, across two 512-row batches and two layers.
The control shifted `14,866,432` source-frame
positions, of which `11,217,652` changed actual
frame matrices. Plain separately materialized `29,491,200`
score slots, including `14,622,720` future
slots masked to zero attention. Its physical future-source read count remains
unknown. Native/control per-query transport read no future source; QKV and
pointwise frame encoding remain vectorized across the input.

The unchanged native map covers 8,192 token IDs and therefore all 4,096 model
tokens. Inference reached 24 H4 frames. No frame expansion, slicing/redefinition,
native exporter run, or frame-mechanics rerun occurred. The retained plain
logits occupied `134,217,728` bytes during the
conditional comparison.

Canonical rows 0..1023 and new output digests were used as frozen. The original
shuffled-development logit digest was not reproduced or relabeled. The original
#1057 `CONTINUATION_MISS` and physical binding-control `NOT_RUN_PRIMARY_MISS`
remain unchanged; this issue's transport control is a different intervention.

### Resources and durable evidence

The primary/control run took `15.959983 s`; replay took
`17.834691 s`; combined time was
`33.794678 s` against 900 s. Maximum observed RSS was
`2,588,475,392` bytes
(`2.410706 GiB`) against 4 GiB.
Both processes used Python 3.12.14 / PyTorch 2.7.1, one CPU process, four Apple
Accelerate intra-op threads and one inter-op thread. This measures the campaign
and replay, not isolated plain/R4 speed or preparation/review wall time.

- [Preparation](r4_zoology_exact_coherent_inference_1061_preparation.json):
  `blake3:c8e97664f7feab8c83ad15d298620da675bc3f156a9b0dcfcfa98ac69fad6c35`.
- [Raw result](r4_zoology_exact_coherent_inference_1061_result.json):
  `blake3:ac2ec4d533ac47d25f8eb9dfd7a41147147d73c0e2d9531352d9f9fb2eb84e58`.
- Inference evidence CID: `blake3:a14e0e6fe0f915acc9f81b446a0ee6da7d3e97723f60c269125de55120399802`.
- [Fresh-process replay](r4_zoology_exact_coherent_inference_1061_replay.json):
  `blake3:af6c239ec2d0e11f26f50f74150c992dea345ec21257141fcec1096a573e708e`.

The committed JSON files are exact copies of the create-once local records.
The six new focused checks, scoped formatting/claim wording/diff checks and
independent prereveal review passed. Historical #1059 mechanics evidence was
reused. Broad QA remained dormant; required queue jobs are delivery
acknowledgements rather than test evidence.

### Decision and next recommendation

Preserve both the #1050 reference and #1057 exact-data model, with their now
working coherent R4 inference path. This completes the #1061 inference transfer
without training, model selection or a geometry change.

The next recommended scientific step is a separately frozen small English
supplied-context binding curriculum under #973 using this working attention
architecture and adapter. Hold the question fixed while supplied facts change;
include distractors and swapped/missing-history controls so the answer must
follow its own supplied context. Freeze lexical encoding, learning dose and
disjoint construction/development populations before fitting. The retained
MQAR checkpoint is not an English model, so zero-shot English failure must not
be used as an attention-existence verdict. This later task needs its own
bounded learning/serialization contract; no fit or English probe runs here.

Geometry expansion remains deferred. More coherent frame labels alone change
coordinates, not capacity. No English generation, correctness, reasoning,
softmax removal, exact/table-native lowering, product chat or release claim
follows; #973 remains open and #954 stays blocked.
