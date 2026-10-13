# INC-0093 Product Phase Translation Cache Residency Mix

## Confirm Read
- The residency decomposition is now explicit on the fixed translated product
  stack.
- `T2500 Q01`
  - dense: `top1=0.052000`, `amortized=0.1035s`
  - chart-only warm: `0.044600`, `cand_frac=0.193328`, `0.1326s`
  - route-only warm: `0.044600`, `0.193328`, `2.5345s`
  - full warm: `0.044600`, `0.193328`, `0.0562s`
- `T40000 Q01`
  - dense: `top1=0.048850`, `amortized=9.5814s`
  - chart-only warm: `0.047325`, `cand_frac=0.183764`, `2.3856s`
  - route-only warm: `0.047325`, `0.183764`, `34.7146s`
  - full warm: `0.047325`, `0.183764`, `2.0185s`

## Interpretation
- Chart residency carries almost all of the operational rescue.
- Route-only residency does not materially help at either anchor point.
- The upper-bank `T40000 Q01` systems win survives under chart-only residency.
- The lower-bank `T2500 Q01` floor remains a full-warm claim on confirm.
- Top-1 and candidate fraction stayed unchanged across all residency states, so
  this is an execution-cost result, not a retrieval-quality change.
