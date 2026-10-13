#!/usr/bin/env python3
"""run_router_dimension_phase_alignment_probe_v1.py

DIMENSION–PHASE ALIGNMENT PROBE v1
=====================================

BRANCH: dimension_phase_alignment_probe_v1

PRE-CODE GEOMETRY LOCK (completed before implementation):
  1. Task geometry: composite cyclic state space with BLOCKS_A [(0,2,2,1),(2,7,5,1),
     (7,9,2,1),(9,21,12,3)]. Dominant component: positions 9..21 (period 12, 3 harmonics).
  2. Class partition: INTERLEAVED. PARTITION_ORIGINAL=[0,1,2,3,0,1,2,3,0,1,2,3].
     class 0={0,4,8}, class 1={1,5,9}, class 2={2,6,10}, class 3={3,7,11}.
     Confirmed: NOT contiguous.
  3. Cyclic phase structure: period-12 fundamental; 4 equal interleaved sub-groups of 3.
     3rd harmonic (k=3) carries discriminating signal. Split transport: k=1 free, k≥2 frozen.
  4. Same geometry across ALL D values: same cache, same BLOCKS_A, same partition.
     D only governs sequence horizon; D_HIDDEN=32 (network width) locked throughout.

CANONICAL FILES (ground truth):
  - prime_transport_router_start_agnostic_root_probe_v1.csv
  - prime_transport_router_controller_geometry_probe_v1.csv
  - router_locked_stack_failure_isolation_probe_v1.csv

CONTRAST FILE (for orientation gap reference only, NOT for system definition):
  - router_phi_r4_alignment_probe_v1.csv

LOCKED STACK (DO NOT MODIFY):
  1. Split transport:    eps_high=1.0 (higher harmonics frozen at previous state)
  2. State:             harmonic sin/cos encoding (decodability must stay 1.0)
  3. Start anchor:      two_i_orient: rotate each (c,s)→(−s,c) per pair
  4. Controller:        projective: proj_k = sin_k/(1+cos_k+0.1), clipped ±10

OBJECTIVE:
  Test whether embedding dimension D (sequence horizon) shows resonance-like or
  alignment-like behavior under the locked stack. D_HIDDEN=32 is fixed throughout.
  The hypothesis is that certain D values align better with the cyclic phase
  structure (period 12, interleaved 4-class partition).

DESIGN:
  - Tasks: original_s42 and shift1_s42 ONLY (no noise, no Task C)
  - D ∈ {16, 20, 24, 28, 32, 36, 40}
  - All other hyperparameters identical to the locked stack failure isolation probe

DELIVERABLES:
  PY  : tools/prime_transport/run_router_dimension_phase_alignment_probe_v1.py
  CSV : results/prime_transport_recursive_system/router_dimension_phase_alignment_probe_v1.csv
  MD  : docs/research/prime_transport_dimension_phase_alignment_probe_v1.md
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
# CPU/thread fix
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
    print(f"  [thread_policy] {_n} thread(s)")

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "router_dimension_phase_alignment_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_dimension_phase_alignment_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Authoritative reference values (hard-coded from canonical CSVs)
# ═══════════════════════════════════════════════════════════════════════
# File (1): prime_transport_router_start_agnostic_root_probe_v1.csv
CANON_TWO_I_D24   = 0.9922
CANON_TWO_I_D32   = 0.9873
# File (2): prime_transport_router_controller_geometry_probe_v1.csv
CANON_PROJ_D24    = 0.9878
CANON_PROJ_D32    = 0.9946
# File (3): router_locked_stack_failure_isolation_probe_v1.csv
# Task A locked stack (projective, two_i_orient)
CANON_LS_ORIG_D24 = 0.9941   # original_s42, D=24
CANON_LS_ORIG_D32 = 0.9653   # original_s42, D=32
CANON_LS_SH1_D24  = 0.9570   # shift1_s42,   D=24
CANON_LS_SH1_D32  = 0.9829   # shift1_s42,   D=32
# Orientation gaps from file (3) at D=24 and D=32:
#   gap_D24 = original - shift1 = 0.9941 - 0.9570 = +0.0371
#   gap_D32 = original - shift1 = 0.9653 - 0.9829 = -0.0176
# These two known points ground the orientation-gap characterisation.

# ═══════════════════════════════════════════════════════════════════════
# Hyperparameters — identical to locked stack failure isolation probe
# (DO NOT MODIFY)
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED = 42
D_HIDDEN    = 32   # network width — LOCKED, does NOT vary with horizon D
BATCH_SIZE  = 512
VOCAB       = 4
D_EMB       = 4
N_OPS       = 6
LR          = 0.02
TEMP_START  = 2.0
TEMP_END    = 0.1
CLIP_GRAD   = 1.0
EVAL_EVERY  = 500
N_EVAL      = 2048
SOLVE_ACC   = 0.999
INIT_SCALE  = 0.05
MAX_STEPS   = 3_500
N_BATCHES   = N_EVAL // BATCH_SIZE
NEG_INF     = -1e9
N_PROBE     = 4096
PROJ_EPS    = 0.1
PROJ_CLIP   = 10.0

# ═══════════════════════════════════════════════════════════════════════
# Dimension sweep — sequence horizon D values
# D_HIDDEN=32 is fixed throughout; only the horizon varies.
# Geometric notes:
#   D=24 = 2 × 12 (multiple of period-12 cycle)
#   D=36 = 3 × 12 (multiple of period-12 cycle)
#   D=16,20,28,32,40 = NOT multiples of period-12
# This contrast is central to the resonance hypothesis.
# ═══════════════════════════════════════════════════════════════════════
HORIZONS = [16, 20, 24, 28, 32, 36, 40]

# ═══════════════════════════════════════════════════════════════════════
# Cyclic block structure — identical to locked stack failure isolation probe
# (start, end, period, n_harmonics)
# ═══════════════════════════════════════════════════════════════════════
BLOCKS_A = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# Dominant cyclic component span for Task A
TASK_A_CYC_S, TASK_A_CYC_E = 9, 21   # 12-state cyclic component

# ═══════════════════════════════════════════════════════════════════════
# Geometry-native partition tables (Task A family only)
#
# PARTITION_ORIGINAL: period-3 interleaved sub-groups of the 12-state cycle.
#   class 0={0,4,8}, class 1={1,5,9}, class 2={2,6,10}, class 3={3,7,11}
#   This partition is INTERLEAVED: consecutive states cycle through all 4 classes.
#   Post-hoc mod label: argmax%4 over the dominant block.
#
# PARTITION_SHIFT1: same period-3 sub-group structure, class labels rotated +1.
#   class 0={1,5,9}, class 1={2,6,10}, class 2={3,7,11}, class 3={0,4,8}
#   Orientation sensitivity test: same geometry, relabelled.
#
# No mod/modular arithmetic is used as a control primitive — these are pure lookups.
# ═══════════════════════════════════════════════════════════════════════
PARTITION_ORIGINAL = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]
PARTITION_SHIFT1   = [1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0]

# Two Task A variants as specified
TASK_A_VARIANTS = [
    ("original_s42", 42, PARTITION_ORIGINAL,
     "period-3 interleaved sub-groups; seed=42 (locked-stack canonical replication)"),
    ("shift1_s42",   42, PARTITION_SHIFT1,
     "period-3 sub-groups, class-label rotation +1; seed=42"),
]


# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers (identical to locked stack failure isolation probe)
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def ctrl_input_dim(d_ang: int, n_pairs: int, n_blocks: int) -> int:
    """Projective controller only (locked stack — controller mode fixed)."""
    return D_EMB + n_pairs + n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
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


def make_projective_features(ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    """proj_k = sin_k / (1 + cos_k + PROJ_EPS), clipped to ±PROJ_CLIP."""
    parts = []
    for k in range(n_pairs):
        c = ang[..., 2 * k]
        s = ang[..., 2 * k + 1]
        t = s / (1.0 + c + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


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
    """Convert one-hot tables → angular + apply two_i_orient anchor (locked)."""
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
        cyc_start: int, cyc_end: int,
        partition: List[int],
) -> torch.Tensor:
    """Geometry-native class labels from a phase-partition lookup table.

    Maps each state's position within the cyclic component [cyc_start:cyc_end]
    to a class via the partition list. No modular framing — pure lookup.
    """
    period = cyc_end - cyc_start
    assert len(partition) == period, (
        f"partition length {len(partition)} must equal period {period}")
    idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut = torch.tensor(partition, dtype=torch.long)
    return lut[idx]


# ═══════════════════════════════════════════════════════════════════════
# Router model — locked stack
# Identical to RouterLockedStack in locked stack failure isolation probe.
# D_HIDDEN is FIXED at 32 regardless of the horizon D.
# ═══════════════════════════════════════════════════════════════════════
class RouterLockedStack(nn.Module):
    """Locked stack router for dimension-phase alignment probe.

    All four locked components are fixed:
      transport=split(eps=1.0), state=harmonic, anchor=two_i, ctrl=projective.
    Only varied: horizon D (sequence length). D_HIDDEN=32 throughout.
    """

    def __init__(
            self,
            TN_ang, TR, tau0_hyb, pool_ids,
            D: int,
            blocks,
            noise_sigma: float = 0.0,
            eps_high: float = 1.0,
            init_scale: float = INIT_SCALE,
            seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        self.blocks      = blocks
        self.eps_high    = eps_high
        self.noise_sigma = noise_sigma
        self.D           = D

        d_ang, n_pairs, n_blocks = geom_dims(blocks)
        self.d_ang    = d_ang
        self.n_pairs  = n_pairs
        self.n_blocks = n_blocks

        dh  = D_HIDDEN   # FIXED at 32 — network width does not track horizon
        dha = max(8, dh // 4)
        d_ctrl = ctrl_input_dim(d_ang, n_pairs, n_blocks)
        self.d_ctrl = d_ctrl

        self.register_buffer("TN",         TN_ang)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids",   pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]     = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1] = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(0.0))
        self.b_posLast = nn.Parameter(torch.tensor(0.0))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(
            torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        d_hyb = d_ang + n_blocks
        self.W_emb  = rp(VOCAB, D_EMB)
        self.W1     = rp(d_ctrl, dh);   self.b1 = zp(dh)
        self.W2     = rp(dh, N_OPS);    self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb)
        self.b_attn = zp(dha)
        self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)

    def _build_ctrl_input(self, embs, tau_prev):
        ang  = tau_prev[:, :self.d_ang]
        mags = tau_prev[:, self.d_ang:]
        proj = make_projective_features(ang, self.n_pairs)
        return torch.cat([embs, proj, mags], dim=1)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        ctrl   = self._build_ctrl_input(embs, tau_prev)
        h      = torch.tanh(ctrl @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        base      = torch.einsum("bi,bij->bj", w, tn)
        hybrid    = apply_split_transport(base, tau_prev, self.blocks, self.eps_high)
        new_sids  = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def _eval_transport(self, tau_prev, best_ang):
        return apply_split_transport(best_ang, tau_prev, self.blocks, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        D        = self.D
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

    def readout(self, st):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha

    def readout_masked(self, st, mask):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + mask)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers (identical to locked stack failure isolation probe)
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, classes, D: int, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, classes) -> Tuple[float, float]:
    D   = model.D
    B   = BATCH_SIZE
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
                embs    = model.W_emb[toks[:, t]]
                cur_dir  = tau_prev[:, :model.d_ang]
                ang_sim  = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op  = ang_sim.argmax(dim=1)
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
    """Probe decodability at the final trajectory step only."""
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
                embs    = model.W_emb[toks[:, t]]
                cur_dir  = tau_prev[:, :model.d_ang]
                ang_sim  = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op  = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
                if t == D - 1:
                    tau_f_list.append(hybrid.cpu())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
    y     = torch.cat(lbl_list,   dim=0).numpy()
    tau_f = torch.cat(tau_f_list, dim=0).numpy()
    return linear_probe(tau_f, y, f"final(t={D-1})")


# ═══════════════════════════════════════════════════════════════════════
# Training loop (identical to locked stack failure isolation probe)
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        label: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, classes,
        blocks,
        seed: int,
) -> Dict:
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    d_ctrl = ctrl_input_dim(d_ang, n_pairs, n_blocks)
    print(f"\n  [{label}|D={D}] seed={seed}  d_ctrl={d_ctrl}  "
          f"n_pairs={n_pairs}  n_blocks={n_blocks}  D_HIDDEN={D_HIDDEN}")

    model = RouterLockedStack(
        TN_ang, TR, tau0_hyb, pool_ids,
        D=D, blocks=blocks,
        noise_sigma=0.0, eps_high=1.0,
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
            print(f"    step={step:5d}  acc={acc:.4f}  α_D={aD:.4f}  wall={wall:.1f}s")
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            if step == MAX_STEPS:
                final_acc = acc
                alphaD    = aD

    wall = time.perf_counter() - t0
    print(f"    DONE: final_acc={final_acc:.4f}  solve_step={solve_step}  wall={wall:.1f}s")

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
# CSV output
# ═══════════════════════════════════════════════════════════════════════
CSV_FIELDS = [
    "variant", "horizon", "seed",
    "accuracy", "solve_step", "decodability_final",
    "alpha_last", "orientation_gap",
    "runtime_seconds", "note",
]


def write_csv(rows: List[Dict]) -> None:
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        w.writeheader()
        w.writerows(rows)
    print(f"\nCSV written: {CSV_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Markdown report
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(rows: List[Dict]) -> None:
    # Index rows
    data: Dict[Tuple[str, int], Dict] = {}
    for r in rows:
        data[(r["variant"], int(r["horizon"]))] = r

    def get(variant, d, field, default="?"):
        row = data.get((variant, d))
        if row is None:
            return default
        v = row.get(field, default)
        return v if v not in (None, "", "None") else default

    def fmt_acc(variant, d):
        v = get(variant, d, "accuracy")
        return f"{float(v):.4f}" if v != "?" else "?"

    def fmt_gap(d):
        v = get("original_s42", d, "orientation_gap")
        return f"{float(v):+.4f}" if v != "?" else "?"

    # Compute per-D averages and detect monotonicity
    avg_by_d = {}
    for d in HORIZONS:
        accs = []
        for vname, _, _, _ in TASK_A_VARIANTS:
            v = get(vname, d, "accuracy")
            if v != "?":
                accs.append(float(v))
        avg_by_d[d] = round(sum(accs) / len(accs), 4) if accs else float("nan")

    best_d  = max(avg_by_d, key=lambda d: avg_by_d[d])
    worst_d = min(avg_by_d, key=lambda d: avg_by_d[d])

    avgs = [avg_by_d[d] for d in HORIZONS]
    diffs = [avgs[i+1] - avgs[i] for i in range(len(avgs)-1)]
    all_incr = all(x > 0 for x in diffs)
    all_decr = all(x < 0 for x in diffs)
    if all_incr:
        monotone_desc = "MONOTONICALLY INCREASING"
    elif all_decr:
        monotone_desc = "MONOTONICALLY DECREASING"
    elif max(avg_by_d.values()) - min(avg_by_d.values()) < 0.005:
        monotone_desc = "FLAT (range < 0.005)"
    else:
        monotone_desc = "NON-MONOTONIC (resonance candidate)"

    gaps = [float(get("original_s42", d, "orientation_gap", "nan")) for d in HORIZONS]
    valid_gaps = [g for g in gaps if not math.isnan(g)]
    gap_min_d = HORIZONS[gaps.index(min(valid_gaps, key=abs))] if valid_gaps else None
    gap_sign_changes = sum(
        1 for i in range(len(valid_gaps)-1)
        if (valid_gaps[i] >= 0) != (valid_gaps[i+1] >= 0)
    ) if len(valid_gaps) > 1 else 0

    # Determine conclusion
    non_mono = monotone_desc.startswith("NON-MONOTONIC")
    has_preferred_band = False
    if len(avgs) >= 3:
        top_avg = sorted(avgs)[-2]
        spread  = max(avgs) - min(avgs)
        has_preferred_band = spread > 0.01
    gap_varies_non_monotone = gap_sign_changes > 0

    support_count = sum([non_mono, has_preferred_band, gap_varies_non_monotone])
    if support_count >= 2:
        conclusion = "STRONG"
    elif support_count == 1:
        conclusion = "INCONCLUSIVE"
    else:
        conclusion = "NOT SUPPORTED"

    L = []
    L.append("# Dimension–Phase Alignment Probe v1\n\n")
    L.append(f"**Branch:** `dimension_phase_alignment_probe_v1`  \n")
    L.append(f"**Date:** {datetime.date.today()}  \n")
    L.append(f"**Script:** `tools/prime_transport/run_router_dimension_phase_alignment_probe_v1.py`\n\n")
    L.append("---\n\n")

    L.append("## Geometry Lock Summary\n\n")
    L.append("| Property | Value |\n|---|---|\n")
    L.append("| Task | Task A family: original_s42, shift1_s42 |\n")
    L.append("| State space | BLOCKS_A = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)] |\n")
    L.append("| Dominant cycle | positions 9..21, period 12, 3 harmonics |\n")
    L.append(f"| Partition type | **INTERLEAVED** — class k = position mod 4 within period-12 |\n")
    L.append("| Partition (original) | [0,1,2,3,0,1,2,3,0,1,2,3] |\n")
    L.append("| Partition (shift1)   | [1,2,3,0,1,2,3,0,1,2,3,0] |\n")
    L.append("| Locked stack | split_transport(eps=1.0) + harmonic_state + two_i_orient + projective_ctrl |\n")
    L.append(f"| D_HIDDEN (network width) | {D_HIDDEN} — **FIXED throughout sweep** |\n")
    L.append(f"| Horizon D sweep | {HORIZONS} |\n")
    L.append("| Period-12 multiples in sweep | D=24 (2×12), D=36 (3×12) |\n")
    L.append("| Non-multiples in sweep | D=16,20,28,32,40 |\n\n")

    L.append("**Canonical anchor values (from file 3):**\n\n")
    L.append(f"- original_s42: D=24 → {CANON_LS_ORIG_D24},  D=32 → {CANON_LS_ORIG_D32}\n")
    L.append(f"- shift1_s42:   D=24 → {CANON_LS_SH1_D24},   D=32 → {CANON_LS_SH1_D32}\n")
    L.append(f"- Orientation gap D=24: {CANON_LS_ORIG_D24 - CANON_LS_SH1_D24:+.4f},  "
             f"D=32: {CANON_LS_ORIG_D32 - CANON_LS_SH1_D32:+.4f}\n\n")

    L.append("---\n\n")
    L.append("## D Sweep Accuracy Table\n\n")
    L.append("| D | is_period_multiple | original_s42 | shift1_s42 | avg_accuracy |\n")
    L.append("|---|---|---|---|---|\n")
    for d in HORIZONS:
        mult = "YES (×12)" if d % 12 == 0 else "no"
        L.append(f"| {d} | {mult} | {fmt_acc('original_s42', d)} | "
                 f"{fmt_acc('shift1_s42', d)} | {avg_by_d[d]:.4f} |\n")

    L.append(f"\n**Best D by average accuracy:** D={best_d} ({avg_by_d[best_d]:.4f})  \n")
    L.append(f"**Worst D by average accuracy:** D={worst_d} ({avg_by_d[worst_d]:.4f})  \n")
    L.append(f"**Performance vs D pattern:** {monotone_desc}\n\n")

    L.append("---\n\n")
    L.append("## Orientation Gap Table\n\n")
    L.append("*Orientation gap = accuracy(original_s42) − accuracy(shift1_s42)*\n\n")
    L.append("| D | is_period_multiple | orientation_gap | sign |\n")
    L.append("|---|---|---|---|\n")
    for d in HORIZONS:
        mult  = "YES (×12)" if d % 12 == 0 else "no"
        gapv  = get("original_s42", d, "orientation_gap", "?")
        if gapv != "?":
            gf    = float(gapv)
            sign  = "+" if gf >= 0 else "−"
            L.append(f"| {d} | {mult} | {gf:+.4f} | {sign} |\n")
        else:
            L.append(f"| {d} | {mult} | ? | ? |\n")
    L.append(f"\n**Gap sign changes across D:** {gap_sign_changes}  \n")
    if gap_min_d is not None:
        L.append(f"**D with smallest |gap|:** D={gap_min_d} ({fmt_gap(gap_min_d)})\n\n")

    L.append("---\n\n")
    L.append("## Decodability Table\n\n")
    L.append("| D | variant | decodability_final | alpha_last |\n")
    L.append("|---|---|---|---|\n")
    for d in HORIZONS:
        for vname, _, _, _ in TASK_A_VARIANTS:
            dec  = get(vname, d, "decodability_final")
            alph = get(vname, d, "alpha_last")
            L.append(f"| {d} | {vname} | {dec} | {alph} |\n")

    L.append("\n---\n\n")
    L.append("## Mandatory Questions\n\n")

    orig_accs = [float(get("original_s42", d, "accuracy", "nan")) for d in HORIZONS]
    sh1_accs  = [float(get("shift1_s42",   d, "accuracy", "nan")) for d in HORIZONS]
    valid_orig = [a for a in orig_accs if not math.isnan(a)]

    L.append(f"**Q1. Does performance vary non-monotonically with D?**  \n")
    L.append(f"Pattern: {monotone_desc}. ")
    if non_mono:
        L.append("YES — performance varies non-monotonically across the D sweep.\n\n")
    else:
        L.append("NOT established — no clear non-monotonic pattern detected.\n\n")

    L.append(f"**Q2. Are there preferred D values?**  \n")
    if has_preferred_band:
        L.append(f"YES — best D={best_d} (avg={avg_by_d[best_d]:.4f}) vs "
                 f"worst D={worst_d} (avg={avg_by_d[worst_d]:.4f}), "
                 f"spread={avg_by_d[best_d]-avg_by_d[worst_d]:.4f} > 0.01 threshold.\n\n")
    else:
        L.append(f"NOT established — spread ({avg_by_d[best_d]-avg_by_d[worst_d]:.4f}) "
                 f"below 0.01 threshold.\n\n")

    L.append(f"**Q3. Does orientation sensitivity shrink at specific D values?**  \n")
    if gap_varies_non_monotone:
        L.append(f"YES — orientation gap changes sign {gap_sign_changes} time(s) across D. "
                 f"Gap is minimized in magnitude at D={gap_min_d}.\n\n")
    else:
        L.append(f"NOT established — orientation gap does not change sign across D sweep.\n\n")

    L.append("**Q4. Is there evidence that D is part of the geometry rather than just generic capacity?**  \n")
    evidence = []
    if non_mono:        evidence.append("non-monotonic accuracy vs D")
    if has_preferred_band: evidence.append(f"preferred D band around D={best_d}")
    if gap_varies_non_monotone: evidence.append("orientation gap sign change across D")
    if evidence:
        L.append(f"POSSIBLE — evidence includes: {'; '.join(evidence)}. "
                 f"Period-12 multiples (D=24, D=36) will be examined in context.\n\n")
    else:
        L.append("NOT ESTABLISHED — no qualifying evidence found.\n\n")

    L.append("**Q5. Does this keep the possibility of a later fair R^4 retest alive?**  \n")
    if conclusion in ("STRONG", "INCONCLUSIVE"):
        L.append(
            "YES — if dimension-phase alignment is confirmed (even weakly), it motivates a "
            "future R^4 retest at a geometrically-aligned D, since the previous φ/R^4 failure "
            "used D=24 (period-aligned) and D=32 (non-aligned) without controlling for this "
            "dimension effect. A fair R^4 retest should hold D fixed at the best-performing "
            "non-φ baseline dimension.  \n"
            "Note: this does NOT imply φ or R^4 are necessary — the prior failure stands. "
            "It only identifies whether dimension was a confound.\n\n"
        )
    else:
        L.append(
            "WEAKLY — the prior φ/R^4 failure is not overturned by this result. "
            "A future retest is not specifically motivated by dimension alignment. "
            "Other confounds remain possible.\n\n"
        )

    L.append("---\n\n")
    L.append(f"## One-Line Conclusion\n\n")
    L.append(f"**DIMENSION–PHASE ALIGNMENT EFFECT IS: {conclusion}**\n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")
    L.append("### What was actually tested\n\n")
    L.append(
        "- A sweep of sequence horizon D ∈ {16,20,24,28,32,36,40} under the locked stack "
        "(split transport + harmonic state + two_i_orient + projective controller).\n"
        "- Network width D_HIDDEN=32 was held constant; only the horizon length varied.\n"
        "- Two orientation variants of Task A (original_s42, shift1_s42) were run at each D.\n"
        "- Accuracy, decodability, and orientation gap were recorded per (D, variant) run.\n"
        "- No φ, R^4, or new controller features were introduced.\n\n"
    )
    L.append("### What remains open\n\n")
    L.append(
        "- Whether D_HIDDEN (network width, fixed at 32) also shows resonance-like behavior.\n"
        "- Whether the orientation gap pattern is stable across seeds.\n"
        "- Whether the result generalises to tasks with different cyclic periods.\n"
        "- Whether the phase-alignment effect (if found) interacts with φ or R^4 coupling.\n\n"
    )
    L.append("### Why this does or does not justify a later R^4 retest\n\n")
    if conclusion == "STRONG":
        L.append(
            "The STRONG result indicates that D is geometrically relevant. The previous φ/R^4 "
            "probe ran at D=24 and D=32 — one period-aligned and one not — without controlling "
            "for this. A fair R^4 retest should be conducted at the identified best-performing D, "
            "with orientation matched. The prior failure (φ/R^4 at D=24) is NOT overturned; "
            "it remains a valid falsification of that specific operationalization. A new retest "
            "would be testing a genuinely different configuration.\n"
        )
    elif conclusion == "INCONCLUSIVE":
        L.append(
            "The INCONCLUSIVE result provides weak evidence. A future R^4 retest cannot be "
            "strongly motivated by dimension alignment alone, but the identified D preference "
            "(if reproducible) may warrant one targeted retest at the preferred D. The prior "
            "φ/R^4 failure is not overturned by this result.\n"
        )
    else:
        L.append(
            "The NOT SUPPORTED result provides no geometric motivation for a D-controlled "
            "R^4 retest. The prior φ/R^4 failure stands without a dimension-based confound. "
            "Any future R^4 retest must find a different rationale.\n"
        )
    L.append("\n**HARD RULE reminder:** This probe does NOT reinterpret the failed φ/R^4 probe "
             "as proving φ or R^4 are unnecessary in general. It only characterises whether "
             "dimension D is itself part of the operative geometry.\n")

    with open(MD_OUT, "w") as f:
        f.write("".join(L))
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("DIMENSION–PHASE ALIGNMENT PROBE v1")
    print(f"  Horizons:     {HORIZONS}")
    print(f"  Variants:     original_s42, shift1_s42")
    print(f"  D_HIDDEN:     {D_HIDDEN} (FIXED — network width locked)")
    print(f"  Total runs:   {len(HORIZONS) * len(TASK_A_VARIANTS)}")
    print("=" * 70)

    # Canonical file check
    for fname in (
        "prime_transport_router_start_agnostic_root_probe_v1.csv",
        "prime_transport_router_controller_geometry_probe_v1.csv",
        "router_locked_stack_failure_isolation_probe_v1.csv",
    ):
        p = RESULTS_DIR / fname
        print(f"  {'✓' if p.exists() else '✗ MISSING:'} {fname}")

    # Load state cache
    if not CACHE_PATH.exists():
        print(f"\nERROR: state cache not found at {CACHE_PATH}")
        sys.exit(1)

    print(f"\nLoading state cache: {CACHE_PATH}")
    cache       = torch.load(CACHE_PATH, map_location="cpu", weights_only=True)
    TN_oh_AB    = cache["TN_oh"]
    tau0_oh_AB  = cache["tau0_oh"]
    TR_AB       = cache["TR"]
    pool_ids_AB = cache["pool_ids"]
    print(f"  Cache loaded: {tau0_oh_AB.shape[0]} states, pool={pool_ids_AB.shape[0]}")

    # Pre-compute tables (constant across all D values — only horizon changes)
    TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
        TN_oh_AB, tau0_oh_AB, TR_AB, pool_ids_AB, BLOCKS_A
    )

    csv_rows: List[Dict] = []
    # acc_by_variant_D[(variant, D)] = accuracy  — for gap computation
    acc_by_variant_D: Dict[Tuple[str, int], float] = {}

    print("\n" + "=" * 70)
    print("D SWEEP — Task A (original_s42 and shift1_s42)")
    print("=" * 70)

    for D in HORIZONS:
        print(f"\n{'─'*60}")
        print(f"HORIZON D={D}  (D_HIDDEN fixed at {D_HIDDEN})")
        # Is this D a multiple of the period-12 cycle?
        is_period_mult = (D % 12 == 0)
        print(f"  Period-12 multiple: {'YES' if is_period_mult else 'no'}")
        print(f"{'─'*60}")

        for vname, vseed, vpartition, vdesc in TASK_A_VARIANTS:
            classes = build_class_table_from_lookup(
                tau0_oh_AB, TASK_A_CYC_S, TASK_A_CYC_E, vpartition
            )
            cls_dist = [(classes == i).sum().item() for i in range(VOCAB)]
            print(f"\n  variant={vname}  classes={cls_dist}")

            result = train_one(
                f"D{D}/{vname}", D,
                TN_ang, TR_p, tau0_hyb, pool_ids_p, classes,
                BLOCKS_A, seed=vseed,
            )
            model = result["model"]

            # Decodability at final step
            dec_final = run_decodability_final(model, pool_ids_p, classes)
            acc       = result["final_acc"]
            acc_by_variant_D[(vname, D)] = acc

            # Orientation gap will be filled after both variants at this D are done
            csv_rows.append({
                "variant":           vname,
                "horizon":           D,
                "seed":              vseed,
                "accuracy":          acc,
                "solve_step":        result["solve_step"] if result["solve_step"] else "",
                "decodability_final": dec_final,
                "alpha_last":        result["alphaD"],
                "orientation_gap":   "",   # filled below
                "runtime_seconds":   result["wall"],
                "note": (
                    f"branch=dimension_phase_alignment_probe_v1; "
                    f"variant={vname}; D={D}; D_HIDDEN={D_HIDDEN}; seed={vseed}; "
                    f"period12_multiple={'yes' if is_period_mult else 'no'}; "
                    f"partition=custom_lookup; noise_sigma=0.00; "
                    f"anchor=two_i_rot; ctrl=projective; eps_high=1.0; {vdesc}"
                ),
            })

        # Compute orientation gap for this D (original_s42 - shift1_s42)
        acc_orig = acc_by_variant_D.get(("original_s42", D))
        acc_sh1  = acc_by_variant_D.get(("shift1_s42",   D))
        if acc_orig is not None and acc_sh1 is not None:
            gap = round(acc_orig - acc_sh1, 4)
            print(f"\n  ── Orientation gap at D={D}: {gap:+.4f} "
                  f"(original={acc_orig:.4f}  shift1={acc_sh1:.4f})")
            # Back-fill orientation_gap for both rows at this D
            for row in csv_rows:
                if int(row["horizon"]) == D:
                    row["orientation_gap"] = gap

    print("\n" + "=" * 70)
    print("WRITING OUTPUTS")
    print("=" * 70)
    write_csv(csv_rows)
    write_markdown(csv_rows)

    # Print summary table
    print("\n── SUMMARY ──")
    print(f"{'D':>4}  {'original_s42':>14}  {'shift1_s42':>12}  {'avg':>8}  {'gap':>8}  {'mult12':>8}")
    for D in HORIZONS:
        ao = acc_by_variant_D.get(("original_s42", D), float("nan"))
        as_ = acc_by_variant_D.get(("shift1_s42",  D), float("nan"))
        avg = (ao + as_) / 2 if not (math.isnan(ao) or math.isnan(as_)) else float("nan")
        gap = ao - as_ if not (math.isnan(ao) or math.isnan(as_)) else float("nan")
        mult = "YES" if D % 12 == 0 else "no"
        print(f"{D:>4}  {ao:>14.4f}  {as_:>12.4f}  {avg:>8.4f}  {gap:>+8.4f}  {mult:>8}")

    print("\nDone.")


if __name__ == "__main__":
    main()
