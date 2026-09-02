# Coherent R4 inference of the preserved associative-attention model (#1059)

- **Initial status at contract freeze:** implementation in progress; fitted-model inference NOT_RUN.
- **Current outcome:** `R4_INTEGRATION_PRESERVED`; complete result and replay appended below.
- **Authority:** [#1059](https://github.com/UOR-Foundation/uor-r4/issues/1059), native child of #973.

Parent: #973. Prerequisites: completed #1050 and #1057. The user has authorized the next inference-only integration described in #973's latest delivery. Preserve the established ordinary/R4 and retained-attention positives and both learned artifacts; there is no new fit or third continuation window.

## Deliverable and project value

Add a reusable, parameter-free coherent-R4 inner-attention adapter for the exact qualified #1050 Zoology model. Execute its retained 3,000 test rows (12,000 query decisions) through plain and R4 inference using identical weights, inputs, ordering and batch boundaries. Preserve its learned associative behavior while connecting it to the existing native Spin/H4 route/frame contract. Deliver the implementation, content-bound result and replay, current programme summaries, and protected PR closure.

## Frozen input and mechanism boundary

- Start from `d424dbee0272eeaa61f23262ff863495be956593`.
- #1050 source result CID: `blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0`.
- Model CID: `blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9`; 2,251,264 bytes. Model-state CID: `blake3:600bdc76cefff79f4be8709197b15252cb531892fad0db2156b36b865c01877e`.
- Retained dataset file CID: `blake3:f6dd39f9e0554df7409ee051e353798b89de8047d9f3ce32b983fa83623754b8`. Read only test tensor values for inference; file hashing may cover the immutable container. No training tensors are fitted or scored.
- Source architecture remains width 64, two layers, one head, vocabulary 8,192, length 64, learned positions, original Q/K/V/output biases, normalization, residual ordering, identity state mixer and tied head. Attribution remains HazyResearch/Zoology `de4e258784224e09909c257ff3ea040f089ed660`.
- Import the unchanged CID-bound Zoology cell; do not edit its historic source/loader/trainer/test files. Install only a new inner-attention module, set the complete model to eval after installation, disable gradients, and reject training use. No new learned parameters.
- Export the full 8,192-token frame-leaf map from the existing Rust `R4SpinFrameAtlas` policy; do not reuse/clamp the older 4,096-token retention map. Reuse the canonical Rust 120-frame H4 sidecar and strict loader. Bind root/product identities, map, exporter and dependency source closure. Frames are causal cumulative products of the existing token leaves. No target/role/outcome-derived geometry.
- Within each four-lane block, encode with `F_s^T`, transport with `F_q^T F_s`, use the unchanged scaled dot product and stable causal softmax, aggregate in the query frame and decode with `F_q` before the original output projection. Floating-point gauge arithmetic is an offline realization, not an exact/table runtime claim. Transport only causal source positions; retain every lawful K/V position.

## Evaluation and decision

The historical #1050 test loader shuffled rows; its terminal checkpoint saved RNG after scoring. Therefore the new inference contract uses canonical row order 0..2999 and batch size 512, with new canonical output digests. It does not pretend to recreate the historical order-dependent logits CID.

Primary integration criteria, frozen before fitted model inference:

1. Plain inference reproduces #1050's 11,900 correct decisions out of 12,000; learned tensor bytes/tied weights and source identities remain unchanged.
2. Coherent R4 preserves all 12,000 plain top-1 IDs, with maximum selected-logit absolute difference at most 0.005, maximum attention-weight difference at most 1e-5, and mean NLL difference at most 1e-5 nats. These are engineering tolerances with exact decision agreement; the logit envelope reuses the prior #1014/Rust attention tolerance rather than #1043's over-tight 2e-5 gate. Report measured differences and accuracy without retroactive threshold changes.
3. Causal prefix isolation, frame-map coverage, identical learned state, source/model/dependency binding and fresh-process replay pass. Read target labels only in the scorer. Report actual dense materialization versus admitted causal work truthfully; do not claim zero physical reads from a vectorized masked square unless implemented.

After primary integration passes, run one same-artifact/same-support destructive transport control: encode source K/V in their true frames but select transport source frames from the existing HELM causal-prefix cyclic source permutation `(s+1) mod (q+1)`. Query frames, positions, masks, parameters and source payloads remain fixed. This is deliberately inconsistent frame transport, not a consistently relabeled valid gauge. Record changed source frames, work, logit/top-1 changes and accuracy. A >=50 percentage-point recall loss establishes strong transport-path sensitivity on this population; a weaker control preserves any positive integration/parity evidence and leads only to reviewing the intervention, not discarding/refitting the model. This control does not establish H4 superiority.

If integration passes, preserve the working R4 inference path and define its next language/context-binding application under #973. If integration misses, preserve #1050/#1057 and localize only the new adapter, precision, or frame-wiring discrepancy. An integrity/resource interruption is incomplete, not a model failure. No new training, data expansion, parameter sweep, English generation, correctness/reasoning, product chat, softmax replacement, W8/integer lowering or release qualification occurs here. #954 stays blocked.

## Work, resources, and activated checks

Reachability: all 12,000 query decisions use the replaced inner-attention path; preserving the source's 11,900 correct decisions is the target. This tests transfer of existing behavior, not improving a 99% training metric.

Reuse four Apple Accelerate intra-op threads, one inter-op thread and one process from #1050's measured plan. CPU only; no CUDA/MPS. Gauge implementation may use bounded per-query/chunk intermediates and f64 transport before returning f32 to the unchanged outer model; bind the selected arithmetic implementation before primary scoring. Inference/scoring plus independent replay is capped at 900 seconds and 4 GiB peak RSS. Log finite batches/arms, elapsed time and result identities; if admission is unexpectedly excessive, stop with retained partial evidence.

Only activate: the new native map exporter build/export; a small synthetic adapter suite for frame semantics, causal prefix isolation, eval/state preservation and mismatch execution; source/artifact identity checks; the matched primary and conditional control; one fresh-process result replay; scoped Python/Rust formatting and claim-wording/diff checks; independent review of this changed path. Reuse existing source reproduction, frame/group, #1055/#1057 and old QA evidence. No full workspace/BDD/fuzz/WASM/audit/conformance campaign. Queue checks remain transport acknowledgements.

Implementation and run binding must be committed and posted here before fitted-model scoring. Completed evidence will be appended; no inherited result is rewritten.

## Definition of done

- [ ] Native parent/prerequisite links and active assignment verified.
- [ ] Exact source and full vocabulary frame mapping bound; reusable inference adapter implemented.
- [ ] Declared focused mechanics checks and review completed.
- [ ] Matched inference result, conditional control status, resource/work ledger and independent replay recorded.
- [ ] Both preserved source checkpoints intact; no optimizer updates or historical module changes.
- [ ] Current summaries updated, protected PR merged, outcome posted to this issue and #973, issue closed with exact scope.

## Pre-inference freeze — 2026-09-02

The implementation and unchanged predecessor were bound before any fitted-model
forward. The native exporter built with `cargo build --release --locked -p
uor-r4-core --bin r4-zoology-frame-export`. Its new export validates all 8,192
token leaves and three Rust-produced cumulative prefix witnesses. All eight
focused synthetic checks passed, including the native-export check with its
fixture explicitly present. Scoped Ruff/rustfmt, claim wording, and diff checks
passed; independent source review returned GO. No broad QA was activated.

The [frozen preparation](r4_zoology_coherent_inference_1059_preparation.json) binds
153 implementation files, including transitive local Python imports, native
Cargo dependencies, the local arrayref patch, tests, locks, and notices.

- Preparation CID: `blake3:bed7eae03c7f3bfa7e2b5ff3786f87d878f42c9eb5d8465b5e37322073cdd588`.
- Implementation tree CID: `blake3:f8facc495ee96e58029f7c15e3117552963f3dc08cb478b4e542144b0aca8b89`.
- Native frame bundle CID: `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.
- Token map artifact CID: `blake3:05969107d1accd6f54bcf0e49a48f68815a00f7273e23a022199f34db828f7f4`.
- Source model remains `blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9`.

Primary inference, conditional transport control, and fresh-process replay are
`NOT_RUN` at this freeze. The published 900-second combined budget and thresholds
above remain unchanged. Future-read accounting refers specifically to per-query
transported support; QKV projection and pointwise frame encoding are vectorized
across the input. Full token coverage does not imply all 120 H4 frames are reached.

## Completed inference and independent replay — 2026-09-02

**Terminal: `R4_INTEGRATION_PRESERVED`.** The implementation frozen at
`800e7d52` ran unchanged. The first and only primary run passed every declared
criterion; the conditional transport control ran and met its separately frozen
strong-sensitivity criterion. A fresh process reproduced the complete inference
evidence, including output/attention digests, metrics, work, and reached frames.

| Measure | Plain source | Coherent R4 | Inconsistent source transport |
| --- | ---: | ---: | ---: |
| Correct / query decisions | 11,900 / 12,000 | 11,900 / 12,000 | 2,307 / 12,000 |
| Recall | 99.1666666667% | 99.1666666667% | 19.225% |
| Mean NLL (nats) | 0.051246106465657554 | 0.051246097564697266 | 7.164478190104167 |
| Top-1 changes versus plain | 0 | 0 | 9,681 |

R4/plain maximum selected-logit difference was `2.002716064453125e-05`
against the unchanged `0.005` engineering envelope. Maximum attention-weight
difference was `8.940696716308594e-07` against `1e-5`; mean-NLL absolute
difference was `8.900960288271698e-09` against `1e-5`. All 12,000 predictions
were identical, including the same 100 incorrect decisions. Historical #1050
correct-count reproduction passed; new canonical-order digests were used as
predeclared. No historical shuffled-order logit identity is claimed.

The destructive control lost `79.94166666666666` percentage points, exceeding
the predeclared 50-point sensitivity threshold. Its 12,000 decisions, causal
support, zero future attention weights, and all work counters matched coherent
R4 except the intervention's changed-frame counters. This is evidence of
sensitivity to deliberately inconsistent transport, not of geometric superiority.

All 8,192 token leaves were available; inference reached exactly 24 frame
indices, recorded in the raw result. The native sidecar validates all 120 H4
matrices. Zero per-query future-source transport reads were recorded; input-wide
pointwise QKV/frame encoding is not counted as a query reading a future source.
The stock dense plain path materialized future score slots and masked them; its
physical future-position-read count remains unknown rather than zero.

Learned tensor state and tied-head identity were unchanged before/after
inference. Optimizer updates, training tensor values loaded, checkpoint reads, model
label arguments, and #1057 model reads were zero. The exact source artifact and
all implementation/native-frame bindings revalidated after each process.

The run took `13.778159 s`, replay `13.707552 s`,
combined `27.485712 s` against 900 s. Peak observed RSS
was `1980219392` bytes (`1.844223 GiB`)
against 4 GiB. Execution used one CPU process, four Apple Accelerate intra-op
threads and one inter-op thread; Python 3.12.14 / PyTorch 2.7.1. Timing covers
the campaign and replay, not earlier build/export/setup, and is not an isolated
plain-versus-R4 speed comparison.

### Durable evidence

- [Frozen preparation](r4_zoology_coherent_inference_1059_preparation.json):
  `blake3:bed7eae03c7f3bfa7e2b5ff3786f87d878f42c9eb5d8465b5e37322073cdd588`.
- [Raw result](r4_zoology_coherent_inference_1059_result.json):
  `blake3:bdf5a440562bf31a6c0d6d53cef0454270638b87508f0a758aaf9eb3a0031f7d`.
- Inference evidence CID: `blake3:0b0d6f61ccd1e97402fa67f60e1eaf5eb6ec4daf0583814e0270902177f69727`.
- [Fresh-process replay](r4_zoology_coherent_inference_1059_replay.json):
  `blake3:458f6f8817203e57089580d851971d7d32234c5d9e4edf96967984097bd7f181`.

The three JSON files are exact copies of the create-once local records. Source
weights/data remain in the preserved `.uor-models/research/issue-1050-zoology-release-reproduction`
artifact store; no weights or training corpus are committed in this delivery.
The source Zoology test population previously guided stopping and remains open
development evidence, not an independently sealed generalization result.

### Decision and next action

Preserve the adapter and both learned artifacts. This completes the requested
#1050 inference integration without a fit or checkpoint selection. The next
recommended separately scoped step is to apply this adapter unchanged to #1057's
final block-40 model on its original exact-data population, comparing all 8,192
query decisions against its retained 8,071-correct reference with a newly declared
inference control. Its original unrun control status stays historical.

Defer geometry expansion. In this coherent adapter, source encoding, transport,
and output decoding cancel as coordinate changes in exact arithmetic. Additional
reachable frames therefore do not by themselves add learned capacity. A later
expansion needs a named representation, memory, update-law, or score change and
a matched capacity comparison. No new training, English-context fit, generation,
#954 correctness, reasoning, exact/table lowering, or product-readiness claim
follows from this result.

Eight focused synthetic checks (including native export), the native exporter
build/export, scoped formatting, claim wording, and independent code review
passed before inference. Broad workspace/BDD/WASM/fuzz/audit/conformance tests
remained dormant as declared; required queue jobs acknowledge delivery and are
not additional model or product QA evidence.
