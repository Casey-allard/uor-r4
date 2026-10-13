# A2 Truncation Decay Note (Candidate Constants)

Generated: February 17, 2026
Artifact: `/Users/adminamn/Documents/New project/research/output/a2_truncation_decay_probe_gauss100_ref512.json`

## Candidate A2 Form
Use
\[
\Delta_M(x;W)\le C_\Delta (\log x)^\beta\,\tau(M)
\]
with
\[
\tau(M)=\sum_{j>M}^{M_{ref}}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|}
\quad(\text{tail\_Fp type} ),
\]
for kernelized run (`K` gaussian, scale 100).

## Calibrated Values (finite-window)
- chosen tail: `tau_tail_Fp`
- `beta = 2.6`
- `C_delta = 5.598810038306168e-04`
- sampled windows: `x<=3e5, 1e6`, bases `30,210,2310,30030`, `M in {64,96,128,160,192,224,256,320,384}`, `M_ref=512`
- sample check: zero violations.

## Interpretation
1. Among tested tail templates, `tail_Fp` gives lower coefficient variation than `tail_F`.
2. This provides an explicit A2 candidate with theorem-style structure.
3. Still finite-window calibration; asymptotic proof is not done.

## Next Proof Task
Bound the same tail expression analytically for general `M` (not just to `M_ref`) and prove the inequality uniformly in `x>=x0` and `W in \mathcal W`.
