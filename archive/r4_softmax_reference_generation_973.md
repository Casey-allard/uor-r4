# R4/Spin source-softmax reference generation (#973)

Date: 2026-08-30 (EDT)

Evidence status: **PASS**

- Eight-token G0-P1 canary: **PASS — exact decision replay observed across two runs**.
- Five-prompt quality: **PASS machine audit in both passes; 4/5 PASS under the
  frozen smoke rubric in each pass**. G0-P3 remains the honest failure because
  it substitutes sunset for the requested morning.
- Five-prompt replay: **PASS — 5/5 reports were exactly equal after deleting
  only timing**.
- Terminal:
  **`PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`**.
- Release, hosted-page, and transformerless-runtime promotion: **NOT AUTHORIZED** by
  this bounded result.

This is an append-only evidence record. The local raw JSON reports remain under
`.uor-models/research/issue-973-r4-softmax-reference-generation/`; they are not
copied into the repository. The compact, committed evidence surface is
[`r4_softmax_reference_generation_attempt_01_result_973.json`](r4_softmax_reference_generation_attempt_01_result_973.json).
The pass-one observations remain unchanged below; the completed replay is
recorded in the dated append after the original evidence-freeze snapshot.

## What executed

`R4SoftmaxReferenceGeneratorV1` ran the pinned SmolLM2 source decoder through
the all-layer R4/Spin transport seam implemented in
[`r4_softmax_reference_generation.rs`](../src/r4_softmax_reference_generation.rs).
The observed policy was:

- learned Q/K from the checkpoint and unchanged scaled dot-product scoring in
  the query gauge;
- unchanged stable causal softmax over the complete prefix;
- unchanged weighted value aggregation in the query gauge;
- compiler-side R4/Spin gauge decoding before `Wo`;
- all 30 decoder layers selected;
- deterministic greedy argmax, with the lower token ID winning an exact tie;
- EOS token `2`, with a safety stop after three repetitions of a period-1
  through period-4 tail.

The attention-transport policy CID was
`blake3:80ab8728579f65423be19286c32649c1000af463c4d4801b16eaa97222901a54`.

### Provenance boundary

HELM-D is credited as the MIT-licensed architectural reference at upstream
commit `7501deca8f413848bfef804be64ce874b72a3cd7`. That credit applies to the
causal geometric-attention/connection-walk design source. No HELM checkpoint,
HELM evaluation result, or HELM generation implementation executed here.

The language model that executed was the existing UOR Rust source-model stack
loaded from `HuggingFaceTB/SmolLM2-135M-Instruct`. Its pinned checkpoint owns
the embeddings, learned Q/K/V, RoPE, grouped-query layout, softmax, value
aggregation, output projections, residual blocks, normalization, feed-forward
blocks, and LM head. UOR's R4/Spin seam transports the attention coordinates;
it does not replace the checkpoint's language model or inherit HELM's reported
quality.

This distinction is material: the observed generator is a source-backed,
Transformer-compatible native reference. It executes the full floating-point
checkpoint and ordinary softmax. It is not the final source-free,
multiplication-free, `no_std`, allocation-free, table-compiled, or browser-WASM
runtime.

## Frozen source and decode contract

- Source revision:
  `7e27bd9f95328f0f3b08261d1252705110c806f8`
