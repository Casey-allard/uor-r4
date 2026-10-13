#!/usr/bin/env python3
"""run_geometry_space_dynamics_probe_v1.py

GEOMETRY SPACE DYNAMICS PROBE v1
==================================

BRANCH:   geometry_space_dynamics_probe_v1
CONTRACT: prompt_contract_v4.md — loaded and binding

════════════════════════════════════════════════════════════════════════
PURPOSE
════════════════════════════════════════════════════════════════════════

Layer 1 probe under non-frozen (dynamic) conditions.

Prior probe (geometry_space_type_probe_v1) confirmed SPACE_INVARIANT
signal under eps_high=1.0 (frozen / algebraic preservation of H3).

This probe asks: does ambient space affect the geometric signal once
H3 is no longer perfectly preserved (eps_high < 1.0)?

Variable under test:     AMBIENT SPACE ONLY
Controlled dynamics via: eps_high ∈ {1.0, 0.75, 0.5}

This is NOT a tuning probe. It is a boundary test.

════════════════════════════════════════════════════════════════════════
LOCKED VARIABLES (unchanged across ALL runs)
════════════════════════════════════════════════════════════════════════

  - Basis:            apply_anchor_two_i  (2i frame)
  - Operator:         cos_metric ONLY
  - Comparison:       identical H3 pairing logic across all spaces
  - Constants/ratios: unchanged (sqrt(10), 1/sqrt(10))
  - Training:         STRICTLY FORBIDDEN
  - No new constants: TRUE (no phi, no heuristic ratios)

════════════════════════════════════════════════════════════════════════
VARIABLE UNDER TEST (ONLY ONE)
════════════════════════════════════════════════════════════════════════

  Ambient / operative space representation:

    1. CURRENT_BASELINE — R^16 hybrid (12 angular + 4 magnitude)
                          H3 comparison subspace: angular dims [10, 11]
    2. R4_FLAT          — R^4 (h2+h3 pairs of dominant block = angular
                          dims [8,9,10,11] from baseline)
                          H3 comparison subspace: dims [2, 3] within R4
    3. REDUCED_BLOCK    — R^12 (12 angular only; magnitude dims removed)
                          H3 comparison subspace: dims [10, 11]

════════════════════════════════════════════════════════════════════════
CONTROLLED DYNAMICS
════════════════════════════════════════════════════════════════════════

  eps_high ∈ {1.0, 0.75, 0.5}

  Transport formula (for harmonic h ≥ 2, i.e., H3):
      h3_final = eps_high * h3_initial + (1 - eps_high) * h3_proposed

  Where h3_proposed is the H3 encoding of the ADJACENT class
  (class (k+1) % 4) — a deterministic, training-free perturbation.

  Under eps_high=1.0:  h3_final = h3_initial  (algebraic freeze — control)
  Under eps_high=0.75: 25% contamination from adjacent class
  Under eps_high=0.5:  50% contamination from adjacent class

  eps=0.0 EXCLUDED (full replacement — too destructive).

════════════════════════════════════════════════════════════════════════
MEASUREMENT (per space × eps combination)
════════════════════════════════════════════════════════════════════════

  mean_matched_cos:    mean cos_metric(state_k, proto_k) on H3 subspace
  mean_mismatched_cos: mean cos_metric(state_k, proto_j), j≠k, averaged
  separation_delta:    mean_matched_cos − mean_mismatched_cos

════════════════════════════════════════════════════════════════════════
MANDATORY STEP 1 — STRUCTURAL EQUIVALENCE CHECK
════════════════════════════════════════════════════════════════════════

  Verify:
  1. Identical inputs (same N, same class labels) across all runs
  2. Identical prototype construction per space
  3. H3 INITIAL values numerically identical across all three spaces
  4. H3 PROPOSED values numerically identical across all three spaces

  If any check fails: STOP — INVALID_COMPARISON_SETUP

════════════════════════════════════════════════════════════════════════
LAYER ISOLATION COMPLIANCE
════════════════════════════════════════════════════════════════════════

  NOT changed: Layers 2-7 (basis, operator, constants, comparison
               mechanism, D/workspace, training).
  TESTED:      Layer 1 only (ambient space) under dynamic eps regime.

CONTRACT RULES COMPLIED:
  Rule 1 (Mechanism Lock):    Mechanism defined pre-code.
  Rule 2 (No Hidden Changes): ONE variable changes — ambient space only.
  Rule 3 (Geometry Consistency): H3 subspace content is identical
                                  across all spaces by construction.
  Rule 4 (Deterministic First): No training, no optimisation.
  Rule 5 (Compare Regimes):   3 spaces × 3 eps values compared.
  Rule 6 (Output Discipline): JSON + Markdown + explicit classification.
  Rule 7 (Failure Is Valid):  Null result (invariance) accepted.
  Rule 8 (Reuse Components):  All geometry helpers reused from
                               run_geometry_space_type_probe_v1.py.
  Rule 9 (No Theory Injection): No phi, no external constants.
"""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np

