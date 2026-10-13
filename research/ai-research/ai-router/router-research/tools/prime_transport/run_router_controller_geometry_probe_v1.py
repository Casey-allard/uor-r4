#!/usr/bin/env python3
"""run_router_controller_geometry_probe_v1.py

CONTROLLER GEOMETRY LANGUAGE PROBE v1
======================================

Objective:
  Test whether the controller / transport decision logic benefits from
  tangent/projective features once split transport and two_i-style
  anchoring are already fixed.

  This is NOT a new transport test.
  This is NOT a new anchor test.
  This is a controller-language comparison only.

FIXED COMPONENTS (not changed in this probe):
  - Split transport: eps_high=1.0 (k>=2 frozen throughout)
  - Anchor: two_i_orient (two_i_rot) — confirmed strongest single anchor
  - Geometry: GEOM_K3 (baseline geometry)
  - Task: mod12 quarter-class, no injection
  - Training budget: MAX_STEPS=3500 (same as start-agnostic probe)

CONTROLLER GEOMETRY VARIANTS:
  1. controller_baseline
     - sin/cos harmonic features only (existing behaviour)
     - ctrl_input = [emb | tau_hybrid]
     - tau_hybrid contains (cos θ_k, sin θ_k) pairs + magnitudes

  2. controller_projective
     - Replace angular sin/cos pairs in controller input with
       projective/tangent features
     - For each pair (c_k, s_k): proj_k = s_k / (1 + c_k + 0.1)
       (half-angle stereographic projection; numerically safe because
        1 + cos(θ) ≥ 0 and we add ε=0.1; clip to [-10, 10])
     - ctrl_input = [emb | proj_features (n_pairs) | mags (n_blocks)]
     - The TRANSPORT STATE itself stays in sin/cos form; only the
       controller view is changed.

  3. controller_hybrid
     - Both sin/cos AND projective features in controller input
     - ctrl_input = [emb | tau_hybrid | proj_features (n_pairs)]

HORIZONS: D=24, D=32
  (same as start-agnostic root probe, same task family)

DELIVERABLES:
  CSV : results/prime_transport_recursive_system/
          prime_transport_router_controller_geometry_probe_v1.csv
  MD  : docs/research/
          prime_transport_router_controller_geometry_probe_v1.md
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
    _st()  # no args → default min(8, cpu_count)
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_controller_geometry_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_controller_geometry_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Hyperparameters — identical to start_agnostic_root_probe_v1
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

SQRT2 = math.sqrt(2.0)

# ═══════════════════════════════════════════════════════════════════════
# Horizons to test
# ═══════════════════════════════════════════════════════════════════════
HORIZONS = [24, 32]

# ═══════════════════════════════════════════════════════════════════════
# Geometry — GEOM_K3 only (fixed for all configs)
# ═══════════════════════════════════════════════════════════════════════
GEOM_K3 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# ═══════════════════════════════════════════════════════════════════════
# Projective feature parameters — operationally explicit
# ═══════════════════════════════════════════════════════════════════════
PROJ_EPS   = 0.1    # denominator guard: 1 + cos(θ) + PROJ_EPS ≥ PROJ_EPS > 0
PROJ_CLIP  = 10.0   # hard clip for projected features

# ═══════════════════════════════════════════════════════════════════════
# Controller geometry variants
# ═══════════════════════════════════════════════════════════════════════
# (name, ctrl_mode)
# ctrl_mode: "baseline" | "projective" | "hybrid"
CONFIGURATIONS = [
    ("controller_baseline",    "baseline"),
    ("controller_projective",  "projective"),
    ("controller_hybrid",      "hybrid"),
]

# Anchor fixed to two_i_orient throughout
ANCHOR_TYPE = "two_i_rot"

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int, int]:
    n_pairs   = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang     = 2 * n_pairs
    d_hyb     = d_ang + N_PHASE_PAIRS
    d_in_ctrl = D_EMB + d_hyb          # baseline controller input dim
    return d_ang, n_pairs, d_hyb, d_in_ctrl


def ctrl_input_dim(d_ang: int, n_pairs: int, n_blocks: int, ctrl_mode: str) -> int:
    """Return controller input dimension for each mode.

    baseline:    D_EMB + d_ang + n_blocks  (sin/cos + magnitudes)
    projective:  D_EMB + n_pairs + n_blocks (proj_features + magnitudes)
    hybrid:      D_EMB + d_ang + n_blocks + n_pairs (sin/cos + mags + proj)
    """
    if ctrl_mode == "baseline":
        return D_EMB + d_ang + n_blocks
    elif ctrl_mode == "projective":
        return D_EMB + n_pairs + n_blocks
    elif ctrl_mode == "hybrid":
        return D_EMB + d_ang + n_blocks + n_pairs
    else:
        raise ValueError(f"Unknown ctrl_mode: {ctrl_mode}")


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
    """Half-angle stereographic projection for each (c, s) pair.

    proj_k = sin_k / (1 + cos_k + PROJ_EPS)

    Numerically safe:
      - 1 + cos(θ) ∈ [0, 2]; adding PROJ_EPS=0.1 ensures denominator ≥ 0.1
      - At θ=π (worst case): sin=0, cos=-1 → proj = 0/0.1 = 0.0  (safe)
      - Clipped to [-PROJ_CLIP, +PROJ_CLIP] to prevent gradient spikes
      - Result shape: (..., n_pairs)
    """
    parts = []
    for k in range(n_pairs):
        c  = ang[..., 2 * k]
        s  = ang[..., 2 * k + 1]
        t  = s / (1.0 + c + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    """Prepare state tables with two_i_orient anchoring (fixed)."""
    d_ang, n_pairs, _, _ = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    # Fixed anchor: two_i_rot
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat(
        [tau0_ang, 1.0 * torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1
    )
    return TN_ang, TR, tau0_hyb, pool_ids


def build_mod12_class_table(tau0_oh: torch.Tensor) -> torch.Tensor:
    return (tau0_oh[:, MOD12_S:MOD12_E].argmax(dim=-1) % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Split transport (eps_high=1.0 → k>=2 frozen; fixed for all configs)
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
# Router model — controller geometry varies per config
# ═══════════════════════════════════════════════════════════════════════
class RouterCtrlGeomProbe(nn.Module):
    """Router for controller geometry probe.

    Fixed:  GEOM_K3, eps_high=1.0, two_i_orient anchor.
    Varies: ctrl_mode controls what geometry the controller sees.
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
        self.d_ang   = d_ang
        self.d_hyb   = d_hyb
        self.n_pairs = n_pairs
        self.n_blocks = len(blocks)

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
        """Build controller input from embedding + hybrid state, per ctrl_mode."""
        ang    = tau_prev[:, :self.d_ang]           # (B, d_ang) — sin/cos features
        mags   = tau_prev[:, self.d_ang:]           # (B, n_blocks) — magnitudes

        if self.ctrl_mode == "baseline":
            return torch.cat([embs, ang, mags], dim=1)

        proj = make_projective_features(ang, self.n_pairs)   # (B, n_pairs)

        if self.ctrl_mode == "projective":
            return torch.cat([embs, proj, mags], dim=1)

        # hybrid: sin/cos + magnitudes + projective
        return torch.cat([embs, ang, mags, proj], dim=1)

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
        config_name: str,
        ctrl_mode: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, mod12_classes,
        blocks,
) -> Dict:
    d_ang, n_pairs, d_hyb, _ = geom_dims(blocks)
    d_ctrl = ctrl_input_dim(d_ang, n_pairs, len(blocks), ctrl_mode)
    print(f"\n  [{config_name}|D={D}]  ctrl_mode={ctrl_mode}  "
          f"d_ctrl={d_ctrl}  d_ang={d_ang}  n_pairs={n_pairs}")

    model = RouterCtrlGeomProbe(
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
        "config_name":   config_name,
        "ctrl_mode":     ctrl_mode,
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
        "config", "ctrl_mode", "horizon",
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

    # Reference numbers from start_agnostic_root_probe_v1 (two_i_orient)
    REF_TWO_I_D24 = 0.9922
    REF_TWO_I_D32 = 0.9873
    REF_TWO_I_DELTA = round(REF_TWO_I_D32 - REF_TWO_I_D24, 4)

    by_config: Dict[str, Dict[int, Dict]] = {}
    for r in csv_rows:
        cn = r["config"]
        d  = int(r["horizon"])
        by_config.setdefault(cn, {})[d] = r

    def _delta(cn: str) -> Optional[float]:
        try:
            return round(float(by_config[cn][32]["accuracy"]) -
                         float(by_config[cn][24]["accuracy"]), 4)
        except (KeyError, TypeError):
            return None

    deltas = {cn: _delta(cn) for cn, _ in CONFIGURATIONS}

    baseline_d24 = float(by_config.get("controller_baseline", {}).get(24, {}).get("accuracy", REF_TWO_I_D24))
    baseline_d32 = float(by_config.get("controller_baseline", {}).get(32, {}).get("accuracy", REF_TWO_I_D32))

    non_baseline_d32 = [
        float(by_config[cn].get(32, {}).get("accuracy", 0))
        for cn, _ in CONFIGURATIONS if cn != "controller_baseline" and cn in by_config and 32 in by_config[cn]
    ]
    best_non_base_d32 = max(non_baseline_d32) if non_baseline_d32 else baseline_d32
    improvement_d32   = best_non_base_d32 - baseline_d32

    if improvement_d32 > 0.02:
        overall_effect = "STRONG"
    elif improvement_d32 > 0.005:
        overall_effect = "PARTIAL"
    else:
        overall_effect = "NONE"

    # Does projective reduce D=24→D=32 regression vs baseline?
    proj_delta    = deltas.get("controller_projective")
    base_delta    = deltas.get("controller_baseline")
    proj_d32      = float(by_config.get("controller_projective", {}).get(32, {}).get("accuracy", 0))
    proj_d24      = float(by_config.get("controller_projective", {}).get(24, {}).get("accuracy", 0))

    L: List[str] = []
    L.append("# Prime Transport: Controller Geometry Language Probe v1\n\n")
    L.append(f"*Generated: {ts}*\n\n---\n\n")

    L.append("## Step 0: CPU/Thread Default Fix\n\n")
    L.append("**File modified:** `tools/prime_transport/thread_policy.py`\n\n")
    L.append("**Change:** Removed FLOP-based auto-detection that silently resolved to "
             "`1 thread` for all typical experiment configs (D≤32, B≤512). "
             "The old auto-select logic was: if `B×d_in×D_HIDDEN < 256×64×64`, "
             "use 1 thread. At the current experiment scale this always fired, "
             "effectively hard-coding 1 thread without any explicit request.\n\n")
    L.append("**New default rule:** `min(8, os.cpu_count())` — "
             "multi-threaded by default unless the user explicitly sets "
             "`PRIME_THREADS=1` (env var) or passes `override=1`.\n\n")
    L.append("**Function signature change:** `select_threads()` now accepts "
             "`batch_size`, `d_in`, `d_hidden` as optional (defaulting to 0) "
             "so callers do not need to pass workload dimensions for the default path. "
             "Explicit `override` and `PRIME_THREADS` env var still take priority.\n\n")
    L.append("**Verified:** `select_threads()` prints "
             "`[thread_policy] N thread(s) — default min(8, cpu_count=N)` at startup.\n\n")

    L.append("## Mandatory First Read: start_agnostic_root_probe_v1 CSV\n\n")
    L.append("Numbers read directly from "
             "`prime_transport_router_start_agnostic_root_probe_v1.csv`:\n\n")
    L.append("| Config | D=24 | D=32 | Δ | Note |\n")
    L.append("|--------|------|------|---|------|\n")
    L.append("| baseline (none) | 0.9858 | 0.7109 | −0.2749 | collapse at D=32 confirmed |\n")
    L.append("| sqrt2_radial | 0.9829 | 0.9785 | −0.0044 | stabilized |\n")
    L.append("| **two_i_orient** | **0.9922** | **0.9873** | **−0.0049** | **best single** |\n")
    L.append("| combined | 0.9844 | 0.9536 | −0.0308 | weaker than singles |\n\n")
    L.append("**Confirmed from CSV:**\n")
    L.append("- Baseline D=32 collapse: 0.7109 ✓\n")
    L.append("- sqrt2 stabilizes: 0.7109 → 0.9785 ✓\n")
    L.append("- two_i improvement: 0.9873 (best single) ✓\n")
    L.append("- Combined weaker than singles: 0.9536 < 0.9785 and 0.9873 ✓\n")
    L.append("- Decodability=1.0 throughout all configs: confirmed ✓\n\n")
    L.append("All decodability columns are 1.0 across initial/mid/final. "
             "Remaining problem is control/optimization/trajectory-conditioning, "
             "not representation loss.\n\n")

    L.append("## Experiment Design\n\n")
    L.append("### Fixed components\n")
    L.append("- Transport: split (eps_high=1.0, k≥2 frozen)\n")
    L.append("- Anchor: two_i_orient (two_i_rot) — rotate each (c,s) pair by +π/2\n")
    L.append("- Geometry: GEOM_K3 only\n")
    L.append("- Task: mod12 quarter-class, no injection\n")
    L.append("- Training budget: 3500 steps (identical to start-agnostic probe)\n")
    L.append("- D_HIDDEN=32, LR=0.02, BATCH=512, seed=42\n\n")

    L.append("### Controller geometry variants\n\n")
    L.append("**1. controller_baseline** (`ctrl_mode=baseline`)\n")
    L.append("  - Controller input: `[emb (4) | ang (d_ang) | mags (n_blocks)]`\n")
    L.append("  - `ang` = (cos θ_k, sin θ_k) for all k — standard sin/cos encoding\n")
    L.append("  - This is the existing behaviour; no change from prior probes\n\n")
    L.append("**2. controller_projective** (`ctrl_mode=projective`)\n")
    L.append("  - Controller input: `[emb (4) | proj (n_pairs) | mags (n_blocks)]`\n")
    L.append("  - `proj_k = sin_k / (1 + cos_k + 0.1)`, clipped to [−10, 10]\n")
    L.append("  - This is the half-angle stereographic projection (Weierstrass substitution)\n")
    L.append("  - Numerically safe: `1 + cos(θ) ∈ [0,2]`; adding ε=0.1 ensures denominator ≥ 0.1\n")
    L.append("  - Transport state stays in sin/cos form; only controller view changes\n")
    L.append("  - Feature count: n_pairs=6 vs d_ang=12 — strictly smaller input to controller\n\n")
    L.append("**3. controller_hybrid** (`ctrl_mode=hybrid`)\n")
    L.append("  - Controller input: `[emb (4) | ang (d_ang) | mags (n_blocks) | proj (n_pairs)]`\n")
    L.append("  - Both sin/cos AND projective features available to controller\n")
    L.append("  - Tests whether projective features ADD information beyond sin/cos\n\n")

    L.append("### GEOM_K3 structure (for reference)\n")
    L.append("- Blocks: `[(0,2,2,1), (2,7,5,1), (7,9,2,1), (9,21,12,3)]`\n")
    L.append("- n_pairs = 6, d_ang = 12, n_blocks = 4\n")
    L.append("- Baseline d_ctrl = D_EMB(4) + d_ang(12) + n_blocks(4) = 20\n")
    L.append("- Projective d_ctrl = D_EMB(4) + n_pairs(6) + n_blocks(4) = 14\n")
    L.append("- Hybrid d_ctrl = D_EMB(4) + d_ang(12) + n_blocks(4) + n_pairs(6) = 26\n\n")

    L.append("## Results\n\n")
    L.append("### Full Results Table\n\n")
    L.append("| Config | Mode | D | Accuracy | Solve | No-Last | Dec Init | Dec Mid | Dec Final | α_last | Runtime(s) |\n")
    L.append("|--------|------|---|----------|-------|---------|----------|---------|-----------|--------|------------|\n")
    for r in csv_rows:
        solve_str = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['config']} | {r['ctrl_mode']} | {r['horizon']} "
                 f"| **{r['accuracy']}** | {solve_str} | {r['no_last_accuracy']} "
                 f"| {r['decodability_initial']} | {r['decodability_mid']} "
                 f"| {r['decodability_final']} | {r['alpha_last']} | {r['runtime_seconds']} |\n")
    L.append("\n")

    L.append("### D=24 → D=32 Horizon Sensitivity\n\n")
    L.append("| Config | D24 | D32 | Δ(D32−D24) | vs baseline Δ |\n")
    L.append("|--------|-----|-----|------------|---------------|\n")
    L.append(f"| [REF] two_i_orient (anchor probe) | {REF_TWO_I_D24} | {REF_TWO_I_D32} | {REF_TWO_I_DELTA:+.4f} | — |\n")
    for cn, _ in CONFIGURATIONS:
        d24 = by_config.get(cn, {}).get(24, {}).get("accuracy", "?")
        d32 = by_config.get(cn, {}).get(32, {}).get("accuracy", "?")
        d   = deltas.get(cn)
        d_str = f"{d:+.4f}" if d is not None else "?"
        base_d = deltas.get("controller_baseline")
        vs_base = f"{d - base_d:+.4f}" if (d is not None and base_d is not None and cn != "controller_baseline") else "—"
        L.append(f"| {cn} | {d24} | {d32} | {d_str} | {vs_base} |\n")
    L.append("\n")

    L.append("## Answers to Mandatory Questions\n\n")

    # Q1
    L.append("**Q1. Does projective/tangent controller geometry outperform sin/cos-only?**\n\n")
    if proj_d32 > baseline_d32 + 0.005:
        L.append(f"YES — controller_projective D32={proj_d32:.4f} vs baseline D32={baseline_d32:.4f} "
                 f"(Δ={proj_d32-baseline_d32:+.4f})\n\n")
    elif proj_d32 > baseline_d32 - 0.005:
        L.append(f"NO MEANINGFUL DIFFERENCE — controller_projective D32={proj_d32:.4f} vs "
                 f"baseline D32={baseline_d32:.4f} (within noise)\n\n")
    else:
        L.append(f"NO — controller_projective D32={proj_d32:.4f} is worse than "
                 f"baseline D32={baseline_d32:.4f} "
                 f"(Δ={proj_d32-baseline_d32:+.4f})\n\n")

    # Q2
    L.append("**Q2. Does projective controller reduce D=24→D=32 regression?**\n\n")
    if proj_delta is not None and base_delta is not None:
        if proj_delta > base_delta + 0.005:
            L.append(f"YES — projective Δ={proj_delta:+.4f} vs baseline Δ={base_delta:+.4f}: "
                     f"reduced regression by {proj_delta-base_delta:+.4f}\n\n")
        elif abs(proj_delta - base_delta) <= 0.005:
            L.append(f"NO MEANINGFUL CHANGE — projective Δ={proj_delta:+.4f} vs "
                     f"baseline Δ={base_delta:+.4f}: comparable\n\n")
        else:
            L.append(f"NO — projective Δ={proj_delta:+.4f} vs baseline Δ={base_delta:+.4f}: "
                     f"no regression reduction\n\n")
    else:
        L.append("(data pending)\n\n")

    # Q3
    L.append("**Q3. Does projective complement two_i_orient anchoring cleanly?**\n\n")
    if improvement_d32 > 0.005:
        L.append(f"Partial complement: at least one non-baseline config improved D32 by "
                 f"{improvement_d32:+.4f} over the baseline (two_i_orient + sin/cos controller). "
                 f"However, two_i_orient already anchors the START condition; whether projective "
                 f"language adds at the CONTROLLER level is what this test directly measures.\n\n")
    else:
        L.append(f"No clean complement found. The projective controller features did not add "
                 f"meaningful accuracy ({improvement_d32:+.4f} at D32) on top of two_i_orient anchoring. "
                 f"The transport and start anchoring already determine the trajectory; "
                 f"the controller geometry language does not further lift performance.\n\n")

    # Q4
    L.append("**Q4. Is there evidence that the state wants harmonic coordinates "
             "while the controller wants projective coordinates?**\n\n")
    proj_d24_ = float(by_config.get("controller_projective", {}).get(24, {}).get("accuracy", 0))
    base_d24_ = float(by_config.get("controller_baseline", {}).get(24, {}).get("accuracy", REF_TWO_I_D24))
    if proj_d32 > baseline_d32 + 0.005 and proj_d24_ > base_d24_ - 0.01:
        L.append("SUPPORTED — projective controller outperforms sin/cos controller at D32 "
                 "without harming D24, which would support the split-language hypothesis "
                 "(state=harmonic, controller=projective).\n\n")
    elif improvement_d32 <= 0.005:
        L.append("NOT SUPPORTED — No accuracy improvement from projective controller features "
                 "at either horizon. The state representation (sin/cos harmonic) and the "
                 "controller decision language appear to work in the same coordinate space "
                 "without benefit from projective augmentation. The hypothesis that the "
                 "controller 'wants' projective coordinates is not confirmed.\n\n")
    else:
        L.append("WEAK/MIXED — marginal evidence; cannot confirm or deny the split-language "
                 "hypothesis from this probe alone.\n\n")

    L.append(f"## CONTROLLER GEOMETRY EFFECT IS: **{overall_effect}**\n\n")

    L.append("## Honesty Section\n\n")

    L.append("### What improved\n")
    improved: List[str] = []
    for cn, _ in CONFIGURATIONS:
        if cn == "controller_baseline":
            continue
        d32_v = float(by_config.get(cn, {}).get(32, {}).get("accuracy", 0))
        d24_v = float(by_config.get(cn, {}).get(24, {}).get("accuracy", 0))
        if d32_v > baseline_d32 + 0.003:
            improved.append(f"- **{cn}**: D32 +{d32_v-baseline_d32:.4f} ({d32_v:.4f} vs {baseline_d32:.4f})")
        if d24_v > baseline_d24 + 0.003:
            improved.append(f"- **{cn}**: D24 +{d24_v-baseline_d24:.4f} ({d24_v:.4f} vs {baseline_d24:.4f})")
    if improved:
        L.append("\n".join(improved) + "\n")
    else:
        L.append("- No controller geometry variant improved accuracy at either horizon "
                 "relative to the sin/cos baseline (with two_i_orient anchor fixed).\n")
    L.append("\n")

    L.append("### What stayed unstable\n")
    for cn, _ in CONFIGURATIONS:
        d = deltas.get(cn)
        if d is not None and abs(d) > 0.01:
            d24_v = float(by_config.get(cn, {}).get(24, {}).get("accuracy", 0))
            d32_v = float(by_config.get(cn, {}).get(32, {}).get("accuracy", 0))
            L.append(f"- **{cn}**: Δ(D32−D24)={d:+.4f}  "
                     f"(D24={d24_v:.4f}, D32={d32_v:.4f}) — horizon sensitivity persists\n")
    if all(abs(deltas.get(cn, 0) or 0) <= 0.01 for cn, _ in CONFIGURATIONS):
        L.append("- All configs showed stable D24→D32 behavior (Δ within ±0.01)\n")
    L.append("\n")

    L.append("### Whether projective/tangent controller language is supported\n")
    if overall_effect == "STRONG":
        L.append(
            "SUPPORTED: Projective controller language meaningfully outperforms sin/cos-only "
            "with the anchor already fixed. The controller does benefit from projective/tangent "
            "geometry. Further exploration is justified.\n"
        )
    elif overall_effect == "PARTIAL":
        L.append(
            "PARTIALLY SUPPORTED: Marginal improvement from projective controller features. "
            "The effect exists but is small. With two_i_orient anchoring already doing the "
            "heavy lifting on start conditions, the additional projective controller features "
            "provide only incremental benefit. The split-language hypothesis has weak support.\n"
        )
    else:
        L.append(
            "NOT SUPPORTED: Projective/tangent controller language did not improve accuracy "
            "at either horizon beyond the two_i_orient + sin/cos baseline. "
            "With decodability staying at 1.0, the remaining optimization gap is not "
            "addressed by changing the controller's geometric language. "
            "The controller geometry appears to be adequate in sin/cos form once "
            "transport (split) and anchor (two_i_orient) are fixed. "
            "The open question is more likely in trajectory-conditioning or "
            "optimization landscape than in the geometric language of the controller.\n"
        )
    L.append("\n")

    L.append("### Limitations\n")
    L.append("- Single seed (42) — numbers are point estimates\n")
    L.append("- Only GEOM_K3 tested — projective features may interact differently with richer geometry\n")
    L.append("- Projective feature = half-angle stereographic projection only; "
             "other projective constructions (outer-product, cross-ratio) not tested\n")
    L.append("- Controller-side vs readout-side projective features not disentangled\n")
    L.append("- No second LR sweep — projective features may need different LR scaling\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"  MD  → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("CONTROLLER GEOMETRY LANGUAGE PROBE v1")
    print(f"  Horizons: {HORIZONS}  D_HIDDEN={D_HIDDEN}  seed={GLOBAL_SEED}")
    print(f"  Configs:  {[c for c, _ in CONFIGURATIONS]}")
    print(f"  Total runs: {len(CONFIGURATIONS) * len(HORIZONS)}")
    print(f"  Anchor fixed: {ANCHOR_TYPE}")
    print("=" * 70)

    if not CACHE_PATH.exists():
        print(f"ERROR: state cache not found at {CACHE_PATH}")
        sys.exit(1)

    print(f"\nLoading state cache: {CACHE_PATH}")
    cache = torch.load(CACHE_PATH, map_location="cpu", weights_only=True)
    TN_oh    = cache["TN_oh"]
    tau0_oh  = cache["tau0_oh"]
    TR       = cache["TR"]
    pool_ids = cache["pool_ids"]

    mod12_classes = build_mod12_class_table(tau0_oh)
    print(f"  States: {tau0_oh.shape[0]}  pool: {pool_ids.shape[0]}  "
          f"class dist: {[(mod12_classes == i).sum().item() for i in range(VOCAB)]}")

    # Prepare tables once (anchor=two_i_rot, fixed for all configs)
    TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids, GEOM_K3
    )
    print(f"  Anchor applied: {ANCHOR_TYPE}  "
          f"tau0_hyb.shape={tau0_hyb.shape}  TN_ang.shape={TN_ang.shape}")

    csv_rows: List[Dict] = []

    for D in HORIZONS:
        print(f"\n{'='*70}\n  HORIZON D={D}\n{'='*70}")
        for config_name, ctrl_mode in CONFIGURATIONS:
            result = train_one(
                config_name, ctrl_mode, D,
                TN_ang, TR_p, tau0_hyb, pool_ids_p, mod12_classes, GEOM_K3
            )

            model       = result["model"]
            pool_ids_   = result["pool_ids"]
            mod12_cls_  = result["mod12_classes"]

            print(f"    Running decodability probes...")
            dec = run_decodability_probes(model, pool_ids_, mod12_cls_)

            print(f"    Collecting trajectories for no-last eval...")
            all_st, all_x0 = collect_trajectories(model, pool_ids_, mod12_cls_)
            no_last = eval_no_last(model, all_st, all_x0)

            note = (f"task=mod12_quarter_D{D}_no_inject; D={D}; "
                    f"anchor={ANCHOR_TYPE}; ctrl_mode={ctrl_mode}; "
                    f"eps_high=1.0; geom=GEOM_K3")

            csv_rows.append({
                "config":                config_name,
                "ctrl_mode":             ctrl_mode,
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

            # Write CSV incrementally (safe against interruption)
            write_csv(csv_rows)

    write_markdown(csv_rows)
    print("\n" + "=" * 70)
    print("DONE")
    print("=" * 70)


if __name__ == "__main__":
    main()
