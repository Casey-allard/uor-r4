# INC-0082: Product Phase Translation Cost Accounting Audit

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
`INC-0081` confirmed the first real `Q04` systems crossover, but the threshold
is not monotone:
- `max_train=36000`: first systems crossover at `Q04`
- `max_train=40000`: first systems crossover stays at `Q08`
- search-work ratio remains stable near `18-19%` of dense at both banks

The next honest question is not “push to another bank.” It is “why does the
amortized threshold move non-monotonically when search work stays stable?”

## Branch Contract
- keep the confirmed `INC-0065` route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed
- keep the confirmed `INC-0081` `Q04/Q08` threshold results fixed
- change only the analysis layer needed to explain the amortized crossover
- do not reopen geometry, spectral probes, retrieval scoring, or bank search

## Minimal Scope
1. Carry only:
   - `DENSE_Q04_T36000`
   - `H4XH4_FIELD_A150_CPX8_Q04_T36000`
   - `DENSE_Q04_T40000`
   - `H4XH4_FIELD_A150_CPX8_Q04_T40000`
   - `DENSE_Q08_T40000`
   - `H4XH4_FIELD_A150_CPX8_Q08_T40000`
2. Add or extend one audit tool to decompose:
   - offline build cost
   - online retrieval cost
   - candidate-scan work
   - secondary-key overhead
   - simple memory-traffic proxies if derivable from existing logs/artifacts
3. Produce one machine-readable audit and one short report that explain why:
   - `Q04` crosses at `36000`
   - `Q04` misses at `40000`
   - while `Q08` still crosses

## Working Hypothesis
The non-monotone `Q04` threshold is caused by amortized cost composition on the
fixed software stack, not by a collapse of the geometric routing or candidate
fraction signal.

## Stop Rule
- If the audit cannot explain the `36000` versus `40000` split from cost
  composition, do not claim a stable `Q04` threshold law.
- If the split is explainable, treat `Q04_T36000` as real but hardware-stack
  dependent, and use that to guide any future implementation-side optimization.

## Resume Note
Resume from the confirmed `INC-0081` threshold artifacts and treat this branch
as a cost-accounting explanation pass, not a new bank sweep.

## Evidence
- Tool:
  - `tools/translated_cost_accounting.py`
- Test:
  - `tests/test_translated_cost_accounting.py`
- Audit artifacts:
  - `results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json`
  - `docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md`

## Audit Read
- The audit stayed on the exact six-route `INC-0081` confirm bracket:
  - `DENSE_Q04_T36000`
  - `H4XH4_FIELD_A150_CPX8_Q04_T36000`
  - `DENSE_Q04_T40000`
  - `H4XH4_FIELD_A150_CPX8_Q04_T40000`
  - `DENSE_Q08_T40000`
  - `H4XH4_FIELD_A150_CPX8_Q08_T40000`
- `Q04` crossover at `T36000` is explained by favorable amortized accounting:
  - online gain per repeat vs dense: `9.862s`
  - offline penalty per repeat vs dense: `7.407s`
  - amortized margin vs dense: `+2.455s`
- `Q04` miss at `T40000` is explained by the same accounting flipping sign:
  - online gain per repeat vs dense: `7.072s`
  - offline penalty per repeat vs dense: `8.038s`
  - amortized margin vs dense: `-0.966s`
- `Q08` still crosses at `T40000` because the same static offline cost is
  spread across more repeats:
  - online gain per repeat vs dense: `7.657s`
  - offline penalty per repeat vs dense: `4.121s`
  - amortized margin vs dense: `+3.536s`
- The routing/search signal itself stays stable across the split:
  - `T36000 Q04`: search-work ratio `0.190206`, bytes saved `0.809794`
  - `T40000 Q04`: search-work ratio `0.183764`, bytes saved `0.816236`
  - route lookup overhead stays tiny:
    - `Q04 T36000`: `0.031s/repeat`
    - `Q04 T40000`: `0.031s/repeat`
    - `Q08 T40000`: `0.015s/repeat`

## Reading
- The non-monotone `Q04` threshold is explained by software-side offline build
  cost composition on the fixed translated stack.
- The geometric pruning signal is not collapsing:
  - candidate fraction and search-work ratios remain stable
  - bytes-saved proxy remains stable
  - the `Q04`/`Q08` split comes from how static route-build cost amortizes
- This preserves the hardware-side story, but narrows the next task:
  - do not reopen geometry
  - do not extend the bank map again yet
  - reduce or reuse the static route-build cost on the fixed translated branch

## Decision
- Close `INC-0082` positive/explanatory.
- Treat the `Q04` onset split as an implementation-side offline-cost issue,
  not as a failure of the product route law.
- Keep the confirmed route law, secondary-key law, and crossover map fixed.
- Move next to a persistent route-cache / offline-cost rescue branch on the
  fixed translated product stack.
