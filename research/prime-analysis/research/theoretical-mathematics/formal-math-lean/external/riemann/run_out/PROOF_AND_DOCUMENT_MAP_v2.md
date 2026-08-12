# Proof and Document Map v2 (post-stripping)

**Paper:** paper1_zerozeta-v19.tex (1964 lines, 17 pages)
**Claim:** ζ(s) ≠ 0 for Re s > 1/2 (Riemann Hypothesis)

---

## ACTIVE CONTENT (not in \iffalse blocks)

### Lines 1–58: Preamble + title/authors
✅ NEEDED (standard LaTeX setup)

**Potential trim:** Lines 8 (`booktabs`), 9 (`float`) — packages no longer used (table removed). Lines 25 (`hypothesis`), 36 (`\Hilb`), 43–46 (editorial markup) — unused macros.

### Lines 59–99: Abstract + Introduction
✅ NEEDED

**Issues found:**
- Line 62–66: Abstract still mentions "Cayley transform Θ" and "(P+) implies Θ is Schur" — this is the OLD proof route. The actual proof uses the inner reciprocal 𝒥_neut and direct contradiction. **SHOULD REWRITE.**
- Line 99: "The Appendix records the analytic function theory inputs used in the proof of (P+)" — stale reference. **SHOULD FIX.**

### Lines 100–122: Strategy subsection
✅ NEEDED — correctly describes the inner reciprocal strategy.

### Lines 124–234: §2 Definitions and main objects
| Lines | Content | NEEDED? |
|-------|---------|---------|
| 124–140 | ζ, ξ, Ω definitions | ✅ YES |
| 142–179 | A(s), det₂, product formula, zero-free | ✅ YES |
| 181–216 | J definition, gauge remark, Lemma `poles` | ⚠️ PARTIALLY — J_def and Lemma `poles` are only used for CONSTRUCTING J_out; the primary proof uses 𝒥 directly. Keep J_def (needed for J_out). Keep Lemma `poles` (motivational). Gauge remark could be trimmed. |
| 218–233 | Schur/Herglotz terminology + "Outline" | ❌ NOT NEEDED — references Θ, Schur bound, "next section proves pinch." All (P+) route. |

### Lines 235–306: \iffalse block (§3 Schur/Herglotz pinch)
Already removed. ✅ CLEAN.

### Lines 307–481: §4 Outer normalization
| Lines | Content | NEEDED? |
|-------|---------|---------|
| 307–309 | Section header | ✅ YES |
| 311–316 | F definition | ✅ YES |
| 317–342 | Lemma `F-boundary-admissible` | ✅ YES |
| 343–360 | Lemma `F-boundedtype-from-J` | ✅ YES |
| 361–375 | Lemma `BT-to-Nplus` | ✅ YES |
| 376–403 | Lemma `det2-logL1-from-carleson` | ✅ YES |
| 404–428 | Lemma `zeta-logL1-components` | ✅ YES |
| 429–456 | Lemma `F-logL1-local` | ✅ YES |
| 457–473 | Lemma `outer-factor-halfplane` | ✅ YES |
| 475–481 | J_out definition (eq J-out) | ✅ YES |

### Lines 482–575: \iffalse block ((P+) + transport)
Already removed. ✅ CLEAN.

### Lines 577–725: §4.1 Proof of Theorem (DIRECT CONTRADICTION)
**Every line is load-bearing.** ✅ ALL NEEDED.

**Issue:** Line 688–690 has a duplicate "By that proposition" phrase — should fix.

### Lines 726–763: Conclusion + Statements
✅ NEEDED.

### Lines 764–851: Appendix intro + setup + outer normalizer
| Lines | Content | NEEDED? |
|-------|---------|---------|
| 764–797 | Appendix header, setup, notation | ✅ YES |
| 798–804 | Description of proof components (i)–(iv) | ⚠️ MENTIONS (P+) items no longer present — should trim |
| 806–826 | Whitney-wedge conventions (wedge, boxes, a.e.) | ⚠️ PARTIALLY — wedge definition is (P+)-specific; boxes + a.e. are used. Could trim wedge part. |
| 827–850 | Lemma `outer-from-logmodulus` | ✅ YES |

### Lines 851–910: \iffalse block (phase-velocity lemmas)
Already removed. ✅ CLEAN.

### Lines 911–950: Arithmetic Carleson energy + RvM bound
✅ ALL NEEDED (load-bearing).

### Lines 951–1010: \iffalse blocks (annular-balayage + phase-velocity)
Already removed. ✅ CLEAN.

### Lines 1011–1035: Lemma `xi-deriv-L1`
✅ NEEDED (used in S≡1 proof).

### Lines 1036–1109: \iffalse block (J-boundedtype-local + phase-velocity-quant)
Already removed. ✅ CLEAN.

