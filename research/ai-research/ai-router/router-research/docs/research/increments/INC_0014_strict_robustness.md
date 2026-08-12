# INC-0014: Larger-Subset Strict-Health Robustness

## Hypothesis
If `R5A_SG16_C10_D30` is a real transfer route rather than a lucky boundary point, it should stay healthy and materially faster than `R0` when both:
- the proxy subset grows
- the gate is enforced seed-by-seed across 4 seeds

## Motivation
`INC-0013` corrected the transfer-health rule and promoted `R5A_SG16_C10_D30` only provisionally. The next step had to answer a harder question:
- does the route survive larger-subset pressure under strict seed-wise health
- and if it does, is it still the best healthy branch once `SG18`, `T080`, and `C06` are retested fairly

## Config
- `configs/proxy_transfer_inc0014_strict_robustness.json`

Key differences from the previous confirm stage:
- `max_train=5000`
- `max_eval=2500`
- `seeds=0,1,2,3`
- `enforce_seed_health=true`

## Evidence
- Analysis: `results/analysis/inc0014_strict_robustness.json`
- Gate note: `docs/governance/gates/gate_20260305_171526.md`

## Confirm Result
Larger-subset 4-seed strict-health comparison on PTB proxy:

- `R0`
  - `test_mse_after=0.0039079`
  - `total_sec=75.653`
  - `eval_shells=1.0`
  - `shell_pmax=1.000`
  - raw-MSE baseline, fully collapsed
- `R5A_SG16_C10_D30`
  - `test_mse_after=0.0039185`
  - `total_sec=50.461`
  - `eval_shells=3.0`
  - `shell_pmax=0.579`
  - `unseen_rate=0.0005`
  - strict-health pass
- `R5A_SG18_C10_D35`
  - `test_mse_after=0.0039183`
  - `total_sec=50.966`
  - `eval_shells=3.0`
  - `shell_pmax=0.658`
  - `unseen_rate=0.0007`
  - strict-health pass
- `R5A_SG16_C10_T080_D35`
  - `test_mse_after=0.0039128`
  - `total_sec=58.827`
  - `eval_shells=2.5`
  - `shell_pmax=0.757`
  - failed strict review because `seed1` and `seed2` crossed `shell_pmax=0.85`
- `R5A_SG16_C06_D35`
  - `test_mse_after=0.0039326`
  - `total_sec=52.589`
  - `eval_shells=7.0`
  - `shell_pmax=0.482`
  - `unseen_rate=0.0009`
  - strict-health pass

## Decision
- Keep `R0` as the raw-MSE transfer baseline.
- Promote `R5A_SG16_C10_D30` as the hardware-efficiency transfer lead.
- Keep `R5A_SG18_C10_D35` as the closest challenger; it survives strict review at this larger subset.
- Demote `R5A_SG16_C10_T080_D35`; it fails the strict seed-wise shell concentration rule at scale.
- Keep `R5A_SG16_C06_D35` as the high-dispersion healthy comparison branch, not the lead.

## Interpretation
This increment clarifies the transfer story:

1. `R0` still owns raw proxy MSE.
2. The adaptive branch now owns the healthy hardware-efficiency frontier.
3. `D30` is selected over `SG18`, not because it is dramatically better, but because it is:
   - slightly faster
   - slightly less concentrated
   - slightly lower unseen-rate
   - effectively tied on larger-subset MSE
4. `T080` looked conservative at smaller scale, but larger-subset seed-wise review showed that the interior `D35` target-shift branch still breaches shell concentration on some seeds.

This is the first increment where the transfer branch does not need to claim raw-MSE leadership to justify promotion.
The promotion is now:
- healthy routing
- materially better runtime
- bounded quality regression inside the configured tolerance

That aligns better with the actual project objective: reducing hardware pressure without route collapse.

## What Changed In The Project Direction
Before this increment:
- transfer lead = provisional `R5A_SG16_C10_D30`
- open problem = does the low-`delta_r` branch survive larger-subset strict review

After this increment:
- transfer quality baseline = `R0`
- transfer hardware-efficiency lead = `R5A_SG16_C10_D30`
- closest healthy challenger = `R5A_SG18_C10_D35`
- open problem = discriminate the `D30` vs `SG18` ridge mechanistically instead of treating them as random hyperparameter neighbors

## Next Increment
`INC-0015`: discriminate the `D30` vs `SG18` hardware-efficiency ridge.

Priority questions:
- why does lower `delta_r` beat the stronger-shell `SG18` branch on runtime and route health while staying tied on quality
- is the `D30` lead really a shell-width law, or is it compensating for a still-missing local merge rule
- can the project derive an explicit radial merge law that reproduces the `D30` effect without relying on one hand-tuned shell width
