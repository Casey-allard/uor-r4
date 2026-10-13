#!/usr/bin/env python3
"""run_router_phi_r4_alignment_probe_v1.py

PHI-R4-DIMENSION ALIGNMENT PROBE v1
=====================================

Falsification-first bounded hypothesis test.

HYPOTHESES:
  H0 (NULL):   Current best stack (two_i + projective + split transport) is sufficient.
               No φ-based or R^4 modification improves orientation robustness.
  H1 (PHI):    φ-weighted canonical angular references improve orientation robustness.
  H2 (PHI+R4): φ-angle improvement only effective combined with R^4-style geometric coupling.
  H3 (DIM):    Performance depends non-monotonically on D (preferred D values exist).

CANONICAL DATA SOURCES (read before coding):
  CANONICAL:
    prime_transport_router_start_agnostic_root_probe_v1.csv
      → two_i_rot best anchor: D24=0.9922, D32=0.9873
    prime_transport_router_controller_geometry_probe_v1.csv
      → projective controller best D32: D24=0.9878, D32=0.9946
  CONTRAST (not used as truth): none

PRE-CODE GEOMETRY LOCK (verified):
  - 12-state cyclic dominant component; block [9:21] in tau0_oh (12 states, n_h=3)
  - Interleaved 4-class partition: (argmax in [9:21]) % 4 → [0,1,2,3,0,1,2,3,0,1,2,3]
  - Two orientation variants (NOT redefining partition structure):
      original_s42: labels = argmax([9:21]) % 4,        seed=42
      shift1_s42:   labels = (argmax([9:21]) + 1) % 4,  seed=42
  - Geometry: GEOM_K3 = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]
  - Do NOT use block-of-3 partitions.

FIXED COMPONENTS (same as canonical best stack):
  - Anchor: two_i_rot (rotate each (c,s) pair by +π/2 → (-s,c))
  - Transport: split, eps_high=1.0 (k≥2 frozen)
  - Training budget: MAX_STEPS=3500

CONTROLLER VARIANTS:
  1. ctrl_baseline
       Projective controller: proj_k = sin_k / (1 + cos_k + eps), clipped ±10
       Single reference frame (no additional angular offsets)
       Controller input: [emb(4) | proj(n_pairs) | mags(n_blocks)]

  2. ctrl_phi
       φ-based angular offsets: θ_k = 2π·frac(k/φ) for k=0..n_pairs-1
       Offsets are NOT evenly spaced (Fibonacci-distributed on [0, 2π))
       Apply rotation by φ_k to each pair before computing projective features:
         (c', s') = (c·cos(φ_k) - s·sin(φ_k),  c·sin(φ_k) + s·cos(φ_k))
       Then: proj_k = s' / (1 + c' + eps)
       Controller input: [emb(4) | phi_proj(n_pairs) | mags(n_blocks)]

  3. ctrl_phi_r4
       Same φ-based offsets as ctrl_phi.
       Plus R^4-style double-angle coupling (geometry-native, no learned params):
         c2' = c'^2 - s'^2  (= cos(2(θ + φ_k)))
         s2' = 2·c'·s'      (= sin(2(θ + φ_k)))
         r4_k = s2' / (1 + c2' + eps)  (projective of double-angle pair)
       Controller input: [emb(4) | phi_proj(n_pairs) | r4_proj(n_pairs) | mags(n_blocks)]

DIMENSION SWEEP:
  Mandatory: D ∈ {24, 32}
  Optional (INCLUDE_OPTIONAL=False by default): D ∈ {20, 28, 36}

MEASUREMENTS per run:
  accuracy, solve_step, no_last_accuracy,
  decodability_initial/mid/final, alpha_last, runtime_seconds

SUMMARY METRICS (computed in markdown):
  Δ(D32 − D24) per (ctrl, orientation)
  orientation_gap = |Δ_original − Δ_shift1| per ctrl
  decodability_final (collapse detection)

SUCCESS CRITERIA (per task spec):
  Hypothesis supported ONLY IF:
    1. Orientation gap reduced vs baseline
    2. D32 performance stable or improved
    3. No decodability collapse

FAILURE CONDITIONS (hypothesis NOT supported if ANY):
  - φ controller does not outperform baseline
  - φ+R4 does not improve over φ alone
  - Performance vs D is monotonic or flat (no alignment behavior)
  - Improvements occur in only one orientation

DELIVERABLES:
  tools/prime_transport/run_router_phi_r4_alignment_probe_v1.py
  results/prime_transport_recursive_system/router_phi_r4_alignment_probe_v1.csv
  docs/research/prime_transport_phi_r4_alignment_probe_v1.md
"""
from __future__ import annotations

import csv
import datetime
import math
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler

# ═══════════════════════════════════════════════════════════════════════
# Step 0: CPU/thread fix (MUST happen before any model work)
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
    print(f"  [thread_policy] {_n} thread(s) — fallback min(8, cpu_count)")

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "router_phi_r4_alignment_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_phi_r4_alignment_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Hyperparameters — identical to controller_geometry_probe_v1 / start_agnostic_root_probe_v1
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED    = 42
D_HIDDEN       = 32
BATCH_SIZE     = 512
VOCAB          = 4
D_EMB          = 4
N_PHASE_PAIRS  = 4
N_OPS          = 6
LR             = 0.02
TEMP_START     = 2.0
TEMP_END       = 0.1
CLIP_GRAD      = 1.0
EVAL_EVERY     = 500
N_EVAL         = 2048
SOLVE_ACC      = 0.999
INIT_SCALE     = 0.05
MAX_STEPS      = 3_500
N_BATCHES      = N_EVAL // BATCH_SIZE
NEG_INF        = -1e9
N_PROBE        = 4096
MOD12_S, MOD12_E = 9, 21

TASK_INJECT    = False
B_POS0_INIT    = 0.0
B_POSLAST_INIT = 0.0

# Projective feature parameters
PROJ_EPS  = 0.1
PROJ_CLIP = 10.0

# Golden ratio
PHI = (1.0 + math.sqrt(5.0)) / 2.0

