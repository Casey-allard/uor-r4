"""run_state_construction_encoding_probe_v1.py

STATE CONSTRUCTION ENCODING PROBE v1
======================================

BRANCH: state_construction_encoding_probe_v1

CANONICAL FILES:
  - results/prime_transport_recursive_system/train_free_geometric_probe_v1.csv
  - results/prime_transport_recursive_system/carrier_vs_computation_probe_v1.csv
  - tools/prime_transport/run_train_free_geometric_probe_v1.py
  - tools/prime_transport/run_carrier_vs_computation_probe_v1.py

STATE CONSTRUCTION LOCK (MANDATORY PRE-CODE):

1. EXACT FUNCTIONS THAT CONSTRUCT tau_init:
   a. convert_onehot_to_angular_multi(tau0_oh, blocks):
      For block 3 (s=9, e=21, m=12, n_h=3), k_idx=argmax(onehot[9:21]) in [0..11].
        h1 at ai=6,7:   cos(2*pi*1*k/12), sin(2*pi*1*k/12)  period-12
        h2 at ai=8,9:   cos(2*pi*2*k/12), sin(2*pi*2*k/12)  period-6
        h3 at ai=10,11: cos(2*pi*3*k/12), sin(2*pi*3*k/12)  period-4

   b. apply_anchor_two_i(tau0_ang, n_pairs):
      Rotates every (cos,sin) pair -> (-sin, cos).
      H3 after anchor: [10]=−sin(pi*k/2), [11]=cos(pi*k/2)
      H2 after anchor: [8]=−sin(pi*k/3),  [9]=cos(pi*k/3)
      H1 after anchor: [6]=−sin(pi*k/6),  [7]=cos(pi*k/6) (replaced each step)

   c. tau0_hyb = cat([tau0_ang, ones(n_blocks)])

2. H2/H3 CARRIER COMPONENTS:
   H2 indices 8,9: (-sin(pi*k/3), cos(pi*k/3))  encodes k%6  period-6
   H3 indices 10,11: (-sin(pi*k/2), cos(pi*k/2)) encodes k%4  period-4
   Both frozen by eps_high=1.0 (h>=2 harmonics: blend=1.0*prev).

3. PREVIOUSLY SOLVED TRIVIALLY:
   original_s42 (k%4 + offset=0): H3 readout acc=1.0
   shift1_s42   (k%4 + offset=1): H3 readout acc=1.0
   H2 target (k%6): H2 readout acc=1.0
   CRT target (k%12): joint (H2,H3) CRT readout acc=1.0

4. EXPLICIT ENCODING: construction formula directly writes k%4 into H3 by
   using h=3 in period-12 block. Chain: k->pi*k/2->H3=(-sin,cos)->k%4.
   EMERGENT MAPPING: label not computable from any single frozen harmonic.

PROBE CASES:
   CASE A: formula audit — trace exact dependency chain from k to H3
   CASE B: ablate h=3 harmonic in construction — measure accuracy collapse
   CASE C: permuted-per-k target (not k%4) — H3 fails, CRT succeeds
   CASE D: soft one-hot interpolation — H3 shifts per explicit formula
"""

from __future__ import annotations

import csv
import math
import time
from pathlib import Path
from typing import List, Tuple

import numpy as np
import torch

# =====================================================================
# Paths
# =====================================================================
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "state_construction_encoding_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_state_construction_encoding_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# =====================================================================
# Task constants — identical to canonical probes
# =====================================================================
GLOBAL_SEED  = 42
VOCAB        = 4
BLOCKS_A     = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
CYC_S        = 9
CYC_E        = 21
N_EVAL       = 2048
D_EVAL       = 20
PARTITION    = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
PART_OFFSET  = 0

H1_IDX0, H1_IDX1 = 6, 7
H2_IDX0, H2_IDX1 = 8, 9
H3_IDX0, H3_IDX1 = 10, 11

ALPHA_STEPS = [round(a * 0.1, 1) for a in range(11)]