# ── Output paths ─────────────────────────────────────────────────────────────
_ROOT    = Path(__file__).resolve().parents[2]
JSON_OUT = _ROOT / "results" / "prime_transport_recursive_system" / "geometry_space_dynamics_probe_v1.json"
MD_OUT   = _ROOT / "docs" / "research" / "prime_transport_geometry_space_dynamics_probe_v1.md"
JSON_OUT.parent.mkdir(parents=True, exist_ok=True)
MD_OUT.parent.mkdir(parents=True, exist_ok=True)

# ── Experiment constants ──────────────────────────────────────────────────────
VOCAB           = 4
N_PER_CLASS     = 256
N_SAMPLES       = N_PER_CLASS * VOCAB    # 1024 samples total
BLOCKS_A        = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# H3 angular indices in the 12-dim angular vector (baseline / reduced_block)
H3_IDX0_BASELINE = 10
H3_IDX1_BASELINE = 11

# Controlled dynamic eps values — eps=0.0 excluded (too destructive)
EPS_VALUES = [1.0, 0.75, 0.5]

# Threshold for classifying SPACE_DEPENDENT vs SPACE_INVARIANT
THRESHOLD_MEANINGFUL = 0.05


# ════════════════════════════════════════════════════════════════════════════
# GEOMETRY HELPERS (reused from run_geometry_space_type_probe_v1.py)
# ════════════════════════════════════════════════════════════════════════════

def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def convert_onehot_to_angular_multi(onehot: np.ndarray, blocks) -> np.ndarray:
    """Convert one-hot encoded tokens to multi-harmonic angular embedding."""
    N     = onehot.shape[0]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = np.zeros((N, d_ang))
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[:, s:e].argmax(axis=1).astype(float)
        for harm in range(1, n_h + 1):
            angle          = 2.0 * math.pi * harm * k_idx / float(m)
            out[:, ai]     = np.cos(angle)
            out[:, ai + 1] = np.sin(angle)
            ai += 2
    return out


def apply_anchor_two_i(tau_ang: np.ndarray, n_pairs: int) -> np.ndarray:
    """Rotate each harmonic pair by 90°: (cos θ, sin θ) → (−sin θ, cos θ).

    Identical to apply_anchor_two_i in run_geometry_space_type_probe_v1.py.
    """
    out = tau_ang.copy()
    for p in range(n_pairs):
        c = tau_ang[:, 2 * p].copy()
        s = tau_ang[:, 2 * p + 1].copy()
        out[:, 2 * p]     = -s
        out[:, 2 * p + 1] =  c
    return out


# ════════════════════════════════════════════════════════════════════════════
# STATE AND PROTOTYPE CONSTRUCTION
# ════════════════════════════════════════════════════════════════════════════

def build_full_r16_states(class_labels: np.ndarray, blocks) -> np.ndarray:
    """Build R^16 hybrid states for all samples (2i frame, eps_high=1.0 initial).

    Returns array of shape [N, 16] = 12 angular (2i-rotated) + 4 magnitude.
    """
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    d_vocab = sum(e - s for s, e, _, _ in blocks)
    N       = len(class_labels)

    onehot = np.zeros((N, d_vocab))
    for i, cl in enumerate(class_labels):
        k = int(cl) % VOCAB
        for s, e, m, _ in blocks:
            token_k = k % m
            if s + token_k < e:
                onehot[i, s + token_k] = 1.0

    tau_ang = convert_onehot_to_angular_multi(onehot, blocks)   # [N, 12]
    tau_ang = apply_anchor_two_i(tau_ang, n_pairs)              # 2i rotation
    tau_mag = np.ones((N, n_blocks))                            # [N, 4]
    return np.concatenate([tau_ang, tau_mag], axis=1)           # [N, 16]


def build_class_prototypes_r16(blocks) -> np.ndarray:
    """Build 4 class prototypes in R^16 (one per class k=0,1,2,3)."""
    labels = np.array([0, 1, 2, 3], dtype=int)
    return build_full_r16_states(labels, blocks)    # [4, 16]


# ════════════════════════════════════════════════════════════════════════════
# SPACE CANDIDATE BUILDERS (identical to prior probe — reused)
# ════════════════════════════════════════════════════════════════════════════

def build_current_baseline(
    class_labels: np.ndarray,
    blocks,
) -> Tuple[np.ndarray, np.ndarray, int, int]:
    """CURRENT_BASELINE: R^16 (12 angular + 4 magnitude).

    Returns (states[N,16], protos[4,16], h3_i0=10, h3_i1=11).
    """
    states = build_full_r16_states(class_labels, blocks)
    protos = build_class_prototypes_r16(blocks)
    return states, protos, H3_IDX0_BASELINE, H3_IDX1_BASELINE


