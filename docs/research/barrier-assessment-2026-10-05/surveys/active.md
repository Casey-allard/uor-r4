# Active-mechanism inventory: UOR-R4 production path and the Codex native attention line

Scope: repo at `71b54adf` (origin/main, 2026-10-04). Code was read only. Nothing was run. Evidence labels used below: **[proof]** means exact by construction in code, **[measured]** means a recorded run within its stated scope, **[hypothesis]** means not tested.

---

## 0. What "production" is today

The production model is the 96M ladder rung. Its runbook settings are in `docs/compute/ladder-runbook.md:147-168`:
- `arch=geometric read=l2 rotation=true`
- `width=1024 heads=16 layers=14 context=384`, 95,957,184 parameters
- pattern `rrarrarrarrarr` (the 14-layer pattern quoted in #1704)

Chat fine-tunes add `pointer=32 protocol=2 policy=full_prefix` (`ladder-runbook.md:197,212`).

The runbook invocations set neither `transport_snap`, `qat` nor `key_shift`.

Chat is served by `GroundedSession` (float `StackModel`). The pieces around the model are the relation compiler, `StackStore` and the log sieve. STATUS.md describes the "current grounded generation" as a floating development path. The D11 integer engine serves exported plain and pointer artifacts in `lut-chat engine=d11` (#1669, #1676).

---

## 1. `geometric_stack.rs`, the trained network

### 1.1 StackModel / StackConfig (`geometric_stack.rs:889-924`, shapes `1086-1162`)

Each layer is pre-norm and residual. The layer loop is at `5767-5797`:

```
x ← x + mixer(norm(x));  x ← x + SwiGLU(norm(x))
```

- `mixer` is `recurrence` for `'r'` and `geometric_read_with_source` for `'a'`.
- The embedding, final RMSNorm and LM head are ordinary.
- The SwiGLU MLP (`mlp.gate/up/down`) is a dense linear map. It holds most of the parameters.
- `StackConfig` has these fields: `pattern`, `read` (Lorentz/Dot/L2), `rotation`, `rotation_group` (Quaternion or the U1 control), optional `memory` (product-key memory with H4/E8 codebook, `stack_memory.rs:20-55`), `select` (flock) and `pointer`.

### 1.2 `r`: quaternion transport recurrence (`recurrence` `5331-5370`; `RecurrenceCore::window` ≈`9047-9118`; output `9371`)

The width is split into `L = d/4` lanes of four channels. Steps per layer:

1. `[a ‖ g] = W_in · RMSNorm(x)` with `W_in ∈ R^{2d×d}`.
2. Gates: `G = W_gate u + b`, giving `L` decay logits plus `4L` raw rotation coordinates.
3. Width-4 causal depthwise conv: `c_t = b_conv + Σ_{s=0..3} w_s ⊙ a_{t−s}`.
4. Decay: `σ_t = sigmoid(G_lane)`, `log a = −softplus(−decay)`, `λ_t = exp(8 σ_t log a)`. Decay timescales are initialised over 2…1000 (`1405-1411`).
5. Transport: `u_t = raw/|raw|`, a unit quaternion in S³. With U1, components `j,k` are zeroed. With a snap, `u_t` is replaced by the nearest of the 120 icosians.
6. Recurrence: `h_t = (λ_t u_t) ⊗ h_{t−1} + √(1−λ_t²) · c_t`, where ⊗ is the Hamilton product (`quaternion_product` `8635`). The floor `1−λ² < 1e−6` gives `keep = 1e−3`.
7. Output: `W_out (h_t ⊙ GELU(g_t))`.

**Geometry actually used.** The transition is a unit quaternion acting by left multiplication per lane, i.e. a block-diagonal SO(4) left-isoclinic rotation scaled by λ.
- This is the only S³ primitive on the default forward path.
- Structurally it is a Griffin/RG-LRU-style gated linear recurrence (√(1−λ²) input scaling, conv-4, GELU output gate; De et al., arXiv:2402.19427) with a non-commuting quaternion transition in place of a real or complex diagonal (cf. LRU, Orvieto et al., arXiv:2303.06349).
- Everything else in the layer is ordinary linear algebra.

**Status.** Trained on CPU/Metal/CUDA (`cuda_stack_kernels.rs`, `metal_stack_kernels.rs`) and served in D11 (`uor-r4-integer/src/stack/session.rs:1702 stack_recurrence`; Hamilton product `kernels.rs:852`).

Measured control: the U1/abelian and `rotation=false` controls exist. #1701 found rotation has no effect on all-read stacks, which is trivially true because `r` layers are absent there.

### 1.3 `a`: read mixer (`geometric_read_with_source` `1819-1897`; scoring `transform` `10045-10075`; `fused_read` `10842`)

Inputs and projections:
- `u = RMSNorm(x)`
- `q = W_q u`, `k = W_k u`, `v = W_v u`, split into heads of width `d/H`.

Score of source `j ≤ t` for head h:
- **Dot:** `q·k/√d_h + A[h, t−j]`
- **Lorentz:** `−β(arcosh(1+e) − o) + A`, where `e = lift(q)lift(k) − q·k − 1` and `lift(x) = √(1+|x|²)` (`lorentz_distance` `10181`)
- **L2, the production setting:** `−β(|q−k| − o) + A`, with `|q−k|² = |q|²+|k|²−2q·k` clamped at 1e−7 (`10066-10072`). `β = exp(log_beta)` and offset `o` are learned per head.

The other components:
- **Age bias `A ∈ R^{H×context}`:** learned per head and per distance. It is ALiBi-initialised: `A[h,δ] = −2^{−8(h+1)/H}·δ` (`1413-1419`).
- **NoRead slot:** logit `w_null,h·u + b_h` with zero value. It joins the softmax denominator, so a head can abstain.
- **Output:** `W_o · merge_heads(Σ_j softmax_j · v_j)`.
- **Optional `select`:** flock keeps the sink, the last `window` positions and the top-`k` sources by the same total score. Unkept sources get weight 0 (doc `55-69`).

**Geometry actually used.** With `read=l2`, none of the primary-geometry primitives is involved. It is Euclidean-distance softmax attention with ALiBi-style learned position bias and a null slot.
- Lorentz, the hyperboloid model of H^n (not R4/S3/H4), was superseded by L2 in the 8M comparisons ("flat L2 wins the three measured splits", STATUS.md).
- Calling this mechanism "geometric attention" means distance-scored softmax, not quaternion or H4 structure.

**Status.** Trained, served in D11 (`session.rs:1898 stack_read`; flat L2 via digit-table squares and digit-by-digit square root, `kernels.rs:841`; #1667) and tested by the oracle parity tests.

### 1.4 `FusedRead`

`fused_read` and `fused_read_selected` (`10842`, `10865`) are a custom candle op with an exact hand-written backward. It covers queries/keys/values, NoRead, age, β/offset and optional RoPE (RoPE is used only by the retired transformer control). The packed `aux` layout is checked by `fused_aux_len` (`9631`).

It is an engineering kernel, not a mechanism. CUDA and Metal ports exist; on Metal the L2 read runs through host copies (#1698).

### 1.5 `read_key_shift` (#1701 `3aa8ba16`, saved by #1704 `2cdae05a`)

- Code: `set_read_key_shift` `2142`; applied in `project("key")` `1856-1858`; `quaternion_j_left` `8913`; `previous_key_channel` `8936`.
- Rule: `k'_t = k_t + j ⊗ k_{t−1}`, with `k_{−1} = 0`.
- Left multiplication by `j` is the signed permutation `(a,b,c,d) → (−c, d, a, −b)` on each 4-lane.
- It has no parameters and no matrix product. Queries, values, NoRead and age are unchanged.

**Exact facts [proof].**
- `j` is a unit pure-imaginary quaternion, so left-multiplication by it is orthogonal and skew-symmetric. Hence `⟨x, jx⟩ = 0` for every lane vector x, `j² = −1` and the order is 4 (tested, #1701).
- A query can match "position whose predecessor carried key κ" through `⟨q_t, j k_{t−1}⟩`, a channel orthogonal per vector to `k_{t−1}` itself.
- At the value position `p+1` of a pair written as (key at p, value at p+1), the shifted key holds `j·k(key token)`. A single head with `W_q ≈ j W_k` therefore reads the value directly. This is the one-layer induction route: the previous-token head plus induction head of Olsson et al. (arXiv:2209.11895), collapsed into one layer as in token-shift designs (RWKV, arXiv:2305.13048; H3's shift SSM, arXiv:2212.14052; MQAR setting from Zoology, arXiv:2312.04927).

**Measured on the MQAR bench** (1.36M params, ctx 512, 1,800 steps, CPU):

| Arm | d16 | d64 | d200 | d400 | held-out class | Source |
|---|---|---|---|---|---|---|
| `aaaaaa` l2, no shift | 0.81 | 0.18 | 0.01 | 0.03 | 0/1024 | #1698 |
| `aaaaaa` + F2 | 1.000 | 1.000 | 1.000 | 1.000 | 1024/1024 | #1701 |

- F2 reaches ≥0.9 by step 400 (#1701).
- `rrarra` without the shift: seed 1 = 0.007, seed 2 = 0.9995. With the shift it is 1.000 on both seeds (#1704).
- `rararr` solves with or without the shift (1022/1024 vs 1024/1024 held-out).
- Probe: one layer-0 head puts 0.99–1.00 weight on the value position (#1701).
- Interpretation [measured]: an `r` layer's width-4 conv can supply predecessor identity, but training finds that route only on some seeds. The shift supplies it exactly.

**Not measured.**
- The D19 grounded MQAR cell with a shifted chat model (pod A/B `key_shift=add` pending, #820 comment 2026-10-05T00:36Z).
- Any matched control isolating the quaternion `j`: identity shift `k_t + k_{t−1}`, a random fixed orthogonal map, or concat/learned `W_prev`.
- **[hypothesis]** Because `W_q` is learned, any fixed orthogonal map that separates the two channels may work equally well. The claim that `j`/S³ specifically is necessary is unestablished.
- No QAT form and no integer export. `set_served_representation`, `export_stack` and `stack_grid_reference` refuse it (`stack_export.rs:346,991,1726`). #1704 estimates about 200 lines for D11 (snapshot/rollback paths).

**Prior art in this repo.** `read_identity_carry` (#1580 `a74f3686`, 2026-10-01; `read_identity_input` `1972`, `set_read_identity_carry` `1997`) has q/k at t project input t−1 while v/NoRead/age stay at t. It reported 192/192 on a fresh 2/8/16-fact draw.
- It replaces rather than adds the current channel.
- It was never adopted into ladder training.
- The predecessor-identity lesson was therefore found three days before F2 and not propagated.

### 1.6 Pointer head and copy mixture (doc `70-104`; `pointer_side` `5817`; `pointer_scores` `11871`; `pointer_weights`)

- Projections: `[q ‖ k ‖ g] = h_t [W_q; W_k; w_g]ᵀ + [0; 0; b_g]` on the final normed state, with `dim = 32` and `b_g = −2` initially.
- Score: Dot `q_t·k_j/√32`, or Lorentz `−β arcosh(1+e)`.
- `a_t = softmax` over sources (optional `PointerSelect`).
- Copy distribution: `p_copy(v) = Σ_j a_tj [x_j = v]`.
- Output distribution: `p(v) = (1−σ(g_t)) softmax(z_t)_v + σ(g_t) p_copy(v)`. The loss is the log-mixture.

**Geometry.** None by default (Dot). An optional `PrimeRoute` (`636-845`) is off in production; see §4.

**Status.** Trained (CUDA mixture loss #1675) and served in Q30 in both integer engines (`kernels.rs:1028-1060 stack_pointer_mixture`; `session.rs:6-24`; #1669). QAT refuses a pointer. Selected or routed pointers have no integer port.

### 1.7 QAT / `D11Interim` (`7668-7679`; `served_plan` `7815`; `StraightThrough` `7915`)

- `D11Interim` round-trips every map through 4-bit values in groups of 32 with a one-byte scale `(16+m)2^{e−4}` (`quantize_matrix`/`dequantize_matrix`, `lut_export`).
- Taps, biases and age are converted to fixed-point grid codes (`Fixed(-16)`, `DecayRate`, `LorentzScale`).
- Gradients pass straight through.

It is ordinary QAT. The ladder runbook does not use `qat`.

### 1.8 `TransportSnap::Icosian` (`7260-7390`; integer `kernels.rs:880-1016`)

- `u_t` snaps to the nearest of the 120 unit icosians of 2I (`canonical_h4_roots` from `uor-r4-core::native_geometric::learner::embedding`), with straight-through gradients.
- The integer kernel selects the root exactly in Z[φ] arithmetic. Coordinates are `(a+bφ)/2`, and the sign test of `da + db·φ` is exact (`stack_snap_select` `966`).
- This is the only place on the stack path where H4/2I and Z[φ] are genuinely exercised.
- It is not used in the ladder runs [measured absence from runbook].

### 1.9 `data_parallel` (`average_replica_gradients` `8425`; `geometric-stack.rs:721-1482`)

Two CUDA replicas with gradients averaged. It requires an even batch and no qat or snap. It is infrastructure (#1660), not a mechanism.

---

## 2. `relation_compiler.rs`: closed relation/act compiler

The closed label set is `relation_names()` (`milestone_world_v2.rs:876`), the M-world v2 relation table plus `none`, with acts `{assert, update, query, none}` (`relation_compiler.rs:33-35`).

**Heads:**
- **`RelationRoute`** (`603-805`): a sparse softmax over a bag-of-words lexicon, optionally with standardised frozen-trunk features appended (`route_row`). It is fitted by full-batch f64 gradient descent.
- **`SpanHead`** (`864-1022`): decodes a value span of at most 4 words.
- **`RelationMode::OpModel`**: a fine-tuned stack generates `Op: assert user_name Ada`. `parse_op` (`1237-1288`) maps that to `CompiledAction::{Assert, Correct, QueryCurrent, Unresolved}`. It locates the generated value inside the source text and aligns it to whole words (#1678). An assert whose value is not in the text becomes a query.
- **`OpPolicy`** `{Op, Table, TableStatements, UnlessQuery}` (`1071-1099`) combines the op model with the table. `UnlessQuery` is used in the L12 session reports (#820, 2026-10-04T20:10Z).

**Geometry.** None. These are logistic-regression heads and a text generator.

**Status.** Trained offline and float-served in the session; not D11. It is a closed-schema parser.
- [measured, STATUS] The E4 route reaches 36/52 open relations, 2/17 closed and 106/109 MQAR.
- Values outside the fixed vocabulary are dropped.

---

## 3. `stack_grounded_session.rs` + `stack_store.rs`: exact memory

**Session flow.** Each user turn goes through:
1. compiler → `CompiledAction`
2. `MemoryEffect` `{Unresolved, Write{written, value_tokens}, WriteDisabled, Read{StoreRead}}` (`163-180`)
3. recall line
4. float stack generation

**Recall format.** One system turn, `Memory: <value>.` or `Memory: none.` (absence only when proven). It is inserted between the user turn and the assistant turn (`≈717-770`).
- A conflict or history read is `Unsupported` in this trained format.
- `RecallDisposition` takes the values `{NotRequested, Value, LogValue, Absent, Disabled, Unsupported}`.
- Store and history commit only after generation succeeds.
- `ContextPolicy` is `{StrictFullHistory, WholeCompletedTurns}`.

**Store.** `StackStore` (`stack_store.rs:336`) wraps `uor_r4_core::native_geometric::learner::scoped_memory::Memory`. It is an exact versioned chain with views Current/PreviousAssertion/PreviousDistinctValue/Initial, plus Absent/Evicted/NoHistory. This is the one substantive `native_geometric` component on the production path.

**Log sieve.** `LogRecall` (`329`) is installed by `with_log_recall`. It fires only when the compiler is `Unresolved`. Its rule is `sieve_value_text` (`milestone_world_v2.rs:2205-2244`):
- atoms = query words outside a fixed `reserved_words` list
- admit clauses of earlier user turns that share ≥1 atom ("gcd>1", implemented as set intersection)
- rank by (shared count, latest turn, latest clause)
- value = the words after the first matched atom, minus a leading copula

**Geometry.** No primes are computed; the "gcd" is a set intersection. It is an authored rule, not learned. The rule is itself a text-level induction copy: find the earlier occurrence of the cue and copy what follows it. That is exactly the circuit F2 makes learnable in-weights.

**[measured]**
- Sieve on: 105/109 MQAR. Sieve off: 3/109 (96M L12 sessions, #820 2026-10-04T20:10Z).
- 29M D19 session: 972/1075 (STATUS). The brief's "1008" was not located in the files I read.
- Panel: 43–46/232 is per brief; STATUS records 28/232 for 20M.
- Caveat: M-world was about 2% of fine-tune tokens, so 3/109 does not separate inability from lack of training (#820).

---

## 4. Primes, zeta, Z[φ], UOR identity on the production path

| Primitive | Where | On the default production path? |
|---|---|---|
| S³ unit quaternion (Hamilton) | `r` transport | **Yes** (trained and served) |
| Quaternion `j` | read key shift | Option; off in the trained ladder; no D11 |
| H4 / 2I icosians, Z[φ] | transport snap, `stack_memory` codebook | Options; not in runbook |
| Lorentz/hyperbolic | read/pointer score | Superseded by L2 |
| Primes | `token_prime` (`717`), `PrimeRoute` exact/ranked/n-gram pointer admission (`636-845`, ADR-0003) | Pointer option, off (`pointer=32` Dot) |
| Zeta phases | none in `geometric_stack.rs` (the only hit is the ADR filename) or `uor-r4-integer/src/stack` | **No** |
| UOR identity | `sha256_hex` artifact binding | Provenance only |

**Conclusion [proof by inspection].** The production network's distinctive geometry is per-lane quaternion transport. Attention, MLP, embedding, pointer and compiler are Euclidean, linear-algebraic or lexical. Prime/zeta/H4 are present as options or in the separate Codex line (§6), not in the trained ladder.

---

## 5. `uor-r4-integer/src/stack`: D11 serving, and `uor-r4-lut`

**`uor-r4-integer/src/stack`** (`mod.rs` doc; `kernels.rs`, `session.rs`, `format.rs`) executes the `UORLUT01` stack artifact with no float and no multiplier instruction:
- 4-bit maps via per-activation nibble tables and byte-pair tables (`stack_gemv_pairs` `675`)
- group scales by shift/add
- runtime×runtime products via radix-16 multiples tables (`mul_i64` `229`)
- exact long division (`stack_div_u64` `280`) and digit square root (`stack_isqrt` `328`)
- exp/sigmoid/GELU/arcosh by tables (`354-441`)
- RMSNorm in integers (`476`)
- L2 distance (`841`), Hamilton product (`852`), Z[φ] snap (`966`), Q30 pointer mixture (`1035`)

Equality with the D10 engine is an empirical oracle criterion (`stack_d11_oracle`), not a proof. #1691 adds a worker pool (3.6× at 4 threads, identical tokens). The flock port (`flock.rs`) is standalone and "awaiting integration". Key shift is not ported.

**`uor-r4-lut`** (`lib.rs` doc) is the D10 integer engine for converted Llama checkpoints, plus a `stack` engine. AGENTS.md classifies it as a frozen non-mission comparator. `lut-chat engine=d10` remains available as a side-by-side reference (`geometric-stack.rs:154-189`). Its `GROUP=32` constant is reused by training.

---

## 6. `uor-r4-core::native_geometric` and the Codex native geometric attention line

**`native_geometric`** has 136 files (7.6 MB). The production stack imports only three things from it:
- `scoped_memory` (the store)
- `canonical_h4_roots` (snap)
- `realtext_support::sha256_hex`

The rest is the earlier `joint_model`-era native model and research machinery. It is separate from the stack path.

**The Codex line** is a different model: an integer, table-driven attention over a bounded selected-record source, not the stack. Its components:

- **`geometric_context.rs`.** Up to 2 heads × 4 lanes, each holding a signed H4 root (one of 120 codes). Step: `action = table[token] ⊕ table[own_old] ⊕ table[neighbor_old]` (Q24 scores, argmax over 120 roots), then `state ← compose(state, action)` via the artifact-bound H4 multiplication table (`h4_tables.rs`) (`step` `523-559`). A separate non-injective readout emits address lanes (root plus radius/presence category, 0 = absence). It is a finite-group automaton over 2I.
- **`geometric_potential.rs`.** Per-lane Q24 tables score query lane vs key lane: presence, directed unaries, radial and cross terms, with order and antipodes kept distinct. Only table reads and adds.
- **`geometric_read.rs`.** Age plus NoRead plus a Q31 exp-table softmax, with exact i128 payload mixing and restoring division.
- **`geometric_no_read.rs`, `_value_q4`, `_age_q4`.** 4-bit learned scalars.
- **`geometric_occurrence_read.rs`.** Policy `selected-original-BPE-occurrences;frame-query-prefix-causal-H4;context-relative-potential;content-absent;zero-age;NoRead-parent-fallback;128-admission`. `MAX_SEQUENCE=128`, `MAX_HEADS=2`. Candidates are only the original source occurrences.
- **`geometric_source_realizer.rs`.** Turns occurrence weights into Copy/Period/Stop actions with one global alias reduction (probability summed over identical tokens). #1697 adds the chronological occurrence bank (multi-source, causal candidate→position map).
- **`geometric_cue_carrier.rs`** (#1705 `71b54adf`). Policy includes `Source-immediate-predecessor-Context-only; inverse(query)*cue; signed120-angular-q4`. Each admitted Copy occurrence gets `+T_q4[h,lane][ rel(q, cue) ]` with `rel = q⁻¹·cue` in 2I and `cue` = the context state at the occurrence's **immediate predecessor**. The `CueUnary` control indexes by `cue` alone.

  This is an exact-H4 instance of the same predecessor-lineage idea as F2, independently arrived at [proof by code]. Measured: dev CE 1.146→1.040 (directed) vs 1.124 (unary); banks 2/64 vs 1/64; fresh complete 0/32 vs 1/32.

- **Training drivers.**
  - `geometric-native-route-{credit,fit,crossing,accepted,fit-panel}`: discrete updates of packed H4 action coefficients. #1695: Joint64 CE 0.739 and Role64 0.810, both worse than the parent's 0.492, so the parent was retained.
  - `geometric-bank-fit` (#1700): bank CE 2.163→1.736 while single-source rows regress.
  - The older `attention-geometric-*` examples fit and q4-export each component (context, potential, value, NoRead, event-age, span, composition, session).

**Status [measured].** Native integer, CPU, ≤128 tokens, authored M-world-style panels. Fresh-panel generation is 0–1/32 throughout the last two weeks. There is no stack integration and no general text.

---

## 7. Adversarial synthesis (hypotheses flagged)

1. **The read is not geometric.** The 3-month "geometric attention" bottleneck on the production path was a missing predecessor-identity channel in an L2 softmax read [measured, #1698/#1701/#1704]. #1580 had already shown the same thing on Oct 1.
2. **Both labs converged on predecessor lineage.** Claude (`k_t + j k_{t−1}`) and Codex (`q⁻¹·cue_{pred}` in 2I) arrived at it independently. **[hypothesis]** A served form can make it genuinely geometric: an exact integer signed permutation in D11, or an exact 2I product in the native line.
3. **The `j`-specific claim is untested.** The value of `j` over any fixed orthogonal shift lacks a matched control. Required arms: identity shift, random SO(4) per lane, learned `W_prev`.
4. **Deciding experiments still pending:**
   - D19 sieve-off MQAR with `key_shift=add` (pod A/B).
   - Whether F2 lets the network replace the authored log sieve.
   - A D11 port of the shift.

Until these run, chat-level effects of F2 are **UNAVAILABLE**, not negative.