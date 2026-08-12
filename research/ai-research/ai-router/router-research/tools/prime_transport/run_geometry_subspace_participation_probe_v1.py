#!/usr/bin/env python3
"""run_geometry_subspace_participation_probe_v1.py

GEOMETRY SUBSPACE PARTICIPATION PROBE v1
==========================================

BRANCH:   geometry_subspace_participation_probe_v1
CONTRACT: prompt_contract_v4.md — loaded and binding
DEPENDENCY TABLE: prime_transport_geometry_dependency_order_probe_v1.md — loaded and binding

════════════════════════════════════════════════════════════════════════
PURPOSE
════════════════════════════════════════════════════════════════════════

Layer 1 probe — with real participation from non-H3 subspaces.

Prior probes confirmed SPACE_INVARIANT signal under both frozen
(eps=1.0) and dynamic (eps<1.0) conditions, but ONLY when H3 was the
comparison subspace.

Root issue: ambient space never participated in transport, comparison,
or projection. H3-local signal is space-invariant. But Layer 1 is
STILL unresolved for non-local comparison and multi-subspace interaction.

This probe asks: does the geometric separation signal extend beyond H3,
and does ambient space matter when MULTIPLE SUBSPACES participate?

════════════════════════════════════════════════════════════════════════
LOCKED VARIABLES (unchanged across ALL candidates)
════════════════════════════════════════════════════════════════════════

  - Basis:              apply_anchor_two_i  (2i frame)
  - Operator:           cos (cosine similarity) ONLY
  - Normalization:      unit-norm per subspace vector (no reweighting)
  - eps_high:           1.0  (frozen — algebraic H3 preservation)
  - No training:        TRUE
  - No new constants:   TRUE (no phi, no heuristic ratios)
  - No radial changes:  TRUE
  - No D sweeps:        TRUE

════════════════════════════════════════════════════════════════════════
VARIABLE UNDER TEST (ONLY ONE)
════════════════════════════════════════════════════════════════════════

  Which subspace participates in the comparison mechanism.

SUBSPACE CANDIDATES (within block3, period=12, n_h=3):
  1. H3_ONLY   — h=3 of block3 (control)   → 2D
  2. H2_ONLY   — h=2 of block3             → 2D
  3. H1_ONLY   — h=1 of block3             → 2D
  4. H2_PLUS_H3 — h=2 and h=3 concatenated → 4D
  5. H1_PLUS_H3 — h=1 and h=3 concatenated → 4D
  6. FULL_B3   — h=1, h=2, h=3 of block3   → 6D

AVAILABILITY per space:
  - CURRENT_BASELINE (R^16):   all 6 subspaces available
  - REDUCED_BLOCK   (R^12):   all 6 subspaces available
  - R4_FLAT         (R^4):    H2_ONLY, H3_ONLY, H2_PLUS_H3 only
                               (H1 not present in R4_FLAT)

════════════════════════════════════════════════════════════════════════
SPACE CANDIDATES (same as prior probes)
════════════════════════════════════════════════════════════════════════

  1. CURRENT_BASELINE — R^16 hybrid (12 angular + 4 magnitude)
  2. R4_FLAT          — R^4 (h2+h3 pairs of dominant block3 only)
  3. REDUCED_BLOCK    — R^12 (12 angular only; magnitude dims removed)

════════════════════════════════════════════════════════════════════════
MEASUREMENT (per space × subspace)
════════════════════════════════════════════════════════════════════════

  mean_matched_cos:    mean cos(state_k, proto_k) on subspace
  mean_mismatched_cos: mean cos(state_k, proto_j) on subspace, j≠k
  separation_delta:    mean_matched_cos − mean_mismatched_cos

════════════════════════════════════════════════════════════════════════
LAYER ISOLATION COMPLIANCE
════════════════════════════════════════════════════════════════════════

  NOT changed: Layers 2-7 (basis family, operator, constants,
               comparison mechanism [beyond subspace choice],
               D/workspace, training).
  TESTED:      Layer 1 only (ambient space representation) and
               Layer 5 (which subspace participates in comparison).

CONTRACT RULES COMPLIED:
  Rule 1 (Mechanism Lock):    Mechanism defined pre-code.
  Rule 2 (No Hidden Changes): Only subspace choice varies.
  Rule 3 (Geometry Consistency): All computations in 2i frame.
  Rule 4 (Deterministic First): No training, no learning, eps=1.0.
  Rule 5 (Compare Regimes):   6 subspaces × 3 spaces compared.
  Rule 6 (Output Discipline): JSON + Markdown + explicit classification.
  Rule 7 (Failure Is Valid):  H3-dominance accepted if observed.
  Rule 8 (Reuse Components):  apply_anchor_two_i and state builders
                               reused from prior probe machinery.
  Rule 9 (No Theory Injection): No phi, no external constants.
"""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np

# ── Output paths ──────────────────────────────────────────────────────────────
_ROOT    = Path(__file__).resolve().parents[2]
JSON_OUT = (
    _ROOT / "results" / "prime_transport_recursive_system"
    / "geometry_subspace_participation_probe_v1.json"
)
MD_OUT = (
    _ROOT / "docs" / "research"
    / "prime_transport_geometry_subspace_participation_probe_v1.md"
)
JSON_OUT.parent.mkdir(parents=True, exist_ok=True)
MD_OUT.parent.mkdir(parents=True, exist_ok=True)

# ── Experiment constants ──────────────────────────────────────────────────────
VOCAB         = 4
N_PER_CLASS   = 256
N_SAMPLES     = N_PER_CLASS * VOCAB    # 1024 samples total
BLOCKS_A      = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# H_n angular indices in the 12-dim angular vector (block3 harmonics):
#   Block3 layout: h1=dims[6,7]  h2=dims[8,9]  h3=dims[10,11]
H1_IDX_BASELINE = [6, 7]
H2_IDX_BASELINE = [8, 9]
H3_IDX_BASELINE = [10, 11]

