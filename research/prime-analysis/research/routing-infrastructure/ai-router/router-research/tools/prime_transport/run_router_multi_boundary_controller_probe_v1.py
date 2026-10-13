#!/usr/bin/env python3
"""run_router_multi_boundary_controller_probe_v1.py

MULTI-BOUNDARY CONTROLLER PROBE v1
====================================

PRE-CODE GEOMETRY LOCK (verified before implementation):

1. Task A geometry (canonical):
   - Cyclic state space: dominant 12-state cyclic component (one-hot positions
     9–21, BLOCKS_A entry (9,21,12,3))
   - Interleaved 4-class partition: [0,1,2,3,0,1,2,3,0,1,2,3]
     class 0={0,4,8}, class 1={1,5,9}, class 2={2,6,10}, class 3={3,7,11}
     NOT contiguous blocks — every class member is 4 cyclic positions apart
   - Projective controller: proj_k = sin_k/(1+cos_k+0.1), clipped ±10
   - Split transport: fundamental harmonic free; k≥2 frozen (eps_high=1.0)
   - two_i anchoring: tau0 rotated +π/2 per pair: (c,s)→(−s,c)

2. Partitions are INTERLEAVED (NOT contiguous blocks). Both probe variants
   (original_s42, shift1_s42) preserve the underlying cycle structure —
   only the class label assignment rotates by +1.

3. Boundary definition: every consecutive pair of cyclic states belongs to
   different class labels (12 boundary transitions total for a 12-state
   cycle with the interleaved 4-class partition). There is NO single cut.
   4 distinct boundary types exist (0→1, 1→2, 2→3, 3→0), each recurring
   3 times around the cycle.

CANONICAL DATA SOURCES:
  CANONICAL:
    router_locked_stack_failure_isolation_probe_v1.csv
      — provides original_s42 and shift1_s42 Task A baselines (probe1_taskA)
      — original_s42: D24=0.9941, D32=0.9653  Δ=−0.0288
      — shift1_s42:   D24=0.9570, D32=0.9829  Δ=+0.0259
  CONTRAST (do not use as primary source of truth):
    prime_transport_router_controller_geometry_probe_v1.csv
    prime_transport_router_start_agnostic_root_probe_v1.csv

HYPOTHESIS:
  A controller that aggregates multiple boundary-relative projective
  contributions will be less sensitive to the orientation of any single
  class boundary relative to the projective reference frame.
  → Expected outcome: smaller |Δ(D32−D24)| gap between original_s42
    and shift1_s42 compared to the baseline projective controller.

FIXED COMPONENTS (unchanged from canonical locked stack):
  - Split transport: eps_high=1.0
  - State representation: harmonic sin/cos (two harmonics for period-12
    block, one each for the others)
  - Start anchor: two_i_orient (rotate each pair by +π/2)
  - Geometry: BLOCKS_A = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]

CONTROLLER VARIANTS:
  ctrl_1  (baseline projective):
    proj_k = sin_k / (1 + cos_k + ε)
    Controller input dim: D_EMB + n_pairs + n_blocks

  ctrl_2  (multi-boundary average):
    For N_BOUNDARIES=4 reference orientations derived from the dominant
    cyclic component (12-state cycle, step_size=π/6):
      φ_j = j × π/6  for j = 0,1,2,3  → {0, π/6, π/3, π/2}
    Note: NOT evenly spaced over 2π — this avoids the cancellation that
    makes mean_j[tan((θ−φ_j)/2)] identically zero for uniformly-spaced φ_j.
      c_rot_j_k = c_k·cos(φ_j) + s_k·sin(φ_j)
      s_rot_j_k = s_k·cos(φ_j) − c_k·sin(φ_j)
      p_j_k     = s_rot_j_k / (1 + c_rot_j_k + ε)   [clipped ±PROJ_CLIP]
      ctrl_k    = mean_j(p_j_k)
    Controller input dim: D_EMB + n_pairs + n_blocks  (same as ctrl_1)

  ctrl_3  (multi-boundary inverse-distance weighted):
    Same boundary rotations as ctrl_2, but weighted by inverse angular
    distance to boundary j:
      d_j_k  = 1 − c_rot_j_k               (proxy for angular distance)
      w_j_k  = 1 / (d_j_k + INV_DIST_EPS)  (closer boundary → higher weight)
      ctrl_k = Σ_j w_j_k·p_j_k / Σ_j w_j_k
    Controller input dim: D_EMB + n_pairs + n_blocks  (same as ctrl_1)

TASK VARIANTS:
  original_s42: PARTITION_ORIGINAL = [0,1,2,3,0,1,2,3,0,1,2,3], seed=42
  shift1_s42:   PARTITION_SHIFT1   = [1,2,3,0,1,2,3,0,1,2,3,0], seed=42

HORIZONS: D ∈ {24, 32}

RUN COUNT: 3 controllers × 2 partitions × 2 horizons = 12 runs (NO MORE)

SUCCESS CRITERION:
  Not accuracy maximization.
  Test whether multi-boundary controller reduces orientation sensitivity:
  → smaller |Δ_orig − Δ_shift| (gap between original and shift1 deltas)
  → stable or improved D32
  → no decodability collapse (decodability_final must remain ≥ 0.95)

DELIVERABLES:
  PY  : tools/prime_transport/run_router_multi_boundary_controller_probe_v1.py
  CSV : results/prime_transport_recursive_system/
          router_multi_boundary_controller_probe_v1.csv
  MD  : docs/research/
          prime_transport_multi_boundary_controller_probe_v1.md
"""
from __future__ import annotations