# CRT lookup (k%4, k%6) -> k%12
_CRT_LUT = torch.full((4, 6), -1, dtype=torch.long)
for _k in range(12):
    _CRT_LUT[_k % 4, _k % 6] = _k
assert (_CRT_LUT >= 0).sum().item() == 12

# =====================================================================
# Geometry helpers — verbatim from canonical probes
# =====================================================================
def geom_dims(blocks):
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*onehot.shape[:-1], d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            angle            = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def convert_soft_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    """Soft version: uses weighted k_idx instead of argmax."""
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*onehot.shape[:-1], d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        idx_range = torch.arange(m, dtype=torch.float)
        k_idx     = (onehot[..., s:e] * idx_range).sum(dim=-1)
        for harm in range(1, n_h + 1):
            angle            = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def convert_onehot_to_angular_h3_ablated(onehot: torch.Tensor, blocks) -> torch.Tensor:
    """CASE B: identical to canonical but sets h=3 term to zero for block 3."""
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*onehot.shape[:-1], d_ang)
    ai    = 0
    for b_idx, (s, e, m, n_h) in enumerate(blocks):
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            # Block 3 (b_idx==3), h=3: zero out instead of encoding
            if b_idx == 3 and harm == 3:
                ai += 2
                continue
            angle            = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def apply_anchor_two_i(tau0_ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    out = tau0_ang.clone()
    for p in range(n_pairs):
        c = tau0_ang[:, 2 * p].clone()
        s = tau0_ang[:, 2 * p + 1].clone()
        out[:, 2 * p]     = -s
        out[:, 2 * p + 1] =  c
    return out


def apply_split_transport(base, tau_prev, blocks, eps_high):
    ang_parts: List[torch.Tensor] = []
    mags:      List[torch.Tensor] = []
    ai = 0
    for _, _, _, n_h in blocks:
        fund = base[:, ai:ai + 2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        ang_parts.append(fund / mag)
        for h_idx in range(1, n_h):
            new_pair  = base[:, ai + h_idx * 2 : ai + h_idx * 2 + 2] / mag
            prev_pair = tau_prev[:, ai + h_idx * 2 : ai + h_idx * 2 + 2]
            ang_parts.append((1.0 - eps_high) * new_pair + eps_high * prev_pair)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], n_blocks)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


def prepare_tables_h3_ablated(TN_oh, tau0_oh, TR, pool_ids, blocks):
    """CASE B: build tau0_hyb with h=3 zeroed in construction."""
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_h3_ablated(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], n_blocks)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


def build_class_table(tau0_oh, cyc_start, cyc_end, partition):
    k_idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut   = torch.tensor(partition, dtype=torch.long)
    return lut[k_idx]


def get_cycle_position(tau0_oh, cyc_start, cyc_end):
    return tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)


# =====================================================================
# Readouts
# =====================================================================
def h3_readout(tau, partition_offset):
    angle  = torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
    k_mod4 = torch.round(-angle / (math.pi / 2)).long() % VOCAB
    return (k_mod4 + partition_offset) % VOCAB


def h2_readout_mod6(tau):
    angle  = torch.atan2(tau[:, H2_IDX0], tau[:, H2_IDX1])
    return torch.round(-angle / (math.pi / 3)).long() % 6


def crt_readout_mod12(tau):
    a = torch.round(-torch.atan2(tau[:, H3_IDX0], tau[:, H3_IDX1])
                    / (math.pi / 2)).long() % 4
    b = torch.round(-torch.atan2(tau[:, H2_IDX0], tau[:, H2_IDX1])
                    / (math.pi / 3)).long() % 6
    return _CRT_LUT[a, b]


def run_trajectory(sids, tau0_hyb, TN_ang, TR, D, d_ang, blocks):
    B         = sids.shape[0]
    tau_prev  = tau0_hyb[sids]
    sids_loop = sids.clone()
    for _ in range(D):
        tn       = TN_ang[sids_loop]
        cur_dir  = tau_prev[:, :d_ang]
        ang_sim  = torch.einsum("bi,bji->bj", cur_dir, tn)
        best_op  = ang_sim.argmax(dim=1)
        best_ang = tn.gather(1, best_op.view(B, 1, 1).expand(B, 1, d_ang)).squeeze(1)
        tau_prev  = apply_split_transport(best_ang, tau_prev, blocks, 1.0)
        sids_loop = TR[sids_loop].gather(1, best_op.unsqueeze(1)).squeeze(1)
    return tau_prev


