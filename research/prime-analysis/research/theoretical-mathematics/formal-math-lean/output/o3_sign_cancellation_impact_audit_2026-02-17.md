# O3 Sign-Cancellation Impact Audit

Generated: 2026-02-17T19:21:13.455523+00:00

## Signed/Abs Ratio Summary

- `min = 0.9980915957907652`
- `mean = 0.9993879965773462`
- `max = 0.9998545179831962`

## Sign-Only Tightening Estimate

- `C_eta_budget = 66.76254525892178`
- `C_eta_best_sign_only = 66.63513533653042`
- `relative_gain = 0.0019084042092347372`

## Conclusion

- sign-only materially closes O3: `False`
- primary next focus: `E2/x asymptotic exponent and constant control`
- target condition from O3 draft: `A_E2 <= 0.0` for `A_H<=2` under naive chain.

Observed signed/abs ratios are too close to 1 for sign-only tightening to materially shrink theorem-side O3 constants.
