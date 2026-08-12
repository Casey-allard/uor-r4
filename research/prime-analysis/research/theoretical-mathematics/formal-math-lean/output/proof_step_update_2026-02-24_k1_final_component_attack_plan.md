# Proof Step Update (2026-02-24): Final K1 Component Attack Plan

## Current single blocker
Unconditional closure still requires one concrete non-circular theorem payload that can feed:

- `PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.K1SourceNonCircularProvider.theorem_term`
- equivalent in current lock: `Nonempty ConcreteLinearPhaseWitnessProvider`

## New formal reduction completed in this step

### A) Weakened analytic target added (strictly easier than exact linear phase)
File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean`

Added:
- `ExplicitFormulaAsymptoticallyLinearPhaseAssumptions` (`:398`)
- `linearPhaseWitnessAssumptionsOfAsymptoticallyLinear` (`:425`)
  - proves: asymptotically linear phase (phase difference -> 0 at `atTop`) + global decomposition
    implies full `ExplicitFormulaLinearPhaseWitnessAssumptions`
- `ConcreteAsymptoticallyLinearPhaseProvider` (`:1008`)
- provider lift to `ConcreteLinearPhaseWitnessProvider` (`:1012`)
- RH closure theorems from this new provider (`:1046`, `:1052`)

### B) Final lock now accepts this weaker path
File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean`

Added:
- `nonempty_concrete_linear_phase_witness_provider_of_nonempty_concrete_asymptotically_linear_phase_provider` (`:757`)
- `rh_of_nonempty_concrete_asymptotically_linear_phase_provider` (`:764`)

Effect:
- final closure can now be attacked via a weaker analytic object
  (`ConcreteAsymptoticallyLinearPhaseProvider`) instead of requiring exact linear phase from the start.

## External-source fit audit (latest)

Primary references checked against the required theorem shape:

1. Schlage-Puchta (2019): [arXiv:1912.00853](https://arxiv.org/abs/1912.00853)
- Provides interval-localized oscillation consequences from a given off-line zero.
- Good fit for interval-oscillation boundary; does **not** directly provide full linear/asymptotically-linear phase witness contract used by final K1 source.

2. Révész (2022): [arXiv:2202.01837](https://arxiv.org/abs/2202.01837)
- Strengthens interval localization constants in related Beurling setting.
- Again interval-oscillation class; no direct drop-in for current K1 witness contract.

3. Pintz (2018 preprint in this chain): [arXiv:1809.03134](https://arxiv.org/abs/1809.03134)
- Emphasizes order/magnitude and omega-style consequences under zero assumptions.
- No direct explicit contract found that matches our exact Lean field
  `zero_to_linear_phase_witness` or the new asymptotically-linear-phase field as currently formalized.

4. RH global status check: [Clay Mathematics Institute](https://www.claymath.org/millennium/riemann-hypothesis/)
- RH remains open; no external unconditional theorem term is available to import as a finished closure artifact.

5. Finite verification context (non-closure): [Platt–Trudgian 2021](https://arxiv.org/abs/2004.00423)
- Massive finite-height verification supports guardrails but does not yield an all-height unconditional theorem term.

## Empirical step started now (reusing cached zero data)

Ran:
- `python3 research/k1_source_shape_probe.py ... --max-zeta-zeros 20000 --x-min 1e7 --x-max 1e20 --grid-size 8192`
- output: `/Users/adminamn/Documents/New project/research/output/k1_source_shape_probe_2026-02-24_asymp_linear_attack.json`

Observed (overall best finite-range fit):
- `beta=0.60`, `tau=14.134725142` (first zero frequency)
- tail ratio (sup/amp) `~1.83`
- interpretation: `weak_single_mode_dominance_finite_range`

Stability check across zero counts (`m=512, 2048, 8192, 20000`) stayed in same weak regime (tail ratio ~1.73 to ~1.84).

Interpretation:
- one-mode dominance is present but not strong enough, on this probe, to treat the final witness as numerically settled.
- this supports the need for a stricter analytic derivation path instead of direct numeric promotion.

## Work plan (sequential, executable)

1. **K1-W1 (done):** reduce final blocker to asymptotically-linear phase contract and wire it into RH closure path.
2. **K1-W2 (next):** formalize a two-mode/finite-mode extension contract (`phase = arg(sum_k c_k e^{i tau_k log x}) + o(1)`) and prove reduction to asymptotically-linear phase under explicit mode-separation assumptions.
3. **K1-W3:** build a deterministic mode-separation probe over cached zero data to identify parameter regions where W2 assumptions are numerically plausible.
4. **K1-W4:** map W2 assumptions to published theorem candidates (if any) and classify each assumption as imported/proved/open.
5. **K1-W5:** if no literature closes W2 assumptions, derive in-repo lemmas for the remaining assumptions one by one; keep each as an independent theorem contract.
6. **K1-W6:** instantiate `ConcreteAsymptoticallyLinearPhaseProvider` and propagate to final RH closure check.

