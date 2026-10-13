# INC-0100 Product Phase Sparse Event Translation Confirm

## Packet
- Config:
  - `configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_prewarm_confirm.json`
  - `configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_confirm.json`
- Analysis:
  - `results/analysis/inc0100_product_phase_sparse_event_translation_prewarm_confirm.json`
  - `results/analysis/inc0100_product_phase_sparse_event_translation_confirm.json`
- Gate:
  - `docs/governance/gates/gate_20260312_110306.md`
  - `docs/governance/gates/gate_20260312_110324.md`

## Confirm Read
- `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` preserves the translated
  product retrieval signal while materially improving on the continuous routed
  product reference:
  - sparse-event routed:
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.11774s`
    - `amortized=0.16861s`
    - `event_gate_mean=0.3191`
    - `event_gate_active_frac=0.0`
  - continuous routed product:
    - `top1=0.0446`
    - `cand_frac=0.193328`
    - `online=0.27591s`
    - `amortized=0.33801s`
- Against dense exact at the same lower-bank `Q01` packet:
  - dense:
    - `top1=0.0520`
    - `amortized=0.16866s`
  - sparse-event routed:
    - `top1=0.0446`
    - `amortized=0.16861s`
- Interpretation:
  - the sparse-event translated point is positive/narrow
  - it clearly improves the routed product tradeoff
  - it only reaches dense exact on a knife-edge lower-bank amortized margin
  - the quality gap to dense exact remains unchanged from the routed product
    branch

## Decision
- Close `INC-0100` confirm positive/narrow.
- Promote `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500` as the first
  sparse-event translated retrieval reference.
- Record the result carefully:
  - positive on soft-sparse translated carry-forward
  - not yet a hard-firing event result
  - not yet a broad dense-frontier replacement
- The next honest mechanism question is whether hard threshold firing can stay
  stable on the fixed product route law, starting again on the proxy harness
  before any broader translated frontier remap.
