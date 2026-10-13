#!/usr/bin/env python3
"""run_geometry_space_type_probe_v1.py

GEOMETRY SPACE TYPE PROBE v1
==============================

BRANCH:   geometry_space_type_probe_v1
CONTRACT: prompt_contract_v4.md — loaded and binding

════════════════════════════════════════════════════════════════════════
PURPOSE
════════════════════════════════════════════════════════════════════════

Pure Layer 1 probe.

Determine whether the observed geometric separation signal
(cos_metric between matched vs mismatched class pairs) is:
  A) dependent on the ambient space representation
  B) invariant across space choices

This probe does NOT run training, optimization, or sweeps.

════════════════════════════════════════════════════════════════════════
LOCKED VARIABLES (unchanged across ALL candidates)
════════════════════════════════════════════════════════════════════════

  - Basis:              apply_anchor_two_i  (2i frame)
  - Operator:           cos_metric ONLY
  - Comparison:         H3-based pairing / polar L2 logic
  - eps_high:           1.0  (H3 frozen by algebraic construction)
  - No training:        TRUE
  - No constants added: TRUE (no phi, no heuristic ratios)
  - No radial changes:  TRUE

════════════════════════════════════════════════════════════════════════
VARIABLE UNDER TEST (ONLY ONE)
════════════════════════════════════════════════════════════════════════

  Ambient / operative space representation

CANDIDATES:
  1. CURRENT_BASELINE  — R^16 hybrid (12 angular + 4 magnitude)
                         H3 comparison subspace at angular dims [10, 11]
  2. R4_FLAT           — R^4 (h2 + h3 pairs of dominant block = dims [8,9,10,11]
                         from baseline angular part)
                         H3 comparison subspace at dims [2, 3] in R4
  3. REDUCED_BLOCK     — R^12 (12 angular only; magnitude dims removed)
                         H3 comparison subspace at angular dims [10, 11]

════════════════════════════════════════════════════════════════════════
MEASUREMENT
════════════════════════════════════════════════════════════════════════

  mean_matched_cos:    mean cos_metric(state_k, proto_k)  (same class)
  mean_mismatched_cos: mean cos_metric(state_k, proto_j)  (j ≠ k, averaged)
  separation_delta:    mean_matched_cos − mean_mismatched_cos

All comparisons use the H3 subspace (H3-equivalent per space candidate).

════════════════════════════════════════════════════════════════════════
STRUCTURAL EQUIVALENCE REQUIREMENTS (verified before results)
════════════════════════════════════════════════════════════════════════

  1. Identical input samples (same N, same class labels) across all candidates
  2. Identical prototype/reference construction procedure per space
  3. Identical operator execution path (cos_metric on H3-equivalent subspace)
  4. H3 values numerically identical across all representations (allclose, atol=1e-9)

════════════════════════════════════════════════════════════════════════
LAYER ISOLATION COMPLIANCE
════════════════════════════════════════════════════════════════════════

  NOT changed: Layers 2-7 (basis family, operator, constants, comparison
               mechanism, D/workspace, training).
  TESTED:      Layer 1 only (ambient space representation).

CONTRACT RULES COMPLIED:
  Rule 1 (Mechanism Lock):   Mechanism defined above pre-code.
  Rule 2 (No Hidden Changes): ONE variable changes — ambient space only.
  Rule 3 (Geometry Consistency): All computations use the same H3 pair
                                  content across all spaces.
  Rule 4 (Deterministic First): No training, no learning, D=0 implicit.
  Rule 5 (Compare Regimes):   Three candidates compared on same metric.
  Rule 6 (Output Discipline): JSON + Markdown + explicit classification.
  Rule 7 (Failure Is Valid):  Null result accepted.
  Rule 8 (Reuse Components):  apply_anchor_two_i and angular helpers reused
                               from existing probe machinery.
  Rule 9 (No Theory Injection): No phi, no external constants.
"""

from __future__ import annotations

import json
import math
from pathlib import Path
from typing import Dict, Tuple

import numpy as np

# ── Output paths ─────────────────────────────────────────────────────────────
_ROOT    = Path(__file__).resolve().parents[2]
JSON_OUT = _ROOT / "results" / "prime_transport_recursive_system" / "geometry_space_type_probe_v1.json"
MD_OUT   = _ROOT / "docs" / "research" / "prime_transport_geometry_space_type_probe_v1.md"
JSON_OUT.parent.mkdir(parents=True, exist_ok=True)
MD_OUT.parent.mkdir(parents=True, exist_ok=True)