- Weights CID:
  `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- Tokenizer: `hf-byte-bpe/1`
- Tokenizer CID:
  `blake3:944d1262d516abd56a8156dd3058a73a1bf3dc19419527592d854d162f288073`
- Tokenizer adapter digest:
  `blake3:1a6ab67d2145f8f96989f529f787fc74b59952ea1be6739b612f041a15f00b5e`
- Added-token digest:
  `blake3:a70b9ca46cf5361989931a623c188fba70c5e189cfe8be55cf62f9faca81643e`
- Chat-template CID:
  `blake3:2ff3b438de276e44f744a575e323cd7c32a47e72d91baf30d965df0c307af35b`
- Model shape excluding the per-prompt sequence capacity: dimension `576`,
  hidden dimension `1536`, 30 layers, 9 query heads, 3 KV heads, head size
  `64`, vocabulary `49152`.
- Projection owner: `uor-matmul exact GEMM` at revision
  `b13c98449948174f590e337c4dc25dfc394a07d0` on `aarch64-macos`.
  Available reported backends were `portable`, `neon`, and `neondotprod`; the
  private selected backend was `UNAVAILABLE` because the dependency does not
  expose that cache.
- Canary: eight generated-token decisions, eight requested/effective workers.
- Five-prompt pass: at most 32 generated-token decisions, four
  requested/effective workers; EOS or a short-cycle safety condition may stop
  a response earlier.

The local command made no Ollama, hosted-provider, or network inference call.

## Frozen operator rubric

The review rubric is unchanged from
[`geometric_decoder_spike_950.md`](geometric_decoder_spike_950.md):

- **Grammar:** PASS when the rendered response contains an intelligible English
  clause or structured list item without corruption or a repetition loop. A
  cap-truncated final clause is reviewed separately.
- **Prompt responsiveness:** PASS when the response directly performs the
  requested explanation, tips, description, or welcome rather than changing
  topic or emitting only boilerplate.
- **Truncation:** the 32-token cap may truncate the last clause; earlier
  complete responsive material remains reviewable, but an otherwise
  fragmentary response fails.
- **Threshold:** at least four of five prompts must pass both grammar and
  responsiveness. The rubric is a bounded smoke criterion, not a claim of
  instruction-following completeness or general model quality.

## Eight-token canary and replay

Prompt: `Explain in three short sentences why plants need sunlight.`

Prompt token IDs (40):

```text
[1, 9690, 198, 2683, 359, 253, 5356, 5646, 11173, 3365, 3511, 308, 34519, 28, 7018, 411, 407, 19712, 8182, 2, 198, 1, 4093, 198, 36971, 281, 1296, 1890, 8545, 1701, 2109, 737, 8118, 30, 2, 198, 1, 520, 9531, 198]
```

Both runs generated the same token IDs:

```text
[34246, 737, 8118, 288, 11205, 19173, 28, 253]
```

Both decoded exactly:

> Plants need sunlight to undergo photosynthesis, a

Both stopped at the eight-token cap with valid UTF-8 and no detected short
cycle. Their stable decision CID and persistent-state CID matched:

- decision CID:
  `blake3:e1843e45e62ab9c5872a26e808e43449841350aaef7c848d80a5a5ef9fbabbd2`
- persistent-state CID:
  `blake3:438436331a5030cb04438eb791baed8313b245e0fc2de4943bf20a034a5feacd`

The decision CID excludes local paths and timing. Its equality therefore binds
the stable source/tokenizer/policy, prompt and token IDs, stop reason, state,
and complete recorded audits while allowing wall-clock fields to differ.

| Run | Positions | Layer calls | Head calls | Causal K/V transports each | R4 blocks encoded | R4 K/V blocks each | R4 outputs | Workers req/eff/max | Forwards | Matrices | Tiles | Output cells | Scalar terms | Load s | Generation s | Total s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| canary run 1 | 47 | 1,410 | 12,690 | 304,560 | 9,948,960 | 4,872,960 | 203,040 | 8/8/8 | 47 | 9,917 | 317,344 | 9,619,584 | 6,320,553,984 | 0.359285416 | 185.485380875 | 185.958830917 |
| canary run 2 | 47 | 1,410 | 12,690 | 304,560 | 9,948,960 | 4,872,960 | 203,040 | 8/8/8 | 47 | 9,917 | 317,344 | 9,619,584 | 6,320,553,984 | 0.346491333 | 171.704796750 | 172.134237708 |

For each canary run, the projection audit recorded 1,410 hooks, 12,690 query
vectors, 4,230 key vectors, 4,230 value vectors, and
`812,160 / 270,720 / 270,720` query/key/value lanes. The causal, projection,
and R4 observed records equaled their harness-computed expected records;
`all_layers_selected`, all three `*_audit_exact` fields, and
`zero_future_reads` were `true`. Causal future reads, R4 future-position reads,
and source-frame permutations were all zero. Each run observed eight active
workers on every one of its 47 physical forwards. Workspace growth was 32
events and 393,216 bytes per run.

Canary verdict: **PASS_REPLAY_EXACT** within this bounded source-backed
reference contract. This does not qualify the five-prompt gate.

## Five-prompt pass one: decoded behavior

| ID | Exact prompt | Retained response | Grammar | Responsive | Result |
| --- | --- | --- | --- | --- | --- |
| G0-P1 | Explain in three short sentences why plants need sunlight. | “Plants need sunlight to undergo photosynthesis, a process that converts light energy into chemical energy stored in glucose, which they use for growth and development.” | PASS | PASS: directly explains why sunlight is needed; EOS ended the response before the cap | PASS |
| G0-P2 | Give three practical tips for staying organized at work, with one brief explanation for each. | “Here are three practical tips for staying organized at work: 1. **Create a To-Do List:** Start by setting aside a few minutes each day to” | PASS: structured first item; final clause is cap-truncated | PASS: directly begins the requested work-organization tips | PASS |
| G0-P3 | Describe a rainy city morning in three vivid sentences. | “As the sun began to set, casting a warm orange glow over the bustling streets of the city, a gentle rain poured down in a relentless stream, washing away” | PASS: intelligible clause; cap-truncated | FAIL: substitutes sunset for the requested morning | FAIL |
| G0-P4 | Explain in simple terms how a bicycle stays balanced while moving. | “When you ride a bicycle, you're essentially using a system of gears and levers to move your body. The bicycle's balance is achieved through a combination of the” | PASS: one complete sentence before truncation | PASS: directly begins a simple balance explanation | PASS |
| G0-P5 | Write a friendly three-sentence welcome message for a new teammate and mention collaboration. | `"Hi [TeamName], welcome to our team. We're excited to have you on board and look forward to working together to achieve our goals. We value` | PASS: two complete sentences before truncation | PASS: friendly welcome and “working together” satisfy the collaboration request | PASS |

