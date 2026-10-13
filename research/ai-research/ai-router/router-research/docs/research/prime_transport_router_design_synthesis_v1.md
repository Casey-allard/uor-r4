# Prime Transport — Router Design Synthesis v1

**Document type:** Design-facing synthesis note  
**Basis:** Completed v25 operator algebra audit chain  
**Status:** Design hypothesis — not yet empirically validated in routing context  
**Date:** 2026-04-07

---

## 1. The Two Operator Families

### Non-transport cluster: {T_b, T_x, T_y}

These three operators share a common signature across the completed audit chain:

- **Short, arithmetically stable tau cycles.** T_x has a single period of 6 (entropy = 0,
  100% of orbits). T_b and T_y are concentrated on periods that divide or are
  close to LCM=60. The non-transport cluster stays largely within the {2,3,5}
  prime arithmetic envelope (combined extra-prime orbit fraction ≈ 3.4%).
- **Shallow tau return depth.** Non-transport mean avg tau return depth ≈ 13.8
  (vs 39.7 for transport). All non-transport orbits resolve fully within cap=200.
- **Preserved coupled_phase.** T_x and T_y do not update `coupled_phase` at all
  (step = 0 by construction). This keeps their tau dynamics confined to two
  sub-fields each, giving them the shortest and most predictable cycles.
- **Swap-geometry coupling.** T_x always disturbs swap_geometry; T_b and T_y are
  more mixed. The non-transport operators are the primary agents of local
  permutation and twist in the operator layer.

### Transport cluster: {T_c, T_z', T_r*}

These three operators are structurally distinct from the non-transport cluster
across every completed audit:

- **Deeper, more dispersed tau cycles.** Transport mean avg tau return depth ≈ 39.7,
  with T_r* averaging 67.8 at cap=1000. Period distributions are broad and
  multimodal; entropy is roughly 2× higher than non-transport.
- **Extra-prime orbit structure.** T_c: 7.9%, T_z': 37.6%, T_r*: 50.9% of tau
  orbits involve periods outside the {2,3,5} envelope. T_r*'s largest observed
  period prime is 173. The arithmetic structure of the transport cluster
  genuinely escapes the LCM-60 scaffold.
- **Near-uniform coupled_phase stepping.** T_c, T_z', and T_r* apply steps drawn
  nearly uniformly across all five coupled_phase residues. This is what makes
  coupled_phase the secondary tau bottleneck for these operators.
- **Sigma-stable carrier coherence.** Transport operators are ~2× more likely
  to preserve swap_geometry within their sigma-stable subset (100% swap_geo
  invariance within sigma-stable states). They carry a coherent geometric
  signature that non-transport operators do not.

---

## 2. Mechanistic Interpretation

### Non-transport: scheduling, local permutation, and recurrence

The non-transport cluster appears to implement **short-range, periodic, metronomic
operations** on the operator state.

- **T_x (composite swap):** A precise local permutation operator. Period=6
  universally, zero tau-period entropy. It is the most clockwork-like operator
  in the layer — every application advances the state by exactly one step in a
  six-step cycle. Its role is structural: enforce local alternation or provide
  a fixed-rate tick that other operators can lock onto.
- **T_y (composite twist):** A slightly richer recurrence operator. Its periods
  concentrate on small multiples of 6 (6, 12, 18). It handles twist-geometry
  transitions while staying predictably periodic. It is the "soft scheduler" —
  less rigid than T_x, more rhythmic than the transport operators.
- **T_b (torus base advance):** A medium-period recurrence operator grounded in
  the torus base geometry. Periods cluster around 60/30/20/15 — the divisors of
  LCM-60. It is the primary carrier of the arithmetic structure observed in the
  prior arithmetic-factor tests. Its role is systematic base traversal: advancing
  through torus positions in a way that is predictable and reversible.

Collectively, the non-transport cluster provides the **stable scaffolding** for
the operator layer: predictable local transitions, bounded return times, and
arithmetic regularity.

### Transport: holonomy, exploratory displacement, and deep coupling

The transport cluster implements **long-range, holonomy-rich, arithmetically
irregular operations** that move the system through genuinely new territory.

- **T_c (coupled torus kick):** A theta-directed transport operator with moderate
  tau depth (avg ≈ 24). It couples torus position to sigma state through the
  interaction step, making its tau dynamics dependent on the (qs, bs) interaction
  structure. Introduces extra primes 7, 11, 13, 17, 31 — more than the non-transport
  cluster but less than the deeper transport operators.
- **T_z' (fiber phase lift):** Also theta-directed but with significantly deeper
  and more dispersed tau behavior (extra-prime fraction 37.6%, largest prime 59).
  It operates on the fiber-phase axis, lifting the state in a direction not
  accessible to the non-transport operators. It is the primary mechanism for
  traversing the fiber dimension.
