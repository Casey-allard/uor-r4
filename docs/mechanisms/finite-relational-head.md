# Finite relational head over native geometric descriptors

**Milestones:** M1 #2029 and M4 #2032. **Authority:** Kimi PM assignment ([#2028 comment 6174783178](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6174783178)) and coordination note ([#2029 comment 6175149631](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)), synthesized from Codex's merged native reconstruction/attention review ([#2480](https://github.com/UOR-Foundation/uor-r4/pull/2480), [`docs/labs/spiralcore-native-attention-review-2026-10-14/README.md`](../labs/spiralcore-native-attention-review-2026-10-14/README.md) and [`reviews/integrator.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/integrator.md)) and first assessment ([#2478](https://github.com/UOR-Foundation/uor-r4/pull/2478), [`docs/labs/spiralcore-attention-assessment-2026-10-14/README.md`](../labs/spiralcore-attention-assessment-2026-10-14/README.md)).

> [!IMPORTANT]
> **Scope and Governance Boundary (Source-Only Design Card — Zero Adoption, Zero Compute):**
> 1. **Design deliverable only:** This document specifies a falsifiable finite design contract. **No head adoption, no replacement of `ReadSpec::Route` or `ReadSpec::Flock`, and no model source mutation are authorized here.**
> 2. **`C6` vs `G64` explicitly UNSELECTED:** The choice between a native SpiralCore geometric state (`G64` / `R16`) and a separately typed six-phase controller (`C6` = $\mathbb{Z}/6\mathbb{Z}$) remains an open PM/Owner-level decision. Both realizations are specified symmetrically below; neither is pre-empted.
> 3. **$\le 16K$ Flock shortlist explicitly EXCLUDED as the head's pool:** The `16K` per-frame shortlist proof applies only to `Flock`'s independent comparator (`discrete.cu:165-182`) and is **not valid** as the candidate pool for a relational head.
> 4. **Independent adversarial review gate:** Claude Fable 5.1's owner-requested independent adversarial review of #2480 ([#2029 comment 6175060216](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175060216) / [`6175149631`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)) lands as an input to this card. If that review returns `REVISE` or `INSUFFICIENT EVIDENCE`, this design card will be amended before any qualification sheet is drafted.
> 5. **Pre-registration, `TEST FITNESS: FIT`, and owner release required before any compute:** Both finite CUDA construction qualification and any downstream trained-in evaluation remain **`NOT_RUN` / `HELD`** and require their own separate pre-registration sheets, `TEST FITNESS: FIT` reviews, and explicit owner releases under D22/D24/D26/D32.

---

## 1. Idea from first principles, and what it adds beyond `Route` and `Flock`

In the current discrete geometric model (`crates/uor-r4-discrete/src/discrete.cu`), context reads already combine multiple candidates after selection, but **candidate selection itself is non-relational**:
- **`ReadSpec::Route` (`discrete.cu:41-77`)** transforms the query block by a learned categorical frame `routes[block * stages + stage]` (`0..64`) and selects one branch/frontier port by minimum `(distance, hamming, port)` to fixed orbit roots (`g[BRANCH + root*3 + port]`), independent of earlier leaf contents.
- **`ReadSpec::Flock` (`discrete.cu:118-201`)** already performs multi-candidate interaction during *payload aggregation* — transporting 3 children into parent summaries (`discrete.cu:148-154`) and combining up to $K$ selected payloads per level via `snap(sum, found, g)` and ternary gates (`discrete.cu:185-196`). However, during *selection* (`discrete.cu:166-182`), `Flock` ranks each eligible node $i$ **independently** by its own query-to-candidate key:
  $$\bigl(d_{\text{L1}}(f_q, f_i),\, d_{\text{Hamming}}(f_q, f_i),\, -\text{last}[i],\, \text{node\_id}[i]\bigr)$$

**What the finite relational head adds:** **Query-conditioned candidate–candidate interaction during selection.** Two causal candidates $i, j$ with identical independent query-distance keys $(d_{\text{L1}}(f_q, f_i), d_{\text{Hamming}}(f_q, f_i))$ can be differentiated — or an older candidate $i$ can be promoted into the selected top $K$ — because another candidate $j$ in the prefix supplies support, conflict/contradiction, structural role binding, or version supersession along an explicit sparse relation edge $(i \leftrightarrow j)$, while their immutable evidence frames $f_i$, payloads $p_i$, downstream SpiralCore BFS transport, and the shared integer vector writer (`discrete.cu:229-253`) remain unchanged.

---

## 2. Five distinct typed objects and architectural boundary

Per [`reviews/integrator.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/integrator.md) (§ "Five objects, one compatible system") and [`reviews/cs.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/cs.md), the read interface keeps five object layers strictly distinct so that compact descriptions, deduplicated storage, and mutable relational states never conflate identity or corrupt evidence:

