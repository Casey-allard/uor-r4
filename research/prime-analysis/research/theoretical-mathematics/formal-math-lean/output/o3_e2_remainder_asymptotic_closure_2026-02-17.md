# O3 E2 Remainder Asymptotic Closure

Generated: 2026-02-17T19:31:09.485680+00:00

## Status

- `conditional_asymptotic_closed`
- scope: inherits O2 theorem_assumptions (HSW2022 explicit N(T) error model and sum-integral domination)

## Derived Envelope

- `tau_infty(M) <= c0 * exp(-k*M)`
- `k = 0.08482120688755276`
- `c0 = 166.2789198268843`

## Truncation Schedule

- `M(x) = 64.0 + 2.0*log(x)`
- `x0 = 3.0`
- `nu = k*lambda = 0.16964241377510553`

## Closed Remainder Bound

- `A_rem = 0`
- `C_rem_uniform = 0.08666111355370633`
- sup term `sup_(x>=x0) x^(-nu)(log x)^beta = 89.7378167655244`

## Finite-Window Crosscheck

- checker all hold: `True`
- max ratio discrete/upper: `0.028628810319348205`
- worst gap discrete-upper: `-2.4695415645853848e-08`

## Inherited Theorem Assumptions

- [HSW2022] Assume explicit zero-count error bound |N(T)-M(T)| <= 0.1038 log(T) + 0.2573 log log(T) + 9.3675 for T>=e, where M(T)=T/(2pi)*log(T/(2pi e)).
- Define derivative majorant by finite difference: N'(t)_up := (M(t+h)-M(t) + E(t+h)+E(t))/h with h>0.
- Use N'(t)_up in the weighted tail integral for tau_infty(M), with kernel-weighted integrability and monotone decay in M.
