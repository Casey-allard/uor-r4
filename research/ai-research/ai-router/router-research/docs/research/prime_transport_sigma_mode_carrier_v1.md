# Prime Transport Sigma Mode Carrier v1

## Purpose

This step refines the sigma parent-mode carrier inside:

- `spin_H_core_v3 = (theta, rho, sigma**, h)`

into:

- `spin_H_core_v4 = (theta, rho, sigma***, h)`

The target is not operator work.
The target is the next refinement of sigma's parent mode carrier so sigma no
longer relies on the bounded one-step `global_regressive_mode_index`.

## What `global_regressive_mode_index` Was Still Approximating

`global_regressive_mode_index` was a bounded one-step generator-action profile.

It carried:

- seed orbit of current/fiber/radial bounded observables
- one-step projected current observable under each lawful generator
- one-step successor local orbit under each lawful generator

What it still approximated was:

- lawful generator composition on the parent regressive mode

So it was nonlocal relative to a single local orbit snapshot, but still only a
first-order parent carrier.

## Refinement Implemented

The sigma slot now uses:

- `RegressiveModeCarrierV4`

This carrier contains:

- `seed_orbit`
- `generator_projection_profile`
- `generator_orbit_profile`
- `generator_composition_profile`

The new nonlocal ingredient is:

- `generator_composition_profile`

This records, for each lawful first generator, the projected current observable
seen after every lawful second generator.

So sigma now carries:

- first-order lawful mode presentation
- plus second-order lawful composition structure

That is the smallest honest refinement beyond the v3 one-step parent carrier.

## Why This Is Less Bounded Than `global_regressive_mode_index`

`global_regressive_mode_index` only described:

- what the mode looks like after one lawful generator action

`RegressiveModeCarrierV4` additionally describes:

- how those generator actions compose at the parent mode level

So the sigma carrier is no longer only a one-step observable-action summary.
It is a composition-aware bounded parent-mode carrier.

## Sigma Hierarchy

The parent-object hierarchy remains unchanged.

Parent state:

- `spin_H_core_v4 = (theta, rho, sigma***, h)`

Derived observables:

- `Pi_pred(spin_H_core_v4) -> spin_h4`
- `Pi_rec(spin_H_core_v4) -> tau`
- `Pi_hol(spin_H_core_v4) -> kappa`

`spin_h4`, `tau`, and `kappa` remain projections.
They are not promoted back into primary state.

## Update-Law Role

The refinement is explicit and mechanical.

- `I`
  - preserve the carrier under identity composition
- `T_b`
  - preserve the carrier's seed orbit and recompute lawful composition profile
    from the base-advanced successor
- `T_x`
  - recompute the carrier from the swap successor, including second-order
    composition rows
- `T_c`
  - recompute the carrier from the coupled successor, including second-order
    composition rows
- `T_y`
  - recompute the carrier from the twist successor, including second-order
    composition rows
- `T_z'`
  - recompute the carrier from the lift successor, including second-order
    composition rows
- `T_r*`
  - recompute the carrier from the radial successor, including second-order
    composition rows

The carrier therefore survives lawful generator composition more faithfully than
the v3 one-step parent profile.

## Bounded Audit

On the same bounded lawful surface:

- primary states examined: `45`
- distinct parent states reached: `38315`
- collision count at parent-state level: `0`
- recursive consistency rate: `1.0`
- sigma still depends on bounded local-observable orbit summaries: `no`
- new sigma carrier is less bounded than `global_regressive_mode_index`: `yes`

Comparison against `spin_H_core_v3`:

- collision change: `0`
- recursive-consistency change: `0`
- distinct-state-count change: `0`

So the refinement is real at the sigma-carrier level, but on the current bounded
lawful surface it does not create a new parent-state partition beyond v3.

## Honesty

Does sigma still rely on bounded `global_regressive_mode_index` as its parent
mode carrier?
- `no`

Was sigma’s parent mode carrier materially refined or replaced?
- `yes`

Is full exact `spin_H` now present?
- `no`

## Next step

Identify the next missing ingredient of the exact regressive sigma mode before
any behavior run.
