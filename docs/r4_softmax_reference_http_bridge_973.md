# Native R4/Spin softmax-reference HTTP bridge (#973)

Date: 2026-08-30 (EDT)

Evidence status: **PASS**

- Terminal verdict: **`PASS_R4_SOFTMAX_REFERENCE_HTTP_BRIDGE`**.
- The frozen eight-token G0-P1 canary returned HTTP 200 and matched the
  previously qualified CLI response, generated tokens, decision CID,
  persistent-state CID, selected layers, and exact audit flags.
- Native dashboard wiring/readiness gating and static-WASM isolation:
  **PASS** by source guard, static assertion, and JavaScript syntax checks.
- Browser interaction/end-to-end execution: **NOT_RUN**. The model canary was
  sent directly to the loopback HTTP endpoint.
- Default engine, static/WASM, source-free-runtime, release, and hosted-page
  promotion: **NOT AUTHORIZED** by this bounded result.

This is an append-only evidence record for issue #973. The earlier CLI
qualification remains recorded in
[`r4_softmax_reference_generation_973.md`](r4_softmax_reference_generation_973.md);
this record establishes only that the dedicated native HTTP endpoint preserves
that already-observed policy for the frozen canary. Dashboard wiring and
isolation are statically verified; browser interaction remains `NOT_RUN`.

The compact structured evidence surface is
[`r4_softmax_reference_http_bridge_result_973.json`](r4_softmax_reference_http_bridge_result_973.json).
Its frozen file BLAKE3 is
`blake3:9795c2e68f175037fa4cf3227697541b050a4db784a297b239c3a14482cdfbc8`.

## Scope and mechanism

The native server now has one dedicated endpoint:

```text
POST /uor/v1/r4-softmax-reference/generate
```

The endpoint invokes the existing `run_r4_softmax_reference_generation`
function directly. It has no cascade, fallback, sampling policy, alternate
engine, Ollama call, hosted-provider call, or route through `/api/chat` or the
OpenAI-compatible surfaces. The source path and worker count are operator
configuration; the request accepts only an exact prompt plus a bounded token
cap. The HTTP default is 8 generated tokens, the maximum is 32, and the body
limit is 16 KiB.

The bridge is disabled by default, native-only, loopback-only, same-origin for
browser callers, and process-wide single-flight with source download, reload,
and compilation. A second reference request conflicts rather than starting
concurrent model work. Worker selection is operator-owned, nonzero, and no
larger than the parallelism available to the process.

The dashboard keeps the reference option hidden and disabled until native
`/api/sysinfo` reports both `enabled` and `source_preflight_ready`. Static/WASM
mode rejects the option explicitly. Reference requests use only the dedicated
endpoint and do not fabricate geometric map telemetry or mutate the default
session engine.

### Evidence boundary

This bridge executes `R4SoftmaxReferenceGeneratorV1`: the full pinned
SmolLM2-135M-Instruct source decoder, including learned Q/K/V, RoPE, ordinary
dot-product/stable-softmax attention, value aggregation, output projections,
residual blocks, normalization, feed-forward blocks, and LM head. UOR's
R4/Spin seam supplies coherent-frame transport around attention; it does not
replace the checkpoint's language model.

