# Bounded independent Metal harmonic-floor diagnosis

Reviewer: Codex `/root/second_council_review`, delegated by `/root`.
Source: `dcb8b107f499395c65b86404d67e9e83916054fd` in `/Users/casey.allard/.codex/worktrees/track-b-conversion/uor-r4`.
Date: 2026-09-30. No code edits, builds, tests, model runs, GitHub writes or infrastructure work performed. This document is a source/math diagnosis and proposed discriminating measurement, not an executed repair.

## What is established

The retained actual Metal test failed at d=32, L=3, perturbation=0.01:

- actual = 1.5497062122449279e-6;
- expected = 1.0000000117348499e-6;
- error = approximately 5.4970620051e-7, exceeding the unchanged approximately 5.00000000235e-7 bound;
- the ideal polynomial contribution above delta is only about 1.173485e-14.

This is a real **Metal numerical failure**, not a model-quality finding. Parent reports the CPU test passes; I did not execute either backend. The failure log is `track-b-dcb8b107-metal-floor.log`, SHA256 `8dd02deb41bd2f4e1959d0123e94f66e376ef2b5d643aad50032254251701106`. Keep delta=1e-6, the frozen shifted-power kernel, registered d/L arm and acceptance tolerance unchanged.

## Actual arithmetic path

`harmonics.rs:84–109` normalizes on the selected device using an F32 sum, sqrt and division. The floor test exports those resulting unit rows and computes the reference cosine in host F64 with their measured norm product (`:613–634`). Therefore CPU and Metal test passes are not automatically comparisons on identical normalized input bits.

Feature construction includes rounded fixed Householder reflection coefficients (`:125–159`), a projection reduction plus subtraction (`:171–173`), quadratic products/scales (`:201–208`), and cubic products, scaled repeated-index groups, trace removal and another scale (`:268–287`). Weighted band coordinates are concatenated (`:410–417`). These are all possible construction-rounding locations.

For d=32, L=3, the packed bands have 1+32+527+5952 = **6512 coordinates**. The ideal normalized Gegenbauer bands at exact antipodes alternate in sign; their positively weighted Gram contributions cancel to leave delta. This mixes order-one band contributions to recover an order-1e-6 score. Small feature or accumulation errors can therefore matter substantially to the floor even when a broad Gram tolerance passes.

The test's actual score is the full F32 feature Gram matmul (`:633`). The compiled log names the repository-patched Candle core, so I inspected **`third_party/candle-core-0.9.2/src/metal_backend/mod.rs:1675–1708`**, not only the registry copy: F32 dispatches to `GemmDType::F32` and `call_mlx_gemm`. Pinned `candle-metal-kernels-0.9.2` uses float accumulators and `simdgroup_multiply_accumulate` in `metal_src/mlx_gemm.metal:244–245,275–305,367–377`. Its reductions use a different parallel order, and affine uses F32 FMA. Source does **not** justify blaming an implicit BF16/F16 conversion or asserting which operation caused this failure.

## Smallest causal diagnostic: identical rows, separated construction and reduction

Run one tiny source-bound diagnostic focused on the failing case, optionally retaining its exact-antipode neighbor as a control. Do not run models or change the acceptance test.

1. Construct/normalize on Metal exactly as the failed test. Export the two resulting F32 unit rows once; record F64 squared norms, raw dot and normalized cosine. Both construction backends below must receive these **identical exported F32 rows without renormalizing again**.
2. Compute Metal features F_M once and export their F32 bits. Record **A**, the existing Metal matmul score. Compute **B**, a host-F64 sum of products of these exact exported feature values. A-B isolates the final Metal Gram reduction from feature construction.
3. Construct CPU features F_C from the same exported unit rows, using the same coefficients and source. Compute **C**, a host-F64 dot of those F32 features. B-C measures device-dependent feature construction error; per-coordinate max/RMS differences and per-band F64 dots localize it. CPU matmul of F_M is an optional cheap extra reduction control, not a replacement for F64 accumulation.
4. Record **D**, the unchanged ideal reference `delta + ((1+cosine)/2)^L`. C-D includes common finite-feature/coefficient and sphere-normalization residual error. The decomposition A-D = (A-B)+(B-C)+(C-D) keeps attribution honest.
5. Record F64 product sums separately for h0/h1/h2/h3 and each band's feature norms. If B-C dominates, compare exported unweighted `bands()` to distinguish Householder/product construction from final sqrt(weight) scaling. If needed, compute the real-arithmetic homogeneous-band prediction using raw t=x.y and n=||x||^2||y||^2:
   - h0 gram = 1;
   - h1 gram = t;
   - h2 gram = (d*t^2 - n)/(d-1);
   - h3 gram = ((d+2)*t^3 - 3*t*n)/(d-1).
   Weight these by the same b_l. Comparing this with D separates slight off-sphere input effects from feature-coordinate rounding; do not alter the test's reference.

