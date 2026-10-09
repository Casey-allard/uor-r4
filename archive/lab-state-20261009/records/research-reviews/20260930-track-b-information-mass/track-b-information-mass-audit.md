# Track B: information preservation and selected-mass audit

Status: exact-arithmetic analysis and source inspection; Rust path NOT_RUN. Codex contributing lab, under owner-assigned Kimi leadership. No change to architecture, frozen arms, acceptance thresholds or execution admission. Source: PR1518 head1b88c563090329f6d21b8c0105a7a6e0146143f9. The unchanged reference-parity prerequisite still comes first.

## Decision this analysis can change

Before spending on harmonic/hybrid transfer, distinguish three causes of poor retrieval: failure to select the right occurrence, suppression of the selected value during normalization, and loss of information before selection. Aggregate layer MSE cannot distinguish them. The existing design already records Q/K norms and corrects harmonic/sparse overlap in numerator and denominator; this note derives additional consequences of its exact declared weights. It is not a proposal to replace the model programme.

## Source binding

- [Raw reciprocal rank weights and single normalization](https://github.com/UOR-Foundation/uor-r4/blob/1b88c563090329f6d21b8c0105a7a6e0146143f9/crates/uor-r4-training/drafts/track_b_hybrid.rs#L70): selected scores are replaced by1/(r+1), not added to the harmonic scores; line142 sums the combined mass. Recurrent correction at178/206 subtracts the selected harmonic contribution from both numerator and mass.
- [Spherical normalization](https://github.com/UOR-Foundation/uor-r4/blob/1b88c563090329f6d21b8c0105a7a6e0146143f9/crates/uor-r4-training/src/track_b/harmonics.rs#L67); [transfer caller](https://github.com/UOR-Foundation/uor-r4/blob/1b88c563090329f6d21b8c0105a7a6e0146143f9/crates/uor-r4-training/drafts/track_b_transfer.rs#L384) normalizes Q/K before learned projection and again afterward.
- [Existing design](https://github.com/UOR-Foundation/uor-r4/blob/1b88c563090329f6d21b8c0105a7a6e0146143f9/docs/integration/track-b-harmonic-design-2026-09-29.md) specifies K_L(t)=delta+((1+t)/2)^L, delta=10^-6 and L in1..3. It already warns about discarded amplitude and numerical cancellation. The five drafts are unregistered/unqualified, not deployed serving.

## Exact consequence 1: reciprocal-rank concentration ceiling

Let C=query_position+1 be the causal prefix length (equal to the full tensor sequence length only on its final row), S the actual deduplicated support, m=|S|, H_m=sum(r=1..m)1/r, and B=sum(j not in S)K_L(qhat dot khat_j). In real arithmetic, with the declared positive kernel and exact overlap replacement:

    selected mass fraction = H_m/(H_m+B)
    highest-ranked selected position weight = 1/(H_m+B) <= 1/H_m.

The inequality is strict if C>m because unselected kernel scores are positive. It does not bound the largest unselected score. For m=128 the upper bound is0.1840553887, even with zero unselected mass. For any m>=2, the highest-ranked selected position cannot receive more than2/3 of the normalized weight under these prescribed rank weights. This is about one attention row, not a limit on whole-model coding or retrieval: values, multiple heads, residuals and downstream layers may encode or cancel mixtures. It is not a universal claim that accurate language requires one-hot attention.

Here m includes deduplicated sinks, window and top-k positions; it is not synonymous with the configured k. Choosing a correct support does not alone establish a strong enough contribution from the desired occurrence. F32 rounding and signed recurrent cancellation can depart from the real-arithmetic identity and require an actual implementation check; this is not an opcode or numerical-fidelity result.

## Exact consequence 2: a constructed background can dominate

Consider a valid unit-vector configuration in dimension at least2 with every unselected key orthogonal to the query. Repeated orthogonal directions are allowed, so this construction does not need C mutually orthogonal vectors. Then each unselected score is c_L=delta+2^-L and B=(C-m)c_L.

For m=128 and L=3:

| Causal prefix C | Total selected mass | Highest-ranked selected weight |
|---:|---:|---:|
|512|10.168048%|1.871484%|
|2048|2.213680%|0.407440%|
|8192|0.536109%|0.098674%|

These are constructed exact-kernel examples, not measured distributions of projected teacher keys and not an assertion that this background is typical. Learned projections can move unselected keys toward low-score regions, and correlation between support selection and harmonic scores matters. Do not infer an expectation from the example.

At C2048,m128,L3, multiplying only harmonic background by gamma would give selected mass H/(H+gamma B). A half-mass crossing occurs at gamma=H/B≈0.02263793. This is a diagnostic calculation, not a recommended hyperparameter and not authorization to alter the frozen arm. Multiplying sparse scores by a positive alpha likewise changes their relative mass: d[alpha H/(alpha H+B)]/d alpha=HB/(alpha H+B)^2>0. Scaling every score together cancels out. Therefore score scale, not just rank/support, is part of the hybrid's mechanism and artifact contract. Scaling alone does not remove the reciprocal-rank concentration ceiling within S.

## Exact consequence 3: radial information can be erased before learning sees it

For nonzero q and positive a, q/||q|| = aq/||aq||, provided both norms remain at or above the fallback threshold and finite. A direction-only path with fixed parameters, support and values therefore cannot distinguish the two at that seam. Dot-product softmax generally can: with effective logits(1,0) versus(8,0) and scalar values(1,0), the outputs are0.73105858 and0.99966465. Any identical prediction for this equal-weight two-example construction has mean squared error at least(diff)^2/4≈0.01803731.

This is a counterexample at a fixed attention interface. It does not prove this pair occurs in the teacher corpus, that Q/K/value changes are independently realizable by a shared hidden state, or that a learned upstream/LoRA path cannot encode the distinction in direction or values. A rank-only support that stays fixed also cannot restore discarded magnitude. A magnitude-aware selector or a changed support could distinguish it; such a path must be inspected rather than assumed. The useful empirical question is how much target variation remains conditional on the representation the candidate actually retains.

## Minimal prospective diagnostics

Reuse the fixed parity windows/teacher activations when they become admitted; do not train a new sweep for this note. Freeze any additional diagnostic fields before the next result:

1. Record actual C and m, H_m, B/H_m, selected harmonic mass, unselected harmonic mass, replaced raw mass, resulting selected fraction, target-attention mass captured by S, and top selected weight, by layer/head/context distance. Keep source/selector/kernel identities and numerical-cancellation counters.
2. Use the same selected set to distinguish support miss from weighting miss. A teacher-weight-on-S oracle is an offline diagnostic with disclosed teacher access, not a deployable candidate or a fair autonomous control. Retain ordinary dense and prescribed rank arms.
3. Keep existing norm-bin operator errors and add a narrowly specified fixed-interface radial counterexample only as a representation diagnostic. Do not infer whole-model error floors from it.
4. If the support finds useful records but unselected mass overwhelms them, propose a separately versioned normalization/weighting successor to Kimi with matched information and tuning budget. If captured teacher mass is poor, prioritize addressing. If mixture mass is adequate but transfer fails, investigate projections/values/composition. No old result or threshold changes.

First actual Rust checks, after admission: quadratic/recurrent agreement for a constructed orthogonal-background row, the m-dependent weight bound within declared F32 tolerance, and consistency with actual canonical support. These are planned, not executed. Integrate them into the existing transfer path rather than create a parallel framework. No additional model fitting is justified until reference parity and consumer integration are qualified.

## Literature and computation scope

[Hedgehog, section4.1](https://arxiv.org/html/2402.04347v1#S4.SS1) connects concentration/monotonicity with attention expressivity and shows polynomial maps can recover those properties in bounded regimes at substantial feature cost. It does not prove this UOR kernel succeeds or fails. The stronger statement that harmonics cannot be spiky should not be used. The existing Scatterbrain-style numerator/denominator replacement is retained; this audit concerns the chosen reciprocal-rank magnitudes.

Wolfram evaluated the retained `track-b-mass-audit.wl` expressions using exact rational delta, harmonic numbers and symbolic derivative, with12-digit display values in `track-b-mass-wolfram-result.json`. The kernel identity and inequalities above are elementary algebra; the computation checks finite examples, not trained behavior. No Cargo, model job, GPU work, binary opcode audit or benchmark was run. This operational note is a proposal/evidence record for Kimi, not an accepted research-policy change.

Independent adversarial source/math review: `/root/independent_systems_review` checked the pinned source and formulas without executing code. Its four clarifications (causal prefix, actual support size, selected-only bound, finite ordinary normalization branch) are incorporated. Full scoped review is retained alongside this note.
