# Adaptive Phase4D Spec

## Purpose
Define a fluid widening/convergence routing field for the 4D phase route so the geometry can expand with time instead of relying on a fixed square `K`.

## Coordinates
Given charted tangent coordinates `z in R^d`, choose two complex phase pairs:
- `q1 = z_i + i z_j`
- `q2 = z_k + i z_l`

Angles:
- `theta1 = atan2(Im(q1), Re(q1)) in [-pi, pi]`
- `theta2 = atan2(Im(q2), Re(q2)) in [-pi, pi]`

Pair magnitudes:
- `rho1 = |q1|`
- `rho2 = |q2|`

Global radius:
- `r = ||z||`

## Time-Expanded Hyperbolic Field
The route treats the 4D object as expanding with training time `tau in [0,1]`.

Radial normalization:
- `r_hat = r / (r + delta_r)`

Divergence score:
- `D(tau, r) = 1 - exp(-gamma_t * pi * tau * r_hat)`

Interpretation:
- small `tau` or small `r` means low widening pressure
- large `tau` and larger `r` mean the local phase field is allowed to widen

## Phi-Balanced Pair Allocation
Let:
- `phi = (1 + sqrt(5)) / 2`
- `balance = (rho1 - rho2) / (rho1 + rho2 + eps)`

The adaptive sector budget is distributed anisotropically across the two phase pairs.

Minimum active budget:
- `K_floor = b_min^2`

Effective local budget:
- `K_eff = round(K_floor + (K - K_floor) * D(tau, r))`

Pair counts:
- `base = sqrt(K_eff)`
- `k1_raw = base * phi^(alpha_balance * balance)`
- `k2_raw = K_eff / k1_raw`

Then clamp:
- `k1 >= b_min`
- `k2 >= b_min`
- `k1 * k2 <= K`

Interpretation:
- if phase pair 1 carries more local energy, it gets more angular resolution
- if phase pair 2 collapses, the route does not waste equal square capacity on it

## Golden-Angle Divergence
Let the golden angle be:
- `gamma_phi = 2 * pi * (1 - 1/phi)`

Apply a mild time/radius-dependent phase warp:
- `delta_theta = alpha_angle * gamma_phi * tau * r_hat * balance`
- `theta1' = wrap(theta1 + delta_theta)`
- `theta2' = wrap(theta2 - delta_theta / phi)`

Interpretation:
- widening is not just more bins
- the field itself slightly diverges as time/radius increase
- golden-angle offsets reduce resonance with fixed bin boundaries

## Sector Assignment
Normalize to unit interval:
- `u1 = (theta1' + pi) / (2*pi)`
- `u2 = (theta2' + pi) / (2*pi)`

Quantize locally:
- `b1 = floor(u1 * k1)`
- `b2 = floor(u2 * k2)`

Sector id:
- `sector = (b1 * k2 + b2) mod K`

This keeps sector ids bounded while allowing the local `(k1, k2)` field to vary with time and geometry.

## Divergence-Aware Shell Geometry
Sector widening alone is not enough if the radial field remains flat.

For the shell branch, the effective radius is expanded by the same divergence field:

- `shell_drive = D(tau, r) * (1 + beta_shell * |balance|)`
- `shell_expand = lambda_shell * shell_drive`
- `shell_overflow = max(shell_expand - (c_target + c_hyst), 0)`
- `shell_converge = lambda_conv * shell_overflow`
- `shell_mult = exp(shell_expand - shell_converge)`
- `r_eff = r * shell_mult`

Then shell assignment uses:
- `shell = floor(r_eff / delta_r)`

Interpretation:
- `lambda_shell` creates outward repulsion as time/radius increase
- `beta_shell` lets phase-pair imbalance widen shells faster in stressed regions
- `lambda_conv` only pushes back after shell expansion exceeds the allowed band
- `c_target` and `c_hyst` define that allowed band explicitly
- `delta_r` controls how finely the radial field is quantized, so it can act like a merge prior in the shell geometry

This matches the intended geometry more closely:
- divergence is the default behavior of the time-expanded field
- convergence is explicit, not accidental

## Convergence Rule
The same field also supports convergence:
- when `tau` is small or `r_hat` is small, `D(tau, r)` shrinks toward `0`
- `K_eff` falls back toward `K_floor`
- angular offsets shrink back toward zero
- shell expansion also falls back because `shell_mult` contracts toward the convergence term

So widening is reversible and local rather than permanently exploding the route count.

## Split/Merge Rules
This repo already has slot-level growth splitting. For adaptive phase routing, the geometry-level rules are:

### Split
Open more local angular resolution when all are true:
- local proxy error remains high
- local sector concentration is high
- `D(tau, r)` is large enough to justify widening

### Merge
Collapse neighboring local phase regions when all are true:
- prototype memories are similar
- local error difference is small
- `D(tau, r)` has dropped or remained low

### Hysteresis
Use separate thresholds for split vs merge, with a `phi` ratio gap, so the route does not chatter.
The current implementation approximates that behavior with the `c_target + c_hyst` overflow band before convergence activates.

