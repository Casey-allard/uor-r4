# Proof and Document Map

**Paper:** paper1_zerozeta-v19.tex (1968 lines)
**Claim:** ζ(s) ≠ 0 for Re s > 1/2 (Riemann Hypothesis)
**Proof type:** Purely analytic, direct contradiction

---

## LOAD-BEARING PROOF CHAIN (minimal path from hypotheses to conclusion)

### Step 0: Setup (§2, lines 137–246)
| Item | Lines | Role | NEEDED? |
|------|-------|------|---------|
| ζ, ξ, Ω definitions | 141–153 | Domain and notation | ✅ YES |
| A(s) operator, HS bound | 155–168 | det₂ is holomorphic on Ω | ✅ YES |
| Lemma `det2-diagonal` | 169–180 | Explicit product formula | ✅ YES |
| det₂ zero-free on Ω | 191–192 | Core structural fact | ✅ YES |
| J definition (eq J-def) | 194–204 | Arithmetic ratio | ✅ YES (for J_out construction) |
| Remark `Ocan-role` | 205–211 | Gauge independence | ⚠️ COULD TRIM |
| Θ Cayley transform | 213–219 | Used in (P+) route only | ❌ NOT NEEDED for primary proof |
| Lemma `poles` | 220–229 | ζ-zeros → poles of J | ✅ YES (but only used for motivation) |
| Schur/Herglotz terminology | 231–239 | Used in (P+) route only | ❌ NOT NEEDED |
| "Outline of far-field strategy" | 241–246 | Refers to Θ Schur bound | ❌ NOT NEEDED (stale narrative) |