- **T_r* (radial transport):** The deepest and most arithmetically irregular
  operator in the layer. Rho-directed (unlike theta-directed T_c and T_z').
  Average tau return depth 67.8, 50.9% extra-prime orbit fraction, largest
  prime factor 173. T_r* is the primary mechanism for radial displacement — it
  moves the system outward through the operator surface in a way that no other
  operator does. Its deep, irregular tau behavior suggests it induces trajectories
  that touch remote, rarely-visited regions of the state space.

Collectively, the transport cluster provides **exploratory displacement** through
the state space: reaching geometric positions inaccessible to the non-transport
operators, at the cost of deeper and less arithmetically predictable tau dynamics.

---

## 3. Key Outliers

### T_x — the short-cycle swap anchor

T_x is structurally unique in the layer:

- Single tau period of exactly 6 — no other operator comes close (T_b's next
  nearest is period 5 or 10; T_y's nearest is also 6 but with wide dispersion).
- Perfectly preserved coupled_phase (step = 0 always) — it never changes the
  coupled-phase degree of freedom.
- Always disturbs swap_geometry — it is the canonical swap-geometry operator
  with no exceptions.

**Router implication:** T_x is the natural **rate control** or **attention-head
alternation** operator. Because it has a deterministic period and zero tau
dispersion, it is the most predictable tool for imposing a fixed-frequency
oscillation on any routing component that needs it. It is also the lowest-risk
operator to apply frequently, since its tau impact is bounded and reversible in
exactly 6 steps.

### T_r* — the deep-holonomy radial transport outlier

T_r* is structurally unique on the transport side:

- Rho-directed (radial), while T_c and T_z' are theta-directed. It operates on
  a geometrically orthogonal axis.
- Deepest tau dynamics in the layer — resolves fully only at cap=1000, with
  50.9% of orbits involving periods outside {2,3,5}.
- Largest prime factors observed (up to 173 in period 519=3×173) — its tau
  period structure is arithmetically the least constrained.

**Router implication:** T_r* is the natural **long-range routing jump** operator.
It is the mechanism for moving a token or attention context into a genuinely
remote state. Its cost is high (deep tau, slow return), but it provides access
to regions of state space that theta-directed operators cannot reach. It should
be used sparingly — activated only when the routing task requires genuine
geometric displacement, not just local permutation or torus traversal.

---

## 4. Algebra Findings Strong Enough to Inform Design Now

The following findings from the audit chain are established with high confidence
and are directly usable for routing design:

1. **The operator layer is non-degenerate.** All compositional pairs are
   distinct; no two operators collapse to the same image distribution. The
   layer is not redundant — each operator provides a genuinely different
   transformation.

2. **The two clusters are mechanistically separable.** Non-transport operators
   produce predictable, short-cycle, arithmetically constrained behavior.
   Transport operators produce deeper, more dispersed, arithmetically richer
   behavior. This is a structural property, not a sampling artifact.

3. **T_x is the unique deterministic-rate operator.** Period=6 with zero
   entropy is a strong constraint. No other operator shares this property.

4. **T_r* is the unique deep-radial operator.** Its rho-direction and deep tau
   behavior are structurally different from all other operators.

5. **The algebra is compositionally non-collapsed on the bounded v25 surface.**
   The commutativity spectrum audit and composition checks confirm that operator
   order matters — the algebra is genuinely non-abelian at the composition level.

6. **Transport operators have higher sigma-stable coherence.** Their sigma-stable
   subsets preserve swap_geometry 100%. This means transport operators applied
   to sigma-stable states behave more uniformly — a property that can be
   exploited for routing predictability within stable contexts.

---

## 5. Proposed Minimal Router Policy Hypothesis

### High-level framing

The operator layer divides naturally into two routing modes:

- **Stabilization mode** (non-transport): apply T_b, T_x, T_y to impose
  rhythmic, predictable, short-range transitions that keep the routing state
  well-anchored to the arithmetic scaffold.
- **Displacement mode** (transport): apply T_c, T_z', T_r* to execute long-range,
  holonomy-rich transitions that shift the routing context to geometrically
  remote positions.

### Policy hypothesis

> A router governed by the v25 operator layer should select non-transport
> operators when the routing task calls for **local refinement, recurrence, or
> pattern consolidation**, and should select transport operators when the task
> calls for **context switching, long-range dependency retrieval, or geometric
> displacement** to a new attentional region.

More concretely:

| Routing need | Candidate operator(s) | Rationale |
|---|---|---|
| Fixed-rate alternation / gating | **T_x** | Period=6, deterministic, zero tau dispersion |
| Torus traversal / base cycling | **T_b** | LCM-60 arithmetic regularity, reversible |
| Twist / recurrence | **T_y** | Short periods (6, 12, 18), mild dispersion |
| Theta-directed displacement | **T_c** | Moderate depth, coupled-phase enrichment |
| Fiber-axis lift | **T_z'** | Accesses fiber dimension not reachable by non-transport |
| Long-range radial jump | **T_r*** | Deepest holonomy, rho-directed, high-cost |

### Architectural role

In a transformer-replacement or routing-controlled attention scaffold, this
operator layer would sit **between the token embedding and the attention score
computation** — or equivalently, as a **routing controller that modulates which
attention heads are activated at each layer**.

The proposed role:

1. **Each routing step selects one operator** from the six (or a weighted
   combination) based on a routing policy learned from downstream task signal.
2. **Non-transport operators are the default mode** — they handle the common
   case of local context management and recurrence.
3. **Transport operators are gated by a displacement signal** — they activate
   when the routing context indicates that local operators are insufficient
   (e.g., high uncertainty, cross-domain retrieval, long-distance dependency).
4. **T_x serves as a synchronization primitive** — it can be used to periodically
   reset or re-anchor the routing state to a known phase, acting as a clock tick
   for multi-head coordination.

This is consistent with a **mixture-of-operators** architecture where the routing
decision is a discrete or soft selection over the operator set, conditioned on
the current attentional state.

---

## 6. Minimum Viable Reintegration Experiment

### What to plug into what

- **Input:** A bounded token sequence processed through the v25 operator surface
  (each token position maps to a state on the surface; the router selects which
  operator to apply at each step).
- **Router:** A lightweight learned selector (e.g., a 2-layer MLP or a learned
  gating vector) that takes the current token embedding plus the current operator
  state and outputs a probability distribution over the six operators.
- **Downstream task:** A simple sequence prediction or classification task where
  long-range dependencies are present (e.g., copying a pattern across a delay,
  or bracket matching). This ensures that the displacement vs. stabilization
  distinction is exercised.

### Baseline to compare against

1. **Fixed-operator baseline:** Apply a single operator uniformly (e.g., T_b
   only) at every step. This tests whether *operator selection* matters at all.
2. **Random-operator baseline:** Select uniformly at random over the six operators
   at each step. This tests whether the *structure* of the policy matters vs.
   mere operator diversity.
3. **Non-transport-only baseline:** Restrict selection to {T_b, T_x, T_y}. This
   tests whether the transport operators contribute meaningfully to routing
   performance, or whether the non-transport scaffold is sufficient.

### Success metrics

| Metric | What it tells you |
|---|---|
| Task accuracy vs. baseline | Whether operator routing improves downstream performance |
| Operator selection frequency per task type | Whether the router learns to use transport vs. non-transport in the expected pattern |
| Mean operator tau depth during correct vs. incorrect predictions | Whether deep-tau (transport) activations correlate with long-range dependencies |
| Fraction of T_r* / T_z' activations on far-context tokens | Whether the router self-organizes to use radial/fiber transport for distant tokens |

The experiment is considered **working** if:

1. Accuracy exceeds both the fixed and random baselines.
2. The learned routing policy shows statistically higher transport-operator
   frequency on tokens that require long-range context, relative to tokens that
   do not.

---

## 7. Honesty Section

### What is established

- The v25 operator layer is algebraically non-degenerate, compositionally
  distinct, and mechanistically differentiated between two clusters.
- The tau-orbit structure clearly separates non-transport (shallow, periodic,
  {2,3,5}-constrained) from transport (deep, dispersed, extra-prime-generating).
- T_x and T_r* are structurally unique outliers with well-characterized
  properties.
- The sigma-stable coherence of transport operators is a measured fact on the
  v25 surface.

### What is still unknown

- Whether the mechanistic separation between clusters **actually correlates
  with any useful routing signal** on real downstream tasks. The algebra says
  the operators are different; it does not say those differences are
  task-relevant.
- Whether the operator layer **scales beyond the bounded surface**. All audits
  used `depth=8`; behavior at deeper surfaces is untested.
- Whether the **discrete operator-selection architecture** is learnable with
  gradient descent, or whether a soft (continuous) relaxation is needed.
- Whether the extra-prime period structure of T_r* and T_z' **persists at
  scale** or is a bounded-surface artifact.

### What is not yet proven

- No end-to-end routing experiment has been run. The design hypothesis is
  derived entirely from algebraic characterization.
- The proposed mapping from operator properties to routing behavior
  (T_x = gating, T_r* = long-range jump) is a structural analogy, not an
  empirical result.
- The LCM-60 arithmetic structure has not been shown to have functional
  significance beyond the tau-orbit characterization.
- Full exact spin_H is not solved; the operator layer rests on the v6 core
  approximation.

---

## Recommended Next Step

Implement the minimum viable reintegration experiment: build a lightweight
operator-selection router on top of the v25 operator surface, test it on a
sequence task with known long-range dependency structure (e.g., bracket
matching or delayed copy), and compare against the three baselines specified
above.  The primary question to answer is whether the learned routing policy
discovers the transport / non-transport distinction without being told to.
