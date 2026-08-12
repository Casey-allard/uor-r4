# Training-Rule Alignment to Angular Manifold Geometry — v1

## Source Material

This analysis is based on direct reading of:

1. **The paper**: `/AI-Research/main.pdf` — "Angular Manifold Routing: Sublinear Compute
   Reduction via Hopf-Base Sector Discretization" (Allard, March 2026)
2. **The implementation**: `RouterV6Scaled` in
   `tools/prime_transport/run_router_execution_scaling_v2.py`
3. **The operator algebra**: `geometry_native_operator_model_v{20–25}.py`,
   `geometry_native_spinH_core_v6.py`, `geometry_native_spinH_candidate_v3.py`
4. **The Hopf routing code**: `hyperbolic_router_so8.py` (lines 826–880, 1754–1795)
5. **The theoretical hypothesis**: `phase_transport_hypothesis.md`
6. **Profiling evidence**: bottleneck probe v3, vectorization probe v1, regime boundary v1

---

## SECTION A — CURRENT FORWARD GEOMETRY

The forward architecture is NOT a generic neural network operating on flat vectors.
It has at least five layers of explicit geometric structure, which I enumerate from the
source code and paper.

### A.1 — The Operator State Space Is a Discrete Fiber Bundle

The underlying state is `OperatorStateV10` with fields:

| Field | Type | Geometric Role |
|-------|------|----------------|
| `b` | int (mod BASE_PERIOD) | **Base angle** on a torus |
| `phi` | int | **Fiber phase** |
| `r` | int (mod 3) | **Radial class** (shell/stratum) |
| `spin_h` | `NativeSpinHV1` | Binary predictive word on horizon |
| `tau` | `NativeTauV3` | 4-phase transport state (see below) |

This state space has a product structure: `(base torus) × (fiber) × (radial shells) × (spin word) × (tau phases)`.

Source: `geometry_native_operator_model_v10.py:36–46`

### A.2 — The Tau State Lives on a Discrete Torus T⁴

`NativeTauV3` consists of four cyclic phase variables:

| Phase | Modulus | Geometric Meaning |
|-------|---------|-------------------|
| `swap_phase` | Z/2 | Parity of swap operations |
| `coupled_phase` | Z/5 | Coupled interaction phase |
| `twist_phase` | Z/2 | Parity of twist operations |
| `lift_phase` | Z/12 | Fiber transport lift phase |

The total tau state space is **T⁴ = Z/2 × Z/5 × Z/2 × Z/12**, a discrete torus with
**2 × 5 × 2 × 12 = 240 points**.

In the neural network, this is represented as a **one-hot encoding** in R²¹
(2 + 5 + 2 + 12 = 21 dimensions).

Source: `geometry_native_spinH_candidate_v3.py:28–39`

### A.3 — The Six Operators Implement Geometric Transformations

The operator algebra consists of six operators partitioned into two classes:

| Operator | Name | Class | Geometric Action |
|----------|------|-------|-----------------|
| T_b | torus_base_advance | non-transport | Advances base angle b → b+1 mod period |
| T_x | composite_swap | non-transport | Swaps phase components (parity flip) |
| T_y | composite_twist | non-transport | Twists phase components (parity flip) |
| T_c | coupled_torus_kick | **transport** | Coupled holonomy with kick |
| T_z' | fiber_phase_lift | **transport** | Transport along fiber with phase lift |
| T_r* | radial_transport | **transport** | Transport across radial shells |

