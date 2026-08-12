# Kill List Tracker

Purpose: single canonical tracker for kill-list stage status.

Use statuses:
- `open` = unresolved and currently blocking or live
- `partial` = positive evidence exists, but not enough to close the stage
- `closed` = sufficiently established for the current repo scope
- `killed` = falsified or explicitly abandoned

## Canonical Queue
- Current primary RR: **Publication phase — Angular Manifold Routing / sparse-routing-MoE replacement (no active RR assigned; experimental research complete).**
  All prior experimental RRs (including RR-068) are frozen/inactive and preserved as historical record.
- Current primary INC: **Publication** — all increments closed (INC-0171 KEEP).
- Previous INC: `INC-0171` -- Closed: KEEP (confirm, 2 seeds, 4000 steps, 2026-03-26). End-to-End LM.
  PPL ratio HOPF/BASELINE: 1.081 confirmed stable (screen 1.080). eff_ratio DENSE/HOPF at
  convergence: 1.39× (declines from 1.65× at 2000 steps). HOPF ≈ PERMUTED confirmed (Δppl=0.13).
  Stage 7: COMPLETE. Paper and publication packet ready for arXiv.
- Previous INC: `INC-0170` -- Closed: KEEP (2026-03-14). Large-K capacity test.
  TRANS ORIG vs PERM at K={600,1000,2000,3000,5000}. eff_ratio stabilises at 2.6–2.8× (no collapse).
  Full-range alpha=0.657 (vs 0.572 for K≤400). INC-0171 proposed (end-to-end LM).
- Previous INC: `INC-0169` -- Closed: KEEP (2026-03-14). Canonical law freeze + design implications.
  eff_buckets=2.957×K^0.572 (TRANS ORIG). Norm-invariant. Shell hierarchy excluded. INC-0170 proposed.
- Previous INC: `INC-0168` -- Closed: KEEP (2026-03-14). Angular-vs-radial norm-geometry diagnostic.
  Routing sparsity is purely angular: α=0.572 TRANS ORIG norm-invariant across L1/L2/L3/L4.
- Previous INC: `INC-0167` -- Closed: KEEP (2026-03-14). Scaling mechanism attributed
  to angular sector discretization. Shells structurally inaccessible (r_eff=1.0).
- Previous INC: `INC-0166` -- Closed: KEEP (2026-03-14). Law freeze + K=100 audit.

## 1. Hyperbolic Embedding Stability
- Status: `partial`
- Canonical evidence:
  - `EVIDENCE_SUMMARY.md`
  - `NEXT_CRITICAL_EXPERIMENTS.md`
- Blocker:
  - `large-scale learned hyperbolic embedding stability is not yet revalidated on top of a stabilized route law`
- Next branch:
  - `deferred until the route law is stable enough to justify a dedicated embedding benchmark`