Pass-one operator review: **4/5 PASS**. All five outputs were byte-decodable,
all five generated-token sequences were distinct, and none entered a detected
period-1 through period-4 cycle. G0-P1 stopped on EOS after 29 generated tokens;
the other four stopped at the 32-token cap.

### Exact token and content identities

#### G0-P1

- Prompt token IDs (40):
  `[1, 9690, 198, 2683, 359, 253, 5356, 5646, 11173, 3365, 3511, 308, 34519, 28, 7018, 411, 407, 19712, 8182, 2, 198, 1, 4093, 198, 36971, 281, 1296, 1890, 8545, 1701, 2109, 737, 8118, 30, 2, 198, 1, 520, 9531, 198]`
- Generated token IDs (29):
  `[34246, 737, 8118, 288, 11205, 19173, 28, 253, 980, 338, 18065, 1420, 1439, 618, 2819, 1439, 6314, 281, 8102, 28, 527, 502, 722, 327, 2063, 284, 1421, 30, 2]`
- Decision CID:
  `blake3:e6215af1dfd88cc8b2b8970672a84281535ad42ca0da383c35b87a50f3146bfb`
- Persistent-state CID:
  `blake3:7acfbb05a4d58d25cda102f7c48ed8c9ed9d4a5068a156e694959e359b2d0c8c`
- Stop: `eos`; sequence capacity: `72`.

#### G0-P2

- Prompt token IDs (47):
  `[1, 9690, 198, 2683, 359, 253, 5356, 5646, 11173, 3365, 3511, 308, 34519, 28, 7018, 411, 407, 19712, 8182, 2, 198, 1, 4093, 198, 26533, 1296, 4786, 5608, 327, 9286, 6554, 418, 746, 28, 351, 582, 5453, 7718, 327, 971, 30, 2, 198, 1, 520, 9531, 198]`