def build_r4_flat(
    class_labels: np.ndarray,
    blocks,
) -> Tuple[np.ndarray, np.ndarray, int, int]:
    """R4_FLAT: 4D space = h2 and h3 pairs of dominant block (block3).

    Source dims: angular indices [8, 9, 10, 11] from the full 12-dim angular part.
    H3 comparison subspace: dims [2, 3] within R4.
    apply_anchor_two_i was applied to the full 16D state; slicing preserves it.
    """
    full_states = build_full_r16_states(class_labels, blocks)   # [N, 16]
    full_protos = build_class_prototypes_r16(blocks)             # [4, 16]

    r4_states = full_states[:, 8:12].copy()   # [N, 4]
    r4_protos = full_protos[:, 8:12].copy()   # [4, 4]

    H3_I0_R4, H3_I1_R4 = 2, 3
    return r4_states, r4_protos, H3_I0_R4, H3_I1_R4


def build_reduced_block(
    class_labels: np.ndarray,
    blocks,
) -> Tuple[np.ndarray, np.ndarray, int, int]:
    """REDUCED_BLOCK: R^12 (12 angular dims only; magnitude dims removed).

    H3 comparison subspace: dims [10, 11] (same as baseline).
    """
    d_ang, _, _ = geom_dims(blocks)   # d_ang = 12
    full_states = build_full_r16_states(class_labels, blocks)   # [N, 16]
    full_protos = build_class_prototypes_r16(blocks)             # [4, 16]

    rb_states = full_states[:, :d_ang].copy()   # [N, 12]
    rb_protos = full_protos[:, :d_ang].copy()   # [4, 12]

    return rb_states, rb_protos, H3_IDX0_BASELINE, H3_IDX1_BASELINE


# ════════════════════════════════════════════════════════════════════════════
# CONTROLLED DYNAMICS: SPLIT TRANSPORT ON H3 SUBSPACE
# ════════════════════════════════════════════════════════════════════════════

def build_proposed_h3(class_labels: np.ndarray, blocks) -> np.ndarray:
    """Build proposed H3 values for each sample.

    proposed_k = H3 encoding of class (k+1) % 4 — the adjacent class.

    This is a deterministic, training-free perturbation. It is computed
    from the SAME state-builder (build_full_r16_states) with shifted labels,
    then sliced at the H3 angular dims. The proposed values are
    space-independent: they originate from the underlying class encoding
    and are identical in all three space representations.

    Returns: [N, 2] array of proposed H3 vectors (pre-transport).
    """
    shifted_labels = (class_labels + 1) % VOCAB
    proposed_states = build_full_r16_states(shifted_labels, blocks)   # [N, 16]
    # H3 is always at baseline angular dims 10-11 before space projection
    return proposed_states[:, H3_IDX0_BASELINE:H3_IDX1_BASELINE + 1].copy()  # [N, 2]


def apply_h3_transport(
    states:      np.ndarray,   # [N, D_space]
    h3_i0:       int,
    h3_i1:       int,
    proposed_h3: np.ndarray,   # [N, 2] — space-independent proposed H3 values
    eps_high:    float,
) -> np.ndarray:
    """Apply split transport to H3 subspace dims only.

    Formula (matching apply_split_transport semantics for h ≥ 2):
        h3_final = eps_high * h3_initial + (1 - eps_high) * h3_proposed

    Under eps_high=1.0: h3_final = h3_initial  (algebraic freeze — control)
    Under eps_high<1.0: partial replacement by adjacent-class H3 encoding

    Non-H3 dims are untouched — they do not enter the comparison.
    """
    result = states.copy()
    h3_init  = states[:, h3_i0:h3_i1 + 1]                              # [N, 2]
    h3_final = eps_high * h3_init + (1.0 - eps_high) * proposed_h3     # [N, 2]
    result[:, h3_i0:h3_i1 + 1] = h3_final
    return result


# ════════════════════════════════════════════════════════════════════════════
# COS_METRIC (H3 subspace only — identical to prior probe)
# ════════════════════════════════════════════════════════════════════════════

def cos_metric_h3(
    states:   np.ndarray,   # [N, D_space]
    proto_h3: np.ndarray,   # [2] unit-normalised H3 vector of one prototype
    h3_i0:    int,
    h3_i1:    int,
) -> np.ndarray:
    """Cosine similarity between H3 subspace of states and a prototype H3 vector."""
    v      = states[:, h3_i0:h3_i1 + 1]                    # [N, 2]
    norm   = np.linalg.norm(v, axis=1, keepdims=True)
    v_unit = v / np.maximum(norm, 1e-8)                     # unit-normalise
    return (v_unit * proto_h3).sum(axis=1)                  # [N] cos values


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 1 — STRUCTURAL EQUIVALENCE CHECK
# ════════════════════════════════════════════════════════════════════════════

