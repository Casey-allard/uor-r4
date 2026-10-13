# Prime Transport Regressive Law Recovery

## Purpose

This step determines whether the current sigma carrier fields

- `seed_orbit`
- `generator_projection_profile`
- `generator_orbit_profile`
- `generator_composition_profile`

are the canonical regressive law itself, or only a bounded encoding of that law.

## 1. Representation Level

The current sigma carrier is:

- not the canonical regressive law itself
- a bounded profile encoding of that law

Reason:

- every field is a finite sample of how sigma presents under named lawful generators
- none of the fields is the generator-equivariant parent update rule itself
- the carrier stores observations of sigma under action, not sigma as an update law

So the representation level is:

- bounded action-profile encoding
- not direct canonical law

## 2. Direct Parent Law

The direct parent law is recoverable.

The smaller parent object is:

- canonical regressive sigma mode `sigma`

with direct lawful update:

- for each lawful generator `G` in `{I, T_b, T_x, T_c, T_y, T_z', T_r*}`
- there exists a canonical regressive update map `R_G`
- such that
  - `sigma' = R_G(sigma)`

The current profile fields are then projections of this law:

- `seed_orbit`
  - bounded orbit sampled from predictive projection of `sigma`
- `generator_projection_profile`
  - bounded table of `Pi_pred(R_G(sigma))`
- `generator_orbit_profile`
  - bounded orbit closures generated from `Pi_pred(R_G(sigma))`
- `generator_composition_profile`
  - bounded table of `Pi_pred(R_H(R_G(sigma)))`

So the parent law is:

- generator-equivariant recursive transport of sigma

and the current fields are:

- finite sampled observables of that law

## 3. Recovered Direct Canonical Law

The direct canonical regressive law is:

- `sigma` is the transport-side regressive mode
- lawful evolution is defined by named maps `R_G`
- predictive observables are read only after the update

Mechanically:

1. state carries `sigma`
2. choose lawful generator `G`
3. update by `sigma' = R_G(sigma)`
4. derive bounded observables afterward
   - `spin_h4 = Pi_pred(sigma)`
   - local orbit samples from repeated projected views of `sigma`
   - profile tables from projected images of `R_G(sigma)` and `R_H(R_G(sigma))`

This is the direct parent law because:

- the law lives at the update level
- the current carrier fields are all readouts of that update level

## 4. Direct Law Versus Profile Encoding

The direct law is preferable because:

- it makes sigma the actual transported state
- it makes the profile tables derived observables instead of identity
- it matches the canonical hierarchy already fixed in `spin_H = (theta, rho, sigma, h)`
- it stops treating sampled action tables as the parent mode carrier

The profile encoding is weaker because:

- it stores finite action snapshots
- it grows by adding more sampled fields
- it obscures the fact that the real object is the regressive update law itself

So the current carrier is useful for bounded recovery, but it is still the wrong representation level once the direct law is recognized.

## 5. Match To Earlier Intended Form

The recovered direct law matches the earlier intended form.

It matches:

- generator-based recursive geometry
- coupled regressive transport
- the previously intended real operator structure

Specifically:

- the generators are named lawful actions
- sigma is the recursively transported parent state
- predictive words are projections of sigma after lawful action
- closure and holonomy remain in `h`, not inside sampled sigma profiles

So the recovered law is not a new theory.
It is the direct restatement of the already intended generator-based recursive transport law, with the bounded carrier recognized as only a finite encoding of it.

## Honesty

Is the current sigma carrier the canonical regressive law itself?

`no`

Is the current sigma carrier only a bounded encoding of a more direct law?

`yes`

What is the direct parent law, in one sentence, if recoverable?

`The direct parent law is generator-equivariant recursive transport of sigma, i.e. sigma' = R_G(sigma) for each lawful generator G, with seed orbit and all action/composition profiles obtained afterward as bounded projections of that updated mode.`

## Single Next Step

Implement the direct regressive update law `sigma' = R_G(sigma)` as the sigma carrier itself, and demote the current profile fields to optional derived diagnostics rather than parent state.