# =====================================================================
# CASE A — Direct Construction Audit
# =====================================================================
def run_case_a(tau0_hyb, tau0_oh):
    """
    Analytically verify H3 = explicit function of k.
    Formula: H3 = (-sin(pi*k/2), cos(pi*k/2)) at indices [10,11].
    Verify vs actual tau0_hyb values. Measure H3 readout at D=0.
    Dependency chain: k -> pi*k/2 -> H3 -> k%4.
    """
    t0 = time.perf_counter()

    k_all = tau0_oh[:, CYC_S:CYC_E].argmax(dim=-1)

    formula_errs = []
    for k in range(12):
        mask = (k_all == k)
        if mask.sum() == 0:
            continue
        angle_h3      = math.pi * k / 2.0
        h3_cos_expect = -math.sin(angle_h3)
        h3_sin_expect =  math.cos(angle_h3)
        h3_actual     = tau0_hyb[mask, H3_IDX0:H3_IDX1+1]
        expected      = torch.tensor([h3_cos_expect, h3_sin_expect])
        err = float((h3_actual - expected).abs().max().item())
        formula_errs.append(err)

    max_formula_err = max(formula_errs)

    # H3 readout at D=0 (no trajectory)
    pred_k4 = h3_readout(tau0_hyb, PART_OFFSET)
    true_k4 = k_all % 4
    acc_d0  = float((pred_k4 == true_k4).float().mean().item())

    wall = time.perf_counter() - t0

    dep_chain = (
        "k=argmax(onehot[9:21]) "
        "-> angle_h3=2*pi*3*k/12=pi*k/2 "
        "[written by convert_onehot_to_angular_multi: block3 h=3 m=12] "
        "-> apply_anchor_two_i: (cos,sin)->(-sin,cos) "
        "-> H3=(-sin(pi*k/2), cos(pi*k/2)) "
        "-> atan2(H3)=-pi*k/2 "
        "-> k%4=round(-atan2/(pi/2))%4"
    )

    return {
        "case":            "A",
        "variant":         "original_s42",
        "D":               0,
        "n_samples":       int(tau0_oh.shape[0]),
        "acc":             round(acc_d0, 6),
        "acc_h3_on_perm":  "N/A",
        "h3_survives":     True,
        "max_formula_err": round(max_formula_err, 12),
        "direct_readable": True,
        "dependency_chain": dep_chain,
        "runtime_s":       round(wall, 6),
        "note": (
            "CASE A: formula audit. "
            f"Max formula error vs actual tau_init: {max_formula_err:.2e}. "
            f"H3 readout at D=0 acc={acc_d0:.4f}. "
            "h=3 harmonic of period-12 block = period-4. "
            "Target k%4 directly written into H3 by construction formula."
        ),
    }


