# Card P4 — Geometry controls (before any more geometry)
Owner (human): Casey        Drafted by: Antigravity        Date: 2026-09-17
Signed:

Hypothesis (one sentence, falsifiable):
H4a: Fixed Riemann zeta-zero phase features demonstrate statistically significant advantage over matched-count random fixed phases on held-out next-piece accuracy; H4b: Prime-modulo root placement (`prime % 120`) demonstrates statistically significant advantage over random token-to-root assignment; H4c: Replacing 120-landmark Hamming signature distance with exact 120×120 angle-class matrix lookup changes 0 outputs (exact mathematical identity check).

Why this and not something else (link to 08 §):
Links directly to 08 §4 R2, §6 P4, and 04 §1.7/§2. The project's geometric features (zeta phases, prime indexing, 120-root projections) have operated without matched negative controls (refit models with random geometry), leaving ambiguous whether observed predictions stem from geometric structure or underlying discrete n-gram statistics.

Data:
Recovery corpus (391,725 tokens/positions) for model refitting, evaluated on the sealed Card P1 held-out test split (2,000 TinyStories-V2, 500 Simple-Wiki). Opened-before: recovery corpus yes, P1 held-out no.

Matched controls:
Each control is a *matched refit* of identical parameter capacity on identical training data, not a runtime zeroing/disable:
1. Control for Zeta: Random fixed phase angles drawn uniformly from $[0, 2\pi)$ matching channel count.
2. Control for Prime placement: Random uniform bijection/surjection of token vocabulary onto 120 roots.
3. Control for Geometry: Unstructured categorical count baseline (5-gram / table).

Primary metric + threshold:
Held-out next-piece prediction accuracy (%) and Bits-Per-Byte (BPB); delta $\Delta\text{BPB}$ between geometric model and matched random control with bootstrap 95% confidence intervals. Threshold: $\Delta\text{BPB} > 0.05$ with $p < 0.01$.

Secondary metrics:
Angle-class equivalence delta (must be exactly 0 discrepancies for H4c), training time, memory consumption during counting.

Budget:
Engineering: ≤ 1 week. Machine: ≤ 6 h (counting fits are computationally lightweight). Storage: ≤ 1 GB. Wall-clock cap: 8 h.

Kill criterion (pre-registered):
If both zeta-zero phase features and `prime % 120` placement fall within the bootstrap 95% confidence interval of their random controls ($\Delta \approx 0$), primes, zeta zeros, Hopf fibration, and E8 are retired from the critical representation learning path permanently, retaining $H_4 / \mathbb{Z}[\phi]$ lookup tables strictly as a discrete state substrate. Decision recorded in `docs/integration/DECISIONS.md`.

Preservation:
All 7,680 historical deterministic verification cases and existing exact memory test suites must pass without regression.

Deliverables:
- Row in `docs/integration/EVIDENCE.md`.
- Full statistical report with confidence intervals in `docs/integration/cards/P4-geometry-controls-RESULT.md`.
- Pre-registered decision entry in `docs/integration/DECISIONS.md`.
