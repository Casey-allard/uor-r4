# Publication Strategy — Angular Manifold Routing Program

**Date:** 2026-03-26  
**Status:** Finalised post-INC-0173. No further experiments required before Paper 1.  
**Canonical authority:** this file governs staged disclosure decisions until Paper 2 is scoped.

---

## Executive Summary

The research program is larger than Paper 1. The narrow empirical anchor (angular
non-uniformity → sublinear routing → LM gate replacement at 6–8% PPL cost) is now
confirmed across two datasets and is ready for arXiv. The broader thesis (H^4 × H^4
architecture, phase transport as a primitive, spectral operators, sparse events,
10×–100× hardware savings) is real, partially proven, and should be staged across
multiple papers and held private until Paper 1 receives priority timestamping.

**Release gate decision: OPEN. No further pre-publication experiments are required.**

---

## 1. Claims Safe for Paper 1 (Tier 1 + Tier 2)

These are the claims the paper can make without overclaiming. Every claim below is
directly supported by completed, closed experiments.

### 1a. Geometric fact (Tier 1 — proxy scale, reproducible)

| Claim | Evidence | Strength |
|---|---|---|
| L2-normalized token embeddings on the unit hypersphere are angularly non-uniform in semantically structured corpora | INC-0143 (4 seeds, 38.5% rel_diff); TurboQuant independent corroboration | Strong |
| A fixed Hopf fibration map exploits this to produce sublinear routing footprint: eff_buckets = 2.957 × K^0.572 (vs K^1.0 for dense) | INC-0169 (160 runs, 7 seeds, K=25–400) | Very strong — canonical law |
| The advantage grows with K and persists at K=5000 (ratio 2.6–2.8×) | INC-0170 (K=600–5000) | Strong |
| The mechanism is the raw fiber mean phase (θ₁+θ₂)/2 — purely angular | INC-0147 (mechanism isolation: +20.6pp over base); INC-0168 (norm-invariant, Δα < 0.015) | Strong |
| Fixed 4D Hopf geometry outperforms 100D adaptive K-means on structured embeddings | INC-0144 (31.2% vs 3.1%) | Strong |
| The scaling exponent is norm-invariant (L1/L2/L3/L4) | INC-0168 (Δα < 0.015) | Strong |
| Hardware proxy: 3.0–4.9× eff_cost reduction, 2.5–2.9× LRU-16 miss reduction | INC-0165 (80 runs, 5 seeds) | Moderate (simulated, not physical) |

### 1b. LM integration (Tier 2 — toy-scale trainable LM, confirmed)

| Claim | Evidence | Strength |
|---|---|---|
| Fixed geometric routing replaces learned top-1 gating in a 2-layer transformer FFN at 6–8% PPL cost with no gate matrix | PTB: INC-0171 confirm (2 seeds, ratio 1.081); WT2: INC-0173 confirm (2 seeds, ratio 1.081 mean) | Confirmed across 2 datasets, 2 seeds each |
| The comparison is against native learned concentration (top-1, no aux loss, eff_b≈32–44) | INC-0172 architectural control (KILL): aux loss destroys concentration; BASELINE is the correct comparator | Confirmed |
| HOPF and PERMUTED are statistically indistinguishable in trainable LM settings | INC-0171 (Δ=0.13 ppl, PTB); INC-0173 (|Δ|=0.03 ppl mean, WT2) | Confirmed |
| HOPF routing footprint vs dense: 1.39–1.56× at convergence (K=64) | INC-0171/INC-0173 | Confirmed |

### 1c. Honest scope boundaries (mandatory disclosures in Paper 1)

1. The Hopf geometry does NOT add advantage over random fixed routing in trainable LM settings — geometry is absorbed by learning.
2. Evidence is at 2-layer toy scale. Large-scale LM is future work.
3. Proxy experiments (Sections 2–3) use PPMI-SVD static embeddings, not backprop. The two regimes do not contradict; they are different experimental settings.
4. Hardware proxy (Section 3) is simulated LRU cache pressure, not physical hardware.
5. Single research group; no external replication for Sections 2–3. TurboQuant is independent corroboration of the geometric fact, not of this work's routing algorithm.

---

## 2. Broader Program Claims (Supported but NOT Paper 1 Headline)

These are real results from the research program that are too broad or insufficiently
confirmed for a headline claim in Paper 1. They may appear as **framing context** or
**future work** in Paper 1, and as **primary claims in Paper 2+**.

### 2a. Phase transport as a computational primitive

- **What's proven:** Fiber phase coordinate (θ₁+θ₂)/2 adds +20.6pp over Hopf base alone (INC-0147, proxy). The Levi-Civita correction adds only ~2pp (below noise on proxy). Phase transport is a real geometric object.
- **What's NOT proven:** Transfer to real LM backprop. Phase transport usefulness in a trainable setting is Stage 4 status: PARTIAL-PASS on proxy only.
- **Paper 1 treatment:** The algorithm description mentions λ=1.0 transport; mechanism isolation is in Section 2.2. Do NOT claim phase transport as an independent contribution.
- **Paper 2 target:** Phase transport as a standalone routing primitive, tested in trainable settings.

