# INC-0147: Stage 4 Lambda Control — Raw Alpha vs Levi-Civita Transport

## Status
Closed: REFINE.

Raw alpha (λ=0, no transport) achieves 66.7% rel_diff at K=75 — just 2.0pp below full transport
(λ=1, 68.7%). Gap is within the 5pp REFINE threshold. Stage 4 finding revised: the **Hopf fiber
phase coordinate alpha** (raw mean phase (θ₁+θ₂)/2) is the primary source of improvement (+20.6pp
over 2D base). The Levi-Civita connection correction contributes no meaningful additional
discrimination on the PPMI-SVD proxy. Stage 4 PARTIAL-PASS confirmed; mechanism revised.

## Trigger
INC-0146 Closed: KEEP (2026-03-13). HOPF_TRANS_K75 rel_diff=65.1% (+18.4pp over HOPF_BASE_K75
at 46.7%). Mathematical audit before closing Stage 4 identified a critical ambiguity:

The transport formula is:
```
transported_alpha = alpha + (λ/2)·cos(2χ)·δ
```
At **lambda=0**, `transported_alpha = alpha` (raw fiber phase, no transport).
At **lambda=1**, `transported_alpha = alpha + (1/2)·cos(2χ)·δ` (Levi-Civita correction applied).

INC-0146 used only lambda=1.0, so the 65.1% result is consistent with two distinct hypotheses:
- **H_transport:** The Levi-Civita correction `(1/2)cos(2χ)·δ` specifically adds discriminative signal.
- **H_alpha:** Raw fiber alpha alone carries the same signal; transport adds nothing.

Without a lambda=0 control these hypotheses are indistinguishable. This increment resolves the
ambiguity before any Stage 4 finalize or Stage 5 escalation.

**This is not a grid sweep.** It is a mandatory 3-point isolation test to precisely characterize
what INC-0146 actually confirmed about Stage 4.

## Kill-List Stage
Primary: **4. Phase Transport Usefulness**

## Mathematical Object Under Test
The isolation of the Levi-Civita connection 1-form from the raw fiber phase coordinate.

In `phase4d_hopf_transport`, the routing address is `(chi_u, delta, transported_alpha)`.
The parameter `phase_transport_lambda` (λ) scales the connection correction:
```
connection_weight = (λ/2) · cos(2χ)
phase_shift       = wrap_to_pi(connection_weight · δ)
transported_alpha = wrap_to_pi(alpha + phase_shift)
```
Algebraically: `transported_alpha = (1/2)(1 + λcos2χ)·θ₁ + (1/2)(1 - λcos2χ)·θ₂`

At λ=0: purely the fiber phase alpha = (θ₁+θ₂)/2.
At λ=1: chi-weighted combination of θ₁ and θ₂ (where chi measures which complex pair dominates).
At λ=0.5: intermediate regime.

The λ sweep isolates whether the chi-dependent correction (the actual transport geometry) is
responsible for the signal, or whether the simpler raw alpha coordinate is sufficient.

