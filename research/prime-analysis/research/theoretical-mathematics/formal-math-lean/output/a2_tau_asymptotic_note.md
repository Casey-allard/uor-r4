# A2 Tau Asymptotic Note

Generated: February 17, 2026
Artifacts:
- `/Users/adminamn/Documents/New project/research/output/a2_tau_model_probe_gauss100_ref512.json`
- `/Users/adminamn/Documents/New project/research/output/a2_tau_model_probe_none_ref512.json`

## Result
For tested `M` grid under gaussian kernel(scale=100), the tail proxy
\[
\tau(M)=\sum_{j>M}\frac{\gamma_j K(\gamma_j)}{|1/2+i\gamma_j|}
\]
is best modeled by
\[
\tau(M)\approx \exp(c_0+c_1\gamma_M^2)
\]
with:
- `c0 = 2.4071685075911513`
- `c1 = -0.00010187257998037246`
- log-correlation `~ -0.999969`
- mean relative error `~ 0.1124`

Interpretation: strong empirical support for Gaussian-type quadratic exponent in `gamma_M`.

For `kernel=none`, the same quadratic-in-`gamma_M^2` log model is still best among tested options, but with weaker theoretical motivation.

## Use in A2 theorem track
This gives a concrete asymptotic candidate:
\[
\tau(M)\lesssim C_\tau\exp(-k\gamma_M^2),
\]
which can be combined with known zero-count growth relations to convert `\tau(M)` into explicit `M`-decay candidates.