### 2b. Spectral / operator structure on the routing manifold

- **What's proven:** Poincaré 4D spectral operator: +91–95% sector_lowfreq_energy vs Euclidean-KNN baseline (INC-0151, 4 seeds); task-signal smoothness +40–48%; spectral compression ratio 1.5× (INC-0156/0157). Operator theory chain geometry → spectral modes → task signals fully confirmed at 4 seeds.
- **What's NOT proven:** Whether spectral structure translates to computational advantage in a trained LM. Stage 5 was PARTIAL-PASS (strong) on proxy.
- **Paper 1 treatment:** Not mentioned. Too broad and not necessary for the narrow claim.
- **Paper 2 target:** "Geometry-Native Spectral Operators on Angular Routing Manifolds" — standalone paper if spectral structure is confirmed in LM settings.

### 2c. Sparse event routing on top of geometry

- **What's proven:** Routing sparsity (Gini 2.12× more concentrated at BASE), bucket coherence (purity ratio 1.976× TRANS K=100, 4 seeds), routing footprint sublinearity (Stage 6 PARTIAL-PASS strong).
- **What's NOT proven:** Whether sparse events can be trained end-to-end as a hardware-efficiency mechanism in a real model.
- **Paper 1 treatment:** The K^0.572 scaling law and eff_ratio results ARE the routing-sparsity result — they appear in Paper 1 as the main efficiency claim. The deeper sparse event trainability question is future work.
- **Paper 2/3 target:** Sparse event routing as a trainable architectural primitive.

### 2d. H^4 × H^4 product manifold architecture

- **What's proven:** First H^4 (routing geometry): characterised, canonical law frozen, LM-validated. Second H^4 (coupled field): architectural specification exists but was not experimentally tested at scale.
- **What's NOT proven:** The product manifold framing has not been experimentally validated. The paper correctly presents only the first H^4.
- **Paper 1 treatment:** NOT mentioned. The paper is presented as routing geometry, not as the full architecture.
- **Long-game target:** Full H^4 × H^4 architecture paper when production-scale LM is available.

### 2e. The 10×–100× hardware savings target

- **What's proven:** 1.39–1.41× eff_ratio vs dense routing in a 2-layer toy LM (real confirmed); 3.0–4.9× simulated LRU cost reduction on proxy (simulated); K^0.572 scaling law (sublinear but not transformative at K=64).
- **What's NOT proven:** The 10×–100× claim requires large K, production scale, and full sparse event mechanism. At K=64, 2-layer toy scale, the confirmed gain is 1.39–1.41×.
- **Paper 1 treatment:** The 1.39× gain is stated honestly. The 10×–100× is NOT claimed.
- **Long-game treatment:** Only claimable after large-scale LM + large-K validation.

---

## 3. Forward-Edge Claims to Withhold (Private for Now)

These components should remain in the repo but should NOT appear in Paper 1 or any
public discussion until specifically validated:

| Component | Why withhold | When to surface |
|---|---|---|
| H^4 × H^4 product architecture framing | Second H^4 (coupled field) not experimentally validated; naming this would invite scrutiny beyond what's proven | After Paper 2 validates phase transport in LM |
| 10×–100× hardware savings claim | 14–17× larger than what Paper 1 proves (1.39×); requires large-K + production scale | After large-scale LM validation |
| Spectral operator as routing component | Not tested in trainable LM; would inflate Paper 1 scope | Paper 2 if confirmed in LM |
| Phase transport as independent contribution | Only proxy-scale; LM transfer not demonstrated | Paper 2 after LM experiment |
| Sparse event routing as architectural primitive | Not trainable end-to-end in current experiments | Paper 3 or after dedicated trainability work |
| Stage 1 (hyperbolic embedding stability) | Still PARTIAL — not resolved | After dedicated embedding benchmark |

**Working title for the private program:** "Geometry-Native Sparse Computation on H^4 Routing Manifolds" — this frames the full program while the public release is narrower.

---

## 4. Minimum First Public Artifact

**What to release now:**

`docs/research/ANGULAR_MANIFOLD_ROUTING_PAPER.md` converted to arXiv-format PDF, submitted to cs.LG (primary), cs.CL (secondary).

**Exact scope of Paper 1:**
- Title: *Angular Manifold Routing: Sublinear Compute Reduction via Hopf-Base Sector Discretization — An Extension of TurboQuant to Transformer Routing*
- Claims: Tier 1 + Tier 2 only (Sections 1–5 as written)
- NOT included: spectral operators, phase transport as independent contribution, H^4 × H^4, 10×–100× claim
- Evidence included: INC-0143 through INC-0173 (scaling law + hardware proxy + LM confirm + WT2 replic.)
- Artifact: `hopf_routing_demo.py` (numpy only, <60s, reproduces Table 2)

