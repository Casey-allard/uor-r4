# INC0126 Product Phase Sparse Event Proxy Translation Gap Audit

## Summary
- Outcome: `translated_systems_cost_branch`
- Quality preserved vs translated soft sparse: `True`
- Candidate omission supported: `False`
- In-candidate ordering loss supported: `False`
- Event-gate overhead supported: `True`
- Route/search interaction cost supported: `True`

## Proxy Read
- Near-hard proxy `H4XH4_FIELD_A150_EVT_T070_TAU002`: `mse=0.003859`, `total_sec=10.213`, `event_gate_mean=0.020038`, `event_gate_active_frac=0.000000`
- Soft sparse proxy `H4XH4_FIELD_A150_EVT_T070`: `mse=0.003895`, `total_sec=10.184`, `event_gate_mean=0.318959`, `event_gate_active_frac=0.000000`

## Translated Read
- Near-hard translated `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`: `top1=0.044400`, `cand_frac=0.189016`, `online=0.215612`, `amortized=0.263435`
- Soft sparse translated `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500`: `top1=0.044400`, `cand_frac=0.189016`, `online=0.117009`, `amortized=0.145747`
- Continuous translated `CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500`: `top1=0.044400`, `cand_frac=0.189016`, `online=0.081392`, `amortized=0.118415`

## Gap Attribution
- Near-hard vs soft sparse top-1 delta: `0.000000`
- Near-hard vs soft sparse candidate-fraction delta: `0.000000`
- Near-hard vs soft sparse online delta: `0.098602`
- Near-hard vs soft sparse amortized delta: `0.117689`
- Near-hard vs soft sparse route-query delta: `0.011108`
- Near-hard vs soft sparse route-index-build delta: `0.018890`
- Near-hard vs soft sparse retrieval-search delta: `0.087495`
- Primary runtime driver: `retrieval_search_sec`
- Runtime shares vs soft sparse: query `0.112652`, index `0.191576`, search `0.887348`
- Near-hard vs continuous top-1 delta: `0.000000`
- Near-hard vs continuous candidate-fraction delta: `0.000000`
- Near-hard vs continuous online delta: `0.134220`

## Interpretation
- The near-hard controller survives as a sparse and healthy proxy mechanism, and it keeps the same translated top-1 plus candidate fraction as the translated soft-sparse and continuous references. The translated failure is therefore a systems-cost problem, not a candidate-quality or trainability problem.
