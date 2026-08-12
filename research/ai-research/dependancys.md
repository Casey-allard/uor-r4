1 - CANONICAL DEPENDANCY LOCKS

┌───────┬─────────────────────┬────────────────────────────────────────────────────────────────────────────────────┬─────────────────────────────────────────────────────────────────────────────────┐
│ Layer │ Description         │ Repo Component(s)                                                                  │ Status                                                                          │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 1     │ Ambient/operative   │ BLOCKS_A config in TN_ang; hybrid R^16 (12 angular+4 mag); "H3 tangent" = h=3      │ UNRESOLVED — R4 / alternative space never falsified                             │
│       │ space               │ harmonic within block, NOT hyperbolic space                                        │                                                                                 │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 2     │ Basis family        │ apply_anchor_two_i (2i rotation); unit-norm (cos,sin) pairs in TN_ang; split basis │ PARTIALLY SPECIFIED — 2i prototypes adopted; sqrt(2)/2i amplitude ambiguity not │
│       │                     │ explored in H3 basis separation probe                                              │ fully closed across all subspaces                                               │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 3     │ Projection/operator │ cos_metric, sin_metric, tan (excluded); joint in training probes; H3 tangent sweep │ PARTIALLY SPECIFIED — cos correct within-frame; operator for routing/dynamics   │
│       │                     │ confirmed tan excluded                                                             │ not settled independent of frame                                                │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 4     │ Constant/ratio      │ full_radial=sqrt(10), subspace_radial=1.0, radial_ratio=1/sqrt(10); carrier_scale  │ NOT SETTLED (as constants) — all radial quantities are SET structural constants │
│       │ family              │ swept [0.5,1.0,2.0]                                                                │ check for future discriminative information such as an emergent correct ratio   │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 5     │ Comparison-support  │ H3 pair (dims 10-11) as subspace; 2i-rotated prototypes; polar L2; H3 tangent      │ PARTIALLY SPECIFIED — subspace choice (H3 vs H1/H2) never swept; comparison     │
│       │ mechanism           │ migration complete                                                                 │ within H3 settled locally                                                       │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 6     │ D / workspace       │ D swept [0,1,5,12,16,20,24,32]; eps=1.0 makes D structurally irrelevant by frozen  │ UNRESOLVED — D only meaningful under eps<1.0 and Layers 1-5 are not yet fully   │
│       │                     │ harmonics                                                                          │ specified                                                                       │
├───────┼─────────────────────┼────────────────────────────────────────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
│ 7     │ Training/reference  │ MINIMAL(0), STRUCTURED(500), EXPANDED(1500) steps; 758-param MLP; 4-class balanced │ UNRESOLVED — training conclusions conditional on unresolved basis/operator      │
│       │ structure           │ pool                                                                               │ upstream                                                                        │
└───────┴─────────────────────┴────────────────────────────────────────────────────────────────────────────────────┴─────────────────────────────────────────────────────────────────────────────────┘

Prior probes that tested downstream-only behavior while upstream layers remained unresolved:

 - h3_coupled_geometry_training_dependency_probe_v1 — tested Layers 6+7 while Layers 1-3 unresolved
 - d_aware_h3_synthesis_ratio_lock_probe_v1 — tested Layer 6 dynamics while Layer 1-2 unresolved
 - radial_ratio_revalidation_probe_v1 — tested Layer 4 radial ratios only (valid), did not constrain Layer 1-3
 - prime_transport_router_optimizer_metric_probe_v1 — tested Layer 6+7 (D/training) under unresolved Layer 2-3

-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

2 — CONCLUSION AUDIT