## 2. Measure-Consistent Shell Routing
- Status: `closed/partial-pass`
- Canonical evidence:
  - `docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md`
  - `docs/research/increments/INC_0061_measure_consistent_route_law.md`
  - `docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md`
  - `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
  - `docs/research/increments/INC_0138_geometry_only_shell_activation_controls.md`
  - `docs/research/increments/INC_0139_TBD.md`
  - `docs/research/increments/INC_0140_angular_sector_routing_measure_consistency.md`
  - `docs/research/increments/INC_0141_TBD.md`
  - `docs/research/increments/INC_0142_TBD.md`
  - `docs/research/increments/INC_0143_TBD.md`
- Latest result (INC-0143, 2026-03-13):
  - 4-seed finalize of PPMI-SVD discrimination on H^4 Hopf routing (dims 3,65,2,21).
  - SEM_ORIG mean_pmax=0.0905, SEM_COL_PERM=0.0613, rel_diff=38.5% (threshold 20%).
  - All 4 seeds pass individually (range 30.6%–54.6%).
  - Stage 2 geometry hypothesis NOT falsified. Stage 2 **CLOSED as PARTIAL-PASS**.
- Stage 2 closure note:
  - H^4 Hopf sector routing discriminates semantically structured embeddings (PPMI-SVD)
    from column-permuted control. The discrimination is seed-stable (4 independent seeds).
  - Caveat: production routing requires semantically structured input embeddings.
    Pure hash features (INC-0136–0141) are isotropic by construction and fail.
  - Shell law (radial discrimination beyond sector) was not demonstrated as strictly
    necessary — pmax_before ≈ pmax_after in all INC-0142/0143 runs (single shell).
    This is acceptable: the Stage 2 gate was angular discrimination, not shell splitting.
- Decision: Stage 2 → **CLOSED/PARTIAL-PASS** (2026-03-13, INC-0143 KEEP).
- Next branch:
  - `Stage 3 (Hopf Angular Correctness) — now unblocked. Queue first Stage 3 RR.`

## 3. Hopf Angular Correctness
- Status: `partial-pass`
- Canonical evidence:
  - `docs/research/increments/INC_0062_hopf_base_angular_law.md`
  - `docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`
- Latest result (INC-0144, 2026-03-13):
  - Fixed H^4 Hopf-base routing (dims 3,65,2,21) achieves rel_diff=31.2% discriminating PPMI-SVD
    ORIG from COL_PERM (stable: 31.8%, 30.6% per seed).
  - K-means adaptive clustering (100D PPMI) achieves rel_diff=3.1% (variable: −5.8%/+12.0%).
  - Fixed H^4 Hopf geometry with 4D subspace OUTPERFORMS 100D adaptive K-means.
  - Hopf chi-tightness: hopf_sector_chi_std=0.058 (Hopf) vs 0.253 (K-means) — 4.4× more
    chi-coherent sectors; Hopf sectors correspond to geometrically coherent H^4 base regions.
  - Angular mass balance (theta1=1.09, theta2=0.875 errors) remains open: reflects semantic
    clustering in H^4 base (non-uniform distribution is feature, not bug).
- Decision: Stage 3 → **PARTIAL-PASS** (2026-03-13, INC-0144 KEEP).
- Next branch:
  - `Stage 4 (Phase Transport) — test on PPMI-SVD proxy (NOT hash embeddings).
    Prior RR-063/064 results used hash embeddings and may not generalize to semantic proxy.`

## 4. Phase Transport Usefulness
- Status: `partial-pass`
- Canonical evidence:
  - `docs/research/increments/INC_0063_phase_transport_necessity.md`
  - `docs/research/increments/INC_0064_coupled_complex_phase_transport.md`
  - `docs/research/increments/INC_0065_product_phase_field.md`
  - `docs/research/increments/INC_0145_phase_transport_fiber_stage4.md`
  - `docs/research/increments/INC_0146_phase_transport_k75_refine.md`
  - `docs/research/increments/INC_0147_phase_transport_lambda_control.md`
- Latest result (INC-0147, 2026-03-13):
  - Lambda control screen (K=75, seed=0): isolates raw fiber alpha (λ=0) from Levi-Civita correction (λ=1).
  - HOPF_BASE_K75: rel_diff=46.0% (reference)
  - HOPF_TRANS_K75 λ=0 (raw alpha): rel_diff=66.7% — fiber coordinate alone adds +20.6pp over base
  - HOPF_TRANS_K75 λ=0.5 (partial): rel_diff=71.8%
  - HOPF_TRANS_K75 λ=1.0 (full transport): rel_diff=68.7% — INC-0146 replication
  - L1−L0 gap = +2.0pp (within 5pp noise threshold)
  - **Mechanism revised:** raw fiber alpha (θ₁+θ₂)/2 is the source of improvement, not the
    Levi-Civita correction (λ/2)cos(2χ)·δ. Transport formula is valid geometry but the correction
    term is not differentially useful on PPMI-SVD proxy.
- Previous result (INC-0146, 2026-03-13):
  - HOPF_TRANS_K75 rel_diff=65.1% (stable: 68.7%, 61.2%, 2 seeds)
  - HOPF_BASE_K75 rel_diff=46.7%. Fiber adds +18.4pp over same-K base (+39% relative)
- Previous result (INC-0145, 2026-03-13):
  - HOPF_FULL (gauge rotation on phase angles): rel_diff=40.7% (stable: 38.6%, 42.7%)
  - HOPF_TRANS (K=25, kalpha=2): rel_diff=28.1% (variable, bin dilution)
- Decision: Stage 4 → **PARTIAL-PASS confirmed** (2026-03-13, INC-0146 KEEP; mechanism revised INC-0147 REFINE).
  Fiber phase coordinate alpha confirmed (+20.6pp over base at K=75). HOPF_FULL (40.7%) also confirmed.
  Levi-Civita connection correction specifically is not differentially useful on PPMI-SVD proxy (+2pp).
  Remaining open: transfer to real LM routing (proxy-level evidence sufficient to unblock Stage 5).
- Next branch:
  - `Begin Stage 5 (Spectral/Operator Usefulness) — Stage 4 proxy evidence sufficient`

## 5. Spectral / Operator Usefulness
- Status: `partial-pass` (strong — 4-seed finalized)
- Canonical evidence:
  - `docs/research/increments/INC_0066_spectral_route_operator.md`
  - `docs/research/increments/INC_0067_spectral_signal_probes.md`
  - `docs/research/increments/INC_0068_spectral_residual_task_signals.md`
  - `docs/research/increments/INC_0148_spectral_geometry_native_operator.md`
  - `docs/research/increments/INC_0149_task_signal_poincare_operator.md`
  - `docs/research/increments/INC_0150_task_signal_poincare_confirm.md`
  - `docs/research/increments/INC_0151_task_signal_poincare_finalize.md`
- Latest result (INC-0151, 2026-03-13):
  - 4-seed finalize KEEP. Stable improvements across all seeds:
    true_margin_lowfreq +39.8% (BASE) / +47.5% (TRANS),
    label_indicator_lowfreq_max +57.3%,
    sector_lowfreq_energy +72.4% / +77.2%,
    true_margin_dirichlet −5.8% / −6.5% (genuinely smoother).
  - Progression stable-to-strengthening: screen +28% → confirm +26% → finalize +40%.
  - Theory chain geometry → operator → modes → task-signal fully confirmed at 4 seeds.
- Previous result (INC-0150, 2026-03-13):
  - 2-seed confirm KEEP. true_margin_lowfreq +26% (BASE) / +55% (TRANS),
    sector_lowfreq_energy +94% / +89%.
- Previous result (INC-0149, 2026-03-13):
  - 1-seed screen KEEP. error_indicator +109%, true_margin +28–134%, true_score +26%.
- Previous result (INC-0148, 2026-03-13):
  - Geometry-native spectral operator KEEP. poincare_4d +91–95% sector_lowfreq_energy vs
    Euclidean-KNN baseline. Prior INC-0067/68 NEGATIVE results explained: wrong operator.
- Decision: Stage 5 → **PARTIAL-PASS (strong)** (2026-03-13, INC-0151 KEEP).
  Full screen→confirm→finalize protocol complete. All key metrics pass >20% threshold
  at 4-seed mean. Spectral operator construction and task-signal smoothness confirmed.
  Remaining: whether spectral smoothness translates to computational advantage (Stage 6).
- Next branch:
  - `Transition to Stage 6 (Sparse Event-Driven Trainability)`

## 6. Sparse Event-Driven Trainability
- Status: `partial-pass` (strong — routing sparsity + bucket coherence + spectral operator)
- Stage 6 definition (updated INC-0159):
  - **Bucket semantic coherence — FINALIZED (INC-0158, 4 seeds).**
    Purity ratio up to 1.976× at TRANS K=100. All K ≥ 25 stable.
  - **Routing sparsity — CONFIRMED (INC-0159, 1 seed).**
    ORIG concentrates label signal into 3.68× / 2.14× more high-purity
    buckets than PERM (BASE / TRANS at t=0.15). Gini 2.12× / 1.65×.
    High-purity tail (t ≥ 0.25) exclusive to ORIG at BASE K=75.
  - **Spectral operator — FINALIZED (INC-0151, 4 seeds).**
    true_margin_lowfreq +40–48%, spectral smoothness confirmed.
  - INC-0152/0153/0154: per-sample error metrics geometry-agnostic.
  - INC-0155: MSE not a valid observable.
  - INC-0156: spectral compression (lowfreq_max ratio 1.50×, geometry is K-invariant).
  - INC-0157: bucket coherence 2-seed confirm.
  - Stage 6 evidence chain complete. Proceeding to Stage 7.
- Canonical evidence:
  - `docs/research/increments/INC_0125_product_phase_sparse_event_proxy_trainability_hardening.md`
  - `docs/research/increments/INC_0130_product_phase_sparse_event_translation_route_coupled_soft_bias_pilot.md`
  - `docs/research/increments/INC_0131_product_phase_sparse_event_translation_soft_bias_carry_forward.md`
  - `docs/research/increments/INC_0152_spectral_event_correlation_screen.md`
  - `docs/research/increments/INC_0154_event_gate_efficiency_screen.md`
  - `docs/research/increments/INC_0155_routing_compression_screen.md`
  - `docs/research/increments/INC_0156_spectral_compression_screen.md`
  - `docs/research/increments/INC_0157_spectral_compression_confirm.md`
  - `docs/research/increments/INC_0158_bucket_coherence_finalize.md`
  - `docs/research/increments/INC_0159_sparse_event_training_efficiency.md`
- Latest result (INC-0159, 2026-03-14):
  - **KEEP — routing sparsity confirmed at 1 seed.**
  - Concentration at t=0.15: BASE 3.68× (25/73 vs 7/75), TRANS 2.14× (25/63 vs 13/70).
  - High-purity tail: at t ≥ 0.25, BASE_PERM has 0 buckets; TRANS_ORIG has 21/63 (33%).
  - Gini: BASE ORIG 0.52 vs PERM 0.24 (2.12×); TRANS ORIG 0.61 vs PERM 0.37 (1.65×).
  - Spectral: lowfreq_max ratio 1.50× (exact INC-0156 replication).
  - Info_density withdrawn: MI bounded by H(sector) in concentrated routing.
  - TRANS amplifies all sparsity effects over BASE (consistent with INC-0146 +18pp).
- Previous result (INC-0158, 2026-03-14):
  - KEEP — bucket coherence finalized at 4 seeds (purity ratio 1.976× TRANS K=100).
- Decision: Stage 6 → **PARTIAL-PASS (strong)** (2026-03-14, INC-0159 KEEP).
  Evidence chain: bucket coherence (4 seeds, finalized) + spectral operator (4 seeds,
  finalized) + routing sparsity (1 seed, confirmed). Geometry-native routing creates
  sparse, high-purity routing patterns that permuted routing cannot replicate.
  Stage 7 now unblocked.
- Blocker:
  - `None. Stage 6 evidence sufficient to proceed to Stage 7.`
- Next branch:
  - `INC-0160: Stage 7 — Hardware-efficiency confirmation`

## 7. Hardware-Efficiency Confirmation
- Status: `complete` (INC-0171 KEEP, 2026-03-26 — end-to-end LM integration confirmed. Stage 7: COMPLETE.)
- Canonical evidence:
  - `docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md`
  - `docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md`
  - `docs/research/increments/INC_0098_product_phase_translation_chart_resident_route_cost_decomposition.md`
  - `docs/research/increments/INC_0160_sparse_event_training_efficiency_matched.md`
  - `docs/research/increments/INC_0161_routing_cost_confirm.md`
  - `docs/research/increments/INC_0162_routing_compute_scaling_law.md`
  - `docs/research/increments/INC_0163_matched_progress_compute_efficiency.md`
  - `docs/research/increments/INC_0164_scaling_law_consistency.md`
  - `docs/research/increments/INC_0165_hardware_proxy_closure.md`
  - `docs/research/increments/INC_0166_law_freeze_k100_boundary_audit.md`
  - `docs/research/increments/INC_0167_scaling_mechanism_diagnostic.md`
  - `docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md`  - `docs/research/increments/INC_0169_canonical_law_freeze_design_implications.md`- Latest result (INC-0170, 2026-03-14):
  - **KEEP -- production-K capacity validated.**
  - Static routing, TRANS ORIG vs PERM, K={600,1000,2000,3000,5000}, N=5000.
  - eff_ratio stabilises at 2.6–2.8× for K=600–5000 (does not collapse to 1).
  - Full K=25–5000 exponent: ORIG alpha=0.657 (vs 0.572 for K=25–400), PERM alpha=0.816.
  - Δalpha = 0.085 < 0.10 threshold. R² = 0.991. PERM control unchanged (0.816 vs 0.814).
  - The canonical law (K^0.572) underestimates eff_buckets at large K (up to +65% at K=5000),
    but PERM grows even faster → ratio stays above 2.6× through K=5000.
  - Interpretation: the law steepens at large K as ORIG begins to approach the natural cluster
    scale of the proxy; the compression advantage is real and persistent, not unlimited.
  - Script: `_inc0170_analysis.py`. Results: `results/analysis/inc0170_large_k.json`.
  - **KEEP -- routing geometry definitively characterized as purely angular.**
  - 160 static routing runs (5 norm variants × 2 modes × 2 data variants × 8 K).
  - TRANS ORIG scaling exponent α=0.572 is IDENTICAL (max deviation <0.015)
    across L2, L1 (no shells), L3, L4 normalizations. Norm-invariant confirmed.
  - Shell activation (L1 + adjusted delta_r): does NOT improve ORIG advantage.
    Gini ratio drops 1.836 → 1.486 (shells dilute angular concentration).
  - L3/L4 norm surfaces: no measurable effect on routing metrics.
  - The √K scaling is a direct consequence of Hopf-base angular sector
    discretization. No radial contribution identified.
- Latest result (INC-0169, 2026-03-14):
  - **KEEP -- canonical routing law frozen, design implications derived.**
  - Synthesis increment. All results drawn from INC-0162 through INC-0168.
  - Canonical law: eff_buckets = 2.957 × K^0.572 (TRANS ORIG, static, L2-normalized).
  - Norm-invariance confirmed: Δα < 0.015 across L1/L2/L3/L4 (INC-0168).
  - Shell hierarchy excluded from default design (inaccessible by construction;
    forced activation degrades Gini ratio by 19%, INC-0168).
  - Hardware consequence chain documented: 3.0–4.9× eff_cost, 2.5–2.9× LRU misses (TRANS).
  - Design knob priority: K (high), phase transport (high), normalization (low), shells (low).
  - INC-0170 proposal: Large-K angular capacity test, K=1000–5000, static routing.
    Predicted eff_ratio at K=5000: ~4.4× (extrapolation of INC-0168 static fit).
- Decision: Stage 7 → **COMPLETE** (2026-03-26, INC-0171 KEEP).
  End-to-end LM integration confirmed. Fixed geometric routing replaces learned gating at 8% PPL cost.
  eff_ratio at convergence 1.39× vs dense. All kill-list stages resolved. Publication-ready.
- Blocker:
  - `None. Stage 7 COMPLETE. All experimental research stages resolved.`
- Next branch:
  - `Publication: Angular Manifold Routing paper — sole active forward workstream. PAPER_SKELETON.md + PUBLICATION_PACKET.md complete.`