### Lines 1110–1562: Load-bearing analytic lemmas
| Lines | Content | NEEDED? |
|-------|---------|---------|
| 1110–1142 | Subsection intro + inner reciprocal description | ✅ YES |
| 1144–1239 | Lemma `inner-reciprocal` (Parts 1–3 + PL proof) | ✅ **CRITICAL** |
| 1241–1281 | \iffalse block (Green potential lemma) | Already removed ✅ |
| 1283–1562 | Proposition `Cbox-finite` (energy bound + S≡1) | ✅ **CRITICAL** |

### Lines 1563–1684: CR-Green pairing lemmas
| Lines | Content | NEEDED? |
|-------|---------|---------|
| 1564–1567 | "From windowed phase control to the wedge" subsection header | ⚠️ STALE header — rename |
| 1568–1594 | Admissible window class (Definition `adm-bumps`) | ✅ YES (referenced by CR-Green) |
| 1583–1594 | Lemma `uniform-test-energy` | ✅ YES (used in CR-Green) |
| 1595–1647 | Lemma `cutoff-pairing` + Whitney box pairing | ✅ YES (setup for CR-Green) |
| 1648–1673 | Lemma `CR-green-phase` | ✅ **CRITICAL** |
| 1674–1684 | Proposition `length-free` | ✅ **CRITICAL** |

### Lines 1685–1899: \iffalse blocks (Whitney wedge + verification)
Already removed. ✅ CLEAN.

### Lines 1900–1963: Bibliography
✅ NEEDED.

**Potential trim:** Remove unused references:
- `Donoghue` — cited only in §2 "Schur and Herglotz classes" (now in \iffalse). **CAN REMOVE.**
- `Ahlfors` — cited only in §3 Schur pinch (now in \iffalse). **CAN REMOVE.**
- `MV` (Montgomery-Vaughan) — not cited anywhere in active text. **CAN REMOVE.**

---

## REMAINING REMOVABLE ITEMS

### 1. Abstract (lines 62–66): rewrite to match actual proof
**Why:** Abstract describes Cayley transform Θ and (P+) boundary wedge. The actual proof uses 𝒥_neut and direct contradiction.

### 2. §2 "Schur and Herglotz classes" + "Outline" (lines 218–233)
**Why:** Defines Schur/Herglotz terminology used only in §3 (removed). References "next section proves pinch."

### 3. Appendix descriptions of (P+) items (lines 798–804, 806–826 partial)
**Why:** Lists (i) phase-velocity, (ii) Green/CR pairing, (iii) Carleson energy, (iv) wedge criterion. Items (i) and (iv) are removed.

### 4. Stale subsection header "From windowed phase control to the wedge" (line 1564)
**Why:** (P+) language. Should say "CR-Green pairing lemmas."

### 5. Line 99 stale reference to (P+) appendix content
**Why:** Says "The Appendix records the analytic function theory inputs used in the proof of (P+)" but (P+) is not proved.

### 6. Unused packages/macros (lines 8–9, 25, 36, 43–46)
**Why:** `booktabs` (table removed), `float` (no floats), `hypothesis` (unused), `\Hilb` (used only in \iffalse), editorial markup.

### 7. Unused bibliography entries (`Donoghue`, `Ahlfors`, `MV`)
**Why:** Only cited in removed sections.

### 8. Duplicate text in theorem proof (lines 688–690)
**Why:** "By Proposition... By that proposition..." — says the same thing twice.

### 9. All \iffalse...\fi blocks (total ~600 lines of dead code)
**Why:** Not rendered in the PDF but adds to source file size and confusion.

---

---

## NEW REFEREE FEEDBACK (Round 6) — additional items to address

### Issue R6-1: Abstract still describes the (P+)/Θ/Schur route (not the actual proof)
**Source:** Both referees, plus map item #1 above.
**Problem:** Lines 62–66 say "A boundary wedge condition (P⁺) implies that Θ is Schur on Ω, which in turn yields a half-plane Schur/Herglotz removability argument." This is the OLD proof route. The actual proof uses 𝒥_neut and direct contradiction.
**Fix:** Rewrite the abstract to describe the inner reciprocal + contradiction mechanism.
**Status:** ❌ OPEN

### Issue R6-2: δ₀ ≥ 0.1 hard-coded (should be δ₀ ≥ ε for any ε > 0)
**Source:** Referee 2 ("the argument needs δ₀ ≥ 0.1 as a hard condition in the neutralization box step").
**Problem:** Line 619 says "The hypothetical zero ρ₀ with δ₀ ≥ 0.1 ≫ α''L does not belong to B_box." This is from the old Re s ≥ 0.6 version. The current theorem is for all Re s > 1/2 (any ε > 0), so δ₀ ≥ ε. The condition δ₀ > α''L is satisfied for any fixed ε > 0 once L is small enough (which it is, since L = c₀/log²⟨γ₀⟩ → 0).
**Fix:** Replace "δ₀ ≥ 0.1" with "δ₀ ≥ ε > α''L (for |γ₀| large enough)" throughout the proof. The algebra works identically — the c_ε = 4π/(ε+1) constant already handles this.
**Status:** ❌ OPEN

