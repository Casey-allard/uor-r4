# K1-W4 Source Mapping (2026-02-24)

## Scope

Map published sources to the finite-mode contract obligations in:
`ExplicitFormulaFiniteDecayingPhaseCorrectionsAssumptions`.

## Source-to-obligation map

1. [On the error term in the explicit formula of Riemann-von Mangoldt II (arXiv:2402.04272)](https://arxiv.org/abs/2402.04272)
- Supports: `W4-O1` (explicit truncated explicit-formula error term control).
- Does not directly provide: `W4-O2` finite decaying mode phase decomposition.

2. [On the error term in the explicit formula of Riemann--von Mangoldt (arXiv:2111.10001)](https://arxiv.org/abs/2111.10001)
- Supports: `W4-O1` class (explicit error terms in Riemann-von-Mangoldt framework).
- Does not directly provide: `W4-O2` finite mode phase representation.

3. [An explicit form of Ingham's zero density estimate (arXiv:2507.15184)](https://arxiv.org/abs/2507.15184)
- Supports: zero-density auxiliary controls that can feed proof-side bounds.
- Does not directly provide: finite-mode phase decomposition.

4. [On sign changes of psi(x)-x in short intervals (arXiv:1912.00853)](https://arxiv.org/abs/1912.00853)
- Supports: oscillation/sign-change consequences from right-half zeros.
- Does not directly provide: global finite-mode phase formula.

5. [On sign changes of psi(x)-x in short intervals, II (arXiv:2202.01837)](https://arxiv.org/abs/2202.01837)
- Supports: sharpened interval localization in related oscillation setting.
- Does not directly provide: `W4-O2` contract term.

6. [Numerical verification of the Riemann hypothesis to height 3*10^12 (arXiv:2004.00423)](https://arxiv.org/abs/2004.00423)
- Supports: finite-height guardrail only.
- Does not provide: all-height theorem term for `W4-O2`.

## Conclusion

- `W4-O1` has multiple published-source candidates.
- `W4-O2` remains the only unmatched obligation for unconditional in-repo closure.
