# #1061 — A1Q-H15: preserve exact-data associative recall through coherent R4 inference

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T19:31:33Z
- **Updated:** 2026-09-02T19:49:34Z
- **Closed:** 2026-09-02T19:48:47Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1061
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Preserve the exact-data checkpoint through coherent R4 inference

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

- [x] Live parent/prerequisite relationships and assignment verified.
- [x] Exact source/data and unchanged #1059 adapter/frame implementation bound.
- [x] Focused new wiring checks and independent prereveal review pass.
- [x] Matched inference, control status, resources/work and fresh-process replay recorded.
- [x] Original artifacts and historical modules remain unchanged; zero optimizer updates.
- [x] Current direction updated; protected PR merged; issue and parent outcome recorded.


## Comments (2)

### comment 5515318730 — 2026-09-02T19:41:28Z — Casey-allard

The implementation is frozen at `42d5eb34`; its preparation and full run binding are committed on `issue-1061-exact-r4-inference` before any fitted-model forward.

- Preparation: `blake3:c8e97664f7feab8c83ad15d298620da675bc3f156a9b0dcfcfa98ac69fad6c35`.
- Complete 188-file implementation tree: `blake3:d9272cbf0fd01535cb6b1aa6c6baaee3bdc59a312b0ea66674ccf307328431e6`.
- Exact retained #1057 model: `blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445`.
- Reused native frame bundle: `blake3:94762441a43b03f596a66131ec34af15bba3afbc2bbc5d28ab7dfdabd9b6d68c`.

Six new scoped checks passed; unchanged source/data/native identities, Ruff, claim wording, diff hygiene and independent prereveal source review passed. #1059 adapter/scorer/native evidence was reused. No inherited tests, native rebuild/export, CPU timing matrix, or fitted inference ran during preparation.

Launching the one canonical 1024-row matched plain/R4 comparison, its conditional inconsistent-transport control, and one fresh-process replay. The reference is 8071/8192 correct with all8192 plain/R4 predictions identical and the published numerical tolerances. Budget remains 900 seconds combined,4GiB peak RSS,4 CPU/Apple Accelerate threads. No training or checkpoint/optimizer/RNG/physical-binding-control reads are authorized.


### comment 5515423623 — 2026-09-02T19:49:34Z — Casey-allard

Completed through protected PR #1062 at `62c64ccf2cba7674e27628775e78fb81c44baee0`. Main's file tree matches the reviewed head.

**`EXACT_DATA_R4_PRESERVED`:** plain and R4 both 8,071/8,192 correct, all prediction IDs identical. Maximum logit difference `2.5272369384765625e-05`, attention difference `2.2351741790771484e-06`, NLL difference `7.450580596923828e-09`. The new inconsistent-transport control scored 1,009/8,192, losing 86.2061 percentage points with matched causal support/work. All frozen criteria passed.

Exact fresh-process replay passed. Run/replay took 33.794678 seconds combined, peak 2,588,475,392 bytes (2.411 GiB), on four CPU threads. Learned state/tied weights remained unchanged; zero training or checkpoint/optimizer/RNG/old physical-binding-control reads occurred. The source code stayed unchanged after the published freeze.

Six new focused wiring/decision checks, unchanged source/data/frame/code bindings, matched inference/control, replay, scoped formatting/claim wording/diff checks, and independent source/evidence review passed. Historical #1059 mechanics evidence was reused; broad QA stayed dormant. Required protected queue jobs acknowledged delivery.

Result `blake3:ac2ec4d533ac47d25f8eb9dfd7a41147147d73c0e2d9531352d9f9fb2eb84e58`; evidence `blake3:a14e0e6fe0f915acc9f81b446a0ee6da7d3e97723f60c269125de55120399802`; replay `blake3:af6c239ec2d0e11f26f50f74150c992dea345ec21257141fcec1096a573e708e`. The three committed JSON records are byte-identical to their create-once local originals.

All DoD items are complete. Repository summaries, #973/#820, and milestone 21 now identify this result and the next separately frozen supplied-context English binding curriculum. #973 stays open and #954 blocked. #1057's historical terminal and unrun control remain unchanged. This closes the inference transfer without claiming English capability, geometric superiority, softmax removal, or exact/table runtime.



---
_Comment/review ids in this file: 5515318730, 5515423623_

