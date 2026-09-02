# #1039 bounded local generation reference surface

- **Issue:** [#1039](https://github.com/UOR-Foundation/uor-r4/issues/1039)
- **Date:** 2026-09-01
- **Scope:** frozen #1017 checkpoint; native CPU execution; raw single-turn
  local generation
- **Result:** `POSITIVE_REFERENCE_SURFACE_GATE`

This record adds a small access surface around the already-qualified #1017
generator. It does not change the checkpoint, sampler, attention mechanism,
training, or geometric programme. The model remains a bounded local generation
prototype rather than the source-free route-native product planned in #962.

## Frozen decision gate

The fresh prompt was committed before execution and was not one of #1017's five
qualification prompts:

> Mira found a small brass key beneath the old pear tree.

The invocation requested 24 new tokens with stable seed `1039`, attention
enabled, four exact-executor workers, and the opt-in Apple Accelerate release backend. The
gate required nonempty valid UTF-8, no immediate period-one through period-four
cycle, exact fixed-seed output/token/audit replay, all-layer attention auditing,
zero future reads, and zero provider or Ollama calls.

The two runs decoded the same continuation:

> She picked it up and said, “I believe it can be a big tree!”
> Mira and her mommy

Measured wall times were `0.22 s` and `0.17 s`. After removing timing, the two
complete JSON reports were byte-identical. The bounded gate therefore passed
and authorized only the dedicated loopback reference seam described below.

## Evidence identities

The immutable local export supplied to both runs was:

- export-manifest CID:
  `blake3:77d5735ccfb4f2ac8a89f2f42a7ad8663b96770ea23a0b4bfae87b3daea7d8f3`
- export-tree CID:
  `blake3:4819f8cbb6e673c4124eaa61e319b42adc54b1de178969916df035aad65a4000`
- weights CID:
  `blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d`
- tokenizer CID:
  `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`
- config CID:
  `blake3:1f1ddb6de22f5c81c04d3093eeff8e0991d63b79ee33bc8ff3cf7c68ef0a9497`
- training-result CID:
  `blake3:7d8e859d30729971c6428a11fdbc4db07890c663a13a1425ae9998efad56ef69`

The replayed generation evidence was:

- decision CID:
  `blake3:86ce45d07684f8abead1e4faca2346024ef7b01e99f9b0e3b51188deafcde61b`
- output CID:
  `blake3:87150ebc68ed1e3902f6e0c9937f7a642f234ac2a3644cc33262cb081715ae49`
- attention-audit CID:
  `blake3:3e7d642eb20c9e1d05c385df2802d120becb91ba88df16a9b61cf2569caec010`
- persistent-state CID:
  `blake3:ff69922ed27437e308852b053f3b3b15baebeb1e5a51c27c704f18f1ba423793`

All six layers were selected. The causal, projection, R4, policy, and
source-read audits passed; future, provider, and Ollama reads were zero. The
selected backend was Apple Accelerate and the effective exact-executor worker
count was four. Accelerate is a CPU-native local inference backend here and
owns its internal CPU scheduling; the worker setting is not a BLAS thread cap.
CUDA and external GPU execution are not part of this surface.

## Native HTTP reference seam

The dedicated endpoint is disabled by default and must bind to a loopback host.
It uses the existing #1017 generation implementation under a process-wide
single-flight reservation; it is not a second inference engine.

The required local export is not downloaded or reconstructed by the server. An
operator supplies it explicitly and owns the fixed exact-executor worker count
(defaulting to up to four, capped by the process's available parallelism).
Preflight admits
exactly the five frozen export files; additional loader inputs, including a
Safetensors shard index, fail closed:

```bash
cargo build --release --offline --features local-inference-accelerate --bin r4
target/release/r4 --host 127.0.0.1 --port 8000 serve \
  --enable-r4-softmax-local \
  --r4-softmax-local-model .uor-models/research/issue-1017/export \
  --r4-softmax-local-workers 4
```

Generate one bounded raw continuation:

```bash
curl --fail-with-body http://127.0.0.1:8000/uor/v1/r4-softmax-local/generate \
  -H 'content-type: application/json' \
  --data '{"prompt":"Mira found a small brass key beneath the old pear tree.","max_tokens":8}'
```

`max_tokens` defaults to 8 and is capped at 32. Requests cannot select a model,
backend, or exact-executor worker count. The response schema is
`uor-r4.r4-softmax-local-http/1`. A successful response contains `schema`,
`generator`, `claim_scope`, checkpoint identities and exact-backend provenance,
`model_shape`, `input_tokens`, `generated_token_ids`, `response_text`,
`stop_reason`, decision/generation-policy/output/audit/persistent-state CIDs,
compact attention/output-policy/decode/source-read audits, `execution`,
`timing`, and `nonclaims`. It does not reflect the prompt, prompt token IDs,
local model path, or filenames. `GET /api/sysinfo` reports
`r4_softmax_local.checkpoint_preflight_ready`. Errors use
`{"error":{"type":"r4_softmax_local_error","code":"...","message":"..."}}`.

This is a dedicated raw single-turn research endpoint. It is not
`/v1/chat/completions`, does not implement chat history, and does not replace the
default server engine, the older pinned-source reference bridge, the dashboard,
or the WASM path. The native-served dashboard may invoke this endpoint only
after `checkpoint_preflight_ready`; it sends the current prompt as a raw request
without a chat template or prior turns. Static/WASM sessions cannot invoke it.
Missing or invalid local export state fails closed.

## Implemented endpoint qualification

The release binary was rebuilt from the #1039 implementation with
`local-inference-accelerate`, then started on `127.0.0.1` with the frozen
export and four operator-owned exact-executor workers. `GET /api/sysinfo` reported the bridge
enabled, `checkpoint_preflight_ready=true`, attention enabled, greedy decoding,
an eight-token default, a 32-token maximum, and `static_wasm=false`.

The documented request was then sent through the real HTTP listener with an
eight-token cap. It returned in `0.16 s` wall time:

> She was so excited to see what it

The report measured `0.036683167 s` generation time and `0.126289625 s` total
time. Its endpoint-specific evidence identities were:

- decision CID:
  `blake3:87d8e9689132584a1437a244bab4e97e03af9d2609905ae6c74db0fa792c2522`
- output CID:
  `blake3:aef50423770f79714247e2e496d7f49381423893cbb8c636a31276ff9a5aa885`
- audit CID:
  `blake3:6ba5f8d8073748f57692e92a10291d507ef6963251e42069fa42256f56072ffc`
- persistent-state CID:
  `blake3:0a837266df1c81d2cc2d54e32c41d6b4e6f0476dfe2bcf04103576278e995390`
- loader checkpoint-tree CID:
  `blake3:66ee347b23e818f1816682f0b942737c88f1eca831cd6d4f00b3d14fc00aaa37`

The loader checkpoint-tree CID covers its admitted local tree and is distinct
from the export manifest's artifact-array tree CID recorded above. Weights and
tokenizer CIDs remained unchanged. Apple Accelerate was selected, all causal,
projection, R4, output-policy, all-layer, and zero-future-read audits passed,
and provider/Ollama calls remained zero. A second HTTP request produced the
same response and complete report after timing was removed. A raw-response scan
also confirmed that prompt text, prompt-token fields, the local model path, and
checkpoint filenames were absent.

Independent review then closed two boundary defects before delivery: preflight
now rejects every file outside the exact five-file export (preventing an
uncommitted shard index from changing loader selection), and the shared
single-flight reservation no longer carries the private checkpoint path. A
post-hardening Accelerate rerun returned the same text and evidence CIDs in
`0.20 s` wall time with the same audit and response-scrubbing predicates.

## Claim boundary and next decision

This positive result establishes only that the frozen #1017 generator can
produce and exactly replay one bounded fresh continuation and can be exposed
through a constrained CPU-native loopback reference seam.

It does **not** establish geometry advantage, transformerlessness,
source-free/table-native/multiply-free execution, broad coherence, correctness,
reasoning, instruction following, chat quality, browser/WASM readiness, release
quality, or frontier capability beyond the explicitly bounded native-dashboard
wiring above. #973's V5 terminal remains
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY` with action
`STOP_WITHOUT_GENERATION`. This reference lane does not close or unblock #954,
and it does not implement #962's source-free multi-turn CLI/HTTP chat stage.

The next product decision must be based on actual use of this bounded surface.
No additional training campaign or geometry mechanism is authorized by this
record.