# =====================================================================
# CASE B — Minimal Construction Ablation
# =====================================================================
def run_case_b(TN_oh, tau0_oh, TR, pool_ids):
    """
    Zero h=3 harmonic inside convert_onehot_to_angular_multi for block 3.
    Everything else unchanged (anchor, transport, readout).
    Measure accuracy collapse.
    """
    t0 = time.perf_counter()

    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)

    TN_ang_abl, TR_abl, tau0_hyb_abl, _ = prepare_tables_h3_ablated(
        TN_oh, tau0_oh, TR, pool_ids, BLOCKS_A)

    # H3 in ablated init should be zero
    h3_max_in_abl = float(tau0_hyb_abl[:, H3_IDX0:H3_IDX1+1].abs().max().item())

    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 11)
    idx  = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng)
    sids = pool_ids[idx]

    tau_f  = run_trajectory(sids, tau0_hyb_abl, TN_ang_abl, TR_abl,
                             D_EVAL, d_ang, BLOCKS_A)
    classes_h3 = build_class_table(tau0_oh, CYC_S, CYC_E, PARTITION)
    target     = classes_h3[sids]
    pred       = h3_readout(tau_f, PART_OFFSET)
    acc        = float((pred == target).float().mean().item())

    wall = time.perf_counter() - t0

    return {
        "case":            "B",
        "variant":         "original_s42",
        "D":               D_EVAL,
        "n_samples":       N_EVAL,
        "acc":             round(acc, 6),
        "acc_h3_on_perm":  "N/A",
        "h3_survives":     h3_max_in_abl > 1e-8,
        "max_formula_err": round(h3_max_in_abl, 12),
        "direct_readable": False,
        "dependency_chain": "H3 term zeroed at construction; h=3 not written",
        "runtime_s":       round(wall, 6),
        "note": (
            "CASE B: h=3 harmonic zeroed in convert_onehot_to_angular_multi. "
            f"H3 max abs in ablated tau_init={h3_max_in_abl:.2e}. "
            f"acc={acc:.4f} after ablation (expected ~0.25 baseline). "
            "Transport and readout unchanged. "
            "Carrier disappears when construction term is removed."
        ),
    }


# =====================================================================
# CASE C — Permuted Target Test
# =====================================================================
def run_case_c(TN_oh, tau0_oh, TR, pool_ids):
    """
    Keep same state construction. Use permuted-per-k target (not k%4).
    H3 readout should fail (~0.25). CRT readout should succeed (~1.0).
    """
    t0 = time.perf_counter()

    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)
    TN_ang, TR2, tau0_hyb, pool2 = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, BLOCKS_A)

    # Build permuted label assignment over k in [0..11]
    # Each of 12 k-positions gets a label in [0..3]; each label used 3 times.
    # Assignment must NOT be k%4 for at least some k values.
    rng_np = np.random.default_rng(seed=77)
    perm_labels = []
    for _ in range(3):
        order = list(range(4))
        rng_np.shuffle(order)
        perm_labels.extend(order)
    # perm_labels[k] = label for cycle-position k, for k in [0..11]

    k4_agreement = sum(1 for kk in range(12) if perm_labels[kk] == kk % 4)
    perm_lut = torch.tensor(perm_labels, dtype=torch.long)

    k_all        = tau0_oh[:, CYC_S:CYC_E].argmax(dim=-1)
    classes_perm = perm_lut[k_all]

    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 13)
    idx  = torch.randint(pool_ids.shape[0], (N_EVAL,), generator=rng)
    sids = pool_ids[idx]

    tau_f = run_trajectory(sids, tau0_hyb, TN_ang, TR2, D_EVAL, d_ang, BLOCKS_A)

    target_perm  = classes_perm[sids]
    pred_h3      = h3_readout(tau_f, PART_OFFSET)
    pred_k12     = crt_readout_mod12(tau_f)
    pred_crt_map = perm_lut[pred_k12.clamp(0, 11)]

    acc_h3_perm  = float((pred_h3 == target_perm).float().mean().item())
    acc_crt_perm = float((pred_crt_map == target_perm).float().mean().item())

    wall = time.perf_counter() - t0

    return {
        "case":            "C",
        "variant":         "original_s42",
        "D":               D_EVAL,
        "n_samples":       N_EVAL,
        "acc":             round(acc_crt_perm, 6),
        "acc_h3_on_perm":  round(acc_h3_perm, 6),
        "h3_survives":     True,
        "max_formula_err": float("nan"),
        "direct_readable": True,
        "dependency_chain": (
            "H3 readout (k%4) fails on perm target; "
            "CRT(H2,H3)->k%12->perm_lut[k] succeeds. "
            "Full k encoded in (H2,H3), any per-k target readable."
        ),
        "runtime_s":       round(wall, 6),
        "note": (
            "CASE C: permuted per-k target (not aligned to k%4). "
            f"perm_labels={perm_labels}. "
            f"k%4 agreement in perm: {k4_agreement}/12. "
            f"H3-only acc on perm target={acc_h3_perm:.4f} (expected ~0.25). "
            f"CRT+perm_lut acc={acc_crt_perm:.4f} (expected ~1.0). "
            "Construction encodes full k; any per-k target trivially readable."
        ),
    }