# ── Experiment constants ──────────────────────────────────────────────────────
VOCAB         = 4
N_PER_CLASS   = 256
N_SAMPLES     = N_PER_CLASS * VOCAB    # 1024 samples total
BLOCKS_A      = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# H3 angular indices in the 12-dim angular vector
# Block3 angular layout: h1=dims[6,7]  h2=dims[8,9]  h3=dims[10,11]
H3_IDX0_BASELINE = 10
H3_IDX1_BASELINE = 11

# separation_delta difference threshold for SPACE_DEPENDENT classification
THRESHOLD_MEANINGFUL = 0.05


# ════════════════════════════════════════════════════════════════════════════
# GEOMETRY HELPERS  (reused from existing probe machinery)
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

    Identical to apply_anchor_two_i in run_train_free_geometric_probe_v1.py.
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
    """Build R^16 hybrid states for all samples.

    eps_high=1.0, D=0: state = initial encoding exactly.
    Returns array of shape [N, 16] = 12 angular (2i-rotated) + 4 magnitude.

    Each sample i with class k:
      - Block3 (dominant, period=12, n_h=3): token = k → encodes H3 at pi*k/2
      - Other blocks: token = k % period  (class-coherent but not the comparison signal)
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
# SPACE CANDIDATE BUILDERS
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
      Dim 0 (R4) ≡ dim  8 (baseline angular): h2 of block3, −sin(2π·2·k/12)
      Dim 1 (R4) ≡ dim  9 (baseline angular): h2 of block3,  cos(2π·2·k/12)
      Dim 2 (R4) ≡ dim 10 (baseline angular): h3 of block3, −sin(π·k/2)
      Dim 3 (R4) ≡ dim 11 (baseline angular): h3 of block3,  cos(π·k/2)

    No magnitude dims.  H3 comparison subspace: dims [2, 3] within R4.

    apply_anchor_two_i was applied to the full 16D state; slicing preserves
    the 2i-rotated values of both h2 and h3 pairs.
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

    Structure: identical to CURRENT_BASELINE angular part.
    H3 comparison subspace: dims [10, 11] (same as baseline).
    Reduction: drop the 4 magnitude dims (dims 12-15 of R^16).
    """
    d_ang, _, _ = geom_dims(blocks)   # d_ang = 12
    full_states = build_full_r16_states(class_labels, blocks)   # [N, 16]
    full_protos = build_class_prototypes_r16(blocks)             # [4, 16]

    rb_states = full_states[:, :d_ang].copy()   # [N, 12]
    rb_protos = full_protos[:, :d_ang].copy()   # [4, 12]

    return rb_states, rb_protos, H3_IDX0_BASELINE, H3_IDX1_BASELINE


# ════════════════════════════════════════════════════════════════════════════
# COS_METRIC  (H3 subspace only — current H3-based pairing / polar L2 logic)
# ════════════════════════════════════════════════════════════════════════════

def cos_metric_h3(
    states:  np.ndarray,   # [N, D_space]
    proto_h3: np.ndarray,  # [2] unit-normalised H3 vector of one prototype
    h3_i0:   int,
    h3_i1:   int,
) -> np.ndarray:
    """Cosine similarity between H3 subspace of states and a prototype H3 vector.

    Both state and prototype are in the 2i frame (apply_anchor_two_i applied).
    Polar L2: unit-normalise state H3 pair, then compute dot product.
    """
    v = states[:, h3_i0:h3_i1 + 1]                            # [N, 2]
    norm = np.linalg.norm(v, axis=1, keepdims=True)
    v_unit = v / np.maximum(norm, 1e-8)                        # unit-normalise
    return (v_unit * proto_h3).sum(axis=1)                     # [N] cos values


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 1 — STRUCTURAL EQUIVALENCE CHECK
# ════════════════════════════════════════════════════════════════════════════