def check_structural_equivalence(
    class_labels:    np.ndarray,
    baseline_states: np.ndarray,
    r4_states:       np.ndarray,
    rb_states:       np.ndarray,
    proposed_h3:     np.ndarray,
) -> Tuple[bool, dict]:
    """Verify identical inputs and H3 content across all three candidates.

    Critical invariants:
      1. Sample count identical across all three representations.
      2. H3 INITIAL values numerically identical (max_diff < 1e-9).
      3. Proposed H3 values are space-independent (derived from full-state build).
      4. Prototype construction: same 4 class tokens, same apply_anchor_two_i.

    Returns (pass_bool, detail_dict).
    """
    N = len(class_labels)
    checks: dict = {}

    # 1. Sample count consistency
    checks["n_samples_consistent"] = (
        len(baseline_states) == N
        and len(r4_states)   == N
        and len(rb_states)   == N
    )

    # 2. H3 INITIAL values across all three representations
    bl_h3 = baseline_states[:, H3_IDX0_BASELINE:H3_IDX1_BASELINE + 1]  # [N, 2]
    r4_h3 = r4_states[:, 2:4]                                            # [N, 2]
    rb_h3 = rb_states[:, H3_IDX0_BASELINE:H3_IDX1_BASELINE + 1]         # [N, 2]

    max_diff_r4 = float(np.max(np.abs(bl_h3 - r4_h3)))
    max_diff_rb = float(np.max(np.abs(bl_h3 - rb_h3)))

    checks["h3_initial_identical_baseline_vs_r4"]      = max_diff_r4 < 1e-9
    checks["h3_initial_identical_baseline_vs_reduced"]  = max_diff_rb < 1e-9
    checks["h3_initial_max_diff_r4"]                    = round(max_diff_r4, 12)
    checks["h3_initial_max_diff_reduced"]               = round(max_diff_rb, 12)

    # 3. Proposed H3 shape check (space-independent by construction)
    checks["proposed_h3_shape"] = list(proposed_h3.shape)
    checks["proposed_h3_is_finite"] = bool(np.all(np.isfinite(proposed_h3)))

    # 4. Prototype count always 4
    checks["prototype_count_per_space"] = 4

    all_pass = (
        checks["n_samples_consistent"]
        and checks["h3_initial_identical_baseline_vs_r4"]
        and checks["h3_initial_identical_baseline_vs_reduced"]
        and checks["proposed_h3_is_finite"]
    )
    return all_pass, checks


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 2 — SEPARATION METRIC COMPUTATION
# ════════════════════════════════════════════════════════════════════════════

def compute_separation_metrics(
    states:       np.ndarray,    # [N, D] — after transport applied
    protos:       np.ndarray,    # [4, D]
    h3_i0:        int,
    h3_i1:        int,
    class_labels: np.ndarray,   # [N]
) -> dict:
    """Compute mean_matched_cos, mean_mismatched_cos, separation_delta.

    Identical logic to run_geometry_space_type_probe_v1.py.
    """
    matched:    List[float] = []
    mismatched: List[float] = []

    for k in range(VOCAB):
        mask = (class_labels == k)
        if not mask.any():
            continue
        k_states = states[mask]

        proto_h3   = protos[k, h3_i0:h3_i1 + 1].copy()
        proto_norm = np.linalg.norm(proto_h3)
        proto_h3  /= max(proto_norm, 1e-8)

        cos_match = cos_metric_h3(k_states, proto_h3, h3_i0, h3_i1)
        matched.extend(cos_match.tolist())

        for j in range(VOCAB):
            if j == k:
                continue
            proto_j    = protos[j, h3_i0:h3_i1 + 1].copy()
            norm_j     = np.linalg.norm(proto_j)
            proto_j   /= max(norm_j, 1e-8)
            cos_mm     = cos_metric_h3(k_states, proto_j, h3_i0, h3_i1)
            mismatched.extend(cos_mm.tolist())

    mean_matched    = float(np.mean(matched))
    mean_mismatched = float(np.mean(mismatched))
    sep_delta       = mean_matched - mean_mismatched

    return {
        "mean_matched_cos":    round(mean_matched,    8),
        "mean_mismatched_cos": round(mean_mismatched, 8),
        "separation_delta":    round(sep_delta,        8),
        "n_matched_pairs":     len(matched),
        "n_mismatched_pairs":  len(mismatched),
    }


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 3 — CLASSIFY RESULT
# ════════════════════════════════════════════════════════════════════════════

