# K1 W65 Math-First Status Audit (2026-02-24)

## Status
- c2_buffered_existence: open_math
- eventual_rounding_preservation: partially_closed_symbolic
- m4_one_sided_q_lt_a1_chain: closed_symbolically_given_tail_bound

## Quantitative Evidence
- rounding anchor_fraction min (c0<=0.5): 0.500000
- rounding both_fraction min (c0<=0.5): 1.000000
- min cos2(integer) on buffered set: 0.684818
- min |cos1(integer)| on buffered set: 0.999999885
- rho=tau2/tau1: 1.487262003881
- max |nonneg_fraction-0.5| up to tested N: 0.010000
- M4 one-sided q_split_neg_max: 0.430301189
- M4 one-sided delta_split_neg_min: 0.549698811
- M4 robust_split_neg: True

## Remaining Math (Not Formalization)
- Prove buffered C2 existence theorem (infinite anchors with cos2>=c0) unconditionally from the rotation case split, not finite-window evidence.
- Upgrade eventual rounding-preservation from numeric evidence to an explicit analytic phase-error bound tied to anchor-to-integer map.
- Replace finite-window tail input in M4 one-sided split with theorem-grade cofinal T4^- and E4^- bounds to instantiate q<a1 unconditionally.

## Ready Now
- Symbolic chain q<a1 => positive lower envelope once constructive gate and one-sided tail bound hypotheses are supplied.
