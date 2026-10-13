# INC-0068: Spectral Residual / Task-Error Signals

## Status
Screen completed inconclusive/negative on 2026-03-11.

## Trigger
`INC-0067` showed that the confirmed product operator remains distinct, but
direct one-hot label smoothness does not beat the Hopf controls. The next
question is whether a more task-relevant signal, such as routed residuals,
margins, or error indicators, aligns with the product modes better than raw
labels do.

## Branch Contract
- keep the confirmed route set fixed
- reuse the measured operator family from `INC-0066`
- measure routed residual, margin, or task-error signals on those same modes
- do not reopen route-law or phase-law search

## Minimal Scope
1. carry the fixed confirmed route set:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `HOPF_PHI2_BAND_PHI`
   - `H4XH4_FIELD_A100`
   - `H4XH4_FIELD_A150`
   - `R0`
2. derive at least one route-specific task signal:
   - prediction residual norm
   - top-1 error indicator
   - margin or confidence signal
3. project that signal onto the measured modes and compare product vs Hopf

## Working Hypothesis
The product operator may organize task-relevant error structure even if it does
not improve raw one-hot label smoothness.

## Acceptance
- operator construction remains stable on the fixed confirmed route set
- at least one residual/task-error signal shows a stable low-mode distinction
  across seeds
- conclusions remain based on explicit signal projections, not route MSE alone

## Evidence
- Config:
  - `configs/spectral_residual_inc0068_screen.json`
- Tools:
  - `tools/spectral_residual_probe.py`
  - `tools/spectral_residual_sweep.py`
- Analysis:
  - `results/analysis/inc0068_spectral_residual_task_signals_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_122236.md`

## Screen Read
- All audited route graphs stayed connected.
- The confirmed operator distinction remained visible:
  - product routes still carried higher participation and stronger shell-focused
    low-frequency energy
  - product routes still carried lower sector low-frequency concentration than
    the controls
- The routed task-error probes were negative versus the Hopf controls:
  - Hopf residual-L2 lowfreq gap: `-0.013560`
  - Hopf residual-L2 Dirichlet gap: `-0.018546`
  - Hopf error-indicator lowfreq gap: `-0.007110`
  - Hopf error-indicator Dirichlet gap: `-0.003283`
  - Hopf true-margin lowfreq gap: `-0.021551`
  - Hopf true-margin Dirichlet gap: `-0.018763`

## Decision
- Close `INC-0068` inconclusive/negative at screen stage.
- Do not carry this branch to confirm:
  - every tracked residual/error signal is already negative
  - the result is cleaner than the `INC-0067` label probe and does not justify
    replay on the same proxy target
- Keep `INC-0066` positive as the operator result and `INC-0065` positive as
  the confirmed product phase-field result.

## Next Step
- Move next to translated-retrieval evaluation of the confirmed product branch
  (`INC-0069`).
- Test whether the product phase-field routes are more useful as retrieval or
  addressing geometry than as low-mode task-signal carriers on proxy
  regression.

## Resume Note
The next test is not another proxy-spectral signal probe. It is task
translation of the confirmed product branch.
