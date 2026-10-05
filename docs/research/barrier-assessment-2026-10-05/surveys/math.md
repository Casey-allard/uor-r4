# UOR-R4 mathematical programme: what is implemented, what is claimed, and the previous-token key channel

**Scope.** I read the repository only, at `71b54adf`. I made no builds, ran no training and changed nothing. Status labels follow `docs/formal_vocabulary.md` §1–2:
- **Proven/defined**: a definition or theorem.
- **Measured**: a result, given with its exact scope.
- **Hypothesis**: not yet measured.

## 1. Each primitive: proven or defined, implemented, measured, or analogy only

The production model path is `crates/uor-r4-training/src/geometric_stack.rs` together with the D19 grounded session. I counted literal mentions of each primitive in that file (21,938 lines):

| Mention | Count |
|---|---|
| `quaternion` | 106 |
| `Lorentz` | 154 |
| `prime` | 38 |
| `icosian` | 17 |
| `H4` | 6 |
| `zeta` | 1, and that one is a doc link to ADR-0003 (line 595) |
| `phi` / `Hopf` / `Zeckendorf` / `chiral` | 0 |

| Primitive | (a) Proven or defined | (b) In the production path | (c) Measured to help | (d) Analogy only |
|---|---|---|---|---|
| **Primes as token addresses, semiprime transition `e_t = p_{t-1}p_t`, ordered n-lets `N_t`** | Defined (`formal_vocabulary.md:90`). By unique factorization, a square-free product identifies an unordered *set* of atoms exactly. That is the whole mathematical content. | Partly. `token_prime` (`geometric_stack.rs:717`) and opt-in pointer routes `prime:`, `prime-ranked:`, `ngram:`, `ngram-ranked:` (#1587, #1589). The relation store's semiprime key is in `stack_prime_route.rs`. That module says itself that the sieve read "is a pairwise membership scan … though no product or gcd is formed". | **Negative or tie.** The exact routes reached MQAR 2–3 against 16 for the soft pointer. The ranked routes are "statistically indistinguishable", with a non-significant edge at 8 facts (4/24 against 0/24) (#1512 comments; scope: emit-1 recipe, M-world v2). | The arithmetic is an injective hash plus set intersection. It carries no metric. "Primes as triangulation" is exact only combinatorially: square-free divisors of N form the face lattice of a simplex. That is a Boolean lattice with no geometry. |
| **Prime/gcd sieve routing** | Defined (ADR-0003, `docs/adr/0003-fixed-zeta-prime-route-attention.md`). | The D19 session's exact log sieve is external memory, not the network. | Measured large, but for the *external store*: grounded sessions 972→1008. Per the task brief (I did not re-check it), the network alone scores 3/109 MQAR with the sieve off. | No. It is real exact lookup. |
| **Fixed zeta-zero phases `θ_j(p) = wrap(γ_j log p)`** | Defined. Critical-line use is an **Assumption**, not RH (`formal_vocabulary.md:91`). | **No.** Only the older core uses it: `uor-r4-core/src/zeta_projection.rs`, `prime_route_attention.rs`, and `native_geometric/training.rs:108–165` (8 phase channels, 16-bit turns). | No measurement against a control. `mathematics.md:18` says "Current native task controls do not universally establish zeta predictive benefit." | Its role is a fixed incommensurate frequency basis. No theorem favours zeta ordinates over other incommensurate frequencies. |
| **R4/S3 unit-quaternion state and transport** | Defined. | **Yes.** The `r` layers compute `h_t = λ_t(u_t h_{t-1}) + √(1−λ_t²) a_t` (`geometric_stack.rs:16–21`) with an exact scan. | Indirectly. #1704 shows `rrarra`/`rararr` reach MQAR 1.000 without the shift on some seeds, through the r-layer's width-4 causal convolution. There is no clean comparison of quaternion against ordinary recurrence. Catalogue 5.2: quaternion 2/5 against ordinary 0/5 greedy prose, which is too small to separate them. | — |
| **Hopf S3→S2 observation and retained fiber** | Defined and implemented: `UnitS3Q30::hopf`, `SpinTorsionState` in `prime_route_attention.rs:467–665`, `native_geometric/hopf_metric.rs`. | **No.** | **Never tested** (`retired-mechanisms-catalogue-2026-10-01.md` §6.1: "conditional, NOT_RUN for language"). | It is a correct observation quotient. Owner framings of an "S3→S2→S1" chain were mathematically corrected (`research-leader-handoff-2026-09-23.md:90`). |
| **H4 / 600-cell / 2I** | Proven and exact: 120-element closure, 9 conjugacy classes `[1,1,12,12,12,12,20,20,30]` (catalogue §4.6), table multiplication (`canonical_lexical_ingestion.rs:1339–1450`). | **Yes, as the transport snap.** `TransportSnap::Icosian` makes every `u_t` an exact 2I element by straight-through snap (`geometric_stack.rs:45–54`, 7261–7368). It is served in the D11 stack (#1506). | Adopted as **not harmful**, not as helpful: snapped 2.5480 against free 2.5374 nats, +0.0106 inside a ≤0.02 band (#1494). Stage A of B1: 2I/reflection lanes track A5 exactly at length 4,096 in 17/18 runs with one byte of state. Not in the model, because the Stage B text-cost gate failed (catalogue §4.1). Post-hoc 2I read codes lose ranking fidelity (D6, §4.5). The trained-in 2I read kernel was HARM (§5.4). | — |
| **E8 = H4 ⊕ φH4 icosians** | **Assumption-typed construction** (`formal_vocabulary.md:99`), with an exact inverse witness (`canonical_lexical_ingestion.rs:2577–2616`). | No. Only `Codebook::E8` exists as a weight-coding option. | B3 weight-coding kill on a disputed instrument (catalogue 8.10). Relationship-state use: **zero measurements**. | "240 roots are not 240 orthogonal registers" (handoff note, quoted in the catalogue). |
| **Exact Z[φ]** | Defined; `ZPhi` is in `prime_route_attention.rs:381`. | **No** (0 mentions of phi in the stack). | None. | Fibonacci recurrence of φ-steps "does not establish semantic value" (`formal_vocabulary.md:95`). |
| **Chirality / polarity** | Chart bookkeeping (`canonical_lexical_ingestion.rs:2650–2720`, `anchors.rs`). | No. | "no semantic axis measured" (catalogue §6.1). | Mostly. |
| **UOR identity** | Content identity of artifacts, tokenizer and data. | Yes, as provenance. | Not a model mechanism. | — |

**Summary of the table.** The production network uses three pieces of the declared geometry:
- quaternion (S3) transport in the recurrence;
- the optional exact 2I snap of that transport;
- a Lorentz/hyperbolic read score. Its parity with Dot was left unresolved (catalogue 8.2).

Zeta phases, Hopf, Z[φ], E8, chirality and prime arithmetic are absent from the network. Primes appear only in the external memory and in opt-in pointer routes. The one geometric mechanism with a large, clean in-context win is the newest one, the previous-token key channel. It is also the most elementary.

## 2. The previous-token key channel, `k̃_t = k_t + j·k_{t−1}`

### 2.1 What the code does

In `geometric_stack.rs:2125–2160`, the key at each `a`-layer read becomes `W_key u_t + L_j(W_key u_{t−1})`, with zero before position 0.
- `L_j` is left multiplication of each 4-channel lane by `j`. `quaternion_j_left` (`:8913`) implements it as `(a,b,c,d) ↦ (−c, d, a, −b)`. I checked this by hand: `j·1 = j`, `j·i = −k`, `j·j = −1`, `j·k = i`.
- It adds no parameters and no matrix product.
- It is refused in the QAT and served paths (`:8115`) and with geometric addressing.
- It is saved as `"read_key_shift": true` (#1704, `2cdae05a`).

### 2.2 Measured results, with their scope

All runs: CPU, context 512, width 128, 4 heads, 6 layers, 1,800 steps, 512 fresh-pairing queries per distance bucket.
- **#1701 (`3aa8ba16`), all-read stack:** 0.807/0.176/0.010/0.029 → **1.000 at d16/d64/d200/d400**, and held-out pairing class 0/1024 → 1024/1024.
  - The probe shows a single layer-0 head putting 0.994–1.000 of its weight on the value position.
  - Without the shift, weight on the key position never rises above uniform.
- **#1704:** `rrarra` without the shift fails on seed 1 (0.007) but solves on seed 2 (≈1.000). With the shift it solves on both seeds, and `rararr` reaches ≥0.9 at step 200 instead of 500.
- **Not measured** (#1704 says so): the D19 grounded cell, the chat panel, and any natural-language recall.

### 2.3 Why it works: the exact algebra

**(i) `L_j` is an isometry.** It is orthogonal (`|jx| = |x|`), so the predecessor channel has the same scale as the current channel, with no learned gain.

**(ii) `L_j` is skew and squares to −1.** `L_jᵀ = −L_j` and `L_j² = −I`, so `⟨x, jx⟩ = Re(x·conj(jx)) = |x|²·Re(−j) = 0`.
- A token's own key is exactly orthogonal to its j-turned copy.
- The same holds for every pure-imaginary unit quaternion `u`: the four vectors `{x, ix, jx, kx}` form an orthonormal basis of the lane, because `⟨ix, jx⟩ = |x|²·Re(i·(−j)) = |x|²·Re(−k) = 0`.

**(iii) It gives a one-layer induction head.** Take a query at token A that should land on the position s with `x_{s−1} = A`.
- With `q_t = L_j W_key x_t`, the dot score is `⟨q_t, k̃_s⟩ = ⟨L_j k(A), k(x_s)⟩ + ⟨k(A), k(x_{s−1})⟩`.
- The second term is the A-against-A match. `W_q` can represent `L_j W_key` exactly, because it is a linear map.
- A standard attention stack needs two layers for this: a previous-token head, then an induction head (Olsson et al., "In-context Learning and Induction Heads", arXiv:2209.11895). The probe in #1701 shows layer 0 doing it alone.

**(iv) Cross-talk is controlled but not zero.** For *different* tokens, `⟨k(a), j·k(b)⟩` does not vanish. It vanishes exactly when the key subspace V satisfies `jV ⊥ V` (V totally real for the complex structure `L_j`).
- Example: keys supported on span{1, i} in each lane. Then `j(c + d i) = c j − d k` lies in span{j, k}.
- In that case `(a, b) ↦ a + j b` with a, b ∈ C is exactly the left-handed **Cayley–Dickson ordered-pair embedding** H = C ⊕ Cj. "Canonical token lineage" therefore has a precise meaning: an injective, *ordered* pair encoding, unlike the commutative semiprime.
- `k_a + j k_b ≠ k_b + j k_a`, which fixes the defect `mathematics.md:9` names ("Commutative prime products … collapse distinctions").
- With full-lane keys it is a superposition with cross-talk, not an injective pair.
- **Read-only diagnostic (hypothesis):** on the F2 checkpoint, measure the ratio E⟨k_a, j k_b⟩² / E⟨k_a, k_b⟩² over distinct token pairs, and the share of key energy in a jV ⊥ V split. This predicts whether capacity is the next limit.

**(v) Interpretations, checked.**
- *Discrete connection/holonomy:* formally a constant connection `U_{t,t−1} = L_j` on the path graph. A path has no loops, so there is no curvature. "Holonomy" adds nothing here beyond `j^m` having period 4.
- *Order-1 Markov blanket:* yes. The key becomes a function of (x_t, x_{t−1}), a bigram identity. It is the soft, learned, ordered counterpart of the already-defined `e_t = p_{t−1}p_t` (`formal_vocabulary.md:90`).
- *Induction shortcut:* yes. It is a fixed-tap instance of known techniques:
  - H3's shift SSM on keys (Fu et al., "Hungry Hungry Hippos", arXiv:2212.14052);
  - RWKV token shift (arXiv:2305.13048);
  - Primer's depthwise convolution on Q/K/V (So et al., arXiv:2109.08668);
  - the short convolutions Zoology finds necessary for MQAR (arXiv:2312.04927).
- The new part is the choice of the second tap: an orthogonal, skew, order-4 signed permutation, which also costs nothing under D11.

**(vi) D11 compatibility.** Shift, signed permutation and add are D11-exact. The blocker is engineering: the integer and served paths refuse the shift, and #1704 estimates about 200 lines for the session snapshot and rollback paths.

**(vii) Equivariance.** `L_j` commutes with every *right* multiplication: `(k + j k′)r = kr + j(k′r)`. So any right-acting gauge or position transform keeps the lineage channel intact.
- #1701 proposes a *left*-acting quaternion rotary, `q_t ↦ ρ^t q_t`. That breaks it, because `ρ^t j ≠ j ρ^t` unless ρ ∈ span{1, j}.
- Any positional or transport rotation added later should act on the right.

### 2.4 A prediction to test before generalizing

MQAR places the key *immediately* before the value. Natural chat does not: in "Alex's friend is Sam", the token before "Sam" is "is", which every fact shares. Multi-piece names such as "lukgostjal" add another layer.
- **Hypothesis:** order-1 lineage alone will not transfer to the D19 cell. The `ngram:4` design (#1589) already assumed a reach of four tokens.
- **Cheapest decisive test:** a gap-MQAR variant (key, g filler or relation tokens, value; g ∈ {0, 1, 2, 3}), with multi-piece keys, on the existing `examples/mqar-bench.rs`.
- **Arms:**
  - shift off;
  - order-1 shift;
  - the multi-lag forms in §3.
- **Decision it changes:** whether to train the chat model with `key_shift=true` as is, or with a multi-lag lineage.

## 3. Generalizations: what algebra allows and limits

1. **Higher-order lineage with mutually orthogonal lag operators.** For zero self-coherence, the lag-m operator must be `L_{g_m}` with `Re(g_a · conj(g_b)) = 0` for every pair of lags.
   - A 4-dimensional lane holds **at most 4** mutually orthogonal unit quaternions. For example `k_t + i k_{t−1} + j k_{t−2} + k k_{t−3}` (any fixed orthonormal frame works).
   - This is a hard bound (**proven**). Lags beyond 3 per lane need more lanes, per-head lag assignment, or accepted coherence (Welch bound).
   - Measurable as a bench arm.
2. **2I elements as lag operators.** Self-coherence of `L_g` is `⟨x, gx⟩ / |x|² = Re(g)`.
   - Zero self-coherence requires Re g = 0, which in 2I is exactly the **conjugacy class of size 30**: the order-4, pure-imaginary unit icosians.
   - Orders 3, 5, 6 and 10 have Re g = ±1/2, ±φ/2 or ±1/(2φ). For example, an order-10 element gives lag-1 coherence cos 36° ≈ 0.809.
   - So **j is already the optimal kind of 2I element** for one step. A "2I lineage" with powers g^m of an order-10 element would alias. **Proven** algebra; the predicted harm is a hypothesis.
3. **Decayed recurrent lineage `K_t = k_t + λ g K_{t−1}`.** With g = j, `j² = −1` puts lag 2 back on the current channel with sign −λ² (aliasing). A D11 form needs λ as a power of 2.
   - This is a linear recurrence on keys, i.e. what the `r` layers already learn. That is consistent with seed-dependent success without the shift (#1704).
   - Value is lower than items 1 and 4.
4. **Transported-key read: data-dependent holonomy, the strongest geometric proposal (hypothesis).**
   - Let `P_t = u_1 ⋯ u_t` be the r-layer's path-ordered transport. With the 2I snap, each `P_t` is an *exact* 2I element: a 7-bit index, composed by the 14,400-entry table.
   - The relative transport telescopes: `P_{t←s} = P_t P_s^{−1}`. Right multiplication is isometric, so `⟨q_t P_t^{−1}, k_s P_s^{−1}⟩` scores every pair at O(1) extra cost per position.
   - Applied on the right, it commutes with `L_j` lineage (§2.3 vii).
   - Icosian coefficients lie in {0, ±1/2, ±1/(2φ), ±φ/2, ±1}. Fixed-point action by them is where **Z[φ] becomes load-bearing** (`φ(a + bφ) = b + (a+b)φ`, `formal_vocabulary.md:95`), done with adds and shifts.
   - External support: PaTH attention (Yang et al., arXiv:2505.16381) uses accumulated *data-dependent* Householder products in q/k and solves NC¹-complete state tracking beyond TC⁰.
   - Theoretical motivation (**proven**): 2I/{±1} ≅ A5, and Barrington's theorem makes width-5 permutation programs over A5 NC¹-complete. Merrill et al. (arXiv:2404.08819) show diagonal SSMs stay in TC⁰. This gives an expressivity reason for 2I, not a learnability or language result.
   - The project already measured exact A5 tracking by 2I lanes (B1 Stage A). This read would expose that state to attention.
   - **Owner conflict:** it belongs to the multiplicative position-encoding family, of which RoPE is the static special case. The owner's "no RoPE" rule needs an explicit ruling.
5. **Zeta-phase lineage or positions.** Fixed rotations with ordinates γ_j are incommensurate. The property that matters is low mutual coherence, and random or log-uniform frequencies supply the same thing.
   - **No supporting theorem.** Testable only as an arm against matched random frequencies.
6. **Zeckendorf / Fibonacci multi-scale lineage.** Zeckendorf decomposition (**proven**: every distance is a unique sum of non-consecutive Fibonacci numbers) gives an O(log_φ n) hierarchical partition of the prefix.
   - It is a Fibonacci analogue of the Fenwick (power-of-2) partition in Log-Linear Attention (Guo et al., arXiv:2506.04761).
   - Testable as hierarchical summary buckets. The required control is the base-2 Fenwick partition; no advantage of φ is known.

## 4. The owner's ideas, sorted

| Idea | Mathematical status | Concrete testable form |
|---|---|---|
| **Canonical token lineage** | **Supported and measured** (MQAR scope only). Exactly the Cayley–Dickson ordered pair when keys are totally real (§2.3 iv). | (1) The cross-talk probe. (2) Gap-MQAR with multi-lag `{1, i, j, k}`. (3) D19 cell and chat with `key_shift=true`/`add`. (4) D11 integer port. |
| **Poincaré/S3 observer at O(1)** | **Not supported** as fixed-state exact recall. Any fixed-size state fails copying beyond capacity (Jelassi et al., arXiv:2402.01032: error > 1 − \|S\|/D^L). Zoology (arXiv:2312.04927) gives the recall–state tradeoff. | **Supported** form: O(1) *work per query* over O(n) exact memory. Index the ordered (p_{t−1}, p_t) pair → positions for exact admission (D11 table), then rank by the soft lineage score. This reconciles #1589's finding (exact admission alone is too permissive, learned ranking is needed) with #1701 (lineage belongs in the score). |
| **Phases growing with φ** | Narrowly supported. φ is the worst-approximable irrational (Hurwitz). Golden-angle increments `t/φ mod 1` minimize the maximal gap, by the three-gap theorem (Sós 1958). This is a real optimality for *uniform position coverage by one frequency*; no language-modelling theorem exists. | Golden-angle right-acting position phase per lane, against log-uniform frequencies and against no position code. Subject to the "no RoPE" ruling. |
| **Fibonacci boundaries** | Zeckendorf partition: proven combinatorics. | Hierarchical Zeckendorf buckets against base-2 Fenwick at matched state (§3.6). |
| **Primes as triangulation / content addresses** | Content addresses: **yes**, as an injective set or multiset encoding. Triangulation: only the Boolean face lattice (combinatorial nerve), with **no metric**. AGENTS.md already states "a prime/hash identity is not a semantic distance". | Keep primes for exact admission and identity (store, n-let index). Put semantic proximity in learned keys. Do not expect gcd structure to supply similarity. |
| **Zeta phases as semantic basis** | Assumption-scoped coordinates (ADR-0003). There is no theorem of predictive advantage, and the production stack does not use them. | Only as a matched-control arm (§3.5). |
| **Hopf / E8 / harmonics** | Defined, never measured (catalogue §6.1, 8.7). | The only concrete entry point I see for 2I/Z[φ] is §3.4. Hopf has no identified role in the read path today. |

## 5. Conclusions

1. **Proven:** the key shift is an isometric, skew, order-4 signed permutation that gives an exactly orthogonal self-copy. With totally-real keys it is the Cayley–Dickson ordered pair, and it is D11-exact.
2. **Measured, on MQAR at context 512 and width 128 only:** it turns in-context recall from 0.255 to 1.000 at every distance, and removes seed dependence in `rrarra`. Its effect on chat, D19 or natural-language facts is **unmeasured**.
3. **Hypotheses, ranked:**
   1. Natural facts separate key and value by relation tokens, so order-1 lineage needs multi-lag `{1, i, j, k}` lineage (bounded at 4 per lane).
   2. Exact ordered-pair or n-let admission plus lineage ranking gives a D11 O(1)-per-query read.
   3. A right-acting, 2I-exact, telescoping transported-key read is the coherent place where 2I, Z[φ] and S3 transport become load-bearing in attention, with Barrington/PaTH-type expressivity.
4. **Not supported:** O(1)-state exact recall; semantic distance from prime arithmetic; any special advantage of zeta or φ frequencies without a matched control; 2I powers of order ≠ 4 as lag operators.

**Files and sources cited:**
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/geometric_stack.rs` (1–127, 595–765, 1347, 2125–2160, 8115, 8913)
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/stack_prime_route.rs`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/formal_vocabulary.md` (90–131)
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/architecture-2026-09/mathematics.md`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/research/retired-mechanisms-catalogue-2026-10-01.md` (§4.1, 4.5, 4.6, 5.4, 6.1, 8)
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/research/external-literature-reconnaissance-2026-10.md`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/adr/0003-fixed-zeta-prime-route-attention.md`
- PRs #1494, #1506, #1587, #1589, #1698, #1701, #1704; issue #1512.