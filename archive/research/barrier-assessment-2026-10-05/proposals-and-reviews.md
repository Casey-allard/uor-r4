# Expert proposals and adversarial reviews

Five experts each proposed one solution to the barrier. Three adversarial reviewers checked each proposal through a distinct lens. A proposal survived if at least 2 of its 3 reviewers failed to refute it. **None survived.** The final assessment uses only the parts the reviewers marked salvageable.

## geometer: Clocked Icosian Lineage (CIL) read: F2 generalized to an exact, multi-lag, self-incoherent lineage operator with a learned salience clock

**Outcome:** refuted

### Mechanism

Scope: this changes only the key, and optionally the query, at each geometric read `a` of the production stack (`crates/uor-r4-training/src/geometric_stack.rs`). Values, NoRead, the age bias, the pointer head and the r layers are unchanged. Every exact claim below is labelled P (proved or exact by construction). Every expected outcome is labelled H (hypothesis).

**0. Why F2 works (P).** For a unit quaternion g and any lane vector x, <g x, x> = Re(g x x̄) = Re(g)|x|^2.
- j is pure imaginary, so <j k, k> = 0 for every k.
- A query tuned to "my predecessor carried key K", which is q ≈ j k_K, therefore gives exactly zero response at the position holding K itself. It responds only at K's successor.
- The identity shift k_t + k_{t-1} lacks this property: the query fires equally on K and on K's successor. The working property is lag self-incoherence, not "j" as such.

**1. Iterating F2 is wrong (P).** The tempting generalization k_t + j k_{t-1} + j^2 k_{t-2} + ... has j^2 = -1. Lag 2 is then exactly anti-coherent with lag 0: <j^2 x, x> = -|x|^2.
- More generally, the self-coherence of the lag pair (a, b) under left codes c_a, c_b is Re(c_a c̄_b)|x|^2 = <c_a, c_b>_{R^4} |x|^2.
- So one S^3 lane admits at most 4 exactly self-incoherent lags, namely {1, i, j, k} up to rotation.
- Multi-lag lineage must therefore be spread across lanes.

**2. Lineage operator with icosian Fourier lag codes (P).**
- Split each read head of 64 dims = 16 quaternion lanes into a block of 10 lanes and a block of 6.
- Let ρ ∈ 2I be an order-10 unit icosian, e.g. ρ = (φ/2) + (1/2) i + (1/(2φ)) j. Let ω = (1+i+j+k)/2 be order 6.
- Lane l of the 10-block uses R_l = L(ρ^l); lane l of the 6-block uses R_l = L(ω^l). Write R for the lane-diagonal operator.
- Lane-normalize keys, k̂_t^l = k_t^l / |k_t^l|, so keys lie on (S^3)^16.
- Define the lineage key over a window of n = 6 clock steps: K_t = Σ_{ℓ=0}^{5} R^{ℓ} k̂_{t-ℓ}.
- Self-coherence of lag difference d, summed over the head, is Σ_{l<10} cos(π l d/5) + Σ_{l<6} cos(π l d/3). This is exactly 0 for d = 1..5 (checked numerically; 6 at d=6 and 10 at d=10). So every pair of lags inside the window is exactly self-incoherent.
- The codes commute, because each lane uses powers of a single element. K therefore has an O(1) rolling update: K_t = k̂_t + R K_{t-1} - R^6 k̂_{t-6}. Its cost does not depend on n. It is an exact Rabin–Karp rolling hash in the group algebra R[2I], restricted to cyclic subgroups.
- Queries carry an offset δ: Q_t = R^δ q̂_t, or optionally a query-side lineage Σ_ℓ R^{ℓ+δ} q̂_{t-ℓ}.
- δ = 1 is F2's successor addressing: the score <Q_t, K_s> picks the term <q̂_t, k̂_{s-1}>. δ = m reads the m-th successor, so multi-piece values chain like an induction copy.
- In the identity-match regime (q^l ∝ k^l, which MQAR and induction need), repeated tokens at different lags contribute exactly 0. Distinct tokens contribute ordinary bounded crosstalk.

**3. Position as a discrete connection: the salience clock.**
- Each read head h gets one learned scalar gate s_t = σ(<w_h, u_t> + b_h). The bias is initialised high, so the model starts as plain multi-lag lineage.
- Transport advances only on salient tokens. The clock is c_t = Σ_{τ≤t} s_τ, and the transport between positions τ ≤ t is the holonomy R^{c_t - c_τ}.
- Training uses the exp-map rotation by angle (c_t - c_τ)·θ_l, which is float and offline. At integer clock values this is exactly the 2I element.
- Training sums over a raw causal window of W = 32 positions, masked by the clock window: K_t = Σ_{τ∈(t-W,t], c_t-c_τ<6} s_τ R^{c_t - c_τ} k̂_τ.
- Serving thresholds s to {0,1}. On a salient token: K_t = k̂_t + R K_{t-1} - R^6 k̂_evict. On a non-salient token: K_t = K_{t-1}. The last 6 salient keys are kept in a queue.
- This is a flat, data-dependent, finite-group connection on the token path. Copulas and fillers can become transparent, so "friend is named V" and "friend V" give V the same lineage.

**4. Two code families per read layer.**
- Ordered heads use the Fourier codes above (n-let alignment).
- Bag heads use a single code R^1 for every lag 1..5: K^bag_t = k̂_t + R Σ_{ℓ=1}^{5} k̂_{t-ℓ}.
- A bag head's score counts the salient query atoms shared with the predecessor window. That is, by construction, the authored log sieve's ranking (`milestone_world_v2.rs:2205-2244`): set intersection over non-reserved atoms, with the latest clause winning through the existing age bias, and the value copied as the successor while skipping the copula, here through non-salient copulas.
- So the sieve becomes a representable special case of a learned in-network read: s = non-reserved indicator, bag code, δ = 1.

**5. Exactness for D11 (P).**
- The coordinates of ρ^l and ω^l lie in ½{0, ±1, ±φ, ±φ^{-1}}.
- Store key lanes in Z[φ] as integer pairs (a,b) = a + bφ. Then ×φ is (a,b)→(b, a+b), ×φ^{-1} is (a,b)→(b-a, a), and ½ is a shift after a ×2 prescale.
- The icosian ring (the Z[φ]-span of 2I, ≅ E8 as a Z-module) is closed under left multiplication by 2I. The rolling recursion is therefore exact integer arithmetic with no drift: no float and no multiplier.

### Why it solves the barrier