# Subspace index definitions per space:
#
# CURRENT_BASELINE (R^16) — state dims = [0..15]:
#   angular dims [0..11], magnitude dims [12..15]
#   Block3: H1=[6,7], H2=[8,9], H3=[10,11]
#
# REDUCED_BLOCK (R^12) — state dims = [0..11]:
#   angular dims only; same layout as angular part of BASELINE
#   Block3: H1=[6,7], H2=[8,9], H3=[10,11]
#
# R4_FLAT (R^4) — state dims = [0..3]:
#   Sliced from baseline angular dims [8,9,10,11]
#   dim 0 = baseline angular dim 8 = block3 H2 cos
#   dim 1 = baseline angular dim 9 = block3 H2 sin
#   dim 2 = baseline angular dim 10 = block3 H3 cos
#   dim 3 = baseline angular dim 11 = block3 H3 sin
#   Block3: H2=[0,1], H3=[2,3]   (H1 NOT AVAILABLE)

SUBSPACE_DEFS: Dict[str, Dict[str, Optional[List[int]]]] = {
    # subspace_name -> {space_name -> dim_indices or None if unavailable}
    "H3_ONLY": {
        "CURRENT_BASELINE": [10, 11],
        "REDUCED_BLOCK":    [10, 11],
        "R4_FLAT":          [2, 3],
    },
    "H2_ONLY": {
        "CURRENT_BASELINE": [8, 9],
        "REDUCED_BLOCK":    [8, 9],
        "R4_FLAT":          [0, 1],
    },
    "H1_ONLY": {
        "CURRENT_BASELINE": [6, 7],
        "REDUCED_BLOCK":    [6, 7],
        "R4_FLAT":          None,   # H1 not present in R4_FLAT
    },
    "H2_PLUS_H3": {
        "CURRENT_BASELINE": [8, 9, 10, 11],
        "REDUCED_BLOCK":    [8, 9, 10, 11],
        "R4_FLAT":          [0, 1, 2, 3],
    },
    "H1_PLUS_H3": {
        "CURRENT_BASELINE": [6, 7, 10, 11],
        "REDUCED_BLOCK":    [6, 7, 10, 11],
        "R4_FLAT":          None,   # H1 not present in R4_FLAT
    },
    "FULL_B3": {
        "CURRENT_BASELINE": [6, 7, 8, 9, 10, 11],
        "REDUCED_BLOCK":    [6, 7, 8, 9, 10, 11],
        "R4_FLAT":          None,   # H1 not present in R4_FLAT
    },
}

SPACE_NAMES     = ["CURRENT_BASELINE", "REDUCED_BLOCK", "R4_FLAT"]
SUBSPACE_NAMES  = ["H3_ONLY", "H2_ONLY", "H1_ONLY", "H2_PLUS_H3", "H1_PLUS_H3", "FULL_B3"]
THRESHOLD_MEANINGFUL = 0.05   # delta range threshold for SPACE_DEPENDENT classification


# ════════════════════════════════════════════════════════════════════════════
# GEOMETRY HELPERS (reused from prior probe machinery)
# ════════════════════════════════════════════════════════════════════════════

def geom_dims(blocks: list) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def convert_onehot_to_angular_multi(onehot: np.ndarray, blocks: list) -> np.ndarray:
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

    Identical to apply_anchor_two_i in prior geometry probes.
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