```text
1. ExactContent (ObjectRef)      : immutable bytes / child IDs + codec/schema + digest scope
2. Occurrence (OccurrenceRef)    : document/version + causal pos + token span + BinderPlan node ID
3. ModelSymbol                   : pinned tokenizer token_id (u32) + byte-span mapping
4. GeometricCarrier              : immutable per-block E8 orbit root (0..16), frame/action, epoch + overlay
5. DerivedReadState (Controller) : causal pool V_q + sparse edges E_q + temporary finite state s_i
                                              │
                                              ▼
                 Deterministic Role-Conditioned Readout (ordered node_ids + actions)
                                              │
                                              ▼
                 Unchanged SpiralCore BFS Payload Transport (`g[BFS + f_i*16 + f_q]`)
                                              │
                                              ▼
                 Unchanged Integer Snap (`snap`) + Ternary Level Gates (`{-1, 0, +1}`)
                                              │
                                              ▼
                 Unchanged Shared Integer Vector Writer (`discrete.cu:229-253`)
```

### Byte-storage DAG vs. Token `BinderPlan` Boundary
- Showcase byte-storage trees group fixed 32-byte slices (`core.js:166-183`), whereas the model's `BinderPlan` (`crates/uor-r4-discrete/src/metadata.rs:14-178`) groups **token occurrences** (`1..=243`). Sharing ternary branching does not make them the same tree (`reviews/math-cross-review.md:27-37`, `reviews/ml-cross-review.md:44-51`).
- **Contract:** This design card preserves the canonical token-occurrence `BinderPlan` and its node IDs for candidate admission and trace parity, linked to backing byte objects (`ObjectRef`) through explicit occurrence-to-byte-span bindings. Any future experiment that instead feeds compression-program chunk trees directly as model structure is a separate input-representation fork that must be declared and tested for encoding sensitivity separately.

---

## 3. Component 1 — Causal Candidate Pool ($\mathcal{V}_q$) and Exclusion of the $\le 16K$ Shortlist

### 3.1 Why the $\le 16K$ Flock Shortlist is Invalid as the Head's Pool
Under `ReadSpec::Flock`, every candidate sharing one of the 16 frame roots $r \in \{0,\dots,15\}$ has identical primary keys `(d_L1(f_q, r), d_Hamming(f_q, r))` for every query frame $f_q$. Consequently, discarding candidates beyond the most recent $K$ within each frame root preserves `Flock`'s global top $K$ (`reviews/math.md`, `reviews/math-cross-review.md:14-20`).

**That dominance proof fails under candidate–candidate interaction:** an older occurrence at rank $K+1$ within frame root $r$ can have a decisive support, contradiction, or version-chain relation to an anchored node that elevates its relational priority above the $K$ newer occurrences in frame $r$. Therefore:
- **Constraint:** The $\le 16K$ Flock shortlist **must never** be used as a silent prefilter for the relational head's candidate pool.

### 3.2 Admitted Causal Pool $\mathcal{V}_q$
- For a causal prefix of valid length $L \in \{1,\dots,C\}$ ($C \le 243$; $C = 192$ in the registered completion shape), `BinderPlan::new(C)` (`metadata.rs:27-72`) defines $N_{\text{nodes}}(C) = \frac{3C - s_3(C)}{2}$ nodes across `stages` levels ($286$ nodes at $C=192$; at most `MAX_NODES = 364` at $C=243$, `discrete.cu:83`).
- The query token is the last valid prefix position $q_{\text{pos}} = L - 1$.
- For each level $l \in \{0,\dots,\text{stages}-1\}$, a node `node < nodes` is admitted to the causal pool $\mathcal{V}_{q,l}$ if and only if:
  $$\text{level}[\text{node}] == l \quad \land \quad \text{last}[\text{node}] + 1 < L$$
  identical to the causal eligibility cut in `discrete.cu:168`. The union $\mathcal{V}_q = \bigcup_{l} \mathcal{V}_{q,l}$ has $|\mathcal{V}_q| < 286$ at $C=192$ (and $|\mathcal{V}_q| < 364$ at $C=243$), requiring no heuristic pre-truncation.
- **Eligible-Count Trace Parity (`reviews/math-cross-review.md:23`):** For every `(row, block, level)`, the head records the exact total `eligible = |\mathcal{V}_{q,l}|` count in `lv[3]` (`discrete.cu:169, 183`) alongside the selected $K$-list.

