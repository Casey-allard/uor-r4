# Integration Plan: Joint File → Riemann-RS.tex

## Overview

The `Riemann-joint-polished-ver-07.tex` ("Joint") claims an unconditional RH proof via boundary positivity methods. The `Riemann-RS.tex` ("RS") has a two-regime approach: unconditional far-field + effective near-field + conditional RS closure.

This plan identifies valuable elements from Joint that could enhance RS.

---

## Section-by-Section Analysis of Joint File

### 1. Abstract and Introduction (Lines 140-186)

**Valuable Elements:**
- [ ] **Cleaner proof summary** (Lines 141-146): "The core insight is that boundary methods suffice to rule out interior zeros via a removability argument."
- [ ] **Historical context** (Lines 154-161): Hadamard, de la Vallée Poussin, Hardy, Selberg, Levinson, Conrey progression - more comprehensive than RS.
- [ ] **Two-obstacle framing** (Line 161): "(i) uncontrolled singularities (singular inner factors) on the boundary... (ii) difficulty of converting 'almost-everywhere' control... into uniform, quantitative control"

**Action:** Incorporate this historical progression and obstacle framing into RS Introduction.

---

### 2. Methods Section (Lines 188-396)

**Valuable Elements:**
- [ ] **Constants checklist with single source of truth** (Lines 208-216): Explicit list of K₀, K_ξ, c₀(ψ), C_H values and their derived forms
- [ ] **Function-space framework paragraph** (Lines 220-221): Clear statement of Poisson semigroup, NT limits, BMO context
- [ ] **External inputs list** (Lines 224-233): Explicit citations for Carleson-BMO, outer/inner factorization, H¹-BMO duality, VK zero-density
- [ ] **Skeletal chain** (Lines 236-244): Clean one-line dependency chain from phase-velocity to RH

**Action:** Add a "Load-bearing dependencies" summary to RS Reader's Guide modeled on the skeletal chain.

---

### 3. Contradiction Framework (Lines 247-315)

**Valuable Elements:**
- [ ] **Three-pillar presentation** (Lines 250-255): (P+) Boundary Positivity, (N1) Right-Edge Normalization, (N2) Non-Cancellation at Zeros
- [ ] **Step 1-2-3 pinch argument** (Lines 290-304): Cleaner presentation of the Schur pinch
- [ ] **Formal Lemma for removability** (Lines 306-308): Lemma 2.1 (Removable singularity under Schur bound)

**Action:** The RS file already has the Schur pinch explanation, but the three-pillar structure is cleaner in Joint.

---

### 4. Proofs of (N1) and (N2) (Lines 319-355)

**Valuable Elements:**
- [ ] **Self-contained (N1) proof** (Lines 319-331): Shows Θ(σ+it)→-1 as σ→+∞ via Stirling
- [ ] **Self-contained (N2) proof** (Lines 341-355): Shows det₂(I-A(ρ))≠0 and O(ρ)≠0

**Action:** These proofs are cleaner than what's in RS; consider replacing or supplementing.

---

### 5. Technical Framework (Lines 399-1209)

**Valuable Elements:**

#### 5a. Key Definitions (Lines 410-421)
- [ ] **Definition of admissible bump windows** (Lines 412-414)
- [ ] **Whitney boxes and Carleson-boxes definition** (Lines 416-418)

#### 5b. Phase-Velocity Identity (Lines 432-442)
- [ ] **Theorem 4.1** with clean statement and proof sketch

#### 5c. Carleson Energy Lemmas (Lines 448-565)
- [ ] **Lemma 4.2** (Carleson-box energy: stable sum bound) - triangle inequality for √C_box
- [ ] **Lemma 4.3** (Analytic ξ Carleson energy on Whitney boxes)
- [ ] **Corollary 4.4** (All-interval Carleson energy for U_ξ)
- [ ] **Lemma 4.5** (L¹-tested control for ∂_σ Re log ξ)

#### 5d. Outer Normalization (Lines 626-689)
- [ ] **Lemma 4.7** (Outer phase and Hilbert transform)
- [ ] **Lemma 4.8** (ζ-normalized outer and compensator)
- [ ] **Corollary 4.9** (No C_P/C_Γ in the certificate)
- [ ] **Lemma 4.11** (Diagonal HS determinant is analytic and nonzero)

#### 5e. Annular Estimates (Lines 691-803)
- [ ] **Lemma 4.12** (Annular Poisson-balayage L² bound) - explicit decay factor 4^{-k}
- [ ] **Lemma 4.13** (Monotonicity of tail majorant)
- [ ] **Lemma 4.15** (Block Gershgorin lower bound)
- [ ] **Lemma 4.16** (Schur-Weyl bound)

#### 5f. Window/Plateau Bounds (Lines 815-1030)
- [ ] **Lemma 4.17** (Poisson plateau lower bound) - full proof with c₀(ψ) ≥ (1/2π)arctan 2
- [ ] **Lemma 4.18** (Derivative envelope for printed window) - detailed 4-step proof
- [ ] **Corollary 4.20** (Unconditional local window constants M_ψ, C_H(ψ))

#### 5g. PSC Certificate (Lines 1052-1209)
- [ ] **Locked constants display** (Lines 1054-1059)
- [ ] **Reproducible numerics paragraph** (Lines 1067-1086)
- [ ] **Constants table** (Lines 1094-1109)
- [ ] **Non-circularity note** (Lines 1116-1118)
- [ ] **Theorem 4.21** (Boundary wedge from product certificate - local form)