import csv
import datetime
import math
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler

# ═══════════════════════════════════════════════════════════════════════
# CPU/thread fix — MUST happen before any model work
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR = Path(__file__).parent
try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st()
except Exception as _te:
    print(f"  [thread_policy] fallback — {_te}")
    import os
    _n = min(8, os.cpu_count() or 1)
    torch.set_num_threads(_n)
    print(f"  [thread_policy] {_n} thread(s) — fallback")

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "router_multi_boundary_controller_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_multi_boundary_controller_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Canonical reference values (hard-coded from canonical CSV)
# Source: router_locked_stack_failure_isolation_probe_v1.csv
#         probe=probe1_taskA, ctrl=projective (locked stack)
# ═══════════════════════════════════════════════════════════════════════
CANON_ORIG_D24   = 0.9941   # original_s42 D=24
CANON_ORIG_D32   = 0.9653   # original_s42 D=32  (Δ = −0.0288)
CANON_SHIFT_D24  = 0.9570   # shift1_s42   D=24
CANON_SHIFT_D32  = 0.9829   # shift1_s42   D=32  (Δ = +0.0259)

# ═══════════════════════════════════════════════════════════════════════
# Hyperparameters — identical to locked stack failure isolation probe
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED  = 42
D_HIDDEN     = 32
BATCH_SIZE   = 512
VOCAB        = 4
D_EMB        = 4
N_OPS        = 6
LR           = 0.02
TEMP_START   = 2.0
TEMP_END     = 0.1
CLIP_GRAD    = 1.0
EVAL_EVERY   = 500
N_EVAL       = 2048
SOLVE_ACC    = 0.999
INIT_SCALE   = 0.05
MAX_STEPS    = 3_500
N_BATCHES    = N_EVAL // BATCH_SIZE
NEG_INF      = -1e9
N_PROBE      = 4096

# Projective parameters (locked from canonical stack)
PROJ_EPS     = 0.1
PROJ_CLIP    = 10.0

# Multi-boundary parameters
N_BOUNDARIES   = 4
# Boundary reference angles: j × (state_spacing of dominant cycle)
# Dominant cycle = 12 states → state_spacing = 2π/12 = π/6
# Using {0, π/6, π/3, π/2}: covers the 4 sequential step offsets within
# one class period, grounding boundaries in the actual cyclic state spacing.
# NOT evenly spaced over 2π — avoids the symmetric cancellation that makes
# mean_j[tan((θ−φ_j)/2)] identically zero for uniformly spaced φ_j over [0,2π).
BOUNDARY_ANGLES = [j * math.pi / 6.0 for j in range(N_BOUNDARIES)]
INV_DIST_EPS   = 0.1   # denominator guard for inverse-distance weights

# ═══════════════════════════════════════════════════════════════════════
# Geometry — BLOCKS_A only (fixed for all configurations)
# ═══════════════════════════════════════════════════════════════════════
BLOCKS_A = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# Dominant cyclic component span for Task A (period-12)
TASK_A_CYC_S, TASK_A_CYC_E = 9, 21

# ═══════════════════════════════════════════════════════════════════════
# Task A partition variants (geometry-native lookup tables)
#
# Interleaved 4-class partition: class c assigned to all cyclic positions
# k where k mod 4 == c. Every consecutive pair is a class boundary.
# ═══════════════════════════════════════════════════════════════════════
# original: [0,1,2,3,0,1,2,3,0,1,2,3]  class 0={0,4,8} class 1={1,5,9} …
PARTITION_ORIGINAL = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
# shift1:   [1,2,3,0,1,2,3,0,1,2,3,0]  same interleaved structure, +1 label rotation
PARTITION_SHIFT1   = [1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0]

# ═══════════════════════════════════════════════════════════════════════
# Controller variants
# (ctrl_name, ctrl_mode)
# ctrl_mode: "projective" | "multi_avg" | "multi_invdist"
# ═══════════════════════════════════════════════════════════════════════
CONTROLLER_VARIANTS = [
    ("ctrl_1_projective",  "projective"),
    ("ctrl_2_multi_avg",   "multi_avg"),
    ("ctrl_3_multi_invdist", "multi_invdist"),
]

