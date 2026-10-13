# Event-Driven Geometric Routing Model
## Formal speculative math note for Codex ingestion

This note formalizes the speculative architecture discussed in conversation:

- representation lives on a **curved routing manifold**
- angular structure is modeled using a **Hopf / complex / quaternionic decomposition**
- cheap internal motion is modeled as **fiber phase transport**
- sparse computation is triggered by **thresholded event firing**
- learned computation replaces dense matrix mixing with **geometric transport + gated accumulation**

This is not a proved model of brains, and not yet an established ML architecture. It is a mathematically coherent speculative framework for future experimentation.

---

# 1. Working geometric assumption

Assume routed states live on a manifold with hyperbolic radial structure and Hopf-style angular structure.

A convenient working state space is:

\[
\mathcal{M} \approx H^4
\]

with coordinates decomposed as:

\[
x = (r,\eta,\delta,\alpha)
\]

where:

- \(r\) = hyperbolic radial depth
- \(\eta\) = amplitude split
- \(\delta\) = relative phase
- \(\alpha\) = common fiber phase

Equivalently, the angular piece can be written as a point on \(S^3 \subset \mathbb{C}^2\):

\[
z_1 = e^{i(\alpha + \delta/2)} \cos \eta,
\qquad
z_2 = e^{i(\alpha - \delta/2)} \sin \eta
\]

so that

\[
|z_1|^2 + |z_2|^2 = 1.
\]

This gives the factorization

\[
(z_1,z_2)
=
e^{i\alpha}
\left(
e^{i\delta/2}\cos\eta,\;
e^{-i\delta/2}\sin\eta
\right)
\]

which cleanly separates:

- common phase \(\alpha\)
- relative phase \(\delta\)
- amplitude split \(\eta\)

---

# 2. Hopf decomposition and routing interpretation

The standard Hopf map \(S^3 \to S^2\) is

\[
h(z_1,z_2)
=
\left(
2\Re(z_1\overline{z_2}),
\;
2\Im(z_1\overline{z_2}),
\;
|z_1|^2 - |z_2|^2
\right).
\]

Substituting the above coordinates gives

\[
h(z_1,z_2)
=
\left(
\sin(2\eta)\cos\delta,\;
\sin(2\eta)\sin\delta,\;
\cos(2\eta)
\right).
\]

Important consequence:

- \(\alpha\) disappears under the Hopf map
- \((\eta,\delta)\) determine the base point on \(S^2\)

Interpretation:

- \((r,\eta,\delta)\) determine **coarse routing location**
- \(\alpha\) determines **cheap internal motion within a routed region**

That is the central architectural split.

---

# 3. Metric structure

On \(S^3\), the standard metric in \((\eta,\phi_1,\phi_2)\) coordinates is

\[
ds^2
=
d\eta^2
+
\cos^2\eta\, d\phi_1^2
+
\sin^2\eta\, d\phi_2^2.
\]

Using

\[
\phi_1 = \alpha + \frac{\delta}{2},
\qquad
\phi_2 = \alpha - \frac{\delta}{2},
\]

this becomes

\[
ds^2
=
d\eta^2
+
d\alpha^2
+
\frac{1}{4} d\delta^2
+
\cos(2\eta)\, d\alpha\, d\delta.
\]

Interpretation:

- \(d\alpha\) = fiber transport direction
- \(d\delta\) = relative-phase / base direction
- \(d\eta\) = amplitude / latitude direction

At

\[
\eta = \frac{\pi}{4}
\]

the coupling term vanishes:

\[
\cos(2\eta)=0
\]

so the metric locally decouples into orthogonal directions. This suggests that balanced amplitude states may provide a favorable operating regime for cheap phase transport.

---

# 4. Event-driven cell model on the manifold

We replace dense attention with a sparse set of geometric cells / buckets / event nodes:

\[
\{B_k\}_{k=1}^{N_c}
\]

Each cell \(k\) has:

- geometric center \(c_k \in \mathcal{M}\)
- membrane state \(u_k(t) \in \mathbb{R}\)
- threshold \(\theta_k\)
- refractory state \(\rho_k(t)\)
- local transport parameters \(T_k\)

This is the push / gated-pop intuition in formal form:

- accumulate evidence into \(u_k\)
- fire when \(u_k \ge \theta_k\)

