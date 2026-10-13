# Prime Transport Operator From spin_H_core_v5

## Purpose

Rebuild exactly one operator component:

- `T_c`

from:

- [geometry_native_spinH_core_v5.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v5.py)

This step uses the direct sigma law with the family-level holonomy/composition
constraint and does not run any behavior benchmark.

## Rebuilt Component

Implementation:

- [geometry_native_operator_model_v19.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_operator_model_v19.py)

Runner:

- [run_geometry_native_operator_model_v19.py](/Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_geometry_native_operator_model_v19.py)

The rebuilt `T_c` uses:

- `theta` for coupled base-step selection
- `rho` for radial/unfolding contribution
- `sigma_direct` through `sigma_update_v5(..., "coupled_torus_kick")`
- `h` for recursive and holonomy-aware tau transport

The sigma-dependent part is materially active because the component now reads:

- `current_mode`
- `fiber_mode`
- `radial_mode`
- `regressive_phase`
- `family_holonomy_class`

from the direct sigma successor under the family holonomy law.

## Measurements

On the bounded lawful surface:

- reachable state count: `119785`
- total nonzero transitions: `838495`
- lawful transition fraction: `1.0`
- illegal transition fraction: `0.0`
- distinct class identities reached: `31465`
- distinct spin classes reached: `16`
- distinct radial classes reached: `3`

Material usage checks:

- `sigma_direct` under family holonomy materially active: `yes`
- `theta` materially used: `yes`
- `rho` materially used: `yes`
- `h` materially used: `yes`

## Comparison vs Prior T_c Rebuild From spin_H_core_v4

Direct comparison against:

- [prime_transport_operator_from_spinH_core_v4.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_from_spinH_core_v4.csv)

Changes:

- state count change: `-5892`
- transition count change: `-41244`
- class diversity change: `-2766`
- spin diversity change: `0`
- radial diversity change: `0`

## Reading

This rebuild is lawful and structurally honest, but it is not a gain step.

So the current family holonomy law is strong enough to improve sigma-family
composition, yet not strong enough to improve the coupled operator surface when
inserted directly into `T_c`.

## Honesty

Was one operator component rebuilt directly from `spin_H_core_v5`?

- yes

Was the rebuilt component lawful?

- yes

Did it materially use the improved sigma family law?

- yes

## Next Step

Identify the next missing ingredient before rebuilding more operators.