Interpret magnitudes together rather than picking a cause from one positive residual. If A-B explains the miss and B itself satisfies the frozen tolerance, the map need not change. If B misses and B-C dominates, changing only the final reducer cannot repair already distorted features. If both matter, preserve both observations.

## Minimal stabilization choices after that measurement

**Reduction dominated:** first try an algebraically identical, explicitly ordered/compensated or chunked-balanced F32 contraction on the same feature coordinates, retaining product precision and all terms. This targets the actual failing operation without changing delta, kernel, band meaning or learned projections. Its successful Metal behavior and gradient/cost implications must be measured. Do not merely change the test to use a stable CPU oracle while the actual model still uses the failing Metal contraction. Per-band contraction can be a diagnostic variant but still has cancellation and is not guaranteed to cure the error.

**Construction dominated:** target the measured offending projection/reduction/scaling first; retain the exact mathematical map and coordinate contract where possible. A separate delta coordinate avoids rounding delta into an order-one band coefficient but cannot repair unrelated 6512-term cancellation and is not justified as the sole fix from this result. Do not clamp a negative score to delta, add epsilon, increase delta, relax tolerance, or remove the failing arm.

**Direct shifted-polynomial evaluation** from the small input dot is a useful independent numerical oracle for this tiny pair. It can avoid the expanded feature cancellation near antipodes. It is not a free replacement for a feature-based recurrent/linear-time contraction: materializing all pairwise scores changes access/cost even when the ideal kernel is identical. Keep that distinction explicit.

## Augmented symmetric tensor map: valid identity, no automatic stability guarantee

Let z(x)=(1,x)/sqrt(2), with m=d+1 coordinates. For each multi-index alpha of total degree L, use `sqrt(L!/alpha!)*z^alpha`. The multinomial theorem gives

`<Phi_L(z(x)),Phi_L(z(y))> = <z(x),z(y)>^L = ((1+x.y)/2)^L`.

Append an independent sqrt(delta) coordinate for the unchanged additive floor. This removes harmonic trace-removal arithmetic, so it is a coherent **algebraically equivalent feature realization** to investigate if construction error dominates. It requires a new feature-order/version and truthful retained artifact/cost scope; it is not a new semantic mechanism or a permission to reinterpret existing feature artifacts.

However, it does **not** eliminate cancellation in the final expanded Gram. At y=-x, terms of different x-monomial parity have alternating signs, despite the augmented coordinate being positive. Their sum realizes `(1-1)^L/2^L`. A generic F32 GEMM can still lose the small residual. Thus the proposed map is not a guaranteed cure for reduction-dominated error.

Packed symmetric degree-L dimension is C(d+L,L); with a separate delta coordinate it is C(d+L,L)+1. At L=3: d=16 gives **970** vs existing **952** (+18); d=32 gives **6546** vs existing **6512** (+34). This is a small feature-count increment, but feature extraction, state contraction, gradients and actual memory traffic still need measurement. A full unsymmetrized tensor would use (d+1)^L coordinates (35,937 at d=32,L=3) and is unnecessary.

**Recommendation:** run the A/B/C/D decomposition first, then one causally targeted numerical repair. Preserve the actual Metal failure and all unchanged acceptance conditions. CPU success or a mathematical identity alone cannot qualify Metal floor fidelity, loaded teacher parity or B2 model quality.

