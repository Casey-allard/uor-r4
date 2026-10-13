# INC-0023: PHI_PHI_PHI Budget Compression

## Hypothesis
If `PHI_PHI_PHI` fixes the shell metric, then the family should no longer need the same angular budget as earlier adaptive branches.
A lower `K` should therefore reduce runtime while keeping route health and quality intact.

## Screen
- Config: `configs/proxy_transfer_inc0023_phi3_budget_screen.json`
- Analysis: `results/analysis/inc0023_phi3_budget_screen.json`
- Gate: `docs/governance/gates/gate_20260305_212430.md`

2-seed larger-subset means:
- `R0`
  - `mse=0.003911258`
  - `total=50.589s`
  - collapsed
- `PHI3_K25_D36_L065`
  - `mse=0.003902477`
  - `total=47.131s`
  - health pass
- `PHI3_K20_D36_L065`
  - `mse=0.003892983`
  - `total=45.955s`
  - health pass
- `PHI3_K16_D36_L065`
  - `mse=0.003902707`
  - `total=46.349s`
  - failed on `seed0_shell_pmax>0.850`
- `PHI3_K16_B2_D36_L065`
  - `mse=0.003913785`
  - `total=54.662s`
  - health pass, but slower and weaker

## Screen Decision
- Promote `PHI3_K20_D36_L065` to 4-seed confirm.
- Keep `PHI3_K25_D36_L065` in confirm as the current family reference.
- Kill `PHI3_K16_D36_L065` as a promotion candidate.
- Keep `PHI3_K16_B2_D36_L065` only as evidence that relaxing the floor is not the right cost path.

## Confirm
- Config: `configs/proxy_transfer_inc0023_phi3_budget_confirm.json`
- Analysis: `results/analysis/inc0023_phi3_budget_confirm.json`
- Gate: `docs/governance/gates/gate_20260305_213556.md`

4-seed larger-subset means:
- `R0`
  - `mse=0.003907888`
  - `total=53.154s`
  - collapsed
- `PHI3_K25_D36_L065`
  - `mse=0.003901309`
  - `total=48.472s`
  - health pass
- `PHI3_K20_D36_L065`
  - `mse=0.003893818`
  - `total=50.990s`
  - health pass

## Confirm Decision
- Do not promote `K20` over `K25`.
- Re-promote `PHI3_K25_D36_L065` as the stabilized proxy-transfer candidate.
- Treat the `K20` screen win as a screen-only effect until it reproduces under a control batch.

## Mechanistic Reading
The compression story is now sharper:
- `PHI_PHI_PHI` can be compressed somewhat, but not arbitrarily.
- `K=20` looked like a better cost point on the 2-seed screen, then lost to `K=25` on the 4-seed confirm.
- `K=16` is too close to the shell concentration wall.
- relaxing `adaptive_min_pair_bins` does not buy cheaper healthy routing here.

Interpretation:
- the family probably does need a minimum coarse angular capacity to stay geometrically healthy.
- budget compression is not dead, but it is not solved by simply shrinking `K`.

## Next Step
- Run a fairness control on `R0` vs `PHI3_K25_D36_L065` because the runtime relation moved materially between `INC-0022` and `INC-0023`.
- After that, test a phase-coupled shell law if the current lead still looks mechanistically incomplete.