## Current Implementation Scope
The initial code branch implements:
- time-expanded divergence via `D(tau, r)`
- phi-balanced anisotropic pair counts
- golden-angle phase offsets
- divergence-aware shell scaling via `shell_mult`

The current code does not yet maintain explicit geometry-level split/merge state across steps; it approximates that behavior through the time-varying local sector/shell field while keeping the memory system unchanged.

## Why This Branch Exists
The fixed `phase4d` route widened from `2` to only `4` active sectors even when `K` increased materially on proxy data.
That implies the right next step is not a larger static square grid, but a fluid anisotropic widening rule tied to time and local phase energy.

`INC-0011` extends that conclusion:
- angular widening alone was insufficient
- radial widening is also real and controllable
- the next problem is not activation, but convergence and hysteresis

`INC-0012` sharpens it further:
- stronger shell divergence is still useful
- but it needs an explicit overflow clip and a sane shell width
- in the current proxy regime, `delta_r=3.5` is part of the control law, not just a display parameter

`INC-0013` adds an important correction:
- the shell-control surface contains distinct regimes, not a single smooth optimum
- `delta_r=4.0` is on the collapse side
- the `D35` ridge can look strong on mean metrics while still breaching shell concentration on one seed
- lower `delta_r=3.0` currently gives the best strict seed-wise health / quality / runtime balance
- future transfer promotion therefore requires seed-wise shell-health, not just route means

`INC-0014` extends that reading at larger subset:
- `R0` still owns raw proxy MSE
- both `D30` and `SG18` survive strict seed-wise review
- `D30` wins the current hardware-efficiency frontier because it is faster and less concentrated while staying effectively tied on proxy quality
- the adaptive branch should now be read as a Pareto improvement in healthy routed efficiency, not as a universal raw-MSE winner

`INC-0015` and `INC-0016` reduce the mechanism further:
- the current controller sits in a saturated cap regime:
  - `shell_mult ≈ exp(c_target + c_hyst)`
- in that regime, changing `adaptive_shell_growth` does not materially change route-health or quality
- `delta_r` is the live radial control variable
- `delta_r=3.0` is the current optimum of the fixed-cap law

That makes the next branch much cleaner:
- keep `pi` for the angular/time field
- stop tuning shell-growth inside the current saturated law
- replace the fixed cap with a ratio-based merge/cap controller, where `phi` is now a plausible design constant

## Phi-Ratio Controller
`INC-0017` implements the first controller replacement:
- keep the divergence field unchanged
- keep the threshold band `c_target + c_hyst`
- replace the hard overflow cap with `phi`-scaled ratio pressure

Fixed controller:
- `overflow = max(shell_expand - (c_target + c_hyst), 0)`
- `shell_converge = lambda_conv * overflow`

Phi-ratio controller:
- `target_band = c_target + c_hyst`
- `ratio = shell_expand / target_band`
- `ratio_pressure = max(ratio - 1, 0)`
- `shell_converge = lambda_conv * target_band * ratio_pressure / phi`

Interpretation:
- fixed mode hard-clips the shell field when `lambda_conv=1`
- `phi_ratio` preserves convergence but leaves a positive post-threshold slope
- this makes the controller axis live again

Current empirical reading from `INC-0017`:
- `phi_ratio` is a real mechanism branch
- `L140` is too aggressive and trips seed-wise shell concentration
- `L100` is healthy but over-dispersed and slower
- `L120` is the best healthy `phi` compromise so far
- `D30_FIXED_SG16` still holds the hardware-efficiency lead, so `phi` has not replaced the fixed controller yet
- once the controller is unsaturated, the old fixed-regime `delta_r` optimum should be re-tested

## Phi Delta Retune
`INC-0018` runs the re-test that `INC-0017` implied:
- keep the `phi_ratio` controller fixed
- retune only `delta_r`

Empirical reading from `INC-0018`:
- the routed hardware-efficiency optimum moves from fixed `D30` to `phi` `D32`
- `PHI_D32_L120` is faster than both `R0` and `D30_FIXED_SG16` while staying inside the transfer-quality tolerance gate
- `PHI_D30_L120` remains the better routed-branch quality point
- `delta_r` therefore belongs to the controller law, not just the shell-index geometry

That changes the interpretation of the route:
- the fixed controller and the `phi` controller do not share one neutral radial optimum
- the radial law must be derived together with the convergence law
- a future discrete `phi` step-ladder controller only makes sense as a stabilization law on top of an already-live `phi` branch

## Seed-Major Control
`CTRL-0001` adds a timing-fairness control to the current routed lead:
- same larger-subset regime as `INC-0018`
- route execution interleaved by seed

Empirical reading:
- `PHI_D32_L120` remains the fastest healthy routed branch
- but the runtime edge vs `R0` is narrow under control
- therefore the `phi` lead survives, but should be read as controlled and narrow rather than dramatic