---

# 5. Event representation

An incoming event packet is

\[
e_j^t = (x_j^t, a_j^t)
\]

where

- \(x_j^t \in \mathcal{M}\) is the state location
- \(a_j^t \in \mathbb{R}_+\) is event amplitude

A practical extended packet may include content payload \(v_j^t\), but the core geometric model can be written without it.

---

# 6. Geometric affinity kernel

Signal compatibility from event \(j\) into cell \(k\) is defined by a geometric kernel.

Let:

- \(d_H\) = hyperbolic distance in radial/base geometry
- \(b(x)\in S^2\) = Hopf base coordinates derived from \((\eta,\delta)\)
- \(\Delta \alpha_{jk}\) = fiber phase mismatch after transport

Define

\[
w_{jk}
=
K_r\!\left(d_H(x_j, c_k)\right)
\cdot
K_b\!\left(d_{S^2}(b(x_j), b(c_k))\right)
\cdot
K_\phi\!\left(\Delta \alpha_{jk}\right).
\]

Useful choices:

\[
K_r(d) = \exp\left(-\frac{d^2}{\sigma_r^2}\right)
\]

\[
K_b(d) = \exp\left(-\frac{d^2}{\sigma_b^2}\right)
\]

\[
K_\phi(\Delta\alpha)
=
\exp\left(\kappa \cos(\Delta\alpha)\right)
\]

or a thresholded version

\[
K_\phi(\Delta\alpha)
=
\max(0,\cos(\Delta\alpha)).
\]

Interpretation:

- nearby in hyperbolic depth
- nearby in Hopf base orientation
- phase aligned after transport

=> strong event influence.

---

# 7. Field-corrected phase transport

If there is a coupled field or connection over the manifold, phase should be compared after transport.

Let \(A\) be a connection one-form. Along a path \(\gamma_{j\to k}\),

\[
\tilde{\alpha}_j
=
\alpha_j
+
\int_{\gamma_{j\to k}} A.
\]

Then define transported phase mismatch

\[
\Delta\alpha_{jk}
=
\tilde{\alpha}_j - \alpha_k.
\]

This makes phase comparison local-frame consistent and gives a mathematically natural notion of phase alignment under routing.

---

# 8. Membrane accumulation dynamics

## Continuous-time form

\[
\frac{d u_k}{dt}
=
-\lambda_k u_k
+
\sum_j w_{jk}\, a_j\, \delta(t - t_j)
-
\beta_k \rho_k(t).
\]

## Discrete-time form

\[
u_k^{t+1}
=
(1-\lambda_k)u_k^t
+
\sum_j w_{jk}^t a_j^t
-
\beta_k \rho_k^t.
\]

where:

- \(\lambda_k\) = leak rate
- \(\beta_k\) = refractory penalty
- \(\rho_k\) suppresses repeated immediate firing

Interpretation:

- integrate incoming compatible events
- decay over time
- inhibit after recent firing

This is biologically inspired sparse event accumulation on a geometric state space.

---

# 9. Firing rule

A cell fires when its membrane state crosses threshold:

\[
s_k^t = \mathbf{1}\{u_k^t \ge \theta_k\}.
\]

Possible reset rules:

## hard reset
\[
u_k^{t+1} \leftarrow u_{\mathrm{reset},k}
\]

## subtractive reset
\[
u_k^{t+1} \leftarrow u_k^{t+1} - \theta_k.
\]

If \(s_k^t=1\), the cell emits a new event.

---

# 10. Emission / transport operator

When a cell fires, it emits a transformed state.

## Coordinate form

\[
(r,\eta,\delta,\alpha)
\mapsto
(r+\Delta r_k,\;
 \eta+\Delta \eta_k,\;
 \delta+\Delta \delta_k,\;
 \alpha+\Delta \alpha_k)
\]

where the \(\Delta\)-parameters are learned or predicted.

## Quaternion / group-action form

Let

\[
q = z_1 + j z_2,
\qquad |q|=1,
\]

so angular state lies in unit quaternions / \(SU(2)\).

Then emission can be written as

\[
q_k^{\mathrm{emit}} = g_k q_k
\]

with \(g_k \in SU(2)\), or in a restricted subgroup such as \(U(1)\) if only fiber-phase motion is allowed.

