# Analytic Kernel Source Map (2026-02-18)

This note maps the two remaining Lean kernel obligations in
`PrimeRiemannBridgeCompletionKernel.lean` to concrete theorem sources.

## K1: `right_half_zero_forces_lower_envelope`

Lean obligation:
- If a nontrivial zero has `Re(s) > 1/2`, derive a lower-envelope/oscillation statement for the prime error term of size `x^β` (for some `β > 1/2`).

Source targets:
1. J. Pintz (2017), *Distribution of zeta zeros and the oscillation of the error term of the prime number theorem*, DOI `10.1134/S0081543817010163`.
   - The abstract explicitly describes the Ingham/Turan conversion program and ties zeta-zero distribution to maximal order of the PNT error term.
2. S. G. Revesz (IMRN 2023), DOI `10.1093/imrn/rnac274`.
   - Proves (for Beurling systems, extending classical paradigm) that a given zeta zero induces oscillation of size proportional to `x^{Re(rho0)}`.
3. G. Tenenbaum, *Introduction to Analytic and Probabilistic Number Theory*, Ch. II.5 (discussion of Ingham/Turán converse methods and oscillation transfer).
   - Used as a standard textbook bridge from zero locations to oscillation statements for prime-counting error terms.

Reference links:
- [Pintz 2017 DOI](https://doi.org/10.1134/S0081543817010163)
- [Revesz 2023 DOI](https://doi.org/10.1093/imrn/rnac274)
- [Cambridge chapter listing (Tenenbaum)](https://doi.org/10.1017/CBO9781107295591.006)

Formalization use:
- Treat K1 as imported theorem interface first (already encoded in `ImportedAnalyticBridgeResults`).
- Then replace import by a theorem pack once a precise classical `psi(x)-x` oscillation statement is formalized.

## K2: `super_half_lower_contradicts_endpoint_upper`

Lean obligation:
- Contradict a super-`1/2` lower envelope with an upper envelope of type
  `O(x^(1/2) (log x)^2)`.

Local formal source target (already in mathlib tree):
1. `Mathlib/Analysis/SpecialFunctions/Pow/Asymptotics.lean`
   - theorem `isLittleO_log_rpow_rpow_atTop`.
   - This gives `(log x)^2 = o(x^eps)` for any `eps > 0`, implying
     `x^(1/2) (log x)^2 = o(x^β)` for `β > 1/2`.

Formalization use:
- Build a reusable asymptotic domination lemma from `isLittleO_log_rpow_rpow_atTop`.
- Use it to show eventual inequality
  `C * sqrt(x) * (log x)^2 < x^β`.
- Combine with lower-envelope "for all large thresholds there exists x" to derive `False`.

## Minimal sequential plan from here

1. Implement the K2 contradiction lemma in Lean (pure asymptotics, no zeta imports).
2. Keep K1 as the only imported analytic theorem until classical oscillation theorem is formally encoded.

## Completed structural reduction in Lean

`super_half_lower_contradicts_endpoint_upper` is now derived from a generic domination premise:

- theorem: `super_half_lower_contradicts_endpoint_upper_of_domination`
- file: `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeCompletionKernel.lean`

So the remaining work is explicitly split into:
1. zero -> oscillation transfer (analytic number theory input), and
2. pure asymptotic domination `sqrt(x) log^2(x) << x^beta` for `beta>1/2` (mathlib-formalizable).


## New formalized result (closed on 2026-02-18)

Closed in Lean:
- `endpoint_upper_power_domination` proves, for `beta > 1/2`, eventual domination
  `C * sqrt(x) * log(x)^2 < x^beta`.
- `super_half_lower_contradicts_endpoint_upper_of_domination` now uses this to derive the K2 contradiction.

Net effect:
- K2 is no longer an independent open analytic kernel item.
- Remaining open kernel is K1 only (zero -> oscillation transfer).


## Proof-object boundary update (2026-02-18)

The final open import is now encoded as a structured proof object:
- `PublishedZeroOscillationPack` in
  `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeCompletionKernel.lean`
- Metadata locks included:
  - `source_tag = PINTZ-2017-OSCILLATION`
  - `source_url = https://doi.org/10.1134/S0081543817010163`
  - `theorem_ref = Thm-2-zero-to-oscillation-transfer`

This replaces a raw standalone axiom with an auditable import-pack interface.
