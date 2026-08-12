# K1 W29 Online Source Map (2026-02-24)

## Objective
Identify primary-source mathematics that can support derivation of the remaining in-repo target:
- `R6DualBandTailRepresentationCandidateShapeTerm`

## Current local conclusion
Local artifacts provide finite-window numerical evidence, not theorem-level closure for:
- exact equality of normalized tail with a finite 256-mode list for all `x >= 10^21`
- universal quantification over all admissible `E` and all right-half zeros

See:
- `/Users/adminamn/Documents/New project/research/output/k1_w29_local_research_audit_2026-02-24.md`

## Primary sources checked

1. Schlage-Puchta (2019): zero-right-of-1/2 => large oscillation
- arXiv: [1912.00853](https://arxiv.org/abs/1912.00853)
- Usefulness: supplies lower-oscillation forcing from off-line zeros (structural direction), not finite-mode exact tail identity.

2. Bellotti (2024): explicit log-free zero-density estimate
- arXiv: [2405.12545](https://arxiv.org/abs/2405.12545)
- Usefulness: explicit `N(σ,T)` control for `σ` near 1; can feed tail-sum majorant lemmas via zero-density inputs.

3. Johnston (2024/2025): zero-density to optimal PNT error transfer
- arXiv: [2411.13791](https://arxiv.org/abs/2411.13791)
- Usefulness: blueprint for turning zero-density + zero-free data into explicit `ψ(x)-x` error envelopes; relevant for proving residual-power majorants.

4. Bellotti (2025): stronger near-1-line density + optimal error term
- arXiv: [2508.02041](https://arxiv.org/abs/2508.02041)
- Usefulness: strongest current explicit density/error-term link in this chain; useful for improved explicit constants in remainder bounds.

5. Fiori-Kadiri-Swidinsky (2022/2023): explicit unconditional bounds for `ψ(x)-x`
- arXiv: [2204.02588](https://arxiv.org/abs/2204.02588)
- Usefulness: robust explicit baseline bounds and region-splitting strategy for practical tail-error control.

6. RH status check (problem remains open)
- Clay Mathematics Institute: [Riemann Hypothesis](https://www.claymath.org/millennium/riemann-hypothesis/)

## Math-path implications for the last term

To close the current term *as written*, we would need to prove an exact all-`x` finite-mode identity (256 modes) on `x >= 10^21`.
Current source scan suggests this exact identity is not available directly in published form.

Most realistic derivation route from available theory:
1. Derive explicit-formula truncation theorem:
   - finite mode sum (first `N` spectral components)
   - plus explicit residual term from omitted zeros.
2. Prove explicit residual majorant:
   - `|residual(x)| <= C x^{-eta}` for `x >= x1`
   - constants informed by zero-density + zero-free region inputs.
3. Promote into Lean as theorem payload.

If exact finite equality is still required, an additional theorem would be needed showing the residual is identically zero on `x >= x1`.
No evidence for that currently exists in local or published sources.