┌──────────────────────┬────────────────────────┬────────────────────────────────────────────────────────────────────────────┬───────────────────────────────────────────────────────────────────────┐
│ Conclusion           │ Layer(s) Tested        │ Blocking Upstream Layer                                                    │ Classification                                                        │
├──────────────────────┼────────────────────────┼────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────┤
│ training-only effect │ Layer 7 vs Layer 6     │ Layer 2 (basis not verified for MLP training context); Layer 3 (joint      │ B: CONDITIONAL ONLY                                                   │
│                      │                        │ fixed, not surveyed)                                                       │                                                                       │
├──────────────────────┼────────────────────────┼────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────┤
│ D irrelevance        │ Layer 6 under eps=1.0  │ Layer 1 (space type); also a TAUTOLOGY: D is frozen by construction when   │ B: CONDITIONAL ONLY — valid under eps=1.0 as tautology, not           │
│                      │                        │ eps=1.0                                                                    │ informative about D in synthesis                                      │
├──────────────────────┼────────────────────────┼────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────┤
│ radial dead          │ Layer 4 (constants     │ NONE — radial being constant is a structural fact                          │ A: SAFE TO KEEP                                                       │
│                      │ confirmed)             │        (only under these conditions)                                       │   (only under these conditions)                                       │
├──────────────────────┼────────────────────────┼────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────┤
│ ratio dead           │ Layer 4 (radial ratio  │ NONE — same structural fact                                                │ A: SAFE TO KEEP                                                       │
│                      │ confirmed)             │        (only under these conditions)                                       │    (only under these conditions)                                      │
├──────────────────────┼────────────────────────┼────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────┤
│ cos vs sin operator  │ Layer 3 under mismatch │ Layer 2 (which frame is correct?) — cos wins when frames match; sin wins   │ B: CONDITIONAL ONLY — conclusion is frame-dependent; in migrated      │
│ conclusion           │ scenario               │ under mismatch                                                             │ system (both 2i) cos is correct                                       │
├──────────────────────┼────────────────────────┼────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────┤
│ 2i frame alignment   │ Layer 2+5 (prototype   │ Layer 1 (ambient space) — 2i rotation is within R^16; if space type        │ B: CONDITIONAL ONLY — valid within current R^16/H3-tangent            │
│ conclusion           │ frame)                 │ changes, rotation definition changes                                       │ representation                                                        │
└──────────────────────┴────────────────────────┴────────────────────────────────────────────────────────────────────────────┴───────────────────────────────────────────────────────────────────────┘

-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

 1. Earliest unresolved layer blocking progress: LAYER 1 — Ambient/operative space
  - The system operates in a FIXED R^16 hybrid representation chosen by convention. Whether this is the correct operative space vs R4 (or another dimensionality) has never been tested. All 
subsequent layers (basis choice, operator choice, comparison mechanism) are defined RELATIVE to this space.
 2. Next probe that should target ONLY Layer 1:
  - geometry_space_type_probe_v1 — compare H3-tangent (current R^16) vs R4 (4-dim flat) vs stripped-block alternatives; use deterministic structural comparison (no training); verify whether the 
geometric signal (cos separation of matched vs mismatched classes) is space-specific or space-invariant.
 3. Probes that remain invalid until Layer 1 is constrained:
  - Any D-sensitivity conclusion under eps<1.0 (Layer 6)
  - Any training-size conclusion (Layer 7)
  - Any operator-family conclusion (Layer 3) as a global claim
  - Any ratio/constant claim as a mechanism driver (Layer 4 constants are settled; any non-trivial ratio claim is blocked)


     1. tools/prime_transport/run_geometry_dependency_order_probe_v1.py — Runs 5 lightweight local validations (Layer 1 config check, Layer 2 2i false-null confirmation, Layer 3 tan exclusion, Layer 4 
    radial constants, Layer 6 D-tautology under eps=1.0). Produces JSON and prints status.
     2. results/prime_transport_recursive_system/geometry_dependency_order_probe_v1.json — Contains all required keys: settled_layers, unresolved_layers, partial_layers, prior_conclusion_audit, 
    conditional_results, invalid_global_conclusions, next_required_layer, next_required_probe, lightweight_validations_run.
     3. docs/research/prime_transport_geometry_dependency_order_probe_v1.md — Full dependency-order table, prior conclusion audit table, lightweight validation section, conditional-conclusions section, 
    earliest-blocking-layer section, and final one-line conclusion.

    -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    Key findings:

    NOT YET Settled (globally): Layer 4 only — radial quantities (full_radial = sqrt(10), subspace_radial = 1.0, radial_ratio = 1/sqrt(10)) are structural constants carrying zero discriminative information while the rest of the variables are in their current configuration. IT IS POSSIBLE UNDER IDEAL OTHER CONDITIONS THIS WILL ALIGN INTO A PERFECTLY RECURSIVE RATIO.
    Safe to keep (and allowed to probe as another conclusive downstream of other variables) in all future work.

    Partially specified (conditional on Layer 1): Layers 2 (basis/2i frame), 3 (cos/sin operator), 5 (H3 comparison mechanism).

    Unresolved — earliest blocking: Layer 1 (ambient/operative space) — R^16 hybrid was chosen by convention; R4 and other spaces have never been tested.

    Downgraded to CONDITIONAL ONLY:

     - training_only_effect — conditional on Layers 2-3 being fixed
     - d_irrelevance — tautology under eps=1.0, uninformative for synthesis
     - cos_vs_sin_operator — frame-dependent, conditional on Layers 1-2
     - 2i_frame_alignment — valid within R^16, conditional on Layer 1

    Next required probe: geometry_space_type_probe_v1 — compare R^16 vs R4 using deterministic structural comparison, no training, one variable changed.

     GEOMETRY DEPENDENCY ORDER STATUS: LAYER_1 BLOCKING