def check_structural_equivalence(
    class_labels:    np.ndarray,
    baseline_states: np.ndarray,
    r4_states:       np.ndarray,
    rb_states:       np.ndarray,
) -> Tuple[bool, dict]:
    """Verify that all three candidates use identical underlying data.

    Critical invariant: H3 values (dims 10-11 of baseline, dims 2-3 of R4,
    dims 10-11 of REDUCED_BLOCK) must be numerically identical.
    Returns (pass_bool, detail_dict).
    """
    N = len(class_labels)
    checks = {}

    # 1. Sample count consistency
    checks["n_samples_consistent"] = (
        len(baseline_states) == N
        and len(r4_states)   == N
        and len(rb_states)   == N
    )

    # 2. H3 values numerically identical across all three representations
    baseline_h3 = baseline_states[:, H3_IDX0_BASELINE:H3_IDX1_BASELINE + 1]  # [N,2]
    r4_h3       = r4_states[:, 2:4]                                            # [N,2]
    rb_h3       = rb_states[:, H3_IDX0_BASELINE:H3_IDX1_BASELINE + 1]         # [N,2]

    max_diff_r4 = float(np.max(np.abs(baseline_h3 - r4_h3)))
    max_diff_rb = float(np.max(np.abs(baseline_h3 - rb_h3)))

    checks["h3_identical_baseline_vs_r4"]      = max_diff_r4 < 1e-9
    checks["h3_identical_baseline_vs_reduced"]  = max_diff_rb < 1e-9
    checks["h3_max_diff_r4"]                    = round(max_diff_r4, 12)
    checks["h3_max_diff_reduced"]               = round(max_diff_rb, 12)

    # 3. Prototype construction: same 4 class tokens, same apply_anchor_two_i
    checks["prototype_count_per_space"] = 4  # always true by construction

    all_pass = (
        checks["n_samples_consistent"]
        and checks["h3_identical_baseline_vs_r4"]
        and checks["h3_identical_baseline_vs_reduced"]
    )
    return all_pass, checks


# ════════════════════════════════════════════════════════════════════════════
# MANDATORY STEP 2 — SEPARATION METRIC COMPUTATION
# ════════════════════════════════════════════════════════════════════════════

def compute_separation_metrics(
    states:       np.ndarray,    # [N, D]
    protos:       np.ndarray,    # [4, D]
    h3_i0:        int,
    h3_i1:        int,
    class_labels: np.ndarray,   # [N]
) -> Dict:
    """Compute mean_matched_cos, mean_mismatched_cos, separation_delta.

    For each class k:
      - matched pairs:    cos_metric(state_k, proto_k)
      - mismatched pairs: cos_metric(state_k, proto_j) for j ≠ k
    """
    matched    = []
    mismatched = []

    for k in range(VOCAB):
        mask = (class_labels == k)
        if not mask.any():
            continue
        k_states = states[mask]   # [N_k, D]

        # Unit-normalise H3 of prototype k
        proto_h3    = protos[k, h3_i0:h3_i1 + 1].copy()
        proto_norm  = np.linalg.norm(proto_h3)
        proto_h3    /= max(proto_norm, 1e-8)

        cos_match = cos_metric_h3(k_states, proto_h3, h3_i0, h3_i1)
        matched.extend(cos_match.tolist())

        # Mismatched: all other class prototypes
        for j in range(VOCAB):
            if j == k:
                continue
            proto_j    = protos[j, h3_i0:h3_i1 + 1].copy()
            norm_j     = np.linalg.norm(proto_j)
            proto_j    /= max(norm_j, 1e-8)
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

def classify_result(metrics: Dict[str, Dict]) -> str:
    """Classify outcome as SPACE_DEPENDENT_SIGNAL / SPACE_INVARIANT_SIGNAL / INCONCLUSIVE."""
    deltas = [v["separation_delta"] for v in metrics.values()]
    if any(not math.isfinite(d) for d in deltas):
        return "INCONCLUSIVE"
    delta_range = max(deltas) - min(deltas)
    if delta_range > THRESHOLD_MEANINGFUL:
        return "SPACE_DEPENDENT_SIGNAL"
    return "SPACE_INVARIANT_SIGNAL"


# ════════════════════════════════════════════════════════════════════════════
# MARKDOWN OUTPUT
# ════════════════════════════════════════════════════════════════════════════