- Generated token IDs (32):
  `[4590, 359, 1296, 4786, 5608, 327, 9286, 6554, 418, 746, 42, 1116, 33, 30, 1903, 11870, 253, 1626, 29, 6248, 5516, 3967, 7734, 411, 4054, 9792, 253, 1443, 3487, 971, 1194, 288]`
- Decision CID:
  `blake3:ed7aacf69172e8a6cd05f78ead59c616e22c05b591ab6586a86c77ec9946b086`
- Persistent-state CID:
  `blake3:53e421590aa5b849ed6d1d3c07277ede3ee1508b6fbfb78fe85a832a225a23eb`
- Stop: `maximum_new_tokens`; sequence capacity: `79`.

#### G0-P3

- Prompt token IDs (40):
  `[1, 9690, 198, 2683, 359, 253, 5356, 5646, 11173, 3365, 3511, 308, 34519, 28, 7018, 411, 407, 19712, 8182, 2, 198, 1, 4093, 198, 37964, 253, 24536, 2240, 5738, 281, 1296, 12999, 8545, 30, 2, 198, 1, 520, 9531, 198]`
- Generated token IDs (32):
  `[1653, 260, 2388, 2585, 288, 932, 28, 17462, 253, 3091, 10245, 14654, 690, 260, 14055, 9018, 282, 260, 2240, 28, 253, 9154, 5249, 21620, 1187, 281, 253, 29092, 5390, 28, 11789, 2025]`
- Decision CID:
  `blake3:4b0ffa503cd7ab270adaf9228e5d88e4faeada194adf4231b9ad0f7a342832de`
- Persistent-state CID:
  `blake3:2d15878b796350c5bfd9d9c29867de99b3cf4e974fda774cf331df0f147b612b`
- Stop: `maximum_new_tokens`; sequence capacity: `72`.

#### G0-P4

- Prompt token IDs (42):
  `[1, 9690, 198, 2683, 359, 253, 5356, 5646, 11173, 3365, 3511, 308, 34519, 28, 7018, 411, 407, 19712, 8182, 2, 198, 1, 4093, 198, 36971, 281, 2232, 2656, 638, 253, 19018, 13895, 8609, 979, 4138, 30, 2, 198, 1, 520, 9531, 198]`
- Generated token IDs (32):
  `[2427, 346, 11031, 253, 19018, 28, 346, 2316, 8009, 1015, 253, 817, 282, 25602, 284, 47726, 288, 1485, 469, 1248, 30, 378, 19018, 506, 3630, 314, 6551, 738, 253, 4925, 282, 260]`
- Decision CID:
  `blake3:582e185f6f542434e310f3e5af90ea5cdd4093be6f9c2f82465f5d453f31fefd`
- Persistent-state CID:
  `blake3:e0c990c89eb7946869ddede6cfaaecbe898e8da090f8bedcc175fc9327ee2afa`
- Stop: `maximum_new_tokens`; sequence capacity: `74`.

#### G0-P5

- Prompt token IDs (47):
  `[1, 9690, 198, 2683, 359, 253, 5356, 5646, 11173, 3365, 3511, 308, 34519, 28, 7018, 411, 407, 19712, 8182, 2, 198, 1, 4093, 198, 19161, 253, 7952, 1296, 29, 28205, 10668, 3714, 327, 253, 725, 31500, 368, 284, 3311, 5339, 30, 2, 198, 1, 520, 9531, 198]`
- Generated token IDs (32):
  `[18, 26843, 933, 33039, 5820, 1750, 10668, 288, 653, 2299, 30, 1046, 2316, 8916, 288, 457, 346, 335, 4411, 284, 1492, 3703, 288, 1891, 1592, 288, 3025, 653, 3949, 30, 1046, 1685]`
- Decision CID:
  `blake3:60868e881494c2f7c3376d5a47cd7b9451d59d4c76ac1066152b2146250ce0c7`
- Persistent-state CID:
  `blake3:91531c938c0defa926b54807c23709689d25a5e85094b3794ecb2a5da7c4bb87`
- Stop: `maximum_new_tokens`; sequence capacity: `79`.