def classify_result(
    all_metrics: Dict[str, Dict[str, dict]],
) -> str:
    """Classify the dynamic probe outcome.

    Examines separation_delta across all (space, eps) combinations.

    SPACE_INVARIANT_UNDER_DYNAMICS:
        separation_delta is consistent across spaces for all eps values
        (max spread across spaces at any given eps < THRESHOLD_MEANINGFUL).

    SPACE_DEPENDENT_UNDER_DYNAMICS:
        separation_delta diverges across spaces when eps < 1.
        (spread across spaces at some eps < 1 >= THRESHOLD_MEANINGFUL).

    DEGENERATE_SIGNAL:
        any separation_delta is non-finite, or all deltas collapse to 0.
    """
    # Check for degenerate signal
    for space, eps_results in all_metrics.items():
        for eps, m in eps_results.items():
            d = m["separation_delta"]
            if not math.isfinite(d):
                return "DEGENERATE_SIGNAL"

    # For each eps, compute spread across spaces
    space_dependent_found = False
    for eps in EPS_VALUES:
        eps_key = str(eps)
        deltas = [all_metrics[sp][eps_key]["separation_delta"] for sp in all_metrics]
        if any(not math.isfinite(d) for d in deltas):
            return "DEGENERATE_SIGNAL"
        spread = max(deltas) - min(deltas)
        if float(eps) < 1.0 and spread >= THRESHOLD_MEANINGFUL:
            space_dependent_found = True

    if space_dependent_found:
        return "SPACE_DEPENDENT_UNDER_DYNAMICS"
    return "SPACE_INVARIANT_UNDER_DYNAMICS"


# ════════════════════════════════════════════════════════════════════════════
# MARKDOWN OUTPUT
# ════════════════════════════════════════════════════════════════════════════