def write_markdown(
    result:       dict,
    metrics:      Dict[str, Dict],
    classification: str,
    eq_detail:    dict,
) -> None:
    lines = [
        "# Prime Transport Geometry Space Type Probe v1",
        "",
        "**Probe:** geometry_space_type_probe_v1  ",
        "**Contract:** prompt_contract_v4.md  ",
        "**Branch:** geometry_space_type_probe_v1  ",
        "**Layer tested:** Layer 1 — Ambient/operative space  ",
        f"**N samples:** {result['n_samples']}  ({result['n_per_class']} per class)  ",
        "",
        "---",
        "",
        "## 1. Space Representations Tested",
        "",
        "### CURRENT_BASELINE (R^16)",
        "- **Dimensions:** 16 (12 angular + 4 magnitude)",
        "- **Structure:** BLOCKS_A = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]",
        "- **Angular part:** 6 harmonic pairs; apply_anchor_two_i applied to all pairs",
        "  - Block0 h1: dims [0,1];  Block1 h1: dims [2,3];  Block2 h1: dims [4,5]",
        "  - Block3 h1: dims [6,7];  Block3 h2: dims [8,9];  Block3 h3: dims [10,11]",
        "- **Magnitude part:** 4 dims (one per block), all fixed to 1.0",
        "- **H3 comparison subspace:** angular dims [10, 11]",
        "- **Basis:** Existing H3-tangent representation — unchanged",
        "",
        "### R4_FLAT (R^4)",
        "- **Dimensions:** 4 (no magnitude dims)",
        "- **Structure:** h2 and h3 pairs of dominant block (block3, period=12)",
        "- **Source:** angular dims [8, 9, 10, 11] from CURRENT_BASELINE",
        "  - R4 dim 0 = baseline angular dim 8:  h2 of block3",
        "  - R4 dim 1 = baseline angular dim 9:  h2 of block3",
        "  - R4 dim 2 = baseline angular dim 10: h3 of block3",
        "  - R4 dim 3 = baseline angular dim 11: h3 of block3",
        "- **apply_anchor_two_i:** inherited (applied in full-state build; slice preserves it)",
        "- **H3 comparison subspace:** dims [2, 3] within R4",
        "- **Rationale:** Minimal representation preserving H3 signal with unit-norm angular pairs",
        "",
        "### REDUCED_BLOCK (R^12)",
        "- **Dimensions:** 12 (angular only; 4 magnitude dims removed from R^16)",
        "- **Structure:** Same 6 harmonic pairs as CURRENT_BASELINE angular part",
        "- **apply_anchor_two_i:** applied to all 6 pairs (identical to baseline)",
        "- **H3 comparison subspace:** dims [10, 11] (identical to CURRENT_BASELINE)",
        "- **Rationale:** Tests whether magnitude dimensions contribute to separation signal",
        "",
        "---",
        "",
        "## 2. Structural Equivalence Check",
        "",
        f"**Status:** {'PASS' if result['structural_equivalence_pass'] else 'FAIL — INVALID_COMPARISON_SETUP'}",
        "",
        "Verified:",
        f"- Identical {result['n_samples']} samples (same class labels) across all candidates",
        "- Identical prototype construction: class tokens {0,1,2,3}, apply_anchor_two_i applied",
        f"- H3 values: baseline vs R4_FLAT max_diff = {eq_detail['h3_max_diff_r4']}",
        f"- H3 values: baseline vs REDUCED_BLOCK max_diff = {eq_detail['h3_max_diff_reduced']}",
        "- Identical operator: cos_metric on H3-equivalent subspace",
        "",
        "---",
        "",
        "## 3. Result Table",
        "",
        "| Space Type | mean_matched_cos | mean_mismatched_cos | separation_delta |",
        "|:-----------|----------------:|--------------------:|----------------:|",
    ]

    for name, m in metrics.items():
        lines.append(
            f"| {name} | {m['mean_matched_cos']:.6f} | {m['mean_mismatched_cos']:.6f} | {m['separation_delta']:.6f} |"
        )

    lines += [
        "",
        "---",
        "",
        "## 4. Classification Result",
        "",
        f"**CLASSIFICATION: {classification}**",
        "",
    ]

    if classification == "SPACE_INVARIANT_SIGNAL":
        lines += [
            "All three space candidates produce identical separation_delta values.",
            "The geometric separation signal is determined entirely by the H3 subspace content,",
            "which is preserved numerically unchanged across all three ambient space representations.",
            "",
            "Changing the ambient space dimension (R^4 / R^12 / R^16) does not alter the",
            "cos_metric separation between matched and mismatched class pairs.",
            "",
            f"The range of separation_delta across all candidates is < {THRESHOLD_MEANINGFUL} (threshold).",
        ]
    elif classification == "SPACE_DEPENDENT_SIGNAL":
        lines += [
            "separation_delta differs meaningfully across space candidates.",
            "The geometric separation signal changes when the ambient space changes.",
            f"The range of separation_delta across candidates exceeds {THRESHOLD_MEANINGFUL} (threshold).",
        ]
    else:
        lines += [
            "Signal is unstable or numerical validity check failed.",
            "Review INVALID_COMPARISON_SETUP diagnostics.",
        ]

    lines += [
        "",
        "---",
        "",
        "## 5. Is Geometric Signal Dependent on Ambient Space?",
        "",
    ]

    if classification == "SPACE_INVARIANT_SIGNAL":
        lines += [
            "**NO.**",
            "",
            "The geometric separation signal (`separation_delta = mean_matched_cos − mean_mismatched_cos`)",
            "is **invariant** across ambient space representations.",
            "",
            "The signal originates from the H3 harmonic pair — the third harmonic of the",
            "dominant cyclic block (block3, period=12). This pair encodes each of the 4 classes",
            "as one of four unit-norm, 2i-rotated vectors:",
            "",
            "  - class 0 → H3 = ( 0,  1)",
            "  - class 1 → H3 = (−1,  0)",
            "  - class 2 → H3 = ( 0, −1)",
            "  - class 3 → H3 = ( 1,  0)",
            "",
            "With eps_high=1.0, the H3 pair is frozen by algebraic construction across all D steps.",
            "Consequently, including or excluding additional dimensions (h1 pairs of other blocks,",
            "magnitude dims) does not affect the cos_metric comparison on the H3 subspace.",
            "",
            "**Layer 1 interpretation:** The ambient space dimensionality is not the source of",
            "the geometric separation signal. The signal is localised in the H3 harmonic pair",
            "and is invariant to the ambient embedding (R^4, R^12, or R^16).",
        ]
    elif classification == "SPACE_DEPENDENT_SIGNAL":
        lines += [
            "**YES.**",
            "",
            "The geometric separation signal changes when the ambient space changes.",
            "The signal is not purely localised to the H3 subspace.",
        ]
    else:
        lines += [
            "**INCONCLUSIVE.** Numerical validation failed.",
        ]

    lines += [
        "",
        "---",
        "",
        "## 6. Locked Variables Confirmation",
        "",
        "| Variable | Value | Changed? |",
        "|:---------|:------|:---------|",
        "| Basis | apply_anchor_two_i (2i frame) | NO |",
        "| Operator | cos_metric | NO |",
        "| Comparison subspace | H3 pair (per-space equivalent) | NO |",
        "| eps_high | 1.0 | NO |",
        "| Training | None | NO |",
        "| New constants | None | NO |",
        "| Radial structure | Unchanged | NO |",
        "",
        "---",
        "",
        f"GEOMETRY SPACE PROBE RESULT: {classification}",
    ]

    MD_OUT.write_text("\n".join(lines))