## Five-prompt pass-one machine audits

Observed causal records:

| ID | Positions | Layer calls | Head calls | Query transforms | Key transports | Value transports | Output transforms | Future reads | Max query | Max source |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| G0-P1 | 68 | 2,040 | 18,360 | 18,360 | 633,420 | 633,420 | 18,360 | 0 | 67 | 67 |
| G0-P2 | 78 | 2,340 | 21,060 | 21,060 | 831,870 | 831,870 | 21,060 | 0 | 77 | 77 |
| G0-P3 | 71 | 2,130 | 19,170 | 19,170 | 690,120 | 690,120 | 19,170 | 0 | 70 | 70 |
| G0-P4 | 73 | 2,190 | 19,710 | 19,710 | 729,270 | 729,270 | 19,710 | 0 | 72 | 72 |
| G0-P5 | 78 | 2,340 | 21,060 | 21,060 | 831,870 | 831,870 | 21,060 | 0 | 77 | 77 |

Observed pre-RoPE projection records:

| ID | Hooks | Query vectors | Key vectors | Value vectors | Query lanes | Key lanes | Value lanes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| G0-P1 | 2,040 | 18,360 | 6,120 | 6,120 | 1,175,040 | 391,680 | 391,680 |
| G0-P2 | 2,340 | 21,060 | 7,020 | 7,020 | 1,347,840 | 449,280 | 449,280 |
| G0-P3 | 2,130 | 19,170 | 6,390 | 6,390 | 1,226,880 | 408,960 | 408,960 |
| G0-P4 | 2,190 | 19,710 | 6,570 | 6,570 | 1,261,440 | 420,480 | 420,480 |
| G0-P5 | 2,340 | 21,060 | 7,020 | 7,020 | 1,347,840 | 449,280 | 449,280 |

Observed R4/Spin implementation records:

| ID | Positions | R4 blocks encoded | Key blocks | Value blocks | Output blocks | Future reads | Frame permutations | Causal exact | Projection exact | R4 exact | All layers | Zero future |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- | --- | --- | --- |
| G0-P1 | 68 | 20,563,200 | 10,134,720 | 10,134,720 | 293,760 | 0 | 0 | true | true | true | true | true |
| G0-P2 | 78 | 26,956,800 | 13,309,920 | 13,309,920 | 336,960 | 0 | 0 | true | true | true | true | true |
| G0-P3 | 71 | 22,390,560 | 11,041,920 | 11,041,920 | 306,720 | 0 | 0 | true | true | true | true | true |
| G0-P4 | 73 | 23,652,000 | 11,668,320 | 11,668,320 | 315,360 | 0 | 0 | true | true | true | true | true |
| G0-P5 | 78 | 26,956,800 | 13,309,920 | 13,309,920 | 336,960 | 0 | 0 | true | true | true | true | true |

For every pass-one prompt, each observed causal, projection, and R4 record
equaled its corresponding expected record in the retained report. The complete
per-position H4 frame-offset sequences remain in the local raw evidence and are
bound into each stable decision CID; they are intentionally not duplicated
here.

## Five-prompt pass-one execution work and timing

| ID | Workers req/eff/max | Multiworker forwards | Forwards | Streams start/end | Matrices | Tiles | Output cells | Scalar terms | Workspace growth events | Workspace bytes | Load s | Generation s | Total s |
| --- | --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| G0-P1 | 4/4/4 | 68 | 68 | 68/68 | 14,348 | 229,568 | 13,917,696 | 9,144,631,296 | 16 | 196,608 | 0.766246750 | 466.144972667 | 467.025546292 |
| G0-P2 | 4/4/4 | 78 | 78 | 78/78 | 16,458 | 263,328 | 15,964,416 | 10,489,430,016 | 16 | 196,608 | 0.799587541 | 509.467477042 | 510.373900459 |
| G0-P3 | 4/4/4 | 71 | 71 | 71/71 | 14,981 | 239,696 | 14,531,712 | 9,548,070,912 | 16 | 196,608 | 0.801591000 | 463.516569291 | 464.421337250 |
| G0-P4 | 4/4/4 | 73 | 73 | 73/73 | 15,403 | 246,448 | 14,941,056 | 9,817,030,656 | 16 | 196,608 | 0.778758875 | 471.233931750 | 472.113081583 |
| G0-P5 | 4/4/4 | 78 | 78 | 78/78 | 16,458 | 263,328 | 15,964,416 | 10,489,430,016 | 16 | 196,608 | 0.347397875 | 307.429277000 | 307.863770125 |

