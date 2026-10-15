# #1059 — A1Q-H14: preserve qualified associative recall through coherent R4 inference

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T19:01:40Z
- **Updated:** 2026-09-02T19:25:18Z
- **Closed:** 2026-09-02T19:23:38Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1059
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Preserve qualified associative recall through coherent R4 inference

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

- [x] Native parent/prerequisite links and active assignment verified.
- [x] Exact source and full vocabulary frame mapping bound; reusable inference adapter implemented.
- [x] Declared focused mechanics checks and review completed.
- [x] Matched inference result, conditional control status, resource/work ledger and independent replay recorded.
- [x] Both preserved source checkpoints intact; no optimizer updates or historical module changes.
- [x] Current summaries updated, protected PR merged, outcome posted to this issue and #973, issue closed with exact scope.


## Comments (2)

### comment 5514984309 — 2026-09-02T19:15:03Z — Casey-allard

Pre-inference freeze is committed at **800e7d52** and pushed on `issue-1059-zoology-r4-inference`. No fitted-model forward has run in this issue.

- Preparation: `blake3:bed7eae03c7f3bfa7e2b5ff3786f87d878f42c9eb5d8465b5e37322073cdd588`.
- Complete implementation tree (153 files): `blake3:f8facc495ee96e58029f7c15e3117552963f3dc08cb478b4e542144b0aca8b89`.
- Native frame bundle: `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`; map artifact: `blake3:05969107d1accd6f54bcf0e49a48f68815a00f7273e23a022199f34db828f7f4`.
- Exact #1050 model remains `blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9`.

The native exporter built successfully; all eight scoped synthetic checks passed, including full8192-token export validation and three native prefix witnesses. Scoped Ruff/rustfmt, claim wording, and diff checks passed. Independent prereveal source review returned GO. No broad QA was activated.

Launching the one canonical3000-row paired plain/R4 inference, its conditional source-frame transport control, and one fresh-process replay under the published **900seconds combined /4GiB RSS /4 CPU threads** contract. Thresholds and source identities are unchanged. No optimizer or #1057 model read occurs. Result evidence will be appended after completion.


### comment 5515110919 — 2026-09-02T19:25:18Z — Casey-allard

Completed through protected PR #1060 at `83e1e90d9fae8e92457e32f16223996f5389f4ed`. Main's tree was checked against the reviewed delivery.

**`R4_INTEGRATION_PRESERVED`:** both plain/R4 11,900/12,000 correct, every top-1 identical; max logit `2.002716064453125e-05`, max attention `8.940696716308594e-07`, NLL delta `8.900960288271698e-09`. Inconsistent transport scored 2,307/12,000 (19.225%), losing 79.9417 percentage points at equal causal support/work. Learned state/tied weights remain unchanged; optimizer updates and #1057 artifact reads were zero.

Eight scoped synthetic checks, native build/export, frozen source/frame/implementation identities, matched inference/control, exact fresh-process replay, scoped formatting/claim wording/diff checks and independent code/evidence review passed. Broad QA remained dormant. The protected merge-group workflow completed successfully with its declared acknowledgement jobs.

Combined CPU run/replay: 27.485712 seconds; peak 1,980,219,392 bytes (1.844 GiB). Result `blake3:bdf5a440562bf31a6c0d6d53cef0454270638b87508f0a758aaf9eb3a0031f7d`; replay `blake3:458f6f8817203e57089580d851971d7d32234c5d9e4edf96967984097bd7f181`. All three committed preparation/result/replay JSON copies match the create-once local records byte for byte.

All DoD items are satisfied. Repository summaries, #973/#820 and milestone 21 now name the retained result and the next recommendation: inference-only transfer of this adapter to #1057's original exact-data artifact/population. #973 remains open and #954 blocked. This closes only the #1059 integration; it does not establish geometric superiority, English context binding, generation, or table-native runtime.



---
_Comment/review ids in this file: 5514984309, 5515110919_