### 3.3 Strict Causal & Suffix/Target Isolation Contract
- The pool $\mathcal{V}_q$, all node frames/payloads, all relation edges $\mathcal{E}_q$, and all controller updates must be functions **strictly of the causal prefix `[0..L)` and the effective model state `(epoch, virtual proposal overlay)`**.
- **Prohibition on backing-store leakage (`reviews/ml-cross-review.md:34-41`):** Even when an underlying compressed byte object stores a full document spanning beyond $L-1$, no dictionary frequency, whole-object digest, post-$L$ occurrence link, or target identity `targets[row]` may enter $\mathcal{V}_q$ or $\mathcal{E}_q$.

---

## 4. Component 2 — Bounded Sparse Edge Generator ($\mathcal{E}_q$)

A sparse final graph does not guarantee bounded work if constructing it scans all $\binom{|\mathcal{V}_q|}{2}$ candidate pairs (`reviews/math-cross-review.md:88`, `reviews/ml-cross-review.md:42`). The edge generator must construct $\mathcal{E}_q$ in **$O(|\mathcal{V}_q|)$ work** using only deterministic, causally observable prefix indices, with **zero answer-aware or target-supplied oracle hints**:

| Edge Family $\tau$ | Construction Rule (All $O(|\mathcal{V}_q|)$ Prefix-Local) | Max Out-Degree per Node $i \in \mathcal{V}_q$ |
|---|---|---:|
| `0: QueryAnchor` | Directed link $(q \to i)$ from the query anchor at $L-1$ to candidate $i$ (implemented as unary query-anchor term or pinned node $q$). | $1$ |
| `1: BinderTree` | Existing `BinderPlan` parent–child and ordered sibling links `(c0, c1, c2) <-> parent` from `plan[node*4..node*4+3]` (`discrete.cu:128-131`) where both endpoints lie in $\mathcal{V}_q$. | $\le 6$ (3 children, 1 parent, 2 siblings) |
| `2: OccurrenceChain` | Single causal predecessor link $(i \to \text{prev}(i))$ connecting leaf/node $i$ to the most recent prior causal occurrence of the same `token_id` (or shared `ObjectRef`), maintained in one forward scan of `[0..L-1)`. | $1$ |
| `3: FrameBucket` | Using a 16-slot running table `last_in_frame[0..16]` updated in one forward scan over `node = 0..nodes`: link $i$ to the most recent prior same-level candidate in its own frame root $f_i$ (`last_in_frame[f_i]`) and in a learned/declared partner frame root $\pi(f_i) \in \{0,\dots,15\}$. | $2$ |

- **Total Edge Count Bound:** Each node $i \in \mathcal{V}_q$ participates in at most $D_{\max} \le 10$ directed edges, so $M = |\mathcal{E}_q| \le 10 |\mathcal{V}_q|$ ($M \le 2,860$ at $C=192$), constructed in a single $O(|\mathcal{V}_q|)$ linear pass with zero quadratic pair scan.
- **Canonical Edge Orientation (`reviews/math-cross-review.md:63`):** Every directed edge $e = (i \to j, \tau) \in \mathcal{E}_q$ with $j < i$ is stored once with canonical orientation $(i \to j)$. When computing the incident energy of node $j$, the edge is evaluated with the reversed offset ($\delta_\tau^{-1}$ in `G64` or $(-\delta_\tau) \bmod 6$ in `C6`) so that both endpoints evaluate the **exact same global integer potential $E(\mathbf{s})$**.

---

## 5. Component 3 — Immutable Evidence, Anchors, and Unselected State Realizations (`C6` vs `G64`)

### 5.1 Immutable Evidence vs. Mutable Controller State
Per [`reviews/physics.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/physics.md) (§ "What the continuous physics does and does not supply") and [`reviews/integrator.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/integrator.md):
- Every node $i \in \mathcal{V}_q$ retains its **immutable frame root** $f_i \in \{0,\dots,15\}$ (`discrete.cu:144-148`, where interior binder nodes inherit `frame[first_child]`) and its **immutable 8D payload** `payload[i]` (`discrete.cu:146-154`).
- The relational head introduces a separate, temporary finite controller state $s_i \in \mathcal{S}$ for $i \in \mathcal{V}_q$.
- **Invariance:** Controller updates modify $s_i$ only and **never overwrite** $f_i$, `payload[i]`, or token/occurrence identities. Consequently, the controller cannot erase a contradiction by moving evidence vectors into agreement.

### 5.2 Input and Query Anchors
Without anchoring, global phase/action shifts are a gauge freedom and uniform relaxation can collapse input distinctions (`reviews/physics.md:52-53`):
1. **Clamped Query Anchor ($s_q$):** The query at $L-1$ has a fixed, non-mutable anchor state $s_q = \alpha_q(f_q) \in \mathcal{S}$.
2. **Candidate Input Anchors ($a_i$):** Each candidate $i \in \mathcal{V}_q$ is initialized to its input-derived state $s_i^{(0)} = a_i = \alpha_{\text{in}}(f_i, p_i) \in \mathcal{S}$ and carries a bounded integer unary potential $U_i(s_i; a_i, f_q) \in \mathbb{Z}$ combining:
   - self-anchoring to its initial input state $a_i$ (weight $\lambda_{\text{self}} \in \{0, 1, 2\}$), and
   - direct query-anchor coupling to $s_q$ (weight $J_{b, l, \text{QueryAnchor}} \in \{-1, 0, +1\}$, offset $\delta_{b, l, \text{QueryAnchor}}$).