**What this preserves:**
- Priority on: angular routing for compute reduction, K^0.572 scaling law, LM gate replacement at toy scale
- Framing: TurboQuant complementary extension (destroys structure vs exploits structure)
- Honest scope: Section 5.2 honest limitations already written

**What this does NOT foreclose:**
- Paper 2 on phase transport / spectral operators — not included in Paper 1
- Paper 3 on sparse events / H^4 × H^4 — not mentioned in Paper 1
- The 10×–100× program — framed as motivation in CORE_PROJECT_GOALS.md, not in paper

---

## 5. Further Pre-Publication Experiments: NOT REQUIRED

**Decision: No further experiments needed before Paper 1 submission.**

Rationale:
- Tier 1 claims: 173 increments, 7 kill-list stages, canonical law frozen, norm-invariant
- Tier 2 claims: PTB confirmed (2 seeds) + WT2 confirmed (2 seeds, INC-0173 seed 1 added 2026-03-26) — cross-dataset validated at equal 2-seed standard
- INC-0172 resolved the comparison scope question (native concentration confirmed as correct regime)
- The honest limitations in Section 5.2 explicitly disclose what's not proven
- WT2 2-seed mean ratio (1.081) is identical to PTB's 2-seed ratio (1.081) — the strongest possible replication outcome

**No further pre-publication experiments are required under any standard reviewer request.**
- If large-scale LM becomes available: that is Paper 2, not a prerequisite for Paper 1

---

## 6. Recommended Disclosure Sequence

### Step 1 — Paper 1: arXiv NOW (unblocked)

**Target:** arXiv cs.LG, 4–5 pages  
**Claim scope:** Tier 1 + Tier 2 (narrow)  
**What it preserves priority on:**
- Angular routing efficiency mechanism
- K^0.572 canonical law
- LM gate replacement (6–8% PPL cost)
- TurboQuant complementary extension framing  

**Remaining execution steps (no experiments):**
1. LaTeX conversion from ANGULAR_MANIFOLD_ROUTING_PAPER.md
2. Figure 1: log-log eff_buckets vs K (data already in repo)
3. ppmi_proxy.npz → include or write regeneration script
4. Final author review of Tier 1/2/3 boundaries before submission

### Step 2 — Follow-on after Paper 1 feedback (~2–6 months)

**Options (choose based on Paper 1 reception):**

**Option A — Phase Transport Paper (if Paper 1 receives traction on the routing mechanism):**
- Validate phase transport usefulness in trainable LM (the open Stage 4 LM transfer question)
- Claim: "Fiber phase coordinate is a geometry-native computational primitive that adds measurable routing advantage at toy LM scale"
- Requires: 1 targeted experiment (HOPF_TRANSPORT vs HOPF_BASE in trainable LM)

**Option B — Spectral Operator Paper (if Paper 1 receives traction on the geometric structure claim):**
- Validate Poincaré spectral operator in trainable LM
- Stage 5 evidence (strong on proxy) needs LM transfer confirmation
- Requires: 1 targeted experiment

**Option C — Second-dataset + larger-K confirmation (if Paper 1 reviewers ask for broader scope):**
- Extend INC-0173 to 2 seeds; run INC-0171-style at K=128 or K=256 to show eff_ratio grows
- No new architecture; pure extension

### Step 3 — Long game (private, 12+ months)

- Large-scale LM with Hopf-routed MoE (extends INC-0171 to production scale)
- H^4 × H^4 product manifold architecture paper (requires production-scale validation)
- 10×–100× hardware savings claim (requires full sparse event mechanism at scale)

---

## 7. Claim Partitioning Summary Table

| Claim | Paper 1 | Paper 2 | Long game | Currently private |
|---|---|---|---|---|
| Angular non-uniformity + K^0.572 law | ✅ Headline | — | — | No |
| Hardware proxy 3–5× | ✅ Supporting | — | — | No |
| LM gate replacement 6–8% PPL | ✅ Headline | — | — | No |
| Cross-dataset (PTB + WT2) | ✅ Strengthening | — | — | No |
| Phase transport as primitive | Framing only | ✅ Target | — | Yes (details) |
| Spectral operator on routing manifold | Not included | ✅ Target | — | Yes |
| Sparse event routing mechanism | Not included | — | ✅ Target | Yes |
| H^4 × H^4 product architecture | Not included | — | ✅ Target | Yes |
| 10×–100× hardware savings | Not included | — | ✅ After large LM | Yes |

---

*This strategy supersedes any prior publication staging notes in PUBLICATION_PACKET.md.*  
*Canonical publication state is tracked in ACTIVE_STATE.md.*  
*Experimental decisions are tracked in DECISIONS.md.*