[HELM-D](https://github.com/Graph-and-Geometric-Learning/helm/tree/7501deca8f413848bfef804be64ce874b72a3cd7)
is credited as the MIT-licensed architectural reference at upstream commit
`7501deca8f413848bfef804be64ce874b72a3cd7`. No HELM checkpoint, HELM
generation code, or HELM paper result executed or is inherited by this result.

## Frozen run contract

The following contract was posted to issue #973 before endpoint-backed model
output was opened:

- **Metric to move:** exact HTTP-to-CLI policy identity for the already
  qualified G0-P1 eight-token canary; the endpoint began as `NOT_IMPLEMENTED`.
- **Reachability ceiling:** 100% of this request reaches the exact reference
  generator because the dedicated route has no cascade, fallback, sampling,
  or alternate engine. Any mismatch would therefore be bridge wiring, not new
  attention evidence.
- **Cheap instrument required:** model-free parser, bounds,
  disabled-by-default, loopback-only, single-flight, source-unavailable,
  scrubbed-response, and injected-executor route tests must pass, together
  with formatting, focused lint, claim-wording, and native/WASM boundary
  checks.
- **Exit rule:** one explicit loopback invocation using the pinned source,
  exactly 8 workers, the exact G0-P1 prompt, and at most 8 generated tokens
  must return HTTP 200 within 600 seconds. Response text, token IDs, decision
  CID, persistent-state CID, all-layer/exact-audit flags, and the
  zero-future-read flag must match the frozen CLI canary.
- **If positive:** publish the opt-in native bridge through the protected
  queue, then use it as the behavioral oracle for the first source-free
  compilation step.
- **If negative:** repair only HTTP request/configuration/response wiring and
  rerun this single canary; do not reopen the established attention result or
  resume intrinsic-score research.
- **Cost estimate:** model-free checks plus one approximately three-minute
  model execution after incremental compilation. No five-prompt rerun,
  release tag, hosted/static-WASM promotion, or broad QA campaign.

## Exact verification surface

The following focused checks passed before the model canary:

| Check | Observed result |
| --- | --- |
| `cargo test --offline -p uor-r4-wasm-router --lib r4_softmax_reference_http_ -- --nocapture` | 5 passed: strict request/bounds, loopback and origin normalization, scrubbed/operator-bounded response, single-flight plus failure/panic release, and real-socket route/method/origin/body-cap behavior |
| `cargo test --offline -p uor-r4-wasm-router --lib source_cache_reservation_closes_download_compile_reload_and_reference_races -- --nocapture` | 1 passed; reference generation participates in the existing process-wide source-cache exclusion |
| `cargo test --offline -p uor-r4-wasm-router --bin r4 serve_keeps_r4_softmax_reference_disabled_unless_explicitly_enabled -- --nocapture` | 1 passed; the serve surface remains opt-in |
| `cargo test --offline -p uor-r4-wasm-router --lib static_wasm_cannot_masquerade_as_the_native_r4_softmax_reference -- --nocapture` | 1 passed; static/WASM cannot expose or impersonate the native bridge |
| `cargo test --workspace --offline --no-fail-fast` | PASS; zero failures, with explicitly ignored evidence/fixture runs remaining ignored |
| `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` | PASS |
| `cargo check -p uor-r4-graph-format --no-default-features --offline` | PASS |
| `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline` | PASS |
| `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline` | PASS; only pre-existing warnings were reported |
| JavaScript module syntax check over the dashboard script | PASS |
| `cargo fmt --check` and `git diff --check` | PASS |
| `python3 scripts/check_claim_wording.py` | PASS after the final evidence and direction synchronization |
| Release build of `r4` | PASS |

The native status response reported the bridge enabled, the source preflight
ready, the exact dedicated endpoint, an eight-token default, a 32-token cap,
eight configured workers, and `static_wasm: false` before the request was
sent.

## HTTP canary result

Request contract:

```json
{
  "prompt": "Explain in three short sentences why plants need sunlight.",
  "max_tokens": 8
}
```

Transport result:

- HTTP status: `200`
- curl wall time: `183.876699` seconds
- source load: `0.579597833` seconds
- generation: `183.197990375` seconds
- generator-reported total: `183.867798542` seconds
- stop reason: `maximum_new_tokens`
- requested/effective/maximum active workers: `8 / 8 / 8`

Decoded response:

> Plants need sunlight to undergo photosynthesis, a

Generated token IDs:

```text
[34246, 737, 8118, 288, 11205, 19173, 28, 253]
```

Stable identities:

- decision CID:
  `blake3:e1843e45e62ab9c5872a26e808e43449841350aaef7c848d80a5a5ef9fbabbd2`
- persistent-state CID:
  `blake3:438436331a5030cb04438eb791baed8313b245e0fc2de4943bf20a034a5feacd`

Machine audit:

| Field | Observed |
| --- | ---: |
| Positions executed | 47 |
| Selected decoder layers | 30 |
| Layer calls | 1,410 |
| Head calls | 12,690 |
| Key transports | 304,560 |
| Value transports | 304,560 |
| Projection hooks | 1,410 |
| R4 blocks encoded | 9,948,960 |
| R4 key blocks transported | 4,872,960 |
| R4 value blocks transported | 4,872,960 |
| R4 output blocks decoded | 203,040 |
| Future reads | 0 |
| R4 future-position reads | 0 |
| Source-frame permutations | 0 |

`causal_audit_exact`, `projection_audit_exact`, `r4_audit_exact`,
`all_layers_selected`, and `zero_future_reads` were all `true`. Every one of
the 47 physical forwards used more than one worker; peak active workers was
eight.

The response-scrubbing tests proved that neither the request prompt nor the
operator's source path appears in a successful serialized response, and that
busy, source-preflight, and generator failures do not return internal source
paths. Detailed failures remain in local server logs. A direct search of the
real canary response likewise found no prompt, rendered-prompt field,
source-path field, or local `/Users` path.

The server also emitted a pre-existing default-R4G1 startup warning concerning
an unrelated staging-path permission. The dedicated reference endpoint
remained enabled and source-ready, returned HTTP 200, and matched the frozen
CLI identities; the warning did not affect this canary.

## Verdict and next decision

**`PASS_R4_SOFTMAX_REFERENCE_HTTP_BRIDGE`**

The positive branch of the frozen contract is selected: ship this bounded,
opt-in native bridge through the protected queue and use the exact source-backed
policy as the behavioral oracle for the next source-free compilation rung.

The proposed next mechanism is a construction-only layerwise teacher-trace
surface followed by the first source-free student/attention-state artifact.
It should record the exact policy's causal token, Q/K/V, transported attention,
value, and logit evidence needed for compilation, then compare the compiled
student against this oracle on decoded tokens and held-out loss. That proposal
is **NOT_IMPLEMENTED** and has no result in this record.

### Strict nonclaims

This PASS does **not** establish source-free or transformerless inference,
multiplication-free execution, `no_std`, allocation-free inference,
browser-WASM execution, geometry advantage over ordinary attention, softmax
replacement, general language quality, meaning, reasoning, frontier-model
quality, hosted-product readiness, or release/tag readiness. It establishes
only faithful native HTTP access to the already-qualified, source-backed
R4/Spin softmax reference under the frozen canary contract, plus statically
verified dashboard wiring and isolation. Browser interaction remains
`NOT_RUN`.