### 5.3 Two Candidate Finite State Realizations (`C6` vs `G64` — EXPLICITLY UNSELECTED)

Neither Option A nor Option B is selected by this card. Any future construction qualification sheet must declare which realization (or matched comparison of both) is under test:

| Property | Option A: Native `G64` / `R16` Geometric Realization | Option B: Separately Typed `C6` ($\mathbb{Z}/6\mathbb{Z}$) Phase Controller |
|---|---|---|
| **State Alphabet $\mathcal{S}$** | **Sub-option A1 (`R16`):** Orbit root label $s_i \in \{0,\dots,15\}$ (4 bits).<br>**Sub-option A2 (`G64`):** Signed-axis action label $s_i \in \{0,\dots,63\}$ (6 bits). | Discrete phase index $s_i = p_i \in \mathbb{Z}/6\mathbb{Z} = \{0, 1, 2, 3, 4, 5\}$ (multiples of $60^\circ$), mapped from roots via $\Pi: \{0,\dots,15\} \to \mathbb{Z}/6\mathbb{Z}$. |
| **Stabilizer / Group Scope (`math-cross-review.md:53, 65`)** | In $G_{64}$, the stabilizer of one anchor root $r_0$ has order $64/16 = 4$; an energy observing only $s_i \cdot r_0$ collapses to the 16-state quotient `R16`. Sub-option A2 (`G64`) must therefore evaluate $s_i$ on two non-collinear roots $(f_i, p_i)$ or via `g[COMPOSE]`. By Lagrange's theorem, $|G_{64}| = 64 = 2^6$ has **no order-3 element**, so Option A does not claim a cyclic $120^\circ$ subgroup. | $\mathbb{Z}/6\mathbb{Z}$ has an exact order-3 subgroup $\{0, 2, 4\}$ ($0^\circ, 120^\circ, 240^\circ$) and order-2 opposition $\{0, 3\}$ ($0^\circ, 180^\circ$). It is a separately typed controller alphabet, not a subgroup of $G_{64}$. It contains no $90^\circ$ state, so no I/Q orthogonality claim is imported. |
| **Exact Integer Lookup Table** | Existing `root_cost` $16 \times 16$ L1 distance $D_{\text{L1}}(r_a, r_b) \in [0, 32]$ and action table `g[COMPOSE]` / `action` (`discrete.cu:19-22, 37-40`). | Exact 6-entry integer doubled-cosine table:<br>$C[\Delta \bmod 6] = [2, 1, -1, -2, -1, 1]$<br>with zero float, trig, or multiplier ops. |
| **Oriented Edge Potential $\Phi_e(s_i, s_j)$ for $e = (i \to j, \tau)$** | With $J_\tau \in \{-1, 0, +1\}$ and target offset $\delta_\tau \in G_{64}$:<br>$\Phi_e^{(G)}(s_i, s_j) = -J_\tau \cdot \bigl(16 - D_{\text{L1}}(\text{obs}(s_i),\, \delta_\tau \cdot \text{obs}(s_j))\bigr) \in [-16, +16]$<br>(for $j \to i$, replace $\delta_\tau$ by $\delta_\tau^{-1} = \text{g[INVERSE + }\delta_\tau\text{]}$). | With $J_\tau \in \{-1, 0, +1\}$ and phase offset $\delta_\tau \in \{0,\dots,5\}$:<br>$\Phi_e^{(C6)}(p_i, p_j) = -J_\tau \cdot C[(p_i - p_j - \delta_\tau) \bmod 6] \in [-2, +2]$<br>(for $j \to i$, replace $\delta_\tau$ by $(6 - \delta_\tau) \bmod 6$). |
| **Frustrated Triad Behavior** | Three-way separation over distinct roots/actions is representable via negative pair terms $J_\tau = -1$, governed by $E_8$ orbit L1 distances $\{0, 8, 16\}$. | On an equal negative triangle ($J = -1, \delta = 0$), $2\sum_{i<j}\cos(\theta_i-\theta_j) = |\sum_i e^{i\theta_i}|^2 - 3 \ge -3$, achieving its exact integer minimum $-3$ on the $120^\circ$ triad $(0, 2, 4)$. |
| **Total Energy Range & Strict-Descent Step Bound ($M = |\mathcal{E}_q|$)** | Pair energy lies in $[-16M, +16M]$ (range $32M$) plus unary range $32 \lambda_{\text{self}} |\mathcal{V}_q|$. Every accepted strict integer decrease drops $E$ by $\ge 1$. | Pair energy lies in $[-2M, +2M]$ (range $4M$) plus unary range $4 \lambda_{\text{self}} |\mathcal{V}_q|$. Every accepted strict integer decrease drops $E$ by $\ge 1$, bounding accepted moves by $\le 4M + 4\lambda_{\text{self}}|\mathcal{V}_q|$. |

