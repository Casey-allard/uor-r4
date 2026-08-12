# Tau12 Rational Uniform Buffer Certificate (2026-02-24)

## Ratio Interval
- `rho_center = 1.487262003881136`
- `rho_interval = [1.487261827912958, 1.487262179849339]`
- `q_scan_max = 1000`

## Small-Denominator Scan
- first denominator with any `p/q` in interval: `None`
- certified denominator lower bound from scan: `q >= 1001`

## Key C2 Certificate
- `q=2` excluded by interval: `True`
- distance from interval to `3/2`: `0.012737820151`
- uniform rational-branch buffer from denominator bound: `c0_uniform = cos(pi/1001) = 0.999995075057`

## Mathematical Implication
If `rho` is rational and lies in this interval, then its reduced denominator satisfies `q >= q_lower_bound_from_scan`, and for every phase shift `beta0` there are infinitely many `k` with `cos(2*pi*rho*k + beta0) >= c0_uniform`.
This removes dependence on fitted `cos(beta0)` for the rational C2 branch.

Finite-precision interval certificate; theorem use is conditional on the interval assumptions.