Pass-one aggregate observed work:

- five runs and 368 source forwards;
- 77,648 matrix calls and 1,242,368 completed tiles;
- 75,319,296 output cells and 49,488,592,896 scalar terms;
- 120,519,360 R4 blocks encoded;
- 59,464,800 key blocks and 59,464,800 value blocks transported;
- 1,589,760 R4 output blocks decoded;
- 3,716,550 causal key transports and the same number of value transports;
- source load time `3.493582041 s`, generation time `2217.792227750 s`, and
  summed per-command total time `2221.797635709 s`.

These timings are measurements on this host, not performance guarantees or a
comparison against another engine. The observed four-worker execution was
genuinely active (`max_active_workers = 4` and every physical forward was
multiworker), but the five prompts were separate single-stream commands rather
than one batched generation.

## Replay gate at the pass-one evidence freeze (historical)

At the original evidence freeze, the eight-token canary already had a stable
two-run replay but the complete five-prompt qualification did not: only
`gate-g0-p1-run-1.json` through `gate-g0-p5-run-1.json` existed.

Five-prompt pass-two status at that freeze: **IN_PROGRESS**.

The completion append below compares, for every prompt, the generated token
IDs, decoded response, stop reason, decision CID, persistent-state CID, causal
audit, projection audit, R4 implementation audit, all-layer selection, and
zero-future-read fields. Timing and local paths are excluded from the stable
decision identity and need not match.

At that freeze, the decision was limited to:

> `R4SoftmaxReferenceGeneratorV1` has produced five distinct, decodable,
> cycle-free source-backed outputs with 4/5 passing the frozen smoke rubric,
> while exercising the established all-layer R4/Spin causal-softmax seam with
> matching observed/expected audit records and zero future reads. Full
> five-prompt deterministic replay was not yet established.

## Replay completion append — 2026-08-30 (EDT)

Pass two completed for `gate-g0-p1-run-2.json` through
`gate-g0-p5-run-2.json`. Canonical JSON comparison of each run-1/run-2 pair
after deleting only the top-level `timing` object passed **5/5**. The prompts,
prompt token IDs, generated token IDs, decoded bytes and retained response,
stop reason, decision CID, persistent-state CID, sequence capacity, complete
causal/projection/R4 audits, and execution-work counters were therefore exactly
equal for every prompt. Local source paths were not part of the decision CID.

The frozen operator review is still **4/5 PASS**, not 5/5: deterministic replay
preserves G0-P3's responsive-content failure (sunset instead of the requested
morning) as well as the four passing outputs. Replay proves repeatability under
this contract; it does not upgrade output quality.

### Pass-two work and timing

Because the execution-work records replayed exactly, the non-timing columns are
the pass-one values already tabulated above. Pass-two timing was:

| ID | Workers req/eff/max | Multiworker forwards | Forwards | Matrices | Tiles | Output cells | Scalar terms | Load s | Generation s | Total s |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| G0-P1 | 4/4/4 | 68 | 68 | 14,348 | 229,568 | 13,917,696 | 9,144,631,296 | 0.938704334 | 485.448713292 | 486.512302333 |
| G0-P2 | 4/4/4 | 78 | 78 | 16,458 | 263,328 | 15,964,416 | 10,489,430,016 | 0.962291750 | 530.161774500 | 531.242926292 |
| G0-P3 | 4/4/4 | 71 | 71 | 14,981 | 239,696 | 14,531,712 | 9,548,070,912 | 0.651839041 | 507.195417541 | 507.958285250 |
| G0-P4 | 4/4/4 | 73 | 73 | 15,403 | 246,448 | 14,941,056 | 9,817,030,656 | 0.637620166 | 517.248135625 | 517.991384125 |
| G0-P5 | 4/4/4 | 78 | 78 | 16,458 | 263,328 | 15,964,416 | 10,489,430,016 | 0.494016500 | 334.340869125 | 334.944143167 |

