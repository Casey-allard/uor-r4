# Design: Integer Residual Wiring for the Shift-Add Dot Path (Phase B)

*Issue #318. Status: DESIGN FOR REVIEW — authorized for review only; no
kernel work until the reproduction requirement below is met. Decision
points marked ⚑ are open.
Claim language: Definitions, Objectives, and Empirical Criteria only.*

## Objective

Capture the measured #318 buildable row in the deployed kernel: dot
assignment **with per-stage residual updates**, realizable under the
runtime contract (no multiply/divide/float; P-4 machine-checked source
scan). The measured evidence (fresh-era rows-only run, 2026-07-31,
equality witness 512/512):

| Row | top-1 | agreement | WB bits | keys |
|---|---|---|---|---|
| A-f32 (ceiling) | 32.1 | 36.4 | 9.70 | 92,903 |
| A-dot-po2x2-resid (buildable shape) | 31.2 | 35.3 | 10.13 | 96,080 |
| A-dot-po2-resid | 31.1 | 35.2 | 10.18 | 89,704 |
| A-dot-po2x2 (shipped shape, no residual) | 30.7 | 34.3 | 9.64 | 47,418 |

Residual composition recovers ~36–48% of the remaining top-1/agreement
gap to the f32 ceiling, and the po2 value-set restriction costs nothing
once residuals carry the refinement (po2-resid ≥ f32 dot-resid). The
terms sweep is saturated (x3 regresses to 30.4; 2 terms is the stop).

## Definitions (the buildable kernel form)

Per sample, per stage st ∈ 0..4:

1. **Norm fold** — the measurement normalizes `work` by its L2 norm
   (f32 division; not kernel-legal). Candidate kernel form: a
   power-of-two fold `work' = work >> s` with
   `s ≈ round(log2(||work||))` derived from the L1 norm
   (`Σ|work_d|` — abs and add only): `s = bit_length(Σ|w|) − CONST`,
   where CONST is a compile-time content-addressed constant. L1→L2
   conversion error and the fold's 2× granularity both land in the
   exponent, which the dot tables absorb as a per-sample scale.
   ⚑ validate certifier-side first (row: A-dot-po2-resid with the po2
   norm fold replacing true division; must lose ≤ 0.1pp against the
   31.1/35.2 row).
2. **Assignment** — existing shift-add dot (`dot_score_plain`) against
   the per-stage po2 tables. DOT_TERMS ⚑ 1 vs 2: the x2 row leads by
   0.1pp at 2× dot cost; decide at Phase B with the op census.
3. **Residual update** — `work −= cent_int[st][code]` where
   `cent_int` are per-stage **integer centroid copies**: unit-scale f32
   centroids quantized with a per-stage power-of-two scale (the token
   stage-book precedent: exponent from IEEE bits, libm-free).
   Subtraction is add/sub — contract §2 legal. ⚑ i8 vs i16 copies
   (fidelity vs 295 KB / 590 KB artifact growth).

Artifact growth: `STAGES × K × D` integer entries + per-stage scale
exponents; format-era bump (next TLA era) with the usual note; P-4 scan
and allocation census extended to the new code.

## The WB / key-dispersion problem (must be addressed, not waived)

Residual codes disperse: keys roughly double (47k → 96k) and Witten–Bell
worsens (9.64 → 10.13 bits/token) even as argmax metrics improve. The
#318 result is therefore a **quality trade-off, not a pure win**, and
Phase B's DoD must include a store-shape answer, decided on the #244
harness (the same discipline that settled A-single vs A-multi):

- Candidate mitigations: beam-depth tuning on the residual codes;
  prefix-collapsed keys (the codes form a natural 4-deep prefix tree);
  ⚑ others from review.
- **Empirical Criterion:** the shipped form must keep ≥ 80% of the
  measured top-1/agreement gain (≥ +0.4pp / +0.8pp over the shipped
  baseline at certification-era conditions) with WB regression bounded
  at ⚑ ≤ 0.3 bits/token against the shipped 9.64. If no store shape
  meets both bounds, Phase B ships nothing: the rows stand as the
  recorded result and the ceiling is declared reached for this
  architecture (teacher upgrade, issue #320, remains).

## Reproduction requirement (before any kernel code)

Per repo discipline (#244 precedent): the #318 rows must reproduce
bit-for-bit across three rows-only runs (same corpus, same artifact κ)
before Phase B implementation is authorized. One run is on record.

## Phasing

- **Phase A.5 (certifier-side, no runtime change):** po2-norm-fold row
  (validates Definition 1) + i8-vs-i16 centroid-copy fidelity rows
  (settles ⚑ width) + the 3× reproduction. One certify run.
- **Phase B (kernel):** residual wiring in the integer kernel behind
  the format-era bump; P-4 scan extension; equality-witness against the
  plain form; op-census budget ⚑ ≤ 2× current dot-path counts.
- **Phase C (adoption):** store-shape decision on the #244 harness;
  κ re-pin with era notes (maintainer decision); BASELINE.md update.

## Explicit non-goals

No floating point, multiply, or divide in the runtime kernel. No change
to κ-label semantics. No sign-space assignment work (saturated, #310).
No teacher change (tracked separately as #320).

## Sign-off

- Casey: ____   - Ari: ____   - Alex: ____

⚑ decisions: po2 norm-fold validation · DOT_TERMS 1-vs-2 · centroid
copy width · WB regression bound · op-census budget