# =====================================================================
# CASE D — Input Perturbation Test
# =====================================================================
def run_case_d(tau0_oh, pool_ids):
    """
    Soft one-hot interpolation: blend adjacent states k and k+1.
    Vary alpha in [0.0..1.0]. Measure H3 angle vs formula prediction.
    Explicit encoding prediction: H3_angle = -pi * ((1-alpha)*k + alpha*(k+1)) / 2.
    """
    t0 = time.perf_counter()

    _, n_pairs, _ = geom_dims(BLOCKS_A)
    cycle_len     = CYC_E - CYC_S   # 12
    k_all         = tau0_oh[:, CYC_S:CYC_E].argmax(dim=-1)

    alpha_errors = []
    per_alpha_detail = []

    for alpha in ALPHA_STEPS:
        per_k_errs = []
        for k in range(cycle_len):
            k_next = (k + 1) % cycle_len
            mask_k     = (k_all == k)
            mask_knext = (k_all == k_next)
            if mask_k.sum() == 0 or mask_knext.sum() == 0:
                continue

            oh_k     = tau0_oh[mask_k][0:1].float()
            oh_knext = tau0_oh[mask_knext][0:1].float()

            # Blend only cycle block; other blocks stay at oh_k
            soft_oh = oh_k.clone()
            soft_oh[0, CYC_S:CYC_E] = (
                (1.0 - alpha) * oh_k[0, CYC_S:CYC_E]
                + alpha       * oh_knext[0, CYC_S:CYC_E]
            )

            tau_ang_soft = convert_soft_onehot_to_angular_multi(soft_oh, BLOCKS_A)
            tau_ang_soft = apply_anchor_two_i(tau_ang_soft, n_pairs)

            actual_angle = float(torch.atan2(
                tau_ang_soft[0, H3_IDX0], tau_ang_soft[0, H3_IDX1]).item())

            # Formula: k_soft = (1-alpha)*k + alpha*(k_next)
            k_soft  = (1.0 - alpha) * k + alpha * k_next
            formula = -(math.pi * k_soft / 2.0)
            # Normalize to [-pi, pi]
            formula = (formula + math.pi) % (2 * math.pi) - math.pi

            err = abs(actual_angle - formula)
            err = min(err, 2 * math.pi - err)
            per_k_errs.append(err)

        mean_err = float(np.mean(per_k_errs)) if per_k_errs else float("nan")
        alpha_errors.append(mean_err)
        per_alpha_detail.append(f"alpha={alpha:.1f}:err={mean_err:.2e}")

    max_err  = max(alpha_errors)
    mean_err = float(np.mean(alpha_errors))

    wall = time.perf_counter() - t0

    return {
        "case":            "D",
        "variant":         "original_s42",
        "D":               0,
        "n_samples":       cycle_len * len(ALPHA_STEPS),
        "acc":             "N/A",
        "acc_h3_on_perm":  "N/A",
        "h3_survives":     True,
        "max_formula_err": round(max_err, 12),
        "direct_readable": True,
        "dependency_chain": (
            "soft k_idx=(1-alpha)*k+alpha*(k+1) "
            "-> H3_angle=-pi*k_soft/2 [linear in alpha]. "
            "H3 shifts continuously per explicit encoding formula."
        ),
        "runtime_s":       round(wall, 6),
        "note": (
            "CASE D: soft one-hot interpolation. "
            f"Mean H3-angle error vs formula: {mean_err:.2e} rad. "
            f"Max error across alpha steps: {max_err:.2e} rad. "
            f"Per-alpha: [{'; '.join(per_alpha_detail)}]. "
            "H3 shifts linearly with alpha per explicit formula. "
            "Confirms: construction writes label directly."
        ),
    }


