# Prime Transport Sigma Family Holonomy v1

## Purpose

Implement the missing family-level sigma holonomy/composition law so the direct
sigma update family acts more coherently around radial transport.

This step does not rebuild operators and does not run any behavior benchmark.
It only updates the sigma-family action.

## Primitive

Implemented primitive:

- `sigma_family_holonomy_law`

Implemented in:

- [geometry_native_spinH_core_v5.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v5.py)

The primitive is native to sigma update logic. It is not an after-the-fact
repair filter.

## Direct Sigma Carrier

`spin_H_core_v5 = (theta, rho, sigma_direct, h)`

with:

- `theta = (b, phi)`
- `rho = (r, unfolding_load, radial_direction, radial_target, radial_target_phi)`
- `sigma_direct = (current_mode, fiber_mode, radial_mode, regressive_phase, family_holonomy_class)`
- `h = (recursive_phase, fiber_recursive_phase, radial_recursive_phase, holonomy_bit)`

The added family-level carrier is:

- `family_holonomy_class`

This is the shared composition carrier for radial interaction with lift,
coupled action, and twist.

## Law

For direct sigma update under generator `G`:

- `sigma' = sigma_family_holonomy_law(sigma, G, proposed_sigma_G)`

where:

1. `proposed_sigma_G` is the direct one-step update candidate from the native
   `R_G` map
2. `family_holonomy_class` advances by a generator charge:
   - `hold -> 0`
   - `T_b -> 1`
   - `T_x -> 2`
   - `T_c -> 3`
   - `T_y -> 4`
   - `T_z -> 5`
   - `T_r -> 3`
3. For holonomy-active generators:
   - `T_c`
   - `T_y`
   - `T_z`
   - `T_r`
   
   sigma is constrained through one shared invariant orbit:
   - `O(sigma) = sort{current_mode, fiber_mode, radial_mode}`
   
   and one shared bridge word:
   - `B(sigma) = rotate(mix(O_0, O_max, family_holonomy_class), family_holonomy_class)`
4. The holonomy-active maps use that same orbit/bridge carrier rather than
   recomputing different local normal forms per generator order
5. The regressive phase update is additive on the family carrier:
   - `phase' = phase + charge(G) + sum(B(sigma)) mod 12`

This is the implemented family-level holonomy/composition law.

## Covered Compositions

The law explicitly constrains:

- `R_Tz ∘ R_Tr`
- `R_Tr ∘ R_Tz`
- `R_Tc ∘ R_Tr`
- `R_Tr ∘ R_Tc`
- `R_Ty ∘ R_Tr`
- `R_Tr ∘ R_Ty`

It does not force naive commutativity globally. It imposes one shared radial
holonomy carrier so these compositions act through the same sigma-family law.

## Measurements

On the same bounded lawful surface:

- primary states examined: `45`
- distinct parent states reached: `38315`
- collision count: `0`
- recursive consistency rate: `1.0`

Exact agreement counts:

- `R_Tz ∘ R_Tr` vs `R_Tr ∘ R_Tz`: `27605 / 38315`
- `R_Tc ∘ R_Tr` vs `R_Tr ∘ R_Tc`: `38315 / 38315`
- `R_Ty ∘ R_Tr` vs `R_Tr ∘ R_Ty`: `22174 / 38315`

Direct improvement vs `spin_H_core_v4`:

- lift-radial agreement improvement: `+27605`
- coupled-radial agreement improvement: `+38315`
- twist-radial agreement improvement: `+13191`
- collision change: `0`
- recursive-consistency change: `0`

## Reading

The result is mechanical:

- one-step sigma maps were not the main problem
- the missing piece was a shared family carrier for radial composition
- once lift, coupled, twist, and radial maps use the same holonomy carrier,
  radial-family coherence increases materially without changing collisions or
  recursive consistency

## Honesty

Was a family-level sigma holonomy/composition law actually implemented?

- yes

Did it materially improve radial-family composition coherence?

- yes

Is full exact `spin_H` now present?

- no

## Next Step

Rebuild the three sigma-dependent operators against `spin_H_core_v5` and only
then run the first bounded behavior comparison.
