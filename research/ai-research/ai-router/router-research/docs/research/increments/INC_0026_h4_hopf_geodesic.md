# INC-0026: H4-Hopf Geodesic Pilot

## Status
Slice A diagnostics implemented and pilot screen completed.

## Hypothesis
The current routed family is missing the true shell-sector growth law of a 4D hyperbolic polar object.
If routing is reformulated as `H^4` geodesic-polar routing with Hopf-aware angular allocation, the project may recover the missing efficiency gain more cleanly than with more local shell-controller tuning.

## Core Idea
Replace:
- mostly global `K`
- ad hoc `phi^balance` pair allocation

with:
- shell-dependent angular capacity
- `chi` latitude on `S^3`
- Hopf-aware `k1, k2`

## Minimal Scope
1. add diagnostics for `chi = atan2(rho2, rho1)`
2. derive `K_shell(r)` from a capped hyperbolic growth law
3. allocate `k1, k2` from `chi`
4. keep current convergence controller fixed

## First Implementation Slice
Do not try to land the whole geometry in one step.

### Slice A: diagnostics only
- compute and emit:
  - `chi_mean`
  - `chi_entropy`
  - `hopf_k1_mean`
  - `hopf_k2_mean`
  - `shell_target_capacity_mean`
- compare those diagnostics against the current `phi^balance` allocation

### Slice A Result
- Implemented diagnostics in:
  - `hyperbolic_router_so8.py`
  - `tasks/router_proxy_eval.py`
  - `tools/proxy_sweep.py`
- Diagnostic batch:
  - `configs/proxy_transfer_inc0026_hopf_diag.json`
  - `results/analysis/inc0026_hopf_diag.json`
  - `docs/governance/gates/gate_20260305_230403.md`
- Mean diagnostic read on the routed family:
  - `chi_mean ~= 0.33`
  - `chi_entropy ~= 0.83`
  - `hopf_shell_capacity ~= 9.0`
  - current `k1,k2 ~= 4.3,4.3`
  - Hopf target `k1,k2 = 3,3`
- Interpretation:
  - the current route is stably over-allocating angular pair bins relative to the capped `H^4` shell-capacity diagnostic
  - the mismatch is in total shell-dependent angular capacity more than in unstable `chi` estimation

### Slice B Result: minimal pilot
- Added:
  - `sector_mode=phase4d_hopf`
- Pilot screen:
  - `configs/proxy_transfer_inc0026_hopf_pilot_screen.json`
  - `results/analysis/inc0026_hopf_pilot_screen.json`
  - `docs/governance/gates/gate_20260305_231636.md`
- Mean result:
  - `HOPF_K25_BASE`: `mse=0.0038888`, `total=75.63s`, `buckets=10.5`, `shells=3.0`, `sectors=4.0`, `sector_pmax=0.697`
  - `PHASE_K25_C035`: `mse=0.0039095`, `total=72.92s`
  - `PHI3_K25_D36_L065`: `mse=0.0039171`, `total=64.67s`
  - `R0`: `mse=0.0039113`, `total=52.47s`
- Interpretation:
  - direct Hopf shell-capacity coupling is a real quality signal
  - the pure pilot compresses too hard
  - the runtime regression means shell-capacity coupling alone is not enough
  - the missing architectural piece is likely explicit `chi` representation or a blended shell-capacity law, not more shell compression

### Slice B: allocation pilot
- add a new sector mode:
  - `phase4d_hopf`
- use:
  - `theta1`
  - `theta2`
  - `chi`
- keep shell controller and growth logic unchanged for the first pilot

### Slice C: shell-capacity coupling
- replace fixed/global local `K` with shell-dependent capped `K_shell(r)`
- do not add shell-phase coupling to the first `H4` pilot

## Initial Acceptance Signal
The first `H4` pilot is worth continuing only if it shows at least one of:
- lower concentration than `PHASE_K25_C035` at similar quality
- better runtime than `PHASE_K25_C035` at similar health
- clearer mechanistic diagnostics showing current `phi^balance` is misallocating angular capacity

## Decision
- Keep the `H4` branch alive.
- Do not promote pure `phase4d_hopf` as-is.
- Next move is not another shell/controller tweak.
- Next move is a low-cardinality `chi` axis or blended Hopf-capacity branch so the route does not collapse angular capacity into only `4` effective sectors.

## Why This Is First
The deep math review suggests the current bottleneck is global geometry mismatch, not a missing local shell correction.