# =====================================================================
# Markdown writer
# =====================================================================
def write_markdown(rows):
    case_a = next(r for r in rows if r["case"] == "A")
    case_b = next(r for r in rows if r["case"] == "B")
    case_c = next(r for r in rows if r["case"] == "C")
    case_d = next(r for r in rows if r["case"] == "D")

    # Determine conclusion
    a_direct    = case_a["direct_readable"] and case_a["acc"] > 0.999
    b_collapsed = case_b["acc"] < 0.30
    c_h3_fail   = float(case_c["acc_h3_on_perm"]) < 0.35
    c_crt_ok    = case_c["acc"] > 0.95
    d_low_err   = case_d["max_formula_err"] < 0.01

    if a_direct and b_collapsed and c_h3_fail and c_crt_ok and d_low_err:
        conclusion = "EXPLICIT ENCODING"
    elif not a_direct and not b_collapsed:
        conclusion = "EMERGENT MAPPING"
    else:
        conclusion = "MIXED / INCONCLUSIVE"

    lines = [
        "# Prime Transport: State Construction Encoding Probe v1",
        "",
        "**Branch**: state_construction_encoding_probe_v1",
        "",
        "## State Construction Lock",
        "",
        "### 1. Functions that construct tau_init",
        "",
        "**`convert_onehot_to_angular_multi(tau0_oh, blocks)`**:",
        "For block 3 (s=9, e=21, m=12, n_h=3), `k_idx = argmax(onehot[9:21])` in [0..11]:",
        "",
        "| harmonic | indices | formula | period |",
        "|----------|---------|---------|--------|",
        "| h=1 | 6,7 | cos(2πk/12), sin(2πk/12) | 12 |",
        "| h=2 | 8,9 | cos(πk/3), sin(πk/3) | 6 |",
        "| h=3 | 10,11 | cos(πk/2), sin(πk/2) | **4** |",
        "",
        "**`apply_anchor_two_i`**: rotates (cos,sin) → (−sin,cos). After rotation:",
        "- H3 at [10,11]: `(−sin(πk/2), cos(πk/2))` — encodes k%4",
        "- H2 at [8,9]:  `(−sin(πk/3), cos(πk/3))` — encodes k%6",
        "- H1 at [6,7]:  `(−sin(πk/6), cos(πk/6))` — **replaced** each transport step",
        "",
        "**`tau0_hyb = cat([tau0_ang, ones(n_blocks)])`**",
        "",
        "### 2. H2/H3 carrier components",
        "",
        "Both frozen by `eps_high=1.0` (h≥2 harmonics: blend = 1.0×prev):",
        "- **H2** (indices 8,9): encodes k%6, period 6",
        "- **H3** (indices 10,11): encodes k%4, period 4",
        "",
        "### 3. Previously solved trivially",
        "",
        "| Target | Readout | Accuracy |",
        "|--------|---------|----------|",
        "| original_s42 (k%4+0) | H3 phase | 1.0000 |",
        "| shift1_s42 (k%4+1) | H3 phase | 1.0000 |",
        "| k%6 | H2 phase | 1.0000 |",
        "| k%12 | CRT(H2,H3) | 1.0000 |",
        "",
        "### 4. Hypothesis definitions",
        "",
        "**Explicit encoding**: `h=3` in a period-12 block directly produces a period-4 component.",
        "Chain: `k → πk/2 → H3=(−sin(πk/2),cos(πk/2)) → atan2 → k%4`.",
        "Target is precomputed at initialization. No transport needed.",
        "",
        "**Emergent mapping**: label not computable from any single frozen harmonic;",
        "requires dynamics or non-trivial multi-component interaction.",
        "",
        "---",
        "",
        "## 4-Case Results",
        "",
        "| Case | Description | acc | H3 survives | direct_readable | max_formula_err | runtime_s |",
        "|------|-------------|-----|-------------|-----------------|-----------------|-----------|",
        f"| A | Formula audit (D=0) | {case_a['acc']:.4f} | {case_a['h3_survives']} | {case_a['direct_readable']} | {case_a['max_formula_err']:.2e} | {case_a['runtime_s']:.3f}s |",
        f"| B | H3 ablated in construction | {case_b['acc']:.4f} | {case_b['h3_survives']} | {case_b['direct_readable']} | {case_b['max_formula_err']:.2e} | {case_b['runtime_s']:.3f}s |",
        f"| C (H3 only) | Permuted target, H3 readout | {float(case_c['acc_h3_on_perm']):.4f} | {case_c['h3_survives']} | — | — | — |",
        f"| C (CRT) | Permuted target, CRT readout | {case_c['acc']:.4f} | {case_c['h3_survives']} | {case_c['direct_readable']} | — | {case_c['runtime_s']:.3f}s |",
        f"| D | Soft-onehot perturbation | N/A | {case_d['h3_survives']} | {case_d['direct_readable']} | {case_d['max_formula_err']:.2e} | {case_d['runtime_s']:.3f}s |",
        "",
        "### Case A — Dependency chain",
        "",
        f"```",
        case_a["dependency_chain"],
        "```",
        "",
        f"**Formula match error (max over all k in [0..11])**: `{case_a['max_formula_err']:.2e}`",
        f"(machine precision — H3 coordinates match formula exactly)",
        "",
        "**H3 readout at D=0** (no trajectory at all): acc = `{:.4f}`".format(case_a["acc"]),
        "The answer is present before the first transport step.",
        "",
        "### Case B — Ablation",
        "",
        case_b["note"],
        "",
        "Accuracy without H3 carrier: `{:.4f}` ≈ random baseline (1/VOCAB={:.2f})".format(
            case_b["acc"], 1.0/VOCAB),
        "",
        "### Case C — Permuted target",
        "",
        case_c["note"],
        "",
        "- H3 readout on permuted target: `{:.4f}` (fails — wrong period)".format(
            float(case_c["acc_h3_on_perm"])),
        "- CRT readout (k%12 → perm_lut): `{:.4f}` (succeeds — full k encoded)".format(
            case_c["acc"]),
        "",
        "Shows the construction encodes **full k** (12-state identity), not just k%4.",
        "Any per-k label function is trivially readable from (H2,H3).",
        "",
        "### Case D — Input perturbation",
        "",
        case_d["note"],
        "",
        f"Max H3-angle error vs formula across all alpha: `{case_d['max_formula_err']:.2e}` rad",
        "(consistent with floating-point precision — H3 shifts exactly per formula)",
        "",
        "---",
        "",
        "## Conclusion",
        "",
        f"**STATE CONSTRUCTION STATUS: {conclusion}**",
        "",
        "---",
        "",
        "## Honesty Section",
        "",
        "### What is explicitly written at initialization",
        "",
        "- **k%4** is written directly into H3 (indices 10,11) by the h=3 harmonic of the",
        "  period-12 block. The formula `cos(2π·3·k/12) = cos(πk/2)` has period 4 by",
        "  construction. This is not accidental: choosing h=3 in a 12-cycle produces",
        "  a period-4 component, which matches VOCAB=4.",
        "- **k%6** is written into H2 by the h=2 harmonic (period 6).",
        "- **k%12 = k** is recoverable from H2+H3 via CRT, meaning the full cycle",
        "  identity is encoded at initialization.",
        "",
        "### What is NOT written",
        "",
        "- H1 (indices 6,7) is NOT preserved — replaced every transport step.",
        "- Nothing about the transport dynamics contributes to correctness for k%4.",
        "- No learned weights are used.",
        "",
        "### Is this clever encoding or genuinely revealing mapping?",
        "",
        "**This is clever encoding, not a genuinely revealing mapping.**",
        "",
        "The mechanism is: the state construction formula includes a harmonic whose",
        "period matches the task period. The target (k%4) is a direct mathematical",
        "consequence of how the h=3 harmonic of a 12-cycle was chosen.",
        "",
        "The readout does not discover anything about the input — it extracts a",
        "pre-written label. The 'geometry' is a convenient coordinate system for",
        "label storage, not a representation that makes the label emergent.",
        "",
        "There is no evidence for emergent mapping: all four cases confirm the label",
        "is written at construction time and extracted trivially.",
        "",
        f"CASE A: formula error = {case_a['max_formula_err']:.2e} (machine precision match).",
        f"CASE B: acc collapses from 1.0 to {case_b['acc']:.4f} when construction term removed.",
        f"CASE C: H3-only acc = {float(case_c['acc_h3_on_perm']):.4f} on non-period-4 target (fails); CRT acc = {case_c['acc']:.4f} (succeeds via full-k encoding).",
        f"CASE D: H3-angle error vs formula = {case_d['max_formula_err']:.2e} rad (explicit linear shift).",
    ]

    MD_OUT.write_text("\n".join(lines))
    print(f"  Markdown written: {MD_OUT}")


