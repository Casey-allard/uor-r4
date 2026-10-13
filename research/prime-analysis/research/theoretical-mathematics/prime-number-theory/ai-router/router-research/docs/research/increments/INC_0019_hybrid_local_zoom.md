# INC-0019: Hybrid Complex Local Zoom

## Hypothesis
If `phase4d_adaptive` is a good coarse router but still leaves local neighborhood discrimination on the table, then a local `complex2` refinement inside the chosen coarse neighborhood should improve the routed frontier without paying the full cost of global complex routing.

The tested mechanism was:
- coarse route with `phase4d_adaptive`
- local refinement with discrete complex multiplication through an imaginary-field neighborhood step
- combine `coarse_sector * local_k + local_sector` into one sector address

## Implementation
A new route mode was added:
- `sector_mode=phase4d_complex_local`

New CLI flags:
- `--hybrid_local_k`
- `--hybrid_complex_roots`

Interpretation:
- `hybrid_complex_roots=4` means the local complex plane is rotated by discrete `i^n` root-of-unity steps derived from the coarse sector before local quantization
- the local zoom uses in-shell radial fraction plus local angle bins to avoid batch-dependent normalization

## Config
- Screen: `configs/proxy_transfer_inc0019_hybrid_screen.json`
- Execution order: `seed_major`

## Evidence
- Analysis: `results/analysis/inc0019_hybrid_screen.json`
- Gate note: `docs/governance/gates/gate_20260305_191832.md`

## Screen Result
2-seed screen (`train=3000`, `test=1500`):
- `R0`
  - `mse=0.00394501`
  - `total=27.740s`
  - collapsed baseline
- `PHI_D32_L120`
  - `mse=0.00393778`
  - `total=29.284s`
  - `shells=3.5`
  - `unseen=0.00033`
  - health pass
- `HYB_L4_R4`
  - `mse=0.00395766`
  - `total=31.961s`
  - `shells=6.0`
  - `unseen=0.01467`
  - health fail on unseen-route exposure
- `HYB_L9_R4`
  - `mse=0.00397838`
  - `total=34.356s`
  - `shells=9.0`
  - `unseen=0.03367`
  - health fail on unseen-route exposure and runtime

## Decision
- Do not promote the hybrid branch to confirm.
- Keep `PHI_D32_L120` as the screened healthy routed branch.
- Hold `phase4d_complex_local` as a mechanism candidate, not a lead candidate.

## Interpretation
The first hybrid implementation is doing the wrong thing operationally:
- it increases shell/sector richness
- but it opens too many unseen addresses
- the local zoom is therefore acting like unchecked fragmentation, not controlled refinement

This is a coherent failure mode, not a random miss:
- `L4` already fails on unseen-route exposure
- `L9` makes the same problem much worse
- the branch is therefore limited by local convergence / merge control, not by lack of capacity

So the current reading is:
- coarse `phase4d_adaptive` is strong enough to keep
- local complex refinement is not yet usable without an explicit local convergence prior

## Next Branch
Do not run a plain larger-`local_k` confirm.

If this branch is revived, it should be through:
- a local convergence / merge law
- a much stricter local address budget
- or a smaller local refinement surface such as `local_k=2`
