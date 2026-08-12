# INC-0024: Phase-Coupled Shell Pilot

## Hypothesis
If shell boundaries should move with phase pressure instead of staying purely radial, then a modest signed phase bias should improve the `PHI_PHI_PHI` family without breaking shell health.

## Mechanism
- Keep the coarse family fixed:
  - `phase4d_adaptive`
  - `adaptive_converge_mode=phi_ladder`
  - `shell_mode=phi_log` baseline
- Replace only the shell metric for the pilot routes:
  - `shell_mode=phi_phase`
  - `shell_phase_coupling in {0.20, 0.35}`
- Phase coupling acts as a signed shell-boundary shift derived from relative phase pressure.

## Why This Branch Exists
`CTRL-0002` demoted the runtime-win claim for the coarse `PHI_PHI_PHI` family.
The most coherent next geometry question is whether shells should behave as phase-shifted objects rather than purely radial shells with angular modulation layered on top.

## Protocol
- Config:
  - `configs/proxy_transfer_inc0024_phase_shell_screen.json`
- Stage:
  - screen, then confirm for the promoted candidate
- Seeds:
  - `0,1`
- Run order:
  - `seed_major`
- Comparators:
  - `PHI3_K25_D36_L065`
  - `R0`

## Promotion Criteria
Promote a phase-coupled candidate only if it:
- passes the configured seed-wise health gate
- improves on `PHI3_K25_D36_L065` on runtime or MSE without creating a new concentration problem
- remains within the configured MSE/runtime bounds vs `R0`

## Expected Failure Modes
- shell concentration rises because the phase term over-biases one side of the manifold
- unseen-route rate rises because shell motion fragments the address field
- runtime worsens because the new shell law increases route complexity without enough routing gain

## Notes
This is intentionally a small pilot.
If the branch is live, the next step is a confirm plus phase-shift law cleanup.
If it is dead, the project should stop treating shell-phase coupling as the main path and look for cheaper cost-side changes instead.


## Screen Result
- Analysis:
  - `results/analysis/inc0024_phase_shell_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_221026.md`
- 2-seed seed-major means:
  - `R0`
    - `mse=0.003911258`
    - `total=52.145s`
    - collapsed
  - `PHI3_K25_D36_L065`
    - `mse=0.003921162`
    - `total=68.920s`
    - runtime gate fail
  - `PHASE_K25_C020`
    - `mse=0.003916593`
    - `total=53.645s`
    - health pass
  - `PHASE_K25_C035`
    - `mse=0.003912563`
    - `total=54.184s`
    - health pass
- Decision:
  - promote `PHASE_K25_C035` to confirm
  - keep `PHASE_K25_C020` as a weaker stability comparator only
  - treat the result as evidence that phase-coupled shells are a live branch, not just a theoretical idea

## Confirm Result
- Config:
  - `configs/proxy_transfer_inc0024_phase_shell_confirm.json`
- Analysis:
  - `results/analysis/inc0024_phase_shell_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260305_222202.md`
- 4-seed seed-major means:
  - `R0`
    - `mse=0.003907888`
    - `total=46.405s`
    - collapsed
  - `PHASE_K25_C035`
    - `mse=0.003916993`
    - `total=53.423s`
    - `runtime_ratio_vs_r0=1.151`
    - failed only on the strict runtime gate
  - `PHI3_K25_D36_L065`
    - `mse=0.003917867`
    - `total=57.203s`
    - failed by a wider runtime margin

## Decision
- Promote `PHASE_K25_C035` over `PHI3_K25_D36_L065` as the current routed family lead candidate.
- Keep `R0` as the transfer control baseline and operational runtime preference.
- Treat continuous shell-phase coupling as a real mechanism, but not yet the finished form.

## Next Increment
`INC-0025`: sparse / quantized shell-phase law.

Goal:
- keep the routed-family gain from phase coupling
- reduce runtime cost by applying phase shifts only where the shell boundary should actually move
