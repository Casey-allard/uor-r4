# INC-0067: Spectral Signal Probes

## Status
Confirm completed inconclusive on 2026-03-11.

## Trigger
`INC-0066` confirmed that the fixed confirmed route set has a real operator
signature: the product routes keep more delocalized low modes and a different
low-frequency shell/sector profile than the controls. The next unresolved
question is whether those modes carry useful structure, not merely a distinct
geometry.

## Branch Contract
- keep the confirmed route set fixed
- reuse the measured geometric operator family from `INC-0066`
- project shell, sector, and task-relevant signals onto the measured modes
- test usefulness of spectral structure without reopening route-law search

## Minimal Scope
1. carry the confirmed `INC-0066` route set:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `HOPF_PHI2_BAND_PHI`
   - `H4XH4_FIELD_A100`
   - `H4XH4_FIELD_A150`
   - `R0`
2. add explicit low-mode projection diagnostics for:
   - shell ids
   - sector ids
   - proxy task labels or residual signals
3. compare whether product routes carry stronger low-mode task signal than the
   Hopf controls

## Working Hypothesis
If the project’s spectral claim is real in a useful sense, the product branch
should not only change the operator spectrum. It should also carry a distinct
and potentially stronger low-mode signal decomposition on relevant observables.

## Evidence
- Config:
  - `configs/spectral_signal_inc0067_screen.json`
  - `configs/spectral_signal_inc0067_confirm.json`
- Tools:
  - `tools/spectral_signal_probe.py`
  - `tools/spectral_signal_sweep.py`
- Analysis:
  - `results/analysis/inc0067_spectral_signal_probes_screen.json`
  - `results/analysis/inc0067_spectral_signal_probes_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_113729.md`
  - `docs/governance/gates/gate_20260311_114258.md`

## Screen Read
- All audited route graphs stayed connected.
- The operator distinction remained visible.
- The direct label probe stayed inconclusive:
  - Hopf label lowfreq gap: `-0.000117`
  - Hopf label Dirichlet gap: `-0.000398`

## Confirm Read
- All audited route graphs stayed connected on 4 seeds.
- The direct one-hot label probe remained slightly negative against the Hopf
  controls:
  - Hopf label lowfreq gap: `-0.000154`
  - Hopf label Dirichlet gap: `-0.000695`
- `HOPF_PHI2_BAND_PHI` slightly led the direct label lowfreq metric:
  - `label_lowfreq=0.023796`
- Product routes still led participation and shell-focused lowfreq energy, but
  that did not translate into a positive raw label-smoothness result.

## Decision
- Close `INC-0067` confirm inconclusive/negative for direct label probes.
- Keep `INC-0066` positive as the operator-level result.
- Do not claim useful task spectral structure from the direct one-hot label
  probe.

## Next Step
- Move next to residual/task-error probes (`INC-0068`) on the fixed confirmed
  operator set.
- Test whether routed residuals, margins, or error signals align with the
  product modes more usefully than raw labels do.

## Resume Note
Do not reopen route search here. The next question is which task-derived signal
actually matches the confirmed operator difference.
