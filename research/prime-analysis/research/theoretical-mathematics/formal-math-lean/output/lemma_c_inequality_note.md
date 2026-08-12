# Lemma C Inequality Note (Finite-Window)

Generated: February 17, 2026
Probe artifact: `/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_none_z64_ref512.json`

## Target Inequality Form
Calibrate constants in
\[
\left|E(x)/\sqrt{x}-(aH_W^{(M)}(x)+b)\right|
\le C_1\,\Delta_M(x)+C_2\,R_{smooth}(x),
\]
where
- `\Delta_M(x)=|H_W^{(M)}(x)-H_W^{(M_ref)}(x)|`,
- `R_smooth(x)=|E(x)/\sqrt{x}-(a_{SE}S_M(x)+b_{SE})|`,
- `S_M` is explicit-formula surrogate.

## Canonical Calibration
- `M=64`, `M_ref=512`
- bases `{30,210,2310,30030}`
- `x<=10^6`, `x_step=2000`
- kernel=`none`, weights `(-1,-1,0,-1)`

Fitted constants:
- `a = -0.009754242579185462`
- `b = -0.10078083580816494`
- `C1 = 0.375`
- `C2 = 0.0`

Grid check:
- violations: `0`
- holds on tested grid: `True`
- max gap `residual - RHS`: `-0.09324363976991279` (strictly below 0 on all tested points)

## Interpretation
1. A finite-window inequality of the desired structural form is achieved.
2. For `M=64`, truncation term `\Delta_M` alone already upper-bounds residual in tested windows; hence `C2` collapses to `0`.
3. This is not tight and does not yet isolate smoothing contribution sharply.

## Next Tightening Step
Repeat same probe at higher `M` (e.g. 128/192/256) where `\Delta_M` is much smaller, to force meaningful nonzero `C2` and better separate truncation vs smoothing effects.