**Geometric interpretation of the improvement from λ=0→1:**
When rho₁>>rho₂ (chi≈0): transported_alpha → θ₁ (first complex pair's phase dominates)
When rho₁≈rho₂ (chi≈π/4): cos(2χ)=0 → transported_alpha = alpha (symmetric, no correction)
When rho₂>>rho₁ (chi≈π/2): transported_alpha → θ₂ (second complex pair's phase dominates)

So transport is geometrically meaningful: it selects the dominant complex-pair's phase angle.
If λ=1 >> λ=0, the PPMI embedding has tokens where norm dominance (chi) is aligned with phase
structure — i.e., the connection 1-form captures real semantic geometry in this proxy.

## Hypothesis
**H_transport (expected KEEP):**
HOPF_TRANS_K75 at λ=1 significantly outperforms λ=0 (by >8pp rel_diff). The chi-dependent
weighting selects the dominant complex-pair phase angle, which carries additional semantic
structure beyond the symmetric mean phase alpha. The Levi-Civita correction genuinely adds
routing discrimination.

**H_alpha (alternative, would require REFINE of Stage 4 claim):**
λ=0 and λ=1 yield similar rel_diff (within 5pp). Raw alpha is sufficient; transport adds no
meaningful correction. Stage 4 claim should be softened to "fiber alpha confirmed" rather than
"Levi-Civita transport confirmed."

## Decision Criteria

### KEEP — transport specifically adds signal:
- `rel_diff(L1) - rel_diff(L0) > 8pp` (clear gap above noise)
- Stage 4 claim stands: "Levi-Civita transported alpha confirmed on PPMI-SVD proxy"
- Next: 4-seed finalize at K=75 (HOPF_TRANS_K75_L1 + HOPF_BASE_K75) OR proceed to Stage 5

### REFINE — alpha carries signal, transport is neutral:
- `|rel_diff(L1) - rel_diff(L0)| ≤ 5pp` (within noise band)
- Stage 4 finding revises to: "fiber alpha (raw) confirmed on PPMI-SVD proxy; Levi-Civita
  correction adds no additional discrimination on this data"
- Transport formula is a valid approximation but the correction term is noise-level here
- Next: 4-seed finalize at K=75 using HOPF_TRANS_L0 (raw alpha) OR L1 (transport), then Stage 5

### KILL transport claim — raw alpha is actively better:
- `rel_diff(L0) > rel_diff(L1) + 5pp`
- Stage 4 finding: fiber alpha confirmed; transport HURTS discrimination (possible address folding)
- Investigate: does the correction push semantically different tokens into the same sector?
- Next: run with L0 for 4-seed finalize; investigate why transport degrades

### Inconclusive screen (λ=0.5 intermediate):
- λ=0.5 very close to max of {L0, L1}: smooth monotone but shallow slope
- Would warrant λ=0.1, 0.3 scan at confirm stage to find crossover point

## Minimal Scope
1. No code changes — `phase4d_hopf_transport` with `phase_transport_lambda` already tested at
   0.0 in prior configs (e.g., INC-0090, INC-0125).
2. Screen (1 seed=0), 4 route pairs at K=75:
   - `HOPF_BASE_K75_ORIG/PERM`: reference (INC-0146 confirmed baseline)
   - `HOPF_TRANS_K75_L0_ORIG/PERM`: lambda=0.0, raw alpha as 3rd coordinate
   - `HOPF_TRANS_K75_L05_ORIG/PERM`: lambda=0.5, partial transport
   - `HOPF_TRANS_K75_L1_ORIG/PERM`: lambda=1.0, full transport (INC-0146 replication)
3. Secondary diagnostics: `phase_transport_shift_abs_mean` must be >0 for L1, =0 for L0.
4. This is a SCREEN only. If results are clear (>8pp gap) → decision; if ambiguous → 2-seed confirm.

## Success Condition
- `phase_transport_shift_abs_mean ≈ 0` for λ=0 (confirms lambda=0 disables transport)
- `phase_transport_shift_abs_mean > 0.05` for λ=1 (confirms transport is active)
- A clear monotone or non-monotone pattern in rel_diff(L0), rel_diff(L05), rel_diff(L1)
- Explicit KEEP / REFINE / KILL decision recorded in ## Status

## Falsification Condition
**REFINE/KILL transport claim (either of):**
- `|rel_diff(L1) - rel_diff(L0)| ≤ 5pp` → raw alpha already captures the signal; Levi-Civita
  correction adds noise or is irrelevant on PPMI-SVD; rephrase Stage 4 claim accordingly.
- `rel_diff(L0) > rel_diff(L1) + 5pp` → transport actively degrades discrimination; possible
  that the correction folds distinct-address tokens into the same sector.

## Scope Guardrails
- Do NOT add more lambda values beyond {0.0, 0.5, 1.0} in the screen — this is isolation, not
  a sweep.
- Do NOT add HOPF_FULL here — it uses a different coordinate system (phase angles with gauge
  transformation) and is not the subject of this control.
- Do NOT proceed to 4-seed finalize before this is resolved.
- Do NOT begin Stage 5 before recording the decision here.

## Acceptance
- Screen artifact in `results/analysis/inc0147_phase_transport_lambda_control_screen.json`
- `phase_transport_shift_abs_mean` verified: ≈0 for L0, >0.05 for L1
- Explicit KEEP / REFINE / KILL decision recorded in ## Status with one-sentence rationale

## Results

### Screen (seed=0, K=75)

| route_id | pmax_after (ORIG) | pmax_after (PERM) | rel_diff | alpha_bins | shift_abs_mean |
|---|---|---|---|---|---|
| HOPF_BASE_K75 | 0.06520 | 0.04080 | **46.0%** | 0 | — |
| HOPF_TRANS_K75_L0 (raw alpha) | 0.06080 | 0.03040 | **66.7%** | 3 | 0.000 |
| HOPF_TRANS_K75_L05 (partial) | 0.07040 | 0.03320 | **71.8%** | 3 | 0.256 |
| HOPF_TRANS_K75_L1 (full, INC-0146 replicate) | 0.07040 | 0.03440 | **68.7%** | 3 | 0.512 |

Key observations:
- **L1 − L0 gap = +2.0pp** (68.7% vs 66.7%): within the 5pp REFINE threshold → transport correction is not the primary mechanism.
- **L0 − BASE gap = +20.6pp** (66.7% vs 46.0%): fiber coordinate alpha ALONE accounts for essentially all of the improvement confirmed in INC-0146.
- **L05 peak (71.8%)**: partial transport (λ=0.5) is marginally higher than both L0 and L1; likely noise at single seed; does not change the REFINE decision.
- **INC-0146 replication**: L1 seed=0 = 68.7% matches INC-0146 seed=0 exactly (confirmed consistent).
- **shift_abs_mean validation**: L0=0.000 (transport disabled ✓), L05=0.256 ≈ L1/2 (λ=0.5 halves shift ✓), L1=0.512 > 0.05 threshold ✓.
- **alpha_bins = 3** for all HOPF_TRANS variants (K=75 gives kalpha=3 ✓).

**Decision: REFINE** — fiber alpha coordinate confirmed (+20.6pp over base); Levi-Civita correction not differentially useful on PPMI-SVD proxy.

Stage 4 revised claim: "Adding the Hopf fiber coordinate alpha = (θ₁+θ₂)/2 as a 3rd routing dimension (alongside chi_u, delta) raises discrimination from 46.0% to 66.7% relative difference over col-perm baseline at K=75. The specific Levi-Civita transport correction (λ=1 vs λ=0) adds only +2.0pp — noise level at screen stage. The mechanism is the fiber phase coordinate, not the transport integral."
