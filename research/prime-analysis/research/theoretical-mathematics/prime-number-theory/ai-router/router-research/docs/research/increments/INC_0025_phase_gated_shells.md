# INC-0025: Sparse / Quantized Phase-Gated Shells

## Status
Planned, not run.

## Hypothesis
`INC-0024` suggests shell phase matters, but a continuous shell bias is still too expensive.
If the shell-phase term activates only near shell boundaries or in discrete phase steps, the routed family may keep the geometry gain while reducing route churn and runtime cost.

## Why This Follows From `INC-0024`
- `PHASE_K25_C035` beat the coarse `PHI_PHI_PHI` family reference on confirm-stage MSE and runtime.
- It still missed the strict runtime gate vs `R0` by a small margin.
- That is evidence for a real mechanism with an over-applied control law, not evidence that shell phase should be abandoned.

## Candidate Laws
1. Boundary-gated phase shells
   - apply shell-phase bias only when `shell_frac` is near a shell boundary
   - intuition: move boundaries where routing decisions are actually ambiguous, not everywhere
2. Quantized phase shells
   - convert phase pressure into a small integer shell offset
   - intuition: discrete geometry may reduce address churn versus a fully continuous bias
3. Sign-only phase shells
   - use only the sign of phase pressure and a fixed small shell shift
   - intuition: preserve directional phase information while minimizing runtime overhead

## Promotion Standard
Promote a new phase-gated candidate only if it:
- stays seed-wise healthy
- beats `PHASE_K25_C035` on runtime or MSE
- is materially closer to `R0` on runtime than the continuous `C035` law

## Failure Interpretation
If the best sparse/quantized law still fails to beat `PHASE_K25_C035`, treat continuous shell-phase coupling as sufficient proof-of-effect and move the program toward cost decomposition rather than more shell-law tuning.