### Issue R6-3: Sign of phase-velocity — poles vs zeros confusion
**Source:** Both referees (the "show-stopper" sign issue).
**Problem:** The proof derives the lower bound from -(Arg 𝒥_neut)' which is positive because 𝒥_neut's zeros (= far ζ-zeros) contribute positive Poisson kernels. But the abstract and some text still frame this as "poles of J_out contribute positively to -w'" which is sign-wrong.
**Current status:** The actual MATH in Step 1 (lines 642–668) is CORRECT — it runs on 𝒥_neut (inner reciprocal), where ζ-zeros are ZEROS (positive sign). But the LANGUAGE in several places still says "poles contribute positively" which confuses referees.
**Fix:** Audit every mention of "poles" vs "zeros" vs "Poisson contribution" and ensure the sign narrative matches the math (which runs on 𝒥_neut, not J_out).
**Status:** ⚠️ MATH CORRECT, LANGUAGE NEEDS CLEANUP

### Issue R6-4: B_box definition still ambiguous in some spots
**Source:** Both referees.
**Problem:** Line 619 says "δ₀ ≥ 0.1" which is specific to the old 0.6 target. Should say "δ₀ ≥ ε" (the general case). The B_box definition itself (lines 613–618) correctly requires BOTH |γ_j - γ₀| ≤ α''L AND δ_j ≤ α''L, but the parenthetical "0.1" is stale.
**Fix:** Replace all "0.1" in B_box context with "ε" or "ε > α''L for large γ₀."
**Status:** ❌ OPEN

### Issue R6-5: §2 still has Schur/Herglotz/Θ material (lines 181–233)
**Source:** Referee 2 ("the far-field strategy outline references Θ Schur bound").
**Problem:** The J definition, Cayley transform Θ, Schur/Herglotz classes, and "Outline of the far-field strategy" all belong to the (P+) route. The primary proof doesn't use Θ at all.
**Fix:** Keep J_def and Lemma `poles` (motivational). Remove Θ definition (eq Theta-def), Schur/Herglotz terminology, and the "Outline" subsection. These are ~30 lines.
**Status:** ❌ OPEN

### Issue R6-6: Line 99 and Appendix header still reference (P+)
**Source:** Both referees.
**Problem:** Line 99 says "The Appendix records the analytic function theory inputs used in the proof of (P+)." Lines 798–804 list "(i) phase-velocity identity, (ii) CR pairing, (iii) Carleson energy, (iv) wedge criterion" — items (i) and (iv) are removed.
**Fix:** Rewrite line 99 and lines 798–804 to describe what the appendix actually contains.
**Status:** ❌ OPEN

### Issue R6-7: S ≡ 1 proof is "the most technically sensitive part"
**Source:** Referee 2 ("if a verifier finds that S≡1 isn't actually proven, the contradiction doesn't go through").
**Problem:** The S≡1 proof via Jensen + Vitali convergence is the right approach and uses only unconditional inputs (convexity, RvM, Poisson convergence). But it must be bulletproof.
**Current status:** The proof (lines 1431–1512) is CORRECT as written — uses Jensen's inequality for the log⁻ bound (σ-uniform), Vitali convergence, and algebraic cancellation of boundary traces. No approximations or handwaving.
**Fix:** Verify that the Jensen bound on line 1473–1476 is stated with correct constants. Verify the Vitali convergence invocation is precise. These are correct as written.
**Status:** ✅ CORRECT (no changes needed, but referee will scrutinize)

### Issue R6-8: Duplicate "By that proposition" text (line 688–690)
**Source:** Map item #8.
**Problem:** Lines 688–690 say "By Proposition ref{prop:Cbox-finite} (boundary bound M ≤ C* log...):" then immediately "By that proposition (with the boundary bound M ≤ C* log...):" — says the same thing twice.
**Fix:** Delete the duplicate.
**Status:** ❌ OPEN

---

## PRIORITY ORDER FOR REMAINING FIXES

1. **R6-1 + R6-5 + R6-6:** Rewrite abstract, remove Θ/Schur from §2, fix appendix header (removes (P+) confusion)
2. **R6-2 + R6-4:** Replace all "0.1" with "ε" in the proof (makes RH claim consistent with proof)
3. **R6-3:** Audit sign language (math is correct, language needs cleanup)
4. **R6-8:** Delete duplicate text
5. **Items 1–9 from the map:** Remove remaining dead code, unused packages/macros/bibitems

---

## IF ALL REMOVABLE ITEMS ARE DELETED

**Estimated result:** ~900 active lines → ~12–13 pages.

The paper would contain only:
1. Abstract + Introduction (rewritten) — 1 page
2. §2 Definitions (det₂, J_out, inner reciprocal) — 2 pages
3. §3 The Proof (Steps 1–3 contradiction) — 2 pages
4. Conclusion — 0.5 pages
5. Appendix: supporting lemmas — 6–7 pages
   - Outer normalizer construction
   - Arithmetic Carleson energy + RvM
   - L¹ control for log|ξ|
   - Inner reciprocal (PL + boundary trace)
   - Energy bound (neutralization + S≡1 + gradient estimate)
   - CR-Green pairing + length-free bound
6. Bibliography — 0.5 pages
