# INC-0097: Product Phase Sparse-Gated Shell Pilot

## Status
Screen completed negative on 2026-03-12.

## Trigger
`INC-0096` stabilizes the translated chart-resident lower-bank single-query
claim across packet composition:
- focused `Q01` at `T2500` is positive
- mixed-packet `Q01` at `T2500` is also positive on hardening

That clears the deferred route-matrix gate. The next honest geometry branch can
reopen sparse or quantized phase-gated shell work, instead of spending more
time on packet or bank accounting.

## Branch Contract
- keep the confirmed `INC-0065` route law fixed as the product-phase routing
  baseline
- keep the confirmed `INC-0071` secondary-key law fixed for downstream
  translated evaluation references
- keep the confirmed `INC-0083` persistent-cache implementation fixed
- keep the stable translated chart-resident lower-bank point from `INC-0096` as
  the systems reference, not as a tuning target
- reopen geometry only through local sparse / gated shell-capacity activation,
  not another global widening pass
- first gate candidates should follow the current theory guidance:
  - `chi` pressure
  - Hopf shell-capacity threshold
  - local route concentration
- if threshold gating still over-concentrates or costs too much, the discrete
  fallback is a small shared `phi^2` rung-state / banded-lattice family
- do not reopen packet-scope or bank-boundary work inside this branch

## Minimal Scope
1. Add one sparse or gated shell candidate on top of the fixed product-phase
   baseline.
2. If feasible in the same screen packet, add one small shared-state discrete
   candidate as the `Q2l` fallback.
3. Screen first on the proxy harness against the current continuous reference.
4. Carry translated retrieval only if at least one candidate is healthy and
   operationally plausible.

## Stop Rule
- If no sparse / gated candidate beats the current continuous reference on
  health plus runtime/quality tradeoff, close the branch and return to
  cost/hardware decomposition rather than more shell-law tuning.
- If at least one sparse / gated or shared-state candidate is healthy and
  materially improves shell concentration or runtime, promote it to confirm.

## Resume Note
Resume from the corrected `INC-0097` screen artifact and treat the next branch
as translated route-cost decomposition (`INC-0098`), with the translated
chart-resident stack kept as the downstream systems reference after the sparse
shell pilot failed health.

## Evidence
- Configs:
  - `configs/proxy_transfer_inc0097_product_phase_sparse_gated_shell_screen.json`
- Analyses:
  - `results/analysis/inc0097_product_phase_sparse_gated_shell_screen.json`
- Reports:
  - `docs/reports/INC0097_PRODUCT_PHASE_SPARSE_GATED_SHELL_SCREEN.md`
- Gate notes:
  - `docs/governance/gates/gate_20260312_095747.md`

## Screen Read
- The authoritative screen is the rerun after the shell-controller args were
  threaded through `optimize_chart(...)`; the earlier failed invocation is not
  evidence for this branch.
- Healthy continuous reference:
  - `H4XH4_FIELD_A150`
    - `mse=0.003899`
    - `total_sec=7.956`
    - `eval_shells=2.0`
    - `shell_pmax=0.5662`
    - `phase_transport_field_shift_abs_mean=0.01451`
- Sparse pilot candidates stayed mechanism-live, but both over-concentrated
  shell mass:
  - `H4XH4_FIELD_A150_G030`
    - `mse=0.003915`
    - `total_sec=7.127`
    - `eval_shells=2.0`
    - `shell_pmax=0.9846`
    - `product_shell_active_frac=0.0640`
    - `product_shell_gate_score_mean=0.02461`
    - `product_shell_multiplier_mean=1.0706`
    - health: failed
  - `H4XH4_FIELD_A150_B035`
    - `mse=0.003908`
    - `total_sec=7.463`
    - `eval_shells=1.5`
    - `shell_pmax=0.9860`
    - `product_shell_active_frac=0.0732`
    - `product_shell_states_used=2.5`
    - `product_shell_multiplier_mean=1.0445`
    - health: failed
- Failure mode:
  - gated and banded shell control change only about `6%-7%` of routed points
  - even that small activation is enough to collapse shell balance
  - neither candidate beats the continuous product reference on the combined
    health plus runtime/quality screen criterion

## Reading
- Sparse / quantized shell control is live as a mechanism on top of the fixed
  product route law.
- The regression is shell-side, not phase-side:
  - field-shift metrics remain nonzero
  - sector structure stays close to the healthy continuous reference
  - the collapse is concentrated in shell occupancy (`shell_pmax≈0.985`)
- This branch hit its explicit stop rule:
  - no healthy sparse or shared-state candidate beat the fixed continuous
    product reference
  - more threshold tuning would be local knob-search, not a new mechanism test

## Decision
- Close `INC-0097` negative at screen stage.
- Keep `H4XH4_FIELD_A150` as the fixed healthy product-phase reference.
- Do not carry the gated or banded shell variants into translated retrieval.
- Return next to cost/hardware decomposition on the fixed chart-resident
  translated stack (`INC-0098`) rather than more shell-law tuning.
