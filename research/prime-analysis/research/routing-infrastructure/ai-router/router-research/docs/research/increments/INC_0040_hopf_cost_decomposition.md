# INC-0040: Hopf Cost Decomposition

## Status
Completed.

## Trigger
`CTRL-0003` resolved the frontier enough to stop guessing:
- `HOPF_K25_BASE` is the only routed branch that survived the strict 4-seed gate
- `HOPF_PHI2_BAND` keeps the widened quality signal, but missed the runtime bar
- the dominant cost is now explicit in the timings, not hypothetical

## Timing Read
4-seed mean timing breakdown from `CTRL-0003`:
- `HOPF_K25_BASE`
  - `chart_opt=39.959s`
  - `training_ema=4.143s`
  - `total=44.838s`
- `HOPF_PHI2_BAND`
  - `chart_opt=40.658s`
  - `training_ema=10.133s`
  - `total=51.541s`
- `R0`
  - `chart_opt=40.634s`
  - `training_ema=0.856s`
  - `total=42.409s`

## Hypothesis
The next meaningful efficiency gain will not come from another shell/sector law alone.
It will come from collapsing chart optimization cost and understanding why widened Hopf inflates training-time EMA cost.

## Minimal Scope
1. Add an explicit cost report for:
   - chart optimization
   - route evaluation
   - training EMA
   - growth
2. Screen cheap variants that only change cost structure, not geometry:
   - reduced chart schedule (`chart_iters`, candidate counts)
   - chart early stop
   - reduced slot pressure for widened Hopf
3. Compare only:
   - `HOPF_K25_BASE`
   - `HOPF_PHI2_BAND`
   - `R0`

## First Screen Plan
- Config:
  - `configs/proxy_transfer_inc0040_cost_screen.json`
- Cost-only variants:
  - `HOPF_K25_BASE_IT60_P4`
  - `HOPF_K25_BASE_ES12`
  - `HOPF_PHI2_BAND_IT60_P4`
  - `HOPF_PHI2_BAND_SLOT3`
  - `HOPF_PHI2_BAND_IT60_P4_SLOT3`
- Report artifact:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
- Tool:
  - `tools/cost_report.py`

## Screen Result
- Analysis:
  - `results/analysis/inc0040_cost_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260306_091429.md`
- 2-seed means:
  - `HOPF_K25_BASE_IT60_P4`
    - `test_mse_after=0.003925725`
    - `total_sec=20.532`
    - `chart_opt=11.214`
    - health pass
  - `HOPF_PHI2_BAND_IT60_P4`
    - `test_mse_after=0.003937058`
    - `total_sec=18.286`
    - `chart_opt=11.559`
    - health pass
  - `HOPF_PHI2_BAND`
    - `test_mse_after=0.003910130`
    - `total_sec=52.211`
    - health pass
  - `R0`
    - `test_mse_after=0.003946221`
    - `total_sec=46.039`
    - shell-collapse health fail
- Reading:
  - chart schedule, not routing evaluation, was the dominant cost lever
  - widened Hopf did not need slot-pressure reduction to become cheap; schedule reduction was enough
  - the screen justified a 4-seed confirm on the two reduced-schedule variants

## Confirm Result
- Config:
  - `configs/proxy_transfer_inc0040_cost_confirm.json`
- Analysis:
  - `results/analysis/inc0040_cost_confirm.json`
- Gate:
  - `docs/governance/gates/gate_20260306_092503.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT60_P4`
    - `test_mse_after=0.003919349`
    - `total_sec=19.905`
    - `chart_opt=12.421`
    - `training_ema=6.520`
    - health pass
  - `HOPF_PHI2_BAND_IT60_P4`
    - `test_mse_after=0.003928139`
    - `total_sec=18.270`
    - `chart_opt=10.387`
    - `training_ema=7.202`
    - health pass
  - `HOPF_PHI2_BAND`
    - `test_mse_after=0.003921230`
    - `total_sec=58.684`
    - `chart_opt=47.402`
    - `training_ema=10.341`
    - health fail on runtime
  - `R0`
    - `test_mse_after=0.003946853`
    - `total_sec=44.240`
    - shell-collapse health fail

## Decision
- Promote `HOPF_K25_BASE_IT60_P4` as the current operational routed lead:
  - better MSE than `HOPF_PHI2_BAND_IT60_P4`
  - much faster than `R0`
  - strict-gate pass on 4 seeds
- Promote `HOPF_PHI2_BAND_IT60_P4` as the current widened efficient lead:
  - fastest healthy routed branch on 4 seeds
  - preserves widened sector usage (`10` sectors, `16` buckets)
- Demote the old `HOPF_PHI2_BAND` reference:
  - quality remains live
  - runtime rescue is now fully superseded by the reduced schedule
- Next branch:
  - larger-subset cost-frontier confirm between the two reduced-schedule leads and `R0`

## Acceptance Signal
Keep any cost branch only if it preserves the current route-health outcome and materially reduces total runtime.

## Fallback
If cost decomposition does not reveal a clean runtime rescue, reopen geometry only where the cost diagnosis specifically points.