Interpretation:

- weights become structured group actions
- transport replaces dense linear mixing

---

# 11. Sparse expert view

This model can be interpreted as sparse expert routing.

Instead of dense attention

\[
\mathrm{softmax}\left(\frac{QK^\top}{\sqrt{d}}\right)V
\]

we use:

- geometric compatibility
- phase alignment
- thresholded event firing

Only cells with sufficiently accumulated evidence fire. This creates a sparse computational graph.

---

# 12. Training objective

A complete training objective should combine:

1. task performance
2. sparse event regularization
3. geometric consistency
4. phase coherence
5. occupancy balancing
6. transport smoothness

We propose

\[
\mathcal{L}
=
\mathcal{L}_{\mathrm{task}}
+
\lambda_{\mathrm{sparse}} \mathcal{L}_{\mathrm{sparse}}
+
\lambda_{\mathrm{geom}} \mathcal{L}_{\mathrm{geom}}
+
\lambda_{\mathrm{phase}} \mathcal{L}_{\mathrm{phase}}
+
\lambda_{\mathrm{bal}} \mathcal{L}_{\mathrm{bal}}
+
\lambda_{\mathrm{trans}} \mathcal{L}_{\mathrm{trans}}.
\]

Each term is described below.

---

## 12.1 Task loss

Task-specific objective, e.g. cross-entropy, next-token prediction, regression, etc.

\[
\mathcal{L}_{\mathrm{task}}
\]

---

## 12.2 Sparsity loss

Encourage few cell firings per step:

\[
\mathcal{L}_{\mathrm{sparse}}
=
\sum_t \sum_k s_k^t
\]

or averaged:

\[
\mathcal{L}_{\mathrm{sparse}}
=
\frac{1}{T}\sum_t \frac{1}{N_c}\sum_k s_k^t.
\]

This pushes the system toward sparse event-driven computation.

---

## 12.3 Geometric consistency loss

If learned states are intended to remain on or near the manifold, penalize deviations from constraints.

For \(S^3\) angular encoding:

\[
\mathcal{L}_{\mathrm{geom}}
=
\sum_j \left(|z_{1,j}|^2 + |z_{2,j}|^2 - 1\right)^2.
\]

If hyperbolic coordinates are represented via a chart or embedding, add regularization preserving chart validity or curvature structure.

---

## 12.4 Phase coherence loss

Encourage phase-aligned co-firing where appropriate.

A simple form:

\[
\mathcal{L}_{\mathrm{phase}}
=
-
\sum_{t,j,k}
s_j^t s_k^t \cos(\Delta\alpha_{jk}^t).
\]

This rewards coherent firing with aligned transported phase.

A more selective version can weight by geometric proximity.

---

## 12.5 Occupancy balancing loss

Prevent routing collapse into a few cells.

Let \(n_k\) be the number of firings or assignments into cell \(k\) over a window. Define probabilities

\[
p_k = \frac{n_k}{\sum_\ell n_\ell}.
\]

Then encourage high occupancy entropy:

\[
\mathcal{L}_{\mathrm{bal}}
=
\sum_k p_k \log p_k.
\]

Minimizing this discourages concentration and promotes balanced use.

---

## 12.6 Transport smoothness loss

Penalize transport operators that vary too violently across nearby cells or time steps.

For coordinate transport parameters:

\[
\mathcal{L}_{\mathrm{trans}}
=
\sum_{(k,\ell)\in \mathcal{N}}
\left(
(\Delta r_k - \Delta r_\ell)^2
+
(\Delta\eta_k - \Delta\eta_\ell)^2
+
(\Delta\delta_k - \Delta\delta_\ell)^2
+
(\Delta\alpha_k - \Delta\alpha_\ell)^2
\right)
\]

where \(\mathcal{N}\) is a neighborhood graph over cells.

In group-action form:

\[
\mathcal{L}_{\mathrm{trans}}
=
\sum_{(k,\ell)\in \mathcal{N}} d_G(g_k,g_\ell)^2
\]

with \(d_G\) a group distance on \(SU(2)\) or the chosen transport subgroup.

---

# 13. Plasticity-style update analog

A biologically inspired compatibility update can be written in Hebbian form.

For cell-event compatibility or kernel adaptation:

