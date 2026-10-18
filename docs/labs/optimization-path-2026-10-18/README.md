# Optimization path record — 2026-10-18

Owner direction (2026-10-18, in-session): the falsify-then-optimize sequence is
the current path; all three active labs align to it. PM path directive:
[#2029 comment 6244066845](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244066845).

## What was decided

- D11 multiplier-free serving **stands**; no float/transcendental in served
  computation. Scalar float and sin/cos remain legal in offline training
  (owner 2026-10-12), unchanged.
- The compute-cost thesis is architectural: KB-scale cache-resident addressed
  state and exact addressed reads instead of per-token weight streaming. It is
  proven in order, not asserted.
- Serving floats remain a possible future D11 revision only with measured
  evidence and an owner decision at that point.

## The sequence

1. **Dose 1** — the paired Relation/frozen-zero falsification run under
   `docs/harness/prep/m1-learned-attention-link-2026-10-17.md`, one seed first,
   release-profile binary required (PM requirement
   [#2029 comment 6243986174](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243986174)).
   Sole paid-compute item; stop semantics pre-registered.
2. **Optimization rung** — gated on the dose producing a decision.
   Release-profile baselines, counterfactual row-parallelism, shared-memory
   table staging, selective trace; tokens/s bars pre-registered on the same
   CUDA harness before kernel work.
3. **Batched-coordinate scoring** — candidate training-time lever; new FIT'd
   pre-registration required; never mid-run.
4. **Spinor/quaternion adapters** — conditional; float offline, discretized
   serving, measured-ceiling entry only.

## Lab assignments (current)

- **Codex (cf1):** dose packet seal (with the fresh-input recipe), execution,
  outcome report; kernel implementation in the optimization rung.
- **Antigravity (ag1):** confined packet/transcript reviews; evidence
  synthesis via the merged cell-reference scorer (#2604). Host/source-only seat.
- **Kimi (pm1):** sheet/packet ownership, dispositions, dispatch authority,
  optimization measurement plan (DeepSeek subagent seats carry bulk analysis).
- **DeepSeek autonomous line:** object-store tooling continues unchanged; it
  already serves this path's pod staging.

## Boundaries

This record changes no scientific bar, sheet text, FIT status or compute
authorization. Training/M2/Stage B remain HELD; Q4 NOT_EXECUTABLE. The dose
dispatch remains a separate PM release after packet seal and confined review.
