# UOR-R4: project knowledge map and diagnosis of the 3-month barrier

Synthesis of seven surveys, plus three spot checks at `71b54adf`, the merge of PR #1705:
- `git log` head confirmed.
- `milestone_world_v2.rs:53-55` confirms that the MQAR phrasings end in "{k} is" and that facts are stated "… is {v}" (`:749-752`).
- `geometric_stack.rs:8936-8944` confirms that `previous_key_channel` is `k + L_j(shift(k))` with a zero pad.

Labels used throughout:
- **P**: proof, or exact by construction.
- **M**: measured, at the stated scope.
- **H**: hypothesis.

---

## 1. What the project is

**Mathematically.** The intended model is an autoregressive state model whose primary mechanisms are:
- exact prime addresses and ordered n-lets;
- fixed zeta-zero phases;
- R4/S³ unit-quaternion state and transport;
- the H4/600-cell / binary icosahedral group 2I;
- E8 = H4 ⊕ φH4;
- exact Z[φ] arithmetic;
- chirality;
- UOR content identity (AGENTS.md; `formal_vocabulary.md:90-131`).

**What actually runs.** Production is `geometric_stack.rs` at about 96M parameters (95,957,184): width 1024, 16 heads, 14 layers, pattern `rrarrarrarrarr`, context 384, `read=l2` (`docs/compute/ladder-runbook.md:147-168`). By inspection (P), only one piece of the declared geometry is on that default path: **per-lane unit-quaternion left transport inside a Griffin/RG-LRU-style gated linear recurrence** (`r` layer: width-4 causal conv, `h_t = λ_t u_t ⊗ h_{t−1} + √(1−λ²) c_t`).

Everything else on the path is ordinary:
- **The `a` read layer** is Euclidean-distance softmax attention. It adds an ALiBi-initialised learned age bias and a NoRead slot (`:1819-1897`, `:10045-10075`).
- **MLP, embedding and LM head** are dense SwiGLU and linear maps.
- **The pointer head** is a dot-scored copy mixture.

The remaining primitives sit off the trained path:
- **Optional, off in the ladder:** 2I transport snap, `PrimeRoute` pointer, and `read_key_shift`.
- **External or absent:** primes appear only as external memory; zeta, Hopf, Z[φ], E8 and chirality are absent from the network.

**As software.** Rust only. Offline training is float, CUDA, Metal or CPU via candle. Serving is either:
- the float `GroundedSession`, or
- the D11 integer engine (`uor-r4-integer/src/stack`). It uses no float and no multiplier instruction: nibble/byte-pair tables, radix-16 multiple tables, digit square root, table transcendentals, and a Z[φ] snap. The FULL PASS audit is from #1691.

Chat is grounded by a non-learned exterior:
- **Relation compiler** (`relation_compiler.rs`): closed 11-label relation set, plus the acts assert/update/query/none.
- **`StackStore`**: an exact versioned chain from `native_geometric::scoped_memory`.
- **Log sieve** (`milestone_world_v2.rs:2205-2244`). It is a set intersection over non-reserved query words that copies the words after the matched cue. No primes are computed.

A separate Codex line (`geometric_context/potential/read/occurrence_read/cue_carrier.rs`) is a fully integer, signed-H4 finite-group automaton with table-scored attention. It works over ≤128-token supplied records and is not integrated with the stack.

**Target (D11, ROADMAP §0, R1–R5).** No float and no multiplier in served kernels, no transformer backbone, and eventually sparse per-token parameter access (D5). Energy claims require J/token measured on the M1. Frontier-like capability on an M1 is the long-term objective. None of it is established.

---

## 2. History in eras, and lessons that held across eras

