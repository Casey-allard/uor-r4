# Math-First Research Brief Toward Proof

Generated: 2026-02-17

## Scope

This brief is research-only: what mathematics is still required to convert the current bounded/explicit-formula-inspired pipeline into a proof-level RH implication.

## Core conclusion

The project is not blocked by missing computation. It is blocked by one theorem-level gap:

- proving a rigorous asymptotic transfer from the project's bridge quantity to a **standard RH-equivalent endpoint bound** (Chebyshev-error class), not just finite-window validation.

## Primary references checked

1. Schoenfeld (Math. Comp. 1976): explicit RH-conditional Chebyshev bounds and constants.
   - AMS record: https://www.ams.org/mcom/1976-30-134/S0025-5718-1976-0457374-X/
   - PDF: https://www.ams.org/mcom/1976-30-134/S0025-5718-1976-0457374-X/S0025-5718-1976-0457374-X.pdf

2. Hasanalizade, Shen, Wong (JNT 2022): explicit bound
   |N(T) - T/(2pi) log(T/(2pi e))| <= 0.1038 log T + 0.2573 log log T + 9.3675.
   - PDF: https://www-math.nsysu.edu.tw/~pjwong/stuff/CountingRiemannZeros.pdf
   - DOI landing: https://doi.org/10.1016/j.jnt.2021.06.032

3. RH equivalence summary (AIM article index):
   - https://aimath.org/WWN/rh/articles/html/95a/

## Research findings that matter for this project

1. Endpoint target must be RH-equivalent
- A proof must reach a standard equivalent statement for psi(x)-x, not merely "small on tested x".
- Therefore all intermediate model bounds must terminate in that endpoint form.

2. Explicit zero-count constants are available now
- The HSW 2022 constants can be used as theorem assumptions in tail estimates.
- This supports replacing ad hoc density placeholders in the A2/O2 tail path.

3. Current O2 finite-reference checks are mathematically useful but not sufficient
- They validate shape and domination on finite windows.
- They do not yet prove asymptotic-for-all-x statements.

4. Main remaining proof risk is not A2 tail size
- Under gaussian weighting and current reference height, added infinite tail mass is tiny numerically.
- The real difficulty is proving the asymptotic transfer/endpoint implications (A3/A4-to-endpoint), not fitting tail constants.

## Best next math step (research priority)

- Focus theorem work on the bridge-to-endpoint implication chain:
  1. prove asymptotic bridge growth control in theorem form (no empirical safety factors),
  2. prove base-uniform constants asymptotically,
  3. derive RH-equivalent endpoint bound.

In short: A2 tail machinery is now largely a support lemma; proof-critical research should center on the transfer/endpoint argument.