The synthesis diagnosis, measured on the bench, is that retrieval failed on Carry (key content), not on score geometry.
- F2 fixed Carry at lag 1: 0.255 → 1.000 and 0/1024 → 1024/1024 held-out (#1701), seed-robust on `rrarra` (#1704).
- But real facts do not put the key at lag 1. `milestone_world_v2.rs:749-752` phrasings ("My friend is named {v}", "My best friend is {v}", "My friend's name is {v}") put the relation word 2–4 tokens before the value, behind copulas shared by every fact. Keys are also multi-piece under BPE.
- Note: the MQAR query phrasings were already revised so that none ends in "{k} is" (`milestone_world_v2.rs:53-55`). The lag problem is on the assertion side.
- At lag 1 F2 tags the value position with j·k("named"/"is"), the A1/recency signature. The same holds for the r-layer conv route, whose width-4 conv is learned and seed-dependent.

CIL attacks the four sub-causes directly.
- **(a) Coverage.** It supplies exact Carry up to 5 salient predecessors, with zero self-interference between lags (P), so the key word is in the value's key at any gap ≤ 5.
- **(b) Gaps.** The salience clock makes gap tokens transparent, so the lag of the key word no longer depends on phrasing.
- **(c) Phrasing.** Bag heads give phrasing-order-invariant matching, the in-network analogue of the 105/109 sieve. Sieve-off is 3/109, so this targets exactly the gap between the two.
- **(d) Length generalization.** No term depends on absolute position or on t - s; the match is between token identities inside lineage windows. Retrieval at d = 400 or beyond is limited only by the existing age bias and softmax sharpness, as with F2 (H). It is not RoPE.
- **(e) Composition.** Offsets compose in the abelian code group (R^a R^b = R^{a+b}). A query offset δ addresses "the m-th token after the match", which covers multi-piece value copying. A second read layer keyed on first-layer outputs inherits exact relative offsets.

It does not address G3 (copy tokenization), G4 (closed compiler labels) or G5 (knowledge ceiling). The first experiment is designed so that a CIL win cannot be confused with those.

### Uses primary geometry

Each primitive below has a stated, load-bearing role, and each is checked by a matched control.
- **R4/S³ lanes.** Keys are lane-normalized onto (S³)^16. Equal lane norms are what make the summed self-coherence exactly Σ_l Re(c_l)·const, so the cancellation in §2 needs S³ lanes. It is not decoration.
- **Binary icosahedral group 2I / H4.** The lag codes are the cyclic subgroups ⟨ρ⟩ of order 10 and ⟨ω⟩ of order 6 inside 2I. 2I is the only finite subgroup of S³ containing both order-5/10 and order-3/6 elements, and 10 + 6 = 16 matches the production head width exactly. This is what gives an exact 6-lag window with O(1) rolling update. With Q8/Hurwitz units only (orders 1, 2, 4, 3, 6 in 2T), the best rolling block is 4- or 6-periodic: fewer exact lags, or non-rolling Hadamard codes. So 2I is used for a specific number-theoretic reason, and the Hadamard ⊗ Q8 arm tests whether that matters.
- **Z[φ].** cos(π/5) = φ/2. The order-10 codes are only exact in Z[φ], and the integer serving form stores key lanes as Z[φ] pairs with add-only ×φ.
- **E8 = H4 ⊕ φH4.** The icosian ring, the Z[φ]-span of 2I, is the ring in which K lives. Its closure under 2I action is why the rolling update has no drift. In the optional snapped serving form (each lane snapped to one of 120 icosians with the existing `stack_snap_select`, `uor-r4-integer/src/stack/kernels.rs:877`), the score Re(Q̄_l K_l) needs no runtime×runtime products at all.
- **Discrete connection and holonomy.** The salience clock is a data-dependent flat connection with values in a cyclic subgroup of 2I. Transport between positions is the holonomy R^{c_t - c_τ}.
- **Prime n-lets / UOR identity (ADR-0003, #958).** An ordered lineage window is a linear, differentiable embedding of the ordered n-let. In the identity limit (orthonormal per-token keys) <Q, K> equals the number of aligned n-let matches. It is the learnable counterpart of the adjacent-semiprime address, and a candidate for the never-run stage-4 language-to-address compiler.
- **Not used:** zeta phases and Hopf maps, which have no role here, so none is claimed.

### D11 serving story

**Per read head per token (P, by construction).**
- (1) Key projection, unchanged: GPTQ ≤4-bit W_key through the existing D11 kernels.
- (2) Lane normalization: the existing digit square root plus a reciprocal table (`uor-r4-integer`, #1691 audit). Alternatively, in the snapped variant, nearest-of-120 icosian selection via `stack_snap_select`.
- (3) Salience: one 4-bit dot product plus a threshold compare, giving s ∈ {0,1}.
- (4) Rolling update if s = 1: K ← k̂ + R·K − R^6·k̂_evict.
  - Each lane action L(ρ^l) or L(ω^l) on a Z[φ]^4 lane is a fixed signed permutation of coordinates with coefficients in ½{1, φ, φ^{-1}}.
  - That is about 16 Z[φ] add/negate/shift operations per lane, ×16 lanes, ≈ 500 integer adds. W_key alone needs ~65k nibble-table reads at width 1024.
  - The 6-entry eviction queue is a ring buffer and is part of session snapshot/rollback.
- (5) Score: <Q, K> with Z[φ] components.
  - Unsnapped: the existing radix-16 multiple-table dot path, run on two integer components and recombined with the existing Q32 φ constant (`uor-r4-integer/src/stack/mod.rs:85`) via shift-add.
  - Snapped queries (Q_l ∈ 2I): Re(Q̄_l K_l) is pure add/shift on the icosian-ring element, with zero runtime products.
- (6) Softmax, NoRead, age: unchanged table paths.

**Properties.** No float and no multiplier. Bit-exactness follows because the group algebra over Z[φ] is closed and finite-window. No transformer component.

**Costs.**
- The key cache doubles if K is stored as Z[φ] pairs. Recomputing K from the raw-key cache plus the queue avoids that, at about +500 adds per position per head.
- Port size: about 400 lines in `uor-r4-integer/src/stack/{session,kernels,format}.rs`, plus a `stack_export.rs` field. That is larger than F2's estimated ~200 (#1704) because of the clock and the queue.
- Required acceptance: a float-training vs integer-session parity test of K at integer clock values on a saved checkpoint, then GPTQ/QAT fidelity on near-one-hot reads (synthesis Q11).
- Laptop throughput impact (H): under 3% on 96M at 32.5 ids/s.

**Sparse access (D5).** Under F2-like sharp reads (0.99–1.00 mass on one position, #1701 probe), CIL keys are near-one-hot. That makes the first fair top-k / flock contest possible on the same checkpoint.

### Prior attempts and why different

**Repo history.**
- **F2, `read_key_shift` (#1701 3aa8ba16, #1704 2cdae05a; `geometric_stack.rs:2125-2160, 8913-8944`).** Lag 1 only, fixed j, unnormalized keys, no gap handling. CIL keeps F2 as its n=1, s≡1 special case. It adds the proven reason F2 works (pure-imaginary self-incoherence), the proof that iterating j fails at lag 2 (j² = −1), exact 6-lag codes, the clock and bag heads.
- **#1580 (Codex, "Preserve identity-to-payload alignment").** Query and key used the previous normalized state (lag-1 carry), 192/192 on its panel. Its stated next step was "variable-gap typed identity capture/hold/commit/reset". #1705 then built a separate learned signed-H4 cue carrier over ≤128-token supplied records, with fresh 0–1/32. CIL is the variable-gap step done parameter-free inside the production stack read: the only learned addition is one scalar gate per head, and the codes are fixed group elements with a proof. It is not a separate automaton or an authored cursor.
- **ADR-0003 adjacent semiprime / prime n-lets (#958, stage 4 NOT_RUN).** Exact but hashed (prime products: a Boolean face lattice, no gradient, no similarity). CIL is the linear, trainable embedding of the same ordered n-let, with exact n-let counting in the orthonormal limit.
- **Spin/HELM-D cumulative frame (retired, P: a gauge that preserves the dot product).** That applied one frame to both q and k, which cancels. CIL superposes different tokens' keys under distinct codes, which adds predecessor information and is not a gauge.
- **"Direct two-token channel" (`ordinary-lexical-audit-2026-09-23.md:108-126`, +0.469 bits worse).** That put lineage into the current readout. CIL puts it only into keys matched later.
- **#1069 owner residual.** Query-side lineage only, one seed. CIL puts it on the key side, with query offset δ.
- **KVAR Q8 relative energy.** Degenerate because the relative action was identity at the read site. CIL's codes act on predecessors, not on the matched token, so they are non-degenerate.
- **Lorentz / L2 / H4-potential / Hopf-sector score swaps (E1–E5).** CIL leaves the score geometry (L2) alone and changes key content, per lesson 3.
- **Peter–Weyl class kernel (retired readout ceiling).** CIL uses 2I class inner products only inside the key match, in the snapped serving form, not as a readout.

**External work.**
- **H3** (Fu et al., "Hungry Hungry Hippos", arXiv:2212.14052) solved associative recall with a learned shift SSM on keys. F2 is a fixed instance of that idea. CIL differs by exact orthogonal multi-lag codes, a clock and multiplier-free exact serving.
- **n-gram heads** (Akyürek et al., "In-Context Language Learning: Architectures and Algorithms", arXiv:2401.12973) are the function CIL hard-wires.
- **Memory Mosaics** (arXiv:2405.06394) uses time-shifted key/value association. The successor offset δ is the same idea.
- **PaTH attention** (arXiv:2505.16381) uses data-dependent cumulative transforms as position. The clock is a finite, abelian, windowed and exact analogue.
- **RoPE** is not what CIL does. Nothing depends on absolute position or on t − s, so CIL needs, but should pass, the owner's "no RoPE" ruling (owner 09-29).

### First experiment

This runs on the existing `crates/uor-r4-training/examples/mqar-bench.rs`, CPU only, with no pod spend before it decides.

**Bench extension (about 150 lines).**
- (i) `gap=0..3`: insert a draw from a small copula set, shared across all pairs ("is", "named", "called"; tokens disjoint from keys and values), between key and value.
- (ii) `key_pieces=1..3`: multi-token keys; the query repeats all pieces.
- (iii) `reorder=true`: in half the sequences the query presents key pieces in a different order. This is a phrasing proxy.

**Stack additions (about 200 lines, `geometric_stack.rs`).**
- `lineage=none|f2|identity|jpow4|hadamard_q8|random_so4|icosian`, `clock=false|true` and `bag_heads=0|2`.
- Unit tests:
  - exact self-coherence zero for lags 1..5 with the icosian codes, and −1 at lag 2 for jpow4;
  - causality;
  - rolling update equals the direct sum;
  - the clock at s ≡ 1 equals the no-clock path;
  - save/load field.

**Arms** (all at bench settings of #1701/#1704: width 128, 4 heads, so head dim 32 = 8 lanes).
- Use the 4 + 4 lane analogue at this width: lags 1..3 exact with the order-4 blocks. Alternatively use width 160 → head dim 40 = 10 lanes → order-10 block, lags 1..9 exact.
- Prefer width 160 for the icosian arm, with width-matched controls.

| Arm | Lineage |
|---|---|
| A0 | none |
| A1 | F2 |
| A2 | identity shift k + k_{t-1} |
| A3 | j-powers n=4 |
| A4 | Hadamard ⊗ Q8 codes n=6, signed permutations, non-rolling |
| A5 | random fixed SO(4) per lane per lag, n=6 |
| A6 | icosian Fourier n=6 |
| A7 | A6 + clock |
| A8 | A7 + 2 of 4 heads bag |

**Grid.**
- Patterns `aaaaaa` (lineage is then the only Carry source) and `rararr`.
- Conditions: C0 = original bench (regression), and C1 = gap 0..3 uniform, key_pieces 1..3, reorder on.
- 2 seeds.
- That is 9 × 2 × 2 × 2 = 72 runs at about 1,000–1,200 s each ≈ 22 CPU-hours at 4 threads. Run 2 concurrently on the M1 (about 11 h wall) through the existing sealed report roots (`report_output::claim`).

**Report.** Per-distance recall, held-out-class recall, the #1701 read probe (mass on the true value position), and the learned clock's s on copula vs key tokens.

**Gated step 2, only if CIL clears.**
- Add CIL to the 29M chat fine-tune on the pod.
- 3 arms (none / F2 / best CIL) × 2 seeds × MQAR dose 15%.
- Evaluate on the sieve-off D19 MQAR cell (109), with the open 232 panel reported separately.

### Kill criterion

All criteria are frozen before running. Kill at bench stage, condition C1, mean of 2 seeds on `aaaaaa`:

1. **F2 already solves gaps.** If A1 ≥ 0.90 overall on C1, the lag-mismatch hypothesis G2 is falsified. Drop multi-lag lineage and the clock, and port F2 only.
2. **Lineage family insufficient.** If no CIL arm (A6–A8) exceeds 0.60 overall on C1, the lineage family cannot bridge real gaps. Stop CIL and move to learned content-gated binding.
3. **Geometry not load-bearing.** If A6 and A7 are within 0.03 of A4 (Hadamard ⊗ Q8) and of A5 (random SO(4)) on both C0 and C1, the 2I/Z[φ] code choice is not load-bearing. Keep the cheapest code (signed permutations) and retract the icosian claim. The salience clock is still judged separately.
4. **Self-incoherence irrelevant.** If A2 (identity shift) ≥ A1 − 0.03 on C0, self-incoherence is not the operative property. Revise the theory before any port.

Chat stage (step 2):
5. **No chat gain.** If best-CIL sieve-off D19 MQAR does not exceed both the F2 arm and the none arm by ≥ 15/109 at both seeds, do not promote it to the ladder.
6. **Text-NLL guard.** If base dev NLL regresses by more than the observed seed spread (0.063 nats, B1 precedent), do not promote.

A pass on the bench without chat transfer is recorded as a scoped bench result only (lesson 2).

### Expected result if right

All of the following are H, stated so that they can fail.

**Condition C0 (original bench).** A1, A4, A6, A7 and A8 all ≈ 1.000, matching #1701. A2 is clearly below A1, because the identity shift fires on the key itself. A3 is at or slightly below A1. A0 ≈ 0.26 reproduces.

**Condition C1 (gap 0–3, multi-piece, reordered).**

| Arm | Expected overall recall | Reason |
|---|---|---|
| A0 | ≤ 0.15 | no Carry source |
| A1 (F2) | ≈ 0.25–0.45 | succeeds mainly at gap 0, fails at gap ≥ 1: lag-1 tag is the copula |
| A3 | below A6 | lag-2 anti-coherence on repeated copulas |
| A5 | below A6 | random codes: crosstalk from repeated copulas |
| A6 | ≥ 0.80 | |
| A7 (clock) | ≥ 0.95, best ≥ 0.9 point earlier than A6 | the learned s on copulas falls below 0.2 while on key pieces it is above 0.8 (probe) |
| A8 (bag heads) | best on the reorder subset | order-invariant matching |

Held-out pairing class for A7/A8: ≥ 1000/1024. Recall flat across d16 to d400, within 0.02.

**Chat stage.** Sieve-off D19 MQAR moves from 3/109 to ≥ 30/109 at 29M with a 15% dose, against F2's smaller gain. The open panel moves by no more than a few points, consistent with G5, and this is reported as such.

### Cost

Engineering, offline (H):
- About 350 lines of Rust: bench extension, the lineage operator with rolling and direct forms, clock gate parameters, save field and unit tests.
- About 1–1.5 engineering days.

Compute:
- About 22 CPU-hours at 4 threads (72 runs × ~1,100 s, per #1701/#1704 timings), about 11 h wall with 2 concurrent on the M1.
- Peak RAM per run is similar to the #1704 arms (width 128–160, 6 layers): well under 2 GB.
- New storage: sealed report roots under `~/uor-r4-local/mqar-bench/`, under 1 GB total.
- No pod and no external cost until the bench gate passes.

Step 2 (pod), only on a pass: 6 fine-tunes at 29M. Cost per the ladder runbook (`docs/compute/ladder-runbook.md`); the exact pod hours are UNAVAILABLE here and must be projected and charged to the cumulative ledger before launch.

D11 port, only after a chat pass:
- About 400 lines in `uor-r4-integer/src/stack` plus `stack_export.rs`, with parity tests.
- Serving overhead (H): about 500 integer adds per head per token. Key cache either 2× (Z[φ] pairs) or recomputed from the queue.

### Risks

**Representation risks.**
1. **Lane normalization.** Normalizing every key lane onto S³ can cost base-LM NLL, because it removes key magnitude. Mitigation is to apply CIL only on read heads, keep a learned per-head score temperature, and gate promotion on the NLL guard.
2. **Clock collapse.** The clock can collapse to all-salient (reduces to fixed lags) or all-silent (no lineage). Mitigations: initialize the bias high, add a small entropy/usage regularizer, and report the s distribution. Serving thresholding s also creates a train/serve gap, so use straight-through QAT on s.
3. **Unequal lane norms.** The exact self-incoherence holds only for equal lane norms and in the identity-match regime (q^l ∝ k^l). With learned W_q it is a structural bias, not a guarantee of recall.
4. **Synthetic phrasing proxy.** The bench's reorder/gap proxy may still not reflect real phrasing (lesson 2, E5). That is why the chat stage and the D19 sieve-off cell are mandatory, and why bench wins are scoped.

**Interpretation risks.**

5. **Generic codes may suffice.** The Hadamard ⊗ Q8 or random-SO(4) controls may match the icosian codes. This is the most likely negative, and the kill rule then retracts the 2I/Z[φ] claim honestly rather than keeping it as decoration.
6. **Width mismatch.** Production head dim 64 = 10 + 6 lanes is exact. The bench at width 128/4 heads is not, so a width-160 bench arm is needed, with width-matched controls to avoid a confound.

**Serving and governance risks.**

7. **D11 fidelity.** Z[φ] key lanes double the score work unless the snapped variant is used. Snapping lanes to 120 icosians (about 7 bits per 4 dims) may be too coarse for semantic keys. Measure fidelity before choosing.
8. **"No RoPE" ruling.** The owner's 09-29 ruling needs an explicit decision on fixed-group lag codes. The argument that this is not RoPE is that no absolute or relative position enters the score.
9. **Scope.** It does not fix G3 (copy tokenization `add_prefix_space`), G4 (closed 11-label compiler) or G5 (knowledge ceiling). A flat open panel after a D19 MQAR gain is the expected outcome, not a failure of the mechanism.
10. **Single-seed history.** Single-seed reversals happened before (#1698 → #1704), so every verdict above requires 2 seeds and pre-frozen thresholds.

### Red team, generalization lens: REFUTED

CIL does not survive this review. Its arithmetic is correct, but the causal diagnosis, the geometric claim and the barrier claim each fail when checked against the repo and the literature.

1. **The diagnosis is contradicted by the repo's own measurements (M).** CIL rests on the claim that real facts put the key word 2–4 tokens before the value, so lag-1 Carry is not enough. But production already has a learned multi-lag key source.
   - Every production read follows `r` layers with a width-4 causal conv (`geometric_stack.rs:9047-9118`), plus recurrence of unbounded reach. Pattern `rrarrarrarrarr` puts three `r` layers before the first read.
   - PR #1704 measured `rararr` (the 14-layer stand-in) at 1.000 at every distance without the shift. It measured `rrarra` without the shift at 0.9995 on seed 2.
   - The lab's own #820 correction says the shift "was not strictly missing from the production mix". It names the alternative cause of the 3/109 sieve-off figure: too little MQAR in the fine-tune, about 0.8% of tokens. The F2 pod A/B (base + Arm C, `key_shift` off vs `add`) was launched to separate the two and has not reported.
   - For the canonical assertion "My friend is named V", "friend" sits 3 tokens before V (at word level; BPE may add more), within the conv's 4 taps. The 2–4-token gap that CIL is built for is already covered on the production path.
   - So the only arm where CIL's mechanism is the sole Carry source is `aaaaaa`, which is not a production pattern.

2. **The geometric claims are decorative or false (P).**
   - Every lane code is a power of one element. Left multiplication by e^{θu} acts on H ≅ C² as multiplication by e^{iθ}. Each lane's lag code is therefore a complex root-of-unity phase.
   - The exact cancellation in §2 is the ordinary discrete-Fourier identity Σ_l ζ^{ld} = 0. It holds for any 10th and 6th roots of unity in any U(1). It needs no quaternions, 2I, H4 or E8.
   - "2I is the only finite subgroup of S³ containing both order-10 and order-6 elements" is false. The cyclic group C30 = ⟨e^{iπ/15}⟩ contains both, and so do binary dihedral groups containing C30.
   - The H4/E8/icosian-ring roles therefore reduce to Z[φ] being a convenient exact coordinate ring for cos(π/5).

3. **It is RoPE inside a window.** The score is Σ_ℓ ⟨q_t, R^{ℓ−δ} k_{t−ℓ}⟩. A rotation indexed by the relative lag ℓ−δ enters the score, and that is RoPE's mechanism applied to the window offset. "Nothing depends on t−s" is true only of the outer window position. This conflicts with the owner's 09-29 no-RoPE direction, as the proposal's own risk 8 concedes.

4. **It is a known non-geometric mechanism in fixed form.** A learned causal short conv or shift on keys is the standard associative-recall fix:
   - H3, arXiv:2212.14052;
   - Based, arXiv:2402.18668;
   - RWKV token-shift (not checked in this review);
   - Canon layers, Allen-Zhu, "Physics of Language Models: Part 4.1", arXiv:2512.17351;
   - n-gram heads, arXiv:2401.12973.

   CIL is a fixed, non-learned multi-tap code for the same function. Its arm list omits the decisive control, a learned depthwise causal conv on k at matched taps. The Hadamard and random SO(4) controls are both fixed codes, so the experiment cannot tell "geometry is load-bearing" from "any multi-tap key mixing works".

5. **The C1 bench is constructed so that it can pass.** It inserts copulas drawn from a 3-token set that is shared across pairs and disjoint from keys and values. The salience gate can learn that set trivially, and any learned key conv would also solve it. This repeats the constructed-success pattern (lesson 2): E3 rule panels, A4's 1,336 vs 28, and Codex's fresh 0–1/32 (#1705).

6. **It does not touch the actual barrier.** The proposal itself predicts the open 232-request panel stays flat (G5). The sieve already does the exact keyed lookup at 105/109 (`milestone_world_v2.rs:2205-2244`), and it does so by word-set intersection that survives paraphrase. CIL's identity-match regime does not survive paraphrase: query "name" against assertion "named", or "buddy" against "friend". It offers nothing for composition, reasoning or coherent generation. "Offsets compose in an abelian group" is just induction-head offset copying.

7. **Minor (P).**
   - The self-incoherence result needs equal lane norms and q^l ∝ k^l. With learned W_q and W_k it is only a structural bias.
   - Summing 6 keys into a 64-dimensional head adds superposition crosstalk that the proposal does not bound.
   - The O(1) rolling update saves nothing at a 6-term window.
   - Serving needs per-lane sqrt and reciprocal plus Z[φ] pairs. That doubles key work compared with F2's signed-permutation-plus-add.
   - The Z[φ] serving form also needs a power-of-two prescale (2^6 over a 6-step window) that the cost estimate does not cover.

**Killing experiment:** **Step 1, already in flight, costs nothing new.** Read the pending F2 pod A/B on #820: same base and Arm C fine-tune, `key_shift` off vs `add`, D19 MQAR with the sieve off.
- If the off arm already moves well above 3/109 once MQAR dose is raised, or the shift adds no meaningful gain over off, then the lag-coverage diagnosis is wrong at chat scale. CIL's premise falls without running it.

**Step 2, the decisive CPU bench.** Run it on `mqar-bench` at the production-like pattern `rararr` (not `aaaaaa`), 2 seeds. Use a C1 variant whose gaps are drawn from content words that also appear as keys and values, not from a fixed 3-token copula set, and whose query uses paraphrased key words.

| Arm | Content |
|---|---|
| A0 | none |
| A1 | F2 |
| A6 | icosian CIL |
| A7 | A6 + clock |
| L6 | learned depthwise causal conv on k, 6 taps, initialised at identity on lag 0 (H3/Canon-style) |
| Z | same lag codes with generic C30 complex phases instead of 2I elements |

**Kill CIL if any of the following holds:**
- `rararr` A0 is already ≥ 0.9 on C1 (the production conv covers the gaps);
- L6 ≥ A7 − 0.03 (a learned conv equals the geometric code);
- Z equals A6 within 0.03 (the icosian structure is irrelevant, which the Fourier identity predicts by construction);
- every arm collapses under paraphrased queries (identity-match does not transfer to language).

**Salvageable part:** **Worth keeping as small, honestly labelled engineering:**
1. **The F2 observation, stated correctly.** A pure-imaginary predecessor code is self-incoherent, because ⟨g x, x⟩ = Re(g)|x|². The warning that iterating j fails at lag 2 (j² = −1) is also correct.
   - This makes A2 (identity shift k + k_{t−1}) a cheap, useful ablation of #1701, independent of the rest.
2. **Two or three exact signed-permutation lag codes** ({1, i, j, k} up to rotation, per lane). These are a D11-friendly, multiplier-free form of a short key conv if a learned conv is shown to help. They go in as an option, without icosian or E8 claims.
3. **A data-dependent "salience clock" or skip gate.** Test it as a learned gap-transparency mechanism against a learned conv, on real paraphrased D19 assertions rather than fixed copulas.
4. **The bench extensions** (multi-piece keys, gaps, reordering) are useful instrument work, provided gap tokens overlap the content vocabulary and queries are paraphrased.
5. **The near-one-hot read observation under F2** (0.99–1.00 mass, #1701 probe). It makes a fair D5 top-k sparse-access contest possible, independent of CIL.

### Red team, cost-serving lens: REFUTED

This review used the D11 serving and cost lens, at origin/main 71b54adf. Multi-lag lineage survives it; the icosian/Z[phi] form does not.

1. **The exact 6-lag window comes from a dyadic Hurwitz element, not from 2I or Z[phi] (P, checked numerically).**
   - The proposal's head-summed self-coherence is S(d) = sum_{l<10} cos(pi l d/5) + sum_{l<6} cos(pi l d/3).
   - The order-10 block alone is zero for every d = 1..9.
   - The order-6 block of powers of omega = (1+i+j+k)/2 is zero for d = 1..5 and equals 6 at d = 6.
   - So the n = 6 window is set entirely by omega. omega is a Hurwitz unit in 2T, its coefficients are only plus or minus 1/2, and it is exact in plain fixed point with one bit of prescale. The Hurwitz order is closed under multiplication, so the rolling update has no denominator growth.
   - A full 16-lane dyadic design exists: two omega blocks on 12 lanes, plus 4 lanes that carry only the current key. It is exactly self-incoherent for lags 1..5, rolls in O(1), and needs no Z[phi].
   - The order-10 lanes therefore add no exact lags at n = 6. They only fill 4 more lanes, and they force every key, query and cache entry into Z[phi] pairs.
   - The claim that "2I is the only finite subgroup containing order 5/10 and 3/6 elements" is true but does nothing here.
   - Arm A4 ("Hadamard x Q8, non-rolling") is a strawman. The right control is the rolling omega-Fourier code, and by construction it matches A6's exactness at lower serving cost. Kill criterion 3 is wired against the wrong control.

2. **The serving cost is understated, and it grows with context, which is the regime the proposal targets.**
   - Q_t = R^delta q_hat is also in Z[phi] because R has phi coefficients. Each score <Q, K> is then a Z[phi] x Z[phi] dot product: 3 to 4 integer dot products plus the PHI_Q32 recombination (`uor-r4-integer/src/stack/mod.rs:87`).
   - That multiplies the score work over all T cached positions by 3 to 4. Today the cache is a single i32 key per position (`session.rs:441, 519`).
   - At context 384 this is small next to the 96M weight reads, so "<3%" can hold there. At the claimed d400+ and long-session regime, attention scoring dominates, and a 3 to 4x score multiplier is not "about 500 adds".
   - The alternative, "recompute K from the raw-key cache plus the queue", costs O(T) per query, not +500 per position. At context 384 that is about 384 x 500 x 16 heads x 4 read layers, roughly 12M ops per token, about 12% of the weight work, and it grows linearly with context.
   - The +500-add figure for the rolling step is itself low by about 2 to 4x. L(rho) on Z[phi]^4 lanes, plus the R^6 eviction, is about 1 to 2k shift-adds per head. That one is not decisive.

3. **The "snapped" zero-product variant gives up the key content the mechanism needs.**
   - It quantizes each 4-dim key lane to 1 of 120 icosians, about 7 bits per lane.
   - That is the same pattern as lesson 3 (keys stripped of content) and the Hopf-sector MRR collapse to 0.0045 (#305).

4. **Exactness holds only for the quantized inputs, and the train/serve mismatch is built in.**
   - Served k_hat goes through a 4-bit GPTQ key projection and a digit-sqrt/reciprocal normalization, so lane norms are only approximately equal and the cancellation is approximate.
   - Six quantized keys are superposed, which adds about sqrt(6) quantization crosstalk on near-one-hot reads. Synthesis Q11 (GPTQ fidelity on near-one-hot reads) is unmeasured even for F2.
   - Training rotates by fractional clock angles (float exp-map, not group elements); serving thresholds s to {0, 1}. The proposed parity test "at integer clock values" checks only the trivial case.
   - Training masks to a 32-token raw window; the serving queue has no raw-distance bound. They diverge whenever more than 32 non-salient tokens intervene.

5. **The D11 port rests on a prerequisite that does not exist yet.**
   - F2 itself has no QAT, no export and no D11 form. Every served path refuses it (`stack_export.rs:346, 991`; test at `:1726`).
   - CIL's D11 story assumes a port path for lineage keys that has never been built or measured.

**Minor:** the cited line numbers are stale. `stack_snap_select` is at `kernels.rs:966`, not `:877`, and PHI_Q32 is at `mod.rs:87`, not `:85`.

**Verdict:** the multiplier-free claim is technically achievable, but the icosian/Z[phi] serving story buys no exact lags over a cheaper dyadic code, and its laptop cost is understated at the long contexts it targets. As proposed, the 2I/Z[phi] form is refuted. Multi-lag self-incoherent lineage is not.

**Killing experiment:** **Main kill test (CPU only, before any port):**
- Add arm A4' to the proposed bench (`examples/mqar-bench.rs`): rolling omega-Fourier lineage, with 12 lanes of omega-blocks plus current-key-only lanes, at width 160 / head dim 40. Fill head dim 40 with 6-lane omega blocks plus current-key-only lanes.
- A4' is width-matched against A6 (icosian, n = 6) and A7, and evaluated on C0 and C1 with 2 seeds.
- Kill rule, frozen in advance: if A4' (and A4' + clock) is within 0.03 of A6/A7 on both conditions, retract the Z[phi]/2I serving form and port only the dyadic code.
- Expected outcome: A4' matches A6/A7, because S(d) for lags 1..5 is identically zero in both.

**Laptop-cost test, at the same time:**
- In the integer engine, microbenchmark ids/s on the 96M model at contexts 384, 2k and 8k for: baseline; dyadic lineage keys (i32 cache); Z[phi]-pair cache with Z[phi] x Z[phi] scores; and the recompute-from-queue variant.
- The icosian serving story is killed if its cost exceeds the dyadic arm by more than 3% at any context of 2k or above. The recompute variant is killed by the same rule.

**Fidelity test:**
- Export a lineage checkpoint through GPTQ 4-bit with fixed-point normalization.
- Measure read-probe mass on the true value position and MQAR recall against float.
- A loss of more than 0.05 kills the serving claim until QAT exists.

**Salvageable part:** Several parts are worth keeping:

- **The diagnosis (P).** F2 works because of lag self-incoherence (Re(j) = 0), not because of j itself. Iterating j fails at lag 2 because j^2 = -1. Both points are correct and useful. Arm A2 (identity shift) is the right test of the first point.
- **Multi-lag lineage.** Keep it with orthogonal Fourier codes in a cyclic subgroup, so the update rolls in O(1). Use the dyadic Hurwitz/2T code omega = (1+i+j+k)/2 in 6-lane blocks, which is exact in plain i32 fixed point with a one-bit shift and leaves the key cache and score path unchanged.
- **Where Z[phi] could still matter.** Reserve the order-10 icosian block for the case where a window longer than 6 is shown to be needed. The order-10 block alone gives exact lags 1..9, which a 2T code cannot do in 10 lanes. Gate it on a measured need, as the AGENTS.md rule on primary geometry requires.
- **The salience clock and bag heads.** Both are worth testing as hypotheses, and both are code-independent.
- **The bench extension.** Gap, multi-piece keys and reorder test a known weakness of F2.
- **Order of work.** First give F2 or dyadic lineage a QAT/export path. The current refusal in `stack_export.rs` blocks every serving claim.

### Red team, process-evidence lens: REFUTED

I verified the proposal against the repo at 71b54adf, against PR #1704's body and against the latest #820 comments. It does not survive the lens: it is mostly an open-ended experiment and its first run cannot decide anything.

1. **Its core claim works against itself (P).** The exact cancellation in §2 is real. Σ_{l<10} cos(πld/5) + Σ_{l<6} cos(πld/3) = 0 for d = 1..5, and a lag pair's self-coherence is Re(c_a c̄_b)|x|². But that property makes the lags mutually orthogonal. A query Q = R^δ q̂ then fires only when the key sits at exactly lag δ (gap δ−1), and gives zero at every other gap. The mechanism sold as solving variable gaps (sub-cause a/b) is the one that cannot bridge them. Gap invariance has to come from one of two other places:
   - The learned salience clock, A7. That is a generic data-dependent selective shift (H3 arXiv:2212.14052, Mamba-style gating), not the icosian structure.
   - Bag heads, A8. These use one code R^1 for every lag, so they drop the self-incoherence that is the claimed P content. They also score copula and value successors alike.
   So the 2I/Z[φ] code carries no load for the stated barrier. The δ offset is not a real parameter either: R^δ is a fixed lane-wise orthogonal map, which a learned W_q can absorb, as it can absorb any Σ c_δ R^δ.

2. **It rests on a diagnosis the repo already weakens.**
   - In #1704 the 14-layer stand-in `rararr` scores 1.000 at every distance without any key shift, with read mass 0.98–1.00. `rrarra` without the shift solved on seed 2 (0.9995), and the seed-1 failure (0.007) did not reproduce.
   - The production r layers already carry a learned width-4 causal conv, i.e. multi-lag Carry over lags 0..3 (`geometric_stack.rs`; #1704 interpretation). The gaps CIL targets, "My friend is named {v}" with gap 2–3 at `milestone_world_v2.rs:749-752`, are already inside that conv's reach.
   - The latest #820 comment states that the sieve-off 3/109 may reflect too little MQAR in the fine-tune (about 0.8% of tokens) rather than missing capability. A pending pod A/B tests exactly that: key_shift off vs `add` on the same base, scored on D19 MQAR with the sieve off.
   - CIL pre-empts that decisive, cheaper result with a 72-run, ~22 CPU-hour grid built on an unconfirmed "Carry at lag>1" hypothesis.

3. **The first experiment cannot decide the question.**
   - Condition C1 is a synthetic copula/reorder proxy. Lesson 2 (constructed success did not transfer: E3, A4 1,336 vs 28, reader series, Codex fresh 0/32) says a bench pass decides nothing about chat. The proposal concedes this.
   - The kill rules leave many ways to pass. Any of A6, A7 or A8 above 0.60 keeps the line alive. Criterion 3 lets the icosian claim be retracted while the clock is "judged separately", so some arm almost always survives into a pod stage.
   - Nine arms × 2 patterns × 2 conditions × 2 seeds is the arm-proliferation loop that D9 and the progress-control rules forbid without a decision the result can change.
   - The decisive control is missing: a **learned** short causal conv on the read keys (the H3/Based arXiv:2402.18668 key-conv). It strictly contains CIL's fixed taps. A5 (random fixed SO(4)) and A4 (Hadamard⊗Q8) are fixed codes, so they cannot show that fixed group codes beat simply learning the taps.
   - The lane-normalisation change, keys forced onto (S³)^16, is untested on base NLL. It alters the production read itself, so a bench gain could not be cleanly attributed.

4. **It repeats earlier work.**
   - The variable-gap identity carry was the stated next step of #1580. #1705 attempted it with a learned signed-H4 cue carrier and got fresh 0–1/32.
   - A fixed-code lineage on keys is VSA permute-and-add, a dormant entry in the synthesis ledger.
   - The novel parts, exact orthogonal multi-lag codes and a rolling Z[φ] hash, solve a serving-exactness problem that is not the barrier.

**Scope of the evidence:**
- **P:** the coherence algebra and the j² = −1 observation.
- **M:** F2 = 1.000 at lag 1 on the bench at 1–2 seeds (#1701, #1704). `rararr` with no shift = 1.000 at 1 seed.
- **H:** everything about gaps, clock, bag heads, and chat transfer.

**Killing experiment:** These steps use only work already in flight or under one CPU-hour each, and the order is fixed in advance.

**(1) The pending pod A/B (#820).** Same base, Arm C fine-tune, key_shift off vs `add`, scored on D19 MQAR with the sieve off (n = 109).
- If the shift-off arm rises well above 3/109 after the dose fix alone, or the two arms land within the seed spread, then missing Carry is not the chat bottleneck. CIL's premise falls before any bench work.

**(2) If a Carry gap is still plausible, run one cheap bench condition before any CIL code.** Use mqar-bench with a fixed copula gap g ∈ {0,1,2,3} between key and value, on pattern `rararr`, 2 seeds, with 4 arms:
- no shift;
- F2;
- a learned width-6 causal conv on read keys (H3/Based key-conv, ~20 lines);
- the icosian fixed-code key conv (A6, no clock, no bag).

Two outcomes kill CIL:
- `rararr` without the shift already reaches 0.95 or more at every g ≤ 3, which is plausible because its r-layer conv covers lags 0..3 (#1704). Then the lag problem does not exist in the production pattern.
- The learned key conv is at least A6 − 0.03 on every g. Then the 2I/Z[φ] codes carry no load and only a generic short conv is justified.

A useful prediction, P from §2 of the proposal: A6 with a single query offset should recall at only one gap per head. If it reaches high recall across all gaps, that comes from learned query mixing, which a learned key conv also provides.

**Salvageable part:** Four parts are worth keeping:

1. **The algebraic explanation of why F2 works (P).** <g x, x> = Re(g)|x|², so a pure-imaginary left code makes a lag self-incoherent: the query fires on K's successor and never on K itself. The proof that iterating j fails at lag 2 (j² = −1, anti-coherent) belongs in the F2 design note and its D11 port. It fixes in advance that a future multi-lag extension must not use j-powers.

2. **The cheap test of that explanation.** Run arm A2, the identity shift k_t + k_{t−1}, against F2 on the existing bench condition C0: 2 runs on aaaaaa, about 35 min total. This directly tests whether self-incoherence is the operative property, and it changes the F2 serving design.

3. **The fixed-copula-gap condition for mqar-bench.** It is a cheap instrument and should be added only after the pod A/B. A learned short key-conv is the matched control.

4. **The exact Z[φ] / signed-permutation serving argument.** It is useful for the F2 D11 port (#1704 estimates about 200 lines): a lane action by a 2I element on Z[φ]⁴ is add/negate/shift only, with no drift. The 10+6 icosian Fourier code is a valid construction to keep on file. It should be revisited only if a learned multi-lag key conv is shown to help and then needs an exact multiplier-free serving form.

## number-theorist: Two-hop Lineage-Exact Admission (LEA): learned cue selection, then an exact word-identity jump to successor positions, then learned geometric ranking. F2 and the log sieve are both special cases.

**Outcome:** refuted

### Mechanism

Source facts were checked at 71b54adf. Labels: P = proof or exact by construction; M = measured; H = hypothesis.

**0. What the deployment task actually requires (M, from source)**

- The bench (`crates/uor-r4-training/examples/mqar-bench.rs:15-24`) uses a single-token key at p, the value at p+1, and the query is the key token repeated. That is exactly lag-1 induction, so F2 (`geometric_stack.rs:8936-8944`) is the matching tool.
- Deployment is different (`milestone_world_v2.rs`):
  - Facts are stated as "{k} is {v}" inside "Please remember: …" (`:1427-1435`).
  - Keys are 2–3-syllable made-up words, so BPE splits them into several pieces.
  - Since revision 2.1 no query phrasing ends in "{k} is" (`:997-1012`). Queries are "What is {k}?" and similar.
  - Replies include "It's {v}." and "That's {v}." (`:1022`).
- So at the first value token the current suffix ("It's") carries no key information. The model must:
  1. pick the key word out of the question, several tokens back;
  2. jump to an earlier occurrence of that word;
  3. read 1–2 tokens after it, past the copula.
- That is a two-hop indirection. Neither lag-1 F2 nor suffix n-let admission expresses it. The latter is #1589 `ngram:4`; it requires the n tokens before the source to equal the query's last n.

**1. Exact identity layer (P)**

- **Word segmentation.** A word is a maximal run of pieces that does not start with a new Ġ-piece or punctuation, read from the #1017 tokenizer's piece strings.
- **Normalisation.** N: piece-id → normalised-piece-id. This is a fixed table: lowercase, Ġ stripped, trailing possessive removed.
- **Word identity.** w(word) = intern(sequence of N(pieces)). Interning is an exact dictionary from canonical byte string to a dense u32 id. Identity therefore has no false positives and no false negatives.
- **Hash buckets.** The hash table uses simple tabulation hashing (Pătraşcu–Thorup, arXiv:1011.5200): h(x) = T0[x0] ⊕ T1[x1] ⊕ T2[x2] ⊕ T3[x3] over the 4 bytes of the id. That is table reads plus XOR with no multiply. It gives O(1) expected probes and O(log n / log log n) maximum load.
- **UOR label.** The prime p_{id} can be carried as the serialised UOR label, for compatibility with ADR-0003 manifests and StackStore. It is never used for computation (see §5).
- **Inverted index.** I[w] is an append-only list of word-end positions, keeping the most recent K_occ = 8. A per-word eviction counter makes "absent" provable: an empty I[w] with evicted = 0 means the word never occurred. This is the AGENTS.md absence-proof rule.

**2. Two-hop read: an "LEA head"**

Some heads of each `a` layer become LEA heads. The proposal is 4 of 16 at 96M and 1 of 4 on the bench; the other heads stay dense L2 reads.

**Hop 1, learned cue selection.**
- Let E_t be the word-ends of the last M = 32 words strictly before t. This covers the current and previous turn, so anaphora is also covered.
- p(m | t) = softmax over m ∈ E_t ∪ {∅} of ℓ1(q1_t, k_m) + a1[slot(m)].
- ℓ1 is the existing L2 score, q1 and k are learned projections, and a1 is a learned bias per recency slot. ∅ is NoRead.

**Hop 2, exact jump and learned ranking.**
- For cue m with word w_m, the admitted set is C_{t,m} = { e + δ : e ∈ I[w_m], e < start(m), δ ∈ {1,2,3}, e + δ < start(m) }. These are the successor positions of earlier occurrences.
- p(s | m, t) = softmax over s ∈ C_{t,m} ∪ {∅} of ℓ2(q2_t, k'_s) + β_δ + b[Z(t − s)].
- k'_s is the F2 lineage key at three lags: k'_s = k_s + i·k_{s−1} + j·k_{s−2}.
- **Orthogonal lag channels (P).** For pure-imaginary unit quaternions u ⊥ v and any lane x, Re((u x)‾ (v x)) = |x|² Re(ū v) = 0. So the lag channels {1, i, j, k} are mutually orthogonal per lane at identical content.
- **Zeckendorf level.** Z(d) = max{ k : F_k ≤ d }, the index of the leading Zeckendorf term. There are 13 levels up to 384 and 25 up to 10^5. b is a learned table per head.

**Output.**
- r_t = Σ_m p(m|t) Σ_s p(s|m,t) · v_s, added to the residual stream like any read.
- When several cue words point at the same successor (multi-piece or multi-word keys), their probabilities add. This is a learned, soft version of the sieve's "most shared atoms" rule.
- The pointer head can copy from the LEA head's argmax position. Copy is still capped by catalogue 8.3 until protocol 2 is used.

**Special cases (P):**
- M = 1 with the cue = the current token, δ = 1, and identity replaced by learned key similarity gives F2/#1701.
- p(m|t) uniform over non-reserved query words, ranking by shared count then recency, and δ = "skip copula" gives the log sieve `sieve_value_text` (`milestone_world_v2.rs:2205-2244`).
- Restricting the cue to the suffix gives #1589 `ngram-ranked`.

**3. Training**

- Offline float with candle. C_{t,m} is computed exactly in the Rust data loader from token ids and passed in as gather indices, as int32 [B, T, M, Cmax] with Cmax = 24 and a pad mask.
- Gradients reach q1, q2, k, v, a1, β and b. Admission itself has no gradient and does not need one.
- Memory per LEA head is B·T·M·Cmax scores: 8·512·32·24 ≈ 3.1M floats on the bench.

**4. Multiscale and durable extension (phase 2, gated on phase 1)**

- Because I[w] holds positions, LEA can address (k'_s, v_s) entries retained beyond the 384-token window. A "lineage cache" keeps only successor positions of content words.
- StackStore records get the same word ids, so store entries become extra admitted candidates in hop 2. The exact store and the network then share one address space, ranked by one learned score: the sieve becomes one admission source, not an authored answer path.
- Zeckendorf blocks give old-context page summaries with carry-merge rules (F_{k−1} + F_{k−2} = F_k), each amortised O(1). They are offered only against a matched binary/Fenwick control, arXiv:2506.04761.

**5. Rigorous verdicts on the owner's ideas**

- **Prime/n-let content keys: sound as identity, dominated as computation (P).**
  - First-seen prime assignment is a dictionary, so it is injective.
  - gcd(P_q, P_r) > 1 holds iff the two sets intersect: divisibility on square-free numbers is the Boolean lattice.
  - Products are commutative, so ordered n-lets need positional encoding. ADR-0003 §2 says this itself (`docs/adr/0003…:125-131`).
  - With V ≈ 5·10^4 interned words, p_V ≈ V ln V ≈ 2^19.2, so a u64 holds three atoms. Larger sets need bigints, where a bitset or inverted index costs O(1).
  - No metric exists. The same verdict was reached in the #820 review of Muse's prime-gcd index.
  - Decision: use primes as UOR labels and integer ids for compute.
- **"Primes as triangulation points": sound only as CRT multi-hashing (P).**
  - Residues mod coprime p_1 … p_r determine x mod ∏ p_i, which gives independent bucket coordinates for Bloom/count-min style admission.
  - That is a known hashing method, not semantics. With exact interning available, it is unnecessary for admission.
- **Fibonacci-prime phases: not sound as a mechanism.**
  - Fibonacci primes have no addressing role, and it is open whether there are infinitely many.
  - What is sound:
    - **Golden-ratio (Fibonacci) multiplicative hashing** (Knuth, TAOCP vol. 3 §6.4). The constant multiply is a shift-add chain.
    - **The three-distance theorem.** The phases {nφ} split the circle into at most 3 gap lengths, a best-spread positional code.
    - **Zeckendorf levels.** These are exact, carry-structured multiscale buckets.
  - No theorem says ratio φ beats ratio 2. Zeckendorf versus binary is an empirical, matched-control question (H).
- **Zeta phases: no theorem favours zeta ordinates as frequencies.** Use them only as a matched-control arm.
- **"O(1) observer": unsound as O(1) state (P).**
  - Exact recall of N arbitrary pairs needs Ω(N log V) bits of state (Jelassi et al., arXiv:2402.01032; Arora et al., Based, arXiv:2402.18668).
  - The sound reformulation, which LEA implements, is O(1) expected access per cue over an O(n) exact index.
- **"Canonical token lineage" (F2).**
  - The orthogonality of the {1, i, j, k} lags is P.
  - Whether j is special is H and untested. Under learned W_q, any fixed orthogonal map with zero self-coherence may be equivalent.
  - LEA treats lineage as the general primitive "identity of a selected earlier word → its successors". F2 is the lag-1, cue = self case.

### Why it solves the barrier

**The diagnosis supported by the evidence.** For three months score geometry was swapped while key content and indirection stayed fixed. The specific deployment failure (G2) is that the cue and the value are separated by a copula and a question template, and keys are several BPE pieces.

Learned similarity has to do three things at once over hundreds of tokens:
- select the key word out of the question;
- match it, across piece splits and Ġ variants, to an earlier occurrence;
- step past "is".

Each is a lock-in lottery (#1704: seed 1 0.007 vs seed 2 0.9995), and they compound.

LEA factorises this. Only hop 1 (local, ≤32 candidates, short range) and the final ranking are learned. The long-range jump is exact:
- **No false negatives (P).** Admission is exact identity, so long-range recall cannot decay with distance; that decay is the A1/#1698 signature (d200 0.01).
- **Small candidate sets (H, to measure).** The ranking problem shrinks from about 2,855 positions per query token to about 10–30, so the read can become near one-hot. #1701's probe shows near one-hot reads are what recall needs.
- **Multi-piece keys** are handled because identity is word-level, normalised across Ġ and case. #1589 used token-piece primes, and its diagnosis named piece-level matching ("any shared common piece admits") as a failure cause.
- **The copula** is handled by successor offsets δ ∈ {1,2,3} with learned bias, together with the 3-lag orthogonal lineage key.
- **"It's {v}" replies and anaphora** ("What is it?") are handled because hop 1 can select a cue anywhere in the last 32 words, including the previous turn. This also targets the 60 anaphoric compiler rows (G4) inside the network.

Two further effects:
- **D5 sparse access.** Fair sparse access was blocked because dense reads were diffuse. LEA reads are sparse by construction, so cost scales with occurrences, not context length.
- **Joint durable memory.** The same index lets the exact StackStore and in-network memory share addresses. That is ADR-0003 stage 4's missing "language-to-address compiler" (`docs/prime_route_attention_qualification_958.md`, stage 4 NOT_RUN): the compiler is learned hop 1 plus exact interning.

**What this does NOT solve.**
- The open-panel plateau, if it is knowledge-absent (G5).
- Copy credit under protocol 1 (G3; use protocol 2, #1591).

I state this up front so a recall gain is not over-read as a capability gain.

### Uses primary geometry

**Genuinely load-bearing:**
1. **R4/S³ quaternion lineage keys.** The lags are left multiplications by the pure-imaginary units: k'_s = k_s + i·k_{s−1} + j·k_{s−2}. These are exact signed permutations. The proof that {1, i, j, k}-turned copies of the same lane are mutually orthogonal (Re(x̄ ū v x) = |x|² Re(ū v) = 0 for u ⊥ v) is what lets one key carry three lags without cross-talk at identical content. This extends F2 (`quaternion_j_left`, `geometric_stack.rs:8913-8933`) rather than replacing it.
2. **Exact Z[φ]/Zeckendorf structure.**
   - Age is bucketed by leading Zeckendorf index, computed by comparisons against a 25-entry F_k table.
   - Phase 2 page merges follow Zeckendorf carry rules exactly.
   - This uses φ-arithmetic as exact multiscale structure (as P4 recommends), not as a semantic metric.
   - Its advantage over binary is H and gets a matched control.
3. **UOR content identity.** Word identity is the canonical normalised piece string, interned. Prime labels p_{id} are serialised for ADR-0003/#958 manifest and StackStore compatibility, so the "ordered n-let" record becomes (cue id, δ, successor position), with the order explicit as ADR-0003 §2 requires.
4. **The existing L2 read, NoRead and age machinery** are reused for both hops.

**Honestly not used, and why:**
- **Prime arithmetic (gcd/products)** is dominated by integer ids (§5 proof).
- **Zeta phases** have no theorem behind them; at most a matched-control age-bias arm.
- **Hopf/E8/2I** play no role in admission. 2I snap could later quantise k' (the #1506 D11 port exists), but that is not claimed here.

**Missing pieces, named:** the Zeckendorf versus binary advantage, and j versus a random orthogonal map, are both untested.

### D11 serving story

**Every served operation is a table read, XOR, compare, shift or integer add, or an existing D11 kernel:**
1. **Segmentation and normalisation.** Two u32 tables over the vocabulary: is-word-start and normalised-piece. Table reads only.
2. **Interning.** An open-addressing hash table keyed by the normalised piece sequence. The bucket is simple tabulation (4 tables × 256 × u32, XOR). Equality is a byte compare. Ids are a counter. No multiply.
3. **Inverted index.** A per-word ring of K_occ = 8 u32 positions, with an evicted counter. Append is O(1); a query is O(K_occ). Session rollback (#1704 named snapshot/rollback as the cost) truncates every structure to a length watermark, which is O(1) per structure using an append log of touched words.
4. **Lineage key.** k'_s = k_s + i·k_{s−1} + j·k_{s−2} on integer keys: signed permutations plus integer adds. This is the #1704 design note extended to 3 lags; it is computed once per position and cached.
5. **Hop 1.** The existing integer L2 read kernel (`kernels.rs:841`) over ≤32 word-end positions, plus a table bias.
6. **Hop 2.** The same kernel over |C_t| ≤ 32·8·3 = 768 worst case, about 10–30 typical. β_δ and b[Z(d)] are integer tables. Z(d) is a binary search over 25 Fibonacci constants.
7. **Mixing.** p(m)·p(s|m) is a product of two runtime values. It is served through the existing radix-16 multiple tables, or more cheaply by adding log-domain integer scores and taking one softmax over the union through the existing table exp. The union form is exact for argmax and is the one trained with QAT.

**Cost.** Hop 2 touches O(occurrences) positions instead of O(t), so LEA heads cut scored positions per query token. This is measured on the bench through the `positions scored / query token` field that already exists. Memory is about 16 B per word occurrence plus the cached k'/v at successor positions.

**Interim.** Dense L2 heads remain a labelled interim under D5.

**GPTQ/QAT.** The near-one-hot hop-2 reads must survive 4-bit keys. This is checked in Experiment 3 (risk list).

### Prior attempts and why different

1. **#1587/#1589 prime-route pointer.**
   - Arms: R1 `prime:1` shared-atom, R2 `prime-ranked:1`, R3 `ngram:4` / `ngram-ranked:4`. Results are in #1512 comments.
   - Exact routes lost (MQAR 2–3 vs soft 16). Ranked routes tied the soft pointer (R3b 15 vs 16; McNemar p ≈ 1).
   - The diagnosis there: "too narrow (identical preceding token)" and "too permissive (any shared common piece)".
   - **What differs here:**
     - (a) Identity is at word level and normalised. Those runs used token-piece primes.
     - (b) The cue is chosen anywhere in the last 32 words by learned hop 1. Those runs used the query suffix, which cannot answer "It's {v}" or "What is {k}?" because M-world 2.1 removed suffix-answerable phrasings (`milestone_world_v2.rs:997-999`).
     - (c) The read is in the residual `a` layer with lineage keys and successor offsets. Those runs were pointer-only, ranked by an untuned gate.
     - (d) Those runs were confounded by copy 0/33 under protocol 1 and by a 1.5M-class model with low MQAR dose. Here the decision runs on a bench without copy confounds before any chat run.
2. **#1701/#1704 F2.** F2 is LEA's degenerate case (M = 1, cue = self, δ = 1, identity by learned similarity). It was validated only on the gap-1, single-token-key bench. LEA adds lags 2–3, the cue hop and exact identity, which target G2.
3. **Log sieve** (`milestone_world_v2.rs:2205-2244`; 105/109 sieve-on).
   - It is an authored rule: a reserved-word list, ranking by shared count then recency, copula skipping, and output as text outside the network.
   - LEA learns the cue choice, ranking, offset and abstention inside the network. Phase 2 makes the sieve an admission source, not an answer path.
4. **ADR-0003/#958.** The substrate passed stages 1–3, but stage 4 (prompt-to-address compiler) was NOT_RUN. LEA's hop 1 plus interning is that compiler, learned.
5. **D2 AERM store** (store 1.000, learned read failed) and **A1/D18** (every arm below 0.5 at d16). Both relied on learned similarity for the long jump. LEA removes that dependence.
6. **Score-geometry swaps** (Lorentz, H4 potentials, Hopf sectors, 2I codes). LEA leaves the score at L2 and changes key content and indirection. Lesson 3 of the synthesis names that axis as the defensible one.
7. **Muse prime-gcd index** (#820 review). That is word-level exact admission with no learned ranking and no in-network read, and it is equivalent to the sieve. LEA keeps its ranking idea as a learned prior only.
8. **Codex geometric bank/cue carrier** (#1705, 0–1/32 fresh). A supplied-record automaton with no stack integration. LEA lives inside the production stack's read, which addresses process gap P1.

Nothing in this repo has combined a learned cue hop, an exact word-identity jump, successor offsets and multi-lag quaternion lineage keys in the read layer.

### First experiment

This is a CPU experiment on mqar-bench, in two steps, with no pod spend until it passes.

**Step A: deployment-geometry bench layout.** About 150 lines in `examples/mqar-bench.rs`, plus a test.
- Add `layout=pair|fact`. The default `pair` keeps today's bytes.
- `fact` writes each pair as [K1 K2 (K3) COP V] with key length 2–3 pieces drawn from a key-piece range and a single shared COP token.
- The query is [Q1 K1 K2 (K3) QM] followed by an answer prefix [A1], where A1 is a filler such as "It's". The weighted target is V at the position after A1.
- So the value's predecessor (COP) is shared by every fact, and the query suffix (A1) carries no key. This reproduces G2 exactly.
- Keep the distance buckets d ∈ {16, 64, 200, 400}, the held-out pairing class and the fresh-seed evaluations as they are.

**Step B: LEA head in `geometric_stack.rs`.** Opt-in `read_lea=M:Kocc:deltas`, saved like `read_key_shift`, refused by every served or export path until ported.
- The data loader computes the C_{t,m} gather tensors exactly from ids. On the bench, words are the key-piece runs; on chat data, Ġ segmentation.
- Tests: an f64 reference and finite differences for both hops; M = 1, δ = 1 with the cue = self equals the F2 read bit for bit when identity admits all; an exact-admission no-false-negative property test.

**Arms.** Fixed: `rrarra`, width 128, 4 heads, MLP 384, context 512, 1800 steps, batch 8, lr 1e-3, l2 read. Seeds 1 and 2 for every arm, on `layout=fact`.

| Arm | Configuration | Purpose |
|---|---|---|
| A | key_shift off | baseline |
| B | key_shift on (F2 lag 1) | tests G2: does F2 fail at deployment geometry? |
| C | F2 + LEA, 1 of 4 heads per read layer, M = 32, K_occ = 8, δ ∈ {1,2,3}, 3-lag key | the proposal |
| D | as C, but hop 2 admits every earlier position (no exact mask; same biases and keys) | isolates exact admission |
| E | as C, but admission is a random set of equal size per query, with the true position excluded | rules out "small sets alone" |
| F | as C, with the Zeckendorf age table replaced by a log2 table | matched φ control |
| G | as C, with lags {1,i,j} replaced by one fixed random SO(4) per lag per lane | j-specificity |

Also run arms A–C on `layout=pair`, 1 seed, as a regression check (C must stay at 1.000).

**Budget.**
- 7 arms × 2 seeds + 3 = 17 runs. #1704 measured about 10–17 min per run at 4 threads; LEA adds gather cost, estimated at no more than 1.5×. Two runs at a time on the 8-core M1 gives about 3.5–4.5 h wall.
- Peak RAM under 2 GiB per run. New retained storage under 200 MiB in sealed report roots under `~/uor-r4-local/mqar-bench/lea-fact-<sha>/`.
- Code and tests take about 6–8 h.
- Charged to the cumulative ledger. No external compute.

**Step C: only if the kill rule passes. One pod A/B, two seeds.**
- Same base, same Arm-C fine-tune recipe, with MQAR/fact dose raised to about 10% to remove the D1 confound and protocol 2 to remove G3.
- Arms: F2 alone vs F2 + LEA.
- Metrics: D19 MQAR cell with the sieve OFF (current 3/109); dev NLL on held-out TinyStories/chat; open panel reported separately and not as the gate.

### Kill criterion

The thresholds are frozen before the run, and every rule must hold on both seeds.

1. **G2 premise falsified.** If arm B (F2 lag 1) reaches ≥0.95 overall and ≥0.90 at d400 on `layout=fact`, then lag-1 lineage already transfers. LEA is unnecessary and is not built further. Record the result and go straight to the Step C chat A/B with F2 only.
2. **LEA killed** if any of the following holds:
   - **(a)** C's overall recall is less than max(A, B) + 0.20.
   - **(b)** C is below 0.90 at d400.
   - **(c)** D, without the exact mask, comes within 0.05 of C. Exact admission then adds nothing; keep only the features.
   - **(d)** C regresses on `layout=pair` below 0.99.
3. **Control rules:**
   - If E (random admission) reaches within 0.10 of C, the gain is set size and not identity. Kill the identity claim.
   - If F or G matches C within 0.02, the Zeckendorf or j-specific claims are retired as unnecessary. LEA survives; those labels do not.
4. **Chat stage (Step C), killed if any of the following holds:**
   - sieve-off D19 MQAR with LEA is below 30/109 on either seed;
   - it is not better than F2 alone by at least 10 items with McNemar p < 0.05 on the paired 109;
   - dev NLL is worse than F2 alone by more than 0.01 nats.

Under D12, a killed arm stays available and is not deleted. The verdict is scoped to this bench, these sizes and these seeds.

### Expected result if right

**Bench, `layout=fact`, both seeds:**

| Arm | Overall | d400 | Held-out pairing class |
|---|---|---|---|
| A | ≤0.3 | — | — |
| B | 0.3–0.7, with d200/d400 ≤0.5 (the value's predecessor COP is shared, so lag 1 cannot discriminate) | ≤0.5 | — |
| C | ≥0.97 | ≥0.95 | ≥1000/1024 |

- C reaches 0.9 at a step no later than B does on `layout=pair`.
- C's hop-2 positions scored per query token is below 100, against about 2,855 dense.
- D sits between B and C. E is near B.
- F and G are within noise of C. **H:** this would retire the "j is canonical" and "φ beats 2" claims, while keeping quaternion lags as cheap orthogonal channels.

**Chat stage, if reached:**
- Sieve-off D19 MQAR rises from 3/109 to ≥50/109 with LEA, while F2 alone at the same dose stays ≤25/109.
- Dev NLL within 0.01 nats.
- Open panel unchanged within ±3/232. That would confirm G5: the panel plateau is knowledge/capacity, which a retrieval mechanism cannot move and which should then be addressed by corpus.
- Phase 2 would then test whether store-as-admission-source keeps session 1008 with the authored sieve removed.

### Cost

**Engineering:** about 6–8 h total.
- Bench layout: about 150 lines plus a test.
- LEA head: about 450–600 lines plus tests, covering the gather-based two-hop read, data-loader admission, save/load and refusal paths.

**Compute, Step A–B:** about 4–4.5 h wall on the local M1 at 2 × 4 threads. RAM under 2 GiB per run. Retained storage under 200 MiB.

**Step C:** two pod fine-tunes at the existing Arm-C recipe size, plus D19 and panel evaluation. Run only after the kill gates pass, and use only an already-authorized pod; no new paid compute is implied by this proposal.

**D11 port (later):** about 300–400 lines.
- The intern table, ring index and watermark rollback are new.
- The scoring reuses the integer L2 kernel and table exp.

Everything is charged to the cumulative ledger, with no silent limit increases.

### Risks

1. **Surface-form brittleness.** Exact identity misses paraphrase and morphology ("Bob's car" vs "Bob", synonyms). Mitigations: normalisation removes case, Ġ and possessive; dense L2 heads remain as a learned fallback. True paraphrase recall still needs learned similarity, and LEA does not claim it.
2. **Common-word floods.** "is" and "what" have many occurrences. K_occ = 8 bounds the cost, but hop 1 must learn to down-weight function words. Arm D versus C and a stopword-free variant will show whether hop 1 learns this. Do not author a stopword list; that would reintroduce the sieve's reserved list.
3. **Sieve-in-disguise objection.** Answered by the D/E controls, by natural-text NLL, and by the requirement that cue choice, offset and ranking are learned. Still, admission by exact word identity is a strong inductive bias that natural text may not reward beyond induction-style copying (Olsson et al., arXiv:2209.11895, and Infini-gram, arXiv:2401.17377, both show exact-match continuation is valuable but not sufficient).
4. **Tie with soft reads again.** #1589's ranked routes tied the soft pointer. The bench is built to expose the specific G2 geometry where they should differ; if they tie there too, kill.
5. **Bench-to-chat transfer** (lesson 2). The `fact` layout reduces but does not remove the gap; the real tokenizer's word segmentation is only exercised in Step C.
6. **QAT/GPTQ fragility** of near-one-hot two-hop reads at 4 bits. Measure the argmax agreement of float vs integer hop-2 picks on the 109 before any served claim.
7. **Single-seed lottery.** Two seeds is the minimum. If C's seeds disagree, add 3 seeds before any verdict.
8. **Over-claim risk.** A recall win does not move the open panel if G5 holds. Reporting must keep the panel separate and use the formal vocabulary.
9. **Scope and ownership.** The `PrimeRoute`/pointer paths were handed to DeepSeek and admission work is Codex-owned (#820). LEA should be claimed as a read-layer option with explicit file ownership in `geometric_stack.rs` reads, coordinated with the Codex `CandidateAdmission` work, not overlapping it silently.
10. **"No RoPE" ruling.** LEA uses no rotary position. The quaternion lags are content lineage, not position rotation, and the Zeckendorf age is a bias table. The owner should still confirm this reading explicitly.

### Red team, generalization lens: REFUTED

Checked at 71b54adf (crates/uor-r4-training/src/milestone_world_v2.rs, examples/mqar-bench.rs).

1. **The premise that deployment needs two hops is half wrong.**
   - What is right: no MQAR query phrasing ends in "{k} is" (`milestone_world_v2.rs:997-1012`, revision 2.1 doc at `:50-56`).
   - What the proposal leaves out: `MQAR_REPLIES` at `:1022` is `["{k} is {v}.", "{k} is {v}.", "It's {v}.", "That's {v}."]`. Half the reply templates restate "{k} is" before the value.
   - The model writes its own reply. It can copy k from the question, which is a short hop, then emit "is", and lag-1 F2 then answers. F2 here is #1701/#1704: the key at the fact's "is" carries the key's last piece through j·k_{t−1}.
   - So the G2 two-hop barrier is a choice of reply template, not a requirement of the task. A model trained to answer "{k} is {v}" turns two-hop into self-cued one-hop. Chain-of-thought-style self-cueing is the standard way around induction indirection.
   - The proposal's own kill rule 1 tests F2 only against a bench answer prefix fixed to "It's". It never tests the self-cue form the data already contains.

2. **This is the sieve and an exact-match induction head under a new name, not geometric attention.**
   - Hop 2 admits positions by exact interned word identity and then reads successors. That is `sieve_value_text` (`:2205-2244`) with a learned softmax replacing "most shared atoms, then recency".
   - It is the same algorithmic family as Infini-gram (arXiv:2401.17377), hard-wired induction heads (Olsson et al., arXiv:2209.11895) and kNN memory (Memorizing Transformers, arXiv:2203.08913).
   - The geometric parts can be absorbed or are cosmetic:
     - The {1, i, j, k} lag channels are fixed signed permutations, so a learned W_q/W_k absorbs them. The proposal's arm G concedes this is untested.
     - The Zeckendorf age is a bucket table, with arm F as its log2 control.
     - The proposal itself drops primes, zeta, Hopf and E8 from the mechanism.
   - What carries the result is an authored oracle mask, not R4/S3 structure.

3. **It will not generalise beyond recall, and the proposal says so.**
   - Its forecast is that the open panel stays within ±3/232, its "G5", which it states up front.
   - So the expected payoff is to move one synthetic MQAR cell that the external sieve already solves (105/109 sieve on) into the network.
   - Exact surface identity cannot handle paraphrase, morphology or coreference. "What is it?" makes "it" the cue, whose successors are wrong unless hop 1 already resolved the antecedent. That leaves the hard part with learned similarity.
   - Nothing in it addresses composition, reasoning or coherent generation, which make up the 3-month barrier.

4. **History argues against it.**
   - In #1587/#1589, exact prime/n-gram routes lost (MQAR 2–3 against soft 16), and the ranked routes only tied the soft pointer (R3b 15 vs 16).
   - Its other precedents follow the same pattern: A4 (1,336 locally vs 28 after load), the E3 authored panels, and the Codex supplied-record fits (#1705, fresh 0–1/32). Constructed exactness did not transfer.
   - The `layout=fact` bench is built by the proposer with oracle word segmentation ("words are the key-piece runs"), unique made-up keys and no paraphrase. With exact admission, arm C winning there is close to true by construction. That is lesson 2 of the synthesis repeating itself.

5. **The gates are too weak to tell a sieve from a model.**
   - Natural text only has to be non-inferior, within 0.01 nats, and only in Step C.
   - No coherent-generation or reasoning metric gates the work.
   - The 6–8 h of engineering plus a 450–600-line read path is spent before testing the cheaper self-cue explanation.

**Killing experiment:** CPU only, mqar-bench extended with `layout=fact`, rrarra, the fixed recipe, seeds 1 and 2. Frozen rule: LEA is dead unless C beats both B1 and D by at least 0.10 at d400 on both seeds and also shows a natural-text gain.

**Two answer-prefix variants:**
- (i) `prefix=It's`: the value is preceded by a shared filler.
- (ii) `prefix=self`: the target sequence is "[K1 K2 (K3)] COP V". The model must first emit the key pieces, copied from the question a few tokens back, then COP, then V. This mirrors the existing "{k} is {v}" replies at `milestone_world_v2.rs:1022`.

**Arms:**
- B1: F2 lag-1 on (ii).
- B2: F2 on (i).
- C: LEA on (i).
- D: C without the exact mask. This is a dense L2 head with the same 3-lag keys and δ/age biases.

**Kill conditions** (each scored on full-sequence exact match of the generated reply):
- B1 reaches ≥0.95 overall and ≥0.90 at d400. The two-hop problem then goes away when the model emits "{k} is", so LEA is unnecessary. Change the reply template or the training target instead.
- D comes within 0.05 of C. The exact mask then adds nothing.
- C does not improve held-out TinyStories/chat dev NLL over F2 by more than the measured seed spread when trained on natural data, with no chance to exploit the synthetic oracle segmentation. It is then an in-network sieve with no language value.

Any one of these retires LEA as a barrier solution. A recall-only win confined to (i) counts as a sieve relocation, not a capability result.

**Salvageable part:** 1. **The `layout=fact` bench.** Multi-piece keys, a shared copula, a key-free answer prefix and a held-out pairing class make it a valuable instrument. Add the `prefix=self` variant. It is the first bench that measures whether F2/#1701 transfers to the actual deployment geometry rather than the gap-1 single-token layout. Run it on F2 before anything else.

2. **The control design.** These controls should be standard for any future read-mechanism claim:
   - D: no exact mask;
   - E: random admission of equal size;
   - F: Zeckendorf vs log2;
   - G: j vs a random SO(4).

   G in particular settles whether "canonical token lineage" means anything specific to j or is just any orthogonal previous-token channel.

3. **Multi-lag orthogonal lineage keys, k + i·k_{t−1} + j·k_{t−2}.** This is a zero-parameter extension of F2, with per-lane orthogonality at identical content proven. It is a cheap arm to add to the F2 pod A/B to cover multi-piece keys, with no new read path.

4. **The exact word-interning index with eviction counters.** It makes absence provable, follows the AGENTS.md absence-proof rule and uses tabulation hashing, which is D11-clean. It is worth keeping for the phase-2 idea of making the StackStore an admission source with a learned ranker, replacing the authored sieve's answer path. That is a grounding and memory-architecture improvement, not a fix for geometric attention.

5. **The proposal's §5 verdicts are rigorous and consistent with the repo's prior reviews.** They are worth recording on #820:
   - primes are identity labels, not computation;
   - Fibonacci primes have no addressing role;
   - zeta phases can be used only as a matched-control arm;
   - O(1) exact-recall state is impossible.

### Red team, cost-serving lens: REFUTED

This is refuted under the D11/laptop-cost lens, for the proposal's cost and serving claims. It is not refuted as a recall experiment; this lens cannot test that.

All repo facts were checked at 71b54adf.

1. **The headline cost comparison mixes two different units.**
   - The proposal's 2,855 "positions per query token" comes from mqar-bench's `positions_scored`. That field is defined as `read_layers × heads × (position+1)` (crates/uor-r4-training/examples/mqar-bench.rs:601-606).
   - For `rrarra` that is 2 read layers × 4 heads × about 357 average positions, so it is a total over 8 heads. Per head, the dense comparator is at most `position+1`, which is at most 512 on the bench and 384 at production context (docs/compute/ladder-runbook.md:147).
   - The proposal sets "C's hop-2 <100" against that 2,855 total. LEA replaces only 1 of 4 heads per read layer, and the other 3 stay dense.
   - Correct total for arm C: 2 × (3 × 357 + 32 + |C_t|) ≈ 2,206 plus 2|C_t|. That is a 20–25% reduction at best, not about 30×. The D5 "sparse by construction, cost scales with occurrences" claim does not hold for the model as proposed, because the dense heads still scan everything.

2. **Hop 2 is not sparse at deployment context, and can cost more than a dense head.**
   - Hop 1 is a softmax over 32 cues plus ∅, so no cue gets exactly zero weight. A faithful forward or served pass must therefore compute hop 2 for every cue.
   - The worst case is M × K_occ × 3 = 768 candidates per head. That is twice the 384 positions a dense head scores at production context.
   - Chat contexts are full of function and template words ("is", "what", "it's", "the", "please", "remember"). Each of these hits the K_occ = 8 cap, giving 24 candidates per such cue. With about half the 32 cues being function words, a head scores about 400 or more candidates against 384 dense.
   - Reaching "10–30 typical" would require either hard top-1 cue selection, which changes the trained function, or an authored stopword list, which the proposal itself forbids (risk 2).
   - The memory pattern also gets worse. Successor positions are scattered, so reads become gathers over the k'/v cache instead of a contiguous scan. On M1, read bandwidth and cache misses dominate cost, not the count of compare operations.
   - The KV cache cannot be dropped, because any position can become a successor. LEA adds memory on top of it: the intern table, the inverted index, and the touched-word log for rollback. On top of that, session rollback has to restore the dense KV cache.

3. **The served function is not the trained function.**
   - Training uses the product of normalised per-hop softmaxes p(m)·p(s|m). The proposal serves a "union" softmax over ℓ1 + ℓ2 and calls it "exact for argmax".
   - That claim is false. p(s|m) is normalised separately for each m, so a cue with a single admitted successor gives that successor p = 1 whatever its ℓ2 score. The union form has no such per-cue renormalisation, so the two forms differ even in argmax.
   - The read is also not an argmax: r_t = Σ p·v is a weighted sum.
   - Making the product form exact on the integer path needs a per-cue log-sum-exp, which means 33 more table softmaxes per head per token. Switching training to the union form instead makes it a different model, with different NoRead/∅ semantics and a different "probabilities of cues add" property.
   - So either the serving cost goes up further, or the factorised model is not what gets served.

4. **The serving story rests on parts that do not exist yet.**
   - F2 itself has no D11 form. Every served path refuses `read_key_shift` (crates/uor-r4-training/src/stack_export.rs:346, :991, test :1726), and `uor-r4-integer` has no key-shift code.
   - There is no QAT for F2 either, and the proposal's own risk 6 admits that 4-bit fragility of near-one-hot two-hop reads is unmeasured.
   - The integer L2 kernel (`stack_l2_distance`, crates/uor-r4-integer/src/stack/kernels.rs:841) does exist. It is a per-coordinate square plus a digit square root over up to 768 candidates, so it is not cheap either.
   - The dynamic intern hash table and the per-word rings also allocate at serving time, which conflicts with the steady-state no-allocation discipline of the frozen kernels.

5. **Smaller exactness overclaim.** "No cross-talk" between the lag channels holds only for identical content. For different lanes x and y, Re(x̄ ū v y) ≠ 0 in general. The 3-lag key therefore does interfere in the L2 score.

What does survive: the operation types (tabulation hashing by XOR and table reads, integer compares, signed permutations, Zeckendorf bucketing by compares) do meet the letter of D11. The objection is cost and fidelity, not prohibited instructions.

**Killing experiment:** Run this before any further engineering. It is a cost-accounting run on the existing bench with the 96M-shaped configuration and context 384.

**Step 1: honest per-token instrumentation.**
- For arm C, log per query token:
  - the true candidate count Σ_m |C_{t,m}| over all 32 cues, with no pruning;
  - hop-1 count + hop-2 count + dense-head count (3 × (t+1)) per read layer;
  - bytes gathered from the k'/v cache;
  - bytes for the intern table, index and rollback log.
- Do this on real #1017-tokenised chat sessions (D19 transcripts, non-sealed), not on bench filler.

**Kill rule for the cost claim:** if the median of (LEA total positions scored) / (all-dense total) exceeds 0.85, or the 90th percentile of the per-LEA-head count exceeds 384, the sparse-access/D5 claim is dead.

**Step 2: training-serving fidelity.**
- Train arm C in the factorised product form.
- Evaluate it on the 109 sieve-off items twice: once in the trained product form, and once in the proposed served union form, both in float.

**Kill rule for the serving story:** if argmax agreement of the hop-2 pick is below 0.99, or recall drops by more than 0.02, the "union is exact for argmax" serving form is dead. The product form, with its per-cue log-sum-exp, must then be costed instead.

**Step 3: J/token on the M1.** Only if steps 1 and 2 pass, measure J/token (Joules per token) on the M1 for the D11 integer engine with and without one LEA head per read layer. Without a measured J/token reduction, no energy or laptop-advantage claim may be made.

**Salvageable part:** 1. **The diagnosis is worth acting on regardless of cost.** In deployment the cue and the value are separated by a copula and a template ("What is {k}? It's {v}"), and keys are multi-piece BPE words. The current bench (lag-1, single-token key, query = key repeated) does not test this geometry. So F2's 1.000 recall (#1701/#1704) is scoped to the bench and does not yet speak to deployment.

2. **Step A is cheap and decisive.** It adds `layout=fact` to examples/mqar-bench.rs, uses only CPU, and changes no model. Arms A and B on it (F2 off/on, two seeds) should run before the pod A/B in #820, because F2 might not transfer.

3. **Arms C, D and E are a clean test.** They compare exact admission against no mask and against random admission of equal size. That separates identity from set size, and is the right causal design if the recall question survives.

4. **Two parts of the serving design are sound and cheap:**
   - exact word-level interning with tabulation hashing (arXiv:1011.5200), which is XOR plus table reads;
   - an inverted index with an eviction counter for provable absence, which meets the AGENTS.md absence-proof rule.

   Their best use is as an exact admission source for the existing StackStore/sieve, letting the store and the network share addresses (ADR-0003 stage 4). There they help without claiming in-network sparsity.

5. **Two of the verdicts on the owner's ideas are correct and should be kept:**
   - primes serve as UOR labels and integer ids do the computing;
   - "j is canonical" and "φ beats 2" are untested and need matched controls (arms F and G).

6. **Any future cost claim must be reported per head and per layer.** The bench's `positions_scored` field is a total over all read heads and layers, not a per-head figure.

### Red team, process-evidence lens: REFUTED

I checked the proposal against 71b54adf (git HEAD 71b54adf86e5). The citations hold, but the plan does not survive this lens.

**What checks out.**
- `milestone_world_v2.rs:51-56` and `:997-1012`: since revision 2.1 no MQAR query ends in "{k} is".
- `:1022`: the replies include "It's {v}." and "That's {v}."
- `sieve_value_text` is at `:2205-2244`.
- `mqar-bench.rs:15-24`: the bench is a single-token key at p, the value at p+1, and the key repeated as the query.
- PR #1704: `rrarra` without the shift scores 0.007 on seed 1 and 0.9995 on seed 2; with the shift, 1.000 on both seeds.
- PR #1589: the exact token route scored 2/109 against the soft pointer's 16.
- So the G2 observation is real.

**1. The first experiment cannot decide the question that matters (decisive).**
- In the proposed `layout=fact`, the generator defines the "words": they are the key-piece runs. Every key piece is a unique token from a disjoint range.
- Exact identity admission therefore puts the true successor in C_{t,m} by construction, with almost no distractors. The proposal's "no false negatives (P)" holds there only because the segmentation is supplied by the generator.
- So arm C ≥ 0.97 is close to a tautology. It shows that an oracle index plus a 32-way learned choice works. That is the A4 / E3 "constructed success" pattern the synthesis lists as never transferring (lesson 2).
- The open question is whether exact word identity holds on the #1017 tokenizer in chat. That is deferred to Step C on the pod, after 6–8 h of engineering and about 4 h of CPU.

**2. The (P) claim is false on real text.**
- On byte-level BPE (#1017, `add_prefix_space:false`, catalogue 8.3), a rare made-up word can split into different pieces with and without a leading space, and at sentence start versus mid-sentence (for example "{k} is {v}" against "What is {k}?").
- Stripping Ġ from each piece does not repair a different split. Exact interning of the normalised sequence then gives false negatives exactly where the deployment needs a match.
- Nobody has measured how often this happens on the 109 D19 items.

**3. It repeats measured history.**
- Exact admission was already tried on the real chat path: #1587 (2/109 against the soft pointer's 16) and #1589's prime-ranked and ngram-ranked routes, which the proposal itself reports as tying the soft pointer (15 vs 16). Its stated diagnosis of those runs was piece-level matching that was too narrow and too permissive.
- The word-level version already exists. The log sieve (`:2205`) does word-level exact admission, recency ranking and copula skipping, and scores 105/109.
- So LEA's best chat outcome is to re-learn inside the network what the authored sieve already delivers. The proposal itself predicts the open panel will not move (G5). Neither outcome changes the next action on the 43–46/232 plateau. That is the loop pattern AGENTS.md says to reject: a repeat without a decision it can change.

**4. The steps are ordered wrongly, and the plan races a result already in flight.**
- The G2 premise can be falsified cheaply: arms A and B on `layout=fact` need only the roughly 150-line bench change.
- More to the point, the chat A/B of F2 on the pod is already running (#820, 2026-10-05T00:36Z). It measures sieve-off MQAR with lineage keys on the real tokenizer.
- The production `r` layers also carry a width-4 causal conv, which can supply lag-2 and lag-3 context, so the F2 stack may already bridge "is". PR #1704 shows exactly this conv route as a seed-dependent source of predecessor identity.
- Building the two-hop gather head before either result is in starts an experiment the cheaper and pending measurements may make unnecessary.

**5. Cost, ownership and serving.**
- The D11 port needs an intern table, ring index and watermark rollback (300–400 lines).
- It overlaps the Codex CandidateAdmission and pointer-route ownership the proposal names itself.
- It is a large new surface for a mechanism whose chat ceiling is the existing sieve.

**What survives.** The proposal does give its own kill rule 1. Its Zeckendorf-vs-log2 control (F) and j-vs-random-SO(4) control (G) are correctly framed matched controls. Its labelling of P, M and H is mostly honest.

**Killing experiment:** These can run in order and cost little.

**(1) Tokenizer audit. No training; uses only the open D19 fixtures, nothing sealed.**
- For each of the 109 D19 MQAR items and the dev chat, tokenize each key's occurrence in the assertion and in the query with the #1017 tokenizer.
- Apply the proposal's normalisation N and its Ġ word segmentation.
- Count the items where the interned word ids differ, which are exact-admission false negatives.
- Also count, per query, how many earlier occurrences the non-reserved query words get.
- Kill the exact-identity claim if false negatives exceed the sieve's 4/109 miss rate. If they are near zero, LEA's admission is the sieve's admission and adds nothing over 105/109 except learned cue choice.

**(2) Bench arms A and B on `layout=fact`, 2 seeds, before any LEA code.**
- This needs only the about 150-line layout change.
- Add one arm B': F2 extended to 3 lags (k + i·k_{-1} + j·k_{-2}) with dense L2 reads and no exact admission.
- If B or B' reaches ≥0.95 overall and ≥0.90 at d400 on both seeds, kill LEA. The two-hop structure is then learnable by the existing stack with lineage keys.

**(3) Read the pending F2 chat A/B on the pod (#820).**
- If F2 alone lifts sieve-off D19 MQAR substantially (for example to ≥30/109), the G2 premise is dead on real data and LEA is not built.
- Under every outcome, the open panel stays the gate. LEA's own prediction of no panel change means it cannot unblock the barrier.

**Salvageable part:** **1. The `layout=fact` bench.**
- This is the most valuable piece. It uses multi-piece keys, a shared copula before the value, and an answer prefix ("It's") that carries no key, with d ∈ {16, 64, 200, 400} and the held-out pairing class.
- It reproduces the deployment geometry and should be added regardless. It is a better instrument than the current lag-1 bench.
- Run arms A and B on it (plus the dense 3-lag F2 arm B') with 2 seeds. That is the cheap, decisive test of whether F2 transfers.

**2. The exact-tokenizer false-negative audit on D19.**
- It costs no training and determines whether any exact word-identity admission is viable on the #1017 tokenizer.

**3. The orthogonality fact.**
- **Proof:** for pure-imaginary unit quaternions u ⊥ v, Re((u x)‾ (v x)) = |x|² Re(ū v) = 0.
- This justifies multi-lag lineage keys {1, i, j, k} as cheap orthogonal channels that are exact signed permutations under D11.
- A dense 3-lag F2 is a minimal, zero-parameter extension worth testing as B'.

**4. The matched controls.**
- Arm F: Zeckendorf versus log2 age buckets.
- Arm G: j versus a fixed random SO(4) per lag.
- These belong in whatever lineage experiment runs next, so the "j is canonical" and "φ beats 2" labels are tested rather than assumed.

**5. Phase 2's framing.**
- Treating StackStore and the sieve as admission sources ranked by a learned score, not as an authored answer path, is a sound direction for D19.
- It should be revisited only after F2 chat results and the tokenizer audit, and coordinated with the Codex CandidateAdmission work.

## ml-architect: Quaternion n-let lineage read (QNR): multi-lag key and query lineage in four exactly self-orthogonal quaternion slots (a fixed, exact n-gram induction head), learned per-head lag gains, mimetic init, and a recall-dose curriculum, all checked on a bench that matches deployment

**Outcome:** refuted

### Mechanism

Labels: P = proof or exact by construction; M = measured, at the stated scope; H = hypothesis.

**1. Where QNR goes.** QNR generalises F2 inside every `a` read layer of `geometric_stack.rs`. F2 today is `previous_key_channel` at `:8936-8944`: k'_t = k_t + L_j k_{t-1}, keys only, every head, fixed lag 1.

**2. Lag slots.** Use e = (e0, e1, e2, e3) = (1, j, k, i). L_u is left multiplication by u on each 4-channel lane, and L_j is the existing `quaternion_j_left`. Each L_u is a signed permutation, so it is exact on integers (P).

**3. Head features.** For read head h at layer l, with k_t = W_k u_t and q_t = W_q u_t:
- **Key lineage** (lags 0..3): k'_s = Σ_{m=0..3} β_{h,m} · L_{e_m} k_{s−m}.
- **Query lineage,** advanced by one lag: q'_t = α_{h,0} q_t + Σ_{m=0..2} α_{h,m+1} · L_{e_{m+1}} q_{t−m}.
- Positions before 0 are zero, as in F2.
- Values, NoRead, the age bias and the L2 score are unchanged: score = −‖q'_t − k'_s‖² + age.

**4. What the score computes (P).** Write ⟨u x, w x⟩ = Re(u w̄)·‖x‖² for unit quaternions. Then L_1, L_i, L_j and L_k are isometries whose pairwise "self-coherence" is exactly zero on every lane and every x. Expanding ⟨q'_t, k'_s⟩, the aligned terms are:

  α_{h,0}β_{h,0}·⟨q_t, k_s⟩ + Σ_{m=0..2} α_{h,m+1}β_{h,m+1}·⟨q_{t−m}, k_{s−1−m}⟩

The remaining cross terms ⟨q_a, L_{ū w} k_b⟩ (u ≠ w) are zero whenever the same vector sits at two lags, and are zero-mean noise otherwise. So:
- **Suffix head.** The second sum is a suffix-alignment kernel. It compares the 3-token suffix ending at t with the 3-token window ending at s−1, then reads the value at s, the token that followed. That is an order-3 n-gram induction head (Akyürek et al., arXiv:2401.12973, "n-gram heads") with exact Carry.
- **Order is kept (P).** Using the identity map for every lag collapses "a b" and "b a" into the same key. The slots keep ordered n-lets apart.
- **Hurwitz–Radon optimality (P).** ρ(4) = 4, so {1, i, j, k} is a maximal family of lane isometries with pairwise zero self-coherence. Four lags, 0..3, is therefore the exact depth limit per 4-lane at the same key width, with no width growth.
- **8-lane extension (H).** An 8-lane variant (unit octonions on paired H4 lanes, E8 = H4 ⊕ φH4; composition algebras satisfy the same identity) would allow lags 0..7. This is a later extension, not part of the first test.

**5. The family contains the existing arms.** The gains α, β ∈ R^4 per head are learned in float, initialised by head group as below, and snapped at export to {0, ½, 1} (shifts).
- α = (1,0,0,0), β = (1,0,0,0): today's plain head.
- α = (1,0,0,0), β = (1,1,0,0): F2 exactly.
- **Suffix group,** about half the heads: α = β = (0,1,1,1).
- **Key-lineage group,** about a quarter: α = (1,0,0,0), β = (1,1,1,1). This is F2 extended to lag 3, with learned queries. It serves non-adjacent cues such as "What is tamir? … It's →V", where the cue reaches the query only through the recurrence state.
- **Content group,** the remaining quarter: plain.

**6. Initialisation (mimetic, Trockman & Kolter, arXiv:2305.09828).** In suffix heads, W_q is initialised equal to W_k. Every aligned term is then a learned token similarity that peaks for identical tokens, so Match holds at step 0 and only Copy (value → logit, plus the pointer) has to be learned.

**7. Curriculum and data.**
- **Base pretraining:** 3–5% of tokens are distance-diverse recall episodes, with log-uniform distance from 8 up to the context, multi-piece nonce keys, both case and prefix-space variants, and a mix of copula-adjacent ("K is →V") and non-adjacent ("It's →V") queries.
- **Chat fine-tune:** MQAR and fact rows rise from about 0.8% (#820, 2026-10-05T00:36Z) to 10–15% of response tokens. Distance and pair count are accuracy-gated: increase them when dev recall ≥ 0.9.
- **Burstiness:** keep natural in-context repetition (TinyStories names) in the mix, since bursty data is what drives in-context learning (Chan et al., arXiv:2205.05055).

### Why it solves the barrier

**The barrier as I read it (H, with M support).** Induction is Carry, then Match, then Copy (Singh et al., arXiv:2404.07129; "The Token Before the Value Is the Key", arXiv:2609.15545). The stack has full softmax reads over the whole context, so read capacity is not the limit. What it lacked was Carry in the keys:
- The all-read bench recalled only through the age bias: d16 0.807, d200 0.010 (#1698, M).
- With F2, one layer-0 head put 0.99–1.00 of its weight on the value (#1701 probe, M).
- `rrarra` without F2 solved on seed 2 (0.9995) and failed on seed 1 (0.007) (#1704, M). The conv route to the predecessor is a learnability lottery.

**F2 is lag 1 only, and deployment facts are not lag 1.** I verified this in the source:
- MQAR facts are asserted as "{k} is {v}" with 2–3-syllable nonce keys (`milestone_world_v2.rs`, `try_mqar`, ~:1388-1440). The value's predecessor is the shared copula " is".
- Replies are "{k} is {v}." or "It's {v}." / "That's {v}." (`MQAR_REPLIES`, :1022), and the reply key is capitalised (`capitalize(&mut answer)`).
- So the discriminating evidence for V sits at lags 2–4, behind " is", and in half the replies the key is not in the local suffix at all.
- F2 at lag 1 tags V with j·k(" is"), which every fact shares. It can reach lag ≥ 2 only by composing with an earlier `r` conv: the same seed lottery, moved one layer down.

**What QNR changes.**
- The suffix heads make lag 1–3 Carry and Match structural, not learned. This removes the lock-in phase transition that has kept turning results into single-seed verdicts.
- The key-lineage heads keep a learned-query route for non-adjacent cues and anaphora.
- On the dose problem: the D19 fine-tune had about 0.8% MQAR tokens. So 3/109 sieve-off does not separate "cannot" from "never asked to". The curriculum fixes the dose, and the bench decides how much of the gap is mechanism.

**Why the open panel plateaued at 43–46/232 from 29M to 96M (H, with M support).** This is three separable ceilings, and QNR addresses only one of them.
1. **Context use is near zero:** 3/109 network-alone. Any panel row that needs an earlier name, fact or referent, or a self-consistent long reply, depends on in-context retrieval. Induction heads carry most in-context loss reduction in small LMs (Olsson et al., arXiv:2209.11895). n-gram heads cut perplexity on natural text by up to 6.7% at 340M (arXiv:2401.12973).
2. **Knowledge is absent:** the 96M rung saw about 1.5B tokens (about 15.6 tokens per parameter) of TinyStories, TinyDialogues and the synthetic chat-v0/chat-v1 corpora, 0.35/0.05/0.10/0.50 (`docs/compute/ladder-runbook.md` §3.4). No mixture component carries world knowledge, so open-domain rows cannot improve with parameters.
3. **Instruction following** was where the 29M → 96M gain went (56 → 78/98).

**What I expect from each.**
- QNR should move the sieve-off recall cell substantially and lower NLL on copyable tokens.
- It should move the open panel only by the recall, consistency and anaphora fraction.
- I therefore require the 186-row failure taxonomy before attributing any panel change. If recall-type rows are under 15%, the panel lever is the corpus, and I say so in advance.

### Uses primary geometry

**Genuinely load-bearing:**
- **R4/S3 unit-quaternion left actions** as the Carry operators. L_1, L_j, L_k and L_i are the four elements of Q8 ⊂ 2I with Re(u w̄) = 0 pairwise.
- **Exactness (P).** Each is a signed permutation, so lineage is exact in float and in integers alike.
- **Zero self-coherence across lags (P).** A token repeated at two lags never interferes with itself. This matters in repetitive text: "is is", code indentation, list markup.
- **Order kept (P).** (a, b) ≠ (b, a) as ordered n-lets.
- **Depth 4 is the Hurwitz–Radon maximum on R^4 (P).** This is the precise sense in which the quaternion structure, not just "some shift", is canonical for multi-lag lineage at fixed width. The isolation arms test whether the property helps measurably.
- **Ordered n-lets (ADR-0003, `docs/adr/0003-fixed-zeta-prime-route-attention.md`).** The suffix head is the learned, vector-valued realisation of the ordered n-let address. Its near-one-hot reads permit a later exact sparse admission by an ordered 3-let hash over token ids: the R-nlet rule in `milestone_world_v2.rs:2138-2173`, generalised. The learned score ranks only admitted candidates. This is the ADR-0003 stage-4 language-to-address join, as a follow-on (H).
- **Optional E8 extension (H).** Paired H4 lanes, E8 = H4 ⊕ φH4, as 8-lanes with octonion units would give 8 exactly self-orthogonal lags. It is not in the first experiment.

**Not used, stated honestly:**
- Zeta phases, Z[φ] coefficients, Hopf observation and prime arithmetic do no semantic work here.
- Content similarity comes from learned W_k and W_q, consistent with P4 in the synthesis.
- This is a structural operator, not a semantic metric.

### D11 serving story

**Served computation.** QNR adds no multiplier and no float, and leaves the per-query read cost unchanged:
- **Keys:** k'_s is computed once when position s is appended. It needs 3 signed permutations of integer key vectors, from a 3-deep ring of previous keys per head, plus up to 4 integer adds. Gains snapped to {0, ½, 1} are arithmetic shifts.
- **Cache:** the KV cache stores k'_s, so the L2 or table score kernel (`uor-r4-integer/src/stack/kernels.rs:841`; `session.rs:1898`) is unchanged.
- **Queries:** q'_t needs a 3-deep ring of previous query vectors and the same permutations and adds.

**State and numerics:**
- Decode state grows by 3 × (d_key + d_query) small integers per head per layer. Snapshot and rollback must cover both rings; #1704 estimated about 200 lines for the key ring alone.
- Magnitude grows by at most 4× (2 bits), so key/query accumulators widen by 2 bits. The L2 table ranges and the GPTQ scale search must be re-checked.

**Quantisation.**
- QAT applies the same operators to the fake-quantised k and q, so float and integer are the same function up to quantisation of W.
- Risk: selection-sharp reads may be fragile under 4-bit W_k and W_q.
- Required measurement: top-1 read agreement and recall, float vs integer engine, on the bench and on the D19 cell. The gate is an integer recall loss ≤ 0.02 absolute.

**Refusals that must be lifted.** `stack_export.rs:346, 991, 1726` currently refuse the key shift. They are lifted only after the integer port passes bit-exact lineage tests against the float operator on integer inputs.

**Sparse access (D5).** Suffix heads with near-one-hot mass admit exact ordered 3-let hash admission. That admission is integer lookup only, and is the first fair D5 access contest. It is a follow-on, not part of the first port.

### Prior attempts and why different

Each prior attempt below is listed with how QNR differs from it.

- **#1701 (3aa8ba16) / #1704 (2cdae05a), F2.**
  - Prior: lag 1 only, keys only, every head, no gains; never tested at gap ≥ 1, with multi-piece or capitalised keys, or on non-adjacent cues.
  - QNR: lags 0..3 in self-orthogonal Q8 slots; the query-side advanced lineage that turns the read into an exact suffix (n-gram) kernel; per-head learned and snapped gains, so plain, F2 and n-gram heads coexist; mimetic W_q = W_k init; a dose curriculum. F2 is the special case β = (1,1,0,0).
- **#1580 (Codex, Oct 1), predecessor carry.**
  - Prior: Q/K replaced by the previous normalised state (192/192 on 320-step fits). Lag 1, replace rather than superpose, on the Codex path, not the production stack.
  - QNR: superposes current and lagged content without self-interference (P), and runs on the production stack.
- **Ordinary lexical audit (`docs/integration/ordinary-lexical-audit-2026-09-23.md` §4), direct two-token channel, +0.469 bits worse.**
  - Prior: put lineage into the current readout input.
  - QNR: puts lineage into keys matched later. This is a different circuit, and that negative does not apply.
- **#1069 owner residual.**
  - Prior: query-side lineage only, 45.59 → 50.27%, rejected on a preservation floor, one seed.
  - QNR: two-sided slots make alignment exact rather than learned.
- **ADR-0003 prime/n-let route.**
  - Prior: exact ordered n-let addresses with no learned language-to-address compiler (stage 4 NOT_RUN).
  - QNR: is that compiler in learned vector form, with exact admission as a follow-on.
- **R-nlet rule (`milestone_world_v2.rs:2138`).**
  - Prior: an untrained exact 2-gram continuation rule, used as a leak detector.
  - QNR: its learned, embedding-soft, in-network generalisation.
- **A1/D18 outcome D; #1043 (MQAR 30/87,360); #1045 (Zoology curriculum).**
  - Prior: all without key lineage. The score geometry (Lorentz, L2, H4, Hopf, 2I) was swapped while key content stayed unchanged.
  - QNR: changes key content and leaves the score untouched.
- **External relations:**
  - QNR is functionally a fixed, parameter-free Canon-B on Q and K (Allen-Zhu, arXiv:2512.17351, where Canon layers are learned weighted sums of neighbours). It is also a hard-wired n-gram head (arXiv:2401.12973). It is not novel as a computation.
  - What is new here is the exact Hurwitz–Radon-optimal superposition at fixed width, the D11-exact integer form, and the test of whether the quaternion slots beat identity, random-orthogonal and learned lag maps.
  - Memory Mosaics (arXiv:2405.06394), with keys from the past and values from the next token, is the same predictive-disentanglement principle.

### First experiment

**Stage 0 (laptop, no training).**
- **Panel taxonomy.** Classify the 186 failing rows of the open 232-request panel (open, not sealed, transcripts only) into knowledge-absent, incoherent/self-inconsistent, instruction, and recall/anaphora, at 96M-C. This freezes the attributable ceiling before any spend.
- **D19 cell format audit.** Count, over the 109 rows, how often the reply key's BPE pieces differ from the assertion's (capitalisation, prefix space) and how often the reply is "It's/That's {v}".

**Stage 1 (laptop, `examples/mqar-bench.rs`).**
- **Code change.** Add a task mode `geometry=natural` with these cells:
  - gap g ∈ {0, 1, 2}: filler-copula tokens between key and value;
  - key length ℓ ∈ {1, 2, 3} pieces, with a case-variant re-tokenisation of the key at query time in half of the queries;
  - query form ∈ {adjacent "K is →V", non-adjacent "K ? … It's →V"};
  - plus a repeated-token cell, where the filler repeats key pieces.
- **Training data.** One mixed training distribution, scored per cell. Settings as in #1698: width 128, 6 layers, context 512, 1,800 steps, 512 queries per cell, the `held_out_class` evaluation retained.
- **Arms** (one change each):
  - (a) none;
  - (b) F2;
  - (c) QNR keys only, β = 1111, α = 1000;
  - (d) full QNR, with head groups and learned gains;
  - (e) full QNR with identity maps L_1 for all lags (order-collapsing control);
  - (f) full QNR with fixed random per-lane SO(4) maps;
  - (g) a learned dense W_prev per lag (parameterised control).
- **Patterns:** `aaaaaa` and `rrarra`.
- **Seeds:** 5 seeds for (b) and (d) on both patterns; 2 seeds for the other arms on `aaaaaa` only.
- **Size:** about 34 runs at the ≤ 20-minute bench cap, about 10 CPU or Metal hours, under 2 GB new storage. Each report root is claimed exclusively and sealed.
- **Probes:** the #1701 per-head key-position probe, steps-to-0.9 recall per seed, and float-vs-integer top-1 agreement after a 4-bit fake-quant pass.

**Stage 2 (pod, only if Stage 1 passes and the owner confirms GPU authorisation; synthesis contradiction #6).**
- **Pretraining:** 29M rung, the exact `run-29m.sh` recipe at lr 5e-4. Arms {F2, QNR}; the "off" reference is reused from the existing 29M lr 5e-4 base, or rerun if the seed differs. 2 seeds per arm, with the 4% recall-episode pretraining mix.
- **Fine-tune:** Arm C fine-tune at 12% MQAR dose.
- **Readouts:**
  - sieve-off D19 MQAR (109 rows);
  - grounded session score;
  - TinyStories valid NLL;
  - NLL on copyable tokens (tokens whose preceding 2-gram occurred earlier in the window);
  - open panel 232, reported separately.
- **Size:** about 4 × 6 GPU-hours plus 4 × 1 h fine-tunes. This folds into the pending F2 A/B rather than duplicating it.

### Kill criterion

**Bench.** Every condition is fixed before the runs, with means over 5 seeds unless stated.
- **(i) Kill multi-lag lineage.** Full QNR (d) fails to beat F2 (b) by ≥ 0.15 mean recall on the union of the gap ≥ 1, ℓ ≥ 2 and non-adjacent cells, on both patterns. Revert to F2 only.
- **(ii) Kill the geometric claim, keep the mechanism.** The random SO(4) arm (f) is within the 5-seed spread of (d) on every cell, including the repeated-token cell, and the learned W_prev arm (g) matches it too. QNR's value is then just "a lag shift", the geometry is reported as non-load-bearing, and the cheapest exact form is kept.
- **(iii) Kill the lock-in claim.** QNR locks in (recall ≥ 0.9) on fewer than 5/5 seeds per pattern.

**Chat (29M, 2 seeds).**
- **Not Carry.** Sieve-off D19 MQAR stays ≤ 15/109 for both QNR and F2 at 12% dose. Carry is then not the binding constraint: go to the Copy/tokenisation (G3) and case-variant matching investigation, and stop lineage work.
- **Lag depth unnecessary.** F2 at the same dose comes within 5/109 of QNR. Ship F2.
- **NLL guard.** TinyStories valid NLL worsens by more than the observed 2-seed spread at 29M. Revert.

### Expected result if right

**Bench (H).**
- **Full QNR:** ≥ 0.95 recall on every natural cell, including g = 2, ℓ = 3 and non-adjacent queries; ≥ 0.9 on held-out pairings; 5/5 seed lock-in by about step 400 on both patterns.
- **F2:** ≥ 0.95 only at g = 0, ℓ = 1. It degrades to ≤ 0.5 on gap ≥ 1 cells for `aaaaaa`, and is seed-dependent on `rrarra`.
- **Identity-map arm (e):** fails the order and repeated-token cells.
- **Random SO(4) arm (f):** close to QNR on random keys but measurably worse on the repeated-token cell, which is where exact zero self-coherence matters.
- **Integer agreement:** float-vs-integer top-1 agreement ≥ 0.98.

**Chat at 29M (H).**
- **Sieve-off D19 MQAR:** from ≤ 5/109 to ≥ 60/109 with QNR plus dose. The case-mismatch rows are the residual misses.
- **Grounded session:** above the 29M figure in the 959 → 1008 series, because store and network now agree.
- **NLL:** ≥ 0.05 nats lower on copyable tokens, 0.005–0.02 nats lower on TinyStories valid overall.
- **Open panel:** a gain of only about +3 to +10 rows, concentrated in the recall, anaphora and consistency categories of the taxonomy. The rest of the plateau stays and is attributed to the knowledge-absent corpus, and that becomes the next lever.

### Cost

**Build:**
- **Code:** about 300 lines of Rust in `geometric_stack.rs` (generalise `previous_key_channel` to lag sets with gains and the query side; save/load fields) and about 250 lines in `mqar-bench.rs` (natural-geometry cells, arms e/f/g, lock-in metrics).

**Stage 0 + 1 (laptop):**
- about 10–12 CPU or Metal hours;
- under 2 GB new storage;
- no external cost.

**Stage 2 (pod):**
- about 28 GPU-hours (4 × 6 h pretraining plus 4 × 1 h fine-tune), folded into the already-planned F2 A/B;
- about 3 h of laptop D19 and panel evaluation;
- GPU use needs a recorded owner authorisation.

**D11 port** (only after Stage 2 passes):
- about 400 lines covering the key and query rings, snapshot and rollback, the 2-bit accumulator widening, the export refusals and bit-exact tests;
- a re-measure of ids/s at 96M, currently 32.5 at 4 threads (#1691). The expected change is under 3%, since the score kernel is unchanged.

### Risks

**1. Tokenisation can defeat Match.** The capitalised and prefix-spaced reply key ("Tamir" vs " tamir") can give disjoint BPE pieces. Exact-suffix heads then match only " is" plus partial pieces. Mitigations:
- case-variant episodes in the curriculum;
- the key-lineage, learned-query group;
- Stage 0 measures how often this happens;
- it may need an instrument or tokenizer fix rather than a mechanism.

**2. Cross terms under learned W.** These are not zero (only the self terms are, P). Noise grows with 4 lags at d_head = 64 (16 lanes). This could lower content-head precision or raise NLL, which is why the gains are learned and the NLL guard exists.

**3. A gain of zero in Q8 slots may be no gain from geometry.** Arm (f) or (g) may tie. The mechanism would then survive and the "canonical lineage" claim would fall. This is acceptable and must be reported plainly.

**4. Quantisation.** 4-bit GPTQ on W_k and W_q may blunt the selection-sharp reads, and the 2-bit magnitude growth stresses the L2 table ranges.

**5. Wrong expectations for the panel.** The plateau is likely dominated by knowledge absence. A real recall fix can look like a null on the panel. Hence the frozen taxonomy and the separate reporting.

**6. Process.**
- The pod spend conflicts with AGENTS.md's no-CUDA text until the owner records a decision.
- Single-seed temptation: all verdicts need ≥ 2 seeds at chat level and 5 at bench level.
- The Codex native bank (P1) remains an unreconciled competing "geometric attention" definition. QNR does not resolve which mechanism serves chat; it only makes the stack read a fair contender.

**7. The n-gram head could encourage over-copying in generation**, i.e. repetition loops. The open-panel transcripts must be checked for this.

### Red team, generalization lens: REFUTED

I checked the proposal against source at 71b54adf. Two things are confirmed. `previous_key_channel` (crates/uor-r4-training/src/geometric_stack.rs:8936-8944) is k + L_j·shift(k) with a zero pad. `MQAR_REPLIES` (milestone_world_v2.rs:1022) is ["{k} is {v}.", "{k} is {v}.", "It's {v}.", "That's {v}."]. So the diagnosis that lag 1 discriminates nothing behind the shared copula " is" is correct. Three of the proposal's central claims do not hold.

1. **The "exact n-gram induction head" claim (labelled P) is false.** The identity ⟨u x, w x⟩ = Re(u w̄)‖x‖² is correct. But it only cancels a cross term when the same vector sits at two lags. The suffix group uses α = β = (0,1,1,1), which gives 9 lag pairs: 3 aligned and 6 cross terms. Each cross term is ⟨q_a, L_v k_b⟩ with v a pure unit quaternion (±i, ±j, ±k). That is a full-rank bilinear form between different tokens, of the same order as an aligned term, not small noise. Under isotropic q and k, the "suffix match" therefore has 3 signal terms against 6 noise terms of equal variance.
   - Under the L2 score, ‖k'_s‖² also carries within-key cross terms ⟨k_{s−m}, L_v k_{s−n}⟩. These add a match-independent bias to each position.
   - The score is a fixed-weight short convolution on Q and K plus interference. It is not an exact kernel.

2. **"Hurwitz–Radon optimality" is optimal only under a constraint the design chooses for itself, and a simpler layout beats it.** The constraint is superposing all lags in the same lanes. A head has d_head = 64 = 16 lanes. Placing lag m in its own disjoint block of lanes gives zero cross terms for every token pair, not only for repeated tokens, at the same key width. ρ(4) = 4 does not make the quaternion slots canonical for lineage. It only says how many lags fit in one 4-lane before self-terms collide, and that problem disappears with concatenation.

3. **It adds nothing geometric that the project has not already seen tie with ordinary controls.** As a computation it is a fixed depthwise causal convolution on Q and K, which the literature already has:
   - the shift SSM in H3 (arXiv:2212.14052), added for associative recall;
   - Primer's multi-dconv-head attention (arXiv:2109.08668);
   - RWKV token-shift;
   - Canon layers (arXiv:2512.17351, which the proposal cites itself).

   The repo has twice found Q8 structure with no advantage over ordinary maps:
   - Q8 causal continuation, 1,024/1,024, was matched by a repaired ordinary signed-permutation learner (docs/integration/causal-continuation-result-2026-09-24.md:59).
   - KVAR Q8 vs C8 residual failed its geometry gate, at −0.1337 bits [−0.37, +0.10] and 0 (current-state-archive-through-2026-09-24.md:90).

   The proposal's own expectation is that arm (f), random SO(4), comes close to QNR on random keys.

**Generalisation lens.** By the proposal's own estimate this is a recall patch, worth about +3 to +10 of the 232 panel rows. It leaves knowledge absence, coherence and composition untouched.
- The "It's {v}" / "That's {v}" replies (2 of the 4 templates) put no key in the local suffix. The suffix heads are useless there.
- Those rows rely on the key-lineage group, which is F2 extended to lag 3 with learned queries. That still needs the query to carry the key through the recurrence state. This learned composition is the actual hard part, and QNR leaves it unchanged.

**Literature.** n-gram heads (arXiv:2401.12973) give a few percent perplexity on natural text, not coherent generation or reasoning. The "lock-in on 5/5 seeds" claim is unsupported: no QNR run exists, and F2 itself flipped verdicts across seeds within hours (#1698 → #1704). The labelling is also wrong in places: several P labels (exact n-gram kernel, canonical depth limit) are really hypotheses.

**Killing experiment:** Run on the proposal's own Stage-1 natural-geometry bench (examples/mqar-bench.rs; width 128, 6 layers, context 512, 1,800 steps, 5 seeds; patterns aaaaaa and rrarra). Add two arms to the proposal's (a)–(g):
- **(h) Disjoint-lane concatenation:** the same lags 0..3 and the same per-head gains, but lag m written into its own block of lanes (4 of the 16), with no quaternion mixing. This gives zero cross terms by construction.
- **(i) Learned depthwise causal conv on Q and K:** width 4, Canon/Primer/H3-shift style, with gains learned per channel.

Score every arm on these cells:
- gap ≥ 1;
- key length ℓ ≥ 2 pieces;
- non-adjacent "It's →V" queries;
- the repeated-token cell;
- held-out pairings.

**Kill rule for the geometric claim:** if (h) or (i) is within the 5-seed spread of full QNR (d) on every cell, or above it, including the repeated-token cell, the quaternion/Hurwitz–Radon claim is dead.

**Kill rule for the barrier claim:** at 29M, 2 seeds, 12% MQAR dose, if sieve-off D19 MQAR improves but the open 232-request panel moves by ≤ 5 rows, QNR is a retrieval patch, not the solution to the barrier. To make the attribution honest, freeze the 186-row failure taxonomy first.

**Cheapest pre-check, no training:** set α = β = (0,1,1,1), take random W_q = W_k at d_head = 64, and measure the ratio of aligned-term variance to cross-term variance in ⟨q'_t, k'_s⟩ over natural TinyStories token windows. A ratio near 3:6 refutes the "exact n-gram kernel" claim analytically.

**Salvageable part:** These parts are worth keeping:

1. **The diagnosis that lag 1 is the wrong lag for deployment.** It is verified: the facts are "{k} is {v}", so the value's predecessor is the shared copula " is", and 2 of the 4 reply templates (milestone_world_v2.rs:1022) never restate the key. This should change the bench.

2. **Stage 0, which needs no training:**
   - the frozen 186-row taxonomy of open-panel failures;
   - the format audit of the 109 D19 rows (BPE mismatch between capitalised and prefix-spaced keys, and the share of "It's/That's" replies).
   It is cheap, and it decides which ceiling is real before any spend.

3. **The natural-geometry bench cells:**
   - gap g ∈ {0,1,2};
   - multi-piece keys;
   - case-variant re-tokenisation;
   - non-adjacent queries;
   - the repeated-token cell.
   They replace a bench that does not match deployment. Add the order-collapse control (e), random SO(4) (f), learned W_prev (g), and the two arms above.

4. **The recall-dose curriculum.** Raise MQAR from about 0.8% to 10–15%, accuracy-gated, with distance-diverse pretraining episodes. This separates "cannot" from "never asked to" in the 3/109 sieve-off result.

5. **A multi-lag key/query shift as an engineering option for D11.** Signed permutations plus gains snapped to {0, ½, 1} stay multiplier-free and leave the score kernel unchanged. It should ship in whichever form wins among (d), (h) and (i), most likely disjoint lanes, and be described as a fixed short convolution, not as canonical geometry.

6. **The 5-seed lock-in metric and the float-vs-integer top-1 agreement gate (≤ 0.02 absolute recall loss)**, so that single-seed verdicts stop flipping.

### Red team, cost-serving lens: REFUTED

My lens was whether the D11 serving story and the laptop-cost case hold up. Checked at 71b54adf, QNR's own incremental arithmetic does: signed permutations, adds, and gains snapped to {0, ½, 1}. Its D11 section fails for four reasons.

**1. The QAT premise is false on the production path (checked in the code).** The proposal says "QAT applies the same operators to the fake-quantised k and q, so float and integer are the same function up to quantisation of W." But the production stack carries a pointer head, and `qat=true` refuses any model with one (`geometric_stack.rs:92` and `:1298-1302`, `pointer_served_refusal`). The ladder models are therefore trained in float and exported post hoc through GPTQ (`stack_export.rs`). There is no QAT step that would make the snapped lineage operator match training. The Stage 1 check, "float-vs-integer top-1 agreement after a 4-bit fake-quant pass", runs in float on the bench, not in the D11 engine. The export still refuses the key shift (`stack_export.rs:346-350`: "no D11 served form"). So no D11 evidence exists until after the Stage 2 GPU spend and a roughly 400-line port.

**2. Gain snapping is an untrained mismatch between training and serving.** The α and β gains are learned freely in float and only rounded to {0, ½, 1} at export. That rounding is not in any training loop, because QAT is refused. In a selection-sharp read, the score is a sum of up to 7 aligned and cross terms. Moving one gain from about 0.7 to ½ or 1 changes which position wins. The claim "the same function up to quantisation of W" is false in this case too.

**3. The laptop-cost story is illusory today, and QNR does not change it.**
- The D11 L2 read (`uor-r4-integer/src/stack/kernels.rs:841`, `stack_l2_distance`) scores every cached position with `stack_square`.
- `stack_square` (`:259`) builds a 16-entry multiples table, about 15 adds, then does 8 lookups and about 7 shift-adds. That is roughly 30 integer operations where the hardware would use one MUL or FMLA.
- My estimate (from the code, not measured): for context 384, width 1024 and the 4 `a` layers of `rrarrarrarrarr`, the read scores alone come to about 58M integer operations per token. A hardware dot product would take about 3M multiply-adds. The value mix (`stack_mix_row`) and the GEMVs pay the same kind of overhead.
- Measured in #1691: 32.5 ids/s at 96M on 4 M1 threads, with no J/token figure.
- QNR's "<3% slowdown" is plausible, but only because it inherits a serving path that does more work per token than a native-MUL path at equal model size.
- The proposal's only route to an actual cost advantage is the D5 ordered-3-let sparse admission. That depends on near-one-hot suffix heads, which have been measured only for F2 on a synthetic bench (#1701). The proposal itself labels it a follow-on hypothesis. No cost advantage is therefore established or tested by this plan.

**4. Precision and range are asserted rather than designed.**
- Keys and queries are stored as `i32` at exponent −16 (`session.rs` Cache, `keys: &mut [i32]`).
- The `stack_square` contract requires |q−k| < 2^32.
- Superposing 4 lags plus 3 query lags adds about 2 bits of magnitude. Nobody has checked this against the 96M key distributions, the L2 isqrt floor (`MIN_SQUARED_CODE`) or the exp-table range of a sharper softmax.

**Mathematical overclaim (P-level, used to sell "canonical").** "Depth 4 is the exact Hurwitz–Radon limit at the same key width" holds only if each map must act on one 4-lane. At head width 64 (1024 / 16 heads), ρ(64) = 12. So 12 pairwise self-orthogonal signed-permutation maps (a Clifford family) exist over the whole head, at the same width and the same D11 cost. Q8 per lane is therefore one admissible choice, not the canonical maximum. Arm (f), random SO(4) per lane, is the wrong control for this claim.

The mechanism, multi-lag lineage, is cheap and may work. But "D11-exact, laptop-advantaged" rests on QAT that is refused, a snap that is never trained, an engine port that does not exist, and a sparse-access follow-on that is pure hypothesis.

**Killing experiment:** Run this before any QNR pod spend.

**Step 1 — port F2 to D11.** Port F2 (β = 11, no gains) to the D11 engine, roughly 200 lines, keys ring only. Take the existing 29M pointer-bearing artifact trained with `read_key_shift`, or the 8M bench model. Export it through the real GPTQ path; there is no QAT, because a pointer model refuses it.

**Step 2 — measure on the actual `d11-evaluate` engine, at threads 1 and 4, with the float and integer paths side by side:**
- top-1 read-position agreement per head over the D19 109-row cell and the #1698 bench cells;
- sieve-off MQAR recall;
- ids/s;
- an instruction count per token from the D11 audit build.

**Step 3 — repeat with QNR.** Use learned gains, snapped to {0, ½, 1} at export, then the same gains trained with a straight-through snap.

**Kill conditions** (any one is enough):
- integer recall falls more than 0.02 absolute below float;
- read top-1 agreement is below 0.98;
- snapped gains lose more than 0.02 recall against the trained-snap version;
- the per-token instruction count shows the read kernel at no less than about 10× a native-MUL equivalent, while D5 admission (top-k on the near-one-hot heads) cannot cut scored positions by at least 8× without losing recall.

If that last condition fires, the D11 claim and the laptop-cost claim are dead regardless of the bench.

**Separate math control.** On the bench, add an arm using a 12-element Clifford signed-permutation family over the full 64-wide head, at the same cost. If it ties or beats Q8 lanes, the "Hurwitz–Radon canonical quaternion depth 4" claim falls.

**Salvageable part:** Four parts are worth keeping.

**1. Stage 0 and Stage 1 on the laptop.** These are cheap and decisive for the mechanism question:
- the 186-row failure taxonomy;
- the D19 BPE/case audit;
- natural-geometry bench cells: gap, multi-piece keys, case variants and non-adjacent "It's →V";
- arms (b), (d), (e) and (g) with 5 seeds.

They decide whether lag ≥ 2 Carry matters at all, and they need no D11 claims.

**2. The lineage operator itself.** Signed permutation plus add, fixed at serving time, is genuinely multiplier-free. Built once at append time into the KV cache, it leaves the score kernel unchanged. That makes it the right shape for a D11 port. Do the port with F2 first, then fixed gains in {0, 1}, before adding learned gains. Learned gains should be trained through a straight-through snap.

**3. The near-one-hot reads.** These reads (measured for F2, #1701 probe) are the first credible opening for D5 exact sparse admission, the ordered n-let hash from ADR-0003. Only that admission could make the D11 read cost sub-linear in context and give a real laptop-cost argument. It should be promoted from "follow-on" to a measured cost gate.

**4. The honest panel expectations.** The proposal predicts only +3 to +10 rows, with the remainder attributed to knowledge absence. Keep that forecast.

### Red team, process-evidence lens: REFUTED

I verified the proposal against the repo at 71b54adf and against PR #1704. As packaged, QNR fails my lens on four grounds.

(1) **Its motivating evidence is over-read from one or two seeds, and the strongest datapoint cuts against it.** The "conv-route lottery" rests on a single failure: `rrarra` without the shift at seed 1 (0.007, #1698). That did not reproduce at seed 2 (0.9995). The table in PR #1704 also shows `rararr`, the stated stand-in for the 14-layer production pattern, solving at seed 1 without any shift: 1.000 at every distance and 1022/1024 held-out. That is one failure in three conv-pattern runs. The only arm that reliably needs F2 is the all-read `aaaaaa`, and nothing deploys it. Yet QNR's bench kill (i) is scored on `aaaaaa` as well as `rrarra`.

(2) **The production stack already supplies lags 0..3.** Every `a` layer in `rrarrarrarrarr` is preceded by `r` layers with a width-4 causal conv (`geometric_stack.rs:5331-5370`, `9047-9118`), so keys already mix x_{t-3..t}. QNR's lag window duplicates that span. Its added value is structural reliability, but at production patterns the measured unreliability is n=1.

(3) **Two mathematical claims are overstated.**
- The isometry identity ⟨ux,wx⟩ = Re(u w̄)‖x‖² is correct. It only zeroes self terms, where the same vector sits at two lags.
- The cross terms ⟨q_a, L_{ūw} k_b⟩ for different tokens are generic O(‖q‖‖k‖) noise, 12 of the 16 terms. The proposal admits this, which makes "exact n-gram kernel" a description of the aligned terms only.
- "Four lags is the Hurwitz–Radon depth limit at fixed width" holds per 4-lane only. A d_head = 64 key admits ρ(64) = 12 mutually orthogonal-design isometries. So Q8 is not canonical at the head level, and the "canonical lineage" framing is weaker than claimed.

(4) **The first experiment cannot decide the deployment question, and the chat test is confounded.**
- Stage 1 is a 34-run, 7-arm synthetic bench on cells the proposer designed. That repeats the project's recurring failure mode: constructed success that did not transfer (E3, A4, the reader series, Codex 0/32 fresh).
- Kill (ii) compares arms with 2 seeds against a "5-seed spread", which is ill-posed.
- The chat stage raises the MQAR dose from 0.8% to 12% for both QNR and F2 but has no lineage-off arm at 12% dose. If sieve-off recall moves, dose and mechanism cannot be separated, which is exactly the "cannot vs never asked" confound the proposal itself names.
- About half of the deployment replies ("It's {v}.", "That's {v}.", `milestone_world_v2.rs:1022`) put the key outside any 3-token suffix. Neither suffix heads nor lag-3 key lineage can reach it; that needs multi-hop composition, which QNR does not supply.
- The only decisive readout (D19 sieve-off at 29M) needs pod spend with unresolved GPU authorisation, and it would pre-empt the F2 pod A/B already in flight on #820.

Net: a heavier, more parameterised generalisation of a mechanism that has not yet been shown to bind on the deployment cell. It risks starting another bench-arm loop.

**Killing experiment:** This uses the existing bench code plus one natural-format cell and one dose control, run before any QNR code.

(a) **Bench.** In `crates/uor-r4-training/examples/mqar-bench.rs`, add only the deployment-format cell: "K is V" with gap 1–2, 2–3-piece keys, and the capitalised re-tokenised query. Train the production stand-in `rararr` and `rrarra` with key_shift off and F2 on, 3 seeds each, about 12 runs at the 20-minute cap. If the conv-equipped patterns, with or without F2, reach ≥ 0.95 on the gap and multi-piece cells across seeds, QNR has no headroom and dies.

(b) **Chat.** In the pending 29M F2 pod A/B, add a key-shift-off arm at the same raised MQAR dose. If off-at-high-dose comes within 5/109 sieve-off of F2-at-high-dose, Carry is not the binding constraint on D19, and multi-lag lineage is moot.

QNR survives only if conv patterns fail the natural cell across seeds while lineage arms pass, and the dose control shows a lineage-specific chat gain.

**Salvageable part:** Four parts are worth keeping.

1. **Stage 0 is cheap and decision-relevant; do it first.** It has two parts: the frozen failure taxonomy of the 186 failing rows (or the open 232-row panel, transcripts only), and the D19 109-row format audit (BPE-piece mismatch of the reply key, share of "It's/That's {v}" replies).
2. **The deployment-format bench cell.** The bench writes value at p+1 with single-token keys (`mqar-bench.rs:16-19`), while deployment is "{k} is {v}" with capitalised, multi-piece keys. This is a real instrument gap and should become a standard bench cell.
3. **The dose observation.** About 0.8% MQAR tokens in the D19 fine-tune means 3/109 sieve-off does not separate "cannot" from "never trained". Add a dose-only control arm to the F2 pod A/B.
4. **The map-family isolation arms**, kept only if lineage later proves load-bearing on the natural cell: identity map (order-collapsing), random SO(4) and learned W_prev against L_j. They are the right test of whether the quaternion structure itself matters.

Mimetic W_q = W_k init is a cheap optional add-on. The multi-lag Q8 slots, query lineage, head groups, the 8-lane octonion extension and the D11 port should wait until (a) and (b) of the killing experiment show headroom.

## systems: Icosian Lineage Cache (ILC): an F2-lineage read served from a 2I product-quantized KV cache, with exact Z[phi] ADC scoring, j-lags applied as code permutations, and a Q8 (cross-polytope) bucket index for sub-linear admission under D11

**Outcome:** refuted

### Mechanism

Scope: production read layer `a` (L2 score, learned age bias, NoRead; D11 form at `crates/uor-r4-integer/src/stack/session.rs:2087-2224`), with the saved F2 option `read_key_shift` (`geometric_stack.rs:8913-8944`: k'_t = k_t + L_j(k_{t-1}), where L_j is the signed permutation (a,b,c,d) -> (-c,d,a,-b)). Shapes are the 96M rung's: d=1024, 16 heads, hd=64, so 16 quaternion lanes per head, and 4 read layers in `rrarrarrarrarr`.

0. ALGEBRA (P). L_j is an isometry, so <q, k_s + j k_{s-1}> = <q, k_s> + <j^-1 q, k_{s-1}>. The squared L2 distance expands the same way:
   |q - k'_s|^2 = |q|^2 + |k_s|^2 + |k_{s-1}|^2 + 2<k_s, j k_{s-1}> - 2<q, k_s> - 2<j^-1 q, k_{s-1}>.
   - Consequence: the cache stores only PER-TOKEN keys k_s. Lineage is composed at read time.
   - The cross term X_s = <k_s, j k_{s-1}> is a per-position constant, computed once at write.
   - Multi-lag lineage (sum over lag operators u_l in {j, i, k, ...} subset of 2I applied to k_{s-l}) adds one query-side term per lag and ZERO KV bytes. This holds for any linear lag map W, because the term is <W^T q, k>.

1. KEY CODEC: shape-gain PQ with the 2I codebook, trained in.
   - Each 4-lane x of a per-token key is coded as g * c. Here c is one of the 120 unit icosians (vertices of the 600-cell; 7 bits) and g = 2^(e/4) is a 4-bit log gain.
   - Training uses a straight-through snap on k_s before the shift. So the float model trains on exactly the keys that are served.
   - The Gram table G[a][b] = <c_a, c_b> takes values in (1/2)Z[phi]: {0, ±1/2, ±1/(2phi), ±phi/2, ±1}. It is stored exactly as integer pairs (m, n) meaning (m + n*phi)/2.
   - (P) j is in 2I, and left multiplication by j is an isometry that permutes 2I. So nearest(j x) = j * nearest(x), and code(j x) = pi_j(code(x)) for a fixed 120-entry permutation table pi_j. The same holds for every lag operator chosen from 2I.
   - Values are stored as int8 per coordinate plus one shared exponent per head row. That is 4x fewer bytes than the current i32 and keeps value fidelity separate from key coding.

2. QUERY SCORING: asymmetric distance computation (ADC; Jegou, Douze & Schmid, "Product quantization for nearest neighbor search", TPAMI 2011).
   - The query stays at full integer precision. Per head and lane, it builds the 120-entry inner-product row R_l[c] = <q_l, c_c>. This costs 120 x 4 constant-coefficient products, and the coefficients 1/2, phi/2 and 1/(2phi) are shift-add chains (no multiplier instruction). A cheaper exact alternative stores q in the same code and reads G.
   - Lag terms reuse the same row: R^{(j)}_l[c] = R_l[pi_{j^-1}(c)]. This is an index permutation with no new table build. It is the one place where choosing j from 2I, rather than a generic W_prev, saves work: a generic W would need a fresh 64x64 rotation of q plus a new ADC row per lag.
   - Per admitted position s, score_s = sum_l shift(R_l[c_{s,l}], e_{s,l}), plus the analogous lag sum over s-1, plus the stored |k|^2 and X_s. Then one isqrt (the trained read uses sqrt-distance), grid_apply(beta), the age bias, the existing exp table, and the Q31 mix of int8 values by radix-16 digits.
   - Near-one-hot reads (probe: 0.988-0.998 on one position under F2, PR #1704) mean weights below 2^-12 can be skipped exactly. The mix typically touches 1-3 rows.

3. ADMISSION: sub-linear, O(L) probes with O(1) expected candidates.
   - Coarse address: per lane, the nearest of the 8 units of Q8 = {±1, ±i, ±j, ±k}. These are the vertices of the 16-cell, i.e. the 4-D cross-polytope. The computation is argmax|x_i| plus a sign (4 compares, multiplier-free).
   - This is exactly cross-polytope LSH (Andoni, Indyk, Laarhoven, Razenshteyn & Schmidt, "Practical and Optimal LSH for Angular Distance", arXiv:1509.02897). The learned key projection plays the role of the random rotation.
   - Q8 < 2T < 2I is a subgroup chain, so a fixed parent table maps the 120-point code to its 24-point and then 8-point ancestors. Coarse codes are also pi_j-equivariant, which matters because j is in Q8.
   - Address of a position: the tuple of Q8 codes of its LINEAGE key k'_s over a fixed lane subset of size L = ceil(log_8 n) + 1. It is computed in integers at write time, using one signed-permutation add, i.e. the D11 F2 port.
   - Index: T = 2 hash tables per head on disjoint lane subsets. Postings are append-only u32 position arrays, giving 2^(3L) buckets.
   - Query probes: the exact bucket, plus query-adaptive multi-probe (Lv et al., "Multi-probe LSH", VLDB 2007). Each probe flips the lane with the smallest |x|_(1) - |x|_(2) margin to its runner-up, for P <= 8 probes.
   - Support set = {sink} ∪ last W positions ∪ (union of probed buckets, capped at M). Defaults: W=32, M=16.
   - Rollback or snapshot = truncate postings at position t. This is exact because postings are position-ordered. Bucket-cap overflow is recorded as proven eviction, as AGENTS.md requires.
   - Training applies the same admission mask in the loop, as orthant64 did. It adds a commitment term that pulls q and the target k' toward the same Q8 cell (in the style of Roy et al., "Efficient Content-Based Sparse Attention with Routing Transformers", arXiv:2003.05997, used here only as sparse-access prior art).

4. COST ACCOUNTING per token. These are derived estimates from the kernel source at `71b54adf`, labelled H until counted.
   - CURRENT dense D11 L2 read, per position and head:
     - 64 differences, each squared by `stack_square` (a 16-entry multiples table plus 8 digit reads; about 40 ops).
     - One u128 digit isqrt (about 300 ops).
     - The exp table.
     - `stack_mix_row`, 64 x radix-16 digit products (about 1,500 ops).
     - Total: about 4.5k integer ops and 512 B of i32 K/V per position and head.
     - Across 64 head-layers that is about 290k ops and 32 KiB per position.
     - Totals per token: n=384 is about 110M ops and 12 MiB; n=4096 about 1.2G ops and 128 MiB; n=32768 about 9.4G ops and 1 GiB (resident and touched).
     - Comparison: the 4-bit pair-table GEMV over 96M weights is about 48M table reads plus adds, and about 48 MB per token. So even at n=384 the read is the same order as all the weights (H, to be counted).
   - ILC per position and head:
     - Storage: 16 lanes x 11 bits = 22 B of key code, 64 B of int8 value, 4 B of |k|^2 + X, and 8 B of postings. About 98 B per position and head, against 512 B now.
     - Score per admitted position: (1 + lags) x 16 x (read + shift + add), about 100 ops for one lag, plus isqrt and exp. The mix runs over only about 1-3 nonzero rows.
     - Support size is about 1 + W + M = 49, so roughly 49 x 120 + 3 x 300 ≈ 7k ops and about 4.8 KB touched per head.
     - Query coding: 16 lanes x hierarchical nearest-icosian, about 3k ops, plus ADC rows of 1,920 entries, about 8k shift-adds.
     - Total: about 18k ops per head, or about 1.2M ops and about 0.3 MB touched per token across 64 head-layers, INDEPENDENT of n except for the L = O(log n) address length and probes.
     - Resident cache: about 6.3 KB per position against 32 KiB (5.2x). A 32K context needs about 200 MB, against 1 GiB.
   - Ratios against dense: about 90x fewer ops and 40x fewer bytes at n=384; about 1,000x and 400x at n=4096.
   - Honest attribution: the asymptotic win (O(n) to O(log n) per token) comes from SPARSITY, and F2's one-hot reads are what make that sparsity lossless. The geometry supplies:
     - (a) a constant-factor win: exact Z[phi] tables, lag terms by code permutation, and a multiplier-free coarse-to-fine address chain Q8 < 2T < 2I with j-equivariance at every level;
     - (b) the codebook, whose optimality is only in the energy sense. Cohn & Kumar, "Universally optimal distribution of points on spheres", JAMS 20 (2007), arXiv:math/0607446, includes the 600-cell. This is NOT a theorem that it is the best quantizer for learned key distributions; that is tested against k-means below.

### Why it solves the barrier

The barrier has two halves. (i) In-network retrieval failed for months because keys lacked Carry. F2 fixes that on the bench (M, PRs #1701/#1704: recall 1.000 at d16–d400, 1024/1024 held-out pairings). (ii) Even a working retrieval mechanism is unservable on an M1 at useful context under D11 if it stays a dense O(n) scan. The current D11 read spends about 4.5k integer ops and 512 B per position per head, because exact squares, isqrt and digit products replace the multiply (`kernels.rs:783-860`, `session.rs:2155-2210`). That puts the read on the same order as all 96M weights at n=384 and makes 4K–32K context about 10–100x the weight cost (H, derived).

ILC closes the loop:
- **It is the D11 serving form of F2** that #1704 deferred. Every served path currently refuses F2 (`stack_export.rs:346,991,1726`).
- **It makes lineage depth free in KV storage.** This bears on the G2 lag question: "k is v" with multi-piece keys needs lag ≥ 2. Adding lags here costs one extra table pass on the query side and zero bytes, so the lag-depth experiment does not trade against serving cost.
- **It converts F2's sharpness into a provably cheap support set:** sink, window, and a few hashed candidates. Orthant64 failed because it ran admission in the bag regime, where the needed position was not separable (17/32 vs 27/32 full, catalogue §2.1). Under F2 the dense read already puts 0.99 mass on one position, so admission only has to contain the dense top-1. That is a measurable recall property, admission-recall vs the dense oracle, not a hope.

This is the first concrete instance of the D5 contest (catalogue 8.8: "zero measurements"). It is judged by top-1 decision equivalence, which is the criterion D18 §9 asked for.

### Uses primary geometry

- **R4/S3 quaternion lanes.** Keys are coded per 4-lane as gain x unit quaternion, the same lane structure as `quaternion_j_left`.
- **Binary icosahedral group 2I (H4 / 600-cell).** It is the fine codebook, 120 points at 7 bits per lane. It is load-bearing because:
  - the lineage operator j lies in 2I, so lag channels act on stored codes as exact 120-entry permutations;
  - multi-lag operators can be chosen as further order-4 or other 2I elements with the same property.
- **Exact Z[phi].** The 600-cell Gram table has entries in (1/2)Z[phi]. Scores accumulate exactly as integer pairs (m + n*phi), and convert to fixed point once per position. This is a case where exact Z[phi] is the arithmetic of the served score, not decoration.
- **Subgroup chain Q8 < 2T < 2I.**
  - It is the hierarchical, multiplier-free quantizer (argmax|x| for Q8, then children) and the coarse address.
  - Q8 is the 16-cell, the 4-D cross-polytope, so the coarse hash is the known-optimal cross-polytope LSH family (arXiv:1509.02897) realised by a group, and it is j-equivariant.
- **What it does NOT use, stated plainly.**
  - Prime addresses, zeta phases, Hopf, E8 and chirality play no role.
  - The prime/n-let idea of ADR-0003 is honoured in its spirit: the bucket address is the coarse code of an ordered lineage pair (token, predecessor). But no prime arithmetic is involved. Primes would add no metric (P, catalogue: square-free products give only a face lattice).
  - The O(log n) advantage comes from sparsity enabled by F2. Geometry gives constant factors and exactness.

### D11 serving story

Every step is integer and uses no multiply instruction, in the style of the existing `uor-r4-integer/src/stack` kernels.

**Write path, per new position and per read head:**
1. The key projection runs through the existing 4-bit pair-table GEMV and gives an i32 key k_t at exponent -16.
2. The integer F2 key k'_t = k_t + signed_perm_j(k_{t-1}). This is one negation-and-add per coordinate, which is the D11 F2 port.
3. Each lane of k_t is coded:
   - Q8 parent: argmax|x| and its sign.
   - The 2T and 2I children, chosen by constant-coefficient dot products. The coefficients 1/2, phi/2 and 1/(2phi) are fixed shift-add chains in Q30, so no runtime-by-runtime product is needed.
   - The log gain: an integer log2 by bit length, plus a 4-entry mantissa table.
4. The per-position constants are stored: |k_t|^2 by summing the gain table, and X_t = <k_t, j k_{t-1}> by 16 Gram-table reads.
5. The value is requantized to int8 with a shared shift (the existing `stack_quantize16` pattern).
6. The Q8 tuple of k'_t over each table's lane subset is computed and the position is appended to the posting array. If the bucket overflows, the slot overwrite is counted so that eviction is provable.

**Read path, per token and per read head:**
1. Build the ADC rows R_l from the integer query. Lag rows are index permutations of R_l (`pi_j` table).
2. Probe T x P buckets, then merge them with the window and the sink into a deduplicated support of size ≤ 1+W+M. The existing `FlockScratch` dedup and support logic (`flock.rs`) is reused.
3. Score each support position:
   - sum of shifted table reads;
   - plus the stored constants;
   - then `stack_isqrt`, `grid_apply`, the age bias and the exp table, as in the current code.
4. Mix only the rows whose Q31 weight is ≥ 2^-12. The skip is exact, and the skipped mass is reported.

**Snapshot and rollback:** the caches are truncated at position t and the postings are popped from the tail. Both are position-ordered, so this is exact.

**Memory bound:** about 6.3 KB per position at the 96M shape, against 32 KiB now. A 32K context fits in about 200 MB on an 8 GB M1.

**Determinism:** codes are a pure function of integer keys, so identical inputs give identical artifacts. The R1 audit (no `mul`/`fmov` in the kernel) applies unchanged.

**Training side:** float with straight-through on the 2I snap of k_s before the shift, plus the hard admission mask in the loop, so served and trained reads coincide by construction. QAT for F2 is currently refused (`qat=true` with `key_shift`, PR #1704) and would be enabled only through this codec.

### Prior attempts and why different

**orthant64 bounded admission** (catalogue §2.1; `bounded-admission-result-2026-09-25.md`, source 78fde571):
- *What it was:* sign-pattern admission of ≤64 events over 256 tokens. It lost complete answers (17/32 vs same-weights full 27/32) and passed NLL.
- *What differs here:*
  - It ran in the bag regime, without Carry, so admission had no single correct target. ILC admits on F2 lineage keys, where the dense read is about 0.99 one-hot (PR #1704 probe).
  - ILC is audited against the dense oracle's top-1, not judged only end to end.
  - It always keeps a recency window, because recent64 was the strong arm in that very study.
  - Its coarse code is the cross-polytope (Q8) hash with multi-probe, rather than raw orthants.

**D6 information audit** (catalogue §4.5; PR #1464):
- *What it was:* a post-hoc 2I direction snap changed top-1 on 46% of positions, and k-means at 10 bits on 35%.
- *What differs here:*
  - D6 was post-hoc, on a natural-text dot read in the bag regime. There, 89% of the damage already appears with per-lane normalization alone (arm U).
  - ILC is trained in, which D6 itself noted had never been evaluated.
  - It is evaluated where reads have large margins.
  - It runs D6's k-means arm head-to-head at equal bits, so the 2I claim can lose cleanly.

**Geometric read kernel 5.4** (HARM, arm C never run):
- *What it was:* a new learned score over the relative 2I element, with 784 parameters.
- *What differs here:* ILC does not change the score function. ADC computes the same trained L2 score on quantized keys. 2I is a codec and an address, not a semantic metric.

**F7 trained flock 0.009 and `flock.rs`:** exact top-k, but it still scores all n positions (O(n), `flock.rs` header). ILC never scores positions outside the support. F7 was also measured in the bag regime.

**B1 2I lanes and the 2I transport snap (#1506):** these use 2I in the recurrence transport, not in the KV cache.

**Codex cue carrier / native bank (#1705, 0–1/32 fresh):** a separate finite-group automaton over ≤128-token supplied records. ILC is the serving form of the stack's own read, which also answers process gap P1 in favour of the stack L2+lineage read.

**PR #1704's D11 F2 note** (about 200 lines, snapshot and rollback): ILC subsumes it, and in addition makes lags free in storage.

**ADR-0003 adjacent semiprime:** this already gave exact token-level lineage, but needed a language-to-address compiler (stage 4, NOT_RUN). ILC's address is the coarse code of the LEARNED lineage key, which is what that missing compiler needed.

**Not tried anywhere in the repo:** KV-cache product quantization; ADC scoring; cross-polytope or multi-probe LSH; code-space lag composition.

### First experiment

**Phase A: training-free fidelity and admission audit on the existing mqar-bench (CPU, 4 threads).**
- *Setup.* The #1704 roots retain only config, log and report, not weights (verified: `~/uor-r4-local/mqar-bench/key-shift-production-787c350d/B-rrarra-key-shift/` holds 5 small files). So first add a `save_model=` option to `examples/mqar-bench.rs` (the example already has `model.save` at `:1618` in a test).
- *Retrain* with #1704's exact settings (ctx 512, width 128, 4 heads, MLP 384, 6 layers, 1,800 steps, batch 8, lr 1e-3, l2):
  - `rrarra`+shift, seeds 1 and 2;
  - `rararr`+shift, seed 1;
  - `rrarra` without the shift, seed 2, as a lineage-from-conv control;
  - about 11–17 min each, about 1.1 h in total.
- *Post-hoc arms,* with weights fixed and recall reported per bucket d16/d64/d200/d400 plus the held-out pairing class:
  - **O:** dense float, the reproduction anchor; it must match the #1704 table.
  - **D:** 2I code with 4-bit log gain on per-token keys, F2 composed in code space.
  - **K:** per-lane k-means at an equal 11 bits.
  - **U:** unquantized per-lane unit direction with the same gain, D6's arm U.
  - **A1:** sink + window 32 only.
  - **A2:** A1 + Q8-tuple buckets, L ∈ {3, 4}, T ∈ {1, 2}, P ∈ {1, 4, 8}, M = 16.
  - **A3:** A1 + oracle top-16, the ceiling.
- *Also report:*
  - admission recall, i.e. the fraction of query tokens whose dense top-1 position is in the support;
  - top-1 Δ, as in D6;
  - positions scored per query token;
  - bucket-size histograms and probes;
  - counted ops and bytes per token from instrumented integer-equivalent kernels.
- *Budget:* ≤ 1.5 h CPU, under 50 MB new storage, with an exclusive report root (`report_output::claim`).

**Phase B: trained-in codec and admission (only if A passes).**
- Arms on `rararr`+shift, 2 seeds each:
  - B0: float dense;
  - B1: STE 2I codec;
  - B2: STE 2I codec + in-loop admission mask (W=32, M=16, L=4, T=2, P=4) + commitment loss;
  - B3: the same with the k-means codec, the matched non-geometric arm.
- Then a gap-g × key-length bench cell (g ∈ {1, 2}, 2-piece keys) with lags {j} vs {j, i}, to price lag depth at zero bytes.
- *Budget:* about 8 runs × 17 min ≈ 2.5 h CPU.

**Phase C: D11 microbenchmark (no training).**
- Implement the integer ILC kernels in `uor-r4-integer/src/stack`, with no-mul audit and snapshot/rollback tests.
- Time ns/token and bytes/token against the current dense L2 read, on synthetic caches at n ∈ {384, 4096, 32768} on the M1.
- Only after A–C: the pod chat fine-tune, paired with dense F2, on the sieve-off D19 MQAR cell, reporting open-panel NLL separately.

### Kill criterion

All criteria are frozen before Phase A. Each one kills only its own component.

1. **Codec, geometric claim.**
   - The claim dies if, at equal bits, K (k-means) beats D (2I) on any F2 checkpoint by more than 0.01 recall in any bucket, or by more than 0.01 on top-1 Δ. Phase B's trained-in B1 vs B3 on 2 seeds decides the same way.
   - If killed, the claim "2I is the right KV codebook" is withdrawn and served PQ uses k-means. The j-permutation lag trick is then lost and lags cost one ADC row build each.
2. **Codec, any form.**
   - The codec dies if D and K both drop recall below 0.98 in any bucket post hoc AND B1 is below 0.98 after training in.
3. **Admission.**
   - Admission dies if A2 at P ≤ 8 has admission recall below 0.95 of the dense top-1 (post hoc), or if B2 recall falls below 0.98 in any bucket on either seed.
   - If killed, ILC degrades to an O(n) scan over packed coarse codes, still multiplier-free and still about 10x cheaper in bytes, and the O(log n) claim is withdrawn.
4. **Language path (at the pod stage).**
   - The language path dies if same-weights NLL under ILC exceeds dense F2 by more than 0.01 nats per token on the 96M chat validation set.
   - It also dies if sieve-off D19 MQAR under ILC is worse than dense F2 by more than 2/109, paired over 2 seeds.
5. **Cost.**
   - The cost claim dies if the measured M1 read-layer time at n=4096 is not at least 10x below dense, or if per-token read time grows by more than 2x from n=4096 to n=32768.

### Expected result if right

**Phase A (H).**
- D and K both keep recall at ≥ 0.995 at every distance on F2 checkpoints, and top-1 Δ is ≤ 0.01. This contrasts with D6's 0.46 in the bag regime, and would show that margin, not the codebook, drove D6.
- A2 at L=4, T=2, P=4 has admission recall ≥ 0.98, and the query-token recall matches dense within 0.01.
- Positions scored per query token fall from about 2,855 to about 100–150 summed over read heads.
- The `rrarra` no-shift seed-2 control shows lower admission recall. Its lineage is smeared through the conv, so it is less bucketable. This would be a measurable argument that explicit lineage is what makes sparse access work.

**Phase B (H).**
- The trained-in 2I codec plus admission reaches recall 1.000 in every bucket on both seeds.
- 2I and k-means tie within noise. Then 2I wins on serving cost: lag rows by permutation, exact Z[phi] tables, no stored codebook.
- Lag {j, i} rescues the g=2 / 2-piece-key cell where {j} alone fails, at zero added KV bytes.

**Phase C (H).**
- About 50–100x fewer read ops and about 40x fewer read bytes per token at n=384.
- About 1,000x / 400x fewer at n=4096.
- Read time is flat in n up to O(log n).
- At 96M on the M1, the read share of per-token work drops from an estimated tens of percent to under 3%. 4K–32K context becomes affordable at about the current 32.5 ids/s (#1691), with about 200 MB of KV at 32K.

None of this claims panel gains. The plateau (G5) is a separate, data-bound question.

### Cost

**Engineering estimate:**
- About 150 lines in mqar-bench: `save_model`, the post-hoc codec/admission evaluator, and counters.
- About 350 lines in `geometric_stack.rs`: STE 2I codec op, code-space lag scoring, admission mask, and commitment loss. This needs a CUDA parity check on the pod before use.
- About 600 lines in `uor-r4-integer/src/stack`:
  - icosian tables and `pi_j`;
  - hierarchical coder;
  - ADC scorer;
  - posting index with eviction counter;
  - snapshot and rollback;
  - no-mul audit and tests.
- Export wiring of about 150 lines, replacing the three refusals in `stack_export.rs`.

**Compute:**
- Phase A: about 1.5 h CPU.
- Phase B: about 2.5 h CPU.
- Phase C: minutes on the M1.
- Pod stage: one paired 2-seed fine-tune pair, about the cost of the pending #820 F2 A/B. It should be folded into that run rather than added.

**Storage:** under 100 MB new for bench checkpoints.

**External spend:** none until the pod stage, which reuses the already-running authorization path (the GPU authorization gap noted in the synthesis stays an owner item).

### Risks

1. **Natural text is not one-hot.**
   - *Risk:* outside MQAR-like spans, reads are diffuse, and sparse admission can cost NLL. This is the orthant64 failure mode.
   - *Mitigation:* the window plus NoRead plus age bias carry diffuse local reads. The language kill criterion (+0.01 nats) is explicit. A per-head policy is possible: dense-window heads vs ILC heads, chosen by measured entropy.
2. **Hash brittleness.**
   - *Risk:* q and k' may land in different Q8 cells even when their dot product is high. Cross-polytope cells are coarse (90° neighbours).
   - *Mitigation:* multi-probe, T=2, training in the loop with a commitment loss, and the post-hoc admission-recall audit before any training.
3. **The 2I codebook may simply lose to k-means.**
   - D6 precedent: k-means was better post hoc. The geometric claim is staked on an equal-bits arm and will be withdrawn if it loses. The sparse index and the D11 F2 port survive either way.
4. **STE on a 120-point snap may destabilise the sharp phase transition of induction.**
   - Lock-in was seed-sensitive (D2 gap).
   - *Mitigation:* warm up dense float F2, then switch on the codec, i.e. QAT after lock-in. Report lock-in step per seed.
5. **Bench geometry mismatch (E5/G2).**
   - Gap 1, single-token keys and width 128 do not equal the deployed "k is v" multi-piece keys at width 1024. Phase B's gap × key-length cell partly addresses this, but chat-scale admission recall remains unknown until the pod stage.
6. **The cost estimates are derived from source inspection, not counted.**
   - The dense-read share at n=384 could be lower than estimated if the compiler vectorises the scalar chains. Phase C measures it, and the cost kill criterion is on measured M1 time.
7. **Values at int8 may cost fidelity for diffuse heads.**
   - Keep an option of int16 values per head.
8. **Process risk: rediscovery and non-propagation (P2).**
   - The result must be recorded against #820 and the catalogue §2.1/§4.5/8.8 rows it reopens. Otherwise the next lab repeats orthant64.

### Red team, generalization lens: REFUTED

I checked the proposal against the repo at 71b54adf and against the PR #1704 body. It fails as an answer to the 3-month barrier. Its algebra is correct; the problem is what it claims to solve.

1. **It does not address the barrier.** The barrier is generation and retrieval quality: the open panel stays at 43-46/232 from 29M to 96M, and sieve-off MQAR is 3/109. ILC is a serving-efficiency layer for a mechanism, F2, that has never been measured on language. The proposal says so itself: "None of this claims panel gains." It also credits half (i) of the barrier to F2, which is PR #1701/#1704, not to ILC. PR #1704 §3 already designs the exact D11 serving form of F2 (cache raw k_t, shift on read, about 200-250 lines, snapshot schema unchanged). That form is O(n) but exact. At the production context of 384 the D11 read cost is a derived estimate (H, never counted), not a demonstrated blocker. Building PQ, LSH and multi-probe on top of an unvalidated read is premature optimisation.

2. **"Lags cost zero KV bytes" is not new.** It holds for any shift-on-read design. PR #1704 §3 already caches only raw keys and composes k_s + j·k_{s-1} at score time (verified in the PR body). The algebra (L_j isometry, cross-term decomposition) is correct, at the level of proof, but it adds nothing over #1704's plan.

3. **The geometry is not load-bearing, by the proposal's own admission.** It concedes that the O(log n) win comes from sparsity and that the geometry gives constant factors.
   - The one structural advantage claimed is "lag rows by permutation". That saves one extra ADC row build per lag: 16 lanes × 120 × 4 constant products, about 8k shift-adds. That is the same order as the query coding it already pays.
   - Any codebook can be symmetrised under the 8-element Q8 orbit to get the same property. So the trick does not single out 2I.
   - D6 (PR #1464) found k-means beat the 2I snap post hoc: top-1 changed on 35% of positions against 46%. The proposal's own risk 3 expects the geometric claim to lose.
   - The Z[phi] Gram table is exact, but exactness of a quantised score does not help predictions.

4. **It is a disguised standard efficient-attention stack, not a geometric mechanism.** Cross-polytope LSH via argmax|x| over a learned rotation is the Reformer hash: Kitaev, Kaiser & Levskaya, "Reformer: The Efficient Transformer", arXiv:2001.04451, which takes argmax over [xR; −xR]. Routing Transformer (arXiv:2003.05997) supplies the commitment-loss clustering. PQ/ADC KV caches are established, for example PQCache (arXiv:2407.12820). "Not tried anywhere in the repo" is true; "novel" is not. The literature on LSH and routing attention shows that hard hashed admission trains finickily and costs quality on natural text. That matches this repo's own orthant64 result, 17/32 vs 27/32 (bounded-admission-result-2026-09-25.md).

5. **The sparsity premise rests on a measurement that does not transfer.** The 0.988-0.998 one-hot read is the best single head, at width 128, on synthetic single-token-key MQAR, at 1-2 seeds (PR #1704 read probe). Nothing shows that the 96M chat model's language reads are near one-hot, even with F2.
   - Natural-text reads in this repo were diffuse (bag regime, D6, F7).
   - An L2 softmax can be one-hot while q is far from every key in absolute terms. In that case the Q8 cells of q and k' on a small lane subset need not agree. The commitment loss then changes the model being served, so "served = trained" holds only for a different model.

6. **Process lesson.** History lesson 3 (score geometry swapped repeatedly while key content stayed unchanged) and lesson 5 (findings not propagated) both apply. ILC spends the next cycle on codec and index engineering, about 1,250 lines across bench, training, integer kernels and export. That is before the one question that matters is answered: does F2 move language NLL, sieve-off D19 MQAR, or the panel at 96M? That pod A/B is already pending under #820.

Verified facts:
- `quaternion_j_left` and `previous_key_channel` are at `crates/uor-r4-training/src/geometric_stack.rs:8913-8944`; L_j is (a,b,c,d) -> (-c,d,a,-b) with a zero pad.
- The refusals are at `crates/uor-r4-training/src/stack_export.rs:346`, `:991` and `:1726` (the test).
- PR #1704 states that `qat=true` with the shift is refused, and that the true 14-layer pattern was never run on the bench (`rararr` stood in).

**Killing experiment:** This is a cheap test, run before any ILC code. Inputs: the first 96M F2 chat checkpoint from the pending #820 pod A/B, or failing that the #1704 bench retrains with `save_model`. Run a training-free audit on (a) the 96M chat validation set and (b) the sieve-off D19 MQAR cell. For every read head at every position, record:
- read entropy and top-1 mass;
- whether the dense top-1 position falls in {sink + last 32} ∪ {Q8-tuple buckets of k', L=4, T=2, P≤8, M=16} — the admission recall;
- the NLL change when the read is restricted to that support, with weights unchanged.

Frozen kill rules:
- If, on natural-text positions, the median top-1 mass is below 0.9, or admission recall is below 0.95, or restricted-support NLL is more than 0.01 nats/token worse than dense, the sparse premise dies, and ILC reduces to a quantised O(n) scan.
- Separately, if equal-bit per-lane k-means (11 bits) beats the 2I codec on top-1 Δ by more than 0.01, the geometric codebook claim dies.
- Upstream of both: if the F2 pod A/B shows no gain in sieve-off D19 MQAR or NLL over the no-shift arm at 2 seeds, there is nothing to serve sparsely, and ILC is moot.

**Salvageable part:** 1. **Exact F2 port.** Implement the plan PR #1704 §3 already designs: cache raw k_t, form k_s + j·k_{s-1} on read in the L2/dot/Lorentz kernels, add a container flag with a schema-string bump, and add a bit-exact grid-reference test across `save_state`/`restore_state`. It is the minimal, unblocked step that lets F2 be served and QAT'd at all.

2. **Phase A diagnostics as a measurement.** Keep the admission-recall versus dense-top-1 metric, per-head read entropy and the positions-scored counts, run only as diagnostics, not as a build commitment. They are the first fair data for the D5 sparse-access contest (catalogue 8.8, "zero measurements"), and they test the hypothesis that explicit lineage makes reads bucketable. Run them on language as well as on MQAR.

3. **Lag-depth bench cell.** Run the gap-g × multi-piece-key cell with lags {j} vs {j, i}. It addresses whether one predecessor is enough for "k is v" keys that BPE splits into several pieces. That bears directly on whether F2 transfers to the chat format. It can run with shift-on-read and needs no codec.

4. **Cost and codebook claims.** Count the D11 read's share of per-token ops and time on the M1 at n=384 and n=4096 before any sparse-index work is justified. The current figures are derived only. If and when a KV codec is needed, keep the equal-bits k-means versus 2I comparison as its frozen acceptance test.

### Red team, cost-serving lens: REFUTED

I refuted the proposal through the systems and cost lens. The algebra is sound: a j-lag acts as an exact code permutation on 2I, and lineage terms can be composed on the query side at no extra KV cost. The case for building ILC is the cost argument, and three of its claims fail against the repo at 71b54adf.

1. The large-context regime does not exist in the served model.
- The D11 session refuses any position at or beyond the trained context: `crates/uor-r4-integer/src/stack/session.rs:1433-1434` returns `StackError::ContextFull` when `position >= s.context`.
- The learned age-bias table is exactly `context` entries long: `r.age.get(age_at..age_at + context)` and `ages[position - j]` in `stack_heads`, `session.rs:~2130-2175`.
- The production rungs all use `context=384`: `docs/compute/ladder-runbook.md:90, 147, 164`.
- So the n=4096 and n=32768 columns describe a model that cannot be served. That includes the "~1,000x/400x" savings, "O(log n)", "a 32K context fits in ~200 MB" and "4K-32K becomes affordable". No length extrapolation of the age bias has been trained or measured.
- The cost kill criterion is set at n=4096 against synthetic caches. It would test a kernel, not a usable product.

2. At the context that can actually be served, the measured profile contradicts the estimate that the read is "the same order as all weights".
- PR #1691 profiled the 96M D11 chat at 1 thread. The weight maps took 84.7% of time (62.6% + 22.1%). The read's L2 distances and mixtures took about 10%: `mul_u128` 3.7%, `isqrt` 2.2%, `square` 2.0%, `div_u128` 1.8%. The recurrence lanes share the remainder.
- By Amdahl's law, even an infinitely fast read gives at most about 1.1-1.2x end to end on the measured workload, and roughly 1.3x if every turn ran at a full 384 positions.
- The memory claim is also moot here. The dense i32 KV at n=384 is about 12 MiB per session, trivial on an 8 GB M1.
- The proposal's own derived numbers (110M read ops vs about 48M weight table reads) are labelled H. The one measurement in the repo refutes them.

3. The exactness claims do not hold as stated.
- "Exact Z[phi] ADC scoring" is internally inconsistent. With the query at full integer precision, R_l[c] = <q_l, c_c> multiplies runtime integers by phi/2 and 1/(2phi). Those are irrational, so any shift-add chain is a rounded fixed-point approximation of phi, not Z[phi] arithmetic.
- Exact Z[phi] holds only for code-to-code Gram reads, i.e. symmetric distance computation with a quantized query. That adds query quantization error which training must also model.
- The 4-bit log gain g = 2^(e/4) is not a shift. Three of every four exponents need a constant mantissa product (2^(1/4), 2^(1/2), 2^(3/4)), which is again approximate.
- Served scores therefore diverge from the float STE training graph by rounding that training does not see. This is the same train/serve gap class that already exists: D11 43 vs float 46 at 96M, p 0.71.

4. The admission and sparsity evidence is scoped to the toy bench.
- The 0.988-0.998 one-hot read mass is measured on the MQAR bench at width 128, 4 heads, 6 layers (PR #1704). It is not measured on 96M natural-text chat, where F2 has not even been trained (the #820 pod A/B is pending).
- Cross-polytope LSH optimality (arXiv:1509.02897) assumes a random rotation and an angular metric. Here the projection is learned and anisotropic, and the score is L2 with a learned offset/beta and an age bias. Bucket skew and cap eviction (M=16) then drop true targets. Recording that as "proven eviction" makes the loss auditable, not lossless.
- Hard admission also renormalizes the softmax, including the NoRead slot, for diffuse heads.
- ILC also ignores the second O(n) read, the pointer head (dim 32 Dot), which keeps scanning all positions.

In short: the components are mathematically fine. The claimed laptop advantage under D11 is illusory at the only context the model serves, and the n >= 4096 economics would first need a context-extension result that does not exist.

**Killing experiment:** Nothing has to be built for this. Re-run the PR #1691 sampling profile of `lut-chat engine=d11` on the 96M artifact, at 1 and 4 threads. Use a session forced to fill all 384 positions: a long prefix, with the last 64 tokens timed. Report the fraction of time in `stack_heads` and the pointer read (`stack_l2_distance`, `stack_square`, `stack_isqrt`, `stack_mix_row`, `stack_exp_neg`, `stack_div_u128`).

The proposal's end-to-end cost claim is dead if:
- the read share at position 383 is below about 30%, which caps ILC below 1.43x even at zero read cost; or
- the read share is below about 15% on the everyday-32 distribution that #1691 already measured, which it is.

A second test for the large-n story: before writing any ILC kernel, continue-train or extrapolate the 96M age bias to context 4096. Measure same-weights NLL and sieve-off D19 MQAR (the network alone, which currently answers 3/109) at positions above 384. If the model cannot use positions beyond 384, the n=4096/32768 accounting has no product to apply to.

**Salvageable part:** Four parts are worth keeping.

1. The query-side lag decomposition (proof): |q - (k_s + j k_{s-1})|^2 expands into per-token terms plus one per-position cross term X_s. That makes F2 servable under D11 with no extra KV bytes and keeps exact rollback.
   - Only the integer add of a signed permutation is needed. This is the cheap D11 port that the three `stack_export.rs:346/991/1726` refusals are waiting for.
   - It should be shipped as a plain dense read: no codec, no hash.
2. The equal-bits comparison of 2I code vs k-means on F2 checkpoints (Phase A arms D/K/U). Run it as a cheap scientific test of whether D6's 46% top-1 damage came from read margin or from the codebook. It is not a cost claim.
3. Admission recall vs the dense top-1 oracle, as a diagnostic of how one-hot F2 reads are per head in natural text. It is worth logging in the pod A/B, but not building around.
4. The pi_j code-permutation identity, kept as a proof note in case a future context extension makes KV cost dominant. Before any ILC kernel work, the profile in the killing experiment and a demonstrated context beyond 384 must exist.

### Red team, process-evidence lens: REFUTED

The proposal's algebra is correct. I checked `quaternion_j_left` at geometric_stack.rs:8913-8944. It maps (a,b,c,d) -> (-c,d,a,-b), which is exact left multiplication by j. Since j lies in Q8 < 2T < 2I, left multiplication permutes the 600-cell, so code(jx) = pi_j(code(x)). The L2 expansion with a per-position cross term is also right.

It still fails this lens on three counts.

(1) It works on a constraint that is not binding, before the result it depends on exists.
- The barrier is capability: open panel 43-46/232 flat from 29M to 96M, and 3/109 MQAR with the sieve off.
- F2's effect on any language or chat path is unmeasured. #1704 says the D19 MQAR cell and a chat model with the shift are "not yet measured". The pod A/B in #820 (comment 2026-10-05T00:36Z: key_shift off vs add, then sieve-off D19 MQAR) has not reported.
- Production context is 384, and the D11 engine already serves 96M at 32.5 ids/s (#1691). The O(log n) payoff at 4K-32K is for contexts this model was never trained on. Length generalisation of the learned ALiBi-style age bias is untested.
- The proposal's own cost figures are labelled H, not counted.
- So it commits about 1,250 lines (bench, trainer, integer kernels, export) to a serving optimisation for a mechanism with zero language evidence. That is the off-course, tooling-first pattern the owner warned against on 10-01 (research before tooling).

(2) The first experiment cannot decide anything, because its outcome is fixed in advance by the bench regime.
- On MQAR with nonce single-token keys and reads at 0.988-0.998 one-hot, almost any codec and any reasonable admission will keep top-1. The proposal predicts exactly that: 2I and k-means "tie within noise".
- Kill criterion 1 fires only if k-means beats 2I by more than 0.01. A tie leaves the "2I is the right codebook" claim neither established nor killed.
- Admission recall on unique nonce keys is the easy case. The documented failure mode, orthant64 (17/32 vs 27/32 full), was natural text and complete answers. The only criterion that touches that failure mode (language NLL within +0.01 nats, sieve-off MQAR within 2/109) is deferred to the pod stage, after all the engineering is done.
- Phase A also spends about 1.1 h retraining four bench models only because #1704 kept no weights. That adds bench-on-bench evidence.

(3) The evidence is over-read and, where it helps, single-seed.
- The 0.988-0.998 one-hot figure is the BEST head of ONE layer at width 128 / 1.36M parameters (the #1704 probe). Sparse admission would be applied to every head.
- The one-hotness is not specific to F2. In #1704, `rararr` WITHOUT the shift also scores 1.000 in every bucket, with 0.98-1.00 on layer 1, head 3. So "F2 makes sparsity lossless" is not established as an F2 property.
- The bench has already flipped once on seeds. `rrarra` seed 1 scored 0.007; seed 2 scored 0.9995.
- The proposal re-opens two previously failed ideas, 2I key snapping (D6: top-1 changed on 46% of positions, PR #1464) and sign/orthant admission (orthant64). It does so in the one regime where both will trivially pass. A pass there would then be read as geometric vindication, which is the over-reading risk.

Separately, the "geometry" is a constant-factor serving codec. Every asymptotic gain is attributed to sparsity, which, per its own text, is not geometric. That is honest, but it means even a full pass would not move the capability barrier.

**Killing experiment:** These are training-free and run on the actual model, before any ILC code is written.

(a) **Oracle-sparse language ceiling** on the existing 96M checkpoint over chat-validation text.
- First, find the 96M chat-validation set and pin the identity of the base vs Arm C checkpoint.
- Replace every read head's softmax with sink + last 32 positions + the oracle top-16 by dense score (proposal arm A3, but at language scale), using the same weights.
- If NLL rises by more than 0.01 nats per token, or the per-head entropy shows that most heads are diffuse, sparse admission is dead for the language path. No hash can beat its own oracle ceiling.
- If F2 later lands on the main path, repeat on the F2 checkpoint, since a dense F2 read could be sharper.

(b) **Read-share profile** of the D11 engine at n=384 on the M1 (#1691 configuration).
- If the `a`-layer read is below about 20% of per-token time, the cost motivation fails at the deployed context.

(c) **The pending #820 pod A/B** (key_shift=add vs off, then sieve-off D19 MQAR).
- If F2 does not improve network-alone recall or the panel, ILC is optimising the serving cost of a mechanism that adds nothing.

Any one of (a), (b) or (c) failing kills the proposal at its stated purpose.

**Salvageable part:** 1. **The D11 serving design for F2.**
   - The read-time decomposition <q, k_s + j k_{s-1}> = <q, k_s> + <j^-1 q, k_{s-1}>, with the cross term X_s computed once at write, means the cache holds only per-token keys. Any linear lag operator then costs query-side work and no KV bytes.
   - This is a cleaner and smaller replacement for the roughly 200-line snapshot/rollback design note in #1704. It removes the three export refusals (`stack_export.rs:346`, `:991` and the refusal test at `:1726`) without new storage.
   - Build it only if the pod A/B shows F2 helps.
2. **The exact fact that lag operators chosen from 2I act on 600-cell codes as fixed permutations.** Record it as a proved property (P) for any future quantized cache.
3. **The oracle-top-k NLL ceiling (A3) and read-share profiling.** These are cheap, decisive measurements that should run before ANY sparse-access or D5 contest work, ILC or otherwise.
4. **Using admission recall against the dense top-1 as the audit metric**, instead of judging only end to end.

## outsider: Offset n-let lineage read (ONL): use a quaternion lag frame on both queries and keys, so the read matches the preceding ordered n-let and returns its successor; pair it with cue rehearsal at decode time and exact n-let candidate admission for D11/D5

**Outcome:** refuted

### Mechanism

**What it computes.** The read matches the ordered m-gram that precedes a position and returns the token at that position, its successor. It is a learned, soft, exact-order generalisation of F2 (#1701/#1704). F2 is only the bigram, key-side, gap-0 special case.

**Definitions.** At every read head, with k = W_key u and q = W_query u as now, and L_u = left multiplication of each 4-channel lane by a unit quaternion u (a signed permutation, as in `quaternion_j_left`, geometric_stack.rs:8913-8932):
- Lag frame: u_0 = 1, u_1 = j, u_2 = k, u_3 = i. At most 4 lags per 4-lane; m ≤ 3.
- Key: K_s = k_s + Σ_{l=1..m} L_{u_l} k_{s-l}, zero-padded before position 0. For m = 1 this is exactly today's `previous_key_channel` (:8935-8944).
- Query (new): Q_t = q_t + Σ_{l=0..m-1} L_{u_{l+1}} q_{t-l}. This is the new part: the query carries its own lineage, offset by one lag relative to the key.
- Score: the existing L2 / age / NoRead read applied to (Q_t, K_s).

**What the score contains.** Expanding ⟨Q_t, K_s⟩ gives three kinds of term:
1. A content term ⟨q_t, k_s⟩.
2. Aligned lineage terms Σ_l ⟨q_{t-l}, k_{s-1-l}⟩, because L_u is orthogonal. These reward x_{s-1-l} = x_{t-l} for l = 0..m-1, that is, "the m tokens before s equal the last m tokens before the prediction". The value read at s is then the successor of the matched n-let.
3. Cross terms ⟨q_a, L_{ū_b u_c} k_b⟩ with ū_b u_c pure imaginary.

**Why the cross terms do not leak (P).** For a pure-imaginary unit w, L_w is skew-symmetric: xᵀ L_w x = 0 for every x. So wherever the learned identity score is maximal (q aligned with k for the same token), a token seen at the wrong lag contributes exactly zero. That makes order-sensitivity exact at the match point.
- {1, i, j, k} is a Hurwitz–Radon family on R⁴: mutually anticommuting orthogonal complex structures, ρ(4) = 4. So four mutually skew lags per 4-lane is the maximum.
- A random fixed rotation R has xᵀRx ≈ tr(R)/4·|x|² ≠ 0, so it leaks the token at the wrong lag.
- An identity "shift" (all u_l = 1) collapses to a bag of n-gram tokens with no order.
- This is the precise, falsifiable sense in which the quaternion units are canonical rather than arbitrary. Beyond m = 3 one would need 8-lanes (octonion, ρ(8) = 8); that is H only.

**Cost.** Zero new parameters; m signed permutations plus m adds on each of q and k.

**Decode-side companion: cue rehearsal.** This is not new data; it already exists. MQAR_REPLIES (milestone_world_v2.rs:1022) has 2 of 4 replies of the form "{k} is {v}", so the training target already teaches the model to re-emit the cue. When it does:
- The key pieces are a short-range copy from the question (d ≈ 3–8).
- Each further key piece is predicted by bigram lineage.
- After "…k_last is", the m = 2/3 suffix equals the assertion's "k_last is", and ONL reads v.
- Multi-piece values continue through the suffix "is v₁" → v₂.

This turns unbounded-gap MQAR into a short copy plus an exact n-let match, both of which the stack can do. It is exactly what the authored log sieve does (milestone_world_v2.rs:2205-2244, "copy the words after the matched cue"), learned in-network on BPE tokens.

**Serving-side companion: exact n-let admission, used for D11/D5 and not in training.**
- Each position s is addressed by the ordered n-let identity A_s = (x_{s-m}, …, x_{s-1}): the ADR-0003 ordered-n-let / UOR address, an exact integer tuple and not a metric.
- The served read scores only positions with A_s = (x_{t-m+1}, …, x_t), plus a recent window W and NoRead.
- Admission is exact and integer; ranking stays learned. This follows the AGENTS.md rule that admission is separate from ranking.

### Why it solves the barrier

**1. The stall is a key-content failure, not a score-geometry failure.**
- #1701's probe shows that arm-A heads never put more than uniform weight on the key position.
- F2 fixes Carry only when the cue token immediately precedes the value: bench gap 0, single-token keys (mqar-bench.rs:15-19, "A pair writes its key at position p and its value at p+1").
- Deployment is different. Facts are "{k} is {v}" with BPE-split names, so the token before every value is "is", which all facts share. Lag-1 F2 tags every value with j·k("is"), which reproduces the bag/recency signature seen in A1/D18 (0.28–0.44 ≈ R-recency 0.36).
- ONL makes the matched object the preceding m-gram ("ak is"), which separates the facts. Copy is unchanged because the value is read at the matched position.

**2. The project's own instrument already found this solution and banned it, as a leak.**
- milestone_world_v2.rs:53-55 and 997-999: revision 2.1 removed every MQAR query phrasing that ends in "{k} is", because the untrained R-nlet rule ("continuation of the latest earlier occurrence of the query's last two words") answered them.
- So n-let continuation is known to solve this task when the suffix is present.
- What was missing is a network that (a) produces that suffix itself by rehearsing the cue in its reply, and (b) has a read that can execute an n-let match. F2 gives (b) only for n = 1; ONL gives it for n ≤ 3. The rehearsal needed for (a) is already in the reply templates.

**3. It explains why "rr…" production patterns score 3/109 with the sieve off despite having a conv route.**
- The width-4 conv can in principle synthesise lag-2 lineage on the key side.
- Matching also needs the query's own lag-1 token, and nothing makes the network learn that offset alignment (the #1704 seed lottery). ONL makes it exact.

**4. It unifies the two incompatible "geometric attention" lines (process gap P1).**
- The served stack's L2 read becomes the learned n-let matcher.
- The exact ordered-n-let address (ADR-0003, #958 stage 4 NOT_RUN) becomes its integer admission set.
- That retires the need for a separate supplied-record bank on the critical path.

**What it does not solve.** It does not fix G5 (open-domain knowledge, the 43–46/232 open panel) or the 11-label compiler ceiling (G4). I predict the open panel moves by a few points at most and refuse to gate on it.

### Uses primary geometry

**R4/S³ quaternions (load-bearing, P).**
- The lag operators are left multiplications by the unit quaternions {1, j, k, i} on every 4-lane.
- Their usefulness rests on a geometric fact, not a convention: left multiplication by a pure-imaginary unit is a skew-symmetric orthogonal map (xᵀL_w x = 0), and the pairwise products ū_a u_b are pure imaginary. The aligned-lag identity match therefore has exactly zero leakage from the same token at a misaligned lag.
- {1, i, j, k} is a maximal Hurwitz–Radon family in R⁴ (ρ(4) = 4). This is the precise content of "canonical token lineage": a lag frame that is exactly order-separating at the match point, not "a rotation".
- The first experiment tests whether the claim survives. Planned controls: an identity shift (order lost) and a random fixed SO(4) per lane (leaky), plus learned W_prev (unconstrained).

**Binary icosahedral group 2I / H4.**
- The 30 order-4 elements of 2I are exactly the pure-imaginary units of the 600-cell vertex set, coordinates in Z[φ].
- So any lag frame drawn from 2I keeps exact Z[φ] arithmetic and snaps exactly under the existing D11 2I/Z[φ] snap (#1506).
- An optional arm uses a 2I order-4 triple other than {i, j, k} to test whether the specific frame matters. Prediction: no difference among mutually anticommuting triples (H). That would mean the geometry is structural, not semantic, which is consistent with lesson P4.

**Prime / ordered n-lets / UOR identity (load-bearing at serving).**
- The serving admission key is the ordered n-let token tuple: the same object as ADR-0003's ordered n-let prime address, as an exact identity.
- It is used for exact candidate admission only, never as a semantic distance. This respects AGENTS.md ("a prime/hash identity is not a semantic distance") and the P result that square-free products carry no metric.

**Not used, stated honestly.** Zeta phases, Hopf, E8, and chirality beyond lane orientation are not part of this mechanism.

### D11 serving story

**Exact integer form.** After GPTQ/QAT the served q and k are integer vectors, so the whole served path stays inside D11:
- **Lag operators:** a signed permutation of each 4-lane (lane reorder plus sign flips; zero multiplies). Q_t and K_s are integer adds of at most m+1 permuted vectors.
- **Score:** the existing integer L2 read kernel (D11 session.rs:1898, kernels.rs:841) is unchanged; it simply receives Q and K.

**Session state.**
- A ring buffer of the last m raw k and q vectors per read layer; for m = 3 at width 1024 int8 this is 3·1024·2 bytes ≈ 6 KiB per layer.
- K_s is cached once computed, exactly as k_s is today.
- Snapshot/rollback copies the ring. This is the "~200 lines" #1704 estimated for m = 1; m = 3 changes only the ring length.

**Admission (D5).**
- A hash map from the ordered m-tuple of token ids (x_{s-m}, …, x_{s-1}) to the list of positions s.
- Hashing uses XOR/rotate/table, as in the frozen TLA kernel style. Membership is an exact tuple compare, not a hash collision, so absence is proven by tuple inequality.
- Per query, the read scores |admitted| + W(recent) + NoRead positions instead of all 384.
- This is the first D5 sparse-access path whose loss can be bounded empirically. Under F2 the attention mass is already 0.99–1.00 on one position (#1701 probe), so restricting to exact matches should be near-lossless where a match exists. Where none exists the read falls back to the window and NoRead.
- Report the dense-versus-admitted positions scored per token, and the J/token only when measured on the M1 (R5).

**No float, no multiplier**, and no transformer backbone (the backbone stays the quaternion recurrence plus reads).

**Owner ruling needed.** The "no RoPE" ruling (09-29) should be confirmed not to cover fixed lineage operators. These are not positional rotations by absolute or relative position: they label which token, at a fixed causal offset, a component came from, functionally a parameter-free width-m convolution.

### Prior attempts and why different

**F2 (#1701 `3aa8ba16`, #1704 `2cdae05a`).** K = k_t + j·k_{t−1}; query unchanged; m = 1.
- It solves only gap-0, single-token-key bench geometry (mqar-bench.rs:15-19).
- It relies on W_q learning L_j·W_k, so query lineage beyond the current token is impossible.
- ONL: m ≤ 3, an explicit query-side offset lineage, and a frame chosen for exact skew separation. F2 is the m = 1 corner of ONL.

**Codex #1580 identity carry (192/192).** Query and key both *replace* the current state with the previous normalized state: lag-1 on both sides, not offset and not additive.
- It worked on its own layout. Its own next step names the unsolved problem: "variable-gap typed identity capture/hold/commit/reset".
- The Codex cue carrier/latch (#1705) then scored 0–1/32 fresh.
- ONL does not latch. It handles bounded gaps (copula, multi-piece keys) through the m-lag frame and unbounded gaps through rehearsal, keeping the content channel.

**#1069 owner residual.** Lineage on the query side only: 45.59% → 50.27%, rejected on a preservation floor, one seed. ONL puts lineage on both sides with a one-lag offset, which is what makes successor reading possible.

**"Direct two-token channel"** (ordinary-lexical-audit-2026-09-23.md:108-126, +0.469 bits worse). It put the two-token context into the current readout, not into the matched keys. That is a different mechanism.

**ADR-0003 / #958 fixed-zeta prime route with ordered n-lets.** Exact addresses; stage 4 (the language-to-address compiler) NOT_RUN; never used as a learned key.
- ONL is that missing compiler: the learned soft matcher over exactly the n-let the address names. The address is demoted to admission only.

**Score-geometry swaps (Lorentz/Dot/L2/H4 potential/Hopf sectors/2I codes, E1–E6).** All kept key content fixed. ONL changes key content and keeps L2.

**R-nlet rule baseline (milestone_world_v2.rs:47-55).** An authored rule, and the reason query phrasings ending in "{k} is" were removed. ONL is a learned network mechanism that obtains the suffix by its own rehearsal.
- The rehearsal plus R-nlet composition must be reported as a reference rule, so that measured gains are attributed to the mechanism and not to the template.

**External.**
- Akyürek et al., "In-Context Language Learning: Architectures and Algorithms", arXiv:2401.12973. "n-gram heads" (higher-order induction) explain transformer in-context-learning advantages, and adding them improves recurrent and convolutional models' perplexity. ONL is a parameter-free, multiplier-free, geometrically exact n-gram head.
- The Canon layers (arXiv:2512.17351) and the Based/Jelassi recall limits (arXiv:2402.18668, arXiv:2402.01032) support keeping explicit retrieval rather than O(1) state.

### First experiment

**Step 0. Code** (Rust only, about 250 lines plus tests, no default-path change).
- Generalise `previous_key_channel` to `lineage_channel(x, lags, frame)`, and add query-side lineage behind `set_read_lineage(m, frame)`.
- Frames:
  - `quat` {j, k, i};
  - `ident` (all 1);
  - `rand` (a fixed seeded SO(4) per lane, exact float, offline only);
  - `learned` W_prev, the D0-b-allowed offline control;
  - `icosian` (another anticommuting order-4 triple of 2I).
- Tests: exact skew (xᵀL_w x = 0); m = 1 key side bit-identical to F2; causality (no t+1 leak); save/load.

**Step 1. Extend `examples/mqar-bench.rs` geometry** to the deployment shape, with the parity freeze as in synthesis Q12.
- Keys are 1–3 pieces drawn from KEYS. A copula of g ∈ {0, 1, 2} tokens, drawn from a 4-token shared set, sits between key and value.
- Values are 1–2 pieces.
- Query form, mixed 50/50 in training:
  - REHEARSE: the query writes the key pieces plus the copula, and the target is the value;
  - BARE: the query writes the key pieces plus a distinct "?" token, and the target is the value.
- Distances 16/64/200/400. Report per cell (pieces × g × form × distance), with the held-out pairing class kept.
- Add task baselines: R-recency, R-nlet on REHEARSE, and "rehearse+R-nlet".
- Freeze the acceptance criteria below *before* running.

**Step 2. Arms** at width 128, 4 heads, 6 layers, context 512, 1800 steps, lr 1e-3, l2 read (the #1701/#1704 settings).
- Patterns: {aaaaaa, rrarra}.
- Mechanisms: {none, F2 (m = 1 key), ONL-quat m = 2, ONL-quat m = 3, ONL-ident m = 3, ONL-rand m = 3}. Add ONL-learned m = 3 only if quat wins.
- 3 seeds, so 2 × 6 × 3 = 36 runs.
- At about 10–20 CPU-min each, run 2 at a time at 4 threads each on the M1: about 4–6 h wall, CPU only, no pod.
- Each report root is claimed exclusively and sealed, per AGENTS.md.

**Step 3, only if Step 2 passes. Chat transfer on the D19 sieve-off MQAR cell (109 rows) at 29M** (cheaper than 96M, and with the same 3/109-class baseline).
- Fine-tune the existing 29M lr5e-4 base for {none, F2, ONL-quat m = 3} × {MQAR dose ~2% as now, ~15%} with 2 seeds: 12 runs on the pod.
- Report per arm:
  - sieve-off recall;
  - the reply-form split (rehearsed vs bare);
  - the rehearse+R-nlet reference;
  - the open panel separately (not a gate);
  - base NLL preservation.
- The in-progress #820 lag-1 F2 pod A/B is pre-registered here as the m = 1 cell. A null from it is predicted by the lag mismatch and must not be read as evidence against lineage.

**Step 4, training-free on the Step-2 and Step-3 checkpoints.** Exact n-let admission (m-tuple equality plus W = 32 window plus NoRead) against full softmax: recall, NLL delta and positions scored.

### Kill criterion

All thresholds below are frozen before the runs.

**Bench, Step 2.** Primary cell: REHEARSE, key pieces ≥ 2, g ≥ 1, all distances, recall averaged over 3 seeds.
- **Kill ONL** if ONL-quat m = 3 is below 0.90 on the primary cell in either pattern, or does not beat F2 by at least 0.30 absolute there.
- **The F2 lag-mismatch prediction (G2) is falsified** if F2 itself reaches ≥ 0.90 on the primary cell. ONL is then redundant; ship F2 and stop.

**Claim test, separate from the kill.** The "canonical quaternion lineage" claim is retired (the mechanism is kept, the geometric claim dropped) if ONL-rand or ONL-learned matches ONL-quat within 0.03 on every cell over 3 seeds. ONL-ident is expected to fail; if it does not, the order-separation argument is wrong.

**Chat, Step 3.** Stop the line if the paired 2-seed ONL arm at the higher dose does not exceed both of the following on sieve-off MQAR by ≥ 15/109:
- the F2 arm at the same dose;
- the "none" arm at the same dose.

A dose-only gain of the same size reassigns the barrier to training data, not mechanism.

**Admission, Step 4.** If exact admission loses more than 2 points of bench recall, or more than 0.01 nats of NLL, on the matched cells, keep dense ONL and drop the D5 claim for this route.

### Expected result if right

**Bench (Step 2).**
- ONL-quat m = 3 reaches ≥ 0.97 recall on every REHEARSE cell (key 1–3 pieces, g 0–2, d16–d400), plus the held-out pairing class. This holds on 3/3 seeds and in both aaaaaa and rrarra, with no seed lottery.
- F2 matches it only at g = 0 with single-piece keys, and falls to near the R-recency level (~0.2–0.4) at g ≥ 1.
- ONL-ident fails on order-sensitive cells. ONL-rand is measurably worse than quat, or slower to lock in.
- BARE cells fail for every arm. That is an honest result: unbounded-gap binding without rehearsal still needs state, and it documents exactly what rehearsal buys.

**Chat (Step 3).**
- Sieve-off D19 MQAR rises from the 0–5/109 class to several tens of rows in the ONL plus 15% dose arm, concentrated in rehearsed replies.
- It clearly exceeds the F2 arm at the same dose, so the authored sieve's contribution is reproduced in-network for the first time.
- Base NLL is preserved within seed spread. The open panel is roughly flat (G5 unchanged).

**Admission (Step 4).**
- Positions scored per query fall from 384 to roughly |matches| + 33.
- The loss in recall and NLL is within noise.
- This gives the first fair D5 sparse-access measurement.

**Integration.** If all of this holds, the next steps are:
- the D11 port (ring buffer plus signed permutation);
- retraining the 96M rung with ONL from scratch;
- retiring the authored log sieve from the MQAR path. The store/compiler is kept for durable cross-session memory.

### Cost

**Engineering.** About 250 lines of Rust for the mechanism and tests, and about 200 lines for the bench extension: roughly 1 working day. No new dependencies and no Python.

**Bench compute (Step 2).** 36 runs × ~10–20 CPU-minutes = 6–12 CPU-hours, about 4–6 h wall on the local M1.
- RAM under 2 GB per run.
- New storage: a few MB per sealed root, under 200 MB total.

**Chat compute (Step 3), gated on Step 2.** 12 fine-tunes at 29M on the existing CUDA pod.
- Estimate: 6–10 GPU-hours, extrapolated from the 4k/12k fine-tune rungs, so it should be confirmed against the pod's cumulative ledger before launch.
- The GPU authorization gap noted in the synthesis must be resolved by the owner first.

**Admission and D11 work.** Step 4 is training-free (minutes on CPU). The D11 port is estimated at about 300 lines (from #1704's ~200 for m = 1, plus a longer ring) and is only started after Step 3 passes.

**What to stop now, to pay for it:**
- Score-geometry swaps (Lorentz/H4/Hopf/2I read codes), until key content is decided.
- Further Codex supplied-record cue-carrier fits on the critical path (#1705, 0–1/32 fresh).
- Ladder spend above 29M (96M showed no panel gain, and the bottleneck is not scale).
- Single-seed verdicts.
- Reading the in-flight lag-1 F2 pod A/B as decisive.
- Using the open panel as a gate for retrieval mechanisms.

### Risks

**1. Rehearsal is not adopted.** The model may prefer "It's {v}" replies (50% of MQAR_REPLIES), which ONL cannot answer by n-let match.
- Mitigation: report the form split. This is a training-data choice, not cheating, but it must be disclosed and checked against the rehearse+R-nlet reference rule, so that gains are not just the template.
- If BARE is required, ONL is insufficient and the binding-state problem (#1580's capture/hold) remains open.

**2. The query-side copy of the cue may itself fail.** The model may not reproduce the key pieces from the question. That is a short-range copy, but the pointer/copy path is capped by the `add_prefix_space:false` boundary (catalogue 8.3, 0/33 copy credit). Fix or measure that tokenizer boundary first, or the result is uninterpretable.

**3. Update facts.** For facts restated with a new value ("k is v1 … k is v2"), both positions match exactly. The latest-wins choice rests on the learned age bias, so the update/version cells must be measured; exact-store absence/eviction semantics are not touched.

**4. Lag-frame capacity.** The frame caps at 3 lags per 4-lane. Longer cues (keys of 4+ pieces plus a copula) may need 8-lanes (octonion Hurwitz–Radon ρ(8) = 8, E8-adjacent; H only) or a second head with a different frame.

**5. The skew argument may be irrelevant in practice.** It is exact only where q ∥ k for the same token. Learned W_q, W_k may make rand/learned frames equivalent. The planned controls decide this, and the result would retire the geometric claim but not the mechanism.

**6. Quantization fragility.** Near-one-hot selection may not survive GPTQ/4-bit keys; check fidelity on the bench checkpoints before the D11 port.

**7. Owner ruling.** "No RoPE" (09-29) may be read as covering fixed lineage operators; this needs an explicit owner ruling before anything is served.

**8. Over-reading.** Success is measured MQAR/in-context recall at the stated scope only. It does not establish general prose, reasoning, the open-panel plateau (G5 knowledge ceiling), energy savings, or any prime/zeta semantic advantage.

**9. Process risk.** The result must be propagated to #820, current-state and the Codex line immediately, to avoid a fourth rediscovery (lesson P2: ADR-0003 → #1580 → #1701).

### Red team, generalization lens: REFUTED

I checked the proposal at 71b54adf. ONL is a reasonable smeared-key/n-gram-head variant. It does not survive this lens as the solution to the barrier, for five reasons.

(1) It is standard attention machinery, not new geometry. ONL is dense softmax attention with fixed Q/K token-shift taps. The bench scores 2,841–8,564 positions per query token (PR #1701/#1704 tables). The construction is known:
- Olsson et al. 2022, "In-context Learning and Induction Heads", smeared keys;
- Primer, arXiv:2109.08668, depthwise conv after the Q/K/V projections. ONL is that design with frozen signed-permutation taps;
- H3's shift SSM, arXiv:2212.14052;
- RWKV token shift;
- the n-gram heads of arXiv:2401.12973, which the proposal itself cites.
PR #1704's own scope note already says F2 is "the standard previous-token / token-shift route that attention models use". The only geometric content is the choice of fixed taps.

(2) The 'canonical, exact no-leak' proof is overstated.
- x^T L_w x = 0 removes only the term where the two vectors are identical. The cross term is <q_X, L_w k_X> with q = W_q u and k = W_k u, two different learned projections. It vanishes only if W_q is a scalar multiple of W_k, and nothing enforces that.
- With m = 3, Q_t contains both q_t and L_j q_t, giving m^2 different-token cross terms against m aligned ones. Under the L2 read, |K_s|^2 adds key-dependent cross biases.
- A Haar-random SO(4) has expected trace 0, so 'random R leaks tr(R)/4' is a variance argument, not a bias. Learned W_q/W_k can absorb any fixed orthogonal frame.
- {1,i,j,k} is not 'mutually anticommuting': 1 commutes with everything. Only i, j and k anticommute.

(3) The primary pass cell is a known trivial rule. The kill cell is REHEARSE, which ends in '{k} is'. That is exactly the R-nlet leak that revision 2.1 removed because an untrained two-word rule answers it (milestone_world_v2.rs:47-55, 997-999). The kill criterion only requires beating F2 and reaching 0.90. It does not require beating the rehearse+R-nlet reference, which is merely 'reported'. A pass would show the network re-learning an authored rule that the exact sieve already executes at 105/109. That is no new capability, and nothing about composition, reasoning or the 43–46/232 open-panel plateau, which the proposal concedes it will not move.

(4) The chat transfer story breaks on a known tokenizer defect, and the proposal never connects the two.
- The MQAR replies are '{k} is {v}.' (milestone_world_v2.rs:1022), so the rehearsed key sits at the start of the reply.
- Assertion keys always appear mid-sentence after ': ' or ', ' (:1420-1435), as leading-space tokens.
- Under the #1017 tokenizer (add_prefix_space=false), a reply-initial word's first token never occurs in the user's text. The catalogue gives 'm'=79 against 'Ġm'=283 (retired-mechanisms-catalogue-2026-10-01.md:255). This is the cause of the persistent 0/33 copy panel.
- So the rehearsed m-gram is a different token-id tuple from the assertion's m-gram. Exact n-let admission (Step 4/D5) therefore matches nothing for reply-initial rehearsal, because the first key piece differs and BPE may split the rest differently. The soft ONL match also depends on the model learning that the two token forms are equivalent.
- The synthetic Step-1 bench uses disjoint integer tokens and cannot see this, so a bench pass would not predict chat transfer.

(5) The query-side lineage ONL adds is largely present already.
- The r layers' width-4 causal conv feeds both W_q and W_k in production patterns.
- PR #1704 measured rararr without the shift at 1.000 on all buckets (1022/1024 held-out). rrarra without the shift reached 0.9995 at seed 2.
- In a stack with two or more reads, F2 plus a lower-layer read can already carry the key across a copula by ordinary induction-head composition.
- 'F2 fails at g ≥ 1' is an untested hypothesis. #1704 itself attributes the chat 3/109 possibly to dose (about 0.8% MQAR tokens), and the pod A/B is pending.
- Pre-declaring a chat-side F2 null as 'predicted by lag mismatch' lets the proposal claim any outcome as support.

Net: the proposal mostly re-derives a known transformer primitive with a fixed frame. It targets a recall pattern an authored rule already solves. Its in-chat route and D5 admission depend on a known tokenizer defect it does not resolve. It cannot plausibly break the coherent-language / composition / reasoning barrier.

**Killing experiment:** Three experiments, in cost order. Each one decides the proposal on its own.

(a) Tokenizer check. Training-free, minutes of CPU, no code beyond a read-only script against the existing tokenizer.
- Take the 109 D19 sieve-off MQAR rows. Tokenize the gold rehearsed reply '{k} is' at reply start and the assertion span '{k} is' as it occurs mid-sentence.
- Count rows whose last m = 2 and m = 3 token-id tuples are identical.
- **Kills:** if the count is near 0, which is expected under add_prefix_space=false, the exact n-let admission story and the rehearsal route are dead in chat until the tokenizer boundary is fixed.

(b) The proposal's own Step 2 with one change: add the existing production patterns rararr and rrarra with the conv, and the arm 'F2 + none-else', at 3 seeds, on the copula/multi-piece REHEARSE and BARE cells. Freeze two kill rules before running:
- **Kills (redundant):** if F2, or no shift at all, in rararr/rrarra reaches ≥ 0.90 on REHEARSE with g ≥ 1 and ≥ 2 key pieces, ONL is redundant.
- **Kills (no gain over the rule):** if ONL-quat does not beat the untrained rehearse+R-nlet reference rule on any cell, it adds nothing over an authored rule.

(c) The frame-specificity control inside (b): quat {j, k, i} vs Haar-random orthogonal per lane vs learned W_prev, 3 seeds.
- **Retires the geometric claim:** if they are within seed spread on every cell, the 'canonical quaternion lineage' claim is retired permanently.

Whatever the bench shows, the chat-level arbiter is the already-pending pod A/B. If F2 at a raised MQAR dose (about 15%) recovers sieve-off MQAR, the barrier is data dose, not lag order.

**Salvageable part:** Four parts are worth keeping.

(1) The bench extension in Step 1: multi-piece BPE-like keys, a shared copula of g tokens, values of 1–2 pieces, REHEARSE vs BARE query forms, and an R-nlet/rehearse reference rule.
- Its parity freeze and 3-seed rule, and its use as a gap-robustness test of the shipped F2, are a cheap, honest instrument that directly measures whether F2 transfers beyond gap-0 geometry.

(2) The frame-control design: ident vs Haar-random orthogonal vs learned vs quaternion lag taps.
- This is the first clean falsification test of whether the quaternion j in F2 is structurally special or just a token shift. The project should run it whatever happens to ONL, and report the result as such.

(3) The requirement to measure attention mass on one position and then test exact top-k / n-let admission against dense softmax.
- This is a fair first D5 sparse-access measurement on the bench, provided admission keys use a tokenizer-normalized identity. The leading-space and reply-initial forms must map to one id; it remains an identity, not a metric.

(4) The process points:
- Treat the reply-form split (rehearsed vs 'It's {v}') as a disclosed training-data choice.
- Report the rehearse+R-nlet reference so gains are attributed correctly.
- Fix the add_prefix_space reply-initial boundary first. It independently caps copy (0/33), pointer and any lineage-match route in chat, and it is probably a higher-leverage fix than any new read mechanism.

### Red team, cost-serving lens: REFUTED

This review used the D11 serving and laptop-cost lens. The bench-side mechanism may work, but the serving and cost claims that sell it do not hold. The proposal does not hold together as written: the D5 claim and the "exact, canonical" claim fail, and the admission design has a mismatch between training and serving. Checked at 71b54adf.

1. **The D5 claim is a category error.**
   - DECISIONS.md:242-256 defines D5 as per-token parameter sparsity: "if serving touches all entries of a learned weight store per token, it is a matrix product". The recorded non-compliance is 611,814 inspections per step.
   - Exact n-let admission prunes KV positions. It does not prune weight rows.
   - At the 96M rung (width 1024, pattern rrarrarrarrarr, so 4 `a` layers, context 384), the read costs about 4×384×1024 ≈ 1.6M lane ops per token. About 96M weight reads stay dense: SwiGLU, embedding and LM head.
   - So admission removes at most about 2% of the per-token work and leaves D5 non-compliance unchanged. Calling it "the first D5 sparse-access path whose loss can be bounded" misstates the target.
   - The laptop cost or energy story gains nothing measurable from this mechanism.

2. **"Exact zero leakage" is not exact in the score that is actually served.**
   - The skew argument xᵀL_w x = 0 applies to a dot product with q ∥ k.
   - The production and D11 read is L2 distance: kernels.rs:834-841 computes `sqrt(max(|q-k|²,1e-7))`.
   - The expansion |Q−K_s|² includes |K_s|². That term carries the cross terms 2⟨k_s, L_u k_{s−l}⟩, which are nonzero and depend on position for any two different tokens. They add a key-norm bias that is not order-neutral.
   - The query/key cross terms ⟨q_a, L_w k_b⟩ vanish only if W_q u ∥ W_k u. Separately learned W_q and W_k give no reason for that.
   - After GPTQ/4-bit quantization of W_q and W_k, even an idealised alignment is broken.
   - So "exact order separation at the match point" is a hypothesis about learned weights, not a proof (P). The Hurwitz–Radon "canonical" framing is decorative until it is measured.

3. **Exact n-let admission creates a mismatch between training and serving.**
   - Training uses dense softmax. Serving would restrict the read to exact token-tuple equality plus a window of 32.
   - The proposal's own decode path relies on the model rehearsing the key in its reply. The rehearsed key and the asserted key often have different BPE ids: the `add_prefix_space:false` boundary at the start of a reply versus mid-sentence, catalogue 8.3, where copy credit is 0/33.
   - In that case the tuple compare rejects exactly the matches the soft learned read would make, and recall falls to the window or NoRead.
   - Every `a` head also serves general language. Restricting all of them to n-let matches is likely to break the 0.01-nat NLL bound.
   - The "absence proven by tuple inequality" is absence of a token tuple, not absence of the fact.

4. **The integer port is not free.**
   - K_s and Q_t become sums of up to 4 permuted int8 vectors, which widens their range by about 2 bits.
   - The D11 L2 kernel and tables either widen, or keys are requantized after the sum. Training never sees that requantization: it applies lineage to float k before any quantization.
   - The repo already refuses even the m=1 form: stack_export.rs:346 and :991 ("no D11 served form").
   - Near-one-hot attention (0.99–1.00 mass on one position under F2) is precisely where small quantization shifts in score flip the argmax. Quantization fidelity on even the F2 checkpoint has not been measured.

5. **No measured cost, by the proposal's own admission.**
   - J/token is "only when measured on the M1".
   - The only D11 throughput on record is 32.5 ids/s at 4 threads at 96M (#1691). It is dominated by dense maps that ONL does not touch.

**Killing experiment:** All of these are training-free and run on the existing #1704 F2 bench checkpoint, plus the Step-2 ONL checkpoints once they exist.

**(a) Exactness probe.**
- On deployment-shape MQAR sequences, log the per-position decomposition of the L2 score: the content term, the aligned-lag terms, the query/key cross terms, and the |K_s|² cross terms.
- The "exact/canonical" claim dies if the misaligned-lag cross terms are not ≤5% of the aligned terms on matched cells.
- It also dies if quat does not beat the random-SO(4) frame on that ratio.

**(b) D11 fidelity.**
- Simulate GPTQ 4-bit W_q/W_k plus int8 activations, fed through the exact integer `stack_l2_distance` (kernels.rs:841). Run it once with keys widened and once with keys requantized after the lineage sum.
- Re-score bench recall.
- Kill the D11 serving story if recall drops by more than 0.05 from float in any distance cell.

**(c) Admission under BPE variation.**
- Build cells where the rehearsed key is tokenized with a different leading-space or casing variant from the assertion, using the real tokenizer with add_prefix_space:false.
- Compare exact-tuple admission against dense ONL.
- Kill the admission/D5 claim if admission loses more than 2 points of recall or more than 0.01 nats of NLL. Also kill it if the read layers are under 5% of the measured per-token ops or bytes at 96M.
- Counting ops/bytes per token across the D11 stack, with and without admission, settles the cost question directly.

**Salvageable part:** The learned bench mechanism is worth running as a dense offline experiment with the planned controls. It adds query-side offset lineage and m ≤ 3 lags beyond F2's m=1 key shift.
- Step 1, the deployment-shape bench (multi-piece keys, a copula gap, REHEARSE vs BARE queries), is the most valuable part. It tests whether F2's gap-0 success transfers.
- Signed-permutation lag operators are D11-friendly in principle: a lane reorder, sign flips and adds, no multiply.
- Re-scope the claims:
  - Drop the D5 claim entirely.
  - Treat "exact canonical quaternion lineage" as a hypothesis the rand/ident/learned controls must decide.
- If exact admission is pursued, admit on a tokenizer-normalized word-level n-let identity rather than raw BPE id tuples, and measure it against the soft read.
- Put the real D5 effort into weight-row routing, where the per-token cost actually sits.

### Red team, process-evidence lens: REFUTED

ONL is refuted as the next critical-path step. The checks below were run read-only at 71b54adf.

1. **The lag-mismatch diagnosis rests on a false premise about what keys contain.** The proposal says F2 "tags every value with j·k('is')", so it must fail at deployment, where a copula sits between key and value.
   - In code, the key is `W_key·u` (geometric_stack.rs:1845-1860, `previous_key_channel` at :8936). Here `u` is the residual stream, not a raw token embedding.
   - In every production-like pattern (rrarrarrarrarr; the stand-ins rrarra and rararr), earlier `r` layers apply a width-4 causal conv. So at the "is" position, `u` already carries x_{t-1..t-3}.
   - This means F2 plus the conv already places "ak is" into the key at the value position, and the query side already has its own conv lineage.
   - PR #1704 measures that route directly: rararr *without* the shift scores 1.000 at every distance on seed 1, and rrarra without it reaches 0.9995 on seed 2.
   - ONL's query-side and m≥2 lineage is therefore new only in `aaaaaa`, which is not a production pattern. The claimed "missing piece" is a redundant fixed form of a route the stack already has. That route is unreliable across seeds, and F2 already fixed that unreliability on the bench.

2. **The "exact zero leakage" proof does not apply to the model as built.**
   - x^T L_w x = 0 holds only when the same vector sits on both sides. The model's cross terms are q_a^T L_w k_b = u^T W_q^T L_w W_k u with W_q ≠ W_k, and these are generically nonzero.
   - The read is L2, not dot. ||Q−K||² adds lag-cross terms from ||Q||² and ||K||².
   - The retained content term ⟨q_t, k_s⟩ rewards positions holding the *current* token. In REHEARSE, the current token is "is", so the content term favours the copula position, one off from the value.
   - Hurwitz–Radon ρ(4)=4 is true but not load-bearing. The proposal concedes this itself in risk 5. The "canonical lineage" claim is therefore a hypothesis, not a P.

3. **The primary bench cell is the leak the project deliberately removed.** milestone_world_v2.rs:44-55 and :997-999 record that revision 2.1 dropped every query ending in "{k} is", because the untrained R-nlet rule answers them.
   - A ≥0.97 result on REHEARSE cells measures a task that a two-word rule already solves.
   - It cannot show that sieve-off chat recall (3/109) is a mechanism failure rather than a dose or decoding failure.

4. **The first experiment cannot decide the barrier, and it pre-empts the experiment that can.**
   - Step 2 is a 36-run synthetic grid, another bench loop.
   - #820 (2026-10-05T00:36Z) has already reopened the cause of 3/109 as possibly MQAR dose (about 0.8% of tokens), and the in-flight pod A/B (F2 off vs `key_shift=add`, sieve-off D19 MQAR) tests exactly that.
   - The proposal pre-registers in advance that a null from that A/B "must not be read as evidence against lineage". That is an unfalsifiability clause, which is the endless-loop pattern under review.

5. **Evidence base.**
   - F2 is measured at 1.36M parameters, gap 0, single-token keys, on 1–2 seeds.
   - ONL extrapolates to multi-token, copula and decode-time rehearsal without any measurement.
   - Rehearsal is assumed (2 of 4 MQAR_REPLIES at :1022) but never shown to be emitted by the existing checkpoints.
   - The tokenizer copy boundary (0/33, catalogue 8.3) is named as a risk but not resolved before the run.

**Killing experiment:** **Part A: training-free probe, about an hour of CPU, on existing artifacts.**
1. Take the current 29M lr5e-4 and 96M production checkpoints, plus the F2 `key_shift=add` checkpoint from the in-flight pod A/B once it lands.
2. Run the 109 D19 MQAR rows with the sieve OFF in three conditions:
   - (a) free-run, recording the reply-form split (does the model emit "{k} is" at all?);
   - (b) teacher-force the question plus the gold "{k} is" (BPE-exact, including the leading-space issue), then score argmax/NLL of the first value piece;
   - (c) the same as (b) with the copula replaced, as an order control.
3. Read the outcome:
   - If (b) is already high on the conv-bearing production checkpoints, or on the F2 checkpoint, n-let matching exists and the barrier is rehearsal/dose. **ONL is dead.**
   - If (b) is low only on the copula and multi-piece rows, and F2 does not help, go to Part B.

**Part B: the proposal's own falsifier, run cheaply first.**
1. Extend mqar-bench with copula g∈{1,2} and 2–3-piece keys.
2. Run only F2 on `rararr` and `rrarra`, 2 seeds each: 4 runs, about 1 h on the M1.
3. Read the outcome:
   - If F2 plus the conv reaches ≥0.90 on the primary cell (the proposal's own G2 falsifier), **ONL is redundant**. I predict this, given that the conv feeds lineage into `u` (#1704).
   - Only if F2 fails there is the 36-run ONL grid justified.

**Salvageable part:** 1. **Deployment-shape bench extension.** Add copula, multi-piece keys and values, REHEARSE/BARE query forms, and an explicit rehearse+R-nlet reference rule, with criteria frozen before the run. This is a real gap: the current bench is gap-0, single-token (mqar-bench.rs:15-19).
2. **MQAR dose arm (~2% vs ~15%) and reply-form split in the chat transfer.** Together these separate data from mechanism.
3. **Insistence on 3 seeds.** No single-seed verdicts.
4. **Not gating retrieval on the open panel**, and stopping score-geometry swaps until key content is settled.
5. **Exact n-let/ordered-tuple candidate admission (ADR-0003) as the D5 sparse-access measurement on F2 checkpoints.** Admission stays separate from ranking, and absence is proven by tuple inequality. F2 already concentrates 0.99–1.00 of read mass on one position (#1701 probe), so this is the first fair top-k/admission test, and it is training-free.
6. **Cheap frame controls (ident / rand / learned vs quaternion j) on F2 itself.** These would settle whether "canonical quaternion lineage" is a geometric claim or merely a token shift. #820 already scopes F2 as "the standard previous-token / token-shift route".
7. **D11 signed-permutation plus ring-buffer serving form.** This is needed for F2 regardless.