## Hybrid Local Convergence Rescue
`INC-0020` changes the hybrid local law in two ways:

1. local activation becomes ratio-based:
   - `local_ratio = local_drive / local_target_band`
   - `local_ratio_pressure = max(local_ratio - 1, 0)`
   - `local_activation = clip(local_ratio_pressure / phi - local_converge, 0, 1)`
2. local zoom gets a floor:
   - `local_k_eff = local_min_k + round((local_k - local_min_k) * local_activation)`

Interpretation:
- the original hybrid branch failed because it opened too much local space without any local convergence prior
- the first rescue attempt failed because the local controller was normalized against absolute shell-scale headroom and stayed effectively closed
- the ratio-based local controller fixes the scale mismatch
- `local_min_k=2` creates a stable two-way imaginary-field split even when local drive is weak

Empirical reading from `INC-0020`:
- `HYB4_M2_T010_C005` becomes the healthiest routed-quality branch
- `HYB4_M2_T005_C005` is slightly more open but slightly worse on quality
- hybrid local zoom no longer explodes unseen-route exposure
- hybrid local zoom still does not buy runtime

That changes the interpretation of Path 4:
- hybrid complex zoom is now a viable quality branch
- it is not yet a hardware-efficiency branch
- the next runtime-oriented mechanism should return to the controller law unless hybrid can be simplified materially

## Phi-Ladder Controller
`INC-0021` replaces the continuous `phi_ratio` overflow law with a quantized ladder:

- `target_band = c_target + c_hyst`
- `overflow = max(shell_expand - target_band, 0)`
- `ladder_steps = ceil(overflow / log(phi))`
- `shell_converge = lambda_conv * ladder_steps * log(phi)`

Interpretation:
- shell convergence is now additive in `log(phi)` units
- shell multiplier contraction is therefore multiplicative in `phi` units
- this is closer to the intended `+phi / -phi` discrete branching picture than the continuous `phi_ratio` law

Empirical reading from `INC-0021`:
- `LADDER_D32_L065` is the healthiest routed controller candidate in the current larger-subset screen
- the ladder is materially faster than `PHI_D32_L120` and healthier than the more aggressive ladder settings
- but no healthy routed branch beats `R0` on runtime yet

That means:
- the controller law improved
- the shell metric still did not
- the next branch should attack shell geometry directly rather than retuning the ladder

## Pi / Phi / Log(Phi) Separation
The current evidence now supports a strict division of labor:

- `pi`:
  - angular normalization
  - angular wrapping
  - polar / phase coordinate geometry
  - time-normalized divergence fields
- hyperbolic / exponential radial law:
  - continuous time-expanded shell growth
  - manifold-scale radial expansion
- `phi`:
  - self-similar branch spacing
  - anisotropic budget allocation
  - split / merge ratios
- `log(phi)`:
  - additive shell hysteresis
  - convergence ladders
  - merge / split thresholds in pressure space

This is a better reading than trying to force `phi` into every geometric component.

## Next Shell Metric Branch: Phi-Log Shells
Current shell assignment is still linear:
- `shell = floor(r_eff / delta_r)`

That is probably the wrong radial metric for an expanding hyperbolic field.

Proposed next branch:
- keep the divergence field and angular geometry as they are
- keep the discrete `phi_ladder` controller
- replace linear shells with `phi`-spaced log shells

Candidate shell law:
- `shell = floor(log(1 + r_eff / r0) / log(phi))`

Equivalent shell boundaries:
- `r_n = r0 * (phi^n - 1)`

Interpretation:
- shell spacing increases geometrically with radius
- shell transitions occur in the same multiplicative family as the controller
- this should reduce the mismatch between hyperbolic radial growth and linear shell indexing

This is now the highest-value next mechanism branch because `INC-0021` suggests the controller family is no longer the main bottleneck.

## INC-0022: PHI_PHI_PHI Confirmation
`INC-0022` closes the loop on the shell-metric proposal:
- keep `pi` in the angular field
- keep hyperbolic / exponential radial growth
- keep the discrete `phi_ladder` controller
- replace linear shell indexing with `phi`-spaced shells

Effective shell law:
- `shell = floor(log(1 + r_eff / delta_r) / log(phi))`

Empirical reading:
- `PHI_PHI_PHI v1` (artifact: `PHILOG_D36_L065`) is now the best healthy transfer-quality route in the repo
- it beats `R0` on quality
- it beats the linear-shell ladder comparator on both quality and runtime
- it stays healthy on shell usage
- it still loses to `R0` on absolute runtime

Interpretation:
- the shell metric mismatch was real
- the route now has a coherent `PHI_PHI_PHI` family:
  - `phi` ladder controller
  - `phi` shell spacing
  - `phi` step convergence
- the next bottleneck is not geometry coherence
- the next bottleneck is route cost

That changes the next branch:
- do not change families again immediately
- compress budget inside `PHI_PHI_PHI`
- only move to chart-cost decomposition if budget compression still cannot recover runtime
