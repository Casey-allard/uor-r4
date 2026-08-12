# Deep Math Review: H4 Geometry and Routing Scaling

## Purpose
Review the project’s results so far and determine whether the current routing family is missing a more principled geometric law.

This is not a sweep result.
It is a mathematical synthesis of:
- repo results and decisions
- the current architecture
- a small set of external primary sources

## Executive Conclusion
The current routed family is probably not failing because it needs one more local shell controller.
It is more likely failing because it is still not implementing the right scaling law for a true 4D hyperbolic polar object.

The strongest mathematical diagnosis is:
1. the route needs shell-sector coupling from hyperbolic geometry, not mostly separate shell and sector rules
2. the 4D phase object should be treated as an `H^4` geodesic-polar object with an `S^3` angular manifold
3. the current route is approximating that `S^3` angular manifold with only two phase circles plus an ad hoc balance term
4. phase coupling is real, but it is probably a local correction on top of the wrong global scaling law

Recommended next architecture:
- `H4-Hopf geodesic routing with alpha-core`

Recommended queue change:
- do **not** make the sparse/quantized shell-phase branch the primary next step
- make it a secondary fallback
- first test the missing `H^4` shell-sector scaling law directly

## Repo-Grounded Empirical Facts
These are the most important facts the repo already established:

1. `phase4d` has a real signal.
- even-index pairing `0,2,4,6` keeps winning
- this strongly suggests the paired-complex view is not accidental

2. fixed sectoring was mathematically wrong for transfer.
- fixed `K` gave only partial widening and unhealthy concentration

3. shell activation was necessary.
- angular widening alone was insufficient
- radial divergence is real and matters

4. `delta_r` behaves like part of the routing law, not a display parameter.
- this means the route is sensitive to the actual radial geometry

5. `phi` helps most as a discrete controller constant.
- `phi_ladder` is better than the earlier continuous `phi_ratio` law in the current family

6. `phi_log` shells were a real improvement.
- this means the shell metric was previously mismatched

7. continuous shell-phase coupling is also real.
- `PHASE_K25_C035` improves on the coarse `PHI_PHI_PHI` family reference
- but it still does not beat `R0` under the strict runtime gate

Taken together, the evidence says:
- the project is seeing real geometry
- but the global scaling law is still off

## External Mathematical Anchors

### 1. Hyperbolic space is the right family for tree-like / power-law structure
Nickel and Kiela argue that many real datasets, including language-like and power-law data, carry latent hierarchical structure and that hyperbolic space is naturally suited for that because it behaves like a continuous tree.

