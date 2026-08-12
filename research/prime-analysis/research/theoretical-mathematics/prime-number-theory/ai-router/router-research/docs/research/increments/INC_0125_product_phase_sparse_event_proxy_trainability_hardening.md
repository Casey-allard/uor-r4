# INC-0125: Product Phase Sparse Event Proxy Trainability Hardening

## Status
Completed positive/explanatory on 2026-03-12.

## Trigger
The downstream `INC-0118` to `INC-0124` branch sequence froze a useful
inheritance contract, but it did not materially advance the kill ladder.

The highest-value unresolved question we could test with the current harness
was sparse event-driven trainability under a materially harder proxy load:
- `INC-0099` proved a healthy soft-sparse proxy point
- `INC-0101` proved a healthy near-hard proxy point
- `INC-0102` showed that the sharper controller did not survive as a
  translated systems lead

This branch hardened the proxy side before reopening any translated sparse
event work.

## Evidence
- Screen config:
  `configs/proxy_transfer_inc0125_product_phase_sparse_event_proxy_trainability_hardening_screen.json`
- Confirm config:
  `configs/proxy_transfer_inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json`
- Screen analysis:
  `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_screen.json`
- Confirm analysis:
  `results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json`
- Gate notes:
  - `docs/governance/gates/gate_20260312_184338.md`
  - `docs/governance/gates/gate_20260312_184913.md`
- Report:
  `docs/reports/INC0125_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRAINABILITY_HARDENING.md`

## Hardening Setup
- fixed product route law from `INC-0065`
- fixed sparse-event controllers only:
  - `H4XH4_FIELD_A150_EVT_T070`
  - `H4XH4_FIELD_A150_EVT_T070_TAU002`
  - `H4XH4_FIELD_A150_HARD_T062`
- harder proxy load:
  - `max_train=10000`
  - `max_eval=5000`
  - `epochs=2`
- four-seed confirm with the existing route-health gate

## Reading
- The fixed sparse-event family remains trainable and health-passing under the
  harder proxy load.
- `H4XH4_FIELD_A150_EVT_T070_TAU002` is the clean near-hard winner:
  - confirm `mse=0.003859`
  - `total_sec=10.213`
  - `event_gate_mean=0.020038`
  - `event_gate_active_frac=0.0`
- `H4XH4_FIELD_A150_EVT_T070` remains a healthy soft-sparse controller:
  - confirm `mse=0.003895`
  - `total_sec=10.184`
  - `event_gate_mean=0.318959`
  - `event_gate_active_frac=0.0`
- `H4XH4_FIELD_A150_HARD_T062` remains mostly-on rather than genuinely hard
  sparse:
  - confirm `mse=0.003892`
  - `event_gate_mean=0.840375`
  - `event_gate_active_frac=0.840375`
- The generic `proxy_sweep` recommendation against `R0` is not the branch
  acceptance rule here. `R0` itself fails the shell-health gate on this
  harder load.

## Decision
- Close `INC-0125` positive/explanatory.
- Promote `H4XH4_FIELD_A150_EVT_T070_TAU002` as the hardened near-hard sparse
  event proxy reference.
- Keep `H4XH4_FIELD_A150_EVT_T070` as the healthy softer sparse comparator.
- Keep `H4XH4_FIELD_A150_HARD_T062` as a mostly-on hard-gate comparator only.
- Do not reopen the downstream packet-manifest loop from this branch.

## Next Honest Branch
- `INC-0126`: product phase sparse event proxy/translation gap audit.
  - `INC-0125` says the near-hard proxy point is real under harder load.
  - `INC-0102` still says the translated near-hard carry-forward fails.
  - The next real question is why the sharpened controller survives on proxy
    trainability but not on translated systems.

## Resume Note
Resume from the completed `INC-0125` confirm artifact and treat the old
downstream `INC-0125` packet-manifest idea as superseded. The live question is
the proxy/translation sparse-event gap, not more downstream packaging.
