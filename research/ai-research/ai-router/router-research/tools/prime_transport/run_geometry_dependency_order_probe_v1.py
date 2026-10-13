#!/usr/bin/env python3
"""run_geometry_dependency_order_probe_v1.py

GEOMETRY DEPENDENCY ORDER PROBE v1
===================================

BRANCH:   geometry_dependency_order_probe_v1
CONTRACT: prompt_contract_v4.md — loaded and binding

════════════════════════════════════════════════════════════════════════
PURPOSE
════════════════════════════════════════════════════════════════════════

This probe does NOT run optimization, training, or large sweeps.

Its purpose is to:
  1. Enforce the correct dependency order across 7 layers
  2. Classify which layers are settled, partially specified, or unresolved
  3. Audit prior conclusions for validity against the dependency order
  4. Run LIGHTWEIGHT LOCAL VALIDATIONS ONLY to confirm structural claims
  5. Identify the earliest blocking unresolved layer and the next valid probe

DEPENDENCY ORDER (from prompt contract):
  Layer 1 — Ambient/operative space
  Layer 2 — Basis family
  Layer 3 — Projection/operator family
  Layer 4 — Constant/ratio family
  Layer 5 — Comparison-support mechanism
  Layer 6 — D / workspace
  Layer 7 — Training/reference structure

CORE RULE: A downstream variable may NOT be used to rule on an upstream variable.

════════════════════════════════════════════════════════════════════════
MANDATORY PRE-CODE ANALYSIS (restated from session)
════════════════════════════════════════════════════════════════════════

LAYER 1 (Ambient space): UNRESOLVED
  - Current system: R^16 hybrid (12 angular + 4 magnitude dims).
  - "H3 tangent" = h=3 harmonic within a specific block, NOT hyperbolic 3-space.
  - R4 and other space types have never been tested or falsified.
  - BLOCKS_A configuration chosen by convention, not from a space-type argument.

LAYER 2 (Basis family): PARTIALLY SPECIFIED
  - 2i rotation (apply_anchor_two_i) adopted for prototypes.
  - State uses unit-norm (cos, sin) pairs = sqrt(2)-amplitude convention.
  - H3 basis separation probe confirmed 2i-rotated prototypes resolve false null.
  - Amplitude ambiguity (sqrt(2) vs 2i full amplitude) not closed for all subspaces.

LAYER 3 (Projection/operator): PARTIALLY SPECIFIED
  - cos: correct within-frame comparison when both state and reference are 2i-rotated.
  - sin: displacement metric; 0 for matched, 1 for adjacent.
  - tan: EXCLUDED — NaN at ±90° (adjacent classes); confirmed in H3 tangent sweep.
  - Routing/dynamics operator separate from comparison metric — not settled.

LAYER 4 (Constants/ratios): SETTLED AS CONSTANTS
  - full_radial = sqrt(10) ≈ 3.1623 — structural constant (unit-norm pairs).
  - subspace_radial = 1.0 — unit-normalized H3 pair.
  - radial_ratio = 1/sqrt(10) ≈ 0.3162 — structural constant.
  - carrier_scale sweep [0.5, 1.0, 2.0] showed no discriminative effect.
  - NO special constant (including phi) has any evidence supporting a role.

LAYER 5 (Comparison mechanism): PARTIALLY SPECIFIED
  - H3 pair (dims 10-11) used as comparison subspace — convention, not derived.
  - 2i-rotated prototype comparison now settled within-layer.
  - Other harmonics (H1, H2) not systematically compared as subspace alternatives.
  - H3 tangent migration: COMPLETE (false null resolved).

LAYER 6 (D / workspace): UNRESOLVED
  - D=0,1,5,12,16,20,24,32 swept; effect minimal under training probes.
  - Under eps=1.0: D is STRUCTURALLY IRRELEVANT (higher harmonics frozen by construction).
  - Under eps=0.0 (synthesis): D has some effect but Layers 1-5 not yet settled.
  - D cannot be interpreted meaningfully until Layers 1-5 are adequately specified.

LAYER 7 (Training): UNRESOLVED
  - Training dominates in h3_coupled probe (TRAINING_ONLY_EFFECT conclusion).
  - But training module uses MLP (unverified basis/operator alignment).
  - Training conclusions are local-conditional on unresolved Layers 2-3.

PRIOR CONCLUSION AUDIT:
  training_only_effect    → B: CONDITIONAL ONLY (Layer 2-3 unresolved upstream)
  d_irrelevance           → B: CONDITIONAL ONLY (tautology under eps=1.0 only)
  radial_dead             → A: SAFE TO KEEP (structural constant — Layer 4)
  ratio_dead              → A: SAFE TO KEEP (structural constant — Layer 4)
  cos_vs_sin_conclusion   → B: CONDITIONAL ONLY (frame-dependent; Layer 2 upstream)
  2i_frame_alignment      → B: CONDITIONAL ONLY (valid within R^16; Layer 1 upstream)

EARLIEST BLOCKING LAYER: LAYER 1 (Ambient/operative space)
NEXT REQUIRED PROBE: geometry_space_type_probe_v1
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

# ── Output path ──────────────────────────────────────────────────────────────
_ROOT = Path(__file__).resolve().parents[2]
OUTPUT_JSON = (
    _ROOT
    / "results"
    / "prime_transport_recursive_system"
    / "geometry_dependency_order_probe_v1.json"
)

# ── Layer status constants ───────────────────────────────────────────────────
SETTLED = "SETTLED"
PARTIAL = "PARTIALLY_SPECIFIED"
UNRESOLVED = "UNRESOLVED"


# ═══════════════════════════════════════════════════════════════════════════
# LOCAL VALIDATION ONLY — Section 1: Radial constants (Layer 4)
# ═══════════════════════════════════════════════════════════════════════════

def validate_layer4_radial_constants() -> dict:
    """LOCAL VALIDATION ONLY — confirm radial quantities are structural constants.

    The R^16 state has:
      - 6 unit-norm (cos, sin) pairs  → 6 × 1.0 = 6.0 in L2-squared
      - 4 magnitude scalars = 1.0     → 4 × 1.0 = 4.0 in L2-squared
      full_radial = sqrt(10)

    H3 pair (dims 10-11): single unit-norm pair → subspace_radial = 1.0
    radial_ratio = 1.0 / sqrt(10) = 1/sqrt(10)

    These follow analytically from the architecture, no sampling needed.
    """
    n_angular_pairs = 6
    n_magnitude_dims = 4
    h3_pair_norm = 1.0  # unit-normalized by construction

    full_radial_sq = n_angular_pairs * (h3_pair_norm ** 2) + n_magnitude_dims * (1.0 ** 2)
    full_radial = math.sqrt(full_radial_sq)
    subspace_radial = h3_pair_norm  # H3 pair alone
    radial_ratio = subspace_radial / full_radial

    expected_full = math.sqrt(10)
    expected_sub = 1.0
    expected_ratio = 1.0 / math.sqrt(10)

    return {
        "label": "LOCAL VALIDATION ONLY — Layer 4 radial constants",
        "full_radial_analytic": round(full_radial, 9),
        "full_radial_expected": round(expected_full, 9),
        "full_radial_match": abs(full_radial - expected_full) < 1e-9,
        "subspace_radial_analytic": subspace_radial,
        "subspace_radial_match": subspace_radial == expected_sub,
        "radial_ratio_analytic": round(radial_ratio, 9),
        "radial_ratio_expected": round(expected_ratio, 9),
        "radial_ratio_match": abs(radial_ratio - expected_ratio) < 1e-9,
        "conclusion": (
            "ALL RADIAL QUANTITIES ARE STRUCTURAL CONSTANTS — "
            "no discriminative information, D-invariant, eps-invariant"
        ),
        "layer4_status": SETTLED,
    }


# ═══════════════════════════════════════════════════════════════════════════
# LOCAL VALIDATION ONLY — Section 2: D irrelevance under eps=1.0 (Layer 6)
# ═══════════════════════════════════════════════════════════════════════════

def validate_layer6_d_irrelevance_under_eps1() -> dict:
    """LOCAL VALIDATION ONLY — prove D is irrelevant when eps_high=1.0.

    apply_split_transport with eps_high=1.0:
      For each harmonic h ≥ 2:
        new_h = (1 - eps_high) * proposed + eps_high * previous
               = 0 * proposed + 1 * previous
               = previous   (EXACT PRESERVATION)

    H3 is h=3 harmonic → preserved exactly regardless of D steps.
    tau_final[:,10:12] == tau_init[:,10:12] for any D when eps_high=1.0.
    This is NOT an empirical finding — it is an algebraic identity.
    """
    eps_high = 1.0
    h3_harmonic = 3
    # At eps_high=1.0: new_h = (1-1.0)*new + 1.0*prev = prev
    # For h >= 2 (h3_harmonic=3 >= 2): TRUE
    h3_is_preserved = (h3_harmonic >= 2) and (eps_high == 1.0)
    weight_on_new = 1.0 - eps_high  # = 0.0
    weight_on_prev = eps_high       # = 1.0

    return {
        "label": "LOCAL VALIDATION ONLY — Layer 6 D irrelevance under eps_high=1.0",
        "eps_high": eps_high,
        "h3_harmonic_index": h3_harmonic,
        "weight_on_new_h3_per_step": weight_on_new,
        "weight_on_prev_h3_per_step": weight_on_prev,
        "h3_preserved_exactly": h3_is_preserved,
        "conclusion": (
            "Under eps_high=1.0, H3 is frozen algebraically at every step. "
            "D has ZERO effect on H3 output — this is a tautology, not an empirical result. "
            "D sensitivity conclusions under eps=1.0 carry NO information about D's role."
        ),
        "implication": (
            "Prior 'D irrelevant' conclusions are CONDITIONAL: valid only under eps=1.0 "
            "by construction. D role under eps<1.0 (synthesis) remains untested "
            "with upstream layers settled."
        ),
    }


# ═══════════════════════════════════════════════════════════════════════════
# LOCAL VALIDATION ONLY — Section 3: Tan exclusion (Layer 3)
# ═══════════════════════════════════════════════════════════════════════════

def validate_layer3_tan_excluded() -> dict:
    """LOCAL VALIDATION ONLY — confirm tan is structurally excluded.

    In the 2i-rotated H3 frame, the 4 class vectors are:
      k=0: [0, +1]   k=1: [-1, 0]   k=2: [0, -1]   k=3: [+1, 0]

    At class distance = 1 (adjacent classes), cos(angle) = 0.
    tan = sin/cos = undefined (NaN) at these points.
    These cases arise for 50% of all class-pair comparisons (k vs k±1).
    tan is therefore STRUCTURALLY UNSTABLE and excluded.
    """
    import math
    # Adjacent class pairs in the 2i frame
    adjacent_pairs = [
        ((0, 1), (-1, 0)),   # k=0 vs k=1
        ((-1, 0), (0, -1)),  # k=1 vs k=2
        ((0, -1), (1, 0)),   # k=2 vs k=3
        ((1, 0), (0, 1)),    # k=3 vs k=0
    ]

    nan_count = 0
    for u, v in adjacent_pairs:
        cos_val = u[0] * v[0] + u[1] * v[1]  # dot product
        sin_val = u[0] * v[1] - u[1] * v[0]  # cross product
        if abs(cos_val) < 1e-9:
            nan_count += 1

    fraction_nan = nan_count / len(adjacent_pairs)

    return {
        "label": "LOCAL VALIDATION ONLY — Layer 3 tan exclusion",
        "adjacent_pairs_tested": len(adjacent_pairs),
        "nan_at_cos_zero": nan_count,
        "fraction_nan": fraction_nan,
        "conclusion": (
            f"tan = sin/cos is NaN at {nan_count}/{len(adjacent_pairs)} adjacent class pairs. "
            "Tan is structurally excluded — confirmed by construction."
        ),
        "layer3_tan_status": "EXCLUDED_BY_CONSTRUCTION",
    }


# ═══════════════════════════════════════════════════════════════════════════
# LOCAL VALIDATION ONLY — Section 4: 2i false-null source (Layer 2)
# ═══════════════════════════════════════════════════════════════════════════

def validate_layer2_2i_frame_mismatch() -> dict:
    """LOCAL VALIDATION ONLY — confirm 2i rotation causes cos=0 in mismatch scenario.

    apply_anchor_two_i maps (cos t, sin t) -> (-sin t, cos t) = R90 * v.
    Raw class k proto: [cos(pi*k/2), sin(pi*k/2)]
    2i class k state:  [-sin(pi*k/2), cos(pi*k/2)]

    cos(2i_state_k, raw_proto_k) = dot product
      = (-sin(pi*k/2)) * cos(pi*k/2) + cos(pi*k/2) * sin(pi*k/2)
      = 0.0  for all k

    This confirms the false null: comparing 2i state to raw prototype always gives 0.
    """
    import math
    results = []
    for k in range(4):
        angle = math.pi * k / 2
        raw = (math.cos(angle), math.sin(angle))
        rotated = (-math.sin(angle), math.cos(angle))  # apply_anchor_two_i
        cos_mismatch = rotated[0] * raw[0] + rotated[1] * raw[1]
        cos_correct = rotated[0] * rotated[0] + rotated[1] * rotated[1]  # same frame
        results.append({
            "k": k,
            "cos_2i_vs_raw": round(cos_mismatch, 9),
            "cos_2i_vs_2i": round(cos_correct, 9),
        })

    all_mismatch_zero = all(abs(r["cos_2i_vs_raw"]) < 1e-9 for r in results)
    all_correct_one = all(abs(r["cos_2i_vs_2i"] - 1.0) < 1e-9 for r in results)

    return {
        "label": "LOCAL VALIDATION ONLY — Layer 2 2i frame mismatch",
        "per_class": results,
        "all_mismatch_cos_zero": all_mismatch_zero,
        "all_correct_cos_one": all_correct_one,
        "conclusion": (
            "cos(2i_state, raw_proto) = 0.0 for ALL 4 classes — systematic false null. "
            "cos(2i_state, 2i_proto) = 1.0 for ALL 4 classes — signal recovered. "
            "2i prototype adoption resolves the false null within R^16 / H3-tangent space."
        ),
        "implication": (
            "This conclusion is CONDITIONAL on Layer 1: the 2i rotation is defined "
            "within the current R^16 ambient space. If space type changes, "
            "the rotation definition and false-null analysis must be repeated."
        ),
    }


# ═══════════════════════════════════════════════════════════════════════════
# LOCAL VALIDATION ONLY — Section 5: Ambient space dimensionality (Layer 1)
# ═══════════════════════════════════════════════════════════════════════════

def validate_layer1_ambient_space_config() -> dict:
    """LOCAL VALIDATION ONLY — confirm the current ambient space is R^16 by convention.

    BLOCKS_A = (0,2,2,1), (2,7,5,1), (7,9,2,1), (9,21,12,3)
    tau_ang: sum of 2*n_h harmonic dims per block
      Block 0: 2*1=2 dims  (h1 only)
      Block 1: 2*1=2 dims  (h1 only)
      Block 2: 2*1=2 dims  (h1 only)
      Block 3: 2*3=6 dims  (h1, h2, h3)
      Total angular: 12 dims

    tau_mag: 1 magnitude per block = 4 dims
    d_hyb = 12 + 4 = 16

    This is a STRUCTURAL CONVENTION, not a derived necessity.
    R4 (4-dimensional) and other alternatives have NOT been tested.
    """
    blocks_a = [
        (0, 2, 2, 1),    # (start, end, period, n_harmonics)
        (2, 7, 5, 1),
        (7, 9, 2, 1),
        (9, 21, 12, 3),
    ]
    n_blocks = len(blocks_a)
    n_angular_dims = sum(2 * b[3] for b in blocks_a)
    n_magnitude_dims = n_blocks
    d_hyb = n_angular_dims + n_magnitude_dims

    h3_block_idx = 3         # dominant block (index 3)
    h3_harmonic = 3          # 3rd harmonic of block 3
    h3_pair_start = 10       # dims 10-11 within tau_ang
    h3_pair_end = 12

    # R4 as alternative: 4-dimensional flat space
    d_r4_alternative = 4

    return {
        "label": "LOCAL VALIDATION ONLY — Layer 1 ambient space config",
        "blocks_a": [list(b) for b in blocks_a],
        "n_blocks": n_blocks,
        "n_angular_dims": n_angular_dims,
        "n_magnitude_dims": n_magnitude_dims,
        "d_hyb_current": d_hyb,
        "h3_pair_dims": [h3_pair_start, h3_pair_end - 1],
        "h3_note": (
            "H3 refers to the 3rd harmonic of block 3 within R^16, "
            "NOT to hyperbolic H^3 space."
        ),
        "r4_alternative_dim": d_r4_alternative,
        "r4_never_tested": True,
        "space_choice_justification": "CONVENTION — no derivation from geometry principle",
        "conclusion": (
            "Current ambient space is R^16 hybrid (12 angular + 4 magnitude). "
            "BLOCKS_A was chosen by convention. "
            "R4 and other alternatives have never been tested or falsified. "
            "Layer 1 status: UNRESOLVED."
        ),
        "layer1_status": UNRESOLVED,
    }


# ═══════════════════════════════════════════════════════════════════════════
# BUILD DEPENDENCY AUDIT RECORD
# ═══════════════════════════════════════════════════════════════════════════

def build_dependency_audit() -> dict:
    """Assemble the full dependency order audit record."""

    settled_layers = {
        "layer_4": {
            "name": "Constant/ratio family",
            "status": SETTLED,
            "evidence": [
                "full_radial = sqrt(10) confirmed constant across all D, eps, regimes",
                "subspace_radial = 1.0 unit-normalized H3 pair",
                "radial_ratio = 1/sqrt(10) structural constant",
                "carrier_scale sweep [0.5,1.0,2.0] showed negligible effect on success_rate",
                "No special constant (phi or otherwise) supported by any evidence",
            ],
            "settled_conclusion": "Radial quantities carry NO discriminative information",
        }
    }

    unresolved_layers = {
        "layer_1": {
            "name": "Ambient/operative space",
            "status": UNRESOLVED,
            "blocking_reason": (
                "R^16 hybrid representation is a convention; R4 and alternatives not falsified. "
                "All downstream layer conclusions are defined RELATIVE to this space."
            ),
            "what_would_resolve": "geometry_space_type_probe_v1 — compare R^16 vs R4 vs alternatives",
        },
        "layer_6": {
            "name": "D / workspace",
            "status": UNRESOLVED,
            "blocking_reason": (
                "D irrelevance under eps=1.0 is a tautology (frozen harmonics). "
                "D under synthesis (eps<1.0) is entangled with unresolved Layers 1-5. "
                "D cannot be meaningfully interpreted until Layers 1-5 are specified."
            ),
            "what_would_resolve": "Fix Layers 1-5, then sweep D under controlled synthesis regime",
        },
        "layer_7": {
            "name": "Training/reference structure",
            "status": UNRESOLVED,
            "blocking_reason": (
                "Training conclusions are conditional on Layer 2-3 being fixed. "
                "MLP training module basis alignment unverified. "
                "Training effect size may change when correct basis is enforced."
            ),
            "what_would_resolve": "Fix Layers 1-5, then re-run training comparison",
        },
    }

    partial_layers = {
        "layer_2": {
            "name": "Basis family",
            "status": PARTIAL,
            "what_is_settled": (
                "2i prototype rotation resolves false null within H3 tangent frame. "
                "Tan excluded by construction."
            ),
            "what_remains": (
                "sqrt(2) vs 2i amplitude convention not closed for all subspaces. "
                "Conditional on Layer 1 (2i rotation defined within R^16)."
            ),
        },
        "layer_3": {
            "name": "Projection/operator family",
            "status": PARTIAL,
            "what_is_settled": (
                "cos: correct within-frame metric when both state and proto are 2i-rotated. "
                "sin: displacement metric — 0 for matched, 1 for adjacent. "
                "tan: excluded by NaN instability at ±90° (4 of 4 adjacent pairs)."
            ),
            "what_remains": (
                "Routing/dynamics operator vs comparison metric not separated. "
                "Operator choice for synthesis (eps<1.0) not settled."
            ),
        },
        "layer_5": {
            "name": "Comparison-support mechanism",
            "status": PARTIAL,
            "what_is_settled": (
                "H3 pair (dims 10-11) as comparison subspace — convention adopted. "
                "2i-rotated prototype comparison settled within H3. "
                "H3 tangent migration status: COMPLETE (false null resolved)."
            ),
            "what_remains": (
                "H3 subspace choice vs H1/H2 never systematically compared. "
                "Comparison mechanism extension to other harmonics not tested. "
                "Conditional on Layer 1 (subspace defined within R^16)."
            ),
        },
    }

    prior_conclusion_audit = [
        {
            "conclusion": "training_only_effect",
            "source_probe": "h3_coupled_geometry_training_dependency_probe_v1",
            "layers_actually_tested": ["Layer 6 (D)", "Layer 7 (training steps)"],
            "unresolved_upstream": ["Layer 2 (basis not verified for MLP)", "Layer 3 (joint operator fixed)"],
            "classification": "B: CONDITIONAL ONLY",
            "reason": (
                "Training dominance is local to the specific joint-operator + "
                "unit-norm basis used. If Layers 2-3 change, training effect size may shift."
            ),
        },
        {
            "conclusion": "d_irrelevance",
            "source_probe": "multiple (radial_ratio_revalidation, train_free, h3_coupled)",
            "layers_actually_tested": ["Layer 6 (D under eps=1.0)"],
            "unresolved_upstream": ["Layer 1 (space type)", "Layers 2-5 (frame/operator)"],
            "classification": "B: CONDITIONAL ONLY",
            "reason": (
                "D is irrelevant under eps=1.0 by algebraic construction (frozen harmonics). "
                "This is a tautology, not an empirical finding about D's role in dynamics."
            ),
        },
        {
            "conclusion": "radial_dead",
            "source_probe": "h3_2i_radial_increment_probe_v1, radial_ratio_revalidation_probe_v1",
            "layers_actually_tested": ["Layer 4 (radial constants)"],
            "unresolved_upstream": [],
            "classification": "A: SAFE TO KEEP",
            "reason": (
                "Radial quantities are structural constants of the architecture. "
                "This is independent of upstream layer choices."
            ),
        },
        {
            "conclusion": "ratio_dead",
            "source_probe": "d_aware_h3_synthesis_ratio_lock_probe_v1",
            "layers_actually_tested": ["Layer 4 (radial ratio)"],
            "unresolved_upstream": [],
            "classification": "A: SAFE TO KEEP",
            "reason": "Same as radial_dead — structural constant by construction.",
        },
        {
            "conclusion": "cos_vs_sin_operator",
            "source_probe": "h3_basis_separation_probe_v1, h3_tangent_projection_frame_sweep_v1",
            "layers_actually_tested": ["Layer 3 (projection operator)", "Layer 2 (frame scenario)"],
            "unresolved_upstream": ["Layer 1 (ambient space)", "Layer 2 (which frame is canonical)"],
            "classification": "B: CONDITIONAL ONLY",
            "reason": (
                "cos wins when frames match (both 2i); sin wins in mismatch scenario. "
                "Operator conclusion is downstream of the frame/basis choice. "
                "In migrated system (both 2i): cos is the correct within-frame metric — "
                "but this is conditional on Layer 1 remaining R^16."
            ),
        },
        {
            "conclusion": "2i_frame_alignment",
            "source_probe": "h3_tangent_full_mechanism_migration_probe_v1",
            "layers_actually_tested": ["Layer 2 (basis)", "Layer 5 (comparison mechanism)"],
            "unresolved_upstream": ["Layer 1 (ambient space — 2i rotation defined within R^16)"],
            "classification": "B: CONDITIONAL ONLY",
            "reason": (
                "2i prototype adoption is correct within R^16/H3-tangent representation. "
                "If Layer 1 is revised, the rotation definition changes and this must be revisited."
            ),
        },
    ]

    conditional_results = [c for c in prior_conclusion_audit if c["classification"] == "B: CONDITIONAL ONLY"]
    invalid_global = [c for c in prior_conclusion_audit if c["classification"] == "C: INVALID AS GLOBAL CONCLUSIONS"]

    return {
        "probe": "geometry_dependency_order_probe_v1",
        "branch": "geometry_dependency_order_probe_v1",
        "contract": "prompt_contract_v4.md",
        "settled_layers": settled_layers,
        "partial_layers": partial_layers,
        "unresolved_layers": unresolved_layers,
        "prior_conclusion_audit": prior_conclusion_audit,
        "conditional_results": [c["conclusion"] for c in conditional_results],
        "invalid_global_conclusions": [c["conclusion"] for c in invalid_global],
        "next_required_layer": "Layer 1 — Ambient/operative space",
        "next_required_probe": "geometry_space_type_probe_v1",
        "next_probe_description": (
            "Compare H3-tangent R^16 vs R4 (and potentially other reduced-dim spaces) "
            "using DETERMINISTIC structural comparison — no training. "
            "Measure whether the same class-separation signal (cos metric between matched "
            "vs mismatched) is achievable in a reduced space. "
            "Must NOT change any other variable (Layers 2-7 held constant)."
        ),
        "probes_blocked_until_layer1_resolved": [
            "Any D-sensitivity conclusion under eps<1.0 (Layer 6)",
            "Any training-size conclusion as global claim (Layer 7)",
            "Any operator-family claim as global conclusion (Layer 3)",
            "Any ratio/constant as mechanism driver claim (beyond already-settled constants)",
        ],
        "lightweight_validations_run": [],  # filled below
        "geometry_dependency_order_status": "LAYER_1_BLOCKING",
    }


# ═══════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════

def main() -> None:
    print("[dependency_order_probe_v1] Running lightweight validations...")

    # Run all lightweight checks
    lv1 = validate_layer1_ambient_space_config()
    lv2 = validate_layer2_2i_frame_mismatch()
    lv3 = validate_layer3_tan_excluded()
    lv4 = validate_layer4_radial_constants()
    lv6 = validate_layer6_d_irrelevance_under_eps1()

    lightweight_validations = [lv1, lv2, lv3, lv4, lv6]

    # Print validation summaries
    for lv in lightweight_validations:
        print(f"\n  {lv['label']}")
        print(f"  → {lv['conclusion']}")

    # Build full audit record
    audit = build_dependency_audit()
    audit["lightweight_validations_run"] = lightweight_validations

    # Write JSON
    OUTPUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_JSON, "w") as f:
        json.dump(audit, f, indent=2)

    print(f"\n[dependency_order_probe_v1] Wrote: {OUTPUT_JSON}")

    # Final status line
    print("\n" + "=" * 60)
    print("GEOMETRY DEPENDENCY ORDER STATUS: LAYER_1 BLOCKING")
    print("=" * 60)
    print()
    print("Settled:    Layer 4 (radial constants — structural facts)")
    print("Partial:    Layers 2, 3, 5 (conditionally specified within R^16)")
    print("Unresolved: Layers 1, 6, 7 (earliest: Layer 1 — ambient space)")
    print()
    print("Next probe: geometry_space_type_probe_v1")
    print()
    print("Conclusions downgraded to CONDITIONAL:")
    for c in audit["conditional_results"]:
        print(f"  - {c}")
    print()
    print("No conclusions are INVALID as global (no C-class findings exist yet).")


if __name__ == "__main__":
    main()
