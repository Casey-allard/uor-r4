# Consistent geometric contextual emitter — executed contract

September 21, 2026. Executes the [principal contextual-emission review](contextual-emission-review-2026-09-21.md)
and the [consistent-emission prompt](deepseek-consistent-emission-step-2026-09-21.md) on reviewed parent
`f07568f8`. The contract below was fixed from the review's prescriptions **before** the fit; the fit
outcomes did not choose it. The executed root is `consistent-emission-2`; `consistent-emission-1` is a
diagnostic (pre-collision-report) attempt.

## Objective

Learn a shared emitter that preserves the relevant payload distinction and actually changes an uncopied
generated answer when older evidence changes, at the **existing** 120-state, width-16 family. Do not
widen on the withdrawn capacity diagnosis.

## Numerical and causal contract (all applied)

- **Calibrated units.** Integer logits are fixed-point at the parent's `f_bits = 10`; both the float
  objective and the served evaluation convert with `2^-f_bits`. The old "15561 bits" figure is gone;
  the corrected fit reports 19.86 → 9.32 (float) / 10.11 (served) bits.
- **Shift fixed before fitting.** The declared residual shift is chosen from the development deficit
  and written into the residual object *before* `train_output_map`, so training and serving share it.
- **Ternary straight-through training.** The forward pass uses the ternary weights that serving
  executes; gradients pass straight through to the latent float map. The deployed map is what is fitted.
- **One target-free reader.** Extraction and serving both call `read_step`; no `choose_scored` path.
  Intended versus selected occurrence and payload value are recorded separately.
- **Value distinction preservation.** Value codes are initialized injectively on the observed value
  bank and the map search uses a **collision-aware objective**
  (`mean margin − 1e6 × incompatible-output collisions`), so the search cannot buy margin by merging
  values the task requires to differ. Collisions are reported before/after.
- **Loaded artifacts before evaluation.** Parameters and residual are written, independently reloaded,
  and the loaded objects drive every reported prediction; a full-predictor comparison fails closed on
  mismatch.

## Populations

Development `0xC0F00011`, tune `0xC0F00012`, and one **untouched** final draw `0xC0F00021`; the
previously used seeds `0xC0F00001..03` are exposed regression data and are not re-used as final.

## Decision rule

Declared before final inspection: at least **50 % both-members-correct** on the new final combinations,
and a clear paired improvement over local, scalar-copy, the development-fitted constant and the
categorical selected-value emitter, with the C120 arm compared at equal initial bytes, capacity and
budget. A partial mechanism below the screen is retained with explicit limitations.
