# #959 — G1: qualify the one-layer geometric mixer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-26T05:02:21Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-26T05:23:03Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/959
- **Merged:** 2026-08-26T05:23:03Z
- **Merge commit:** effceeda7d89218a7d09f01d2975b1ab8ea9e131
- **Base / Head:** main / UOR-Foundation:issue-951-one-layer-mixer-qualification
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add the mixer-specific analytical training lane and deterministic checkpoint schema for the existing layer-29 R4 seam
- capture frozen source residual/Q/K/V/attention/logit surfaces without exposing source parameters; keep every learned dense forward/gradient product on pinned `uor-matmul`
- add fail-closed source-free overfit, finite-difference, and checkpoint gates plus bounded teacher/student, coordinate-permuted, memory-permuted, and rollout evaluation
- retain the single bounded result, negative checkpoint, five 32-token transcripts, and exact `REDESIGN_REPRESENTATION` verdict
- update the live roadmap mirrors for replacement-design issue #958, which now natively blocks #952

## Bounded result

Source/tokenizer/base bindings match merged G0. The dataset was frozen at 18 training and 9 held-out positions with seed `95120260826`.

Preflight before source access:

- 8-example/320-step synthetic operator loss `0.008245710516348481 -> 0.00023744947884551948` (97.1203% reduction)
- analytic gradient `0.003943587187677622` vs finite difference `0.003948807716369629`; error `0.000005220528692007065` within `0.002078976249322295`
- bit-identical checkpoint round trip, digest `blake3:cf899bef600e97cd8f03fc89da4eabf17e2aac172b7fa79817b367d86283b579`

Three permitted 80-step rounds completed in 72.205s, 72.683s, and 72.681s. Support loss improved `1.0233210822 -> 0.6371028324`, while the operator and sampled-token terms remained effectively 1.0.

Primary gate:

- real held-out `0.9414138396581014`
- coordinate-permuted `0.9851714902453952`
- real advantage `0.04441627779585279` (**FAIL**, required `0.05`)
- teacher advantage `0.044031180967715605`
- student advantage `0.04519020495825049`

Memory was load-bearing (`0.10343818697664472` real probability vs `0.04907133450938596` memory-permuted), but insufficient to promote. Five real 32-token sequences were distinct and had no period-1 through period-4 cycle. Human 4/5 review is `NOT EXERCISED` because the machine gate already made the result terminal.

Retained evidence:

- report digest `blake3:98cae8e27808b7a5bd4e330bfd389a13f4720643da8315224e0510b89ab72469`
- negative checkpoint digest `blake3:8c84b8fe5b2d19be635fccf505b23a92e5daae1d3a93eda8d3465387ac75e139`
- negative mixer identity `blake3:65f9f3a6e4db1690acf1d56ed5cbe0352a02f7ff36f1a992347e447652ecb18c`
- negative memory-adapter identity `blake3:5ec83ccdd60b7fb7db985cef2b719fbe42e897285753fcdcc62e360c1d96897c`

## Focused verification

- `cargo fmt --check`
- `python3 scripts/check_claim_wording.py`
- `cargo check -p uor-r4-model-source --all-targets --offline`
- `cargo check -p uor-r4-router --all-targets --offline`
- `cargo check -p uor-r4-wasm-router --all-targets --offline`
- `cargo test -p uor-r4-model-source --lib geometric_training::tests --offline` (3 passed)
- `cargo test --lib geometric_mixer_qualification::tests --offline` (3 passed)
- `cargo test --bin r4 tests::parses_geometric_mixer_qualification_preflight --offline -- --exact` (1 passed)
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`

Broad BDD, Gate C, kappa reproduction, fuzz, Kani, corpus-scale evaluation, full teacher parity, and exhaustive certification are `NOT_RUN` by design; protected CI carries repository-wide product checks.

## Downstream state

#958 is an open native child of #949 and is blocked by #951 until this result merges. #952 is natively blocked by both #951 and #958. Closing #951 therefore makes #958 eligible, not #952.

Closes #951

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