# =====================================================================
# CSV writer
# =====================================================================
CSV_FIELDS = [
    "case", "variant", "D", "n_samples", "acc", "acc_h3_on_perm",
    "h3_survives", "max_formula_err", "direct_readable",
    "dependency_chain", "runtime_s", "note",
]


def write_csv(rows):
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        w.writeheader()
        for row in rows:
            w.writerow({k: row.get(k, "") for k in CSV_FIELDS})
    print(f"  CSV written: {CSV_OUT}")


# =====================================================================
# Main
# =====================================================================
def main():
    print("=" * 70)
    print("STATE CONSTRUCTION ENCODING PROBE v1")
    print(f"  D_EVAL={D_EVAL}, N_EVAL={N_EVAL}")
    print("=" * 70)

    print(f"  Loading state cache: {CACHE_PATH}")
    cache = torch.load(str(CACHE_PATH), map_location="cpu", weights_only=True)
    TN_oh, tau0_oh, TR, pool_ids = (
        cache["TN_oh"], cache["tau0_oh"], cache["TR"], cache["pool_ids"])

    print(f"  States: {tau0_oh.shape[0]}, Pool: {pool_ids.shape[0]}, "
          f"Vocab: {TN_oh.shape[-1]}")

    TN_ang, TR2, tau0_hyb, pool2 = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, BLOCKS_A)

    rows = []

    print("\n--- CASE A: Direct Construction Audit ---")
    r = run_case_a(tau0_hyb, tau0_oh)
    rows.append(r)
    print(f"  acc={r['acc']:.4f}  max_formula_err={r['max_formula_err']:.2e}  "
          f"direct_readable={r['direct_readable']}")

    print("\n--- CASE B: H3 Construction Ablation ---")
    r = run_case_b(TN_oh, tau0_oh, TR, pool_ids)
    rows.append(r)
    print(f"  acc={r['acc']:.4f}  h3_survives={r['h3_survives']}  "
          f"h3_max_in_abl={r['max_formula_err']:.2e}")

    print("\n--- CASE C: Permuted Target Test ---")
    r = run_case_c(TN_oh, tau0_oh, TR, pool_ids)
    rows.append(r)
    print(f"  acc_h3_on_perm={r['acc_h3_on_perm']:.4f}  "
          f"acc_crt_on_perm={r['acc']:.4f}")

    print("\n--- CASE D: Input Perturbation Test ---")
    r = run_case_d(tau0_oh, pool_ids)
    rows.append(r)
    print(f"  max_formula_err={r['max_formula_err']:.2e} rad  "
          f"n_samples={r['n_samples']}")

    print("\n--- Writing outputs ---")
    write_csv(rows)
    write_markdown(rows)

    print("\n" + "=" * 70)
    # Quick conclusion
    b_acc = rows[1]["acc"]
    c_h3  = float(rows[2]["acc_h3_on_perm"])
    c_crt = rows[2]["acc"]
    d_err = rows[3]["max_formula_err"]
    print("SUMMARY:")
    print(f"  A: formula match err={rows[0]['max_formula_err']:.2e}, D=0 acc={rows[0]['acc']:.4f}")
    print(f"  B: acc after ablation={b_acc:.4f}")
    print(f"  C: H3 acc on perm={c_h3:.4f}, CRT acc on perm={c_crt:.4f}")
    print(f"  D: H3 angle err vs formula={d_err:.2e}")
    print("=" * 70)


if __name__ == "__main__":
    main()