| Era | Dates | Content | How it ended |
|---|---|---|---|
| E0 | Jun 9–11 | Browser/WASM prime-router demo. Text came from Ollama/WebLLM (`engines.md` §1). | Nothing generated natively. |
| E1 | Jul 18–Aug 22 | Compiled transformerless TLA/R4G1 graphs: XOR/popcount/tables. Hopf sectors 7/512, then occupancy 456/512 with sector MRR falling to 0.0045 (#305). E8 keying negative (#403). Retrieval compared the wrong vectors for months (#486). About 40 refactor PRs. W33 found "NO GEOMETRIC ADVANTAGE". | Count graphs cannot learn context. |
| E2 | Aug 25–Sep 3 | Learned R4-framed softmax attention. #1014: attention is load-bearing, a 2.68-nat penalty without it. #1017 sealed NLL 1.5728, gate <1.50 failed. MQAR 30/87,360 (#1043). Zoology curriculum 87.1% (#1045). | Gates failed; no compute. |
| E3 | Sep 4–17 | About 50 authored-curriculum skill PRs. 2,304/2,304 neighbour transfer under authored rules (#1281). | Takeover: "not general prose or an integrated chat model". |
| E4 | Sep 18–24 | TinyStories `.rgm`: 1.2372 training-time bits/byte vs 1.8055 deployed (#1283). Takeover found three disjoint model paths. Reader series: constructed gains, natural text worse by 0.12–1.29 bits/token (#1319–#1333). A4: 1,336/1,336 local vs 28/167 after load. | D4–D8 issued. |
| E5 | Sep 25–28 | Geometric stack chosen: 1.998 vs 2.011 over a transformer, one seed (#1437). S2 dialogue 2/38 answered, 0/10 recall. D2 AERM store 1.000, learned read failed; G-binding 0.967 with the text guard failing. D10, then D11. S1 D11 engine bit-identical to D10 (#1467). | — |
| E6 | Sep 29–Oct 1 | Governance: D13–D17, a council, and 13.4 h of gating. CI executed nothing while main carried about 110 errors (#1547). **A1 outcome D:** every arm below 0.5 at d16; the transformer control scored 0; pointers ≈ recency (D18). | D19. |
| E7 | Oct 1–5 | Grounded sessions plus the scale ladder 8M → 96M on a CUDA pod. Open panel flat at 43–46/232. Sieve-off MQAR 3/109. MQAR bench (#1698), then F2 key shift (#1701), then correction (#1704). Codex cue carrier (#1705): 0–1/32 fresh. | Current. |

**Lessons that held in every era** (each is observed at least three times):

1. **The instrument failed before the mechanism did.** Examples: the dead cosine (#486), the fit/serve threshold gap (#1328), the logit drift bound (#1043), zero certifiable samples (#846), B3 token-ID mismatch (D16 §4), a transformer control scoring 0 (A1), panels with contradictory duplicate rows, and the 256 vs 317 context premise. A null result should first be read as a possible instrument defect.
2. **Local or constructed success did not transfer.** E3 rule panels, A4's 1,336 vs 28, the reader series, and the Codex 64-row fits (fresh 0/32).
3. **Score geometry was swapped repeatedly while the key content stayed unchanged.** Lorentz, Dot, L2, H4 potentials, Hopf sectors and 2I read codes all came and went. The negatives that are most defensible are about what keys contain: #1069's owner residual, KVAR Q8's identity-action degeneracy, and #999's "current-only beats connection".
4. **Exact external memory works; learned in-network retrieval did not**, until F2 on a synthetic bench (D2 store 1.000; sieve 105/109 vs 3/109).
5. **Findings were not propagated.** Predecessor identity was found by Codex on Oct 1 (#1580, 192/192) and found again by Claude on Oct 4 (#1701). It also existed implicitly in August as the adjacent-token semiprime in ADR-0003.
6. **Single-seed verdicts flipped.** #1704 reversed #1698 within hours; `rrarra` gave seed 1 = 0.007 and seed 2 = 0.9995.

---

## 3. Mechanism ledger

### ACTIVE (production or served path)

| Mechanism | Location | Role | Status |
|---|---|---|---|
| Quaternion transport recurrence `r` with width-4 conv | `geometric_stack.rs:5331-5370, 9047-9118`; D11 `session.rs:1702` | Sequence mixing; the conv is the only native predecessor route | **M:** beats U(1) by 0.0234 nats at 8M, 2 seeds (#1639) |
| L2 read `a` with age bias and NoRead | `:1819-1897`; D11 `session.rs:1898`, `kernels.rs:841` | Content retrieval | **M:** beat Lorentz by 0.0345 nats (#1639). **M:** zeroing age drops bench d16 from 0.807 to 0.115 |
| Dense SwiGLU MLP, embedding, LM head | — | Most of the parameters | Interim under D5/R3 |
| Pointer copy mixture (dim 32, Dot) | `:5817, :11871`; D11 Q30 `kernels.rs:1028-1060` | Copy from context | Served (#1669). Copy capped at 0/33 by the #1017 `add_prefix_space:false` boundary (catalogue 8.3) |
| Relation compiler (closed 11 labels) + StackStore + log sieve | `relation_compiler.rs`, `stack_store.rs`, `milestone_world_v2.rs:2205` | Grounding | **M:** session 1008 at 96M-C, sieve on. Float only; not D11 |
| D11 integer engine + GPTQ export | `uor-r4-integer/src/stack` | Serving | **M:** 43 vs 46 at 96M (p 0.71); 32.5 ids/s at 4 threads at 96M (#1691) |

### SAVED OPTION, NOT YET ON THE PRODUCTION PATH

**`read_key_shift`, F2: `k'_t = k_t + j·k_{t−1}`** (#1701 `3aa8ba16`, #1704 `2cdae05a`).
- **P:** zero parameters; a signed permutation plus an add.
- **M:** bench recall 1.000 at all distances.
- **Missing:** no QAT, no export and no D11 form. Every served path refuses it (`stack_export.rs:346,991,1726`).
- **In progress:** the pod A/B (#820, 2026-10-05T00:36Z).

### DORMANT-BUT-PROMISING (scoped negative; lineage or other evidence reopens it)

**R1: failure plausibly caused by missing Carry.**
- **A1/D18 outcome D.** One seed, 2,590 steps, with the "k is v" gap and BPE-split keys. Its signature matches #1701 arm A: a 1/N decay toward the latest value. The kill rule fired cleanly; the causal reading behind it is untested.
- **Lorentz vs L2 parity.** Undecided at 2 seeds and measured in the bag-of-values regime, where score geometry cannot matter.
- **Lorentz-scored pointer.** Never run (D18 §10).
- **KVAR Q8 relative energy.** The doc proves its own degeneracy: the relative action is identity at the read site. It becomes non-degenerate with lineage keys.
- **#1069 owner residual.** Lineage on the query side only: 45.59% → 50.27%, rejected on a preservation floor, one seed.
- **#999 direct V3 / gated-delta.** Tiny panels; "current-only" beat the connection arm.

**R2: lineage newly enables the mechanism.**
- **Flock / route top-M / VP-tree / orthant64.** Under F2 the attention mass is 0.99–1.00 on one position (M, #1701 probe), so top-k becomes near-lossless and the D5 sparse-access contest becomes fair for the first time. F7's 0.009 was measured in the bag regime.
- **Prime router with ordered n-lets** (ADR-0003, #958). Stage 4, the language-to-address compiler, was NOT_RUN. F2 keys are a candidate for that compiler.
- **B1 2I finite-group lanes.** **M:** 17/18 runs tracked A5 at length 4,096. Closed only on a +0.018 text-NLL gate, inside a 0.063 seed spread.
- **VSA permute-and-add.** The same algebraic form as F2, but never scored on keyed retrieval.
- **Zeta-phase relative rotation, as a matched-control arm only.** No theorem favours zeta ordinates over other frequencies.

**R3: not lineage-related; revisit on their own grounds.**
- D2 margin gate (unsatisfiable at 0.30).
- G-binding +0.119-nat text guard.
- G v2 2I relation keys: 0.327 vs 0.491 held-out, one run.
- joint_model 2I read kernel: the HARM verdict lacks attribution arm C.
- B3 E8 codec: disputed instrument.
- **Do not cite** the "direct two-token channel" (+0.469 bits worse; `ordinary-lexical-audit-2026-09-23.md:108-126`) against F2. It put lineage into the current readout, not into keys matched later.

### RETIRED-FOR-GOOD (at least for lineage purposes)

- **Spin/HELM-D cumulative frame.** **P:** a gauge reparameterisation that preserves the dot product, so it cannot add information.
- **Peter–Weyl harmonic class kernel.** A readout ceiling of one linear map over 120 classes.
- **#970 heatmap identifiability.** Exact aliasing of outcomes.
- **Hamiltonian flow.** **P:** a conservative flow cannot forget.
- **Q8 causal-continuation geometric claim.** An ordinary control also reached 1,024/1,024.
- **D4/E8 weight codecs** at their scope.
- **E1 compiled count-graph language path, and Hopf sector routing** (MRR 0.0045).
- **Prime arithmetic as a semantic metric.** **P:** square-free products give only a Boolean face lattice and carry no metric. AGENTS.md already says this.
- **O(1)-state exact recall**, the "Poincaré/S³ observer". **P:** Jelassi et al., arXiv:2402.01032; Based, arXiv:2402.18668.

---

## 4. Evidence ledger

**Scales with parameters and tokens (M):**
- **Base NLL:** 1.59 (8M) → 1.386 (20M, 2 seeds) → 1.2857 (29M, lr 5e-4) → 1.1729 (96M). The 96M rung's mix was only 35% TinyStories, so the rungs are not like for like.
- **Open panel 8M → 29M:** 21 → 28 → 43, p = 0.0001 against 8M.
- **Grounded session:** 959 → 1008. The 29M → 96M gain is mostly instruction following, 56 → 78/98.
- **GPTQ fidelity gap:** −10 at 29M, −3 at 96M.

**Does not scale (M):**
- **Open panel 29M → 96M:** 43 → 45/46 (p 0.76–0.87). Flat across 4k/12k/40k fine-tunes; the per-arm 12k/40k figures are UNAVAILABLE.
- **Sieve-off session MQAR:** 0–5/109 at every scale; 3/109 at 96M.
- **Disjoint-panel `store_read`:** 8/7/7/8 from 8M to 96M. Fixed by the compiler's closed label set, not by the model.
- **8M learned binding:** d16 1/37, below recency's 13/37.

**Changed by mechanism, not scale (M, bench only).** Bench: ~1.36M params, width 128, context 512, synthetic, gap 1, single-token keys, 1–2 seeds.
- F2 takes all-read from 0.255 to 1.000 overall, and the held-out pairing class from 0/1024 to 1024/1024.
- It removes the `rrarra` seed lottery.
- It brings `rararr` to 0.9 at step 200 instead of 500.
- The probe shows the arm-A heads never put more than uniform weight on the key position.

**Compiler (M, dev only).** Fix B plus the `?` rule takes 84 → 137/200, against a ceiling of 140. The remaining 60 rows are anaphoric. The false-write rate is unmeasured.

**Unmeasured (UNAVAILABLE, not negative):**
- F2 in any chat model or in the D19 cell;
- F2 at gap ≥ 2, with multi-piece keys, or on natural text;
- an isolation of `j` against an identity shift, a random fixed orthogonal map or a learned `W_prev`;
- F2 under D11;
- a matching-vs-copy split in production models;
- open-panel failure categories, quantified;
- multi-seed lock-in rates;
- any prime, zeta or H4 semantic or recall advantage. Every Codex native fit scores 0–1/32 fresh.

**Contradictions resolved:**
1. **Session "1008".** The active survey could not locate it. Roadmap and evidence cite #1552 comment 5980927216, so **1008 stands**, with 908 when the store is off.
2. **"28/232" vs "43".** No conflict: 28 is the 20M recipe B and 43 is 29M lr 5e-4.
3. **Production read = Lorentz** (math survey) **vs L2** (active, evidence). **L2 wins.** The runbook says `read=l2` and #1639 records the switch. Lorentz was the 8M default only.
4. **2I snap "served" vs "not used".** Both are true: it is ported to D11 (#1506) but absent from the ladder runbook.
5. **Is F2 "the missing piece"?** **#1704 wins over #1701's first framing.** The conv route can supply lineage on some seeds, so F2 is a robustness and learnability fix, not the only route. "Canonical token lineage is a missing math mechanism" remains **H**.
6. **GPU authorization.** AGENTS.md and project-track say no CUDA; STATUS and ROADMAP and the pod say otherwise. No DECISIONS entry records the authorization. This is a process gap to be resolved by the owner, not a scientific contradiction.
7. **Which mechanism is "geometric attention"?** project-track and current-state mean the Codex native bank; the production stack is the L2 read plus F2. The plan does not reconcile them (§5, process gap P1).

---

## 5. The barrier: a causal diagnosis

**Core claim (H, supported by M).** For about three months the programme treated in-context retrieval as a problem of **score geometry**: Lorentz, H4 potentials, Hopf sectors, 2I codes. The failure was in **key content**, specifically the *Carry* step of induction (Carry, Match, Copy; arXiv:2609.15545).

When no key encodes "what preceded me", every score geometry sees a bag of values. The #1701 probe shows exactly that: about uniform weight on the key position, and 0.045 mass on the value at d200.

Four further gaps block chat even once Carry is fixed. They are independent of it and each is separately falsifiable.

### Mechanism gaps

**G1. Carry was missing or seed-dependent (M on the bench).** All-read stacks have no route to the predecessor at all. `r` stacks have one only through a width-4 conv that training finds on some seeds. F2 makes Carry exact and parameter-free.
- **Falsifier:** if a multi-seed bench (≥5 seeds) without F2 locks in at the same rate as with F2, then G1 is a data/lock-in effect, not a mechanism gap.

**G2. Lag mismatch between the bench and real facts (H, specific and checked against the source).** M-world states facts as "… is {v}" and queries end in "{k} is" (`milestone_world_v2.rs:53-55, 749-752`). Keys are made-up names that BPE splits into several pieces.

So the value's predecessor is "is", which every fact shares. F2 at lag 1 tags the value position with `j·k("is")`. That does not discriminate facts. It reproduces the recency/bag signature seen in A1 (0.28–0.44 ≈ the R-recency rule's 0.36).

Discrimination needs either:
- **(a)** lag ≥ 2 lineage, e.g. `k_t + j k_{t−1} + k·k_{t−2}` (≤ 4 mutually orthogonal lags per lane; P, math §3.1); or
- **(b)** a *successor-value* route: attend to the copula position whose key carries `j·k(key-last-piece)`, then read the value at s+1. This is causal when s+1 < t (cf. Memory Mosaics, arXiv:2405.06394).

**Prediction:** F2 alone moves sieve-off D19 MQAR only modestly above 3/109. Lag-2 lineage or successor-value reading moves it substantially. On a gap-g bench (g ∈ {0,1,2,3}, multi-piece keys), lag-1 F2 should fail at g ≥ 1.
- **This is the single most decision-relevant prediction in the map.** It also explains why the production `rr…` patterns score 3/109 even though they have a conv route.

**G3. The copy path is capped by tokenization (M).** The pointer copies the token at the attended position, but `m`(79) vs `Ġm`(283) under `add_prefix_space:false` caps copy credit at 0/33 (catalogue 8.3). Any pointer re-test is uninterpretable until this boundary is fixed.

**G4. The compiler has a closed identity (M).** 11 labels; only 1 of the disjoint panel's 10 relations is in it. `store_read` is flat from 8M to 96M. This structural ceiling is independent of the model, and Fix B's word-derived relation phrase (137/200, dev) is the right shape for it. The 60 anaphoric rows ("What is it?") need previous-turn state: a *lineage* problem at turn level.

**G5. Open-domain content: a capacity and knowledge ceiling, not retrieval (H).** The panel failures are "mostly incoherent content on open-domain questions" (#820, 10-04T19:36). The base corpus is TinyStories, TD, chat-v0 and chat-v1, none of which carries world knowledge. Every non-MQAR session category is identical with the sieve on and off.
- **Falsifier:** categorise the 186 failing panel rows into (knowledge-absent / incoherent / instruction / recall). If recall-type failures are under 15%, no attention fix can move the panel beyond a few points, and the 43–46 plateau is a data-domain ceiling.

### Training and data gaps

- **D1. Dose.** MQAR-format data was about 0.8–2% of the fine-tune mix (#820, 10-05T00:36). So 3/109 does not separate "cannot" from "never trained to". This confounds every chat-level retrieval claim to date.
- **D2. Lock-in lottery.** Induction forms in a sharp phase transition (F2: 0.15 at step 300, 0.98 at step 400). External evidence says distance-diverse, accuracy-gated curricula raise lock-in from 1/10 to 7/10 seeds (arXiv:2609.16183, 2512.18634). The project's one curriculum test (#1642: 14 → 3/120) halved practice, so the cause is not isolated.
- **D3. Learning-rate sensitivity** of recurrent hybrids on recall (arXiv:2508.19029). It matches the 29M lr findings, and every ladder rung is single-seed.

### Evaluation gaps

- **E1.** Network-alone recall was first measured on Oct 4, after months of session scores dominated by the sieve.
- **E2.** No Carry/Match/Copy probes on production models. The #1701 probe exists only on the bench.
- **E3.** A1's transformer control scored 0 at d16, which makes the instrument or budget suspect. The result was still read as "no learned retrieval at this scale".
- **E4.** Single-seed rungs and arms throughout. qwen2.5:7b judges the open panel, and inherited panels contained contradictory duplicates.
- **E5.** The bench geometry (gap 1, single-token keys) does not match the deployment geometry (G2), so a bench success was poised to be over-read.

### Process gaps

- **P1.** Two incompatible definitions of "geometric attention". The canonical plan tracks the Codex 128-token supplied-record bank (0–1/32 fresh for two weeks). The served model uses the L2 read. Nothing decides which mechanism chat will serve.
- **P2.** Lessons are not propagated. #1580 → #1701 was a three-day rediscovery; ADR-0003's adjacent semiprime was already exact lineage.
- **P3.** Governance absorbed research time: 13.4 h of gating, CI that executed nothing, the leases deadlock.
- **P4.** The architecture expected geometry to supply *semantics* (prime/zeta/H4 as metric). The evidence says geometry's measured value is as cheap, exact, orthogonal *structural operators*: transport (+0.023 nats), lineage (F2) and finite-group tracking (B1). Content similarity must come from learned keys.

---

## 6. Constraints and degrees of freedom

### Hard constraints

- **D11 / R1–R2.** No float and no multiplier in served kernels. Runtime×runtime products go through tables or exact structure. Offline training may use float and matmul (D0-b, D8).
- **R4.** No transformer backbone, and no transformer controls or baselines (owner, 3 Oct; comparisons are geometric vs geometric and vs the previous best).
- **R3/D5.** Sparse per-token access is the end state; dense ≤4-bit maps are a labelled interim.
- **R5.** Energy claims only from measured M1 J/token.
- Rust only, with no new Python model code.
- No hidden teacher authoring responses.
- Exact store semantics: absence only when proven; eviction proven separately.
- "No RoPE" (owner 09-29). Its application to fixed right-acting quaternion or zeta rotations needs an explicit ruling.
- **Laptop cost.** 96M runs at 32.5 ids/s on 4 threads. Context is 384 now, and conversations need ≥317.

### Degrees of freedom that remain

- **Key content.** Lineage order, choice of lag operators (any pure-imaginary unit quaternion; the size-30 order-4 class of 2I is exactly the zero-self-coherence set, P), successor-value reading, lineage on queries and values, and Canon-style neighbour mixing (arXiv:2512.17351).
- **Data.** MQAR and fact dose, distance diversity, accuracy-gated curriculum, an open-domain knowledge corpus.
- **Exact-learned join.** Index the sieve/store by the same ordered lineage tuple as the keys, then put a kNN-LM-style learned gate over both (arXiv:1911.00172).
- **Sparse access.** Under one-hot F2 reads: flock/top-k, sign-LSH, hierarchical Fenwick or Zeckendorf pages (arXiv:2506.04761), product keys (arXiv:1907.05242).
- **The genuinely geometric extension (H).** A right-acting, 2I-exact, telescoping transported-key read `⟨q_t P_t⁻¹, k_s P_s⁻¹⟩`, where Z[φ] coefficients are load-bearing.
  - **P:** right actions commute with `L_j`.
  - **P:** A5 ≅ 2I/{±1} gives NC¹ expressivity via Barrington's theorem; cf. PaTH, arXiv:2505.16381.
- **D11 port of F2.** A signed permutation plus an integer add, with an estimated ~200 lines of snapshot/rollback work (#1704).

---

## 7. Open questions for the expert panel

1. **Lag depth (G2).** Does lag-1 F2 transfer to "k is v" facts with multi-piece keys? Or is lag-2/3 lineage (`{1, i, j, k}` lags), or a successor-value read, required? Decide this on a gap-g × key-length bench *before* any pod spend.
2. **Is `j` special?** Run matched arms: identity shift, random fixed SO(4) per lane, learned `W_prev`, and another order-4 icosian. Under learned `W_q`, any orthogonal separation may be equivalent (H). The claim "canonical token lineage" stands or falls here.
3. **Mechanism or lottery?** Under a distance-diverse, accuracy-gated curriculum, what is the lock-in rate per pattern, with and without lineage, over ≥5 seeds?
4. **Attribution of 3/109.** Split it among dose (D1), lag (G2), copy tokenization (G3) and missing Carry (G1). Paired 2-seed chat fine-tunes crossing {F2, lag-2} × {MQAR dose 2% vs ~15%}, on the sieve-off D19 cell, with the open panel reported separately.
5. **What bounds the panel plateau (G5)?** A failure taxonomy of the 186 rows. If knowledge-absent dominates, the next lever is base-corpus domain and scale, not attention. Then: what corpus is admissible and affordable?
6. **Exact-learned unification.** Can F2/n-let keys serve as the language-to-address compiler that ADR-0003 stage 4 lacked, so that the authored sieve becomes a learned, exact-admitted read? What gate decides store vs network?
7. **Sparse D5 access.** On an F2 checkpoint, training-free: recall vs positions scored for top-1/7/16, sign-LSH, and orthant admission. This is the first fair access contest.
8. **Where 2I/Z[φ] becomes load-bearing.** Is the right-acting transported-key read worth a bench arm, and does the "no RoPE" ruling permit it? Does B1 lane state make a better long-lag lineage source than shifted keys at equal bytes?
9. **Compiler.** Should the closed 11-label identity be replaced by word-derived open relation phrases (Fix B) plus turn-level lineage for anaphora? How is the false-write rate bounded?
10. **One plan.** Which attention mechanism will the served chat model use: the Codex native bank or the stack L2+lineage read? What evidence would retire or merge the other?
11. **D11 lineage.** Port F2 or multi-lag lineage exactly, then measure whether GPTQ/QAT preserves the near-one-hot reads. Selection-sharp attention may be more fragile under 4-bit keys.
12. **Instrument hygiene.** Freeze bench–deployment geometry parity: gap, key length, tokenizer prefix space, context ≥ 317. Require ≥2 seeds for every chat-level verdict.

Every claim above is scoped to the repo at `71b54adf`, issues #820 and #1552, and PRs #1580, #1639, #1698, #1701, #1704 and #1705. Nothing was re-run beyond the three source spot checks.

**Key files:**
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/geometric_stack.rs`: key shift at `:2125-2160` and `:8913-8944`.
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/milestone_world_v2.rs`: templates at `:53-55` and `:749-752`; sieve at `:2205-2244`.
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/relation_compiler.rs`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/DECISIONS.md`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/research/retired-mechanisms-catalogue-2026-10-01.md`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/compute/ladder-runbook.md`