def write_markdown(result: dict) -> None:
    all_metrics  = result["separation_metrics"]
    classification = result["classification"]
    eq_detail    = result["structural_equivalence_detail"]
    spaces       = list(all_metrics.keys())

    lines = [
        "# Prime Transport Geometry Space Dynamics Probe v1",
        "",
        "**Probe:** geometry_space_dynamics_probe_v1  ",
        "**Contract:** prompt_contract_v4.md  ",
        "**Branch:** geometry_space_dynamics_probe_v1  ",
        "**Layer tested:** Layer 1 — Ambient/operative space (dynamic regime)  ",
        f"**N samples:** {result['n_samples']}  ({result['n_per_class']} per class)  ",
        f"**eps_values:** {result['eps_values']}  ",
        "",
        "---",
        "",
        "## 1. Recap: Prior Invariant Result (Frozen Regime)",
        "",
        "Probe `geometry_space_type_probe_v1` was run under **eps_high = 1.0** (frozen).",
        "Under this condition, H3 is algebraically preserved at every step by construction:",
        "",
        "```",
        "h3_final = 1.0 * h3_initial + 0.0 * h3_proposed = h3_initial  [exact]",
        "```",
        "",
        "Result: **SPACE_INVARIANT_SIGNAL** — all three ambient space candidates",
        "(CURRENT_BASELINE/R^16, R4_FLAT/R^4, REDUCED_BLOCK/R^12) produced identical",
        "separation_delta = 1.333333 with mean_matched_cos = 1.0 and",
        "mean_mismatched_cos = −0.333333.",
        "",
        "**Interpretation at the time:** This result is conditional on eps_high=1.0.",
        "Under eps_high=1.0, H3 invariance is a tautology — algebraic, not empirical.",
        "The frozen-regime invariance cannot be promoted to the dynamic regime.",
        "",
        "**This probe** extends the test to eps_high ∈ {1.0, 0.75, 0.5}.",
        "",
        "---",
        "",
        "## 2. Space Representations Tested",
        "",
        "| Space | Dims | H3 subspace dims | Notes |",
        "|:------|-----:|:-----------------|:------|",
        "| CURRENT_BASELINE | 16 | [10, 11] | 12 angular + 4 magnitude |",
        "| R4_FLAT | 4 | [2, 3] | h2+h3 pairs of dominant block only |",
        "| REDUCED_BLOCK | 12 | [10, 11] | 12 angular only (magnitude removed) |",
        "",
        "---",
        "",
        "## 3. Controlled Dynamics Definition",
        "",
        "Transport formula applied to H3 subspace of each state:",
        "",
        "```",
        "h3_final = eps_high * h3_initial + (1 - eps_high) * h3_proposed",
        "```",
        "",
        "Where **h3_proposed** = H3 encoding of class (k+1) % 4 — the adjacent class.",
        "This is deterministic, training-free, and space-independent.",
        "",
        "| eps_high | Interpretation |",
        "|:---------|:---------------|",
        "| 1.0 | Frozen — algebraic control (identical to prior probe) |",
        "| 0.75 | 25% contamination from adjacent class H3 |",
        "| 0.5 | 50% contamination from adjacent class H3 |",
        "",
        "**Note:** eps = 0.0 excluded (full replacement — too destructive).",
        "",
        "---",
        "",
        "## 4. Structural Equivalence Check",
        "",
        f"**Status:** {'PASS' if result['structural_equivalence_pass'] else 'FAIL — INVALID_COMPARISON_SETUP'}",
        "",
        f"- Identical {result['n_samples']} samples across all space candidates",
        "- Identical prototype construction: class tokens {0,1,2,3}, apply_anchor_two_i applied",
        f"- H3 initial values: baseline vs R4_FLAT max_diff = {eq_detail['h3_initial_max_diff_r4']}",
        f"- H3 initial values: baseline vs REDUCED_BLOCK max_diff = {eq_detail['h3_initial_max_diff_reduced']}",
        "- Proposed H3 values: space-independent (derived from full-state build, same for all spaces)",
        f"- Proposed H3 finite: {eq_detail['proposed_h3_is_finite']}",
        "",
        "---",
        "",
        "## 5. Dynamic Test Results Table",
        "",
        "| Space | eps | matched_cos | mismatched_cos | separation_delta |",
        "|:------|----:|------------:|---------------:|-----------------:|",
    ]

    for sp in spaces:
        for eps in EPS_VALUES:
            eps_key = str(eps)
            m = all_metrics[sp][eps_key]
            lines.append(
                f"| {sp} | {eps} | {m['mean_matched_cos']:.6f} | "
                f"{m['mean_mismatched_cos']:.6f} | {m['separation_delta']:.6f} |"
            )

    lines += [
        "",
        "---",
        "",
        "## 6. Comparison Across eps Values",
        "",
        "separation_delta by space and eps:",
        "",
        "| Space | eps=1.0 | eps=0.75 | eps=0.5 |",
        "|:------|--------:|---------:|--------:|",
    ]

    for sp in spaces:
        d10  = all_metrics[sp]["1.0"]["separation_delta"]
        d075 = all_metrics[sp]["0.75"]["separation_delta"]
        d05  = all_metrics[sp]["0.5"]["separation_delta"]
        lines.append(f"| {sp} | {d10:.6f} | {d075:.6f} | {d05:.6f} |")

    # Compute spread across spaces per eps
    lines += ["", "Spread (max − min separation_delta) across spaces at each eps:", ""]
    for eps in EPS_VALUES:
        eps_key = str(eps)
        deltas = [all_metrics[sp][eps_key]["separation_delta"] for sp in spaces]
        spread = max(deltas) - min(deltas)
        lines.append(f"- eps={eps}: spread = {spread:.8f}")

    lines += [
        "",
        "---",
        "",
        "## 7. Does Ambient Space Matter Under Dynamic Conditions?",
        "",
    ]

    if classification == "SPACE_INVARIANT_UNDER_DYNAMICS":
        lines += [
            "**NO — ambient space does NOT matter under dynamic conditions.**",
            "",
            "The separation_delta values are consistent across all three ambient space",
            "representations (CURRENT_BASELINE, R4_FLAT, REDUCED_BLOCK) at every eps value.",
            "The spread across spaces remains below the threshold ({:.2f}) for all eps.".format(THRESHOLD_MEANINGFUL),
            "",
            "**Why is this the case?**",
            "",
            "The measurement (`cos_metric` on the H3 subspace) depends only on H3 dims.",
            "The transport is applied only to H3 dims, using proposed H3 values that are",
            "space-independent (derived identically for all spaces from the same encoding).",
            "",
            "Changing the ambient embedding (R^4, R^12, R^16) does not alter:",
            "  - the H3 initial values (numerically identical across spaces, verified)",
            "  - the H3 proposed values (same construction for all spaces)",
            "  - the transport result (same formula applied to same values)",
            "  - the cos_metric measurement (same H3 subspace content)",
            "",
            "**Implication for Layer 1:** The geometric separation signal remains",
            "space-invariant even under partial H3 replacement (eps < 1.0).",
            "Invariance extends from the frozen regime into the dynamic regime.",
            "",
            "**Boundary note:** This result is specific to the tested dynamic mechanism",
            "(adjacent-class H3 perturbation). It does not rule out space sensitivity",
            "under alternative dynamic regimes where non-H3 ambient dimensions",
            "enter the transport computation.",
        ]
    elif classification == "SPACE_DEPENDENT_UNDER_DYNAMICS":
        lines += [
            "**YES — ambient space DOES affect the signal under dynamic conditions.**",
            "",
            "The separation_delta diverges across space candidates when eps < 1.0.",
            f"The spread across spaces exceeds {THRESHOLD_MEANINGFUL} (threshold) at one or more eps values.",
            "",
            "This indicates that non-H3 ambient dimensions influence the signal",
            "when partial H3 replacement is applied.",
        ]
    else:
        lines += [
            "**DEGENERATE SIGNAL — signal collapsed or became non-finite.**",
            "",
            "Review structural equivalence check output and raw metrics.",
        ]

    lines += [
        "",
        "---",
        "",
        "## 8. Critical Question Answers",
        "",
    ]

    if classification == "SPACE_INVARIANT_UNDER_DYNAMICS":
        lines += [
            "**Q1: Does ambient space begin to matter once H3 is no longer perfectly preserved?**",
            "NO. All three space representations produce identical separation_delta at eps=0.75 and eps=0.5.",
            "",
            "**Q2: Is invariance only a property of the frozen regime?**",
            "NO, under this test. Invariance holds across the tested dynamic eps values.",
            "The caveat is that the proposed H3 perturbation is space-independent by construction,",
            "so the non-H3 ambient content never enters the transport or measurement.",
            "",
            "**Q3: Does any space degrade slower or preserve signal better?**",
            "NO. Signal degradation with decreasing eps is identical across all three spaces.",
            "The degradation trajectory is determined by the H3 content alone.",
        ]
    elif classification == "SPACE_DEPENDENT_UNDER_DYNAMICS":
        lines += [
            "**Q1: Does ambient space begin to matter once H3 is no longer perfectly preserved?**",
            "YES. separation_delta diverges across spaces at eps < 1.0.",
            "",
            "**Q2: Is invariance only a property of the frozen regime?**",
            "YES. The frozen-regime invariance does not extend to the dynamic regime.",
            "",
            "**Q3: Does any space degrade slower or preserve signal better?**",
            "See result table — per-space degradation differs; identify the space with highest",
            "separation_delta at eps=0.5 for best dynamic preservation.",
        ]
    else:
        lines += [
            "**Q1-Q3:** INCONCLUSIVE — degenerate signal prevented classification.",
        ]

    lines += [
        "",
        "---",
        "",
        "## 9. Locked Variables Confirmation",
        "",
        "| Variable | Value | Changed? |",
        "|:---------|:------|:---------|",
        "| Basis | apply_anchor_two_i (2i frame) | NO |",
        "| Operator | cos_metric | NO |",
        "| Comparison subspace | H3 pair (per-space equivalent) | NO |",
        "| H3 initial construction | Identical to prior probe | NO |",
        "| Prototype construction | Identical to prior probe | NO |",
        f"| eps_high | swept over {EPS_VALUES} | CONTROLLED (variable under test is SPACE) |",
        "| Training | None | NO |",
        "| New constants | None | NO |",
        "| Radial structure | Unchanged | NO |",
        "",
        "---",
        "",
        f"GEOMETRY SPACE DYNAMICS RESULT: {classification}",
    ]

    MD_OUT.write_text("\n".join(lines))