# Task A probe configurations: (variant_name, seed, partition, description)
TASK_A_CONFIGS = [
    ("original_s42", 42, PARTITION_ORIGINAL,
     "interleaved 4-class partition [0,1,2,3,...]; seed=42 (canonical failure case)"),
    ("shift1_s42",   42, PARTITION_SHIFT1,
     "interleaved 4-class partition shifted +1 [1,2,3,0,...]; seed=42"),
]

HORIZONS = [24, 32]


# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def ctrl_input_dim(n_pairs: int, n_blocks: int) -> int:
    """Controller input dimension — same for all three ctrl_modes."""
    return D_EMB + n_pairs + n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    """Convert one-hot state representation to angular (cos, sin) pairs."""
    shape = onehot.shape[:-1]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*shape, d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            angle            = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def apply_anchor_two_i(tau0_ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    """two_i_orient: rotate each (c,s) pair by +π/2 → (−s, c)."""
    out = tau0_ang.clone()
    for p in range(n_pairs):
        c = tau0_ang[:, 2 * p].clone()
        s = tau0_ang[:, 2 * p + 1].clone()
        out[:, 2 * p]     = -s
        out[:, 2 * p + 1] =  c
    return out


def apply_split_transport(
        base: torch.Tensor,
        tau_prev: torch.Tensor,
        blocks,
        eps_high: float,
) -> torch.Tensor:
    """Split transport: k=1 freely transported; k≥2 frozen (eps_high=1.0)."""
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
            blended   = (1.0 - eps_high) * new_pair + eps_high * prev_pair
            ang_parts.append(blended)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    """Convert state tables → angular + apply two_i_orient anchor (locked)."""
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat(
        [tau0_ang, 1.0 * torch.ones(tau0_ang.shape[0], n_blocks)], dim=1
    )
    return TN_ang, TR, tau0_hyb, pool_ids


def build_class_table_from_lookup(
        tau0_oh: torch.Tensor,
        cyc_start: int,
        cyc_end: int,
        partition: List[int],
) -> torch.Tensor:
    """Geometry-native class labels from a phase-partition lookup table.

    Maps each state's position within the cyclic component to a class via
    the partition list. Pure lookup — no modular framing required.
    """
    period = cyc_end - cyc_start
    assert len(partition) == period
    idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut = torch.tensor(partition, dtype=torch.long)
    return lut[idx]


# ═══════════════════════════════════════════════════════════════════════
# Controller feature functions
# ═══════════════════════════════════════════════════════════════════════
def make_projective_features(ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    """Baseline projective feature: tan(θ/2) per angular pair.

    proj_k = sin_k / (1 + cos_k + PROJ_EPS), clipped to ±PROJ_CLIP.
    Shape: (..., n_pairs)
    """
    parts = []
    for k in range(n_pairs):
        c = ang[..., 2 * k]
        s = ang[..., 2 * k + 1]
        t = s / (1.0 + c + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


def make_multi_boundary_features(
        ang: torch.Tensor,
        n_pairs: int,
        ctrl_mode: str,
) -> torch.Tensor:
    """Multi-boundary aggregated projective features.

    For each of N_BOUNDARIES reference orientations φ_j = j × π/2:
      Rotate the angular pair (c_k, s_k) by −φ_j:
        c_rot = c_k cos(φ_j) + s_k sin(φ_j)
        s_rot = s_k cos(φ_j) − c_k sin(φ_j)
      Compute projective value at rotated frame:
        p_j_k = s_rot / (1 + c_rot + PROJ_EPS), clipped ±PROJ_CLIP

    ctrl_mode="multi_avg":
      output_k = mean_j(p_j_k)    [uniform weights]

    ctrl_mode="multi_invdist":
      Inverse-distance weight: closer to boundary j → higher weight
        distance_proxy_j_k = 1 − c_rot  (in [0, 2])
        w_j_k = 1 / (distance_proxy_j_k + INV_DIST_EPS)
      output_k = Σ_j(w_j_k · p_j_k) / Σ_j(w_j_k)

    Output shape: (..., n_pairs)
    """
    cos_phis = [math.cos(phi) for phi in BOUNDARY_ANGLES]
    sin_phis = [math.sin(phi) for phi in BOUNDARY_ANGLES]

    result_pairs = []
    for k in range(n_pairs):
        c_k = ang[..., 2 * k]      # (...,)
        s_k = ang[..., 2 * k + 1]

        p_list = []
        w_list = []
        for cos_phi, sin_phi in zip(cos_phis, sin_phis):
            c_rot = c_k * cos_phi + s_k * sin_phi
            s_rot = s_k * cos_phi - c_k * sin_phi
            p_j   = (s_rot / (1.0 + c_rot + PROJ_EPS)).clamp(-PROJ_CLIP, PROJ_CLIP)
            p_list.append(p_j)

            if ctrl_mode == "multi_invdist":
                # Angular distance proxy: 1 − cos(θ − φ_j) ∈ [0, 2]
                dist = 1.0 - c_rot
                w_j  = 1.0 / (dist + INV_DIST_EPS)
                w_list.append(w_j)

        if ctrl_mode == "multi_avg":
            agg = sum(p_list) / N_BOUNDARIES
        else:  # multi_invdist
            w_sum = sum(w_list)
            agg   = sum(p * w for p, w in zip(p_list, w_list)) / (w_sum + 1e-12)

        result_pairs.append(agg.unsqueeze(-1))

    return torch.cat(result_pairs, dim=-1)   # (..., n_pairs)


# ═══════════════════════════════════════════════════════════════════════
# Router model
# ═══════════════════════════════════════════════════════════════════════
class RouterMultiBoundaryProbe(nn.Module):
    """Router for multi-boundary controller probe.

    Fixed: BLOCKS_A, eps_high=1.0, two_i_orient anchor.
    Varies: ctrl_mode ∈ {projective, multi_avg, multi_invdist}.
    """

    def __init__(
            self,
            TN_ang, TR, tau0_hyb, pool_ids,
            D: int,
            blocks,
            ctrl_mode: str,
            eps_high: float = 1.0,
            init_scale: float = INIT_SCALE,
            seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        self.blocks    = blocks
        self.eps_high  = eps_high
        self.ctrl_mode = ctrl_mode
        self.D         = D

        d_ang, n_pairs, n_blocks = geom_dims(blocks)
        self.d_ang    = d_ang
        self.n_pairs  = n_pairs
        self.n_blocks = n_blocks

        dh  = D_HIDDEN
        dha = max(8, dh // 4)
        d_hyb  = d_ang + n_blocks
        d_ctrl = ctrl_input_dim(n_pairs, n_blocks)
        self.d_ctrl = d_ctrl
        self.d_hyb  = d_hyb

        self.register_buffer("TN",         TN_ang)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids",   pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]         = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1]     = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(0.0))
        self.b_posLast = nn.Parameter(torch.tensor(0.0))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(
            torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb  = rp(VOCAB, D_EMB)
        self.W1     = rp(d_ctrl, dh);    self.b1     = zp(dh)
        self.W2     = rp(dh, N_OPS);     self.b2     = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb);    self.b_attn = zp(dha)
        self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB);  self.b_pred = zp(VOCAB)

    def _build_ctrl_input(self, embs: torch.Tensor,
                          tau_prev: torch.Tensor) -> torch.Tensor:
        """Build controller input: [embedding | projective_features | magnitudes].

        All three ctrl_modes use the same input dimension:
          D_EMB + n_pairs + n_blocks
        Only the projective feature computation differs.
        """
        ang  = tau_prev[:, :self.d_ang]     # (B, d_ang)
        mags = tau_prev[:, self.d_ang:]     # (B, n_blocks)

        if self.ctrl_mode == "projective":
            proj = make_projective_features(ang, self.n_pairs)
        else:
            proj = make_multi_boundary_features(ang, self.n_pairs, self.ctrl_mode)

        return torch.cat([embs, proj, mags], dim=1)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        ctrl = self._build_ctrl_input(embs, tau_prev)
        h    = torch.tanh(ctrl @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax(
                (logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        base      = torch.einsum("bi,bij->bj", w, tn)
        hybrid    = apply_split_transport(base, tau_prev, self.blocks, self.eps_high)
        new_sids  = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def _eval_transport(self, tau_prev, best_ang) -> torch.Tensor:
        return apply_split_transport(best_ang, tau_prev, self.blocks, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        D        = self.D
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(
                tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = hybrid
            soft_taus.append(tau_prev)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return (torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred,
                alpha)

    def readout_masked(self, st: torch.Tensor,
                       mask: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + mask)
        alpha = torch.softmax(sc, dim=1)
        return (torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred,
                alpha)


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, classes, D: int, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, classes) -> Tuple[float, float]:
    D = model.D
    B = BATCH_SIZE
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, classes, D, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)
                ).squeeze(1)
                hybrid    = model._eval_transport(tau_prev, best_ang)
                taus.append(hybrid)
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            pred, alpha = model.readout(st)
            correct += (pred.argmax(1) == x0).sum().item()
            aD_sum  += alpha[:, D - 1].mean().item()
    model.train()
    return round(correct / N_EVAL, 4), round(aD_sum / N_BATCHES, 4)


def collect_trajectories(model, pool_ids, classes):
    D  = model.D
    B  = BATCH_SIZE
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st: List[torch.Tensor] = []
    all_x0: List[torch.Tensor] = []
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, classes, D, rng)
            all_x0.append(x0)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)
                ).squeeze(1)
                hybrid    = model._eval_transport(tau_prev, best_ang)
                taus.append(hybrid.clone())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
    return all_st, all_x0


def eval_no_last(model, all_st, all_x0) -> float:
    D    = model.D
    mask = torch.zeros(1, D)
    mask[0, D - 1] = NEG_INF
    correct = 0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            pred, _ = model.readout_masked(st, mask)
            correct += (pred.argmax(1) == x0).sum().item()
    return round(correct / N_EVAL, 4)


def linear_probe(X: np.ndarray, y: np.ndarray, label: str) -> float:
    import warnings
    scaler = StandardScaler()
    Xs     = scaler.fit_transform(X)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        clf = LogisticRegression(max_iter=2000, C=1.0, solver="lbfgs")
        clf.fit(Xs, y)
    acc = float((clf.predict(Xs) == y).mean())
    print(f"      [{label}] decodability={acc:.4f}")
    return round(acc, 4)


def run_decodability_final(model, pool_ids, classes) -> float:
    """Linear decodability at the final trajectory step."""
    D    = model.D
    B    = BATCH_SIZE
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    n_b  = (N_PROBE + B - 1) // B
    tau_f_list: List[torch.Tensor] = []
    lbl_list:   List[torch.Tensor] = []
    with torch.no_grad():
        for _ in range(n_b):
            sids, toks, x0 = make_batch(pool_ids, classes, D, rng)
            lbl_list.append(x0.cpu())
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)
                ).squeeze(1)
                hybrid = model._eval_transport(tau_prev, best_ang)
                if t == D - 1:
                    tau_f_list.append(hybrid.cpu())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
    y     = torch.cat(lbl_list,   dim=0).numpy()
    tau_f = torch.cat(tau_f_list, dim=0).numpy()
    return linear_probe(tau_f, y, f"final(t={D-1})")


# ═══════════════════════════════════════════════════════════════════════
# Training loop
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        label: str,
        ctrl_mode: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, classes,
        blocks,
        seed: int,
) -> Dict:
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    d_ctrl = ctrl_input_dim(n_pairs, n_blocks)
    print(f"\n  [{label}|D={D}]  ctrl_mode={ctrl_mode}  "
          f"d_ctrl={d_ctrl}  n_pairs={n_pairs}  n_blocks={n_blocks}")

    model = RouterMultiBoundaryProbe(
        TN_ang, TR, tau0_hyb, pool_ids,
        D=D, blocks=blocks, ctrl_mode=ctrl_mode, eps_high=1.0,
        seed=seed,
    )
    opt   = torch.optim.Adam(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 1)
    t0    = time.perf_counter()

    final_acc  = 0.0
    solve_step: Optional[int] = None
    alphaD     = 0.0

    for step in range(1, MAX_STEPS + 1):
        temp = TEMP_START * (TEMP_END / TEMP_START) ** (step / MAX_STEPS)
        sids, toks, x0 = make_batch(pool_ids, classes, D, rng_t)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()

        if step % EVAL_EVERY == 0:
            acc, aD = eval_acc(model, pool_ids, classes)
            wall    = time.perf_counter() - t0
            print(f"    step={step:5d}  acc={acc:.4f}  α_D={aD:.4f}  "
                  f"wall={wall:.1f}s")
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            if step == MAX_STEPS:
                final_acc = acc
                alphaD    = aD

    wall = time.perf_counter() - t0
    print(f"    DONE: final_acc={final_acc:.4f}  "
          f"solve_step={solve_step}  wall={wall:.1f}s")

    return {
        "model":      model,
        "pool_ids":   pool_ids,
        "classes":    classes,
        "final_acc":  final_acc,
        "solve_step": solve_step,
        "alphaD":     alphaD,
        "wall":       round(wall, 1),
    }


