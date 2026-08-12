# Prime Transport spin_H Candidate V1

## Purpose

This step defines the first structured candidate for `spin_H` that is fuller
than a future-word prefix alone.

The candidate transport chart is:

- `(b, spin_H_candidate)`

with:

- `spin_H_candidate = (phi, r, spin_h4)`

where:

- `phi` supplies angular/fiber structure
- `r` supplies radial / unfolding depth
- `spin_h4` supplies bounded predictive admissibility structure

This is not full exact `spin_H`, but it is not just a future-word prefix
either.

## Candidate Definition

### 1. What `spin_H_candidate` contains

`spin_H_candidate` contains exactly three components:

1. angular-fiber component:
   - `phi`
2. radial / unfolding component:
   - `r`
3. predictive component:
   - `spin_h4 = spin_H[:4]`

### 2. Angular identity contribution

Angular identity is represented by:

- base angle `b`, preserved explicitly in the transport chart
- fiber refinement `phi`, carried inside `spin_H_candidate`

### 3. Radial / unfolding contribution

Radial / unfolding structure is represented by:

- `r`, carried natively inside `spin_H_candidate`

### 4. Predictive / admissibility contribution

Predictive unfolding is represented by:

- bounded admissibility word `spin_h4`

### 5. How this differs from prefix-only lifts

Prior lifts used only:

- `(b, spin_h4)`
- or `(b, r, spin_h4)`

This candidate instead uses:

- `(b, (phi, r, spin_h4))`

So angular-fiber structure is now part of the transport identity itself,
rather than left outside or only inferred indirectly.

### 6. Why this is a more faithful spin candidate

It is more faithful because it combines:

- stable angle
- fiber refinement
- radial depth
- predictive word

instead of treating the admissibility prefix as the whole transport identity.

## Required Honesty Section

### Is this candidate fuller than a future-word prefix truncation?

yes

### Is full exact spin_H now present?

no

### Is this a more faithful structured candidate for spin_H than prior prefix-only lifts?

yes

Observed bounded-surface measurements:

- primary states examined:
  - `5390`
- distinct transport states reached:
  - `5390`
- collision count:
  - `0`
- collision fraction:
  - `0.0`
- ambiguous primary states:
  - `2345`
- ambiguous primary fraction:
  - `0.43506493506493504`
- skipped short-spin observations:
  - `0`

Direct comparisons:

- collision reduction vs `v1`:
  - `5339`
- collision reduction vs `v2`:
  - `5296`
- collision reduction vs `v3`:
  - `5160`
- ambiguity reduction vs `v1`:
  - `0`
- ambiguity reduction vs `v2`:
  - `0`
- ambiguity reduction vs `v3`:
  - `2187`

Representation checks:

- `phi` represented:
  - `yes`
- `r` represented under bounded discrimination:
  - `no`
- predictive unfolding represented:
  - `yes`
- angular identity represented explicitly:
  - `yes`

So this candidate is clearly fuller than prefix-only transport identity and is
far more faithful on the bounded surface, even though it is still not full
exact `spin_H`.

## Results

The candidate transport chart is:

- `(b, spin_H_candidate)`

with:

- `spin_H_candidate = (phi, r, spin_h4)`

This changes the lift geometry in the important way that the transport identity
now includes:

1. angular fiber structure:
   - `phi`
2. radial / unfolding structure:
   - `r`
3. predictive structure:
   - `spin_h4`

The result is mechanically significant:

- the representative lift becomes collision-free on the bounded surface
- ambiguity remains, but it is no worse than the older fixed-horizon lifts and
  much better than the adaptive-prefix lift

So the main failure mode of the earlier lifts was not merely short prefix
length. It was that prefix-only transport identity omitted the geometric fiber
structure that should have been part of the spin-side transport identity.

## Required Outputs

The CSV is:

- [prime_transport_spinH_candidate_v1.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_spinH_candidate_v1.csv)

It reports:

- number of primary states examined
- number of distinct transport states reached
- collision count and collision fraction
- ambiguous primary states and fraction
- direct comparisons against lifts `v1`, `v2`, and `v3`
- representation checks for `phi`, `r`, predictive unfolding, and angular identity

## Single Next Step

Replace the current transport lift with this structured `spin_H_candidate`
before more operator work, because collisions are materially reduced and the
transport identity is now much closer to the specified cross-space geometry.
