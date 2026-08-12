# INC-0099: Product Phase Sparse Event Proxy Pilot

## Status
Confirm completed positive/narrow on 2026-03-12.

## Trigger
`INC-0098` closed the translated chart-resident cost question cleanly:
- the fixed chart-resident product stack was already positive against dense at
  both lower and upper anchors
- the remaining gap to full warm was split between route-index build and
  retrieval search overhead, not a hidden route-materialization failure
- translated systems optimization was therefore no longer the highest-value
  branch

That reopened the kill ladder at sparse event-driven trainability.

## Branch Contract
- keep the confirmed `INC-0065` product route law fixed
- keep the confirmed `INC-0071` secondary-key law fixed wherever retrieval is
  used as a downstream reference
- keep the translated chart-resident stack frozen as the current hardware-side
  reference, not as a tuning target
- reopen mechanism work only through sparse event activation on top of the
  fixed product route law
- use soft thresholding first; do not start with hard discontinuous firing
- do not reopen translated bank/cache accounting inside this branch

## Implementation
- Added a proxy-only sparse-event controller in `tasks/router_proxy_eval.py`:
  - `event_gate_mode={off,soft_error}`
  - `event_gate_threshold`
  - `event_gate_tau`
- The controller measures prediction error per update and scales the local EMA
  update by a soft gate rather than changing the route family.
- Added summary metrics:
  - `event_gate_error_mean`
  - `event_gate_mean`
  - `event_gate_active_frac`
  - `event_gate_cost_proxy`
- Threaded the new surface through:
  - proxy CLI help
  - proxy cache payloads
  - proxy sweep route summaries
  - regression coverage

## Working Hypothesis
The fixed product route law is mature enough that the next real falsification
step is not another translated rescue; it is whether sparse event gating can be
added without immediate collapse.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0099_product_phase_sparse_event_proxy_screen.json`
  - `configs/proxy_transfer_inc0099_product_phase_sparse_event_proxy_confirm.json`
- Analyses:
  - `results/analysis/inc0099_product_phase_sparse_event_proxy_screen.json`
  - `results/analysis/inc0099_product_phase_sparse_event_proxy_confirm.json`
- Reports:
  - `docs/reports/INC0099_PRODUCT_PHASE_SPARSE_EVENT_PROXY_SCREEN.md`
  - `docs/reports/INC0099_PRODUCT_PHASE_SPARSE_EVENT_PROXY_CONFIRM.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_104037.md`
  - `docs/governance/gates/gate_20260312_104345.md`

## Screen Read
- The screen carried:
  - fixed no-fiber control `HOPF_BASE_K25_PHI`
  - fixed product reference `H4XH4_FIELD_A150`
  - sparse-event probes `H4XH4_FIELD_A150_EVT_T062` and
    `H4XH4_FIELD_A150_EVT_T070`
  - baseline `R0`
- `H4XH4_FIELD_A150_EVT_T070` was the only healthy sparse point:
  - `mse=0.003895`
  - `total_sec=6.888`
  - `shell_pmax=0.5662`
  - `event_gate_mean=0.3193`
  - `event_gate_cost_proxy=0.3193`
- `H4XH4_FIELD_A150_EVT_T062` stayed mechanism-live but missed the strict
  runtime gate:
  - `mse=0.003916`
  - `total_sec=10.022`
  - `event_gate_mean=0.5158`
  - `event_gate_active_frac=0.7719`
- The continuous product reference remained healthy but screened with an
  unstable runtime outlier, so the confirm packet is the authoritative read for
  branch closure.

## Confirm Read
- The confirm packet carried:
  - `HOPF_BASE_K25_PHI`
  - `H4XH4_FIELD_A150`
  - `H4XH4_FIELD_A150_EVT_T070`
  - `R0`
- `H4XH4_FIELD_A150_EVT_T070` held across 4 seeds:
  - `mse=0.0038966`
  - `total_sec=6.558`
  - `shell_pmax=0.5702`
  - `eval_shells=2.0`
  - `event_gate_mean=0.3191`
  - `event_gate_cost_proxy=0.3191`
  - health: passed
- The fixed continuous product reference also stayed healthy:
  - `mse=0.0039004`
  - `total_sec=7.427`
  - `shell_pmax=0.5702`
  - `event_gate_mean=1.0000`
- The sparse point therefore preserved route health and slightly improved both:
  - proxy quality versus the continuous product reference
  - runtime versus the continuous product reference
- Important nuance:
  - `event_gate_active_frac=0.0` for the promoted point because the gate stays
    below the hard `0.5` cutoff
  - the positive result is therefore soft-sparse, not hard-firing
  - the meaningful sparsity signal is the update-mass proxy
    `event_gate_mean≈0.319`, not the hard active fraction

## Reading
- Sparse event trainability is now positive on the proxy harness for the fixed
  product route law.
- The positive point is not a brittle shell-side artifact:
  - shell balance stays healthy
  - unseen-rate stays low
  - route structure remains the same fixed product branch
- The branch does not yet prove hard threshold firing is trainable.
- What it does prove is narrower and still important:
  - the product route can absorb a real soft event controller
  - update mass can drop to about `32%` of dense EMA pressure
  - quality and route health can stay intact on the proxy contract

## Decision
- Close `INC-0099` confirm positive/narrow.
- Promote `H4XH4_FIELD_A150_EVT_T070` as the current sparse-event proxy
  reference on the fixed product route law.
- Record the event result as soft-sparse rather than hard-firing:
  - do not overclaim end-to-end event discreteness from this branch
- Keep the translated chart-resident stack frozen as the downstream hardware
  reference.
- Move next to translated carry-forward of the fixed sparse-event point
  (`INC-0100`), not to more threshold shaving on the proxy harness.