# ═══════════════════════════════════════════════════════════════════════
# Dimension sweep — mandatory only by default
# Set INCLUDE_OPTIONAL = True to also run D ∈ {20, 28, 36}
# ═══════════════════════════════════════════════════════════════════════
INCLUDE_OPTIONAL = False
HORIZONS_MANDATORY = [24, 32]
HORIZONS_OPTIONAL  = [20, 28, 36]
HORIZONS = HORIZONS_MANDATORY + (HORIZONS_OPTIONAL if INCLUDE_OPTIONAL else [])

# ═══════════════════════════════════════════════════════════════════════
# Geometry — GEOM_K3 (fixed, identical to prior probes)
# ═══════════════════════════════════════════════════════════════════════
GEOM_K3 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# Anchor fixed to two_i_rot throughout (canonical best from start_agnostic probe)
ANCHOR_TYPE = "two_i_rot"

# ═══════════════════════════════════════════════════════════════════════
# Task orientation variants
# ═══════════════════════════════════════════════════════════════════════
# (variant_name, label_shift)
ORIENTATIONS = [
    ("original_s42", 0),
    ("shift1_s42",   1),
]

# ═══════════════════════════════════════════════════════════════════════
# Controller variants
# ═══════════════════════════════════════════════════════════════════════
# (ctrl_name, ctrl_mode)
CTRL_VARIANTS = [
    ("ctrl_baseline", "baseline"),
    ("ctrl_phi",      "phi"),
    ("ctrl_phi_r4",   "phi_r4"),
]


# ═══════════════════════════════════════════════════════════════════════
# φ-offset computation (precomputed constants — not learned)
# ═══════════════════════════════════════════════════════════════════════
def make_phi_offsets(n_pairs: int) -> List[float]:
    """φ-based angular offsets for n_pairs reference frames.

    θ_k = 2π · frac(k / φ)   for k = 0, 1, ..., n_pairs - 1

    These are the golden-angle Fibonacci-distributed positions on [0, 2π).
    They are explicitly NOT evenly spaced (successive differences are irrational).

    k=0: θ=0.0000 (0)
    k=1: θ≈3.8832 (2π × 0.6180)
    k=2: θ≈1.5425 (2π × 0.2361)
    k=3: θ≈5.4258 (2π × 0.8541)
    k=4: θ≈2.6217 (2π × 0.4721)
    k=5: θ≈6.4674 (2π × 1.0000 - tiny wrap → ≈0.3)
    """
    offsets = []
    for k in range(n_pairs):
        frac = (k / PHI) % 1.0
        offsets.append(2.0 * math.pi * frac)
    return offsets


# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int, int]:
    n_pairs   = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang     = 2 * n_pairs
    d_hyb     = d_ang + N_PHASE_PAIRS
    d_in_ctrl = D_EMB + d_hyb
    return d_ang, n_pairs, d_hyb, d_in_ctrl