\[
\Delta w_{jk}
\propto
s_j^t s_k^{t+\tau}\, K_\phi(\Delta\alpha_{jk}).
\]

Interpretation:

- co-firing strengthens compatibility
- phase coherence modulates the update
- temporal ordering can be encoded through \(\tau\)

This is a geometric / phase-aware analog of Hebbian or STDP-like adaptation.

---

# 14. Differentiability issue and practical relaxation

Hard threshold dynamics are not friendly to gradient descent.

A smooth approximation is:

\[
s_k^t \approx \sigma\left(\frac{u_k^t - \theta_k}{\tau_s}\right)
\]

where \(\sigma\) is logistic sigmoid and \(\tau_s\) is a temperature.

As \(\tau_s \to 0\), this approaches a hard threshold.

This gives a differentiable surrogate for training, after which harder gating may be reintroduced.

---

# 15. Summary of the model

A full speculative event-driven geometric computation cycle:

1. represent each input/token/state as \(x_j^t = (r_j,\eta_j,\delta_j,\alpha_j,a_j)\)
2. compute geometric / phase compatibility with cells
3. accumulate evidence into membrane states \(u_k\)
4. apply threshold gate to determine which cells fire
5. emit transported states via local geometric transforms or group actions
6. train with task + sparsity + geometry + phase + balancing + smoothness losses

This is a coherent alternative to dense attention / dense MLP computation.

---

# 16. Why the math makes sense

The framework is internally coherent because:

- hyperbolic geometry encodes hierarchical / radial routing
- Hopf decomposition separates coarse angular identity from cheap fiber motion
- thresholded accumulation creates sparse event dynamics
- phase kernels define a natural compatibility law
- group actions provide structured local transport
- explicit losses can enforce the desired operating regime

In short:

- geometry determines **where**
- phase determines **cheap motion within where**
- thresholds determine **when computation actually happens**

---

# 17. Key open risks

Mathematical coherence does not guarantee empirical success.

Important failure modes:

1. learned embeddings may not stabilize on the intended manifold
2. thresholded routing may be hard to train
3. phase transport may not preserve useful semantic structure
4. cells may collapse or over-fragment
5. practical implementations may lose the theoretical compute advantage

Still, the framework is sufficiently well-posed to justify prototype experiments.

---

# 18. Recommended prototype sequence

Codex should prioritize the following order:

1. **static geometry prototype**
   - fixed manifold coordinates
   - no learning of transport
   - test sparse event routing only

2. **soft-threshold prototype**
   - differentiable firing
   - basic task loss + sparsity + balancing

3. **phase kernel ablation**
   - compare no phase / raw phase / transported phase

4. **transport operator ablation**
   - coordinate-shift transport vs quaternion/group transport

5. **full coupled-field prototype**
   - include learned connection / phase transport field

This staged path reduces the risk of building an elegant but impossible training setup all at once.

---

# 19. Final compressed formulation

A concise version of the speculative model is:

## state
\[
x_j^t \in \mathcal{M}
\]

## compatibility
\[
w_{jk}^t = K_{\mathrm{geom}}(x_j^t, c_k)\, K_{\mathrm{phase}}(\Delta\alpha_{jk}^t)
\]

## membrane update
\[
u_k^{t+1} = (1-\lambda_k)u_k^t + \sum_j w_{jk}^t a_j^t - \beta_k \rho_k^t
\]

## firing
\[
s_k^{t+1} = \mathbf{1}\{u_k^{t+1}\ge\theta_k\}
\]

## emission
\[
x_k^{t+1,\mathrm{emit}} = T_k(x_k^t)
\]

## total loss
\[
\mathcal{L}
=
\mathcal{L}_{\mathrm{task}}
+
\lambda_{\mathrm{sparse}} \mathcal{L}_{\mathrm{sparse}}
+
\lambda_{\mathrm{geom}} \mathcal{L}_{\mathrm{geom}}
+
\lambda_{\mathrm{phase}} \mathcal{L}_{\mathrm{phase}}
+
\lambda_{\mathrm{bal}} \mathcal{L}_{\mathrm{bal}}
+
\lambda_{\mathrm{trans}} \mathcal{L}_{\mathrm{trans}}.
\]

That is the formal event-driven geometric-routing architecture implied by the working assumptions so far.