### Step 1: Schur/Herglotz pinch (§3, lines 248–315)
| Item | Lines | Role | NEEDED? |
|------|-------|------|---------|
| Lemma `removable-schur-p1` | 256–273 | Removability | ❌ NOT NEEDED (primary proof doesn't use Schur) |
| Corollary `no-poles` | 276–299 | Schur → no poles | ❌ NOT NEEDED |
| "Conclusion: Schur implies Theorem" | 301–315 | | ❌ NOT NEEDED |

**Entire §3 is NOT USED by the primary proof.** It belongs to the (P+) route.

### Step 2: Outer normalization + (P+) route (§4, lines 316–584)
| Item | Lines | Role | NEEDED? |
|------|-------|------|---------|
| F definition | 322–326 | Building block for J_out | ✅ YES |
| Lemma `F-boundary-admissible` | 327–352 | F ∈ N⁺, boundary traces | ✅ YES (for outer construction) |
| Lemma `F-boundedtype-from-J` | 353–374 | | ✅ YES (supports above) |
| Lemma `BT-to-Nplus` | 375–389 | | ✅ YES (supports above) |
| Lemma `det2-logL1-from-carleson` | 390–417 | det₂ boundary trace | ✅ YES |
| Lemma `zeta-logL1-components` | 418–442 | ζ boundary trace | ✅ YES |
| Lemma `F-logL1-local` | 443–470 | | ✅ YES (supports outer) |
| Lemma `outer-factor-halfplane` | 471–487 | O_ζ construction | ✅ YES |
| J_out definition (eq J-out) | 489–494 | Core object | ✅ YES |
| (P+) definition + Remark | 496–513 | (P+) not used | ⚠️ KEEP as Remark only |
| "Hypothetical (P+)" subsection | 515–584 | Transport lemmas | ❌ NOT NEEDED |
| Lemmas `smirnov-regularity` thru `herglotz-schur-transport` | 525–584 | (P+) route only | ❌ NOT NEEDED |

### Step 3: THE PROOF (§4.1, lines 586–743)
**Every line here is load-bearing.**
| Item | Lines | Role | NEEDED? |
|------|-------|------|---------|
| Contradiction hypothesis | 587–590 | Assume ζ(ρ₀) = 0, β₀ ≥ 1/2+ε | ✅ YES |
| Whitney parameter choice | 592–604 | c, c₀, L definitions | ✅ YES |
| Sign lemma | 606–617 | -(d/dt Arg b) = positive Poisson | ✅ YES |
| Neutralization of 𝒥 | 620–649 | 𝒥_neut = 𝒥/B_box, W̃ harmonic | ✅ YES |
| Phase-velocity lower bound | 651–677 | -(Arg 𝒥_neut)' ≥ 0, includes ρ₀ | ✅ YES |
| CR-Green upper bound | 679–716 | ≤ A√c₀ · L | ✅ YES |
| Contradiction | 718–725 | c_ε ≤ c_ε/2, impossible | ✅ YES |
| Small-height: vacuous | 727–732 | First zero at |γ| ≈ 14.13 | ✅ YES |
| Remark `Pplus-route` | 734–743 | Alternative route reference | ⚠️ COULD REMOVE |

### Step 4: Conclusion (lines 745–781)
| Item | Lines | Role | NEEDED? |
|------|-------|------|---------|
| Conclusion text | 745–776 | Summary | ✅ YES (trim stale refs) |
| Statements and Declarations | 777–781 | Journal requirement | ✅ YES |

### Step 5: Appendix A — supporting lemmas (lines 782–1791)
| Item | Lines | Role | NEEDED? |
|------|-------|------|---------|
| Appendix intro | 782–823 | Setup, notation | ✅ YES |
| Outer normalizer lemma | 846–869 | Outer function construction | ✅ YES |
| Phase-velocity subsection intro | 871–874 | | ⚠️ TRIM |
| Lemma `outer-phase-HT` | 875–896 | Hilbert transform identity | ❌ NOT NEEDED (used only for (P+)) |
| Lemma `det2-unsmoothed` | 897–925 | | ❌ NOT NEEDED |
| Lemma `carleson-arith` | 926–947 | Arithmetic Carleson energy K₀ | ✅ YES (used in det₂ BMO trace) |
| Whitney scale + RvM bound | 949–964 | N(T;H) bound | ✅ YES |
| Lemma `annular-balayage` | 965–999 | | ❌ NOT NEEDED (not used in primary proof) |
| Phase-velocity identity subsection | 1001–1113 | | ❌ NOT NEEDED (primary proof uses sign lemma directly) |
| Lemma `pv-distributional` | 1005–1017 | | ❌ NOT NEEDED |
| Lemma `xi-deriv-L1` | 1020–1043 | L¹ control for log|ξ| | ✅ YES (used in S≡1 proof) |
| Lemma `J-boundedtype-local` | 1044–1078 | | ❌ NOT NEEDED |
| Theorem `phase-velocity-quant` | 1079–1113 | Phase-velocity identity | ❌ NOT NEEDED (primary proof uses 𝒥_neut directly) |
| Assembly ingredients intro | 1115–1148 | | ⚠️ TRIM |
| Lemma `poisson-plateau` | 1122–1138 | | ❌ NOT NEEDED |
| **Lemma `inner-reciprocal`** | 1156–1251 | 𝒥 holomorphic, |𝒥|≤1, W≥0, PL | ✅ **CRITICAL** |
| Lemma `green-potential-Jout` | 1253–1289 | Green potential representation | ⚠️ NOT NEEDED for primary proof |
| **Proposition `Cbox-finite`** | 1291–1569 | Energy bound + S≡1 proof | ✅ **CRITICAL** |
| Whitney wedge lemma | 1572–1727 | | ❌ NOT NEEDED (primary proof doesn't use) |
| Admissible window class | 1576–1590 | | ❌ NOT NEEDED |
| Test energy lemma | 1591–1602 | | ❌ NOT NEEDED |
| Cutoff pairing lemma | 1603–1655 | | ❌ NOT NEEDED |
| **CR-Green pairing lemma** | 1656–1681 | ✅ **CRITICAL** (used in Step 2) |
| **Proposition `length-free`** | 1682–1692 | Upper bound for admissible tests | ✅ **CRITICAL** |
| Whitney-uniform wedge | 1693–1727 | | ❌ NOT NEEDED |
| Energy-budget remark | 1728–1768 | | ❌ NOT NEEDED |
| (P+) status remark | 1770–1791 | | ❌ NOT NEEDED |

### Step 6: Removed verification appendix (lines 1793–1903)
All inside `\iffalse...\fi`. **DELETE ENTIRELY.**

### Step 7: Bibliography (lines 1904–1967)
✅ YES — keep all cited references.

---

## REMOVAL JUSTIFICATIONS (why each item is not load-bearing)

### Removal 1: §3 Schur/Herglotz pinch mechanism (lines 248–315)
**WHY NOT LOAD-BEARING:** The primary proof (§4.1) uses a direct contradiction via the inner reciprocal 𝒥_neut. It never invokes Corollary `no-poles` or the Schur bound on Θ. The pinch mechanism is only needed for the (P+) → Schur → removability route, which is explicitly labeled "not used."

### Removal 2: §4 "(P+) to Schur bound" subsection (lines 496–584)
**WHY NOT LOAD-BEARING:** This includes the (P+) definition, the Smirnov regularity lemma, the Schur/Herglotz transport lemmas, and Proposition `herglotz-schur-transport`. All are used ONLY in the (P+) route. The direct contradiction proof references none of these.

### Removal 3: Introduction strategy text referencing Θ/Schur (lines 101–135)
**WHY NOT LOAD-BEARING:** The "Strategy: Schur pinching via a Cayley field" subsection describes the (P+) route. The primary proof doesn't use Θ or the Schur bound. Replace with a brief description of the actual proof strategy (direct contradiction via inner reciprocal).

### Removal 4: Appendix phase-velocity identity and supporting lemmas (lines 871–1113)
**WHY NOT LOAD-BEARING:** Includes `outer-phase-HT`, `det2-unsmoothed`, `pv-distributional`, `J-boundedtype-local`, and Theorem `phase-velocity-quant`. The direct contradiction proof uses the sign lemma (self-contained in the theorem proof) and the inner reciprocal's Blaschke factorization for positivity. It does NOT invoke the full phase-velocity theorem.

### Removal 5: Appendix Whitney wedge assembly (lines 1115–1791)
**PARTIAL — keep only the load-bearing items:**
- KEEP: Lemma `inner-reciprocal` (lines 1156–1251) — CRITICAL
- KEEP: Proposition `Cbox-finite` (lines 1291–1569) — CRITICAL
- KEEP: Lemma `CR-green-phase` (lines 1656–1681) — CRITICAL
- KEEP: Proposition `length-free` (lines 1682–1692) — CRITICAL
- REMOVE: `poisson-plateau` (not used)
- REMOVE: Admissible window class definition (used only in (P+) assembly)
- REMOVE: `uniform-test-energy` (used only in (P+) assembly)
- REMOVE: `cutoff-pairing` (only setup for CR-green)
- REMOVE: Whitney-uniform wedge lemma (P+ route)
- REMOVE: Energy-budget remark (P+ route)
- REMOVE: (P+) status remark (P+ route)

### Removal 6: Lemma `green-potential-Jout` (lines 1253–1289)
**WHY NOT LOAD-BEARING:** The Green potential representation U = 2log|B| + W is already established in Lemma `inner-reciprocal`. The distributional identity -Δ(U-2log|B|) = 2πμ_ξ is never directly used in the contradiction proof.

### Removal 7: Lemma `annular-balayage` (lines 965–999)
**WHY NOT LOAD-BEARING:** This lemma was part of an earlier (failed) approach to bound the far-field energy via annular sums. The current proof uses the Poisson-integral boundary bound M ≤ C* logT instead.

### Removal 8: \iffalse verification block (lines 1793–1903)
**WHY NOT LOAD-BEARING:** Already disabled. Contains verification commands for computational certificates that are no longer logically required.

### Removal 9: Stale "0.6" references
**WHY NOT LOAD-BEARING:** Several lines still reference "Re s ≥ 0.6" instead of "Re s > 1/2". These are stale from the pre-RH version and should be updated.

## WHAT CAN BE REMOVED

### Entire sections that are NOT load-bearing:
1. **§3 Schur/Herglotz pinch** (lines 248–315) — used only in (P+) route
2. **§4 "Hypothetical (P+)" subsection** (lines 515–584) — transport lemmas for (P+)
3. **Appendix: Phase-velocity identity** (lines 871–1113) — includes `outer-phase-HT`, `det2-unsmoothed`, `pv-distributional`, `J-boundedtype-local`, `phase-velocity-quant`
4. **Appendix: Whitney wedge + admissible tests** (lines 1572–1791) — includes `poisson-plateau`, admissible class, test energy, cutoff pairing, Whitney-uniform wedge, energy-budget remark, (P+) status
5. **Removed verification appendix** (lines 1793–1903) — already in `\iffalse`
6. **Lemma `annular-balayage`** (lines 965–999) — not used
7. **Lemma `green-potential-Jout`** (lines 1253–1289) — not used in primary proof

### Stale narrative references:
- Line 115: "certify Schur bound for Θ_raw on {Re s > 0.6-ε}" — should reference direct contradiction
- Line 153: "fixed far region {Re s ≥ 0.6} ⊂ Ω" — should say Re s > 1/2
- Lines 241–246: "Outline of far-field strategy" references Θ Schur bound
- Lines 301–315: "Schur on far half-plane implies Theorem" — (P+) route language

---

## THE MINIMAL PAPER (load-bearing only)

A stripped version would contain only:

1. **§1 Introduction** (claim + strategy sketch) — ~30 lines
2. **§2 Definitions** (A(s), det₂, J_out, B, Ω) — ~100 lines
3. **§3 The proof** (Steps 1–3 of the direct contradiction) — ~150 lines
4. **Appendix: Supporting lemmas** containing only:
   - Outer normalizer construction (Lemma `outer-from-logmodulus`)
   - Arithmetic Carleson energy (Lemma `carleson-arith`)
   - RvM bound (eq `RvM-short`)
   - L¹ control for log|ξ| (Lemma `xi-deriv-L1`)
   - Boundary admissibility lemmas for F (Lemmas `F-boundary-admissible` through `outer-factor-halfplane`)
   - **Inner reciprocal** (Lemma `inner-reciprocal`) — with PL proof and S≡1 proof
   - **Energy bound** (Proposition `Cbox-finite`)
   - **CR-Green pairing** (Lemma `CR-green-phase`)
   - **Length-free proposition** (Proposition `length-free`)
5. **Bibliography**

**Estimated minimal size: ~12–15 pages** (down from 25).

---

## DEPENDENCY GRAPH (what references what)

```
Theorem 1 (RH)
├── Step 1: Sign lemma (self-contained)
├── Step 1: Neutralization (uses Lemma inner-reciprocal)
│   └── Lemma inner-reciprocal
│       ├── Part 1: 𝒥 holomorphic (uses det₂ zero-free, B·ζ holomorphic)
│       ├── Part 2: |𝒥*| = 1 (uses |B*|=1, |J_out*|=1)
│       └── Part 3: |𝒥| ≤ 1 (uses PL + boundary trace from Parts 1-2)
│           ├── Boundary trace uses: det₂ BMO trace (Lemma carleson-arith)
│           ├── Boundary trace uses: ζ L¹_loc trace (Lemma zeta-logL1-components)
│           └── PL: subharmonic + boundary ≤ 0 + growth o(|s|) → u ≤ 0
├── Step 1: -(Arg 𝒥_neut)' ≥ 0 (uses sign lemma + S≡1)
│   └── S ≡ 1 (inside Proposition Cbox-finite)
│       ├── log|B| convergence: uniform (B continuous)
│       ├── log|O_ζ| convergence: Poisson (outer construction)
│       ├── log|det₂| convergence: explicit Fourier
│       └── log|ζ| convergence: Jensen + RvM + Vitali
├── Step 2: CR-Green upper bound (uses Proposition length-free)
│   └── Proposition length-free
│       └── Lemma CR-green-phase (Green's identity + Cauchy-Schwarz)
├── Step 2: Energy bound (uses Proposition Cbox-finite)
│   └── Proposition Cbox-finite
│       ├── Step 1: Whitney neutralization (B_near, B_far, S)
│       ├── Step 2: Boundary bound M ≤ C* logT (Green kernel sum + PL)
│       ├── Step 3: Interior gradient estimate (odd reflection + Cauchy)
│       └── Step 4: Assembly (E_neut ≤ C log²T |I|, C independent of c)
├── Step 3: Contradiction algebra (c_ε ≤ c_ε/2)
└── Small height: vacuous (first zero at |γ| ≈ 14.13)
```
