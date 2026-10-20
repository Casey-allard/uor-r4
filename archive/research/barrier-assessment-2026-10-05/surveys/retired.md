# Retired, dormant and exploratory mechanisms, re-read after the previous-token key channel (#1701)

Source: read-only, `origin/main` `71b54adf`. All numbers come from the cited files and PRs. Labels: **Measured** means a result at the stated scope. **Proof** means it follows algebraically. **Hypothesis** means it has not been tested.

## 0. What #1701 establishes, and the test I applied to each retired mechanism

**What F2 changes.** PR #1701 (merged `3aa8ba16`) and #1704 (`2cdae05a`) add one change to the read key: `k_t ← k_t + j·k_{t−1}`. Here `j·` is the signed permutation `(a,b,c,d)→(−c,d,a,−b)`, and the change adds no parameters.

**Measured scope.** The bench is `examples/mqar-bench.rs` from #1698:
- 6 layers, width 128, 4 heads, context 512, vocabulary 512, 1,800 steps of batch 8, CPU.
- Arm A (all reads, l2 score) recalls 0.807 / 0.176 / 0.010 / 0.029 at d16 / d64 / d200 / d400. With F2 it recalls 1.000 at every distance and gets 1024/1024 on held-out pairings, at seed 1.
- `rrarra` without F2 scores 0.007 at seed 1 and 1.000 at seed 2. With F2 it scores 1.000 at both seeds (#1704).

**Why it works (measured, and partly provable).** The #1701 probe shows that arm A's heads never put more than uniform weight on the key position. They read a "bag of values" (0.045 mass at d200). The PR states the mechanism: *"Layer 0 cannot do better. Its keys know only the current token, and the value position's own token says nothing about which key preceded it."* This is the previous-token half of an induction head (Olsson et al., arXiv:2209.11895). The project's own literature brief already cited that paper on 16 September (`docs/integration/review-2026-09-16/05-literature-brief.md:88`).

**Limits of the F2 result that every revisit must respect:**
1. In the bench the value sits at `p+1`, exactly one token after the key. In the m-world MQAR the assertion template is `"{k} is {v}"`, under the #1017 BPE tokenizer (`milestone_world_v2.rs:984–1022`). There the value's predecessor is " is", not the key. So **one-step lineage does not reach the key**, and two-step lineage or a convolution chain would be needed. That is a hypothesis to test, not an assumption.
2. The bench uses synthetic tokens, one task family, and at most two seeds per arm.
3. D11 serving is not built. Every integer path refuses a key-shift model (#1704 §3).

**The test I applied to each entry:**
- Did its keys at the matched position carry predecessor identity?
- Was its decisive failure a matching or binding failure, as opposed to cost, storage or decode?
- Does the negative's scope (size, seeds, data) leave room for a re-test?

**Revisit grades:**
- **R1**: the failure is plausibly the missing lineage channel. Re-test with `key_shift=true`.
- **R2**: lineage newly enables the mechanism, usually by making attention near one-hot. Re-test after R1.
- **R3**: not lineage-related, but the negative was scoped narrowly. Revisit on its own grounds.
- **C**: clean retirement. Do not revisit for lineage reasons.

---

## 1. Retrieval and attention mechanisms most likely confounded by missing lineage (R1)

### 1.1 A1 / D18 outcome D: Lorentz read, flock, pointer and T arms on M-world v2 (R1, the highest priority)

**What it was.** The geometric stack with a Lorentz or Dot read score, an optional flock top-k, and an optional dot-scored pointer head. It was trained on M-world v2 with the #1017 tokenizer (`crates/uor-r4-training/src/milestone_world_v2.rs`, `examples/m-world.rs`).

**Decisive measurement.**
- Development cell, 300 conversations, seed 9101, about 2.1M parameters, 2,590 steps, **one training seed per arm**.
- P (Lorentz), F7 (flock) and T (transformer control, no pointer) all score MQAR about 0.009 and **0 at d16**. So does T at 2× steps.
- The pointer arms score 0.28–0.44, which is within ±0.05 of the untrained R-recency rule (0.36).
- The kill rule fired: every arm was below 0.5 at d16.
- Sources: `DECISIONS.md` D18 §2–§3, `docs/research/retired-mechanisms-catalogue-2026-10-01.md` §1.6, #1552 at 01:47:44Z and 01:48:29Z.

**Why this looks like the lineage confound:**
- The failure signature matches #1701 arm A: recall near zero, and a 1/N decay toward the latest value ("copying the latest value", catalogue §1.6).
- The m-world stack had `r` layers with a width-4 causal convolution in place since #1414 (`geometric_stack.rs:7032`). #1704 measured that this route to predecessor identity is "found only sometimes". It failed at seed 1 at 1,800 steps, and A1 had only 2,590 steps at one seed.
- The m-world key–value gap is at least 2 tokens ("k is v"), and BPE splits made-up keys such as "bol" and "tamir" into several pieces. That raises the lineage depth needed beyond what F2 supplies.
- The transformer control without a pointer also scored 0. This is consistent with a 2-layer previous-token→induction circuit failing to form at this dose, not with geometry failing.

**Verdict.** The kill was clean on the letter of its rule. Its causal reading ("no learned retrieval at this scale") is untested against the lineage hypothesis.

**Re-test design (one-arm change, two seeds):**
- `rrarra` with `key_shift=true`, and a 2-step variant (`k_t + j·k_{t−1} + k·k_{t−2}`, if added).
- Run on the frozen development-phrasing × development-value cell with the sieve off.
- The instrument freeze (R-nlet and R-recency both below 0.6) still applies. Under D19 §3 this is a materially changed successor.

### 1.2 Lorentz versus Dot read parity: undecided at two seeds (R1)

**What it was.** `ReadGeometry::Lorentz` (`uor-r4-integer/src/lorentz.rs`, `joint_model.rs:29–37`), using the Minkowski score `−q0k0 + q·k` and arcosh.

**Decisive measurement.**
- D17 v2 pairs: seed 1 gave MQAR d=+0.018 and open-relation d=−0.096. Seed 2 gave MQAR d=+0.055 and open-relation d=+0.288.
- That required a third seed, and D18 forbade new training (catalogue §1.6, Part 8 item 8.2).
- Separately, the hyperbolic-cycle-3 learner showed Lorentz ahead by 0.018 nats/token in 3 of 4 seeds on repository code (`hyperbolic-cycle3-2026-09-26.md` §0).
- On a synthetic hierarchy, hyperbolic keys reached 77.5% against 0.4% for dot (`hyperbolic-cycle2-2026-09-26.md` §0).

**Lineage flag.** A score geometry cannot matter when no key encodes the bound pair (the "bag of values" regime). Both parity seeds were measured in that regime, so the comparison is not interpretable until keys bind.

**Re-test.** The paired Lorentz and l2 arms with `key_shift=true` on `mqar-bench` at d400 and d1000 (context ≥1101), then on m-world. A hierarchical key `k_t + j·k_{t−1}` is a natural use for hyperbolic radius (cycle 2). That is a hypothesis.

### 1.3 Lorentz-scored pointer, the arm D18 specified but never ran (R1)

D18 §10 says *"Every pointer arm run used the default dot-scored pointer"* (catalogue Part 8 item 8.3). The pointer's credit was also capped at **0/33 copy** by the #1017 `add_prefix_space:false` boundary (`m` = 79 against `Ġm` = 283). That tokenizer cap persists independently of lineage and must be fixed first. Otherwise the pointer re-test is uninterpretable.

### 1.4 KVAR Q8 relative-energy residual (R1, the clearest structural case)

**What it was.** A learned integer residual over the exact directed Q8 signed action between query and stored key, on the KVAR store (`crates/uor-r4-core/src/bin/kvar-recall.rs`, `bin/support/kvar_relative_energy.rs`).

**Decisive measurement.**
- Two frozen base seeds, 204 held episodes each.
- Q8 minus ordinary was −0.1337 bits [−0.371, +0.098] at seed 1 and exactly 0 at seed 2, where the kernel learned zero. The pre-registered gate required ≥0.5 bits. Rejected (`kvar-relative-energy-result-2026-09-24.md`).

**The document's own diagnosis.** *"At the read location, the queried key and stored key coincide, so their relative action is always identity."* That is a **proof** that the residual could carry no information on that panel.

**Lineage flag.** With a lineage key, the relative action between the query's frame and the stored predecessor's frame is no longer identity. F2's `j` is itself a Q8 element. So the mechanism was tested in the one configuration where it was degenerate by construction.

**Re-test.** A Q8 or 2I relative code on (current, predecessor) key pairs, against a matched ordinary cyclic C8 arm, on a KVAR variant where the key is two tokens long.

### 1.5 Native owner-plus-object query residual, #1069 (R1, a historical precursor of lineage)

**What it was.** In the "R4 zoology" ordinary-softmax cell (2 layers, 1 head, width 64), the query embedding at position 37 got the owner word from position 35 added: `x37 = E(t37) + P37 + E(t35)`. This is lineage at a fixed position, on the query side only (`docs/r4_zoology_joint_query_1069.md`).

**Decisive measurement.**
- Construction accuracy rose from 45.59% to **50.27%**.
- Owner-changing pairs improved from 47/2,048, the dominant error class (67.5% of in-history errors were wrong-owner/same-object).
- It was rejected on an **object-pair preservation floor**: `JOINT_QUERY_PRESERVATION_MISS`, with one fit and one seed.

**Flag.** This is the same binding defect F2 fixes: the key could not say "the object whose owner is X". It was repaired by a positional hack, applied asymmetrically to the query only. F2 applies lineage to every key, symmetrically.

**Re-test.** The #1063/#1067 English-binding population with key lineage in place of the owner residual. This is an ordinary cell, so it is a mechanism-isolation test and not a geometric claim.

### 1.6 Direct geometric attention V3 and gated-delta R4, #999 (R1)

**What it was.** Mixed-gauge H4 projection and connection attention over cumulative R4 frames (`crates/uor-r4-core/src/helm_d_r4_attention.rs`, `docs/direct_causal_geometric_attention_973.md`, `docs/geometric_gated_delta_retention_973.md`).

**Decisive measurement (PR #999).**
- Geometric 3/12, plain fixed-tangent 12/12, **current-only 6/12**, order-shuffled 5/12.
- Gated-delta geometric 16/28 and 55/112, against plain 23/28 and 98/112.
- These are tiny construction panels, one fit each.

**Flag.** A "current-only" arm beating the geometric connection arm means the connection was not supplying useful predecessor information. The panels are too small to conclude more.

**Revisit.** Low cost. Fold it in as an arm of the `mqar-bench` registry (`ContextArm`). Do not revive the HELM-D harness.

---

## 2. Mechanisms that lineage may newly enable (R2)

### 2.1 Flock top-k, R4RouteAttentionV1 top-M, and the VP-tree (R2: the D5 sparsity contest becomes meaningful)

**What they are:**
- `crate::flock` (`crates/uor-r4-training/src/flock.rs`): sink + window + exact top-k over Minkowski rank scores.
- `R4RouteAttentionV1` (`crates/uor-r4-graph-runtime/src/route_attention.rs`, certify/format twins): masked XOR+popcount distance over 288-bit codes, then top-M, with no multiplier, P-4 scanned and dormant (`model/ledger.toml` `r4-route-attention-dormant`).
- The VP-tree exact nearest-neighbour index (`crates/uor-r4-graph-runtime/src/vp_tree.rs`), valid only for a single shared mask.

**Decisive measurements:**
- Flock F7 scored 0.009 MQAR in A1, with one seed.
- Route attention's real-teacher stage on SmolLM2 traces (62,875 records) returned **FAIL (instrument vacuous)**. The shift-by-one N2 null reached 0.292 against 0.396 fitted, and 119 of 120 heads were vacuous (#605/#804, `ledger.toml:117`).
- The VP-tree is a serving heuristic, slower than a linear scan below 512 nodes.

**Lineage flag (measured precondition).** Under F2 the solving head puts **0.99–1.00** of its mass on one position (#1701 probe table). Top-1 or top-k selection is therefore nearly lossless, and a sparse or indexed read becomes a fair test for the first time. Under arm A, with 0.045 mass, any top-k discards the "bag of values" the model was actually using. So flock F7's 0.009 tells us nothing about sparse selection.

**Re-test (training-free, post hoc on an F2 checkpoint):**
- flock `k ∈ {1, 7, 16}` and route top-M over binarized lineage keys;
- report positions scored per query token (already in the bench schema) and recall.

This is the D5 "geometric router versus LSH or learned-kNN at matched access budget" contest that catalogue Part 8 item 8.8 says has never run.

### 2.2 `orthant64` and `recent64` bounded admission, and `exact_cache64` (R2/R3)

**What they are.** Sign-pattern partition admission of at most 64 events (`joint_admission.rs`).

**Decisive measurement.** On the 32-row complete-answer panel, orthant64 lost on answers but beat its parent on NLL. `recent64` was the best same-weights policy (2.119 / 2.109) and was withdrawn by D9 before any training (catalogue §2.1, §2.2, Part 8 item 8.1).

**Lineage flag (hypothesis).** An orthant of `k_t + j·k_{t−1}` partitions by (token, predecessor) sign pattern. That is a hashed-bigram bucket, much closer to the content of an exact n-let address than the old state-derived keys were.

**Re-test.** After 1.1, apply orthant admission to F2 keys and compare it with recent64 at equal access.

### 2.3 Prime router: semiprime experts, ordered n-lets, gcd sieve (R2, and a conceptual unification)

**What it is:**
- ADR-0003 (`docs/adr/0003-fixed-zeta-prime-route-attention.md`): each token is a prime; a *transition* is the square-free semiprime of **adjacent** atoms; ordered n-lets carry order; `p²` self-loops are kept.
- In the stack, `stack_prime_route.rs` keys the relation store by the semiprime of the clause's two *key atoms*, which is role-agnostic by design.

**Decisive measurement.**
- #958 stages 1–3 passed for mechanics only. Stage 4 ("no manifest-bound compiler from arbitrary prompt tokens into those addresses") was **NOT_RUN** (`docs/prime_route_attention_qualification_958.md:25–27`).
- Status: `RETAIN_STORAGE_RECALL_ONLY`.

**Lineage flag (proof of correspondence, not of benefit).**
- The adjacent-token semiprime `p_{t−1}·p_t` is exactly an exact, unlearned previous-token key. The project had lineage in its *exact* address system from August.
- The router failed for want of a language-to-address compiler, not for want of lineage.
- A semiprime is **commutative**: it identifies {a,b} and loses order. Ordered n-lets restore order. F2's `j` does the same thing in the learned path: `k_t + j·k_{t−1} ≠ k_{t−1} + j·k_t`, because left multiplication by `j` is not symmetric. So "canonical token lineage" is the learned, ordered, chiral analogue of the router's n-let edge.

**Revisit.** Use F2 keys as the learned compiler that #958 stage 4 lacked: map a lineage key to the nearest registered n-let address. This is the most direct way to join the exact sieve with the network. It is a hypothesis.

### 2.4 Fixed zeta-zero phases (R2 as a positional basis)

**What it is.** Word vectors `sin(ln p · γ_i)` over 512 zeros (`uor-r4-core/src/zeta_zeros.rs`, `uor-r4-router/tests/zeta_state_retrieval.rs`).

**Decisive measurement.** As a predictive feature, "no predictive gain" (`geometric-toolbox-2026-09-28.md` §C). The #434 retrieval work is wiring and de-banding history, not a semantic test.

**Lineage flag.** #1701 §F3 describes, but did not build, a relative rotation `ρ_l^t` with angles taken from the fixed zeta phases. With it, `⟨ρ^t q, ρ^s k⟩` depends only on `s−t` (**proof** in the PR). F2 is a single fixed step of that construction, applied only to the predecessor.

**Revisit.** A zeta-phase relative-rotation arm against F2, on d1000. Note that the owner's "no RoPE" direction (memory note dated 09-29) would need an explicit ruling. This is an exact quaternion table rotation, not learned RoPE.

### 2.5 Finite-group tracking lanes, B1 (R2)

**What it is.** `h_t = M[token]·h_{t−1}` over 2I or reflection pairs (`b1-finite-group-lanes-2026-09-27.md`).

**Decisive measurement.**
- Stage A: 17 of 18 runs tracked A5 exactly at length 4,096.
- It closed on the text-NLL gate: six-seed mean +0.018 against 0.05, with one seed over. The baseline's own seed spread was 0.063 (catalogue §4.1).

**Lineage flag (proof of relation).** F2 is the depth-1 truncation of an ordered group product over the history. B1 lanes are the unbounded-depth version.

**Revisit.** Use lane state (or a 2–3-step window product) as the *key lineage channel* rather than as residual text lanes. This also answers the "k is v" gap problem in §0.1 with exact one-byte state. It is a hypothesis.

### 2.6 VSA role binding (R2)

**Decisive measurement.** About 0.0013 bits/byte on language modelling, "within noise" (toolbox §C), at the Sep 19 model and one dose.

**Flag.** Permute-and-add sequence encoding (Kanerva) is the same algebraic form as F2. VSA was scored on aggregate bits, never on keyed retrieval. Re-test it as a key code on `mqar-bench`.

---

## 3. Memory-address and compiler lines: lineage is a candidate, not the main suspect (R3)

**D2/AERM exact store and G v1 read** (`stack_aerm.rs`, `examples/aerm-probe.rs`):
- D2 scored 1.000 on every in-distribution class at 3 seeds and failed an unsatisfiable 0.30 margin gate.
- G v1's held-out failure was `Unavailable` (460/484/519) on the write/key side (`d2-aerm-probe-result-2026-09-28.md`, `g1-always-on-read-result-2026-09-28.md`).
- G-binding masking alone moved held-out Updated from 0.158 to **0.967** at seed 1 (`g-binding-result-2026-09-29.md`).
- The binding failure was fixed without lineage. Revisit the margin gate and the +0.119-nat guard on their own grounds.

**G v2 2I relation-atom keys** (`examples/aerm-keys.rs`, `docs/evidence/g2-key-probe-2026-09-29.md`):
- Held-out key accuracy at split 4 was 0.327 (2I) against 0.491 (softmax), with in-distribution about 0.98 and a frozen S4 trunk, one run (`keys-9`).
- The documented failure is held-out relation transfer from a single-position linear readout.
- A lineage-trained trunk could change which positions carry relation identity. That is a hypothesis, and lower priority than 1.1.

**E3/v19–v25 relation compiler** (`relation_compiler.rs`, #1552):
- Linear probes on frozen trunk features fail on held-out phrasing.
- Same note as G v2: re-probe on an F2 trunk once one exists. That is cheap.

**`joint_model` copy mixture and the 2I read kernel (5.4)** (`joint_model.rs:1600`):
- Keys are `read.key(normalized(state))`. The recurrent state retains the previous token at 97.84% and the one before at 74.86% (catalogue §7.4). So lineage was present but entangled with decay.
- Read localization (5.5) found a READ_RANKING signature.
- The 5.4 HARM verdict lacks its attribution arm C.
- These are R3 items: the recommended oracle re-rank intervention and arm C remain the decisive missing tests.

**Direct two-token channel (catalogue §7.2)** (`ordinary-lexical-audit-2026-09-23.md:108–126`):
- Measured: +0.469 bits worse, with seed 13.
- **This must not be cited against F2.** It placed the last two tokens, "with the retained fixed order rotation", in the *readout* at the current position. F2 places lineage in the *key matched by later queries*. The math is similar and the role is different, and the document itself says it does *"not … retire the mechanism family."*

**Individual-history row (catalogue §7.1).** A sub-ε effect (0.007 bits) with a defective comparator. It is unrelated to lineage and remains R3.

---

## 4. Clean or orthogonal retirements (C, or not lineage-relevant)

**Peter–Weyl harmonic class kernel** (`docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`):
- Five levers were ruled out, the code was reverted, and the 9-class result was kept.
- The ceiling is "one linear map over one read vector decoding 120 classes". That is a readout limit, not a key limit. **C.**
- The **Track B harmonic arms** (branch-only commits `05a909d3`, `9cf381af`, `648ba939`) were parked under D18 §6 by owner direction, unmeasured against Taylor-2 (D16 item 3). They are not lineage-relevant.

**Track B conversion and the B3 E8 lattice codes** (`b3_e8_codecs.rs`, `b3-e8-smollm2-result-2026-09-29.md`):
- The kill fired at +4.89 nats on a disputed instrument (float reference at 9.45 nats/token suggests a token-ID mismatch; DECISIONS D16 item 4).
- These are weight codecs, so not lineage-relevant. They are **R3 on instrument grounds only**: the one-matrix exhaustive-encoder check was never run.

**D4/E8 weight codecs** (`d4-e8-and-d11-cost-result-2026-09-28.md`): every arm missed the gate. Not lineage-relevant. **C** at its scope.

**Spin and HELM-D learned-manifold R4 (#1004/#1006/#1007)** (`helm_d_r4_attention.rs`, `tests/helm_d_learned_manifold_r4_construction_973.rs`):
- This is the "mutable Riemannian manifold" line as built. #973 tests it, and the owner's "mutable manifold" is identified with the delta rule at `geometric-attention-2026-09-26.md:54,243`.
- The cumulative Spin frame `F_j` with `P_(j→i)=F_iᵀF_j` is, by construction, a gauge reparameterization that preserves the dot product (**proof**, in the module doc).
- Attempt 02 returned `FAIL_HELM_D_MANIFOLD_CONSTRUCTION_…`, and intrinsic Lorentz R4 returned `UNAVAILABLE` (barycenter covariance 9.1e-8 against a frozen 1e-8; PR #1003).
- No advantage was possible by construction. **C.**
- `bounded_global_exact_spin_attention.rs` ranks only exact max-count ties from #953. **C.**

**Paired-H4 heatmap identifiability, #970** (`candidate_relative_identifiability_a1p_970.md`):
- The exact classes alias incompatible outcomes.
- This is a readout identifiability negative, explicitly not a verdict on H4. **C** for lineage purposes.

**Hamiltonian flow.** The written no-go says a conservative flow cannot forget (`geometric-attention-2026-09-26.md:55`). It was never tested, and that is sound. **C.**

**TLA/R4G1 frozen runtime.** A scoped XOR/popcount/table kernel contract (AGENTS.md, toolbox §C). It is not a retrieval mechanism. It is the target substrate for porting F2's signed permutation, which is itself XOR-free sign-and-swap and so admissible.

**Q8 relative-action learner (causal continuation, catalogue §1.5).** The geometric claim was self-refuted by an ordinary control that also reached 1,024/1,024. **C.**

---

## 5. Recommended re-test order (ranked by decision value per unit cost)

Each step is one causal change with two predeclared seeds under D17 v2.

1. **M-world A1 cell with lineage (1.1).** Run `rrarra key_shift=true` at 1-step lineage, plus a 2-step lineage variant, sieve off, on the frozen development × development cell. This decides whether outcome D was a lineage confound.
   - Prerequisite: fix or route around the 0/33 tokenizer copy cap (1.3).
   - Predeclared prediction: 1-step lineage alone stays below 0.5 at d16 if the "k is v" gap matters, and 2-step lineage reaches it.
2. **Gap ablation on `mqar-bench`.** Add a filler-gap parameter g ∈ {1, 2, 3} between key and value, plus multi-token keys. This establishes the lineage depth needed before any m-world spend.
3. **Third Lorentz/l2 parity seed under lineage (1.2).** It completes the D17 v2 procedure in a regime where the score can matter.
4. **Post hoc sparsity on an F2 checkpoint (2.1).** Flock k=1/7/16, route top-M on binarized lineage keys, and orthant64 (2.2). This is training-free and gives the first fair D5 access measurement.
5. **Exact join (2.3).** Map F2 keys to ordered n-let addresses: the learned compiler that #958 stage 4 lacked.
6. **KVAR two-token-key variant with the Q8/2I relative code (1.4).** It removes the identity-action degeneracy that the original pilot itself proved.
7. **B1 lanes or a short exact group window as the lineage source (2.5)**, against the j-shift, at equal state bytes.

**Two points that cut across all of these:**
- The D11 serving form of F2 does not exist yet (#1704 §3).
- No entry above supports a claim of general retrieval, chat or geometric advantage until it is re-measured at the stated scope.