Source identities: harmonics.rs SHA256 `e34af495b82d9d33a30146e8135de21fc098add0a6f979f143ad54a77733cc54`; actual patched Metal backend SHA256 `16bc99ee096c581eaa83ce6cf9d007674d4eb6bb5b6e30560ccb62a3daac07a1`. An untracked `.local-recovery/` directory was present; I did not modify it or infer its contents as authoritative source.


## Diagnostic source review and executed-evidence addendum

The parent added one 71-line ignored diagnostic test to `harmonics.rs` on top of `dcb8b107f499395c65b86404d67e9e83916054fd`. I independently reviewed that exact uncommitted diff and read the executed diagnostic log. **APPROVE for this diagnostic scope; no required source fixes identified.** This is not an approval of Metal floor fidelity or loaded model parity.

The diagnostic normalizes the failing fixture once on Metal, copies those exact F32 rows without renormalizing, constructs features on each backend, and contracts each resulting matrix through host F64, CPU GEMM, Metal GEMM, and Metal elementwise multiplication plus reduction. Thus construction and final contraction are usefully separated. Host F64 products of the finite F32 inputs provide a high-precision contraction reference. The elementwise path rounds products separately and has a different reduction tree than GEMM; this experiment isolates the final contraction as dominant, not a specific GPU instruction or summation order alone. Returning test success means diagnostic execution succeeded; its `DIAGNOSTIC_ONLY` label correctly avoids treating this as an acceptance gate.

Read actual outputs, expected = 1.0000000117348499e-6:

| Construction | Host F64 dot | CPU GEMM | Metal GEMM | Metal elementwise sum |
|---|---:|---:|---:|---:|
| CPU | 9.537330709852786e-7 | 1.009088009595871e-6 | 1.550938691252668e-6 | 9.611248970031738e-7 |
| Metal | 9.538933082305312e-7 | 1.0086223483085632e-6 | 1.5497062122449279e-6 | 9.611248970031738e-7 |

For Metal-constructed features, Metal GEMM adds approximately **5.95812904014e-7** relative to the F64 contraction of the identical stored coordinates. Changing construction backend changes the F64 dot by approximately **1.60237245e-10**. The common feature/reference discrepancy is approximately -4.61e-8. Therefore the observed miss is dominated by final Metal GEMM numerical error, not the CPU-vs-Metal feature-construction difference. This conclusion is scoped to this fixture and execution.

The diagnostic log reports one executed test, success, 0.33 s test execution and 36.03 s complete elapsed including build (35.02 s reported build). I ran no test myself. A separately retained largest-arm Metal gradient test is also execution evidence for its own gradient assertion only, not the failed floor gate, loaded checkpoint parity, or B2 quality.

**Immediate next action:** preserve the unchanged failing Metal GEMM floor result, preserve this diagnosis, and continue the independent dense checkpoint parity gate with its unchanged CPU+Metal all-logit 1e-4 criterion. A source search found `HarmonicFeatures` used only within `harmonics.rs` tests in the current training source/examples; the dense model path does not call it. Consequently a harmonic reducer repair is not a prerequisite to the independent dense reference comparison. This permits useful model progress without pretending the harmonic gate passed.

Do not introduce a production helper solely to change this assertion. When the actual harmonic B2 consumer is implemented, numerical behavior of its real contraction remains unresolved: use and validate that actual operation, retaining kernel, delta, arms and tolerance, and report gradient/storage/compute implications. A tiny pairwise elementwise dot does not qualify a recurrent feature-state contraction, its accumulation over history, or its cost. No Candle-kernel rewrite or augmented-feature programme is justified by this bounded diagnosis.

Diagnostic-source SHA256: `e34af495b82d9d33a30146e8135de21fc098add0a6f979f143ad54a77733cc54`.
Diagnostic-log SHA256: `c2f2ebfd5360e454bf21e15eb9fd981af619481ada82d6e02695a7831a068c5c`.
Separate gradient-log SHA256: `b15716f44ebcd8c18459252777ba3dfb878114308be313c414234a1e1e56c9a6`.