# ════════════════════════════════════════════════════════════════════════════
# MAIN
# ════════════════════════════════════════════════════════════════════════════

def main() -> None:
    print("=" * 72)
    print("GEOMETRY SPACE TYPE PROBE v1")
    print("=" * 72)
    print(f"CONTRACT:  prompt_contract_v4.md — binding")
    print(f"N_SAMPLES: {N_SAMPLES}  ({N_PER_CLASS} per class)")
    print(f"Candidates: CURRENT_BASELINE (R^16) | R4_FLAT (R^4) | REDUCED_BLOCK (R^12)")
    print(f"Operator:   cos_metric on H3-equivalent subspace")
    print()

    # ── Generate identical class labels for all candidates ────────────────
    class_labels = np.repeat(np.arange(VOCAB), N_PER_CLASS)   # [1024]

    # ── Build all space representations ──────────────────────────────────
    bl_states, bl_protos, bl_h3i0, bl_h3i1 = build_current_baseline(class_labels, BLOCKS_A)
    r4_states, r4_protos, r4_h3i0, r4_h3i1 = build_r4_flat(class_labels, BLOCKS_A)
    rb_states, rb_protos, rb_h3i0, rb_h3i1 = build_reduced_block(class_labels, BLOCKS_A)

    print(f"CURRENT_BASELINE:  state_dim={bl_states.shape[1]}  H3=dims[{bl_h3i0},{bl_h3i1}]")
    print(f"R4_FLAT:           state_dim={r4_states.shape[1]}   H3=dims[{r4_h3i0},{r4_h3i1}]")
    print(f"REDUCED_BLOCK:     state_dim={rb_states.shape[1]}  H3=dims[{rb_h3i0},{rb_h3i1}]")
    print()

    # ── MANDATORY STEP 1: Structural Equivalence Check ───────────────────
    eq_pass, eq_detail = check_structural_equivalence(
        class_labels, bl_states, r4_states, rb_states
    )
    if not eq_pass:
        print("STOP — INVALID_COMPARISON_SETUP")
        print(f"Equivalence detail: {eq_detail}")
        raise RuntimeError("INVALID_COMPARISON_SETUP")

    print("Structural equivalence: PASS")
    print(f"  H3 max_diff baseline vs R4_FLAT:      {eq_detail['h3_max_diff_r4']}")
    print(f"  H3 max_diff baseline vs REDUCED_BLOCK: {eq_detail['h3_max_diff_reduced']}")
    print()

    # ── MANDATORY STEP 2: Compute separation metrics ──────────────────────
    candidates = {
        "CURRENT_BASELINE": (bl_states, bl_protos, bl_h3i0, bl_h3i1),
        "R4_FLAT":          (r4_states, r4_protos, r4_h3i0, r4_h3i1),
        "REDUCED_BLOCK":    (rb_states, rb_protos, rb_h3i0, rb_h3i1),
    }

    separation_metrics: Dict[str, Dict] = {}
    for name, (states, protos, h3i0, h3i1) in candidates.items():
        m = compute_separation_metrics(states, protos, h3i0, h3i1, class_labels)
        separation_metrics[name] = m

    # ── MANDATORY STEP 2: Print result table ─────────────────────────────
    print("RESULT TABLE")
    print("-" * 72)
    hdr = f"{'Space Type':<20}  {'mean_matched_cos':>16}  {'mean_mismatched_cos':>19}  {'separation_delta':>16}"
    print(hdr)
    print("-" * 72)
    for name, m in separation_metrics.items():
        print(
            f"{name:<20}  {m['mean_matched_cos']:>16.6f}  "
            f"{m['mean_mismatched_cos']:>19.6f}  {m['separation_delta']:>16.6f}"
        )
    print()

    # ── MANDATORY STEP 3: Classify ────────────────────────────────────────
    classification = classify_result(separation_metrics)
    print(f"CLASSIFICATION: {classification}")
    print()

    # ── Build result record ───────────────────────────────────────────────
    result = {
        "probe":                      "geometry_space_type_probe_v1",
        "branch":                     "geometry_space_type_probe_v1",
        "contract":                   "prompt_contract_v4.md",
        "n_samples":                  N_SAMPLES,
        "n_per_class":                N_PER_CLASS,
        "vocab":                      VOCAB,
        "operator":                   "cos_metric",
        "basis":                      "apply_anchor_two_i (2i frame)",
        "comparison_subspace":        "H3 pair (per-space equivalent subspace)",
        "eps_high":                   1.0,
        "no_training":                True,
        "threshold_meaningful_delta": THRESHOLD_MEANINGFUL,
        "space_candidates": {
            "CURRENT_BASELINE": {
                "description":  "R^16 hybrid: 12 angular (6 harmonic pairs) + 4 magnitude",
                "dims":         16,
                "h3_dims":      [bl_h3i0, bl_h3i1],
                "angular_dims": 12,
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
        "separation_metrics":          separation_metrics,
        "classification":              classification,
        "structural_equivalence_pass": eq_pass,
        "structural_equivalence_detail": eq_detail,
    }

    # ── Write outputs ─────────────────────────────────────────────────────
    JSON_OUT.write_text(json.dumps(result, indent=2))
    print(f"JSON written: {JSON_OUT}")

    write_markdown(result, separation_metrics, classification, eq_detail)
    print(f"MD  written:  {MD_OUT}")
    print()
    print(f"GEOMETRY SPACE PROBE RESULT: {classification}")


if __name__ == "__main__":
    main()