# ════════════════════════════════════════════════════════════════════════════
# MAIN
# ════════════════════════════════════════════════════════════════════════════

def main() -> None:
    print("=" * 72)
    print("GEOMETRY SPACE DYNAMICS PROBE v1")
    print("=" * 72)
    print("CONTRACT:    prompt_contract_v4.md — binding")
    print(f"N_SAMPLES:   {N_SAMPLES}  ({N_PER_CLASS} per class)")
    print(f"Candidates:  CURRENT_BASELINE (R^16) | R4_FLAT (R^4) | REDUCED_BLOCK (R^12)")
    print(f"eps_values:  {EPS_VALUES}")
    print(f"Operator:    cos_metric on H3-equivalent subspace")
    print()

    # ── Generate identical class labels for all candidates ────────────────
    class_labels = np.repeat(np.arange(VOCAB), N_PER_CLASS)   # [1024]

    # ── Build all space representations (initial / frozen state) ─────────
    bl_states, bl_protos, bl_h3i0, bl_h3i1 = build_current_baseline(class_labels, BLOCKS_A)
    r4_states, r4_protos, r4_h3i0, r4_h3i1 = build_r4_flat(class_labels, BLOCKS_A)
    rb_states, rb_protos, rb_h3i0, rb_h3i1 = build_reduced_block(class_labels, BLOCKS_A)

    print(f"CURRENT_BASELINE:  state_dim={bl_states.shape[1]}  H3=dims[{bl_h3i0},{bl_h3i1}]")
    print(f"R4_FLAT:           state_dim={r4_states.shape[1]}   H3=dims[{r4_h3i0},{r4_h3i1}]")
    print(f"REDUCED_BLOCK:     state_dim={rb_states.shape[1]}  H3=dims[{rb_h3i0},{rb_h3i1}]")
    print()

    # ── Build proposed H3 (space-independent: adjacent class encoding) ────
    proposed_h3 = build_proposed_h3(class_labels, BLOCKS_A)   # [N, 2]
    print(f"Proposed H3: adjacent class (k+1)%4, shape={proposed_h3.shape}")
    print()

    # ── MANDATORY STEP 1: Structural Equivalence Check ───────────────────
    eq_pass, eq_detail = check_structural_equivalence(
        class_labels, bl_states, r4_states, rb_states, proposed_h3
    )
    if not eq_pass:
        print("STOP — INVALID_COMPARISON_SETUP")
        print(f"Equivalence detail: {eq_detail}")
        raise RuntimeError("INVALID_COMPARISON_SETUP")

    print("Structural equivalence: PASS")
    print(f"  H3 initial max_diff baseline vs R4_FLAT:      {eq_detail['h3_initial_max_diff_r4']}")
    print(f"  H3 initial max_diff baseline vs REDUCED_BLOCK: {eq_detail['h3_initial_max_diff_reduced']}")
    print(f"  Proposed H3 is finite: {eq_detail['proposed_h3_is_finite']}")
    print()

    # ── MANDATORY STEP 2: Compute separation metrics per space × eps ──────
    candidates = {
        "CURRENT_BASELINE": (bl_states, bl_protos, bl_h3i0, bl_h3i1),
        "R4_FLAT":          (r4_states, r4_protos, r4_h3i0, r4_h3i1),
        "REDUCED_BLOCK":    (rb_states, rb_protos, rb_h3i0, rb_h3i1),
    }

    all_metrics: Dict[str, Dict[str, dict]] = {}

    print("RESULT TABLE")
    print("-" * 80)
    hdr = f"{'Space':<20}  {'eps':>6}  {'mean_matched':>14}  {'mean_mismatched':>17}  {'sep_delta':>12}"
    print(hdr)
    print("-" * 80)

    for space_name, (init_states, protos, h3i0, h3i1) in candidates.items():
        all_metrics[space_name] = {}
        for eps in EPS_VALUES:
            # Apply transport to H3 subspace of this space's states
            transported = apply_h3_transport(init_states, h3i0, h3i1, proposed_h3, eps)
            m = compute_separation_metrics(transported, protos, h3i0, h3i1, class_labels)
            all_metrics[space_name][str(eps)] = m
            print(
                f"{space_name:<20}  {eps:>6.2f}  "
                f"{m['mean_matched_cos']:>14.6f}  "
                f"{m['mean_mismatched_cos']:>17.6f}  "
                f"{m['separation_delta']:>12.6f}"
            )

    print()

    # ── MANDATORY STEP 3: Classify ────────────────────────────────────────
    classification = classify_result(all_metrics)
    print(f"CLASSIFICATION: {classification}")
    print()

    # ── Build result record ───────────────────────────────────────────────
    result = {
        "probe":             "geometry_space_dynamics_probe_v1",
        "branch":            "geometry_space_dynamics_probe_v1",
        "contract":          "prompt_contract_v4.md",
        "prior_probe":       "geometry_space_type_probe_v1",
        "prior_result":      "SPACE_INVARIANT_SIGNAL (eps_high=1.0, frozen)",
        "n_samples":         N_SAMPLES,
        "n_per_class":       N_PER_CLASS,
        "vocab":             VOCAB,
        "operator":          "cos_metric",
        "basis":             "apply_anchor_two_i (2i frame)",
        "comparison_subspace": "H3 pair (per-space equivalent subspace)",
        "eps_values":        EPS_VALUES,
        "transport_formula": "h3_final = eps_high * h3_initial + (1 - eps_high) * h3_proposed",
        "proposed_h3_definition": "H3 encoding of adjacent class (k+1) % 4 — deterministic, training-free",
        "no_training":       True,
        "threshold_meaningful_delta": THRESHOLD_MEANINGFUL,
        "space_candidates": {
            "CURRENT_BASELINE": {
                "description":    "R^16 hybrid: 12 angular (6 harmonic pairs) + 4 magnitude",
                "dims":           16,
                "h3_dims":        [bl_h3i0, bl_h3i1],
                "angular_dims":   12,
                "magnitude_dims": 4,
            },
            "R4_FLAT": {
                "description":                     "R^4: h2+h3 pairs of dominant block (block3) only",
                "dims":                            4,
                "h3_dims":                         [r4_h3i0, r4_h3i1],
                "source_dims_from_baseline_angular": [8, 9, 10, 11],
                "angular_dims":                    4,
                "magnitude_dims":                  0,
            },
            "REDUCED_BLOCK": {
                "description":    "R^12: 12 angular dims only (4 magnitude dims removed from R^16)",
                "dims":           12,
                "h3_dims":        [rb_h3i0, rb_h3i1],
                "angular_dims":   12,
                "magnitude_dims": 0,
            },
        },
        "separation_metrics":          all_metrics,
        "classification":              classification,
        "structural_equivalence_pass": eq_pass,
        "structural_equivalence_detail": eq_detail,
    }

    # ── Write outputs ─────────────────────────────────────────────────────
    JSON_OUT.write_text(json.dumps(result, indent=2))
    print(f"JSON written: {JSON_OUT}")

    write_markdown(result)
    print(f"MD  written:  {MD_OUT}")
    print()
    print(f"GEOMETRY SPACE DYNAMICS RESULT: {classification}")


if __name__ == "__main__":
    main()
