# Proof Step Update (2026-02-24): External Formal Source Audit for Final K1 Witness

## Goal
Find an externally formalized theorem term that can instantiate:

- `PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.K1SourceNonCircularProvider.theorem_term`
- target type: `PrimeRiemannBridgeOscillatoryReduction.ZeroToCosSinPhaseTransfer`
- constraint: non-circular (no RH-in-hypothesis and no RH-equivalent contradiction route)

## Audited candidates

### 1) `AlexKontorovich/Lean-RH`
- local clone: `research/external/Lean-RH`
- HEAD: `870f78c`
- result:
  - provides RH *formulation layer* and related eta/zeta infrastructure
  - `src/riemann_hypothesis.lean` defines `form.dirichlet_eta` as a proposition; it does not expose a concrete theorem term proving RH or an equivalent zero-oscillation transfer statement
  - no direct theorem term matching our K1 target shape
- verdict: **not importable for K1 closure**

### 2) `jonwashburn/riemann`
- local clone: `research/external/riemann`
- HEAD: `12a9db8`
- result:
  - repository contains broad programmatic and manuscript artifacts; claims of unconditional closure are documentation-level
  - Lean corpus has unresolved proof placeholders: `176` occurrences of `sorry` under `Riemann/**/*.lean`
  - at least one explicit Lean `axiom` declaration exists (`Riemann/Mathlib/MeasureTheory/Integral/Carleson/Backup.lean`)
  - no clean exported theorem term discovered that directly provides our K1 target proposition without trust gaps
- verdict: **not importable as non-circular closure term**

## Current frontier status (unchanged)

Still exactly one open mathematical object:

- construct one concrete non-circular instance of
  `PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.K1SourceNonCircularProvider`
  by supplying `theorem_term : ZeroToCosSinPhaseTransfer`.

No external audited source currently closes this object at the required trust level.

## Immediate next action

Proceed with in-repo derivation path (non-import), by deriving a concrete `ZeroToCosSinPhaseTransfer` witness from explicit assumptions already encoded in `PrimeRiemannBridgeOscillatoryReduction` / `PrimeRiemannBridgeSpinningTopFrontier`, then progressively discharging those assumptions with fully non-circular lemmas.

## In-repo reduction completed in this step

File updated:

- `research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean`

New proved equivalences:

- `Nonempty K1SourceNonCircularProvider ↔ Nonempty ConcreteLinearPhaseWitnessProvider`
- `RHStatement ↔ Nonempty ConcreteLinearPhaseWitnessProvider`

Interpretation:

- the final blocker can now be tracked directly as one concrete object:
  `ConcreteLinearPhaseWitnessProvider.linear_phase_witness_assumptions`
  with no additional ambiguity about intermediate provider families.
