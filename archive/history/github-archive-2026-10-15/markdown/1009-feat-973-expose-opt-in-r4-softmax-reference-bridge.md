# #1009 — feat(#973): expose opt-in R4 softmax reference bridge

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T09:15:44Z
- **Updated:** 2026-10-09T14:17:21Z
- **Closed:** 2026-08-30T09:16:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1009
- **Merged:** 2026-08-30T09:16:57Z
- **Merge commit:** 28acc8278c2c02b6f923b86c23eb5b728ab96bc0
- **Base / Head:** main / UOR-Foundation:issue-973-r4-softmax-http-bridge
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add a dedicated opt-in native `POST /uor/v1/r4-softmax-reference/generate` route for the already-qualified `R4SoftmaxReferenceGeneratorV1`
- keep the bridge disabled by default, loopback-only, same-origin for browsers, single-flight, request-bounded, operator-worker-bounded, and isolated from `/api/chat`, OpenAI routes, the default engine, router state, and static/WASM execution
- expose the dashboard option only after native source readiness, label reference output explicitly, and avoid fabricating router telemetry
- publish the frozen #973 HTTP evidence and synchronize the living roadmap, research, lifecycle, vocabulary, and programme documentation

## Frozen result

- terminal: `PASS_R4_SOFTMAX_REFERENCE_HTTP_BRIDGE`
- HTTP 200 in 183.876699 seconds
- decoded response: `Plants need sunlight to undergo photosynthesis, a`
- generated IDs: `[34246, 737, 8118, 288, 11205, 19173, 28, 253]`
- decision CID: `blake3:e1843e45e62ab9c5872a26e808e43449841350aaef7c848d80a5a5ef9fbabbd2`
- persistent-state CID: `blake3:438436331a5030cb04438eb791baed8313b245e0fc2de4943bf20a034a5feacd`
- all 30 decoder layers; exact causal, projection, and R4 audits; zero future reads; exact CLI parity
- requested/effective/maximum active workers: 8/8/8
- success and error responses omit prompts and internal source paths
- structured result BLAKE3: `blake3:9795c2e68f175037fa4cf3227697541b050a4db784a297b239c3a14482cdfbc8`

Dashboard wiring/readiness and static/WASM isolation checks pass. Browser interaction/E2E remains `NOT_RUN`; the model canary directly exercised the loopback HTTP endpoint.

HELM-D is credited as the MIT architectural reference at commit `7501deca8f413848bfef804be64ce874b72a3cd7`. No HELM checkpoint or upstream generation code executed and no upstream paper result is inherited.

## Verification

- `cargo test --workspace --offline --no-fail-fast`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --check`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- focused HTTP/source-cache/CLI/static-WASM tests
- dashboard JavaScript syntax, JSON validation, diff check, and claim-wording gate

## Boundary and next action

This is a source-backed native CPU reference. It does not establish source-free, table-native, multiply-free, transformerless, browser-WASM, geometry-advantaged, reasoning, frontier, or release-ready inference.

The proposed next #973 rung is `R4SoftmaxTeacherTraceV1` plus `R4SoftmaxTraceCompilerV1`: construction-only layerwise token/QKV/attention/value/logit traces from this exact oracle, followed by the first source-free student/attention-state comparison on decoded tokens and next-token loss. Those names are `NOT_IMPLEMENTED` and `NOT_RUN` in this PR.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