# ═══════════════════════════════════════════════════════════════════════
# CSV writer
# ═══════════════════════════════════════════════════════════════════════
FIELDNAMES = [
    "controller", "ctrl_mode", "partition_variant", "horizon", "seed",
    "accuracy", "delta_d32_minus_d24",
    "solve_step", "no_last_accuracy", "decodability_final",
    "alpha_last", "runtime_seconds", "note",
]


def write_csv(rows: List[Dict]) -> None:
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=FIELDNAMES)
        w.writeheader()
        w.writerows(rows)
    print(f"  CSV → {CSV_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(rows: List[Dict]) -> None:
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")

    # Index by (controller, partition_variant, horizon)
    idx: Dict = {}
    for r in rows:
        key = (r["controller"], r["partition_variant"], int(r["horizon"]))
        idx[key] = r

    def get(ctrl, var, d, field, default="?"):
        r = idx.get((ctrl, var, d))
        return r[field] if r else default

    def fmt(ctrl, var, d):
        v = get(ctrl, var, d, "accuracy")
        return f"{float(v):.4f}" if v != "?" else "?"

    def delta(ctrl, var):
        a24 = get(ctrl, var, 24, "accuracy")
        a32 = get(ctrl, var, 32, "accuracy")
        if a24 != "?" and a32 != "?":
            return round(float(a32) - float(a24), 4)
        return None

    L: List[str] = []
    L.append("# Prime Transport: Multi-Boundary Controller Probe v1\n\n")
    L.append(f"*Generated: {ts}*\n\n---\n\n")

    # ── Geometry lock section ──────────────────────────────────────────
    L.append("## Pre-Code Geometry Lock (verified)\n\n")
    L.append("**Cyclic state space:** dominant 12-state cyclic component "
             "(one-hot positions 9–21, BLOCKS_A block `(9,21,12,3)`).\n\n")
    L.append("**Interleaved 4-class partition (CONFIRMED INTERLEAVED):**\n"
             "`[0,1,2,3,0,1,2,3,0,1,2,3]`\n"
             "class 0 = {0,4,8}, class 1 = {1,5,9}, class 2 = {2,6,10}, "
             "class 3 = {3,7,11}\n"
             "NOT contiguous blocks — every class member is 4 cyclic positions apart.\n\n")
    L.append("**Projective controller (canonical):** "
             "`proj_k = sin_k / (1 + cos_k + 0.1)`, clipped ±10.\n\n")
    L.append("**Split transport:** fundamental harmonic free; k≥2 frozen "
             "(`eps_high=1.0`).\n\n")
    L.append("**two_i anchoring:** tau0 rotated +π/2 per pair: `(c,s)→(−s,c)`.\n\n")
    L.append("**Boundary definition:** every consecutive state pair is a class "
             "boundary (12 total). 4 distinct boundary types (0→1, 1→2, 2→3, 3→0). "
             "No single cut — boundaries are uniformly distributed.\n\n---\n\n")

    # ── Canonical data ─────────────────────────────────────────────────
    L.append("## Canonical Data Source\n\n")
    L.append("**CANONICAL:** `router_locked_stack_failure_isolation_probe_v1.csv` "
             "(probe1_taskA, projective locked stack)\n\n")
    L.append("| variant | D=24 | D=32 | Δ(D32−D24) |\n"
             "|---------|------|------|------------|\n")
    L.append(f"| original_s42 | {CANON_ORIG_D24} | {CANON_ORIG_D32} "
             f"| {CANON_ORIG_D32 - CANON_ORIG_D24:+.4f} |\n")
    L.append(f"| shift1_s42   | {CANON_SHIFT_D24} | {CANON_SHIFT_D32} "
             f"| {CANON_SHIFT_D32 - CANON_SHIFT_D24:+.4f} |\n\n")
    L.append(f"Baseline Δ spread (|orig − shift1|): "
             f"D24 = {abs(CANON_ORIG_D24 - CANON_SHIFT_D24):.4f}, "
             f"D32 = {abs(CANON_ORIG_D32 - CANON_SHIFT_D32):.4f}\n\n---\n\n")

    # ── Multi-boundary design ──────────────────────────────────────────
    L.append("## Multi-Boundary Controller Design\n\n")
    L.append(f"N_BOUNDARIES = {N_BOUNDARIES}, "
             f"reference angles φ_j = j×π/2 = "
             f"{{{', '.join(f'{a:.3f}' for a in BOUNDARY_ANGLES)}}}\n\n")
    L.append("For each pair k and boundary j:\n"
             "  c_rot = c_k·cos(φ_j) + s_k·sin(φ_j)\n"
             "  s_rot = s_k·cos(φ_j) − c_k·sin(φ_j)\n"
             "  p_j_k = s_rot / (1 + c_rot + 0.1)   [clipped ±10]\n\n")
    L.append("**ctrl_1 (baseline projective):** p_0_k only (φ_0 = 0)\n")
    L.append("**ctrl_2 (multi_avg):** output_k = mean_j(p_j_k)\n")
    L.append(f"**ctrl_3 (multi_invdist):** "
             f"w_j_k = 1/(1 − c_rot + {INV_DIST_EPS}), "
             f"output_k = Σ(w·p)/Σ(w)\n\n")
    L.append(f"All three controllers: input dim = D_EMB + n_pairs + n_blocks "
             f"= {D_EMB} + 6 + 4 = {D_EMB + 6 + 4}\n\n---\n\n")

    # ── Results table ──────────────────────────────────────────────────
    L.append("## Results\n\n")
    headers = "| controller | variant | D=24 | D=32 | Δ(D32−D24) | dec_final | Δ_spread |\n"
    sep     = "|------------|---------|------|------|------------|-----------|----------|\n"
    L.append(headers); L.append(sep)

    for ctrl_name, ctrl_mode in CONTROLLER_VARIANTS:
        d_orig  = delta(ctrl_name, "original_s42")
        d_shift = delta(ctrl_name, "shift1_s42")
        spread_d32 = (abs(float(get(ctrl_name, "original_s42", 32, "accuracy", 0)) -
                          float(get(ctrl_name, "shift1_s42",   32, "accuracy", 0)))
                      if d_orig is not None and d_shift is not None else None)
        for var, _ in [("original_s42", None), ("shift1_s42", None)]:
            a24 = fmt(ctrl_name, var, 24)
            a32 = fmt(ctrl_name, var, 32)
            d   = delta(ctrl_name, var)
            dec = get(ctrl_name, var, 32, "decodability_final")
            d_str = f"{d:+.4f}" if d is not None else "?"
            sp_str = f"{spread_d32:.4f}" if spread_d32 is not None else "?"
            L.append(f"| {ctrl_name} | {var} | {a24} | {a32} | "
                     f"{d_str} | {dec} | {sp_str} |\n")

    L.append("\n---\n\n")

    # ── Orientation sensitivity analysis ──────────────────────────────
    L.append("## Orientation Sensitivity Analysis\n\n")
    L.append("**Goal:** Does multi-boundary aggregation reduce the gap between "
             "original_s42 and shift1_s42?\n\n")
    L.append("Metric: |Δ_original − Δ_shift1| at D32 (smaller = more invariant)\n\n")
    L.append("| controller | Δ_original | Δ_shift1 | |Δ_orig − Δ_sh1| "
             "| D32_orig | D32_shift1 | D32_spread |\n"
             "|------------|------------|----------|-----------------|"
             "----------|------------|------------|\n")

    for ctrl_name, _ in CONTROLLER_VARIANTS:
        d_orig  = delta(ctrl_name, "original_s42")
        d_shift = delta(ctrl_name, "shift1_s42")
        a32_orig  = get(ctrl_name, "original_s42", 32, "accuracy", None)
        a32_shift = get(ctrl_name, "shift1_s42",   32, "accuracy", None)
        if d_orig is not None and d_shift is not None:
            gap     = abs(d_orig - d_shift)
            spread  = abs(float(a32_orig) - float(a32_shift))
            d_o_s   = f"{d_orig:+.4f}"
            d_s_s   = f"{d_shift:+.4f}"
            gap_s   = f"{gap:.4f}"
            sp_s    = f"{spread:.4f}"
        else:
            d_o_s = d_s_s = gap_s = sp_s = "?"
            a32_orig = a32_shift = "?"
        L.append(f"| {ctrl_name} | {d_o_s} | {d_s_s} | {gap_s} "
                 f"| {a32_orig} | {a32_shift} | {sp_s} |\n")

    L.append("\n")
    L.append(f"**Baseline (ctrl_1 projective):**\n"
             f"  Δ_original = {CANON_ORIG_D32 - CANON_ORIG_D24:+.4f},  "
             f"Δ_shift1 = {CANON_SHIFT_D32 - CANON_SHIFT_D24:+.4f},  "
             f"|gap| = {abs((CANON_ORIG_D32 - CANON_ORIG_D24) - (CANON_SHIFT_D32 - CANON_SHIFT_D24)):.4f}\n\n")

    # ── Success evaluation ─────────────────────────────────────────────
    L.append("## Success Evaluation\n\n")
    L.append("Success criteria (per prompt contract):\n"
             "1. Smaller gap between original_s42 and shift1_s42 (reduced orientation sensitivity)\n"
             "2. Stable or improved D32 accuracy\n"
             "3. No decodability collapse (final decodability ≥ 0.95)\n\n")

    baseline_gap = abs(
        (CANON_ORIG_D32 - CANON_ORIG_D24) - (CANON_SHIFT_D32 - CANON_SHIFT_D24)
    )
    for ctrl_name, _ in CONTROLLER_VARIANTS:
        d_orig  = delta(ctrl_name, "original_s42")
        d_shift = delta(ctrl_name, "shift1_s42")
        a32_orig  = get(ctrl_name, "original_s42", 32, "accuracy", None)
        dec_orig  = get(ctrl_name, "original_s42", 32, "decodability_final", None)
        dec_shift = get(ctrl_name, "shift1_s42",   32, "decodability_final", None)
        if all(v is not None for v in [d_orig, d_shift, a32_orig, dec_orig, dec_shift]):
            gap        = abs(d_orig - d_shift)
            gap_str    = "REDUCED" if gap < baseline_gap else "NOT_REDUCED"
            d32_str    = "STABLE" if float(a32_orig) >= CANON_ORIG_D32 - 0.02 else "DEGRADED"
            dec_str    = ("OK" if float(dec_orig) >= 0.95 and
                          float(dec_shift) >= 0.95 else "COLLAPSED")
            L.append(f"- {ctrl_name}: gap={gap_str} ({gap:.4f} vs baseline {baseline_gap:.4f}), "
                     f"D32={d32_str} ({a32_orig}), decodability={dec_str}\n")
        else:
            L.append(f"- {ctrl_name}: (data pending)\n")

    L.append("\n---\n\n")
    L.append("*This is a mechanism isolation step, NOT a performance optimization step.*\n"
             "*No new tasks, φ/Fibonacci, or transport modifications were introduced.*\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"  MD  → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    n_runs = len(CONTROLLER_VARIANTS) * len(TASK_A_CONFIGS) * len(HORIZONS)
    print("=" * 70)
    print("MULTI-BOUNDARY CONTROLLER PROBE v1")
    print(f"  Controllers:      {len(CONTROLLER_VARIANTS)}")
    print(f"  Partition variants: {len(TASK_A_CONFIGS)}")
    print(f"  Horizons:         {HORIZONS}")
    print(f"  Total runs:       {n_runs}  (must be 12)")
    assert n_runs == 12, f"Run count violation: expected 12, got {n_runs}"
    print("=" * 70)

    # Verify canonical file
    canon_file = RESULTS_DIR / "router_locked_stack_failure_isolation_probe_v1.csv"
    print(f"\n  {'✓' if canon_file.exists() else '✗ MISSING'} "
          f"router_locked_stack_failure_isolation_probe_v1.csv")
    print(f"  CANONICAL baseline — original_s42: "
          f"D24={CANON_ORIG_D24}  D32={CANON_ORIG_D32}  "
          f"Δ={CANON_ORIG_D32 - CANON_ORIG_D24:+.4f}")
    print(f"  CANONICAL baseline — shift1_s42:   "
          f"D24={CANON_SHIFT_D24}  D32={CANON_SHIFT_D32}  "
          f"Δ={CANON_SHIFT_D32 - CANON_SHIFT_D24:+.4f}")

    # Load state cache
    if not CACHE_PATH.exists():
        print(f"\nERROR: state cache not found at {CACHE_PATH}")
        sys.exit(1)

    print(f"\nLoading state cache: {CACHE_PATH}")
    cache      = torch.load(CACHE_PATH, map_location="cpu", weights_only=True)
    TN_oh      = cache["TN_oh"]
    tau0_oh    = cache["tau0_oh"]
    TR         = cache["TR"]
    pool_ids   = cache["pool_ids"]
    print(f"  Cache loaded: {tau0_oh.shape[0]} states, pool={pool_ids.shape[0]}")

    # Pre-build tables once (same anchor/geometry for all runs)
    TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, BLOCKS_A
    )
    d_ang, n_pairs, n_blocks = geom_dims(BLOCKS_A)
    print(f"  BLOCKS_A: d_ang={d_ang}  n_pairs={n_pairs}  n_blocks={n_blocks}")
    print(f"  Controller input dim: {ctrl_input_dim(n_pairs, n_blocks)}")

    csv_rows: List[Dict] = []
    run_num = 0

    for D in HORIZONS:
        print(f"\n{'=' * 70}")
        print(f"HORIZON D={D}")
        print("=" * 70)

        for vname, vseed, vpartition, vdesc in TASK_A_CONFIGS:
            classes   = build_class_table_from_lookup(
                tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E, vpartition
            )
            cls_dist  = [(classes == i).sum().item() for i in range(VOCAB)]
            print(f"\n── variant={vname}  classes={cls_dist} ──")

            for ctrl_name, ctrl_mode in CONTROLLER_VARIANTS:
                run_num += 1
                label = f"{ctrl_name}/{vname}"
                print(f"\n[Run {run_num}/12]")

                result = train_one(
                    label, ctrl_mode, D,
                    TN_ang, TR_p, tau0_hyb, pool_ids_p, classes,
                    BLOCKS_A, seed=vseed,
                )
                model = result["model"]
                model.eval()

                dec_f           = run_decodability_final(model, pool_ids_p, classes)
                all_st, all_x0  = collect_trajectories(model, pool_ids_p, classes)
                no_last         = eval_no_last(model, all_st, all_x0)

                csv_rows.append({
                    "controller":        ctrl_name,
                    "ctrl_mode":         ctrl_mode,
                    "partition_variant": vname,
                    "horizon":           D,
                    "seed":              vseed,
                    "accuracy":          result["final_acc"],
                    "delta_d32_minus_d24": "",   # filled in post-hoc in MD
                    "solve_step":        result["solve_step"] or "",
                    "no_last_accuracy":  no_last,
                    "decodability_final": dec_f,
                    "alpha_last":        result["alphaD"],
                    "runtime_seconds":   result["wall"],
                    "note":              (
                        f"ctrl={ctrl_mode}; variant={vname}; D={D}; seed={vseed}; "
                        f"partition={vpartition}; anchor=two_i_rot; "
                        f"eps_high=1.0; n_boundaries={N_BOUNDARIES}; "
                        f"boundary_angles=j*pi/2; {vdesc}"
                    ),
                })
                write_csv(csv_rows)

    print(f"\nTotal runs completed: {run_num}/12")
    assert run_num == 12

    # Compute Δ(D32−D24) per (controller, variant) and fill into rows
    acc_by_key: Dict = {}
    for r in csv_rows:
        key = (r["controller"], r["partition_variant"], int(r["horizon"]))
        acc_by_key[key] = float(r["accuracy"])

    for r in csv_rows:
        ctrl  = r["controller"]
        var   = r["partition_variant"]
        d24   = acc_by_key.get((ctrl, var, 24))
        d32   = acc_by_key.get((ctrl, var, 32))
        if d24 is not None and d32 is not None:
            r["delta_d32_minus_d24"] = round(d32 - d24, 4)

    write_csv(csv_rows)
    write_markdown(csv_rows)
    print("\nDone.")


if __name__ == "__main__":
    main()