**Action:** Many of these lemmas have cleaner proofs than RS. Create a mapping table.

---

### 6. Results Section (Lines 1212-1249)

**Valuable Elements:**
- [ ] **Theorem 5.1** (Riemann Hypothesis) - clean 4-step proof
- [ ] **Theorem 5.2** (Globalization across Z(ξ))
- [ ] **Summary of proof architecture** (Lines 1247-1249)

**Action:** The RS file has a different main theorem (two-regime) but the proof architecture summary is valuable.

---

### 7. Appendices (Lines 1295-1636)

**Valuable Elements:**
- [ ] **Canonical auxiliary statements** (Lines 1299-1364): 18 lemmas in compact form
- [ ] **Canonical auxiliary theorems** (Lines 1366-1411): 7 theorems/corollaries
- [ ] **Catalogue of lemmas/theorems/corollaries** (Lines 1412-1474)
- [ ] **Constants and definitions table** (Lines 1476-1494)
- [ ] **CE-constant appendix** (Lines 1507-1521)
- [ ] **VK-to-annuli-to-K_ξ enclosure** (Lines 1533-1551)
- [ ] **C_ψ^{(H¹)} numerical evaluation** (Lines 1564-1575)
- [ ] **Weighted p-adaptive model** (Lines 1595-1636)

**Action:** The catalogue structure is excellent for reference; consider adding to RS.

---

## Priority Actions

### HIGH PRIORITY (directly improves main theorem presentation)

1. **Add historical context paragraph** (from Lines 154-161)
   - Location: RS Introduction or Historical Context section
   - Content: Hadamard → de la Vallée Poussin → Hardy → Selberg → Levinson → Conrey

2. **Add "Load-bearing dependencies" skeletal chain** (from Lines 236-244)
   - Location: RS Reader's Guide
   - Content: Phase-velocity → Carleson → Wedge → Transport → Pinch → RH

3. **Improve (N1) and (N2) proofs** (from Lines 319-355)
   - Location: RS introduction or Methods section
   - Content: Self-contained proofs of normalization and non-cancellation

4. **Add Lemma catalogue appendix** (from Lines 1412-1474)
   - Location: New appendix in RS
   - Content: Quick-reference catalogue of all lemmas, theorems, corollaries

### MEDIUM PRIORITY (technical improvements)

5. **Add explicit annular L² bound** (from Lines 693-735)
   - Lemma: Annular Poisson-balayage L² bound with 4^{-k} decay
   - This is cleaner than what RS has

6. **Add Poisson plateau full proof** (from Lines 828-859)
   - Currently RS only sketches this; Joint has complete proof

7. **Add derivative envelope 4-step proof** (from Lines 875-947)
   - Detailed proof of C_H(ψ) ≤ 2/π

8. **Add constants table and non-circularity note** (from Lines 1094-1118)
   - Clean reference table for auditing

### LOW PRIORITY (nice to have)

9. **Add admissible bump windows definition** (from Lines 412-414)
10. **Add weighted p-adaptive certificate variant** (from Lines 1595-1636)
11. **Add Schur-Weyl spectral gap lemma** (from Lines 794-803)

---

## Mapping: Joint Lemma → RS Equivalent

| Joint Reference | Joint Content | RS Equivalent | Status |
|-----------------|---------------|---------------|--------|
| Theorem 4.1 | Phase-Velocity Identity | thm:phase-velocity-main? | Check |
| Lemma 4.2 | Carleson sum bound | lem:carleson-sum? | Check |
| Lemma 4.3 | Whitney Carleson energy | lem:carleson-xi? | Check |
| Lemma 4.5 | L¹-tested control | lem:xi-deriv-L1? | Check |
| Lemma 4.11 | HS determinant nonzero | lem:hs-diagonal? | Check |
| Lemma 4.12 | Annular balayage | lem:annular-balayage? | Check |
| Lemma 4.17 | Poisson plateau | lem:poisson-plateau? | Check |
| Lemma 4.18 | Derivative envelope | lem:CH-derivative-explicit? | Check |

---

## Notes

- The Joint file claims **unconditional** RH; RS claims **conditional** closure
- RS has the Recognition Science framework; Joint does not
- Both use the Schur/Herglotz/Cayley machinery
- Joint has cleaner proofs for many technical lemmas
- RS has the two-regime (far-field/near-field) structure that Joint lacks
- RS has Lean formalization references; Joint does not

---

## Next Steps

1. Review this plan with user
2. Decide which elements to integrate
3. Perform integration in priority order
4. Compile and test after each integration

---

## COMPLETED INTEGRATION (Session 2025-01-02)

The following elements were successfully integrated from Joint → RS:

| Element | Status | Location in RS |
|---------|--------|----------------|
| Historical context (Hadamard → Conrey) | ✅ Done | Subsection "Historical context" |
| Two obstacles framing | ✅ Done | Same section |
| Skeletal chain diagram | ✅ Done | Reader's Guide |
| (N1) proof (Stirling + HS continuity) | ✅ Done | After Standing Properties |
| (N2) proof (determinant nonvanishing) | ✅ Done | After Standing Properties |
| Constants checklist table | ✅ Done | After Dependency Map |
| External inputs list | ✅ Done | Same section |
| Lemma catalogue | ✅ Done | Before Conclusion |

### PDF Status
- Pages: 101 (up from 99)
- No LaTeX errors
- All existing mathematics preserved
- Claims unchanged (conditional RH via RS)