### 5.4 Deterministic Bounded Update Schedule
To guarantee reproducibility and bounded GPU execution without assuming rapid convergence to a global minimum (`reviews/math-cross-review.md:86-89`, `reviews/physics.md:77`):
1. **Initialization:** Set $s_i^{(0)} = a_i$ for all $i \in \mathcal{V}_q$.
2. **Sequential Sweep Order:** For sweep $t = 1, \dots, S_{\max}$ (where $S_{\max} \in \{1, 2, 4\}$ is a fixed compile/spec constant):
   - Visit candidates $i \in \mathcal{V}_q$ in ascending canonical `BinderPlan` `node` index.
   - Evaluate candidate states $s' \in \mathcal{S}$ in ascending numerical index $0, \dots, |\mathcal{S}|-1$ against the **exact sum of all incident oriented edge terms and unary anchor terms** $\Delta E_i(s_i \to s')$.
   - Choose $s^*$ minimizing $\Delta E_i(s_i \to s')$, breaking ties by keeping the current state $s_i$ first, then lowest state index $s'$.
   - Accept $s_i \leftarrow s^*$ **if and only if $\Delta E_i(s_i \to s^*) < 0$** (strict integer energy decrease).
3. **Stopping & Trace Receipt:** Terminate early if a full sweep accepts $0$ moves; otherwise stop at $t = S_{\max}$ and record `cap_hit = 1` in the trace (never misreporting a cap-bounded state as a local minimum).

---

## 6. Component 4 — Deterministic Role-Conditioned Readout

### 6.1 Rejection of Unconditional Synchronization / `PAS >= 0.82` Filtering
Per [`reviews/physics.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/physics.md) (lines 55, 81) and [`reviews/ml-cross-review.md`](../labs/spiralcore-native-attention-review-2026-10-14/reviews/ml-cross-review.md) (line 27):
- A candidate that is anti-aligned ($180^\circ$ / opposite root) or frustrated ($120^\circ$ triad) relative to the query or another premise may hold the **exact contradiction, exception, or superseding correction** the query requires.
- Unconditionally filtering for in-phase synchronization (or enforcing a global Phase Alignment Score threshold such as `PAS >= 0.82`) is **prohibited** in this design.

### 6.2 Role-Conditioned Relational Priority Key and Exact `Flock` Reduction
After the bounded update loop produces settled controller states $\mathbf{s}^* = \{s_i^*\}_{i \in \mathcal{V}_q}$:
1. **Per-Level Target Relation Regime ($\rho_{b,l}$):** Each `(block b, level l)` has a discrete readout regime parameter $\rho_{b,l} \in \{\text{Neutral}=0,\, \text{Support}=1,\, \text{Conflict}=2,\, \text{IncidentHarmony}=3\}$:
   - $\rho_{b,l} = 0$ (`Neutral`): $d_{\text{rel}}(q, i;\, \mathbf{s}^*) = 0$.
   - $\rho_{b,l} = 1$ (`Support`): $d_{\text{rel}}(q, i;\, \mathbf{s}^*)$ rewards agreement with the query anchor $s_q$ (e.g., $-C[(p_i^* - s_q) \bmod 6]$ in `C6`, or $D_{\text{L1}}(\text{obs}(s_i^*), \text{obs}(s_q))$ in `G64`).
   - $\rho_{b,l} = 2$ (`Conflict` / `Contradiction`): $d_{\text{rel}}(q, i;\, \mathbf{s}^*)$ rewards opposition or frustrated separation from $s_q$ / neighbors (e.g., $+C[(p_i^* - s_q) \bmod 6]$ in `C6`, promoting $180^\circ$ opposition $p_i^* - s_q \equiv 3$ and $120^\circ$ frustrated states $p_i^* - s_q \equiv 2, 4$).
   - $\rho_{b,l} = 3$ (`IncidentHarmony`): $d_{\text{rel}}(q, i;\, \mathbf{s}^*) = E_i^{\text{incident}}(\mathbf{s}^*)$, prioritizing candidates that strongly satisfy their local relational neighborhood.
2. **Deterministic Selection Order (Top-$K$ per Level):**
   Within each level $l \in \{0,\dots,\text{stages}-1\}$, eligible candidates $i \in \mathcal{V}_{q,l}$ are ranked by the 5-tuple:
   $$\operatorname{key}_{b,l}(i) = \bigl(d_{\text{rel}}(q, i;\, \mathbf{s}^*),\, d_{\text{L1}}(f_q, f_i),\, d_{\text{Hamming}}(f_q, f_i),\, -\text{last}[i],\, \text{node\_id}[i]\bigr)$$
   - **Exact Reduction Property:** When $\rho_{b,l} = 0$ (or all couplings $J = 0$), $d_{\text{rel}} \equiv 0$ for every candidate, and $\operatorname{key}_{b,l}(i)$ reduces **bit-for-bit** to the existing `Flock` selector (`discrete.cu:170-182`).
3. **Unchanged Payload Transport, Consensus Snap, and Shared Writer:**
   - The top `found <= kept` ($K \in 1..=8$) candidates $i_0, \dots, i_{\text{found}-1}$ at level $l$ have their **immutable** payloads `payload[i_k]` transported from frame $f_{i_k}$ to query frame $f_q$ via the existing exact SpiralCore BFS action `a = g[BFS + frame[i_k]*16 + fq]` (`discrete.cu:186-189`).
   - The carried payloads are summed in `i32[8]` and snapped via `m = snap(sum, found, g)` (`discrete.cu:190`).
   - Level consensuses are combined into the query payload `u` using the ternary gate `gates[b*stages + l] in {-1, 0, +1}` and final `out = snap(u, terms, g)` (`discrete.cu:193-198`).
   - The resulting vector `output[0..width]` feeds the **unchanged** shared integer vector writer `writer(...)` (`discrete.cu:229-253`).

---

## 7. Component 5 — Outer Learning Slots, Schedule Exposure, and Cache Invalidation

### 7.1 Three Strictly Separated Contracts (`reviews/ml-cross-review.md:20-32`)
1. **Inner controller relaxation:** Minimizes integer potential $E(\mathbf{s})$ over temporary states $\mathbf{s}$ on a fixed prefix and fixed parameters.
2. **Deterministic readout:** Selects top-$K$ immutable candidate references per level using $\operatorname{key}_{b,l}(i)$ and transports their payloads via SpiralCore BFS actions.
3. **Outer counterfactual learning:** Evaluates candidate parameter moves solely by the **shared writer's integer hinge loss** $8\cdot\text{token\_hinge} + \text{width}\cdot\text{address\_hinge}$ over baseline-eligible rows (`discrete_counterfactual`, `discrete_choose`, `discrete_apply` in `discrete.cu:281-345`).
   - **Rule:** A lower inner energy $E(\mathbf{s}^*)$ or higher phase synchronization **never** counts as a training improvement, acceptance criterion, or model-quality metric.

### 7.2 Learnable Parameter Inventory and Counterfactual Move Lanes
All learnable relational parameters are small, finite categorical slots compatible with the 16-lane `discrete_counterfactual` / `discrete_choose` architecture (`discrete.cu:280-325`):

| Parameter Family | Shape at $(B=4, \text{stages}=5, |\mathcal{T}|=4)$ | Alphabet / Active Candidate Lanes | Tie-Break Rule in `discrete_choose` |
|---|---|---|
| `Parameter::VectorBlock` (existing, `kind=1`) | $V \times B = 16,384$ | $16$ lanes: identity (`0`) + $15$ $\mathrm{Cl}(0,6)$ bivector generators (`1..16`). | Strict loss decrease; identity first, then lowest `resulting_port`, then lane index (`discrete.cu:319-323`). |
| `Parameter::Gate` (existing, `kind=3`) | $B \times \text{stages} = 20$ | $3$ active lanes (`[0, -1, +1]` with current trit at lane `0`; lanes `3..15` stay `MAX`). | Strict loss decrease; lowest ordinal wins ties (`discrete.cu:315-318`). |
| `Parameter::RelCoupling` (new, `kind=4`) | $B \times \text{stages} \times |\mathcal{T}| = 80$ | $3$ active lanes: ternary coupling sign $J_{b,l,\tau} \in \{0, -1, +1\}$ (lanes `3..15` stay `MAX`). | Strict outer writer loss decrease; lowest ordinal wins ties (default initialized to $0$). |
| `Parameter::RelOffset` (new, `kind=5`) | $B \times \text{stages} \times |\mathcal{T}| = 80$ | **In `C6`:** $6$ active lanes ($\delta_{b,l,\tau} \in \{0,\dots,5\}$).<br>**In `G64` / `R16`:** $16$ active lanes (identity + $15$ bivector moves). | Strict outer writer loss decrease; current state at lane `0`, then lowest target state/port, then lane index. |
| `Parameter::RelReadout` (new, `kind=6`) | $B \times \text{stages} = 20$ (plus optional 16-slot $\Pi: R_{16} \to C_6$ in Option B) | $4$ active lanes for $\rho_{b,l} \in \{0, 1, 2, 3\}$ (and $6$ active lanes per root for $\Pi(r) \in \{0,\dots,5\}$ in Option B). | Strict outer writer loss decrease; current state at lane `0`, then lowest ordinal. |

### 7.3 Coordinate Schedule Exposure Requirement (`reviews/ml-cross-review.md:30`)
In the current training driver (`crates/uor-r4-discrete/src/training.rs:224-245`), all `16,384` `VectorBlock` coordinates are scheduled before the `20` read coordinates (`16,384..16,404`), so a short pilot (e.g., `1,640` batches) never attempts a single read parameter.
- **Mandatory Schedule Contract for Any Future Relational Head Test:** An interleaved or phase-budgeted coordinate schedule must be registered so that `RelCoupling`, `RelOffset`, `RelReadout`, and `Gate` coordinates are **actively attempted throughout the run**, and evaluation receipts must report `attempted`, `any_candidate_differs` (`sensitive`), and `accepted` counts broken down by each parameter family alongside writer cell admission (`B3`) and conditional ranks (`top-1`, `top-8`).

### 7.4 Virtual Proposal Overlay and Cache Invalidation (`reviews/cs.md:53-55`, `reviews/ml-cross-review.md:56-58`)
During `discrete_counterfactual`, each lane evaluates a virtual proposal `(kind, index, candidate)` without incrementing `epoch`. Any index or cache must bind `(epoch, kind, index, candidate)`:
- A virtual `VectorBlock` move on frame block $f = \text{frame\_block}(b, B)$ (`discrete.cu:107`) changes `frame[node]`, `FrameBucket` edges, query frame $f_q$, BFS transport actions, and — because $f \in \{0, B-1\}$ is an endpoint block — **also changes writer product-cell occupancy `token_cell` (`discrete.cu:220-224`)**. Both the relational graph and writer cell admission must refresh.
- A virtual `VectorBlock` move on an interior payload block $b \notin \{0, B-1\}$ preserves `frame[node]`, $\mathcal{V}_q$, $\mathcal{E}_q$, and writer cell occupancy, changing only carried payloads `payload[node]` and `token_distance`.
- A virtual `Gate` move changes only the level-consensus combination (`discrete.cu:192-196`), preserving $\mathcal{V}_q$, $\mathcal{E}_q$, $\mathbf{s}^*$, and selected candidates.
- A virtual `RelCoupling`, `RelOffset`, or `RelReadout` move preserves immutable frames $f_i$, payloads `payload[i]`, $\mathcal{V}_q$, $\mathcal{E}_q$, and writer cell occupancy, re-running only the bounded controller sweep and level selection for the affected `(block, level)`.

---

## 8. What has been tried, honestly

1. **`ReadSpec::Route` and `ReadSpec::Flock` (`crates/uor-r4-discrete/src/discrete.cu:41-201`)**:
   - `Route` (`11/11` finite controls PASS) and `Flock` (`6/6` finite controls PASS, #2395) are implemented and qualified on CUDA. The active seed-1 Rung-0 completion run (`#2029`) evaluates `Route`; `Flock` has not yet been run in a corpus training comparison.
2. **SpiralCore assessment and native reconstruction/attention review ([#2478](https://github.com/UOR-Foundation/uor-r4/pull/2478), [#2480](https://github.com/UOR-Foundation/uor-r4/pull/2480))**:
   - Established the byte-level golden-route/FBS round-trip primitives, selective range reads over Showcase ternary DAGs, the conditional $\le 16K$ `Flock` index theorem, and the mathematical/ML requirements for a finite relational head.
3. **Finite relational head implementation and execution (`NOT_RUN`)**:
   - **Zero code for the relational head exists in `crates/uor-r4-discrete/` today, and zero GPU or CPU runs of this head have been executed.**

---

## 9. What a fair test needs (Stage-1 Construction vs. Stage-2 Trained-In Bar)

### Stage 1 — Finite CUDA Construction Qualification (`NOT_RUN`, requires separate pre-registration + `TEST FITNESS: FIT` + owner release)
Before any corpus exposure, a standalone finite qualification must verify:
1. **`admit_ptx` Zero-Float Gate:** `0` floating-point or tensor-core instructions in the compiled PTX module.
2. **Exact Zero-Coupling `Flock` Parity Control:** When $J_{b,l,\tau} = 0$ and $\rho_{b,l} = 0$, the relational head produces bit-identical traces (`eligible`, `found`, `top[0..K]`, actions, carried roots, consensus root), output vectors, and writer losses to `ReadSpec::Flock` across all 6 existing Flock controls.
3. **Synthetic Edge-On / Edge-Off Selection Sensitivity Control (`reviews/ml-cross-review.md:15`):** With query $q$, candidate pool $\mathcal{V}_q$, independent frame distances `(d_L1, d_Hamming)`, recency, and immutable payloads held fixed, activating a single declared relation edge $(i \to j, \tau)$ changes the selected candidate reference `top[0]` in the exact direction predicted by the finite integer energy and readout law.
4. **Matched Relevant-Change vs. Irrelevant-Neighbor Control (`reviews/integrator.md:101`, `reviews/math-cross-review.md:82`):** Changing an anchored relevant relation alters selection as predicted, whereas inserting or changing an uncoupled ($J=0$) or orthogonal-role neighbor leaves selection unchanged (ruling out indiscriminate neighbor sensitivity).
5. **Opposed / Contradictory Evidence Retrieval Control (`reviews/physics.md:55, 81`):** On a negative-coupled pair ($J = -1$) or frustrated triangle, setting $\rho_{b,l} = \text{Conflict}$ selects the anti-aligned/frustrated candidate rather than erasing it or averaging its payload into consensus before selection.
6. **Prefix / Suffix / Target Causal Isolation Control (`reviews/ml-cross-review.md:38-40`):** Holding prefix `[0..L)` fixed while altering tokens at `[L..C)` and altering `targets[row]` leaves $\mathcal{V}_q$, $\mathcal{E}_q$, $\mathbf{s}^*$, selected node IDs, and output vector $q_{\text{out}}$ 100% bit-identical.
7. **Strict Integer Descent & Sweep-Cap Termination Control:** Every accepted state transition strictly decreases $E(\mathbf{s})$ ($\Delta E \le -1$), immutable evidence frames/payloads are untouched, and `cap_hit` is accurately recorded when $S_{\max}$ is reached.

### Stage 2 — Downstream Trained-In Comparison (`NOT_RUN`, `HELD` — requires Stage 1 PASS + separate sheet + `TEST FITNESS: FIT` + owner training release)
- **End-to-End Causal Input Relations Only:** All edges $\mathcal{E}_q$ must be generated strictly from the prefix via the declared $O(|\mathcal{V}_q|)$ generators (`QueryAnchor`, `BinderTree`, `OccurrenceChain`, `FrameBucket`) with zero answer-aware or oracle relation labels.
- **Active Schedule Exposure:** The coordinate schedule must actively attempt `RelCoupling`, `RelOffset`, `RelReadout`, `Gate`, and `VectorBlock` slots, reporting `attempted / sensitive / accepted` counts per parameter family.
- **Matched Baseline & Separated Observables:** Compare against `ReadSpec::Flock` (tangent baseline: identical pool, transport, gates, and writer with $J \equiv 0$) and `ReadSpec::Route` at matched coordinate budget and writer lineage across $\ge 2$ seeds or $\ge 200$ fresh rows, reporting:
  1. Writer target-cell admission (`B3`),
  2. Conditional top-1 and top-8 ranks among admitted rows,
  3. Separated token hinge and address hinge loss numerators,
  4. Grounded reply quality and fresh transfer, and
  5. Complete synchronized GPU wall-time and memory cost (including graph construction, controller sweeps, counterfactual replay, and writer scans).

---

## 10. Open design questions (Explicitly Unselected — PM / Owner Decision Level)

1. **`C6` ($\mathbb{Z}/6\mathbb{Z}$) vs. `G64` / `R16` State Realization:** Should the first construction prototype implement Option A (native `R16`/`G64` geometric states using existing $E_8$ root tables) or Option B (separately typed `C6` phases with exact `[2, 1, -1, -2, -1, 1]` integer coupling and order-3 frustrated triads), or qualify both as matched arms?
2. **Cross-Level vs. Within-Level Relational Edges:** In Component 2, `BinderTree` parent–child edges connect level $l$ to level $l+1$, whereas `Flock` selects candidates level-by-level (`l = 0..stages`). Should controller relaxation run once across the unified multi-level graph $\mathcal{V}_q = \bigcup_l \mathcal{V}_{q,l}$ before per-level top-$K$ readout, or independently within each level $\mathcal{V}_{q,l}$?
3. **Interleaved Coordinate Schedule Ratio:** When training is eventually pre-registered, what ratio of `VectorBlock` proposals to relational/gate proposals (`RelCoupling`, `RelOffset`, `RelReadout`, `Gate`) balances representation movement against relational readout adaptation without multiplying full-pass wall time?
4. **Incorporation of Independent Adversarial Review ([#2029 comment 6175060216](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175060216)):** Any counterexample or interface refinement identified by Claude Fable 5.1's independent review of #2480 will be folded into this design card prior to drafting any Stage-1 construction qualification sheet.
