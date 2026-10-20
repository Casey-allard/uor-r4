# Finite relational head: `ReadSpec::Relation` bounded exact-relation reranker

**Milestones:** M1 #2029 and M4 #2032. **Authority:** Owner Decision [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) (`4cef409fc`, [#2028 comment 6176107062](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6176107062)), Kimi PM tasking and review ([#2028 comment 6175624248](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6175624248), [`6175886770`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6175886770), [`6178356343`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6178356343), [#2486 comment 6179014209](https://github.com/UOR-Foundation/uor-r4/pull/2486#issuecomment-6179014209)), Claude's independent adversarial review ([#2482](https://github.com/UOR-Foundation/uor-r4/pull/2482) at `618e422ce`, [`docs/labs/spiralcore-attention-independent-review-2026-10-14/README.md`](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md)), and Codex's native reconstruction/attention reviews ([#2478](https://github.com/UOR-Foundation/uor-r4/pull/2478), [#2480](https://github.com/UOR-Foundation/uor-r4/pull/2480), [`docs/labs/spiralcore-native-attention-review-2026-10-14/README.md`](../../archive/labs/spiralcore-native-attention-review-2026-10-14/README.md)).

> [!IMPORTANT]
> **Scope and Governance Boundary (D34 Reranker-First Revision — Source-Only Design Card, Zero Adoption, Zero Compute):**
> 1. **Primary falsifiable artifact is `ReadSpec::Relation { kept, sweeps }`:** Per [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) and [#2482](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) (§4.2–§4.4), the energy-controller class of machinery (`C6` / `G64` strict-descent settling) is **dropped as the first artifact** because a sweep-capped update over a fixed bounded graph ($\le 286$ nodes, depth $5$ at $C=192$) is already a deterministic $S$-sweep neighborhood aggregate whose energy, descent proof, and rejected-trial tie policy are never consumed by the readout. The minimal falsifier of relational selection is a **bounded exact-relation reranker** (`ReadSpec::Relation`) sharing `Flock`'s node construction (`discrete.cu:123-155`), using edges from retained structure only, and scoring via one ternary trit per `(edge_type, level)` on the **existing `Parameter::Gate` kind** (`lib.rs:125-131`; `discrete.cu:88-94, 315-317, 329-335`), with **`relation-off ≡ Flock`** as the exact rollback identity.
> 2. **Capability target is the copy-successor (induction) read (`3,847 / 14,287 = 26.9%` of pinned DEV targets):** Neither `ReadSpec::Route` (`discrete.cu:56-71`) nor `ReadSpec::Flock` (`discrete.cu:167-189`) can express *"the query token occurred earlier at $p$; carry the payload of the successor token at $p+1$."* On the pinned DEV corpus (`14,287` eligible targets at $C=192$), **`3,847` targets (`26.9%`)** equal the token following the most recent earlier in-window occurrence of the query token ([`evidence/README.md`](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/evidence/README.md)).
> 3. **`C6` vs `G64` demoted to a contingent arm (explicitly UNSELECTED, not needed for the first test):** Structural phase (`C6` = $\mathbb{Z}/6\mathbb{Z}$) and group (`G64` / `R16`) controllers are retained in §8 solely as contingent downstream extensions — opened only if the `ReadSpec::Relation` reranker result justifies structural controller machinery — and remain explicitly unselected.
> 4. **Pool = all eligible `C192` `BinderPlan` nodes ($\le 16K$ Flock shortlist excluded); shared writer untouched:** The candidate pool $\mathcal{V}_q$ is all causally eligible `BinderPlan` nodes (`last[node] + 1 < length`, $< 286$ nodes at $C=192$). The $\le 16K$ Flock shortlist remains excluded as the head's pool. The shared integer vector writer (`discrete.cu:229-253`) is untouched, and writer target-cell admission (`B3`) is reported separately from in-cell rank in every arm.
> 5. **D19 exact object-store crate is separate (not attention):** Per [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate), exact compressed-data reads belong to the D19 durable-memory line as a standalone Rust crate (`uor-r4-object-store`; warm/edit reuse benefit, whereas unique content expands $2.17\times$ in records and `corpus.rs:143-186` has no byte-transfer step in the model path).
> 6. **Stage M1 & Stage M2 pre-registration sheets (execution strictly gated):** Stage M1 finite CUDA construction qualification is pre-registered at `docs/harness/prep/m1-finite-relational-head-2026-10-14.md` (PR [#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494), `TEST FITNESS: FIT` by Claude at `4f0181b2c`), and the Stage M2 trained-in `Relation`-vs-`Route` discriminator under `SchedulePolicy::InterleavedReadEvery8V1` is pre-registered in draft form (`TEST FITNESS: PENDING`) at [`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md) on M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Any Stage M2 execution **strictly waits for seed-1's `Route` rows at cursor `16,404`** (first evidence that the proposal learner accepts any read parameter under `training.rs:224-241`), Stage M1 construction `PASS`, schedule/sub-panel controls `S1`–`S4` & `E1`–`E3`, prospective `TEST FITNESS: FIT` from Claude, and explicit owner release. Zero model/CUDA compute, zero head adoption, and zero hold changes occur here.

---

## 1. Idea from first principles, and the source-derived capability gap it targets

### 1.1 Why `Route` and `Flock` cannot express a copy-successor (induction) read
In the current discrete geometric model (`crates/uor-r4-discrete/src/discrete.cu`), both registered read modes cannot select the successor $p+1$ of an earlier match at $p$ to the current query token at $t = L - 1$:

- **`ReadSpec::Route` (`discrete.cu:41-77`):** Transforms the query block by a learned categorical action `routes[block * stages + stage]` (`0..=63`, validated by `if (frame >= 64) return false;` at `discrete.cu:50-51`) and selects one branch/frontier port by minimum `(distance, hamming, port)` to fixed orbit roots (`g[BRANCH + root*3 + port]`, lines `56-62`), carrying the leaf reached at the bottom of the ternary tree (`67-71`). Port selection does not inspect which tokens the leaves hold.
- **`ReadSpec::Flock` (`discrete.cu:118-201`):** Ranks each causally eligible node `node` at level `l` (`level[node] == l && last[node] + 1 < length`, line `168`) **independently** by its own frame root's distance to the query frame $f_q$, then recency (`170-182`):
  $$\bigl(d_{\text{L1}}(f_q, f_{\text{node}}),\, d_{\text{Hamming}}(f_q, f_{\text{node}}),\, -\text{last}[\text{node}],\, \text{node}\bigr)$$
  and transports *that node's own* payload `payload[node]` (`186-189`). When the query token $A$ at position $t = L-1$ appeared earlier at leaf $p$ followed by token $B$ at leaf $p+1$:
  - Leaf $p$ (holding $A$) has the exact same frame root as the query $f_q$, so $p$ ranks first in its frame class and `Flock` carries $A$'s own payload again.
  - Leaf $p+1$ (holding $B$) is ranked solely by $B$'s own frame root $f_{p+1}$, which has no deterministic relation to $f_q$. Nothing in `Route` or `Flock` can express: **"because $p$ matched the query $A$, promote its successor $p+1$ and carry $B$'s payload."**

### 1.2 Pinned DEV Corpus Capability Target (`3,847 / 14,287 = 26.9%`)
A model-free data statistic on the pinned DEV corpus (`tokens.u16` `0919e90f…`, `response_mask.u8` `a838bb1a…`, context window $C = 192$, document boundaries at BOS token `0`, `corpus.rs:246-290`, recorded in [`docs/labs/spiralcore-attention-independent-review-2026-10-14/evidence/README.md`](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/evidence/README.md)) establishes the exact scale of this missing capability:

| DEV Panel Quantity ($C = 192$, `corpus.rs:246-290`) | Row Count | Share of `14,287` Eligible Targets |
|---|---:|---:|
| Total eligible response-masked DEV targets | `14,287` | `100.0%` |
| Query token at $L-1$ recurs earlier inside the $192$-token causal window | `8,648` | `60.5%` |
| **Target equals the token immediately following the most recent earlier occurrence of the query token (copy-successor sub-panel)** | **`3,847`** | **`26.9%`** |
| … broken down by query-to-match distance buckets `1–8 / 9–32 / 33–96 / 97–192` | `50 / 2,872 / 900 / 25` | — |
| Target equals the successor of *any* earlier in-window occurrence of the query token | `5,182` | `36.3%` |
| Target token appears anywhere in the $192$-token causal window (copy ceiling) | `8,248` | `57.7%` |

For comparison on the same `14,287`-row DEV panel, the seed-1 `Route` model at Q1 (checkpoint `4,101`) achieves `1,090` top-8 hits (`2,201` admitted targets) and the masked-TRAIN count reference achieves `4,060` top-8 hits. The `3,847`-row copy-successor sub-panel is nearly $20\times$ larger than D22's `200`-row minimum evaluation bar.

### 1.3 Why the Energy Controller is Dropped as the First Artifact
In [#2480](../../archive/labs/spiralcore-native-attention-review-2026-10-14/README.md), relational selection was framed around a finite settling controller (`G64` root-state energy or `C6` phase energy) with strict integer energy descent, a sweep cap $S_{\max}$, and tie-breaking before readout. As shown in [#2482 §4.2](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md):
- Any sparse deterministic update run for at most $S$ sweeps over the bounded `BinderPlan` graph ($\le 286$ nodes, depth $5$ at $C=192$) is already a **deterministic $S$-sweep aggregate of each node's neighborhood**.
- The global integer energy $E(\mathbf{s})$, strict-descent proof, rejected-move tie policy, and `cap_hit` flag exist only to certify local energy-optimality — a property the top-$K$ readout never consumes.
- Testing the energy controller first would confound a negative result between the energy functional, the settling schedule, and the relational-selection hypothesis itself. The minimal falsifier is the **bounded $S$-sweep exact-relation reranker** (`ReadSpec::Relation`).

---

## 2. Architecture and Interface of `ReadSpec::Relation { kept, sweeps }`

### 2.1 Mode Dispatch and Five-Object Boundary
`ReadSpec::Relation { kept, sweeps }` adds a third variant to `ReadSpec` (`crates/uor-r4-discrete/src/lib.rs:61-107`) and kernel mode `MODE_RELATION = 2u` in `read_vector` (`crates/uor-r4-discrete/src/discrete.cu:204-207`), sharing `Flock`'s node construction (`discrete.cu:123-155`) and downstream payload transport, `snap`, level gates, and shared integer vector writer (`discrete.cu:185-201, 229-253`):

```text
1. ExactContent (D19 ObjectStore): separate Rust crate on D19 memory line (outside model path)
2. Occurrence & Identity         : retained token ID `ids` + causal leaf/span [first[i]..=last[i]]
3. BinderPlan Node Pool V_q      : all nodes < nodes with last[node] + 1 < length (< 286 at C=192)
4. Immutable Geometric Carrier   : per-node frame[i] in 0..16 and 8D payload[i] (`discrete.cu:141-155`)
5. ReadSpec::Relation Reranker   : retained-structure edges + S sweeps of Gate-kind trits g[e, l]
                                               │
                                               ▼
       Top-K Selection per Level: (rel_score desc, d_L1 asc, d_Hamming asc, last desc, node asc)
       (when all relation trits g[e, l] == 0, reduces bit-for-bit to ReadSpec::Flock)
                                               │
                                               ▼
       Unchanged SpiralCore BFS Payload Transport (`g[BFS + frame[node]*16 + fq]`, `186-189`)
                                               │
                                               ▼
       Unchanged Integer Consensus (`snap(sum, found, g)`) + Level Combine Gates (`190-198`)
                                               │
                                               ▼
       Unchanged Shared Integer Vector Writer (`writer(...)`, `discrete.cu:229-253`)
```

### 2.2 Candidate Pool $\mathcal{V}_q$ = All Eligible `C192` `BinderPlan` Nodes ($\le 16K$ Shortlist Excluded)
1. **Full Eligible `BinderPlan` Pool:** At registered context $C = 192$, `BinderPlan::new(192)` (`metadata.rs:27-72`) has $192 / 64 / 21 / 7 / 2$ nodes at levels $0..4$ ($286$ nodes total across `stages = 5` levels; capped at `MAX_NODES = 364` at $C = 243$, `metadata.rs:5`, `discrete.cu:83`). A node `i < nodes` belongs to level-$l$ pool $\mathcal{V}_{q,l}$ if and only if:
   $$\text{level}[i] == l \quad \land \quad \text{last}[i] + 1 < \text{length}$$
   (`discrete.cu:168`).
2. **Exclusion of the $\le 16K$ Flock Shortlist:** The $\le 16K$ per-frame top-$K$ shortlist dominance theorem holds only for `Flock`'s independent comparator `(d_L1, d_Hamming, -last, node)`. Under `ReadSpec::Relation`, an older occurrence $p$ or its successor $p+1$ outside the top-$K$ recency slice of its own frame root $f_{p+1}$ can receive a positive relational score from $p$'s match to the query and enter the global top-$K$. Moreover, at $C = 192$, scanning all eligible nodes across $5$ levels touches $< 286$ nodes total, and `frame_block(b, blocks)` (`discrete.cu:107`) maps `b = 0 -> blocks - 1` and `b in {1, 2, 3} -> 0`, so payload blocks `1..3` at `B = 4` already share a single query frame $f_q$ and candidate frame assignment ([#2482](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) §3 item 1, lines `148-153`). Thus the $\le 16K$ shortlist is excluded as the head's pool.

---

## 3. Retained-Structure Edge Types and Bounded Sweep Reranker Law

### 3.1 Five Edge Types from Retained Structure Only ($O(|\mathcal{V}_q|)$, Compare/Gather Only)
Edges are generated strictly from retained structure in `discrete.cu:119-155` (`plan`, `ids`, `level`, `last`, `frame`, plus a derived leftmost-leaf index `first[MAX_NODES]` computed in the same bottom-up loop `discrete.cu:123-132`: `first[node] = atom` when `atom != ABSENT`, and `first[node] = first[c0]` where `c0 = plan[node*4]` for interior nodes at `level >= 1`), with bounded degree, no quadratic all-pairs scan beyond the bounded node list, and zero target/oracle hints:

| Edge Type $e \in \{0..4\}$ | Retained-Structure Definition (Both Endpoints Must Satisfy `last <= length - 1`) | Role in Copy-Successor & Contextual Routing |
|---|---|---|
| `0: ParentChild` | Directed links between a `BinderPlan` interior node `node` and its three children `c0, c1, c2 = plan[node*4..node*4+3]` (`discrete.cu:128-131`). | Propagates phrase-level frame matches down to constituent leaves or leaf matches up to parent summaries. |
| `1: Sibling` | Directed links between adjacent siblings `(c0 <-> c1, c1 <-> c2)` sharing the same `BinderPlan` parent node. | Local within-phrase contextual transfer inside a 3-token or 3-child ternary binder group. |
| `2: SequentialSuccessor` | Same-level causal adjacency `last[j] + 1 == first[i]` (at level $0$, `first[node] == last[node] == atom`, so leaf `atom_i == atom_j + 1`; at levels $\ge 1$, `first[i] = first[plan[i*4]]` is the leftmost leaf of node `i`). | **Copy-successor (induction) primitive:** when earlier leaf $j = p$ matches the query token/frame at $L-1$, edge $(p \to p+1)$ transfers $p$'s match score to its immediate successor $i = p+1$. |
| `3: SameToken` | Retained token-ID equality (`ids[row*context + atom_i] == ids[row*context + atom_j]` at level $0$, linked to the most-recent prior occurrence of the same `token_id`). Compares retained token identity (`lib.rs:116-124`), never geometric address collision. | Connects repeated occurrences of the exact same vocabulary token across the causal prefix. |
| `4: SameFrame` | Orbit frame-root equality (`frame[i] == frame[j]` in `0..16`, linked to the most-recent prior same-level node in the same frame bucket). | Connects candidates sharing the same transported $E_8$ orbit frame root without requiring identical token IDs. |

- **Strict Causal Endpoint Invariant (`last <= length - 1`):** Because `BinderPlan` is precomputed for the full context capacity `s[2]` while `length = plan[0]` is the valid prefix length for the current row (`discrete.cu:120, 168`), **every edge generator must enforce `last[endpoint] <= length - 1` on both endpoints in the kernel**. Candidate eligibility for top-$K$ readout continues to enforce `last[i] + 1 < length` (`last[i] <= length - 2`, strictly before the query at `length - 1`), while a match at `length - 2` may have its `SequentialSuccessor` at `length - 1` (the query itself) without ever reading at or beyond `length`.

### 3.2 Node Match Signal and Bounded $S$-Sweep Integer Update (`d_L1 ∈ {0, 4, 8}`, Unclamped `i32`-Safe)
For each row and frame block $f = \text{frame\_block}(b, s[1])$ (`discrete.cu:107, 134`):
1. **Exact Orbit-Root L1 Distance Set `{0, 4, 8}` and Base Query-Match Score $m_i^{(0)} \in \{0, 4, 8\}$:**
   Every orbit root in `g[ROOT + r*8..]` (`r in 0..16`) is a signed-permutation image of the anchor `[2, 2, 0, 0, 0, 0, 0, 0]` (`crates/uor-r4-discrete/build.rs:217`), with exactly two non-zero coordinates of magnitude `2` (`±2`). Consequently, for any two orbit roots $f_q, f_i \in \{0..15\}$, `root_cost(query_frame, g, frame[i], &distance, &hamming)` (`discrete.cu:37-40, 170`) produces pairwise L1 distances in the exact three-element set:
   $$d_{\text{L1}}(f_q, f_i) \in \{0, 4, 8\}$$
   (with maximum `8`, as recorded in `crates/uor-r4-discrete/src/flock_qualification.rs:232-233`). For every node $i$ with `last[i] <= length - 1`, define the non-negative integer base match score:
   $$m_i^{(0)} = 8 - d_{\text{L1}}(f_q, f_i) \in \{0, 4, 8\}$$
   which achieves its maximum `8` when `frame[i] == fq` ($d_{\text{L1}} = 0$, as occurs for every earlier occurrence $p$ of the query token), `4` at intermediate alignment ($d_{\text{L1}} = 4$), and `0` at maximum orbit separation ($d_{\text{L1}} = 8$).
2. **Bounded $S$-Sweep Relation Score Accumulation (Unclamped, Exact `i32`):**
   Initialize the per-node integer relation score $r_i^{(0)} = 0$. For sweep $t = 1, \dots, S$ (`sweeps` $\in \{1, \dots, 5\}$, a declared `ReadSpec::Relation` constant; $S = 1$ suffices for 1-hop `SequentialSuccessor` copy-successor reads, and $S = 5$ reaches the full depth-5 `BinderPlan` tree):
   $$r_i^{(t)} = r_i^{(t-1)} + \sum_{e \in \{0..4\}} g[e, \text{level}[i]] \cdot \sum_{j \in \mathcal{N}_e(i)} s_{j}^{(t-1)}, \qquad s_j^{(t-1)} = \begin{cases} m_j^{(0)}, & t = 1 \\ r_j^{(t-1)}, & t \ge 2 \end{cases}$$
   where:
   - $g[e, l] \in \{-1, 0, +1\}$ is the learned ternary Gate trit for edge type $e$ at level $l$,
   - $\mathcal{N}_e(i)$ is the bounded set of retained-structure neighbors of node $i$ under edge type $e$ (all satisfying `last[j] <= length - 1`, total degree $D_{\max} \le 8$ across the 5 edge types), and
   - **No clamp is needed or applied:** Sweep $1$ propagates each neighbor's base match score $s_j^{(0)} = m_j^{(0)} \in \{0, 4, 8\}$, and sweeps $t \ge 2$ propagate the previous sweep's accumulated relation score $s_j^{(t-1)} = r_j^{(t-1)}$ directly without clamping. Because $|m_j^{(0)}| \le 8$, $|g[e,l]| \le 1$, degree $\le 8$, and $S \le 5$, the worst-case magnitude after $5$ sweeps is $|r_i^{(5)}| \le 8 \cdot (1 + 8)^5 = 472,392 \ll 2^{31} - 1$, which is strictly `i32`-safe without clamping. All arithmetic is exact `i32` addition/subtraction with zero floating point, zero division, zero multiplier instructions, and zero energy-minimization loop.

### 3.3 Top-$K$ Selection Comparator and `relation-off ≡ Flock` Identity
At each level $l \in \{0, \dots, \text{stages}-1\}$, eligible candidates (`level[node] == l && last[node] + 1 < length`) are inserted into the level's top-`kept` list ($K \in 1..=8$) by the deterministic 5-key comparator:
$$\operatorname{better}(i, o) \iff \begin{cases}
r_i^{(S)} > r_o^{(S)}, & \text{if } r_i^{(S)} \ne r_o^{(S)} \quad (\text{higher relation score wins}) \\
d_{\text{L1}}(f_q, f_i) < d_{\text{L1}}(f_q, f_o), & \text{else if } d_{\text{L1}} \text{ differs} \\
d_{\text{Hamming}}(f_q, f_i) < d_{\text{Hamming}}(f_q, f_o), & \text{else if } d_{\text{Hamming}} \text{ differs} \\
\text{last}[i] > \text{last}[o], & \text{else if recency differs (most recent wins)} \\
i < o, & \text{otherwise (lowest node ID)}
\end{cases}$$

- **`relation-off ≡ Flock` Rollback Identity:** When all relation trits are zero ($g[e, l] = 0$ for all $e, l$), $r_i^{(S)} = 0$ for every node $i$, and $\operatorname{better}(i, o)$ reduces **identically** to `discrete.cu:175-176`. Every selected node ID, BFS action, carried root, level consensus root, output vector, and writer loss is bit-for-bit identical to `ReadSpec::Flock`.
- **Copy-Successor Mechanism Trace (`x A B y z A`):** With query $A$ at $L-1$ and an earlier occurrence of $A$ at leaf $p$ followed by $B$ at leaf $p+1$, leaf $p$ has $m_p^{(0)} = 8$ (`frame[p] == fq`, $d_{\text{L1}} = 0$). Setting the level-0 `SequentialSuccessor` trit $g[\text{SequentialSuccessor}, 0] = +1$ (with $S = 1$) gives leaf $p+1$ (token $B$) a positive relation score $r_{p+1}^{(1)} = +8$, promoting $p+1$ ahead of any non-successor candidate ($r = 0$) regardless of that candidate's own frame distance $d_{\text{L1}} \in \{0, 4, 8\}$, and transporting $B$'s immutable payload into the readout.
- **Conflicting Successors Tie Order (`A B … A C … A`):** If $A$ occurred multiple times earlier with successors $B$ and $C$ (and neither $B$ nor $C$ is preceded by another matching token), both $B$ and $C$ receive equal relation score $r_B^{(1)} = r_C^{(1)} = +8$. Because `Flock`'s secondary keys `(d_L1(f_q, ·) asc, d_Hamming(f_q, ·) asc)` precede `last[node] desc` in $\operatorname{better}(i, o)$, **when $B$ and $C$ are tied on $(d_{\text{L1}}(f_q, \cdot), d_{\text{Hamming}}(f_q, \cdot))$** (e.g., when $f_B = f_C$ or $B$ and $C$ lie at equal L1/Hamming distance from $f_q$), the recency key (`last[node] desc`) deterministically selects the most recent successor $C$; when $(d_{\text{L1}}(f_q, f_B), d_{\text{Hamming}}(f_q, f_B)) \ne (d_{\text{L1}}(f_q, f_C), d_{\text{Hamming}}(f_q, f_C))$, the candidate whose frame root is closer to $f_q$ wins before recency is consulted.

---

## 4. Learning Slots on Existing `Parameter::Gate` (`kind = 3`) and Schedule Prerequisite

### 4.1 Reuse of Existing `Parameter::Gate` Kind (`kind = 3`)
Per [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) and [#2482 §4.4](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md), `ReadSpec::Relation` introduces **no new parameter kind and no new counterfactual lane**:
- Every relation trit $g[e, l] \in \{-1, 0, +1\}$ uses the existing **`Parameter::Gate(u32)`** variant (`crates/uor-r4-discrete/src/lib.rs:125-131`, CUDA `GATE_KIND = 3u`, `discrete.cu:84`).
- Virtual proposals use the existing `gate_value(gates, at, kind, index, candidate)` helper (`discrete.cu:88-94`), which evaluates the current trit at ordinal `0` and its two alternatives from `[0, -1, +1]` at ordinals `1, 2` (with lanes `3..15` padded to `MAX`).
- `discrete_choose` (`discrete.cu:313-318`) already sets `active = 3u` for `GATE_KIND`, reports `sensitive` if either alternative changes the outer writer loss, and accepts only a strict outer writer loss decrease (`losses[i] < losses[best]`, lowest ordinal winning ties).
- **Gate Inventory in `Relation` Mode:** At context $C = 192$ (`stages = 5`, `blocks = 4`, `edge_types = 5`):
  $$\text{Gate slots} = \underbrace{\text{blocks} \times \text{stages}}_{\text{level combine gates } = 20} + \underbrace{\text{edge\_types} \times \text{stages}}_{\text{relation trits } g[e,l] = 25} = 45 \text{ trits}$$
  all initialized to `0` (`relation-off ≡ Flock` at initialization).
- **Exact Source Touchpoints for a Future M0 Prototype (Documented, Not Mutated Here):**
  1. `crates/uor-r4-discrete/src/lib.rs:61-107`: add `ReadSpec::Relation { kept: u32, sweeps: u32 }` to `ReadSpec` (`61-66`), `Wire` (`75-83`), `validate` (`88-99`, checking `kept in 1..=8`, `sweeps in 1..=5`, `spec.blocks >= 2`), and `words` (`100-106`, encoding mode `2`, `kept`, and `sweeps`).
  2. `crates/uor-r4-discrete/src/device.rs:470-489, 590-660`: grow `gates` device allocation (`472-474`) and `from_snapshot` gate-length check (`607`) from `blocks * stages` (`20`) to `blocks * stages + EDGE_TYPES * stages` (`45`) for `ReadSpec::Relation`, and plumb `sweeps` into `spec_gpu` / `gpu_spec` (`476-489, 645-660`, e.g., `s[10] = sweeps`).
  3. `crates/uor-r4-discrete/src/discrete.cu:79-85, 123-132, 204-207, 265, 275, 284-287, 329-337, 354-355`:
     - Add `#define MODE_RELATION 2u` and `#define EDGE_TYPES 5u`, compute `first[MAX_NODES]` alongside `last[MAX_NODES]` in the bottom-up node loop (`123-132`), and dispatch `s[8] == MODE_RELATION` in `read_vector` (`204-207`) to `relation_vector` (extending `TRACE_LEVEL` with the kept nodes' relation scores);
     - Extend `discrete_init` gate-zero bound (`265`) and `discrete_validate` gate-trit range and mode/parameter checks (`354-355`) to cover `s[1]*s[5] + EDGE_TYPES*s[5]` Gate trits and validate `s[8] == MODE_RELATION` (`kept in 1..=MAX_KEPT`, `sweeps in 1..=5`, `s[1] >= 2`);
     - Extend `discrete_forward` trace condition (`275`, passing `trace` when `s[8] == MODE_FLOCK || s[8] == MODE_RELATION`);
     - Extend `discrete_counterfactual` (`284-287`) and `discrete_apply` (`329-337`) so `GATE_KIND` is admitted in both `MODE_FLOCK` and `MODE_RELATION` (and `kind == 0` refused in both), with `index` bounded by `s[1]*s[5] + EDGE_TYPES*s[5]` in `MODE_RELATION`.
  4. `crates/uor-r4-discrete/src/training.rs:179-189, 236-259, 285`: include the `EDGE_TYPES * stages` relation Gate trits in `parameter_count` (`16,429` at `C192 / B4`) when `config.read_spec` is `ReadSpec::Relation { .. }`, map `ReadSpec::Relation { .. } => CoordinateKind::Gate` in `PreparedDriver::coordinate()`, and compute `next_coordinate = (completed_batches % schedule_cycle) as u32` modulo the declared `schedule_cycle` (`16,404` under `InterleavedReadEvery8V1`).
  5. `crates/uor-r4-discrete/src/main.rs:102-133`: `init-read` (`102-109`) accepts `ReadSpec::Relation { kept, sweeps }` via `InitConfig` deserialization/validation, and `propose` (`129-133`) refuses `Parameter::Route` on `ReadSpec::Relation` snapshots just as on `ReadSpec::Flock`.

### 4.2 Cost at the Registered Shape (`C192 / B4 / V4096`)
Per row and frame block (2 unique frame blocks at `B = 4`, since blocks `1..3` share frame block `0` via `discrete.cu:107`), `ReadSpec::Relation` performs at most $286 \text{ nodes} \times \le 8 \text{ edges} \times \text{sweeps} \approx 10^4$ integer operations — approximately **`0.2%` of the shared writer's $\approx 5 \times 10^6$ vocabulary-cell comparisons per row** (`discrete.cu:231, 241`; [#2482](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) §3 item 3 lines `158-165`, §4.4 lines `268-271`, and §8 line `350`).

### 4.3 Learning-Slot Prerequisite: Wait for Seed-1 Cursor `16,404` + Interleaved Schedule ([`m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md))
Under the currently registered schedule `canonical_vector_then_read_v1` (`crates/uor-r4-discrete/src/training.rs:224-243`), `PreparedDriver::coordinate(index)` maps the first `vocabulary * blocks = 16,384` batches to `CoordinateKind::VectorBlock` and proposes read parameters (`Route` or `Gate`) **only in the last `blocks * stages = 20` batches (`16,384..16,404`) of a pass**.
- **Consequence 1 (Wait for Seed-1 at `16,404`):** Whether the counterfactual proposal learner ever accepts *any* read-parameter update is unmeasured until the active seed-1 `Route` completion run crosses batches `16,384..16,403` and finishes at cursor `16,404`.
- **Consequence 2 (Interleaved Schedule Required on Both Arms):** Under `canonical_vector_then_read_v1`, a `Relation` head would receive zero read-parameter proposals during the entire `0..16,383` vector pass. A fair trained-in `Relation`-vs-`Route` comparison therefore requires a **declared interleaved coordinate schedule** (`SchedulePolicy::InterleavedReadEvery8V1`: proposing the next read coordinate every 8th batch, $s \bmod 8 == 7$, wrapping across `20` `Route` or `45` `Relation` Gate coordinates for `2,050` read proposals and `14,354` `VectorBlock` proposals across `schedule_cycle = 16,404` batches, with `next_coordinate = (completed_batches % schedule_cycle) as u32`) applied symmetrically to **both** `Route` and `Relation`, registered explicitly as a learner change with its own `validate_cursor` arithmetic check (`training.rs:245-259`).
- **Governance Rule:** This Stage M2 trained-in comparison is pre-registered in draft form (`TEST FITNESS: PENDING`) at [`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md) on M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032); its execution remains **strictly gated** until seed-1's `16,404` `Route` rows are observed ([#2492](https://github.com/UOR-Foundation/uor-r4/pull/2492)), Stage M1 construction qualification passes ([#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494)), schedule/sub-panel controls `S1`–`S4` & `E1`–`E3` pass, Claude records `TEST FITNESS: FIT`, and the owner explicitly releases the hold.

---

## 5. Shared Writer Boundary and Separated Observables

Per [#2482 §5](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) and [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate):
1. **Shared Writer Untouched:** `writer(...)` (`discrete.cu:229-253`) is unchanged across `Route`, `Flock`, and `Relation`.
2. **Endpoint-Block Admission Fact (`discrete.cu:208-210, 220-224`):** Product-cell admission (`s[3] = 8` cells out of `256`) is determined by `address_distance(q, g, s, cell)`, which reads **only endpoint blocks `0` and `B - 1` (`s[1] - 1`)** of the read output `q`. Interior payload blocks (`1` and `2` at `B = 4`) cannot change which cells are admitted; they affect only in-cell `token_distance` ranking (`discrete.cu:211-215, 241-245`).
3. **Mandatory Separated Observables:** Because target-cell admission is the measured ceiling (`2,201 / 14,287` DEV targets admitted at seed-1 Q1, with `0` hits outside admitted rows), every evaluation and future comparison must report separately:
   - Admitted-target count (`B3`, overall and on the `3,847`-row copy-successor sub-panel),
   - Conditional `top-1` and `top-8` accuracy inside admitted rows (overall and on the `3,847`-row copy-successor sub-panel),
   - Separated `token_hinge` and `address_hinge` loss components (`discrete.cu:249-252`), and
   - Per-parameter-family `attempted / sensitive / accepted` proposal counts (`VectorBlock`, `Route`, `Gate` combine trits, and `Gate` relation trits).

---

## 6. What has been tried, honestly

1. **`ReadSpec::Route` and `ReadSpec::Flock` (`crates/uor-r4-discrete/src/discrete.cu:41-201`):**
   - `Route` (`11/11` finite controls PASS) and `Flock` (`6/6` finite controls PASS, #2395) are implemented and qualified on CUDA. Seed-1 Rung-0 completion (`#2029`) is currently running `Route`; `Flock` has not been trained on the corpus.
2. **SpiralCore byte/tree assessments and independent review ([#2478](https://github.com/UOR-Foundation/uor-r4/pull/2478), [#2480](https://github.com/UOR-Foundation/uor-r4/pull/2480), [#2482](https://github.com/UOR-Foundation/uor-r4/pull/2482)):**
   - Reproduced byte-exact golden-route/FBS codecs and selective range reads over Showcase ternary DAGs; proved that unique content expands $2.17\times$ in records (placing the exact object store on the D19 durable-memory line under D34); proved that neither `Route` nor `Flock` can express the copy-successor read; and measured the `3,847 / 14,287` (`26.9%`) copy-successor DEV sub-panel.
3. **`ReadSpec::Relation` implementation and execution (construction and discriminator `NOT_RUN`; one trained arm executed):**
   - Implemented at `discrete.cu:78-267`, `device.rs:341-356`, `device.rs:1231-1246`, `device.rs:1512-1514`, `lib.rs:76-79,116-127,140-153`; qualified by `relation_qualification.rs` (C1–C8) (2026-10-17 correction; a stale "zero code" sentence was struck 2026-10-18). A trained `rows_v1` arm executed on 2026-10-18: Relation against frozen-zero reached 4,096 batches in both arms and stopped at the configured Q1 admission collapse (`P_succ` target admission 2,034 → 225 treatment / 205 control), with no accuracy or paired win and no KEEP ([diagnosis](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md)). What remains unexecuted is the **Stage M1 construction qualification** and the **Stage M2 Relation-vs-Route discriminator**, both owner-release-gated; see [d34-successor-table-control-2026-10-17.md](../harness/prep/d34-successor-table-control-2026-10-17.md) §6.9.

4. **Every trained Relation run so far started from aliased keys (finding, 2026-10-18).** The default xorshift initializer gives **16 distinct token vectors** at checkpoint 0. That has been known since `a2-pilot-diagnosis-2026-10-14`, and the W0 census and `coverage-census` (#2678) confirm it: 256 tokens share each vector.
   - The Relation read ranks context nodes by relation score over those payloads, so at initialization its keys separate only 16 classes. The Dose 2 `rows_v1` arms above began there.
   - Their codebook could diversify only through single-coordinate moves: 2.4% of attempts were accepted, and the census reached 264 distinct vectors at a full Q4 dose. Those moves also drove the admission collapse.
   - So **no trained run so far isolates whether the Relation read can learn.**
   - An injective initialization is now available for new lineages (#2692/#2694: PrimeZeta, 4,096 vectors, DEV alias ceiling 14,287 / 14,287, zero tie mass). The draft sheet `m1-attention-injective-init-DRAFT-2026-10-18.md` asks this head's trained question on it, with exactly one variable changed from `m1-learned-attention-link-2026-10-17.md`.

---

## 7. What a fair test needs (M1 Construction Controls vs. Later M2 Trained-In Discriminator)

**Prerequisite for any trained Relation test (added 2026-10-18).** The run must start from a codebook whose keys can be told apart, and it must show that with artifact-level coverage gates (D39 item 2):
- the DEV alias ceiling, raw and net of admission, is at or above the sheet's bar;
- within-cell tie mass is reported;
- occupied cells are more than 16.

A run on the 16-vector xorshift initialization measures codebook diversification as much as the read, and it cannot decide this head.

### 7.1 Stage M1 — Finite CUDA Construction Qualification (`NOT_RUN` — pre-registered in `docs/harness/prep/m1-finite-relational-head-2026-10-14.md`, [#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494), `TEST FITNESS: FIT` by Claude at `4f0181b2c`; execution gated on owner release)
Before any corpus training or model adoption, a finite construction qualification must pass all 10 controls enumerated in [#2482 §9](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) (lines `368-374`; previously grouped as 9 when controls 7 and 8 were combined into a single bullet) and [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate):
1. **`relation-off ≡ Flock` Parity Control:** With all relation trits $g[e, l] = 0$, `ReadSpec::Relation` produces bit-identical outputs, writer losses, and kept-node traces (`eligible`, `found`, `top[0..K]`, BFS actions, carried roots, consensus root) to `ReadSpec::Flock` across all inputs.
2. **Copy-Successor Fixture Control (`x A B y z A`):** On a prefix containing an earlier occurrence of the query token `A` followed by `B` (with distinct frame roots chosen so neither `Route` nor `Flock` selects `B`), setting the level-0 `SequentialSuccessor` trit to $+1$ gives `B`'s occurrence relation score $r = +8$ and selects `B` as `top[0]`, verified directly in the kept-node trace.
3. **Irrelevant-Change Control:** Modifying unrelated context positions (`y`, `z` in `x A B y z A`) leaves the relation score of `B` and the selected candidate `top[0]` unchanged.
4. **Conflicting Successors Control (`A B … A C … A`):** When the query token `A` occurred twice earlier with different successors `B` and `C` chosen with equal query-frame distances `(d_L1(f_q, f_B), d_Hamming(f_q, f_B)) == (d_L1(f_q, f_C), d_Hamming(f_q, f_C))`, both `B` and `C` receive equal positive relation score $r = +8$ from `SequentialSuccessor`, tie on `d_L1` and `d_Hamming`, and the recency tie key (`last[node] desc`) deterministically selects the most recent successor `C` (whereas if `B` and `C` are given unequal `d_L1(f_q, ·)`, the secondary `d_L1 asc` key breaks the tie as specified in §3.3).
5. **Suffix / Target Causal Isolation Control:** Holding prefix `[0..length)` fixed while mutating tokens at `[length..context)` or mutating `targets[row]` leaves the pool, edges, relation scores, kept-node trace, and output vector 100% bit-identical.
6. **`last <= length - 1` Endpoint Boundary Control:** Explicitly verifies that no edge is formed with any node having `last[node] >= length` and that a match at `length - 2` cannot read a successor past `length - 1`.
7. **Deterministic Ties Control:** Verifies that equal-score ties across candidates are broken deterministically in the exact order `(rel_score desc, d_L1 asc, d_Hamming asc, last desc, node asc)`.
8. **Bounded Work (`sweeps`) Control:** Verifies that `sweeps` executes the exact declared constant number of sweeps ($S \in 1..=5$) with bounded work and rejects out-of-range `sweeps` (`0` or `> 5`) in `ReadSpec::validate` and `discrete_validate`.
9. **17 Base Controls Unchanged:** All `11` `Route` controls and `6` `Flock` controls continue to pass unchanged.
10. **`admit_ptx` Zero-Float Gate:** `0` floating-point or tensor-core instructions in the compiled PTX module.

### 7.2 Stage M2 — Trained-In Discriminator (`NOT_RUN`, `HELD` — pre-registered in draft form at [`m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md), `TEST FITNESS: PENDING` on M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032); execution strictly gated behind seed-1 at `16,404` + Stage M1 `PASS` + `FIT` + owner release)
Pre-registered in [`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md) per [#2482 §10](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) (execution remains strictly `HELD` until seed-1 reaches cursor `16,404`, Stage M1 passes, schedule/sub-panel controls `S1`–`S4` & `E1`–`E3` pass, Claude records `TEST FITNESS: FIT`, and the owner explicitly releases the hold):
- **Matched Arms:** `Route-interleaved` (`ReadSpec::Route`) vs. `Relation-interleaved` (`ReadSpec::Relation { kept: 8, sweeps: 1 }`) sharing the exact same D5 initializer, corpus, response masks, tokenizer, shared writer, batch rows (`64`), seed, panel, and symmetric interleaved coordinate schedule (`SchedulePolicy::InterleavedReadEvery8V1`: every 8th batch $s \bmod 8 == 7$ proposes the next read coordinate, yielding `2,050` read proposals and `14,354` `VectorBlock` proposals across `16,404` batches); only `read_spec` differs.
- **Primary Pre-Registered Bar (`B1`–`B3`):** Top-8 hits (with non-decreasing top-1 and admitted-target count `B3`) on the **`3,847`-row copy-successor DEV sub-panel** $\mathcal{P}_{\text{succ}}$ (`B1`), required to exceed `Route-interleaved` by strictly more than the seed-to-seed noise floor $\Delta_{\text{seed}}(\mathcal{P}_{\text{succ}}) = \max(|\text{seed } 1 - \text{seed } 2|, 39\text{ rows})$, without full-panel (`14,287`-row) top-1, top-8, or admitted-target (`B3`) regression beyond full-panel seed noise (`B2`).
- **Zero-Trit Attribution Check (`B3`):** Evaluate the trained `Relation-interleaved` cursor-`16,404` checkpoint once with all `25` relation trits `gates[20..45]` forced to `0` at evaluation time (no retraining) and require a strict positive sub-panel top-8 delta (`trained Relation > trained Relation with g[e,l]=0`) plus `accepted_updates(GateRelation) >= 1` to isolate the exact contribution of learned relational selection at the trained state.
- **D22 Closure Scope:** A negative result on `Relation-interleaved` closes only that specific `(edge_set, sweeps = 1, kept = 8, C192/B4/V4096, InterleavedReadEvery8V1)` configuration, never the relational attention mechanism family ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)).

---

## 8. Contingent Downstream Extensions: Structural Controller Realizations (`C6` vs `G64` — Demoted, Explicitly Unselected)

Per [D34](../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) and Kimi PM's direction ([#2028 comment 6175886770](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6175886770)), the structural state/energy controllers from [#2480](../../archive/labs/spiralcore-native-attention-review-2026-10-14/README.md) are **demoted to contingent arms**: they are **not needed for the first test (`ReadSpec::Relation`)**, remain **explicitly unselected**, and will be opened only if the `ReadSpec::Relation` reranker result demonstrates that multi-state constraint settling or frustrated phase/group separation is needed beyond additive $S$-sweep Gate-trit reranking. Their exact specifications from `#2480` are preserved below so no design derivation is lost:

| Property | Contingent Arm A: Native `G64` / `R16` Geometric Controller | Contingent Arm B: Separately Typed `C6` ($\mathbb{Z}/6\mathbb{Z}$) Phase Controller |
|---|---|---|
| **When Opened** | Only if `ReadSpec::Relation` shows that 64-element signed-axis / 16-root state transitions on nodes are required beyond scalar score reranking. | Only if `ReadSpec::Relation` shows that order-3 frustrated triads ($120^\circ$) or signed phase offsets are required beyond scalar score reranking. |
| **State Alphabet $\mathcal{S}$** | `R16` orbit root label $s_i \in \{0,\dots,15\}$ (4 bits) or `G64` action label $s_i \in \{0,\dots,63\}$ (6 bits). By Lagrange's theorem, $|G_{64}| = 64 = 2^6$ has **no order-3 element**; the anchor stabilizer has order $4$ (`math-cross-review.md:53, 65`). | Discrete phase index $p_i \in \mathbb{Z}/6\mathbb{Z} = \{0, 1, 2, 3, 4, 5\}$ (multiples of $60^\circ$), with order-3 subgroup $\{0, 2, 4\}$ ($120^\circ$ triad) and order-2 opposition $\{0, 3\}$ ($180^\circ$), separately typed from $G_{64}$ and carrying no $90^\circ$ I/Q claim. |
| **Exact Integer Table & Potential** | Uses `root_cost` $16 \times 16$ orbit-root L1 distance $D_{\text{L1}} \in \{0, 4, 8\}$ (`discrete.cu:37-40`) and `g[COMPOSE]` / `g[INVERSE]` with oriented pair potential $\Phi_e^{(G)}(s_i, s_j) = -J_\tau \cdot (4 - D_{\text{L1}}(\text{obs}(s_i), \delta_\tau \cdot \text{obs}(s_j))) \in [-4, +4]$. | Uses exact 6-entry integer doubled-cosine table $C[\Delta \bmod 6] = [2, 1, -1, -2, -1, 1]$ with oriented pair potential $\Phi_e^{(C6)}(p_i, p_j) = -J_\tau \cdot C[(p_i - p_j - \delta_\tau) \bmod 6] \in [-2, +2]$, achieving $-3$ on frustrated negative triangles $(0, 2, 4)$. |
| **Prohibited Filtering** | Unconditional synchronization filtering and global thresholds (`PAS >= 0.82`) remain prohibited in both contingent arms so opposed/contradictory evidence is never erased before readout (`reviews/physics.md:55, 81`). | Same prohibition: readout must remain role-conditioned (`Neutral`, `Support`, `Conflict`, `IncidentHarmony`) over immutable evidence frames $f_i$ and payloads `payload[i]`. |
| **Selection Status** | **UNSELECTED (Contingent — deferred behind `ReadSpec::Relation`).** | **UNSELECTED (Contingent — deferred behind `ReadSpec::Relation`).** |

---

## 9. Open design questions (Scoped to `ReadSpec::Relation`)

1. **Sweep Count Constant (`sweeps = 1` vs. `sweeps = 2..5`):** A single sweep ($S = 1$) suffices for 1-hop `SequentialSuccessor` copy-successor reads on level-0 leaves (`x A B y z A`), whereas $S \ge 2$ allows `SameToken` or `ParentChild` edges to chain with `SequentialSuccessor` across binder levels. Should the first M1 construction qualification test `sweeps in {1, 2}` as a parameter of `ReadSpec::Relation { kept, sweeps }`?
2. **Cross-Level `ParentChild` Edge Inclusion in the First Pass:** `Flock` loops over levels `l = 0..stages` sequentially (`discrete.cu:159`). If `ReadSpec::Relation` computes node relation scores $r_{\text{node}}^{(S)}$ once across all `nodes` before the level loop `l = 0..stages`, `ParentChild` edges between level $l$ and $l+1$ work naturally alongside within-level edges (`Sibling`, `SequentialSuccessor`, `SameToken`, `SameFrame`).
3. **Interleaved Schedule Ratio for Stage M2 ([`m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md), Gated on Seed-1 Reaching `16,404`):** [`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md) pre-registers `SchedulePolicy::InterleavedReadEvery8V1` (`s % 8 == 7` proposing the next read coordinate wrapped across `20` `Route` or `45` `Relation` Gate coordinates for `2,050` read proposals and `14,354` `VectorBlock` proposals across the dose-matched `16,404`-batch pass, alongside the `18,432`-batch `s % 9 == 8` full-vector coverage option) for Claude's `TEST FITNESS` review once seed-1's `Route` rows at `16,404` are inspected.
