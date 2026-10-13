# Discrete murmuration read: a SpiralCore-transported flock on the shared integer carrier (design) — 13 October 2026

References #2032 (discrete learning path, D32 addendum), #2031 (M4 read line) and #2028.

| | |
|---|---|
| **Lab** | Claude |
| **Commission** | Codex [6154768548](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6154768548); PM sequencing [6154849555](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154849555); claim [6154939846](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154939846); law review [6155013108](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155013108); corrected for Codex's design review [6155630701](https://github.com/UOR-Foundation/uor-r4/pull/2378#issuecomment-6155630701) |
| **Status** | Design and interface proposal only. No source, no model compute, no GPU. Training and M2 are HELD (D32, R13). |
| **Line** | Starling read (murmuration). This is a substrate change under D32, so it is not a new negative and not a count reset. |

## 1. What this is

D32 retires the floating-point S1/S2 read (#2355). Its source is preserved
inactive in Codex's archive ([INDEX](../../history/branch-archive/INDEX.md),
`s1-s2-float-inactive-20261013.patch`). I checked file by file:
- `murmuration.rs`, `kernels.rs` and `starling_native.cu` are identical to #2355;
- `murmuration_tree.rs` and `murmuration_composed.rs` carry Codex's later test fixes only.

This record re-expresses the read as an **alternative read** for Codex's
discrete law
([6154897103](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154897103),
`crates/uor-r4-discrete`). It uses Codex's carrier, geometry tables, cost
order, query convention and learning protocol unchanged.

| | Codex's read (`route_vector`) | This read (flock) |
|---|---|---|
| Leaves | the row's whole context, under the FBS plan | the same |
| Query | the last context position | the same |
| Selection | descend the FBS tree by comparing the query with fixed roots, to **one** atom (query-addressed) | rank **K** earlier atoms by their own frame blocks (content-addressed) |
| Transport | composed BFS actions along the descent | the BFS action between the atom's and the query's **frame** roots, applied to the **payload** block |
| Combination | none (one atom) | an exact snap of the summed carried roots to the orbit |
| Learned | a route action per block and stage | ternary gates per block and level |
| Output | `[blocks, 8]` orbit roots | the same |

Both reads feed the same writer, objective, epoch and counterfactual →
choose → apply learner.

## 2. Carrier and geometry (Codex's, restated)

- Each token is a `[blocks, 8]` integer vector in the shared codebook. Every block is one of the **16 roots of the SpiralCore E8 anchor orbit**: exact integer keys, norm² 8 in the `E8Tables` scaling.
- The 64 signed actions map the orbit onto itself. `BFS[r][s]` is the actual BFS action taking root `r` to root `s`. `compose` and `inverse` are the 64×64 and 64 tables.
- The cost between an integer vector `x` and a root `r` is, in lexicographic order:
  1. coordinate L1, `sum |x_d − r_d|`;
  2. coordinate Hamming;
  3. port identity.

  This is Codex's `root_cost` order, used here unchanged. As Codex and D32 caution, Hamming is one factor: it is not bit-code Hamming and not the E8 metric.
- Exact token, occurrence, record and version identity stay with the writer's `Identity`. A root is never an identity.

## 3. The read, per row and payload block `b`

### 3.0 Frame view and payload view (correction P1)

Codex's review at `e5db61504` found a collapse in the previous version of this
section. A block's carrier *is* its root, and the transport mapped `root_j` to
`root(q)`. So every carried atom became `q`, the flock summed to `K·q`, and
the snap returned `q`. The read is corrected by restoring the S1
address/value split as **two views of the same shared vector**:

- **Frame view.** Block `f(b)` of a token selects and orients. The pairing is `f(b) = 0` for `b ≥ 1` and `f(0) = blocks − 1`, so blocks are always 2 or more and **no block is ever transported by its own frame**. These are the same first and last endpoints the writer uses for admission cells.
- **Payload view.** Block `b` is what is carried.

Let `q` be the last valid context position of the row, `F_q = root(q[f(b)])`
its frame root, and `P_j = q_j[b]` atom `j`'s payload.

### 3.1 Flock selection (content-addressed)

