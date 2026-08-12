# K1 Source Import Feasibility (2026-02-19)

## Question
Can the remaining source theorem term be imported directly as an existing formal term, rather than derived in-repo?

Target term family:
- `PrimeRiemannBridgeOscillatoryReduction.ZeroToCosSinPhaseTransfer`
- canonical reduction target: `LogDerivativeLinearPhaseKernelTerm` / `LinearPhaseKernelTerm`

## Sources checked

1. Mathlib docs for zeta/RH
- https://leanprover-community.github.io/mathlib4_docs/Mathlib/NumberTheory/LSeries/RiemannZeta.html
- Result: has `RiemannHypothesis : Prop`, zeta/functional-equation infrastructure, no theorem term matching K1 source transfer.

2. AFM 2025 (Loeffler–Stoll) formalization paper
- https://doi.org/10.46298/afm.15328
- https://afm.episciences.org/15954/pdf
- Result: confirms Lean formalization of zeta/L-functions and Dirichlet's theorem; does not provide an unconditional RH proof or our K1-source theorem term.

3. Pintz 2017 explicit-formula oscillation paper
- https://doi.org/10.1134/S0081543817010163
- https://journals.rcsi.science/0081-5438/article/view/174251
- Result: strong oscillation/order statements linked to zero distribution; no ready-made Lean theorem object with exact K1 signature.

4. Pintz 1980 (Ingham-related)
- http://eudml.org/doc/205693
- Result: relevant analytic theorem lineage, but not available as a Lean theorem term in this repo.

5. Isabelle AFP: PNT with remainder
- https://www.isa-afp.org/entries/PNT_with_Remainder.html
- Result: high-quality formalization in Isabelle/HOL; not directly importable as a Lean term (cross-prover translation would be required).

6. Lean-RH repository (GitHub, Lean 3-era project)
- https://github.com/AlexKontorovich/Lean-RH
- Result: independent RH formalization project with its own assumptions and Lean toolchain profile; not a drop-in Lean 4 theorem object for this repo's K1 source signature.

7. Lean Millennium Prize statements repository (Lean 4)
- https://github.com/lean-dojo/LeanMillenniumPrizeProblems
- Result: formalizes Clay problem statements (including RH statement alignment), explicitly not a repository of RH solutions/theorem-term closure.

## Practical conclusion
- No exact drop-in Lean theorem term for K1-source was found.
- Best external assets are mathematical references and non-Lean formalizations.
- Current in-repo bridge status (2026-02-19 update):
  - `LogDerivativeLinearPhaseKernelTerm -> LinearPhaseKernelTerm` is derived.
  - `LinearPhaseKernelTerm -> LogDerivativeLinearPhaseKernelTerm` is derived.
  - Kernel equivalence is proved, so only one concrete non-circular source witness remains to be supplied.
- Additional in-repo status (2026-02-19):
  - `ZeroToCosSinPhaseTerm ↔ RHStatement` is now derived in `PrimeRiemannBridgeSpinningTopFrontier`.
  - This confirms the remaining source witness is not a minor bridge artifact; it is equivalent to the core RH content in this framework.
- Therefore immediate path is to derive/import exactly one concrete source theorem term in either equivalent kernel form, then apply existing K1->RH bridges.

## Verification note (local Lean synthesis)
Current workspace still has no concrete provider instances for the final source slots by default:
- `PublishedZeroToCosSinPhaseProvider`
- `PublishedLinearPhaseWitnessProvider`
- `ConcreteImportedLinearPhaseWitnessProvider`

(Checked via temporary Lean `#synth` probe; synthesis fails for all three.)
