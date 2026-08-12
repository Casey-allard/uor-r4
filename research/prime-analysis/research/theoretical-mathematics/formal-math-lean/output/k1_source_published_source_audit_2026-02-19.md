# K1 Source Published-Source Audit (2026-02-19)

## Objective
Identify whether the remaining K1-source term can be closed by importing published mathematics, and map that directly to the in-repo Lean class boundary.

## Primary sources checked
- Schlage-Puchta, *The oscillation of the error term in the prime number theorem* ([arXiv:1912.00853](https://arxiv.org/abs/1912.00853))
  - Relevance: matches the in-repo class shape `SchlagePuchta2019IntervalOscillationFormalized` (given right-half zero -> interval-localized oscillation lower bound).
  - Action taken: used as the explicit citation lock for the imported theorem slot.

- Pintz (2017), theorem-level zero -> oscillation transfer reference
  - DOI link used in repo metadata: <https://doi.org/10.1134/S0081543817010163>
  - Relevance: aligns with `Pintz2017ZeroToOscillationFormalized` endpoint contract.

- Clay Mathematics Institute RH problem statement ([official RH page](https://www.claymath.org/millennium/riemann-hypothesis/))
  - Relevance: baseline status check; RH remains an open Millennium problem.

## Conclusion
- A published source matching the Schlage-Puchta interval-oscillation interface exists and has now been imported at the Lean boundary.
- In this repo, that import is represented as an explicit axiom term (`schlagePuchta2019_given_zero_interval_oscillation_term`).
- Therefore:
  - Trusted-import closure path is complete.
  - Unconditional/non-circular in-repo closure is not complete until the imported boundary is replaced by a fully formalized theorem term.
