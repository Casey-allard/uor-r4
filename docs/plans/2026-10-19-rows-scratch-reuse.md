# Implementation plan: row proposal scratch reuse

Owner approved the source design and immediate implementation in Codex chat, 19 October UTC. No additional handoff is required. No GPU allocation or model execution is part of this delivery.

## Single bounded delivery

1. `crates/uor-r4-discrete/src/rows_device.rs`: replace per-call internal target/base/cost/flag/status allocation with exclusive per-Device storage. Retain call-owned losses and proposal data. Preserve exact active extents, initialization, kernel arguments, status reset/readback, stream ordering and synchronization. Failure must never expose scratch still in flight. Risk: high (CUDA lifetime); depends on ownership tests.
2. `crates/uor-r4-discrete/src/device.rs`: initialize device-owned synchronized storage. Scope is exact model and stream; no sharing across Devices or caching model-dependent values. Risk: medium; depends on step1 API.
3. A small host-only storage lifecycle module/test if needed: tests first for repeated reuse, growth, failed allocation, exclusive access and error handling. No CPU model/reference calculations. Risk: medium.
4. Canonical dated record and current-state entry: distinguish host compile/lifecycle checks from NOT_RUN CUDA equivalence/timing and no claimed speedup. Risk: low; depends on review.

## Validation

Observe meaningful host-only ownership/lifecycle tests fail before implementation, then pass. Typecheck the CUDA boundary with existing stub-nvcc workflow (not a real CUDA build). Run focused source/policy checks. Independent spec review precedes Rust/code-quality review. GPU equivalence (loss/proposal/commit/epoch/cursor, repeated and varying rows, outstanding proposals, refusal/fault paths) and matched timing remain required under a separate actual release before optimization performance claims.

## Exclusions

No CUDA kernel, objective, chooser, learning schedule, sheet, codebook, block-driver integration, evaluation or scientific-bar changes. No donor work or paid resource. Keep failed-attempt evidence intact.