The three transport operators (T_c, T_z', T_r*) implement **parallel transport with
holonomy corrections** via `active_transport_lift_core_v6` and the `sigma_update_v6`
family holonomy law. The code explicitly names these corrections "holonomy":

```python
# From geometry_native_spinH_core_v6.py:76
def coupled_holonomy_residue_v6(source_sigma, proposed_sigma):
    ...
def sigma_family_holonomy_law_v6(source_sigma, component, proposed):
    ...
    role_shift = coupled_holonomy_residue_v6(source_sigma, proposed_sigma)
```

Source: `geometry_native_spinH_core_v6.py:76–154`,
`geometry_native_operator_model_v{20–25}.py`

### A.4 — The Hopf Fibration Map (Paper Section 2.1)

The paper defines a four-step Hopf angular routing mechanism operating on S³:

1. **Project to 4D Hopf subspace**: select (a, b, c, d) from embedding
2. **Hopf base coordinates**: compute ρ₁, ρ₂, χ_u (base), δ (fiber difference),
   α = ½(θ₁ + θ₂) (fiber mean phase)
3. **Levi-Civita transport**: α_t = α + ½λ cos(2χ) δ
4. **Sector assignment**: quantize (χ, δ, α_t) into K sectors

The implementation in `hyperbolic_router_so8.py:826–880` computes these exactly:

```python
connection_weight = 0.5 * float(phase_transport_lambda) * np.cos(2.0 * chi)
phase_shift = wrap_to_pi(connection_weight * delta)
transported_alpha = wrap_to_pi(alpha + phase_shift)
```

**Key finding from the paper (Section 2.2, Table 2)**: The raw fiber mean phase
α = ½(θ₁ + θ₂) provides +20.6 percentage points of Gini ratio improvement. The
Levi-Civita transport adds only ≈2 more. The mechanism is primarily **fiber phase
concentration**, not base coordinate routing.

Source: `main.pdf` Section 2.1–2.2, `hyperbolic_router_so8.py:826–880, 1754–1795`

### A.5 — The Phase Transport Hypothesis

The project's own `phase_transport_hypothesis.md` states:

> "If routing is organized on a hyperbolic manifold with Hopf-style angular structure,
> then phase coordinates are not optional — they are intrinsic to the geometry."

And:

> "Phase shifts move information within sectors of the routing manifold without
> changing shell membership."

This framing is consistent with the operator algebra: the operators literally
advance phases on a torus, lift along fibers, and transport across radial shells —
and the tau state tracks the accumulated phase of these operations.

### A.6 — The D-Step Recurrence as a Trajectory on the State Space

The forward pass of `RouterV6Scaled` executes a 24-step recurrence:

```
for t in range(D):        # D = 24
    tn_batch = TN[state_ids]              # lookup 6 operator tau-nexts
    w = gumbel_softmax(MLP(emb_t, tau_prev))  # route: which operator?
    tau_prev = einsum("bi,bij->bj", w, tn_batch)  # soft mixture of nexts
    state_ids = TR[state_ids].gather(argmax(w))    # hard state transition
```

Each step:
- **Queries** the current geometric state (via TN/TR tables)
- **Routes** between geometric operators (via soft Gumbel-softmax)
- **Updates** the tau phase state (via weighted mixture of operator outputs)
- **Transitions** the discrete state (via hard argmax + TR table lookup)

The trajectory {tau_0, tau_1, ..., tau_23} is then attention-pooled to produce a
prediction. This is a **path on the discrete torus T⁴**, viewed through the soft
Gumbel-softmax lens.

### A.7 — What is Explicitly Angular / Fibered / Manifold-Like

Summarizing the geometric inventory:

1. ✅ **Discrete torus state space** — tau lives on Z/2 × Z/5 × Z/2 × Z/12
2. ✅ **Fiber bundle structure** — base (b), fiber (phi), radial (r) coordinates
3. ✅ **Transport operators** — T_c, T_z', T_r* with explicit holonomy corrections
4. ✅ **Phase accumulation** — operators update cyclic phases additively mod periods
5. ✅ **Hopf fibration** — paper establishes (χ, δ, α_t) decomposition on S³
6. ✅ **Levi-Civita connection** — phase transport correction α_t = α + ½λcos(2χ)δ
7. ✅ **Sector discretization** — angular coordinates quantized into routing sectors

---

## SECTION B — CURRENT TRAINING RULE

### B.1 — What the Training Rule Actually Does

The training loop (from `run_router_execution_scaling_v2.py:364–378`):

```python
optimizer = torch.optim.SGD(scripted.parameters(), lr=0.02)

for step in range(BENCH_BUDGET):
    temp = TEMP_START * (TEMP_END / TEMP_START) ** (step / (BENCH_BUDGET - 1))
    pred = scripted(sids, toks, x0, temp)
    loss = F.cross_entropy(pred, x0)
    optimizer.zero_grad()
    loss.backward()
    nn.utils.clip_grad_norm_(scripted.parameters(), max_norm=1.0)
    optimizer.step()
```

### B.2 — What Each Component Does in Optimization Terms

**Loss function**: Cross-entropy between predicted x0 and true x0. This is a standard
classification loss. It measures log-probability of the correct label under a softmax
distribution over VOCAB=4 classes. No geometric structure.

**Gradient computation**: `loss.backward()` computes ∂L/∂θ for all parameters θ via
reverse-mode automatic differentiation (backpropagation). This traces backward through
the full 24-step recurrence, computing Jacobians of each operation in the **ambient
Euclidean space** of the tensors.

Specifically, autograd computes:
- ∂L/∂W_pred, ∂L/∂b_pred — from the prediction head (standard linear layer Jacobian)
- ∂L/∂(attention weights) — from the pooling layer (softmax Jacobian)
- ∂L/∂W1, ∂L/∂W2, etc. — accumulated over 24 time steps via chain rule through:
  - `torch.cat` (identity Jacobian, just routes gradients)
  - `@ W1 + b1` (linear Jacobian in R^(25×D_HIDDEN))
  - `torch.tanh` (element-wise tanh'(h) Jacobian)
  - `@ W2 + b2` (linear Jacobian in R^(D_HIDDEN×6))
  - `torch.softmax` (softmax Jacobian in R^6, the simplex tangent space)
  - `torch.einsum("bi,bij->bj", w, tn_batch)` (bilinear Jacobian)

**Optimizer**: SGD with flat Euclidean metric. Update rule:

    W ← W − 0.02 · ∂L/∂W

This is steepest descent in the **Euclidean metric** of the parameter space R^n.
The metric is the identity matrix. There is no curvature correction, no
preconditioner, no manifold awareness.

**Gradient clipping**: `clip_grad_norm_(params, 1.0)` rescales the global gradient
vector if its L2 norm exceeds 1.0. The norm is Euclidean.

**Temperature schedule**: Exponential decay from 2.0 to 0.1. This anneals the
Gumbel-softmax from soft (high temperature → flat weights) to hard (low temperature →
peaked weights). The schedule is time-based, not geometry-based.

### B.3 — What the Training Rule is NOT Doing

The training rule has **none** of the following:

1. **No manifold-aware metric** — all gradients computed in flat R^n
2. **No cyclic-group awareness** — the one-hot tau encoding is treated as a generic
   vector; neighboring phases on the torus are distant in gradient space
3. **No transport-aware credit assignment** — the 3 transport operators and 3
   non-transport operators receive gradients through the same Euclidean mechanism
4. **No connection/holonomy term** — despite the forward pass using explicit holonomy
   corrections, the backward pass has no corresponding geometric structure
5. **No angular coordinate system** — all optimization occurs in the ambient one-hot
   encoded space, not in the natural phase coordinates
6. **No fiber/base decomposition of the loss surface** — the gradient is a single
   monolithic vector, not decomposed into base, fiber, and radial components

---

## SECTION C — MISMATCH ANALYSIS

### C.1 — PROVEN: The Tau Coordinate System Is Geometrically Misaligned

**Fact**: The tau state lives on T⁴ = Z/2 × Z/5 × Z/2 × Z/12 (240 points on a
discrete torus). The one-hot encoding maps this into R²¹.

**Proven consequence**: In the one-hot encoding, **neighboring phases are maximally
distant**. For example:
- `lift_phase=11` and `lift_phase=0` are **adjacent** on Z/12 (they differ by one
  step), but their one-hot vectors are **orthogonal** in R¹² (Euclidean distance √2,
  the maximum possible distance between one-hot vectors).

This means the MLP gradient ∂L/∂W1 treats the transition from lift_phase=11 to
lift_phase=0 (wrapping around the cycle) as completely unrelated to the transition from
lift_phase=10 to lift_phase=11 (incrementing normally). But in the actual operator
algebra, both are single-step advances along the same cyclic group.

**Impact on learning**: The MLP must learn the cyclic adjacency structure of each
phase entirely from data. The one-hot encoding provides zero inductive bias about
the torus topology. Every pair of distinct phase values is equidistant in the encoding.

This is a **provable geometric misalignment** between the representation and the
underlying structure.

**Evidence from the code**:
```python
# geometry_native_spinH_candidate_v3.py:28-31
SWAP_PHASE_MODULUS_V3 = 2      # Z/2
COUPLED_PHASE_MODULUS_V3 = 5   # Z/5
TWIST_PHASE_MODULUS_V3 = 2     # Z/2
LIFT_PHASE_MODULUS_V3 = 12     # Z/12
```
vs.
```python
# run_router_reintegration_v6_torch.py:137-142
f = np.zeros(D_TAU, dtype=np.float32)    # 21-dim one-hot
f[off + tau.swap_phase] = 1.0;    off += 2
f[off + tau.coupled_phase] = 1.0; off += 5
f[off + tau.twist_phase] = 1.0;   off += 2
f[off + tau.lift_phase] = 1.0            # 12
```

### C.2 — PROVEN: The Soft Mixture Operates in the Wrong Space

**Fact**: The tau update at each step is:
```python
base = torch.einsum("bi,bij->bj", w, tn_batch)  # (B, D_TAU)
```

This computes a **convex combination** of one-hot vectors in R²¹. The result `base`
is a point in the interior of the convex hull of one-hot vertices — a vector with
fractional entries that **does not correspond to any point on the discrete torus T⁴**.

For example, if operators yield lift_phases {3, 7, 11} with weights {0.5, 0.3, 0.2},
the resulting soft-tau has fractional values at positions 3, 7, and 11 in the lift_phase
block. This barycentric coordinate has no natural interpretation on Z/12.

In angular coordinates, the same mixture would produce a **mean angle** on the unit
circle — a geometrically meaningful operation (the circular mean). In one-hot
coordinates, it produces a probability vector that is geometrically meaningless on
the torus.

**Impact on backward pass**: Gradients flowing through the soft mixture respect the
Euclidean metric of R²¹, not the circular metric of T⁴. The gradient ∂L/∂w (operator
weights) is computed as if the mixing were in flat space.

### C.3 — PLAUSIBLE: SGD in Ambient Space Ignores Loss Surface Curvature

**Observation**: The loss surface L(W1, W2, ...) inherits curvature from the
composition of:
1. The one-hot encoding (discrete → continuous)
2. The nonlinear MLP (tanh, softmax)
3. The TN table lookup (discrete, state-dependent)
4. The 24-step sequential composition

SGD's flat Euclidean metric assumes all directions in parameter space are equally
expensive to traverse. But the actual loss surface likely has strong anisotropy:
- Changes to W1 columns corresponding to the lift_phase block (12 dims) may have
  very different curvature than changes to the swap_phase block (2 dims)
- The coupled_phase block (5 dims) encodes a Z/5 structure whose periodicity is
  invisible to SGD

**Status**: Plausible but not yet measured. A comparison with a preconditioned optimizer
would test this directly.

### C.4 — PLAUSIBLE: The Backward Pass Reverses a Geometric Trajectory Using Non-Geometric Credit Assignment

**Observation**: The forward pass traces a path on the state space:
```
s₀ →[op_k₀]→ s₁ →[op_k₁]→ ... →[op_k₂₃]→ s₂₄
```
where each operator is a geometric transformation (torus advance, fiber lift, etc.).

The backward pass reverses this trajectory via the chain rule in R²¹ × R^(D_HIDDEN):
```
∂L/∂tau₂₃ → ∂L/∂tau₂₂ → ... → ∂L/∂tau₀
```

But this reversal uses the **Euclidean Jacobian** of each step, not the **geometric
connection** of the state space. In differential geometry terms: the forward pass
computes parallel transport along a discrete connection, but the backward pass computes
the adjoint of the linearized Euclidean embedding of that transport.

**If the state space has non-trivial curvature (as suggested by the holonomy corrections
in the operators), then the Euclidean adjoint and the geometric adjoint may disagree.**

**Status**: Plausible. The forward operators explicitly compute holonomy corrections
(`coupled_holonomy_residue_v6`). If these holonomy corrections are large, the Euclidean
backward pass is not computing the correct geometric adjoint of the transport.

### C.5 — SPECULATIVE: The Full Fiber Bundle Interpretation

**Speculation**: The operator algebra (base, fiber, radial) with holonomy corrections
implements something analogous to a connection on a fiber bundle. If the full
state space has fiber bundle structure with non-trivial curvature, then:
- The "correct" optimizer would be a **Riemannian optimizer** on the fiber bundle
- The "correct" gradient would be the **covariant derivative** of the loss, not the
  Euclidean gradient
- The **curvature tensor** of the bundle would appear as a preconditioner

**Status**: Speculative. The fiber bundle interpretation is suggestive but not proven
for the discrete operator algebra. The paper proves that the Hopf fibration
captures angular structure on S³ for routing, but does not prove that the
full operator algebra state space has the same smooth manifold structure.

### C.6 — Mismatch Summary Table

| Mismatch | Severity | Status |
|----------|----------|--------|
| One-hot encodes torus; destroys cyclic adjacency | High | **Proven** |
| Soft mixture operates in ambient R²¹, not on T⁴ | Medium | **Proven** |
| SGD uses flat metric ignoring torus curvature | Medium | **Plausible** |
| Backward Euclidean adjoint ≠ geometric adjoint | Medium | **Plausible** |
| No transport/non-transport distinction in gradients | Low-Medium | **Plausible** |
| Full fiber bundle Riemannian structure | Unknown | **Speculative** |

---

## SECTION D — NEXT EXPERIMENT

### D.1 — Recommended Experiment: Angular Phase Encoding

**Name**: Angular Phase Encoding vs One-Hot Convergence Comparison

**Rationale**: The largest proven mismatch (C.1) is that the one-hot encoding
of tau destroys the cyclic adjacency structure of the discrete torus T⁴. The
simplest direct test is to replace the one-hot encoding with an angular encoding
that respects the cyclic topology, and measure whether learning improves.

**The change**: For each cyclic phase variable with modulus m and current value k,
encode as:

    (cos(2πk/m), sin(2πk/m))

This maps each Z/m to the unit circle S¹, preserving adjacency: values k and k+1
(mod m) are nearby on S¹, including the wrap-around from m−1 to 0.

Concretely:

| Phase | Current Encoding | Angular Encoding |
|-------|-----------------|-----------------|
| swap_phase ∈ Z/2 | one-hot in R² | (cos(πk), sin(πk)) ∈ R² |
| coupled_phase ∈ Z/5 | one-hot in R⁵ | (cos(2πk/5), sin(2πk/5)) ∈ R² |
| twist_phase ∈ Z/2 | one-hot in R² | (cos(πk), sin(πk)) ∈ R² |
| lift_phase ∈ Z/12 | one-hot in R¹² | (cos(2πk/12), sin(2πk/12)) ∈ R² |

**Dimension change**: tau encoding goes from 21 dims → 8 dims.
D_IN goes from 25 → 12.

**What stays the same**:
- All operator semantics (TN/TR tables are unchanged — they still store the same
  geometric transitions)
- The task (predict x0 from trajectory)
- The recurrence structure (D=24 steps)
- The Gumbel-softmax routing mechanism
- The attention pooling with pos0 bias fix
- SGD with lr=0.02, same temperature schedule, same gradient clipping
- The device/batch/thread configuration

**What changes**:
- The TN table entries: each (N, N_OPS, D_TAU) entry is re-encoded from 21-dim
  one-hot to 8-dim angular. The table is rebuilt once at startup.
- The tau0_table: same re-encoding.
- W1 shape: from (25, D_HIDDEN) to (12, D_HIDDEN) — fewer parameters
- W_tok_inject shape: from (VOCAB, 21) to (VOCAB, 8)
- W_attn shape: from (D_HIDDEN_ATTN, 21) to (D_HIDDEN_ATTN, 8)
- W_pred shape: from (21, VOCAB) to (8, VOCAB)

**Measurement protocol**:
1. Run both models (one-hot baseline vs angular) on the same task, same seeds,
   same training budget (3000 steps or more)
2. Compare: convergence speed (loss curve), final accuracy, gradient norms over
   training, operator selection entropy
3. Run both models at D=24 (the solved delay) and optionally at harder delays
4. Record wall-time per step for both (angular model has smaller matrices)

**What a positive result means**: If the angular model converges faster or achieves
higher accuracy, it proves the one-hot encoding was fighting the torus geometry.
The training rule's credit assignment was misaligned because gradients were computed
in coordinates that destroy the cyclic adjacency of the phase state space.

**What a negative result means**: If the angular model performs the same or worse,
it means either (a) the MLP is powerful enough to learn the cyclic structure from
one-hot data, or (b) the one-hot encoding provides useful information (e.g., it's
a richer basis for the attention pooling). This would redirect the investigation
toward the other plausible mismatches (C.3, C.4).

**Why this experiment and not others**:
- It is the **smallest change** that directly tests the largest proven mismatch
- It does NOT change operator semantics, task, recurrence, or scientific conclusions
- It does NOT require implementing a custom optimizer or Riemannian gradient
- It does NOT require a full fiber bundle formalization
- It produces a clear, measurable comparison with minimal confounds
- A positive result opens a natural follow-up path (angular loss, angular
  preconditioner, Riemannian SGD) with a concrete geometric justification

### D.2 — If the Angular Encoding Experiment Succeeds: Follow-Up Path

If angular encoding measurably improves learning:

1. **Angular auxiliary loss**: Add a term that penalizes the soft tau mixture for
   deviating from valid angular states (i.e., regularize toward unit circle per
   phase pair).

2. **Phase-aware preconditioner**: Block-diagonal preconditioner on W1 with separate
   learning rates for the angular phase blocks vs the embedding block.

3. **Circular mean for soft mixture**: Replace the Euclidean weighted average with a
   **circular weighted mean** per phase pair, which is the geometrically correct
   averaging operation on S¹.

These are listed in order of increasing invasiveness. Each is bounded and testable.

---

## HONESTY SECTION

### What Is Proven

1. The tau state space is a discrete torus T⁴ = Z/2 × Z/5 × Z/2 × Z/12.
   This is directly from the code: the moduli are constants, the operators update
   phases additively mod their periods.

2. The one-hot encoding destroys cyclic adjacency. This is a mathematical fact:
   one-hot vectors for distinct values are all equidistant (distance √2), regardless
   of whether they are adjacent on the cycle.

3. The current training rule is flat Euclidean SGD with no manifold awareness.
   This is directly from the code: `torch.optim.SGD`, no preconditioner, no
   custom metric.

4. The soft tau mixture operates in ambient R²¹, not on the torus.
   This is directly from the einsum operation.

5. The forward operators implement explicit geometric transformations with holonomy.
   This is directly from `geometry_native_spinH_core_v6.py`.

6. The paper (main.pdf) establishes that the Hopf fibration map captures angular
   structure on S³, that the mechanism is primarily fiber-phase-based (α contributes
   +20.6pp, transport only +2pp more), and that the geometry is purely angular
   and norm-invariant.

### What Is Plausible But Not Yet Proven

1. That the one-hot → angular encoding change will measurably improve learning.
   The geometric argument is strong, but the MLP might already learn the cyclic
   structure implicitly.

2. That SGD's flat metric is a material bottleneck (as opposed to the coordinate
   encoding being the main issue).

3. That the Euclidean backward pass produces materially different credit assignment
   than a geometry-aware backward pass.

4. That the 3 transport operators would benefit from different gradient treatment
   than the 3 non-transport operators.

### What Is Speculative

1. That the full operator algebra state space has smooth manifold structure
   (it is discrete, with 343K states enumerated by BFS).

2. That a full Riemannian optimizer would provide large gains beyond what angular
   encoding provides.

3. That the holonomy corrections in the operators correspond to curvature that
   materially affects the loss landscape.

### What Previous Claims Were Too Confident

1. Prior work described the bottleneck as purely a runtime problem (serial loop,
   tiny ops, thread overhead). This was correct but incomplete: it focused on
   the computational cost of backpropagation without asking whether backpropagation
   in the current coordinate system was the right computation to be doing.

2. The vectorization analysis correctly proved the recurrence is mathematically
   sequential, but framed this as a dead end. In fact, the sequentiality of the
   *forward* pass is intrinsic to the geometry. The question was always whether
   the *backward* pass — which dominates 40% of runtime — needs to traverse the
   same sequential chain in the same coordinates, or whether a geometry-aware
   update rule could compute credit assignment more efficiently.

---

## APPENDIX: Formal Correspondence Between Paper and Implementation

| Paper Concept | Paper Location | Implementation |
|--------------|---------------|----------------|
| Hopf base χ_u | Eq. (1) | `hyperbolic_router_so8.py:846` |
| Fiber difference δ | Eq. (1) | `hyperbolic_router_so8.py:849` |
| Fiber mean α | Eq. (1) | `hyperbolic_router_so8.py:850` |
| Levi-Civita transport α_t | Eq. (2) | `hyperbolic_router_so8.py:874` |
| Sector assignment | Step 4 | `hyperbolic_router_so8.py:1782–1793` |
| Base angle b | "torus" in naming | `OperatorStateV10.b` |
| Fiber phase phi | "fiber_phase" in naming | `OperatorStateV10.phi` |
| Radial shell r | "radial_class" in naming | `OperatorStateV10.r` |
| Transport operators | Paper §4 (routing) | `_OPS[3:6]` — T_c, T_z', T_r* |
| Phase accumulation | Paper §2 (mechanism) | `NativeTauV3` phases mod periods |
| D-step recurrence | Not in paper | `RouterV6Scaled.forward` loop |
| Gumbel-softmax routing | Not in paper | `RouterV6Scaled.forward` |
| One-hot tau encoding | Not in paper | `build_state_tables` / `_tau_encode` |
| SGD training rule | Not in paper | Training loop |

The paper establishes the geometric substrate (Hopf fibration, angular non-uniformity,
transport). The implementation builds an operator algebra and recurrent router on
this substrate. The training rule is standard reverse-mode SGD — it is not discussed
in the paper, and this analysis identifies it as the primary site of geometric
misalignment.