Pass-two aggregate source-load time was `3.684471791 s`, generation time was
`2374.394910083 s`, and summed per-command total time was
`2378.649041167 s`. Both passes executed 368 source forwards, all of them
multiworker, 77,648 matrix calls, 1,242,368 tiles, 75,319,296 output cells,
and 49,488,592,896 scalar terms. Each pass recorded 80 workspace-growth events
and 983,040 bytes across the five separately launched commands.

### Exact causal, projection, and R4 replay

Every pass-two audit counter equals the corresponding pass-one counter in the
three tables above. Per pass, the five prompts accumulated:

- 368 executed positions, 11,040 layer calls, 99,360 head/query/output
  transforms, and 3,716,550 causal key transports plus 3,716,550 value
  transports;
- 11,040 pre-RoPE projection hooks;
- 120,519,360 R4 blocks encoded, 59,464,800 key blocks transported,
  59,464,800 value blocks transported, and 1,589,760 output blocks decoded;
- zero causal future reads, zero R4 future-position reads, and zero
  source-frame permutations.

For all ten qualification reports, observed causal records equaled expected
causal records, observed projection records equaled expected projection
records, and observed R4 implementation records equaled expected R4 records.
`causal_audit_exact`, `projection_audit_exact`, `r4_audit_exact`,
`all_layers_selected`, and `zero_future_reads` were all `true`.

### Ordinary-donor comparison

The generated behavior also reproduces the frozen source-control donor in
[`geometric_decoder_spike_950_raw.json`](geometric_decoder_spike_950_raw.json).
G0-P1 matches the donor response and its generated-token prefix through EOS:
the reference stops after 29 tokens at EOS offset 28, while the older donor
artifact deliberately retained 32 token IDs, including three post-EOS tokens,
under its fixed-length transcript policy. G0-P2 through G0-P5 each match the
donor's complete 32-token sequence and retained response exactly.

### Evidence packaging and terminal

The compact aggregate
[`r4_softmax_reference_generation_attempt_01_result_973.json`](r4_softmax_reference_generation_attempt_01_result_973.json)
contains both passes' exact output/token identities, stops, decision and
persistent-state CIDs, audit and execution counters, timings, raw-report
BLAKE3 hashes, frozen review, and provenance/nonclaim boundaries. The twelve
full reports remain local and are not copied into Git.

Final bounded decision:

> `R4SoftmaxReferenceGeneratorV1` exactly reproduces the frozen source donor's
> retained responses under an all-layer R4/Spin gauge transport around ordinary
> causal dot-product/softmax attention. Both the eight-token canary and the
> five-prompt gate replay exactly after timing is removed; all recorded causal,
> projection, and R4 audits are exact, and future reads are zero. The frozen
> decoded-quality rubric remains 4/5. This establishes the source-backed native
> reference bridge only, not a transformerless runtime or geometry advantage.

Terminal:
**`PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`**.

## Nonclaims

The retained reports declare, and this record preserves, all of the following:

- not source-free or transformerless; the full pinned source decoder executes;
- not multiplication-free, `no_std`, allocation-free, browser-WASM, or
  compiled-runtime inference;
- no geometry advantage and no replacement of dot-product/softmax;
- this bounded five-prompt smoke does not establish general text quality,
  meaning, reasoning, or release readiness.

The individual raw reports retain their per-run one-sample wording; this
aggregate record states the scope of the complete five-prompt gate. That gate
does not erase the architectural and product nonclaims. In particular,
the result does not authorize a release tag, a public “frontier model” claim,
or presentation of the static webpage as running this native source-backed
decoder.