def build_full_r16_states(class_labels: np.ndarray, blocks: list) -> np.ndarray:
    """Build R^16 hybrid states for all samples (2i frame, eps_high=1.0).

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


def build_class_prototypes_r16(blocks: list) -> np.ndarray:
    """Build 4 class prototypes in R^16 (one per class k=0,1,2,3)."""
    labels = np.array([0, 1, 2, 3], dtype=int)
    return build_full_r16_states(labels, blocks)    # [4, 16]


# ════════════════════════════════════════════════════════════════════════════
# SPACE CANDIDATE BUILDERS
# ════════════════════════════════════════════════════════════════════════════

def build_current_baseline(
    class_labels: np.ndarray,
    blocks: list,
) -> Tuple[np.ndarray, np.ndarray]:
    """CURRENT_BASELINE: R^16 (12 angular + 4 magnitude).

    Returns (states[N,16], protos[4,16]).
    """
    states = build_full_r16_states(class_labels, blocks)
    protos = build_class_prototypes_r16(blocks)
    return states, protos


def build_r4_flat(
    class_labels: np.ndarray,
    blocks: list,
) -> Tuple[np.ndarray, np.ndarray]:
    """R4_FLAT: 4D space = h2 and h3 pairs of dominant block (block3).

    Source: baseline angular dims [8, 9, 10, 11].
    apply_anchor_two_i is inherited (applied in full-state build; slice preserves it).
    """
    full_states = build_full_r16_states(class_labels, blocks)   # [N, 16]
    full_protos = build_class_prototypes_r16(blocks)             # [4, 16]

    r4_states = full_states[:, 8:12].copy()   # [N, 4]
    r4_protos = full_protos[:, 8:12].copy()   # [4, 4]

    return r4_states, r4_protos


def build_reduced_block(
    class_labels: np.ndarray,
    blocks: list,
) -> Tuple[np.ndarray, np.ndarray]:
    """REDUCED_BLOCK: R^12 (12 angular dims only; magnitude dims removed).

    H3 comparison subspace: dims [10, 11] (same as baseline).
    """
    d_ang, _, _ = geom_dims(blocks)   # d_ang = 12
    full_states = build_full_r16_states(class_labels, blocks)   # [N, 16]
    full_protos = build_class_prototypes_r16(blocks)             # [4, 16]

    rb_states = full_states[:, :d_ang].copy()   # [N, 12]
    rb_protos = full_protos[:, :d_ang].copy()   # [4, 12]

    return rb_states, rb_protos


# ════════════════════════════════════════════════════════════════════════════
# COS METRIC (generic subspace version)
# ════════════════════════════════════════════════════════════════════════════

def cos_metric_subspace(
    states:    np.ndarray,   # [N, D_space]
    proto_sub: np.ndarray,   # [S] — unit-normalised subspace vector of one prototype
    sub_dims:  List[int],    # which dims to extract from states
) -> np.ndarray:
    """Cosine similarity between state subspace and prototype subspace vector.

    Extracts sub_dims from states, unit-normalises the subspace slice as a
    whole vector, then computes dot product against proto_sub.

    Same operator (cos), same normalization, no reweighting.
    """
    v      = states[:, sub_dims]                              # [N, S]
    norm   = np.linalg.norm(v, axis=1, keepdims=True)        # [N, 1]
    v_unit = v / np.maximum(norm, 1e-8)                      # [N, S]
    return v_unit @ proto_sub                                 # [N]


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 1 — VALIDITY CHECK
# Verify all subspaces are correctly extracted; no silent projection collapse
# ════════════════════════════════════════════════════════════════════════════

def check_subspace_validity(
    baseline_states: np.ndarray,   # [N, 16]
    reduced_states:  np.ndarray,   # [N, 12]
    r4_states:       np.ndarray,   # [N, 4]
    N:               int,
) -> Tuple[bool, dict]:
    """Verify:
    1. All spaces have the same N samples.
    2. Shared subspace values (H2, H3) are numerically identical across spaces.
    3. No silent dimension collapse (all subspace slices are non-degenerate).
    """
    checks: dict = {}

    # 1. Sample count consistency
    checks["n_samples_consistent"] = (
        len(baseline_states) == N
        and len(reduced_states) == N
        and len(r4_states)     == N
    )

    # 2. H3 values: baseline ↔ reduced ↔ r4
    bl_h3 = baseline_states[:, [10, 11]]    # [N, 2]
    rd_h3 = reduced_states[:, [10, 11]]     # [N, 2]
    r4_h3 = r4_states[:, [2, 3]]            # [N, 2]

    max_diff_rd_h3 = float(np.max(np.abs(bl_h3 - rd_h3)))
    max_diff_r4_h3 = float(np.max(np.abs(bl_h3 - r4_h3)))
    checks["h3_baseline_vs_reduced_max_diff"] = round(max_diff_rd_h3, 12)
    checks["h3_baseline_vs_r4_max_diff"]      = round(max_diff_r4_h3, 12)
    checks["h3_baseline_vs_reduced_pass"]     = max_diff_rd_h3 < 1e-9
    checks["h3_baseline_vs_r4_pass"]          = max_diff_r4_h3 < 1e-9

    # 3. H2 values: baseline ↔ reduced ↔ r4
    bl_h2 = baseline_states[:, [8, 9]]     # [N, 2]
    rd_h2 = reduced_states[:, [8, 9]]      # [N, 2]
    r4_h2 = r4_states[:, [0, 1]]           # [N, 2]

    max_diff_rd_h2 = float(np.max(np.abs(bl_h2 - rd_h2)))
    max_diff_r4_h2 = float(np.max(np.abs(bl_h2 - r4_h2)))
    checks["h2_baseline_vs_reduced_max_diff"] = round(max_diff_rd_h2, 12)
    checks["h2_baseline_vs_r4_max_diff"]      = round(max_diff_r4_h2, 12)
    checks["h2_baseline_vs_reduced_pass"]     = max_diff_rd_h2 < 1e-9
    checks["h2_baseline_vs_r4_pass"]          = max_diff_r4_h2 < 1e-9

    # 4. H1 values: baseline ↔ reduced (only, not in R4_FLAT)
    bl_h1 = baseline_states[:, [6, 7]]     # [N, 2]
    rd_h1 = reduced_states[:, [6, 7]]      # [N, 2]
    max_diff_rd_h1 = float(np.max(np.abs(bl_h1 - rd_h1)))
    checks["h1_baseline_vs_reduced_max_diff"] = round(max_diff_rd_h1, 12)
    checks["h1_baseline_vs_reduced_pass"]     = max_diff_rd_h1 < 1e-9

    # 5. Non-degenerate check: no subspace slice is all zeros
    # (unit-norm pairs after 2i rotation should never collapse to zero)
    subspace_slices_to_check = {
        "bl_h1": baseline_states[:, [6, 7]],
        "bl_h2": baseline_states[:, [8, 9]],
        "bl_h3": baseline_states[:, [10, 11]],
        "r4_h2": r4_states[:, [0, 1]],
        "r4_h3": r4_states[:, [2, 3]],
    }
    degenerate_flags = {}
    for name, arr in subspace_slices_to_check.items():
        norms = np.linalg.norm(arr, axis=1)
        degenerate_flags[f"{name}_min_norm"] = round(float(np.min(norms)), 8)
        degenerate_flags[f"{name}_all_nonzero"] = bool(np.all(norms > 1e-8))
    checks["subspace_norms"] = degenerate_flags

    all_pass = (
        checks["n_samples_consistent"]
        and checks["h3_baseline_vs_reduced_pass"]
        and checks["h3_baseline_vs_r4_pass"]
        and checks["h2_baseline_vs_reduced_pass"]
        and checks["h2_baseline_vs_r4_pass"]
        and checks["h1_baseline_vs_reduced_pass"]
        and all(v for k, v in degenerate_flags.items() if k.endswith("_all_nonzero"))
    )
    return all_pass, checks


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 2 — SEPARATION METRIC COMPUTATION
# ════════════════════════════════════════════════════════════════════════════

def compute_separation_metrics(
    states:       np.ndarray,    # [N, D]
    protos:       np.ndarray,    # [4, D]
    sub_dims:     List[int],
    class_labels: np.ndarray,   # [N]
) -> Dict:
    """Compute mean_matched_cos, mean_mismatched_cos, separation_delta.

    For each class k:
      - matched:    cos_metric_subspace(state_k, proto_k[sub_dims])
      - mismatched: cos_metric_subspace(state_k, proto_j[sub_dims]) for j ≠ k
    """
    matched:    List[float] = []
    mismatched: List[float] = []

    for k in range(VOCAB):
        mask = (class_labels == k)
        if not mask.any():
            continue
        k_states = states[mask]   # [N_k, D]

        # Unit-normalise prototype k's subspace vector
        proto_sub    = protos[k, sub_dims].copy().astype(float)
        proto_norm   = np.linalg.norm(proto_sub)
        proto_sub   /= max(proto_norm, 1e-8)

        cos_match = cos_metric_subspace(k_states, proto_sub, sub_dims)
        matched.extend(cos_match.tolist())

        for j in range(VOCAB):
            if j == k:
                continue
            proto_j     = protos[j, sub_dims].copy().astype(float)
            norm_j      = np.linalg.norm(proto_j)
            proto_j    /= max(norm_j, 1e-8)
            cos_mm      = cos_metric_subspace(k_states, proto_j, sub_dims)
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
# MANDATORY STEP 3 — CLASSIFICATION
# ════════════════════════════════════════════════════════════════════════════

def classify_result(separation_metrics: Dict[str, Dict[str, Dict]]) -> str:
    """Classify the probe result into one of four categories.

    Criteria:
      H3_ONLY_SIGNAL:
        - H3_ONLY separation_delta is the maximum (across all subspaces)
          for all spaces, AND all non-H3 subspaces have substantially
          lower delta (< H3_ONLY delta - THRESHOLD_MEANINGFUL).

      MULTI_SUBSPACE_SIGNAL:
        - Other subspaces or combinations ALSO produce usable signal
          (delta > THRESHOLD_MEANINGFUL) comparable to or exceeding H3_ONLY.

      SPACE_DEPENDENT_SUBSPACE_BEHAVIOR:
        - The pattern of which subspace is strongest differs across spaces.

      DEGENERATE:
        - signal collapses or is non-finite.
    """
    # Collect delta per (space, subspace)
    finite_check = True
    for space_name, sub_dict in separation_metrics.items():
        for sub_name, m in sub_dict.items():
            if m is None:
                continue
            if not math.isfinite(m["separation_delta"]):
                finite_check = False
    if not finite_check:
        return "DEGENERATE"

    # Best subspace per space (highest delta)
    best_sub_per_space: Dict[str, str] = {}
    for space_name, sub_dict in separation_metrics.items():
        best_delta = -999.0
        best_sub   = None
        for sub_name, m in sub_dict.items():
            if m is None:
                continue
            if m["separation_delta"] > best_delta:
                best_delta = m["separation_delta"]
                best_sub   = sub_name
        best_sub_per_space[space_name] = best_sub  # type: ignore

    # Check if best-subspace is consistent across spaces that share it
    best_subs = set(v for v in best_sub_per_space.values() if v is not None)

    # H3_ONLY_SIGNAL: H3 is best everywhere, and others are substantially worse
    h3_is_best_everywhere = all(s == "H3_ONLY" for s in best_subs)

    if h3_is_best_everywhere:
        # Check whether other subspaces are substantially worse than H3
        h3_dominates = True
        for space_name, sub_dict in separation_metrics.items():
            h3_delta = sub_dict.get("H3_ONLY", {})
            if h3_delta is None:
                continue
            h3_d = h3_delta["separation_delta"] if h3_delta else 0.0
            for sub_name, m in sub_dict.items():
                if sub_name == "H3_ONLY" or m is None:
                    continue
                if m["separation_delta"] > h3_d - THRESHOLD_MEANINGFUL:
                    h3_dominates = False
                    break
        if h3_dominates:
            return "H3_ONLY_SIGNAL"

    # SPACE_DEPENDENT_SUBSPACE_BEHAVIOR: best subspace differs by space
    if len(best_subs) > 1:
        return "SPACE_DEPENDENT_SUBSPACE_BEHAVIOR"

    # MULTI_SUBSPACE_SIGNAL: other subspaces also carry usable signal
    # (within THRESHOLD_MEANINGFUL of H3_ONLY)
    multi_sub_signal = False
    for space_name, sub_dict in separation_metrics.items():
        h3_m = sub_dict.get("H3_ONLY")
        if h3_m is None:
            continue
        h3_d = h3_m["separation_delta"]
        for sub_name, m in sub_dict.items():
            if sub_name == "H3_ONLY" or m is None:
                continue
            if m["separation_delta"] > h3_d - THRESHOLD_MEANINGFUL:
                multi_sub_signal = True
                break

    if multi_sub_signal:
        return "MULTI_SUBSPACE_SIGNAL"

    # Fallback: all non-H3 subspaces substantially below H3, but H3 not unique best
    return "H3_ONLY_SIGNAL"


# ════════════════════════════════════════════════════════════════════════════
# MARKDOWN OUTPUT
# ════════════════════════════════════════════════════════════════════════════

def write_markdown(
    result:             dict,
    separation_metrics: Dict[str, Dict[str, Optional[Dict]]],
    validity_pass:      bool,
    validity_detail:    dict,
    classification:     str,
) -> None:
    lines: List[str] = [
        "# Prime Transport Geometry Subspace Participation Probe v1",
        "",
        "**Probe:** geometry_subspace_participation_probe_v1  ",
        "**Contract:** prompt_contract_v4.md — binding  ",
        "**Dependency table:** prime_transport_geometry_dependency_order_probe_v1.md — binding  ",
        "**Branch:** geometry_subspace_participation_probe_v1  ",
        "**Layer tested:** Layer 1 (ambient space) × Layer 5 (comparison subspace)  ",
        f"**N samples:** {result['n_samples']}  ({result['n_per_class']} per class)  ",
        "**Basis:** apply_anchor_two_i (2i frame)  ",
        "**Operator:** cos (cosine similarity)  ",
        "**Normalization:** unit-norm per subspace vector (no reweighting)  ",
        "**eps_high:** 1.0 (frozen — algebraic H3 preservation)  ",
        "**Training:** NONE  ",
        "",
        "---",
        "",
        "## 1. Subspace Definitions",
        "",
        "All subspaces are harmonics of **block3** (dominant block, period=12, n_h=3).",
        "apply_anchor_two_i is applied to ALL harmonic pairs in the 2i frame before",
        "any subspace slice is extracted.",
        "",
        "### Block3 Harmonic Layout (in 12-dim angular vector of R^16)",
        "",
        "| Harmonic | Angle formula | Classes at (in 2i frame) | Angular dims (R^16) |",
        "|:---------|:--------------|:-------------------------|:-------------------|",
        "| H1 (h=1) | θ_k = π·k/6   | 30° spacing (not orthogonal) | [6, 7] |",
        "| H2 (h=2) | θ_k = π·k/3   | 60° spacing (not orthogonal) | [8, 9] |",
        "| H3 (h=3) | θ_k = π·k/2   | 90° spacing (orthogonal for 4 classes) | [10, 11] |",
        "",
        "**Why H3 is special:** With period=12 and h=3, the 4 classes are spaced at",
        "exactly 90° intervals → mutually orthogonal unit vectors → maximal separation.",
        "",
        "### Subspace Candidates",
        "",
        "| Subspace | Dims (BASELINE/REDUCED) | Dims (R4_FLAT) | Size |",
        "|:---------|:------------------------|:---------------|:-----|",
        "| H3_ONLY | [10, 11] | [2, 3] | 2D |",
        "| H2_ONLY | [8, 9] | [0, 1] | 2D |",
        "| H1_ONLY | [6, 7] | N/A | 2D |",
        "| H2_PLUS_H3 | [8, 9, 10, 11] | [0, 1, 2, 3] | 4D |",
        "| H1_PLUS_H3 | [6, 7, 10, 11] | N/A | 4D |",
        "| FULL_B3 | [6, 7, 8, 9, 10, 11] | N/A | 6D |",
        "",
        "Note: H1 is unavailable in R4_FLAT (which only contains h2 and h3 of block3).",
        "",
        "---",
        "",
        "## 2. Mandatory Step 1 — Validity Check",
        "",
        f"**Status:** {'PASS' if validity_pass else 'FAIL — INVALID_SUBSPACE_SETUP'}",
        "",
        "Checks performed:",
        f"- Sample count consistent: {validity_detail.get('n_samples_consistent')}",
        f"- H3 baseline vs reduced max_diff: {validity_detail.get('h3_baseline_vs_reduced_max_diff')}",
        f"- H3 baseline vs R4_FLAT max_diff: {validity_detail.get('h3_baseline_vs_r4_max_diff')}",
        f"- H2 baseline vs reduced max_diff: {validity_detail.get('h2_baseline_vs_reduced_max_diff')}",
        f"- H2 baseline vs R4_FLAT max_diff: {validity_detail.get('h2_baseline_vs_r4_max_diff')}",
        f"- H1 baseline vs reduced max_diff: {validity_detail.get('h1_baseline_vs_reduced_max_diff')}",
        "",
    ]

    # Subspace norms
    snorms = validity_detail.get("subspace_norms", {})
    for k, v in snorms.items():
        lines.append(f"- Subspace norms {k}: {v}")
    lines.append("")

    if not validity_pass:
        lines += [
            "**INVALID_SUBSPACE_SETUP — experiment results are invalid.**",
            "",
            f"GEOMETRY SUBSPACE PARTICIPATION RESULT: INVALID_SUBSPACE_SETUP",
        ]
        MD_OUT.write_text("\n".join(lines))
        return

    lines += [
        "---",
        "",
        "## 3. Mandatory Step 2 — Full Result Table",
        "",
        "| Space | Subspace | mean_matched_cos | mean_mismatched_cos | separation_delta |",
        "|:------|:---------|----------------:|--------------------:|----------------:|",
    ]

    all_rows: List[Tuple[str, str, float]] = []
    for space in SPACE_NAMES:
        for sub in SUBSPACE_NAMES:
            m = separation_metrics.get(space, {}).get(sub)
            if m is None:
                lines.append(
                    f"| {space} | {sub} | N/A | N/A | N/A |"
                )
            else:
                lines.append(
                    f"| {space} | {sub} | {m['mean_matched_cos']:.6f}"
                    f" | {m['mean_mismatched_cos']:.6f}"
                    f" | {m['separation_delta']:.6f} |"
                )
                all_rows.append((space, sub, m["separation_delta"]))

    lines += [
        "",
        "---",
        "",
        "## 4. Comparison Across Subspaces",
        "",
        "### Separation delta ranked by subspace (averaged over available spaces):",
        "",
        "| Subspace | mean delta (BASELINE) | mean delta (REDUCED) | mean delta (R4_FLAT) | Notes |",
        "|:---------|---------------------:|--------------------:|--------------------:|:------|",
    ]

    for sub in SUBSPACE_NAMES:
        vals = []
        notes = []
        for space in SPACE_NAMES:
            m = separation_metrics.get(space, {}).get(sub)
            if m is None:
                vals.append("N/A")
                if space == "R4_FLAT":
                    notes.append("H1 not in R4")
            else:
                vals.append(f"{m['separation_delta']:.6f}")
        note_str = "; ".join(notes) if notes else ""
        lines.append(f"| {sub} | {vals[0]} | {vals[1]} | {vals[2]} | {note_str} |")

    lines += [
        "",
        "---",
        "",
        "## 5. Does Signal Extend Beyond H3?",
        "",
    ]

    # Collect H3 deltas and others
    h3_deltas = {}
    other_best_deltas = {}
    for space in SPACE_NAMES:
        h3_m = separation_metrics.get(space, {}).get("H3_ONLY")
        if h3_m is not None:
            h3_deltas[space] = h3_m["separation_delta"]
        best_non_h3 = max(
            (separation_metrics[space].get(sub, None) or {}).get("separation_delta", -999.0)
            for sub in SUBSPACE_NAMES if sub != "H3_ONLY"
        )
        other_best_deltas[space] = best_non_h3

    if classification in ("H3_ONLY_SIGNAL",):
        lines += [
            "**No.** The geometric separation signal is confined to H3_ONLY.",
            "",
            "Non-H3 subspaces (H1, H2) and combined subspaces provide substantially",
            "lower separation_delta than H3_ONLY. Combining H3 with other harmonics",
            "does not improve and may reduce separation.",
            "",
        ]
    elif classification in ("MULTI_SUBSPACE_SIGNAL",):
        lines += [
            "**Yes.** Other subspaces or combinations also produce usable signal,",
            "within the threshold of H3_ONLY separation.",
            "",
        ]
    elif classification in ("SPACE_DEPENDENT_SUBSPACE_BEHAVIOR",):
        lines += [
            "**Partially.** Which subspace is strongest depends on the ambient space.",
            "",
        ]
    else:
        lines += [
            "**Unclear.** Signal collapsed or inconsistent.",
            "",
        ]

    lines += [
        "| Space | H3_ONLY delta | Best non-H3 delta | H3 dominates? |",
        "|:------|-------------:|------------------:|:--------------|",
    ]
    for space in SPACE_NAMES:
        h3_d = h3_deltas.get(space, float("nan"))
        oth_d = other_best_deltas.get(space, float("nan"))
        dominates = (
            (h3_d - oth_d) > THRESHOLD_MEANINGFUL
            if math.isfinite(h3_d) and math.isfinite(oth_d)
            else "N/A"
        )
        lines.append(
            f"| {space} | {h3_d:.6f} | {oth_d:.6f} | {dominates} |"
        )

    lines += [
        "",
        "---",
        "",
        "## 6. Does Ambient Space Matter When Multiple Subspaces Participate?",
        "",
    ]

    # Check if delta for each subspace is the same across BASELINE and REDUCED_BLOCK
    # (R4_FLAT is partially excluded due to unavailability of H1 subspaces)
    space_dependent = False
    for sub in SUBSPACE_NAMES:
        m_bl = separation_metrics.get("CURRENT_BASELINE", {}).get(sub)
        m_rd = separation_metrics.get("REDUCED_BLOCK", {}).get(sub)
        if m_bl is not None and m_rd is not None:
            if abs(m_bl["separation_delta"] - m_rd["separation_delta"]) > THRESHOLD_MEANINGFUL:
                space_dependent = True
                break
        # Compare BASELINE vs R4 for shared subspaces
        m_r4 = separation_metrics.get("R4_FLAT", {}).get(sub)
        if m_bl is not None and m_r4 is not None:
            if abs(m_bl["separation_delta"] - m_r4["separation_delta"]) > THRESHOLD_MEANINGFUL:
                space_dependent = True
                break

    if not space_dependent:
        lines += [
            "**No.** The separation_delta for every shared subspace is identical",
            "across CURRENT_BASELINE, REDUCED_BLOCK, and R4_FLAT (where available).",
            "",
            "The ambient space (R^4, R^12, R^16) does not affect the geometric",
            "separation signal even when multiple subspaces participate in the comparison.",
            "",
            "This extends the SPACE_INVARIANT result from the prior probes:",
            "- geometry_space_type_probe_v1:     H3_ONLY under frozen conditions → SPACE_INVARIANT",
            "- geometry_space_dynamics_probe_v1: H3_ONLY under dynamics → SPACE_INVARIANT_UNDER_DYNAMICS",
            "- THIS PROBE: multi-subspace comparison → space also does not matter",
        ]
    else:
        lines += [
            "**Yes.** The separation_delta for at least one subspace differs meaningfully",
            "across ambient spaces.",
            "",
            "Ambient space affects multi-subspace comparison.",
        ]

    lines += [
        "",
        "---",
        "",
        "## 7. Explicit Answers to Critical Questions",
        "",
        "**Q1: Is the signal truly confined to H3?**  ",
    ]

    if classification == "H3_ONLY_SIGNAL":
        lines.append("A1: YES. H3_ONLY dominates with maximum separation_delta across all spaces.")
    elif classification == "MULTI_SUBSPACE_SIGNAL":
        lines.append("A1: NO. Other subspaces also carry usable signal.")
    else:
        lines.append("A1: AMBIGUOUS. See result table.")

    lines += [
        "",
        "**Q2: Do other harmonic subspaces carry usable signal?**  ",
    ]
    if classification == "H3_ONLY_SIGNAL":
        lines.append("A2: NOT COMPARABLY. H1 and H2 have substantially lower separation_delta.")
    else:
        lines.append("A2: YES. See table for quantitative comparison.")

    lines += [
        "",
        "**Q3: Does combining subspaces change behavior?**  ",
    ]
    # Compare H3_ONLY vs H2_PLUS_H3 and FULL_B3
    h3_d_bl  = (separation_metrics.get("CURRENT_BASELINE", {}).get("H3_ONLY") or {}).get("separation_delta", float("nan"))
    h23_d_bl = (separation_metrics.get("CURRENT_BASELINE", {}).get("H2_PLUS_H3") or {}).get("separation_delta", float("nan"))
    fb3_d_bl = (separation_metrics.get("CURRENT_BASELINE", {}).get("FULL_B3") or {}).get("separation_delta", float("nan"))

    if math.isfinite(h3_d_bl) and math.isfinite(h23_d_bl):
        diff_h23 = h3_d_bl - h23_d_bl
        if abs(diff_h23) < THRESHOLD_MEANINGFUL:
            lines.append(
                f"A3: Combining H2+H3 yields delta={h23_d_bl:.6f}, essentially the same as H3_ONLY (delta={h3_d_bl:.6f})."
            )
        elif diff_h23 > 0:
            lines.append(
                f"A3: Combining H2+H3 reduces separation (delta={h23_d_bl:.6f} vs H3_ONLY={h3_d_bl:.6f})."
            )
        else:
            lines.append(
                f"A3: Combining H2+H3 increases separation (delta={h23_d_bl:.6f} vs H3_ONLY={h3_d_bl:.6f})."
            )
    else:
        lines.append("A3: Not available (data missing).")

    lines += [
        "",
        "**Q4: Does ambient space matter once multiple subspaces are used?**  ",
    ]
    if not space_dependent:
        lines.append(
            "A4: NO. All shared subspace comparisons yield identical deltas across R^4, R^12, R^16."
        )
    else:
        lines.append(
            "A4: YES. separation_delta differs by ambient space for at least one subspace."
        )

    lines += [
        "",
        "---",
        "",
        "## 8. Classification",
        "",
        f"**CLASSIFICATION: {classification}**",
        "",
        "| Classification | Criterion | Observed? |",
        "|:---------------|:----------|:----------|",
        f"| H3_ONLY_SIGNAL | Only H3 produces separation; all others fail | {classification == 'H3_ONLY_SIGNAL'} |",
        f"| MULTI_SUBSPACE_SIGNAL | Other subspaces also produce signal | {classification == 'MULTI_SUBSPACE_SIGNAL'} |",
        f"| SPACE_DEPENDENT_SUBSPACE_BEHAVIOR | Subspace effectiveness differs by space | {classification == 'SPACE_DEPENDENT_SUBSPACE_BEHAVIOR'} |",
        f"| DEGENERATE | Signal collapses or inconsistent | {classification == 'DEGENERATE'} |",
        "",
        "---",
        "",
        "## 9. Locked Variables Confirmation",
        "",
        "| Variable | Value | Changed? |",
        "|:---------|:------|:---------|",
        "| Basis | apply_anchor_two_i (2i frame) | NO |",
        "| Operator | cos (cosine similarity) | NO |",
        "| Normalization | unit-norm per subspace vector | NO |",
        "| eps_high | 1.0 (frozen) | NO |",
        "| Training | None | NO |",
        "| New constants | None | NO |",
        "| Radial structure | Unchanged | NO |",
        "| D sweeps | None | NO |",
        "| phi / heuristic ratios | None | NO |",
        "",
        "---",
        "",
        f"GEOMETRY SUBSPACE PARTICIPATION RESULT: {classification}",
    ]

    MD_OUT.write_text("\n".join(lines))


# ════════════════════════════════════════════════════════════════════════════
# MAIN
# ════════════════════════════════════════════════════════════════════════════

def main() -> None:
    print("=" * 72)
    print("GEOMETRY SUBSPACE PARTICIPATION PROBE v1")
    print("=" * 72)
    print("CONTRACT:   prompt_contract_v4.md — binding")
    print("DEP TABLE:  prime_transport_geometry_dependency_order_probe_v1.md — binding")
    print()
    print("CONTRACT + DEPENDENCY TABLE LOADED")
    print()
    print("Relevant constraints:")
    print("  - Layer 1 (ambient space) is UNRESOLVED — tested here with multi-subspace")
    print("  - Basis locked: apply_anchor_two_i (2i frame) — NOT changed")
    print("  - No training, no optimization, no D sweeps")
    print("  - No new constants (no phi)")
    print("  - Operator: cos only; same normalization; no reweighting")
    print()
    print(f"N_SAMPLES: {N_SAMPLES}  ({N_PER_CLASS} per class × {VOCAB} classes)")
    print(f"Spaces:    {SPACE_NAMES}")
    print(f"Subspaces: {SUBSPACE_NAMES}")
    print()

    # Build class labels
    class_labels = np.repeat(np.arange(VOCAB), N_PER_CLASS)   # [N_SAMPLES]

    # ── Build states for all three spaces ────────────────────────────────────
    baseline_states, baseline_protos = build_current_baseline(class_labels, BLOCKS_A)
    reduced_states,  reduced_protos  = build_reduced_block(class_labels, BLOCKS_A)
    r4_states,       r4_protos       = build_r4_flat(class_labels, BLOCKS_A)

    space_data = {
        "CURRENT_BASELINE": (baseline_states, baseline_protos),
        "REDUCED_BLOCK":    (reduced_states,  reduced_protos),
        "R4_FLAT":          (r4_states,        r4_protos),
    }

    print(f"State shapes: BASELINE={baseline_states.shape}, "
          f"REDUCED={reduced_states.shape}, R4={r4_states.shape}")
    print()

    # ── MANDATORY STEP 1: Validity Check ─────────────────────────────────────
    print("STEP 1: Validity check ...")
    validity_pass, validity_detail = check_subspace_validity(
        baseline_states, reduced_states, r4_states, N=N_SAMPLES,
    )
    print(f"  → {'PASS' if validity_pass else 'FAIL — INVALID_SUBSPACE_SETUP'}")
    for k, v in validity_detail.items():
        if k != "subspace_norms":
            print(f"     {k}: {v}")
    if not validity_pass:
        print()
        print("STOP: INVALID_SUBSPACE_SETUP")
        print("Subspace validity check failed. Results would be invalid.")
        raise SystemExit(1)
    print()

    # ── MANDATORY STEP 2: Separation metrics ────────────────────────────────
    print("STEP 2: Computing separation metrics ...")
    separation_metrics: Dict[str, Dict[str, Optional[Dict]]] = {
        s: {} for s in SPACE_NAMES
    }

    for space_name in SPACE_NAMES:
        states, protos = space_data[space_name]
        for sub_name in SUBSPACE_NAMES:
            sub_dims = SUBSPACE_DEFS[sub_name][space_name]
            if sub_dims is None:
                separation_metrics[space_name][sub_name] = None
                print(f"  [{space_name}] [{sub_name}] → UNAVAILABLE (subspace not in this space)")
                continue

            m = compute_separation_metrics(states, protos, sub_dims, class_labels)
            separation_metrics[space_name][sub_name] = m
            print(
                f"  [{space_name}] [{sub_name:12s}] "
                f"matched={m['mean_matched_cos']:.6f}  "
                f"mismatched={m['mean_mismatched_cos']:.6f}  "
                f"delta={m['separation_delta']:.6f}"
            )

    print()

    # ── MANDATORY STEP 3: Classification ────────────────────────────────────
    print("STEP 3: Classification ...")
    classification = classify_result(separation_metrics)
    print(f"  → CLASSIFICATION: {classification}")
    print()

    # ── Assemble result dict ─────────────────────────────────────────────────
    result = {
        "probe":             "geometry_subspace_participation_probe_v1",
        "branch":            "geometry_subspace_participation_probe_v1",
        "contract":          "prompt_contract_v4.md",
        "dependency_table":  "prime_transport_geometry_dependency_order_probe_v1.md",
        "prior_probes":      [
            "geometry_space_type_probe_v1 → SPACE_INVARIANT_SIGNAL",
            "geometry_space_dynamics_probe_v1 → SPACE_INVARIANT_UNDER_DYNAMICS",
        ],
        "n_samples":         N_SAMPLES,
        "n_per_class":       N_PER_CLASS,
        "vocab":             VOCAB,
        "operator":          "cos (cosine similarity)",
        "normalization":     "unit-norm per subspace vector; no reweighting",
        "basis":             "apply_anchor_two_i (2i frame)",
        "eps_high":          1.0,
        "no_training":       True,
        "threshold_meaningful_delta": THRESHOLD_MEANINGFUL,
        "space_candidates":  {
            "CURRENT_BASELINE": {
                "description": "R^16 hybrid: 12 angular (6 harmonic pairs) + 4 magnitude",
                "dims": 16,
            },
            "REDUCED_BLOCK": {
                "description": "R^12: 12 angular dims only (4 magnitude dims removed from R^16)",
                "dims": 12,
            },
            "R4_FLAT": {
                "description": "R^4: h2+h3 pairs of dominant block3 only",
                "dims": 4,
                "note": "H1 subspace not available",
            },
        },
        "subspace_types":    {
            sub_name: {
                "description": {
                    "H3_ONLY":    "h=3 of block3; θ=π·k/2; 4 classes at 90° → orthogonal (control)",
                    "H2_ONLY":    "h=2 of block3; θ=π·k/3; 4 classes at 60° → not orthogonal",
                    "H1_ONLY":    "h=1 of block3; θ=π·k/6; 4 classes at 30° → high overlap",
                    "H2_PLUS_H3": "h=2 and h=3 of block3 concatenated (4D)",
                    "H1_PLUS_H3": "h=1 and h=3 of block3 concatenated (4D)",
                    "FULL_B3":    "h=1, h=2, h=3 of block3 concatenated (6D)",
                }[sub_name],
                "dims_baseline": SUBSPACE_DEFS[sub_name]["CURRENT_BASELINE"],
                "dims_r4":       SUBSPACE_DEFS[sub_name]["R4_FLAT"],
            }
            for sub_name in SUBSPACE_NAMES
        },
        "separation_metrics":  separation_metrics,
        "classification":      classification,
        "validity_pass":       validity_pass,
        "validity_detail":     validity_detail,
    }

    # ── Write outputs ─────────────────────────────────────────────────────────
    JSON_OUT.write_text(json.dumps(result, indent=2))
    print(f"JSON → {JSON_OUT}")

    write_markdown(result, separation_metrics, validity_pass, validity_detail, classification)
    print(f"MD   → {MD_OUT}")

    # ── Summary table ─────────────────────────────────────────────────────────
    print()
    print("RESULT TABLE")
    print("-" * 72)
    header = f"{'Space':<22} {'Subspace':<14} {'matched':>10} {'mismatched':>12} {'delta':>10}"
    print(header)
    print("-" * 72)
    for space in SPACE_NAMES:
        for sub in SUBSPACE_NAMES:
            m = separation_metrics[space][sub]
            if m is None:
                print(f"{'  ' + space:<22} {sub:<14} {'N/A':>10} {'N/A':>12} {'N/A':>10}")
            else:
                print(
                    f"{'  ' + space:<22} {sub:<14} "
                    f"{m['mean_matched_cos']:>10.6f} "
                    f"{m['mean_mismatched_cos']:>12.6f} "
                    f"{m['separation_delta']:>10.6f}"
                )
    print("-" * 72)
    print()
    print(f"CLASSIFICATION: {classification}")
    print()
    print(f"GEOMETRY SUBSPACE PARTICIPATION RESULT: {classification}")
    print()
    print("Task complete. Do NOT continue autonomously.")


if __name__ == "__main__":
    main()
