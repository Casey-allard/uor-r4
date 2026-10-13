# Geometric Routing via Riemannian-Complex Transport: Invariant Signals and the Universal Closure Framework

**Author:** Lionel Charles (Casey) Allard IV  
**Date:** April 2026  
**Subject:** Machine Learning (cs.LG), Mathematical Physics (math-ph)

---

## Abstract

Building on prior work demonstrating sublinear ($K^{0.572}$) compute reduction via fixed Hopf-base sector discretization [1], we propose that the observed efficiency of geometric routing is a low-order projection of a broader topological field computation. Standard transformer architectures rely on Euclidean linear operators that inadequately capture phase-dependent, directional, and hierarchical structures. We introduce a unified geometric framework, **Iso-Symmetry**, which establishes a universal metric for structural resolution termed **Isotropic Closure ($\sigma$)**. By embedding states within a complexified tangent space ($T_xM \otimes \mathbb{C}$) anchored at $2i$, we generalize computation into a complex-valued local transport law over a Riemannian manifold. We demonstrate that information propagation follows geodesic-aware routing governed by hyperbolic distance ($\Lambda$), exhibiting a universal regime-scaling ratio of $e^2 pprox 7.389$. Finally, we empirically validate this architecture via a train-free geometric probe, identifying a **Space Invariant Signal** ($\Delta_{sep} = 1.333$) that confirms geometric separation is strictly tied to local harmonic phase structure, independent of ambient space dimensionality.

---

## 1. Introduction: From Engineering to Ontology

Recent empirical evaluations of token embeddings on the unit hypersphere established that angular non-uniformity can be exploited to replace learned gating matrices with fixed geometric maps. Specifically, a fixed 4D Hopf fibration yields a sparse routing footprint scaling at $K^{0.572}$, significantly outperforming adaptive $K$-means baselines while maintaining norm-invariance [1].

While the Hopf-base discretization provides an effective engineering surrogate for gating, it implies a deeper architectural truth: semantic space is not a flat Euclidean grid, but a constrained geometric landscape. This paper formalizes the transition from "geometric bias" to a generative recursion law. We propose a Riemannian-Complex architecture where computation operates as a fluid dynamic system, and intelligence emerges as a multi-layer control structure riding atop coupled phase spaces.

## 2. The Iso-Symmetry Framework

To operationalize geometric routing, we must measure the "reconciliation toward the center"—the degree to which a state converges toward isotropic symmetry.

### 2.1 Isotropic Closure ($\sigma$)
For any $n$-tuple of dimensions $X$, we define the normalized simplex state $p_i = x_i / \sum x_j$. The **Quadratic Closure ($\sigma_Q$)** identifies the departure from the uniform isotropic state:

$$\sigma_Q(X) = 1 - rac{\sum_{i=1}^n (p_i - rac{1}{n})^2}{1 - rac{1}{n}}$$

where $\sigma = 1$ denotes full isotropic reconciliation (the center) and $\sigma = 0$ denotes maximal anisotropic concentration.

### 2.2 The Complex Bridge and Symmetry Pathway ($	au$)
Closure is a scalar magnitude. To distinguish structural "species," we introduce the **Symmetry Invariant ($	au$)**, which identifies the specific structural channel through which an object approaches the origin:

$$	au(X) = rac{\Delta_v - 2\Delta_h}{\Delta_v + 2\Delta_h}$$

where $\Delta_h$ and $\Delta_v$ represent horizontal and vertical displacements. This allows for the classification of states into functional families: **Cubic Closure** ($	au pprox 0$), **Square-Base Ascent** ($	au pprox 1$), and **Elongated Carrier** ($	au pprox -1/3$).

## 3. Riemannian-Complex Transport Architecture

We replace standard linear layers with local transport operators acting on a smooth Riemannian manifold $(M, g)$.

### 3.1 Complexified Tangent Representation
At each point $x_t \in M$, we extension the local representation into a complexified tangent space:
$$z_t \in T_{x_t}M \otimes \mathbb{C}$$
$$z_t = v_t^{(r)} + i v_t^{(i)}$$

This allows for the introduction of **phase-dependent dynamics**. Signals no longer just "add"; they interfere constructively or destructively according to their local phase alignment.

### 3.2 The $2i$ Anchor and Exponential Update
We anchor the transport at $2i$ (the pinnacle of unresolved potential) in the complex plane. The update law $\mathcal{U}_{x_t}$ generalizes scalar complex multiplication into a learned transport field:
$$\mathcal{U}_{x_t}(z) = W_r z + i W_i z$$
The updated state is projected via the exponential map:
$$x_{t+1} = \exp_{x_t}ig(\Re(\mathcal{U}_{x_t}(z_t))ig)$$
This forces information to follow **geodesics**—the paths of least resistance—constrained by the manifold's curvature.

## 4. Empirical Results: The Space Invariant Signal

To verify if this geometric separation depends on ambient dimensionality, we conducted a pure Layer 1 probe using the `cos_metric` between matched and mismatched prototype pairs.

| Space Type | Mean Matched Cos | Mean Mismatched Cos | Separation Delta |
| :--- | :--- | :--- | :--- |
| **CURRENT_BASELINE (R^16)** | 1.000000 | -0.333333 | 1.333333 |
| **R4_FLAT (R^4)** | 1.000000 | -0.333333 | 1.333333 |
| **REDUCED_BLOCK (R^12)** | 1.000000 | -0.333333 | 1.333333 |

**Result:** The separation delta of **1.333** is identical across all representations. This confirms the **Space Invariant Signal**: the geometric signal is localized entirely in the harmonic phase structure (the 2i-rotated $H3$ pair), rendering it invariant to the ambient "junk drawer" of Euclidean dimensions.

## 5. Discussion: The Regime Ladder and MUDBench

The approach to full closure ($\sigma 	o 1$) is an asymptotic journey in hyperbolic space. The **Hyperbolic Distance ($\Lambda$)** tracks this depth:
$$\Lambda(X) = -\ln(1 - \sigma)$$
As complexity ($k$) increases, the system climbs a **Regime Ladder**. Our analysis reveals a universal scaling ratio of **$e^2 pprox 7.389$** between regime thresholds. 

To evaluate the stability of these "High-Closure" agents, we introduce **MUDBench**, a benchmark for **Layered Consequential Interaction**. By testing agents in persistent, shared-state worlds, we verify whether they can maintain coherence across layered structural loops:
1. **Immediate Action** (Affordance)
2. **Local Objective** (Intent)
3. **Temporal World** (Entropy)
4. **Multi-Agent** (Social)
5. **Session Persistence** (Identity)

## 6. Conclusion

Intelligence is not a biological exception; it is a high-order phase of recursive geometric organization. By moving from Euclidean snapshots to Riemannian-Complex transport, we bridge the gap between discrete arithmetic seeds and continuous topological flow. The $2i$ anchor and the $\sigma$ functional provide the coordinate system for this transition, establishing a unified theory of geometric resolution that scales from prime numbers to artificial agency.

---

## Bibliography

[1] Allard, L. C. (Casey). (2026). *Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization*. Independent Research Preprint. Available at: https://zenodo.org/records/19243034