def ctrl_input_dim(d_ang: int, n_pairs: int, n_blocks: int, ctrl_mode: str) -> int:
    """Return controller input dimension.

    baseline:  D_EMB + n_pairs + n_blocks   (projective features + mags)
    phi:       D_EMB + n_pairs + n_blocks   (φ-projective + mags — same dim)
    phi_r4:    D_EMB + n_pairs + n_pairs + n_blocks  (φ-proj + r4-proj + mags)
    """
    if ctrl_mode in ("baseline", "phi"):
        return D_EMB + n_pairs + n_blocks
    elif ctrl_mode == "phi_r4":
        return D_EMB + n_pairs + n_pairs + n_blocks
    else:
        raise ValueError(f"Unknown ctrl_mode: {ctrl_mode!r}")


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    shape = onehot.shape[:-1]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*shape, d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            angle = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def apply_anchor_two_i(tau0_ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    """Apply two_i_rot anchoring: rotate each (c, s) pair by +π/2 → (−s, c)."""
    out = tau0_ang.clone()
    for p in range(n_pairs):
        c = tau0_ang[:, 2 * p].clone()
        s = tau0_ang[:, 2 * p + 1].clone()
        out[:, 2 * p]     = -s
        out[:, 2 * p + 1] =  c
    return out


def make_projective_features(ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    """Standard half-angle stereographic projection (baseline).

    proj_k = sin_k / (1 + cos_k + PROJ_EPS), clipped to [-PROJ_CLIP, +PROJ_CLIP].
    """
    parts = []
    for k in range(n_pairs):
        c = ang[..., 2 * k]
        s = ang[..., 2 * k + 1]
        t = s / (1.0 + c + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


def make_phi_projective_features(
        ang: torch.Tensor,
        n_pairs: int,
        phi_offsets: List[float],
) -> torch.Tensor:
    """φ-rotated projective features.

    For each pair k:
      1. Rotate (c, s) by φ_offset[k]: (c', s') = (c·cos(φ_k) - s·sin(φ_k),
                                                     c·sin(φ_k) + s·cos(φ_k))
      2. proj_k = s' / (1 + c' + PROJ_EPS)

    This is the same projective formula applied to the φ-rotated reference frame.
    The offsets are irrational (golden-angle) and NOT evenly spaced.
    """
    parts = []
    for k in range(n_pairs):
        c = ang[..., 2 * k]
        s = ang[..., 2 * k + 1]
        cos_pk = math.cos(phi_offsets[k])
        sin_pk = math.sin(phi_offsets[k])
        c_rot = c * cos_pk - s * sin_pk
        s_rot = c * sin_pk + s * cos_pk
        t = s_rot / (1.0 + c_rot + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


def make_r4_projective_features(
        ang: torch.Tensor,
        n_pairs: int,
        phi_offsets: List[float],
) -> torch.Tensor:
    """R^4-style double-angle projective features (geometry-native, no learned params).

    For each pair k:
      1. Rotate (c, s) by φ_offset[k] → (c', s')
      2. Compute double-angle: c2' = c'^2 - s'^2,  s2' = 2·c'·s'
         These are cos(2θ') and sin(2θ') — pure Fourier harmonics, no parameters.
      3. r4_k = s2' / (1 + c2' + PROJ_EPS)

    The double-angle expansion maps S^1 → S^1 via the Hopf-fibration double cover,
    increasing representational dimensionality of each pair while preserving
    geometry-native cyclic structure. No learned weights are added.
    """
    parts = []
    for k in range(n_pairs):
        c = ang[..., 2 * k]
        s = ang[..., 2 * k + 1]
        cos_pk = math.cos(phi_offsets[k])
        sin_pk = math.sin(phi_offsets[k])
        c_rot = c * cos_pk - s * sin_pk
        s_rot = c * sin_pk + s * cos_pk
        # Double-angle (geometry-native, no params)
        c2 = c_rot * c_rot - s_rot * s_rot
        s2 = 2.0 * c_rot * s_rot
        t = s2 / (1.0 + c2 + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    """Prepare angular state tables with two_i_orient anchoring (fixed)."""
    d_ang, n_pairs, _, _ = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat(
        [tau0_ang, 1.0 * torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1
    )
    return TN_ang, TR, tau0_hyb, pool_ids


def build_class_table(tau0_oh: torch.Tensor, shift: int = 0) -> torch.Tensor:
    """Build 4-class label table from the 12-state cyclic block.

    original_s42 (shift=0): labels = argmax([9:21]) % 4
    shift1_s42   (shift=1): labels = (argmax([9:21]) + 1) % 4

    Partition is interleaved 4-class [0,1,2,3,...] — NOT redefined.
    Only the cyclic offset of the label assignment changes.
    """
    return ((tau0_oh[:, MOD12_S:MOD12_E].argmax(dim=-1) + shift) % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Split transport (eps_high=1.0 → k≥2 frozen — fixed for all configs)
# ═══════════════════════════════════════════════════════════════════════
def apply_split_transport(
        base: torch.Tensor,
        tau_prev: torch.Tensor,
        blocks,
        eps_high: float,
) -> torch.Tensor:
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


# ═══════════════════════════════════════════════════════════════════════
# Router model — controller mode varies per config
# ═══════════════════════════════════════════════════════════════════════
class RouterPhiR4Probe(nn.Module):
    """Router for φ-R4-dimension alignment probe.

    Fixed:  GEOM_K3, eps_high=1.0, two_i_orient anchor.
    Varies: ctrl_mode ∈ {baseline, phi, phi_r4}.

    ctrl_baseline:
        projective controller — same as controller_geometry_probe_v1 / projective
        single reference frame (standard angular)

    ctrl_phi:
        φ-offset projective controller
        each pair rotated by golden-angle offset before projective computation
        no learned parameters added

    ctrl_phi_r4:
        φ-offset projective + R^4 double-angle projective
        double-angle expansion is purely Fourier (geometry-native, no params)
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

        d_ang, n_pairs, d_hyb, _ = geom_dims(blocks)
        self.d_ang    = d_ang
        self.d_hyb    = d_hyb
        self.n_pairs  = n_pairs
        self.n_blocks = len(blocks)

        # Precompute φ-offsets as fixed list (not learned parameters)
        self._phi_offsets: List[float] = make_phi_offsets(n_pairs)

        dh  = D_HIDDEN
        dha = max(8, dh // 4)

        d_ctrl = ctrl_input_dim(d_ang, n_pairs, self.n_blocks, ctrl_mode)
        self.d_ctrl = d_ctrl

        self.register_buffer("TN",         TN_ang)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids",   pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]         = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1]     = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(float(B_POS0_INIT)))
        self.b_posLast = nn.Parameter(torch.tensor(float(B_POSLAST_INIT)))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb  = rp(VOCAB, D_EMB)
        self.W1     = rp(d_ctrl, dh);   self.b1 = zp(dh)
        self.W2     = rp(dh, N_OPS);    self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb);   self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)

    def _build_ctrl_input(self, embs: torch.Tensor, tau_prev: torch.Tensor) -> torch.Tensor:
        """Build controller input depending on ctrl_mode."""
        ang  = tau_prev[:, :self.d_ang]
        mags = tau_prev[:, self.d_ang:]

        if self.ctrl_mode == "baseline":
            proj = make_projective_features(ang, self.n_pairs)
            return torch.cat([embs, proj, mags], dim=1)

        if self.ctrl_mode == "phi":
            phi_proj = make_phi_projective_features(ang, self.n_pairs, self._phi_offsets)
            return torch.cat([embs, phi_proj, mags], dim=1)

        # phi_r4: φ-projective + R^4 double-angle projective
        phi_proj = make_phi_projective_features(ang, self.n_pairs, self._phi_offsets)
        r4_proj  = make_r4_projective_features(ang, self.n_pairs, self._phi_offsets)
        return torch.cat([embs, phi_proj, r4_proj, mags], dim=1)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        ctrl = self._build_ctrl_input(embs, tau_prev)
        h    = torch.tanh(ctrl @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        base   = torch.einsum("bi,bij->bj", w, tn)
        hybrid = apply_split_transport(base, tau_prev, self.blocks, self.eps_high)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def _eval_transport(self, tau_prev, best_ang) -> torch.Tensor:
        return apply_split_transport(best_ang, tau_prev, self.blocks, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        D = self.D
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(tau_prev, state_ids, tokens[:, t], temp)
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
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha

    def readout_masked(self, st: torch.Tensor, mask: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + mask)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, mod12_classes, D: int, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = mod12_classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, mod12_classes) -> Tuple[float, float]:
    D = model.D
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, D, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
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


def collect_trajectories(model, pool_ids, mod12_classes):
    D  = model.D
    B  = BATCH_SIZE
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st: List[torch.Tensor] = []
    all_x0: List[torch.Tensor] = []
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, D, rng)
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
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
                taus.append(hybrid.clone())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
    return all_st, all_x0


def eval_no_last(model, all_st, all_x0) -> float:
    D    = model.D
    mask = torch.zeros(1, D); mask[0, D - 1] = NEG_INF
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
    print(f"      [{label}] decodability={acc:.4f}  (n={X.shape[0]}, d={X.shape[1]})")
    return round(acc, 4)


def run_decodability_probes(model, pool_ids, mod12_classes) -> Dict[str, float]:
    D    = model.D
    MID  = D // 2 - 1
    B    = BATCH_SIZE
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    n_b  = (N_PROBE + B - 1) // B

    tau0_list:  List[torch.Tensor] = []
    tau_m_list: List[torch.Tensor] = []
    tau_f_list: List[torch.Tensor] = []
    lbl_list:   List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(n_b):
            sids, _, x0 = make_batch(pool_ids, mod12_classes, D, rng)
            tau0_list.append(model.tau0_table[sids].cpu())
            lbl_list.append(x0.cpu())
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid  = model._eval_transport(tau_prev, best_ang)
                if t == MID:
                    tau_m_list.append(hybrid.cpu())
                if t == D - 1:
                    tau_f_list.append(hybrid.cpu())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)

    y        = torch.cat(lbl_list,  dim=0).numpy()
    tau0_arr = torch.cat(tau0_list, dim=0).numpy()
    tau_m    = torch.cat(tau_m_list, dim=0).numpy()
    tau_f    = torch.cat(tau_f_list, dim=0).numpy()

    return {
        "initial": linear_probe(tau0_arr, y, "initial"),
        "mid":     linear_probe(tau_m,    y, f"mid(t={MID})"),
        "final":   linear_probe(tau_f,    y, f"final(t={D-1})"),
    }


# ═══════════════════════════════════════════════════════════════════════
# Training loop
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        run_label: str,
        ctrl_mode: str,
        orientation: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, mod12_classes,
        blocks,
) -> Dict:
    d_ang, n_pairs, d_hyb, _ = geom_dims(blocks)
    d_ctrl = ctrl_input_dim(d_ang, n_pairs, len(blocks), ctrl_mode)
    print(f"\n  [{run_label}|D={D}]  ctrl={ctrl_mode}  orient={orientation}  "
          f"d_ctrl={d_ctrl}  n_pairs={n_pairs}")

    model = RouterPhiR4Probe(
        TN_ang, TR, tau0_hyb, pool_ids,
        D=D, blocks=blocks, ctrl_mode=ctrl_mode, eps_high=1.0,
    )
    opt = torch.optim.Adam(model.parameters(), lr=LR)

    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 1)
    t0    = time.perf_counter()

    final_acc  = 0.0
    solve_step: Optional[int] = None
    alphaD     = 0.0

    for step in range(1, MAX_STEPS + 1):
        temp = TEMP_START * (TEMP_END / TEMP_START) ** (step / MAX_STEPS)
        sids, toks, x0 = make_batch(pool_ids, mod12_classes, D, rng_t)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()

        if step % EVAL_EVERY == 0:
            acc, aD = eval_acc(model, pool_ids, mod12_classes)
            wall    = time.perf_counter() - t0
            print(f"    step={step:5d}  acc={acc:.4f}  α_D={aD:.4f}  wall={wall:.1f}s")
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            if step == MAX_STEPS:
                final_acc = acc
                alphaD    = aD

    wall = time.perf_counter() - t0
    print(f"    DONE: final_acc={final_acc:.4f}  "
          f"solve_step={solve_step}  wall={wall:.1f}s")

    return {
        "run_label":     run_label,
        "ctrl_mode":     ctrl_mode,
        "orientation":   orientation,
        "D":             D,
        "model":         model,
        "pool_ids":      pool_ids,
        "mod12_classes": mod12_classes,
        "final_acc":     final_acc,
        "solve_step":    solve_step,
        "alphaD":        alphaD,
        "wall":          round(wall, 1),
    }


# ═══════════════════════════════════════════════════════════════════════
# CSV writer
# ═══════════════════════════════════════════════════════════════════════
def write_csv(rows: List[Dict]) -> None:
    fieldnames = [
        "config", "ctrl_mode", "orientation", "horizon",
        "accuracy", "solve_step", "no_last_accuracy",
        "decodability_initial", "decodability_mid", "decodability_final",
        "alpha_last", "runtime_seconds", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"  CSV → {CSV_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(csv_rows: List[Dict]) -> None:
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")

    # Index rows by (ctrl_mode, orientation, D)
    idx: Dict[Tuple[str, str, int], Dict] = {}
    for r in csv_rows:
        key = (r["ctrl_mode"], r["orientation"], int(r["horizon"]))
        idx[key] = r

    def get_acc(ctrl, orient, D) -> Optional[float]:
        r = idx.get((ctrl, orient, D))
        if r:
            try:
                return float(r["accuracy"])
            except (TypeError, ValueError):
                pass
        return None

    def get_dec_final(ctrl, orient, D) -> Optional[float]:
        r = idx.get((ctrl, orient, D))
        if r:
            try:
                return float(r["decodability_final"])
            except (TypeError, ValueError):
                pass
        return None

    # Compute Δ(D32 - D24) per (ctrl, orient)
    def delta(ctrl, orient) -> Optional[float]:
        a32 = get_acc(ctrl, orient, 32)
        a24 = get_acc(ctrl, orient, 24)
        if a32 is not None and a24 is not None:
            return round(a32 - a24, 4)
        return None

    # Compute orientation_gap = |Δ_orig - Δ_shift| per ctrl
    def orientation_gap(ctrl) -> Optional[float]:
        d_orig  = delta(ctrl, "original_s42")
        d_shift = delta(ctrl, "shift1_s42")
        if d_orig is not None and d_shift is not None:
            return round(abs(d_orig - d_shift), 4)
        return None

    # Reference values from canonical CSV files
    # From controller_geometry_probe_v1: projective = current best
    REF_PROJ_D24 = 0.9878
    REF_PROJ_D32 = 0.9946
    REF_PROJ_DELTA = round(REF_PROJ_D32 - REF_PROJ_D24, 4)

    ctrl_names  = [c for _, c in CTRL_VARIANTS]
    orient_names = [o for o, _ in ORIENTATIONS]

    # Decodability collapse check: any final dec < 0.9?
    def has_decodability_collapse() -> bool:
        for r in csv_rows:
            try:
                if float(r["decodability_final"]) < 0.9:
                    return True
            except (TypeError, ValueError):
                pass
        return False

    # H1 support: ctrl_phi outperforms ctrl_baseline in orientation robustness
    def h1_verdict() -> str:
        gap_base = orientation_gap("baseline")
        gap_phi  = orientation_gap("phi")
        if gap_base is None or gap_phi is None:
            return "INCONCLUSIVE (data incomplete)"
        phi_d32_orig  = get_acc("phi", "original_s42", 32)
        base_d32_orig = get_acc("baseline", "original_s42", 32)
        phi_d32_shift = get_acc("phi", "shift1_s42", 32)
        base_d32_shift = get_acc("baseline", "shift1_s42", 32)
        if phi_d32_orig is None or base_d32_orig is None:
            return "INCONCLUSIVE (data incomplete)"
        # Requires: smaller gap AND both orientations improved
        gap_reduced   = gap_phi < gap_base
        orig_improved = (phi_d32_orig > base_d32_orig + 0.002)
        shift_improved = (phi_d32_shift is not None and
                          base_d32_shift is not None and
                          phi_d32_shift > base_d32_shift + 0.002)
        dec_ok = not has_decodability_collapse()
        if gap_reduced and orig_improved and shift_improved and dec_ok:
            return f"SUPPORTED: gap {gap_base:.4f}→{gap_phi:.4f}, D32 both orientations improved"
        elif not orig_improved and not shift_improved:
            return (f"NOT SUPPORTED: φ does not outperform baseline "
                    f"(gap {gap_base:.4f}→{gap_phi:.4f}; "
                    f"neither orientation improved at D32)")
        elif not gap_reduced:
            return (f"NOT SUPPORTED: φ gap={gap_phi:.4f} ≥ baseline gap={gap_base:.4f} "
                    f"(orientation robustness not improved)")
        elif not (orig_improved and shift_improved):
            return (f"NOT SUPPORTED: φ improvement is single-orientation only "
                    f"(orig_improved={orig_improved}, shift_improved={shift_improved})")
        else:
            return "INCONCLUSIVE"

    # H2 support: phi_r4 improves over phi alone
    def h2_verdict() -> str:
        gap_phi  = orientation_gap("phi")
        gap_r4   = orientation_gap("phi_r4")
        if gap_phi is None or gap_r4 is None:
            return "INCONCLUSIVE (data incomplete)"
        r4_d32_orig  = get_acc("phi_r4", "original_s42", 32)
        phi_d32_orig = get_acc("phi",    "original_s42", 32)
        r4_d32_shift  = get_acc("phi_r4", "shift1_s42", 32)
        phi_d32_shift = get_acc("phi",    "shift1_s42", 32)
        if r4_d32_orig is None or phi_d32_orig is None:
            return "INCONCLUSIVE (data incomplete)"
        gap_reduced    = gap_r4 < gap_phi
        orig_improved  = r4_d32_orig  > phi_d32_orig  + 0.002
        shift_improved = (r4_d32_shift is not None and
                          phi_d32_shift is not None and
                          r4_d32_shift > phi_d32_shift + 0.002)
        dec_ok = not has_decodability_collapse()
        shift_worse = (r4_d32_shift is not None and phi_d32_shift is not None and
                       r4_d32_shift < phi_d32_shift - 0.002)
        if gap_reduced and orig_improved and shift_improved and dec_ok:
            return f"SUPPORTED: gap {gap_phi:.4f}→{gap_r4:.4f}, R^4 improves both orientations"
        elif not orig_improved and not shift_improved:
            return (f"NOT SUPPORTED: φ+R^4 does not improve over φ alone "
                    f"(gap {gap_phi:.4f}→{gap_r4:.4f})")
        elif not gap_reduced:
            return (f"NOT SUPPORTED: R^4 does not reduce orientation gap "
                    f"(gap {gap_phi:.4f}→{gap_r4:.4f}, WORSENED). "
                    f"Failure condition: φ+R^4 orientation gap ≥ φ alone.")
        elif shift_worse:
            return (f"NOT SUPPORTED: R^4 worsens shift1_s42 at D32 "
                    f"(phi_r4={r4_d32_shift:.4f} < phi={phi_d32_shift:.4f}). "
                    f"Failure condition: single-orientation only.")
        else:
            return "INCONCLUSIVE"

    # H3 support: non-monotonic performance vs D (requires optional dims)
    def h3_verdict() -> str:
        if not INCLUDE_OPTIONAL:
            return "NOT TESTED (INCLUDE_OPTIONAL=False; only D={24,32} run)"
        # Check if any ctrl shows non-monotonic behavior across D ∈ {20,24,28,32,36}
        for ctrl in ctrl_names:
            for orient in orient_names:
                vals = []
                for D in sorted(HORIZONS):
                    a = get_acc(ctrl, orient, D)
                    if a is not None:
                        vals.append((D, a))
                if len(vals) >= 4:
                    accs = [v[1] for v in vals]
                    # Non-monotonic: not always increasing and not always decreasing
                    diffs = [accs[i+1] - accs[i] for i in range(len(accs)-1)]
                    has_increase = any(d > 0.003 for d in diffs)
                    has_decrease = any(d < -0.003 for d in diffs)
                    if has_increase and has_decrease:
                        return (f"SUPPORTED: non-monotonic D-performance found for "
                                f"ctrl={ctrl}, orient={orient}: "
                                f"{[(d, round(a, 4)) for d, a in vals]}")
        return "NOT SUPPORTED: performance is monotonic or flat across tested D values"

    L: List[str] = []
    L.append("# Prime Transport: φ-R4-Dimension Alignment Probe v1\n\n")
    L.append(f"*Generated: {ts}*\n\n---\n\n")

    L.append("## Canonical Source Declaration\n\n")
    L.append("**CANONICAL** (source of truth for all reasoning):\n")
    L.append("- `prime_transport_router_start_agnostic_root_probe_v1.csv`\n")
    L.append("  - two_i_rot: D24=0.9922, D32=0.9873, Δ=−0.0049\n")
    L.append("  - Decodability=1.0 throughout all configs\n")
    L.append("- `prime_transport_router_controller_geometry_probe_v1.csv`\n")
    L.append("  - controller_projective: D24=0.9878, D32=0.9946, Δ=+0.0068\n")
    L.append("  - Best stack: two_i_rot anchor + projective controller + eps_high=1.0\n\n")
    L.append("**CONTRAST**: none (no other files used as truth)\n\n")

    L.append("## Pre-Code Geometry Lock (verified)\n\n")
    L.append("- 12-state cyclic dominant component: block [9:21] in tau0_oh (n_h=3)\n")
    L.append("- Partition: (argmax[9:21]) % 4 → interleaved [0,1,2,3,...]\n")
    L.append("- original_s42: labels = argmax % 4, seed=42\n")
    L.append("- shift1_s42:   labels = (argmax + 1) % 4, seed=42\n")
    L.append("- Partitions NOT redefined. Block structure NOT changed.\n")
    L.append("- GEOM_K3: [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]\n\n")

    L.append("## Experiment Design\n\n")
    L.append("### Fixed components (canonical best stack)\n")
    L.append("- Anchor: two_i_rot (rotate each (c,s) pair by +π/2)\n")
    L.append("- Transport: split, eps_high=1.0 (k≥2 frozen)\n")
    L.append("- D_HIDDEN=32, LR=0.02, BATCH=512, seed=42, MAX_STEPS=3500\n\n")

    L.append("### Controller variants\n\n")
    L.append("**1. ctrl_baseline** (`ctrl_mode=baseline`)\n")
    L.append("  - Standard projective: proj_k = sin_k / (1 + cos_k + 0.1)\n")
    L.append("  - Single reference frame — identical to controller_projective in prior probe\n")
    L.append(f"  - d_ctrl: D_EMB(4) + n_pairs(6) + n_blocks(4) = 14\n\n")
    L.append("**2. ctrl_phi** (`ctrl_mode=phi`)\n")
    L.append(f"  - φ = (1+√5)/2 ≈ {PHI:.6f}\n")
    n_pairs_geom3 = sum(n_h for _,_,_,n_h in GEOM_K3)
    phi_offs = make_phi_offsets(n_pairs_geom3)
    L.append(f"  - φ-offsets for n_pairs={n_pairs_geom3}: "
             f"{[round(o,4) for o in phi_offs]}\n")
    L.append(f"  - (NOT evenly spaced; successive differences are irrational)\n")
    L.append("  - Apply rotation by φ_k to each (c,s) pair, then compute projective\n")
    L.append(f"  - d_ctrl: D_EMB(4) + n_pairs(6) + n_blocks(4) = 14\n\n")
    L.append("**3. ctrl_phi_r4** (`ctrl_mode=phi_r4`)\n")
    L.append("  - Same φ-offsets + double-angle R^4 coupling:\n")
    L.append("    c2 = c'^2 - s'^2  (cos 2θ'),  s2 = 2c's'  (sin 2θ')\n")
    L.append("    r4_k = s2 / (1 + c2 + 0.1)\n")
    L.append("  - No learned parameters added — purely Fourier double-angle\n")
    L.append(f"  - d_ctrl: D_EMB(4) + n_pairs(6) + n_pairs(6) + n_blocks(4) = 20\n\n")

    L.append("### Task orientation variants\n")
    L.append("- original_s42: standard interleaved 4-class, seed=42\n")
    L.append("- shift1_s42: same partition shifted by +1 label (cyclic), seed=42\n\n")

    L.append(f"### Dimension sweep: D ∈ {HORIZONS}  "
             f"(INCLUDE_OPTIONAL={INCLUDE_OPTIONAL})\n\n")

    L.append("## Full Results Table\n\n")
    L.append("| Config | Ctrl | Orient | D | Accuracy | Solve | No-Last | "
             "Dec_i | Dec_m | Dec_f | α_last | RT(s) |\n")
    L.append("|--------|------|--------|---|----------|-------|---------|"
             "-------|-------|-------|--------|-------|\n")
    for r in csv_rows:
        solve_str = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['config']} | {r['ctrl_mode']} | {r['orientation']} "
                 f"| {r['horizon']} | **{r['accuracy']}** | {solve_str} "
                 f"| {r['no_last_accuracy']} | {r['decodability_initial']} "
                 f"| {r['decodability_mid']} | {r['decodability_final']} "
                 f"| {r['alpha_last']} | {r['runtime_seconds']} |\n")
    L.append("\n")

    L.append("## Δ(D32 − D24) Analysis\n\n")
    L.append("| Ctrl | Orient | D24 | D32 | Δ(D32−D24) |\n")
    L.append("|------|--------|-----|-----|------------|\n")
    L.append(f"| [REF] projective (ctrl_geometry) | original | {REF_PROJ_D24} "
             f"| {REF_PROJ_D32} | {REF_PROJ_DELTA:+.4f} |\n")
    for ctrl in ctrl_names:
        for orient in orient_names:
            a24 = get_acc(ctrl, orient, 24)
            a32 = get_acc(ctrl, orient, 32)
            d   = delta(ctrl, orient)
            a24s = f"{a24:.4f}" if a24 is not None else "?"
            a32s = f"{a32:.4f}" if a32 is not None else "?"
            ds   = f"{d:+.4f}" if d is not None else "?"
            L.append(f"| {ctrl} | {orient} | {a24s} | {a32s} | {ds} |\n")
    L.append("\n")

    L.append("## Orientation Gap Analysis\n\n")
    L.append("orientation_gap = |Δ_original − Δ_shift1|\n")
    L.append("A smaller gap means the controller is more robust to orientation.\n\n")
    L.append("| Ctrl | Δ_original | Δ_shift1 | Orientation Gap |\n")
    L.append("|------|------------|----------|-----------------|\n")
    base_gap = orientation_gap("baseline")
    for ctrl in ctrl_names:
        d_orig  = delta(ctrl, "original_s42")
        d_shift = delta(ctrl, "shift1_s42")
        gap     = orientation_gap(ctrl)
        d_orig_s  = f"{d_orig:+.4f}"  if d_orig  is not None else "?"
        d_shift_s = f"{d_shift:+.4f}" if d_shift is not None else "?"
        gap_s     = f"{gap:.4f}"      if gap     is not None else "?"
        vs_base   = ""
        if gap is not None and base_gap is not None and ctrl != "baseline":
            vs_base = f"  (Δvs_base: {gap - base_gap:+.4f})"
        L.append(f"| {ctrl} | {d_orig_s} | {d_shift_s} | {gap_s}{vs_base} |\n")
    L.append("\n")

    L.append("## Hypothesis Verdicts\n\n")

    L.append("### H0 (NULL): Current best stack is sufficient\n\n")
    # H0 criterion: "no φ-based or R^4 modification improves orientation robustness"
    # Orientation robustness is measured by orientation_gap (smaller = better).
    # H0 is SUPPORTED if both phi and phi_r4 have LARGER (or equal) gaps than baseline.
    gap_base   = orientation_gap("baseline")
    gap_phi    = orientation_gap("phi")
    gap_phi_r4 = orientation_gap("phi_r4")
    if (gap_base is not None and gap_phi is not None and gap_phi_r4 is not None and
            gap_phi >= gap_base and gap_phi_r4 >= gap_base):
        L.append(f"**SUPPORTED**: Neither φ nor φ+R^4 reduces the orientation gap vs baseline. "
                 f"baseline_gap={gap_base:.4f}, phi_gap={gap_phi:.4f} (+{gap_phi-gap_base:.4f}), "
                 f"phi_r4_gap={gap_phi_r4:.4f} (+{gap_phi_r4-gap_base:.4f}). "
                 f"Orientation robustness is NOT improved by either modification. "
                 f"Null hypothesis stands.\n\n")
    elif gap_base is not None and gap_phi is not None and gap_phi < gap_base:
        L.append(f"**NOT SUPPORTED**: φ reduces orientation gap "
                 f"({gap_base:.4f}→{gap_phi:.4f}). Null hypothesis rejected.\n\n")
    else:
        L.append("INCONCLUSIVE (data incomplete)\n\n")

    L.append("### H1 (PHI ANGLE): φ-based controller improves orientation robustness\n\n")
    L.append(f"Verdict: **{h1_verdict()}**\n\n")

    L.append("### H2 (PHI + R4): φ improvement only effective with R^4 coupling\n\n")
    L.append(f"Verdict: **{h2_verdict()}**\n\n")

    L.append("### H3 (DIMENSION ALIGNMENT): Non-monotonic performance vs D\n\n")
    L.append(f"Verdict: **{h3_verdict()}**\n\n")

    L.append("## Decodability Collapse Check\n\n")
    if has_decodability_collapse():
        L.append("**WARNING: decodability collapse detected** (final dec < 0.9). "
                 "Results for affected configs are unreliable.\n\n")
    else:
        L.append("No decodability collapse. All final decodability values ≥ 0.9.\n\n")

    L.append("## Failure Condition Audit\n\n")
    L.append("Per task spec, hypothesis is NOT supported if ANY of:\n\n")
    fc_items = []
    # FC1: φ does not outperform baseline globally (both D values, both orientations)
    # φ outperforms ONLY if it's better at D32 in both orientations AND
    # does not catastrophically degrade at D24.
    phi_beats_d32 = all(
        get_acc("phi", o, 32) is not None and
        get_acc("baseline", o, 32) is not None and
        get_acc("phi", o, 32) > get_acc("baseline", o, 32) + 0.002
        for o in orient_names
    )
    phi_collapses_d24 = any(
        get_acc("phi", o, 24) is not None and
        get_acc("baseline", o, 24) is not None and
        get_acc("phi", o, 24) < get_acc("baseline", o, 24) - 0.01
        for o in orient_names
    )
    if phi_beats_d32 and not phi_collapses_d24:
        fc1 = "PASS (φ outperforms baseline at D32, no D24 collapse)"
    elif phi_beats_d32 and phi_collapses_d24:
        fc1 = "FAIL (φ beats baseline at D32 but collapses at D24 — not robust)"
    else:
        fc1 = "FAIL (φ does not outperform baseline)"
    fc_items.append(f"1. φ outperforms baseline (both D, both orientations): **{fc1}**")

    # FC2: φ+R4 does not improve over φ alone (requires BOTH orientations AND gap reduction)
    r4_beats_phi_both = all(
        get_acc("phi_r4", o, 32) is not None and
        get_acc("phi",    o, 32) is not None and
        get_acc("phi_r4", o, 32) > get_acc("phi", o, 32) + 0.002
        for o in orient_names
    )
    r4_gap_better = (orientation_gap("phi_r4") is not None and
                     orientation_gap("phi") is not None and
                     orientation_gap("phi_r4") < orientation_gap("phi"))
    if r4_beats_phi_both and r4_gap_better:
        fc2 = "PASS (R^4 improves over φ in both orientations and reduces gap)"
    elif r4_beats_phi_both and not r4_gap_better:
        fc2 = "FAIL (R^4 beats φ at D32 in both orientations but gap is LARGER)"
    else:
        fc2 = "FAIL (R^4 does not improve over φ in both orientations)"
    fc_items.append(f"2. R^4 improves over φ (both orientations + gap): **{fc2}**")

    # FC3: monotonic D behavior
    fc_items.append(f"3. Non-monotonic D performance: **{h3_verdict()}**")

    # FC4: single-orientation only improvement (check phi at D32 vs baseline)
    both_orient_phi = (
        get_acc("phi", "original_s42", 32) is not None and
        get_acc("phi", "shift1_s42",   32) is not None and
        get_acc("baseline", "original_s42", 32) is not None and
        get_acc("baseline", "shift1_s42",   32) is not None and
        get_acc("phi", "original_s42", 32) > get_acc("baseline", "original_s42", 32) + 0.002 and
        get_acc("phi", "shift1_s42",   32) > get_acc("baseline", "shift1_s42",   32) + 0.002
        and not phi_collapses_d24
    )
    fc4 = "PASS (improvements in both orientations without D24 collapse)" if both_orient_phi else "FAIL (single-orientation or D24 collapse)"
    fc_items.append(f"4. Improvements in both orientations (no D24 collapse): **{fc4}**")

    for fc in fc_items:
        L.append(f"- {fc}\n")
    L.append("\n")

    L.append("## Limitations\n\n")
    L.append("- Single seed (42) — all results are point estimates\n")
    L.append("- Only GEOM_K3 tested\n")
    L.append("- φ-offsets use golden-angle Fibonacci distribution; other irrational "
             "angle sequences not tested\n")
    L.append("- R^4 coupling uses double-angle only; other geometry-native expansions "
             "(Hopf fibration embedding, outer-product cross-terms) not tested\n")
    L.append("- Orientation variants use cyclic shift of label assignment only; "
             "other orientation perturbations (rotation in state space) not tested\n")
    if not INCLUDE_OPTIONAL:
        L.append("- H3 not tested: only D ∈ {24, 32} run (INCLUDE_OPTIONAL=False)\n")
    L.append("\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"  MD  → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 72)
    print("PHI-R4-DIMENSION ALIGNMENT PROBE v1")
    print(f"  Horizons: {HORIZONS}  D_HIDDEN={D_HIDDEN}  seed={GLOBAL_SEED}")
    print(f"  Ctrl variants: {[c for _, c in CTRL_VARIANTS]}")
    print(f"  Orientations: {[o for o, _ in ORIENTATIONS]}")
    total_runs = len(CTRL_VARIANTS) * len(ORIENTATIONS) * len(HORIZONS)
    print(f"  Total runs: {total_runs}")
    print(f"  Anchor fixed: {ANCHOR_TYPE}")
    print(f"  INCLUDE_OPTIONAL: {INCLUDE_OPTIONAL}")
    print("=" * 72)

    # φ-offset diagnostic
    d_ang, n_pairs, _, _ = geom_dims(GEOM_K3)
    phi_offs = make_phi_offsets(n_pairs)
    print(f"\nφ-offsets (n_pairs={n_pairs}):")
    for k, o in enumerate(phi_offs):
        print(f"  k={k}: {o:.6f} rad  ({o/(2*math.pi):.6f} × 2π)")
    all_same_spacing = all(
        abs((phi_offs[(k+1)%n_pairs] - phi_offs[k]) - (phi_offs[1] - phi_offs[0])) < 1e-10
        for k in range(n_pairs)
    )
    print(f"  Evenly spaced: {all_same_spacing} (must be False)\n")

    if not CACHE_PATH.exists():
        print(f"ERROR: state cache not found at {CACHE_PATH}")
        sys.exit(1)

    print(f"Loading state cache: {CACHE_PATH}")
    cache = torch.load(CACHE_PATH, map_location="cpu", weights_only=True)
    TN_oh    = cache["TN_oh"]
    tau0_oh  = cache["tau0_oh"]
    TR       = cache["TR"]
    pool_ids = cache["pool_ids"]

    # Prepare angular tables (anchor=two_i_rot, shared across all orientations/ctrls)
    TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, GEOM_K3
    )
    print(f"  tau0_hyb.shape={tau0_hyb.shape}  TN_ang.shape={TN_ang.shape}")

    # Build class tables for each orientation variant
    class_tables: Dict[str, torch.Tensor] = {}
    for orient_name, shift in ORIENTATIONS:
        ct = build_class_table(tau0_oh, shift=shift)
        class_tables[orient_name] = ct
        dist = [(ct == i).sum().item() for i in range(VOCAB)]
        print(f"  {orient_name}: class dist={dist}")

    csv_rows: List[Dict] = []

    for D in HORIZONS:
        print(f"\n{'='*72}\n  HORIZON D={D}\n{'='*72}")
        for ctrl_name, ctrl_mode in CTRL_VARIANTS:
            for orient_name, _ in ORIENTATIONS:
                mod12_classes = class_tables[orient_name]
                run_label = f"{ctrl_name}_{orient_name}"

                result = train_one(
                    run_label, ctrl_mode, orient_name, D,
                    TN_ang, TR_p, tau0_hyb, pool_ids_p, mod12_classes, GEOM_K3,
                )

                model      = result["model"]
                pool_ids_  = result["pool_ids"]
                mod12_cls_ = result["mod12_classes"]

                print("    Running decodability probes...")
                dec = run_decodability_probes(model, pool_ids_, mod12_cls_)

                print("    Collecting trajectories for no-last eval...")
                all_st, all_x0 = collect_trajectories(model, pool_ids_, mod12_cls_)
                no_last = eval_no_last(model, all_st, all_x0)

                note = (f"task=mod12_quarter_D{D}_no_inject; D={D}; "
                        f"anchor={ANCHOR_TYPE}; ctrl_mode={ctrl_mode}; "
                        f"orientation={orient_name}; eps_high=1.0; geom=GEOM_K3")

                csv_rows.append({
                    "config":                run_label,
                    "ctrl_mode":             ctrl_mode,
                    "orientation":           orient_name,
                    "horizon":               D,
                    "accuracy":              result["final_acc"],
                    "solve_step":            result["solve_step"] or "",
                    "no_last_accuracy":      no_last,
                    "decodability_initial":  dec["initial"],
                    "decodability_mid":      dec["mid"],
                    "decodability_final":    dec["final"],
                    "alpha_last":            result["alphaD"],
                    "runtime_seconds":       result["wall"],
                    "note":                  note,
                })

                # Write CSV incrementally
                write_csv(csv_rows)

    write_markdown(csv_rows)
    print("\n" + "=" * 72)
    print("DONE")
    print("=" * 72)


if __name__ == "__main__":
    main()