Source:
- [Poincare Embeddings for Learning Hierarchical Representations](https://papers.nips.cc/paper/7213-poincare-embeddings-for-learning-hierarchical-representations.pdf)

Relevant reading:
- hyperbolic space as a continuous version of trees
- language and scale-free data connected to power-law / hierarchy

### 2. Hyperbolic geometry couples radius to angular capacity exponentially
In the hyperbolic plane, circumference and area grow as:
- `C(r) = 2*pi*sinh(r)`
- `A(r) = 4*pi*sinh^2(r/2)`

Source:
- [Hyperbolic circle formulae](https://www.maths.gla.ac.uk/wws/cabripages/hyperbolic/circleformulae.html)

This matters because it means angular capacity should not stay effectively fixed as radius grows.
That is the mathematical signature of negative curvature.

### 3. Tangent-space-only processing is not equivalent to manifold-native hyperbolic processing
Ganea, Becigneul, and Hofmann explicitly show that hyperbolic models need principled versions of distances, geodesics, Möbius matrix operations, and parallel transport.
They also report that applying Euclidean models after `log_0` projection is weaker than using true hyperbolic layers.

Source:
- [Hyperbolic Neural Networks](https://papers.neurips.cc/paper/7780-hyperbolic-neural-networks.pdf)

This directly matters for this repo because the current route still does most of its work in tangent/chart space.

### 4. Biological retino-cortical geometry is log-polar with a central correction, not pure log everywhere
Schwartz’s complex-log model and later biologically motivated retina work show:
- log-polar structure compresses peripheral detail efficiently
- pure log mapping has a central singularity
- introducing a small `alpha` creates a quasi-linear fovea and log-like periphery
- this improves continuity and data efficiency

Sources:
- [Schwartz 1980 PubMed abstract](https://pubmed.ncbi.nlm.nih.gov/6772241/)
- [A Space-Variant Visual Pathway Model for Data Efficient Deep Learning](https://www.frontiersin.org/journals/cellular-neuroscience/articles/10.3389/fncel.2019.00036/pdf)

The key lesson is not “copy the eye”.
The key lesson is:
- efficient geometric routing often needs a linear core plus non-linear periphery

## Diagnosis: What The Current Architecture Is Probably Getting Wrong

## D1. The route still treats shell and sector scaling as too separable
Current families improved by:
- better shell laws
- better sector laws
- better phase coupling

But true hyperbolic polar geometry couples them.

In negative curvature, moving outward changes:
- how many angular cells are needed
- how much boundary length exists
- how likely nearby points are to remain in the same local neighborhood

The current route mostly says:
- choose shell
- choose sector
- then patch the mismatch with controllers

That is probably backwards.
The shell-sector product should be derived from the geometry first.

## D2. The current 4D route is not yet a full 4D angular model
Inference from the current coordinate choice:

- `q1 = z_i + i z_j`
- `q2 = z_k + i z_l`
- current routing uses `theta1`, `theta2`, and a balance proxy from `rho1, rho2`

Mathematically, a 4D radius-plus-angle decomposition has:
- one radial coordinate
- an angular manifold `S^3`

The natural angular coordinates here are close to Hopf coordinates:
- `chi = atan2(rho2, rho1)` or equivalent latitude / balance angle
- `theta1`
- `theta2`

The current route uses:
- `theta1`
- `theta2`
- an ad hoc balance scalar

That is only a partial and not fully measure-aware treatment of `S^3`.

### Why this matters
The angular measure on `S^3` is not uniform in `theta1` and `theta2` alone.
It carries a latitude factor.

Inference:
- the project’s paired-phase route is probably seeing a real `S^3` effect
- but the current `phi^balance` rule is an ad hoc substitute for the true `S^3` angular measure

## D3. The route is missing hyperbolic shell-sector growth law
For geodesic polar coordinates in `H^n`, angular boundary size grows like `sinh^(n-1)(r)`.

So for `H^4`, the surface measure scales like:
- `sinh^3(r)`

Inference:
- total angular capacity per shell should grow roughly like `sinh^3(kappa * r)`
- not stay globally fixed with only mild local reallocation

This is the single strongest mathematical reason the current families may still be underperforming.

## D4. Continuous shell-phase bias is likely over-applied
`INC-0024` says phase coupling is real.
But the current law biases shells continuously for all points.

That is probably too expensive because it moves the whole address field.

If phase is a geometric correction, it should likely act as:
- a boundary correction
- a sparse gate
- or a discrete connection / step

not a globally active drift term.

So the current shell-phase branch is useful evidence, but likely not the final law.

## D5. The route is still too tangent-space-centric
The repo’s current route:
- log-maps into tangent space
- applies chart transforms
- routes mostly in Euclideanized coordinates

The hyperbolic neural network evidence suggests this is not enough if the architecture wants true hyperbolic behavior at scale.

This does not mean tangent charts are wrong.
It means:
- tangent charts are probably fine locally
- but the global scaling law should be manifold-native

## The Strongest New Hypothesis
The project should test:

## H4-Hopf Geodesic Routing With Alpha-Core

### Coordinates
Given paired complex coordinates:
- `q1 = z_i + i z_j`
- `q2 = z_k + i z_l`

Define:
- `rho1 = |q1|`
- `rho2 = |q2|`
- `theta1 = arg(q1)`
- `theta2 = arg(q2)`
- `chi = atan2(rho2, rho1)` in `[0, pi/2]`

Then treat the angular state as an `S^3` object:
- one latitude `chi`
- two phase circles `theta1`, `theta2`

### Radial law
Use a radial coordinate with a linear core and hyperbolic/log-like periphery.

Two practical options:
1. manifold-native:
   - `r_h = d_H(0, x)` if points are maintained in the Poincare ball / hyperboloid
2. transition proxy:
   - `r_alpha = asinh(||z|| / alpha)` or similar linear-core/log-periphery surrogate

Why:
- near origin: approximately linear
- far away: approximately logarithmic
- avoids singular behavior at the center

### Angular capacity law
Do not keep one mostly global `K`.

Instead, use shell-dependent angular capacity:
- `K_shell(r) ~ sinh^3(kappa * r)`

If full `H^4` shell growth is too expensive, cap it, but keep the dependence.

### Hopf-aware angular allocation
Instead of ad hoc `phi^balance`, allocate angular resolution using the `S^3` structure:
- `k1(r, chi) ~ sinh(kappa * r) * cos(chi)`
- `k2(r, chi) ~ sinh(kappa * r) * sin(chi)`
- optional low-cardinality `k_chi(r)` for the missing latitude axis

This is the key architectural change:
- the two phase circles should not be allocated only from heuristic balance
- they should be allocated from the actual geometry of `S^3`

## Why This Fits The Repo Results
It explains several otherwise awkward facts:

1. Why even-index pairings win
- because the route is seeing a real paired-complex / fiber structure

2. Why `K=25` became a minimum healthy scale
- because the current route is trying to approximate a higher-dimensional angular manifold with too few effective angular degrees of freedom

3. Why `delta_r` keeps acting like a law
- because radial quantization is currently compensating for missing shell-sector coupling

4. Why phase coupling helps but does not finish the job
- because phase is a correction on top of a still-misaligned global scaling law

## Ranked Candidate Directions

## Primary: H4-Hopf geodesic routing
Best mathematical fit to the project’s stated goal.

Why:
- consistent with hyperbolic tree-like growth
- consistent with the current two-phase-pair architecture
- explains the observed wins
- directly targets the likely missing scaling law

## Secondary: sparse / quantized phase-gated shells
Still worthwhile, but now looks like a local correction branch rather than the main route.

Why:
- `INC-0024` proved phase matters
- but this does not fix the deeper `H^4` shell-sector mismatch by itself

## Tertiary: horocyclic / Busemann routing
Worth keeping as a later branch if geodesic polar `H^4` still does not convert to a runtime win.

Why:
- LLM structure is not only hierarchical but directional / causal
- horocyclic coordinates may be better than isotropic shells for rooted or prefix-like hierarchies

This is not the first correction to try.

## Recommended Queue Change

## New highest-priority increment
- `INC-0026`: `H4-Hopf geodesic pilot`

### Minimal viable version
Do **not** start with full complexity.
First implement only:
1. `chi = atan2(rho2, rho1)` diagnostics
2. shell-dependent angular budget `K_shell(r)`
3. Hopf-aware `k1, k2` from `chi`
4. optional very small `chi` bin count, e.g. `k_chi in {1,2,3}`

### What to hold constant
- keep the current `phi_ladder` convergence law for now
- do not add more shell-phase tuning during the first pilot

Reason:
- isolate the missing geometric scaling law first

## Demote, do not delete
- `INC-0025` sparse / quantized shell-phase law

New status:
- secondary fallback if `H4-Hopf` does not help enough

## Concrete Experiment Design

## EXP-A: Geometry audit only
No new routing family yet.

Add diagnostics to the current route:
- `chi_mean`, `chi_entropy`
- predicted `k1_hopf`, `k2_hopf`
- compare current `phi^balance` allocation vs Hopf-weight allocation
- estimate shell-dependent target capacity from `sinh^3(r)`

Goal:
- measure whether the current route is systematically misallocating angular capacity

## EXP-B: H4-Hopf pilot
Implement a new route family:
- `sector_mode=phase4d_hopf`

with:
- `r_alpha`
- `chi`
- `theta1`, `theta2`
- shell-dependent `K_shell`
- Hopf-aware `k1, k2`

Goal:
- test the geometry directly before more controller work

## EXP-C: Tangent vs manifold-aware ablation
Compare:
- current tangent/chart route
- same route with more manifold-native radial law / transform discipline

Goal:
- determine whether tangent flattening is masking the hyperbolic advantage

## What This Review Changes
Before this review, the repo was correctly moving toward:
- sparse / quantized shell-phase coupling

After this review, that should no longer be the primary next step.

The new primary next step should be:
- test whether the route needs true `H^4` shell-sector growth and Hopf-aware angular allocation

## Sources Used
- [Poincare Embeddings for Learning Hierarchical Representations](https://papers.nips.cc/paper/7213-poincare-embeddings-for-learning-hierarchical-representations.pdf)
- [Hyperbolic Neural Networks](https://papers.neurips.cc/paper/7780-hyperbolic-neural-networks.pdf)
- [Hyperbolic circle formulae](https://www.maths.gla.ac.uk/wws/cabripages/hyperbolic/circleformulae.html)
- [Notes on Hyperbolic Geometry](https://people.reed.edu/~ormsbyk/341/hyperbolic_notes.pdf)
- [Schwartz 1980 PubMed abstract](https://pubmed.ncbi.nlm.nih.gov/6772241/)
- [A Space-Variant Visual Pathway Model for Data Efficient Deep Learning](https://www.frontiersin.org/journals/cellular-neuroscience/articles/10.3389/fncel.2019.00036/pdf)