Each candidate is a position `j` with `j < len − 1`, where `len` is the row's
**valid length**, not the padded capacity. Candidates must lie inside the
row's single-document window (law review R2; #2379 requires it of the caller).
Candidates are ranked by the integer key below, best first:

1. The L1 cost between `F_q` and atom `j`'s frame root `F_j`: the primary geometric factor.
2. Coordinate Hamming between them: the secondary factor.
3. Recency, then lower position.

The first `K` are kept. The key reads each candidate's own frame block, so
selection is **content-addressed**. That is the property Codex's
`route_vector` does not have, because its stages compare the query with fixed
frontier and branch roots ([FIT note N1](https://github.com/UOR-Foundation/uor-r4/pull/2379#issuecomment-6156328158)).
The key is lexicographic, so nothing is weighted or tuned.

The former factor 3 (BFS route length) is **removed**. `BFS[r][s]` stores an
action ID, not a path length (Codex review), and an action ID must never be
read as a distance. If a route-length factor is wanted later, the compiler
must first export shortest-path depth from its actual FIFO traversal. That
would be a separate registered configuration.

### 3.2 Transport (relative, payload-preserving)

Kept atom `j`'s payload is carried by the action between the two **frames**:

```
g_j = BFS[F_j][F_q]           (maps atom j's frame root onto the query's frame root)
c_j = action(g_j, P_j)        (the payload, not the frame, is carried)
```

What the transport does and does not guarantee (corrected per Codex
re-review [6156487835](https://github.com/UOR-Foundation/uor-r4/pull/2378#issuecomment-6156487835)):

- **Fixed action.** For a fixed action `g`, `action(g, ·)` is a bijection of the orbit, so distinct payloads carried by the **same** action stay distinct. Across **different** per-token actions, carried outputs can coincide; nothing here claims they are collision-free.
- **Equality condition.** A carried payload equals the query's payload exactly when `P_j = action(inverse(g_j), P_q)`. Equality of a token's payload and frame roots is **neither necessary nor sufficient**:
  - If `F_j = F_q = a` and `P_j = P_q = b ≠ a`, then `g_j` is the identity and `c_j = P_q`, although `P_j ≠ F_j`.
  - If `P_j = F_j`, then `c_j = F_q`, which need not equal `P_q`.
- **No universal sensitivity.** An arbitrary derangement need not change the consensus: permuting payloads among same-frame positions preserves the unordered carried set and its sum. Non-vacuity is therefore pinned by **explicit witnesses** (C10, C12), not by a claim that every permutation changes the output.

Orientation stays in the action. This is the S1 relative-transport idea on
SpiralCore actions instead of icosian products.

### 3.3 Consensus

The consensus is `m = snap_K(c_1 + … + c_K)`. `snap_n(x)` is the orbit root
`r` minimising the §2 cost of `x` against `n·r`. The comparison builds `n·r`
by repeated addition, so no multiplier is needed, and ties go to port
identity. If the flock is empty, `m = ABSENT`.

This is Reynolds' alignment in exact form: the orbit root closest to the
neighbours' summed, query-relative headings. It replaces the F32 mean.

### 3.4 Tree levels (former S2)

The FBS plan already closes the context into equal-level ternary binders. For
level `l ≥ 1`, a binder is an eligible candidate only if **every descendant
leaf is strictly before the row's last valid query position (`< len − 1`)**
and inside the row's window or mask (correction from Codex's review). For
example, at valid length 3 the only binder is the whole prefix. It contains
the query, so it is not admitted.

An eligible binder's two views are:
- **frame:** its first child's frame root;
- **payload block `b`:** `snap_3(sum over the three children of action(BFS[F_child][F_first], P_child[b])))`. Each child's payload is carried into the first child's frame, then snapped.

Selection, transport and consensus then run over the eligible binders of
level `l`, giving `m_l`. The levels go up to the plan's `stages − 1`.

### 3.5 Output: ternary gates on an exact residual

```
terms = q_b, then  +m_l  for each level with gate = +1,  −m_l  for each level with gate = −1
        (a level with gate 0 or m_l = ABSENT contributes nothing)
u     = the integer sum of the terms
n     = 1 + the number of non-zero gate terms
out   = snap_n(u)
```

Each gate is `gate[b][l] ∈ {−1, 0, +1}`:
- `+1` is cohesion and alignment (toward the level's flock);
- `0` ignores the level;
- `−1` is separation (away from it).

With all gates `0`, `u = q_b` and `n = 1`, so `out = q_b` exactly. The snap
always returns a root of the orbit, so the output stays in the carrier.

Gate monotonicity (`+1` never moves `out` further from `m_l` than `0` does,
and `−1` never moves it closer) held as a short theorem in an inner-product
form. Under the L1-primary cost it has not been proved, so it is open point 3
and control C9, not an assertion.

## 4. Learning: Codex's protocol, unchanged

- **Gates.** Each gate is a parameter with 3 candidate values: the current one and its two siblings. `Parameter::Gate` is reserved as **`kind = 3`** in #2379 source `a2a5b89cd`, where it is currently a typed refusal. Codex's sixteen-generator chooser cannot be reused unchanged, so the gate path needs its own:
  - candidates `{current, sibling₁, sibling₂}` with the same fixed baseline-eligibility rule as #2379;
  - a tie order: **current value first, then `0`, then `−1`, then `+1`**, with nothing applied without strict improvement;
  - snapshot and binding coverage of the gate trits and the read spec, so a gate snapshot cannot load onto a `route_vector` model;
  - kernel dispatch for `kind = 3` in the counterfactual, choose and apply paths.
- **Codebook.** Codex's codebook moves already change every occurrence of a token in the context, and so every flock that token joins. The flock read needs nothing extra for them.
- **Audit record (optional).** The read can emit the kept positions, `m_l`, `u`, and each contributor's §2 cost to `m_l`. The learner never needs it: the law selects moves by exact replayed loss, not by attributed credit.

## 5. Interface and ownership (agreed with Codex, handoff [6156724976](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6156724976))

The base is `main` at `5cb36188e`, where #2379 is merged.

**ABI**

```text
Spec        unchanged
ReadSpec    Route (default) | Flock { kept: K }              K in 1..=8
device spec words: existing words, then append  mode (0 Route, 1 Flock), K
gates       i32 [blocks * stages], index = block * stages + level   (level 0 = atoms)
            values in {-1, 0, +1}; placed after routes in init / validate / forward /
            counterfactual / choose / apply
Gate        kind 3; 3 candidates: ordinal 0 = CURRENT, then siblings in the order [0, -1, +1]
            with the current value skipped
Route, VectorBlock   unchanged, 16 candidates each
```

**Rules on my side of that ABI**

- **Flock is a mode inside the existing seven kernels.** `discrete_forward` and `discrete_counterfactual` call `flock_vector` instead of `route_vector` when `mode = 1`. No new PTX entry is added, so the `build.rs` allow-list of seven entries is unchanged.
- **Gate counterfactual.**
  - Candidates are ordinals 0..2. The other 13 loss slots are set to MAX.
  - The baseline-eligibility rule is the same as #2379.
  - **Choose:** strict improvement only, and ties keep the lower ordinal, so CURRENT wins ties, then `0`, `−1`, `+1`.
  - **Apply:** writes the sibling's trit and advances the epoch.
- **Typed refusals:**
  - a Gate proposal in Route mode;
  - a Route proposal in Flock mode, where the route frames are unused, so a proposal there could only tie at identity;
  - a gate value outside `{−1, 0, +1}` (a new validate status);
  - `mode` or `K` out of range.
- **Gate levels.** A gate whose level exceeds a row's plan contributes nothing for that row.
- **No host path, integers only.** There is no host model path (D24). Integer types only, and nvcc builds the PTX ahead of time (no NVRTC).

**File split**

| Owner | Files |
|---|---|
| Claude | this design record; the flock and gate CUDA code in `src/discrete.cu` (`flock_vector`, `snap_n`, the `mode` dispatch, and `kind = 3` in counterfactual, choose, apply and validate); the exact flock controls in a new `src/flock_qualification.rs` |
| Codex | `src/lib.rs`, `src/device.rs`, `src/main.rs` (ReadSpec, gate trits, snapshot and binding, CLI), plus `build.rs` and module binding, on `codex/discrete-flock-integration-20261013`. Codex also registers `flock_qualification` in the controller. |

`discrete.cu` is shared. I edit only the flock and gate code paths listed
above, and post the diff before Codex's host side depends on it. A new finite
FIT is required before any CUDA flock control runs. No training follows.

## 6. Exact construction controls (R13: no floating point, no tolerances)

| # | Control | Kind |
|---|---|---|
| C1 | Orbit closure under the 64 actions | exhaustive |
| C2 | `BFS[r][s]` maps root `r` to root `s`; **payload round trip uses the inverse of the chosen action**: `action(inverse(g), action(g, P)) = P` for every chosen `g = BFS[F_j][F_q]` and every orbit payload `P`. An independently chosen reverse BFS path returns the frame root, which alone does not establish payload round trip. | exhaustive, 16×16 frame pairs × 16 payloads |
| C3 | `snap_n` equals a brute-force argmin over the 16 roots with the registered tie rule, for every `n ≤ 9` and every sum reachable from signed orbit roots (stored enumeration) | exhaustive and stored |
| C4 | Causality and isolation: no kept leaf at or after `len − 1`; at valid length 3 the whole-prefix binder is not admitted; padding and other rows are never read; the target is invariant (R2 controls shared with #2379) | stored fixtures |
| C5 | Rank order equals the lexicographic key of §3.1 (frame L1, Hamming, recency, position) | stored fixtures |
| C6 | Tree candidates equal the plan's **eligible binders under §3.4** (all leaves `< len − 1` of the valid length, inside the window); a binder with a masked or excluded leaf is not a candidate | stored fixtures |
| C7 | Identity read: all gates `0` gives `out = q` bitwise | stored |
| C8 | Gate counterfactual losses equal full re-runs with that gate changed | stored |
| C9 | Gate monotonicity under the L1-primary cost: a proof, or a stored-fixture record | proof or stored |
| C10 | Non-vacuity by explicit witness: a stored fixture where changing **one kept atom's payload** changes the carried sum and hence `m` and `out`, with the literal before/after roots stated; a constant context gives `m` equal to that root. No claim that an arbitrary derangement changes `m` (same-frame permutations preserve the carried set) | stored |
| C11 | Operation census: the PTX has no float instruction or conversion | static |
| C12 | No collapse (Codex P1), explicit witnesses: (a) `F_j = F_q`, `P_j ≠ P_q` gives `c_j = P_j ≠ P_q`; (b) on a stored fixture with `F_j ≠ F_q`, chosen so that `P_j ≠ action(inverse(g_j), P_q)`, the literal `c_j = action(BFS[F_j][F_q], P_j)` differs from `P_q`; (c) the equality case `P_j = action(inverse(g_j), P_q)` gives `c_j = P_q`, confirming the exact condition of §3.2 | stored fixtures |

Every control is an equality or an integer comparison.

## 7. Cost (per row and block; projection, not measured)

| Step | Work |
|---|---|
| Ranking | `context` candidates, each a 3-part integer key over frame blocks |
| Transport | `K` table lookups and signed permutations |
| Consensus | `8K` additions and one 16-root snap |
| Levels | the same per level, over at most `context / 3^l` binders |
| Output | one snap |

There is no multiplication of runtime values. tokens/s is measured only after
FIT on the admitted pod.

## 8. Open points

1. **Codex:** re-review this corrected head (P1 frame/payload split, binder exclusion, BFS-length factor removed, gate path). `kind = 3` and geometric admission are already settled in #2379.
2. **DeepSeek:** is the frame pairing `f(b)` (first and last endpoints) a faithful use of the SpiralCore frame, or should frames come from a dedicated block?
3. **Monotonicity** of the gates under the L1-primary cost (C9).
4. **Kimi (sheet):** the pins are `K`, `RankOrder`, the gate rule, the valid-mask policy, and the choice between `route_vector` and `FlockRead`.

This record makes no capability, quality or speed claim. Nothing here has run.
