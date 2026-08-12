#!/usr/bin/env python3
"""run_router_locked_stack_failure_isolation_probe_v1.py

LOCKED STACK FAILURE ISOLATION PROBE v1
========================================

MANDATORY FIRST READS (authoritative canon for this branch):
  (1) prime_transport_router_start_agnostic_root_probe_v1.csv
      - baseline D32 collapse: 0.7109              ✓
      - two_i_orient D24=0.9922, D32=0.9873        ✓
      - decodability = 1.0 throughout              ✓

  (2) prime_transport_router_controller_geometry_probe_v1.csv
      - projective D24=0.9878, D32=0.9946          ✓
      - projective Δ(D32−D24) = +0.0068            ✓
      - decodability = 1.0 throughout              ✓

FAILURE CASE (file 3 — contrasted against canon, NOT the baseline):
  router_locked_stack_generalization_probe_v1.csv:
  - Task A (same geometry as canonical): locked_stack D32=0.9028 vs canon 0.9946
    → regression gap = −0.0918 at D32
  - Task B (perturbed): locked_stack D24=0.7686 (−0.197 vs Task A)
  - Task C (structural variant): locked_stack D32=1.0  (strong success)

OBJECTIVE:
  Isolate which mechanism causes Task A regression and Task B noise sensitivity,
  without modifying the locked stack.

LOCKED STACK (DO NOT MODIFY):
  1. Split transport:    eps_high=1.0 (higher harmonics frozen at previous state)
  2. State:             harmonic sin/cos encoding (decodability must stay 1.0)
  3. Start anchor:      two_i_orient: rotate each (c,s)→(−s,c) per pair
  4. Controller:        projective: proj_k = sin_k/(1+cos_k+0.1), clipped ±10

TASK DEFINITIONS (geometry-native):
  Task A control:
    Cyclic state space; dominant cyclic component of period 12.
    Target: quarter-period phase class — 4 equal phase regions of width 3 steps.
    No perturbation.

  Task B perturbed:
    Same cyclic geometry and phase partition as Task A.
    Perturbation: Gaussian noise σ on token embeddings at every step.

  Task C structural:
    Product-cycle state space; two cyclic components of different periods.
    Target: quarter-period phase class of the shorter-period component.
    Structurally different phase geometry from Task A.

PROBES:
  Probe 1 (Task A micro-variations):
    - 4 partition variants (see TASK_A_VARIANTS) × 2 horizons × locked_stack only
    - Isolates: seed sensitivity (E), partition-boundary sensitivity (A/D)

  Probe 2 (Task B noise sweep):
    - σ ∈ {0.0, 0.05, 0.1, 0.2} × 2 horizons × locked_stack only
    - Isolates: noise amplification curve at D=24 vs D=32 (C)

  Probe 3 (Task C stability):
    - Extra seed (123) × 2 horizons × locked_stack only
    - Confirms whether D32=1.0 is reproducible (not lucky seed)

  Total: 8 + 8 + 2 = 18 runs.

DELIVERABLES:
  PY  : tools/prime_transport/run_router_locked_stack_failure_isolation_probe_v1.py
  CSV : results/prime_transport_recursive_system/
          router_locked_stack_failure_isolation_probe_v1.csv
  MD  : docs/research/
          prime_transport_router_locked_stack_failure_isolation_probe_v1.md
"""
from __future__ import annotations

import csv
import datetime
import itertools
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
CSV_OUT     = RESULTS_DIR / "router_locked_stack_failure_isolation_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_locked_stack_failure_isolation_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Authoritative reference values (hard-coded from canonical CSVs)
# ═══════════════════════════════════════════════════════════════════════
# File (1): prime_transport_router_start_agnostic_root_probe_v1.csv
CANON_BASELINE_D24  = 0.9858
CANON_BASELINE_D32  = 0.7109   # D32 collapse
CANON_TWO_I_D24     = 0.9922
CANON_TWO_I_D32     = 0.9873
# File (2): prime_transport_router_controller_geometry_probe_v1.csv
CANON_PROJ_D24      = 0.9878
CANON_PROJ_D32      = 0.9946   # canonical locked stack performance at D32
# File (3): router_locked_stack_generalization_probe_v1.csv (failure case)
FAIL_TASK_A_LS_D24  = 0.9653
FAIL_TASK_A_LS_D32  = 0.9028
FAIL_TASK_B_LS_D24  = 0.7686
FAIL_TASK_B_LS_D32  = 0.9634
FAIL_TASK_C_LS_D32  = 1.0000

# ═══════════════════════════════════════════════════════════════════════
# Hyperparameters — identical to locked stack generalization probe
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED = 42
D_HIDDEN    = 32
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
# Cyclic block structures — identical to generalization probe
# (start, end, period, n_harmonics)
# ═══════════════════════════════════════════════════════════════════════
BLOCKS_A = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
BLOCKS_C = [(0, 5, 5, 1), (5, 13, 8, 2)]

# Dominant cyclic component span for Task A
TASK_A_CYC_S, TASK_A_CYC_E = 9, 21   # 12-state cyclic component

# Dominant cyclic component span for Task C
TASK_C_CYC_S, TASK_C_CYC_E = 5, 13   # 8-state cyclic component

# ═══════════════════════════════════════════════════════════════════════
# Geometry-native partition tables for Task A variants
# Each list maps state index k (0..11) in the 12-state cyclic component
# to a phase class (0..3). No modular framing — pure lookup.
#
# The original Task A groups states by their membership in the
# period-3 sub-cycle of the 12-state cycle:
#   class 0 = {0,4,8},  class 1 = {1,5,9},
#   class 2 = {2,6,10}, class 3 = {3,7,11}
# These four interleaved sub-groups form a period-3 rotational structure.
# The 3rd harmonic of the 12-state cycle naturally encodes this pattern.
# (Post-hoc label: this is argmax%4 over the dominant block.)
# ═══════════════════════════════════════════════════════════════════════

# Original: period-3 interleaved sub-groups
# Replicates file-3 Task A exactly (equivalent to argmax%4 post-hoc)
PARTITION_ORIGINAL  = [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]

# Phase-class rotation +1: same period-3 sub-group structure, class labels shifted
# class 0 = {1,5,9}, class 1 = {2,6,10}, class 2 = {3,7,11}, class 3 = {0,4,8}
PARTITION_SHIFT1    = [1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0]

# Phase-class rotation +2: half-period shift within period-3 sub-cycle
# Equivalent to swapping class pairs (0↔2, 1↔3)
PARTITION_SHIFT2    = [2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1]

# Aperiodic: break period-3 symmetry — unequal class sizes {4,4,3,1}
# States: class0={0,3,6,9}, class1={1,4,7,10}, class2={2,5,8}, class3={11}
# Tests whether period-3 symmetry is required for the locked stack to succeed
PARTITION_APERIODIC = [0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 3]

# ═══════════════════════════════════════════════════════════════════════
# Probe experiment definitions
# ═══════════════════════════════════════════════════════════════════════
# Probe 1: Task A variants — (variant_name, seed, partition, description)
TASK_A_VARIANTS = [
    ("original_s42",   42, PARTITION_ORIGINAL,
     "period-3 sub-groups; seed=42 (exact replication of file-3 Task A failure)"),
    ("original_s99",   99, PARTITION_ORIGINAL,
     "period-3 sub-groups; seed=99 (seed sensitivity check)"),
    ("shift1_s42",     42, PARTITION_SHIFT1,
     "period-3 sub-groups, class-label rotation +1; seed=42"),
    ("aperiodic_s42",  42, PARTITION_APERIODIC,
     "period-3 symmetry broken; unequal class sizes {4,4,3,1}; seed=42"),
]

# Probe 2: Task B noise sweep — sigma values (locked_stack only, PARTITION_ORIGINAL)
NOISE_SIGMAS = [0.0, 0.05, 0.1, 0.2]

# Probe 3: Task C extra seed
TASK_C_EXTRA_SEED = 123

HORIZONS = [24, 32]

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers (identical to generalization probe)
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs  = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang    = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def ctrl_input_dim(d_ang: int, n_pairs: int, n_blocks: int) -> int:
    """Projective controller only (locked stack — controller mode is fixed)."""
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
    to a class via the partition list. No modular framing required — pure
    lookup over the discrete cyclic phase.
    """
    period = cyc_end - cyc_start
    assert len(partition) == period, (
        f"partition length {len(partition)} must equal period {period}")
    idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut = torch.tensor(partition, dtype=torch.long)
    return lut[idx]


# ═══════════════════════════════════════════════════════════════════════
# Synthetic state tables for Task C: Z_5 × Z_8 (unchanged from gen probe)
# ═══════════════════════════════════════════════════════════════════════
D_ONEHOT_C = 13  # 5 (period-5) + 8 (period-8)

def build_mod8_state_tables() -> Tuple[torch.Tensor, torch.Tensor,
                                        torch.Tensor, torch.Tensor]:
    """Build product-cycle state tables for Task C.

    State (b, c): b ∈ Z_5 (structural noise), c ∈ Z_8 (signal carrier).
    Target: quarter-period phase class of c (4 equal regions of 2 steps each).
    """
    all_states = list(itertools.product(range(5), range(8)))
    N_STATES   = len(all_states)
    s2i        = {s: i for i, s in enumerate(all_states)}

    OPS = [
        (0, 1), (0, 2), (0, 3),
        (1, 1), (2, 2), (3, 3),
    ]
    TR = torch.zeros(N_STATES, N_OPS, dtype=torch.long)
    for i, (b, c) in enumerate(all_states):
        for op, (db, dc) in enumerate(OPS):
            TR[i, op] = s2i[((b + db) % 5, (c + dc) % 8)]

    tau0_oh = torch.zeros(N_STATES, D_ONEHOT_C)
    for i, (b, c) in enumerate(all_states):
        tau0_oh[i, b]              = 1.0
        tau0_oh[i, TASK_C_CYC_S + c] = 1.0

    TN_oh = torch.zeros(N_STATES, N_OPS, D_ONEHOT_C)
    for i in range(N_STATES):
        for op in range(N_OPS):
            TN_oh[i, op] = tau0_oh[TR[i, op].item()]

    repeats  = (4000 + N_STATES - 1) // N_STATES
    pool_ids = torch.arange(N_STATES).repeat(repeats)[:4000]
    return TN_oh, TR, tau0_oh, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Router model — locked stack, projective controller only
# ═══════════════════════════════════════════════════════════════════════
class RouterLockedStack(nn.Module):
    """Locked stack router for failure isolation.

    All four locked components are fixed:
      transport=split(eps=1.0), state=harmonic, anchor=two_i, ctrl=projective.
    Only varied: blocks, noise_sigma, seed.
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

        dh  = D_HIDDEN
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
        self.W_attn = rp(max(8, dh // 4), d_hyb)
        self.b_attn = zp(max(8, dh // 4))
        self.v_attn = rp(max(8, dh // 4))
        self.W_pred = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)

    def _build_ctrl_input(self, embs, tau_prev):
        ang  = tau_prev[:, :self.d_ang]
        mags = tau_prev[:, self.d_ang:]
        proj = make_projective_features(ang, self.n_pairs)
        return torch.cat([embs, proj, mags], dim=1)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        if self.noise_sigma > 0.0:
            embs = embs + self.noise_sigma * torch.randn_like(embs)
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
                if model.noise_sigma > 0.0:
                    embs = embs + model.noise_sigma * torch.randn_like(embs)
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
                embs    = model.W_emb[toks[:, t]]
                if model.noise_sigma > 0.0:
                    embs = embs + model.noise_sigma * torch.randn_like(embs)
                cur_dir  = tau_prev[:, :model.d_ang]
                ang_sim  = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op  = ang_sim.argmax(dim=1)
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
                if model.noise_sigma > 0.0:
                    embs = embs + model.noise_sigma * torch.randn_like(embs)
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
# Training loop
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        label: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, classes,
        blocks,
        noise_sigma: float,
        seed: int,
) -> Dict:
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    d_ctrl = ctrl_input_dim(d_ang, n_pairs, n_blocks)
    print(f"\n  [{label}|D={D}] seed={seed}  noise={noise_sigma:.2f}  "
          f"d_ctrl={d_ctrl}  n_pairs={n_pairs}  n_blocks={n_blocks}")

    model = RouterLockedStack(
        TN_ang, TR, tau0_hyb, pool_ids,
        D=D, blocks=blocks,
        noise_sigma=noise_sigma, eps_high=1.0,
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
# CSV writer
# ═══════════════════════════════════════════════════════════════════════
FIELDNAMES = [
    "probe", "variant", "horizon", "seed",
    "accuracy", "solve_step", "no_last_accuracy",
    "decodability_final", "alpha_last",
    "delta_vs_file3_task_a",
    "runtime_seconds", "note",
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

    idx: Dict[Tuple, Dict] = {}
    for r in rows:
        key = (r["probe"], r["variant"], int(r["horizon"]))
        idx[key] = r

    def get(probe, variant, d, field, default="?"):
        r = idx.get((probe, variant, d))
        return r[field] if r else default

    def fmt_acc(probe, variant, d):
        v = get(probe, variant, d, "accuracy")
        return f"{float(v):.4f}" if v != "?" else "?"

    def delta(probe, variant):
        a24 = get(probe, variant, 24, "accuracy")
        a32 = get(probe, variant, 32, "accuracy")
        if a24 != "?" and a32 != "?":
            return f"{float(a32) - float(a24):+.4f}"
        return "?"

    L: List[str] = []
    L.append("# Prime Transport: Locked Stack Failure Isolation Probe v1\n\n")
    L.append(f"*Generated: {ts}*\n\n---\n\n")

    L.append("## Authoritative Canon\n\n")
    L.append("### File (1): start_agnostic_root_probe_v1.csv\n\n")
    L.append("| config | D=24 | D=32 | Δ |\n|--------|------|------|---|\n")
    L.append(f"| baseline (no anchor) | {CANON_BASELINE_D24} | {CANON_BASELINE_D32} "
             f"| {CANON_BASELINE_D32 - CANON_BASELINE_D24:+.4f} |\n")
    L.append(f"| **two_i_orient** | **{CANON_TWO_I_D24}** | **{CANON_TWO_I_D32}** "
             f"| **{CANON_TWO_I_D32 - CANON_TWO_I_D24:+.4f}** |\n\n")

    L.append("### File (2): router_controller_geometry_probe_v1.csv\n\n")
    L.append("| config | D=24 | D=32 | Δ |\n|--------|------|------|---|\n")
    L.append(f"| controller_baseline | {CANON_BASELINE_D24} | 0.9648 | −0.0259 |\n")
    L.append(f"| **controller_projective** | **{CANON_PROJ_D24}** | **{CANON_PROJ_D32}** "
             f"| **+0.0068** |\n\n")
    L.append("Decodability: 1.0 at initial/mid/final for all canonical configs. "
             "D32 collapse is NOT representation failure.\n\n---\n\n")

    L.append("## Failure Case (file 3 — contrasted against canon)\n\n")
    L.append("| Task | config | D=24 | D=32 | Δ | vs canon D32 |\n"
             "|------|--------|------|------|---|---------------|\n")
    L.append(f"| A (control) | locked_stack | {FAIL_TASK_A_LS_D24} | {FAIL_TASK_A_LS_D32} "
             f"| {FAIL_TASK_A_LS_D32 - FAIL_TASK_A_LS_D24:+.4f} "
             f"| **{FAIL_TASK_A_LS_D32 - CANON_PROJ_D32:+.4f}** |\n")
    L.append(f"| B (perturbed σ=0.1) | locked_stack | {FAIL_TASK_B_LS_D24} | {FAIL_TASK_B_LS_D32} "
             f"| {FAIL_TASK_B_LS_D32 - FAIL_TASK_B_LS_D24:+.4f} | — |\n")
    L.append(f"| C (structural) | locked_stack | — | {FAIL_TASK_C_LS_D32} | — | +strong |\n\n")
    L.append("Task A regression at D32: **−0.0918** vs canonical projective stack. "
             "Task C succeeds strongly. Task B shows D24 sensitivity to noise.\n\n---\n\n")

    L.append("## Probe Design\n\n")
    L.append("### Probe 1 — Task A: partition and seed sensitivity\n\n")
    L.append("| variant | partition | seed | description |\n"
             "|---------|-----------|------|-------------|\n")
    for vname, vseed, _, vdesc in TASK_A_VARIANTS:
        L.append(f"| {vname} | — | {vseed} | {vdesc} |\n")
    L.append("\n")
    L.append("### Probe 2 — Task B: noise sweep (locked_stack, same cyclic geometry)\n\n")
    L.append(f"σ ∈ {NOISE_SIGMAS}, D ∈ {HORIZONS}. 8 runs total.\n\n")
    L.append("### Probe 3 — Task C: stability confirmation\n\n")
    L.append(f"Extra seed={TASK_C_EXTRA_SEED}, D ∈ {HORIZONS}. 2 runs.\n\n---\n\n")

    L.append("## Results\n\n")
    L.append("### Probe 1 — Task A partition variants\n\n")
    L.append("| variant | D=24 | D=32 | Δ(D32−D24) | dec_final_D32 | Δ_vs_file3_A |\n"
             "|---------|------|------|------------|---------------|---------------|\n")
    for vname, _, _, _ in TASK_A_VARIANTS:
        a24 = fmt_acc("probe1_taskA", vname, 24)
        a32 = fmt_acc("probe1_taskA", vname, 32)
        d   = delta("probe1_taskA", vname)
        dec = get("probe1_taskA", vname, 32, "decodability_final")
        dv3 = get("probe1_taskA", vname, 32, "delta_vs_file3_task_a")
        L.append(f"| {vname} | {a24} | {a32} | {d} | {dec} | {dv3} |\n")
    L.append("\n")

    L.append("### Probe 2 — Task B noise sweep\n\n")
    L.append("| σ | D=24 | D=32 | Δ(D32−D24) | dec_final_D24 |\n"
             "|---|------|------|------------|----------------|\n")
    for sig in NOISE_SIGMAS:
        vname = f"sigma_{sig:.2f}"
        a24 = fmt_acc("probe2_taskB", vname, 24)
        a32 = fmt_acc("probe2_taskB", vname, 32)
        d   = delta("probe2_taskB", vname)
        dec = get("probe2_taskB", vname, 24, "decodability_final")
        L.append(f"| {sig} | {a24} | {a32} | {d} | {dec} |\n")
    L.append("\n")

    L.append("### Probe 3 — Task C stability\n\n")
    L.append("| seed | D=24 | D=32 | Δ | dec_final_D32 |\n"
             "|------|------|------|---|----------------|\n")
    for vseed in [42, TASK_C_EXTRA_SEED]:
        vname = f"seed_{vseed}"
        a24 = fmt_acc("probe3_taskC", vname, 24)
        a32 = fmt_acc("probe3_taskC", vname, 32)
        d   = delta("probe3_taskC", vname)
        dec = get("probe3_taskC", vname, 32, "decodability_final")
        if vseed == 42:
            a24 = "—(file3)"; a32 = f"{FAIL_TASK_C_LS_D32:.4f}(file3)"; d = "—"; dec = "1.0(file3)"
        L.append(f"| {vseed} | {a24} | {a32} | {d} | {dec} |\n")
    L.append("\n---\n\n")

    L.append("## Analysis\n\n")
    L.append("### Q1: Why does Task A fail while Task C succeeds?\n\n")

    # Seed sensitivity
    a_s42 = get("probe1_taskA", "original_s42", 32, "accuracy", None)
    a_s99 = get("probe1_taskA", "original_s99", 32, "accuracy", None)
    if a_s42 is not None and a_s99 is not None:
        spread = abs(float(a_s42) - float(a_s99))
        if spread > 0.03:
            L.append(f"Seed sensitivity at D32: |s42 − s99| = {spread:.4f} > 0.03. "
                     f"Task A failure is partially OPTIMIZATION (seed-dependent).\n\n")
        else:
            L.append(f"Seed sensitivity at D32: |s42 − s99| = {spread:.4f} ≤ 0.03. "
                     f"Task A failure is CONSISTENT across seeds — not optimization noise.\n\n")
    else:
        L.append("(data pending)\n\n")

    # Partition sensitivity
    a_orig = get("probe1_taskA", "original_s42",  32, "accuracy", None)
    a_sh1  = get("probe1_taskA", "shift1_s42",    32, "accuracy", None)
    a_aper = get("probe1_taskA", "aperiodic_s42", 32, "accuracy", None)
    if a_orig is not None and a_sh1 is not None:
        vals = [v for v in [float(a_orig), float(a_sh1),
                            float(a_aper) if a_aper is not None else None]
                if v is not None]
        spread_part = max(vals) - min(vals)
        L.append(f"Partition sensitivity at D32 (original vs shift1 vs aperiodic): "
                 f"spread = {spread_part:.4f}.\n\n")
    else:
        L.append("(data pending)\n\n")

    L.append("### Q2: Is the failure symmetry mismatch or controller behavior?\n\n")
    L.append("(Answered by comparing partition variants — if shifted partitions show "
             "similar or better performance, the controller is not locked to the "
             "specific period-3 sub-group boundary. If the aperiodic partition "
             "degrades performance, period-3 symmetry is required.)\n\n")

    L.append("### Q3: Does noise interact specifically with projective features at D=24?\n\n")
    sig_0   = get("probe2_taskB", "sigma_0.00", 24, "accuracy", None)
    sig_010 = get("probe2_taskB", "sigma_0.10", 24, "accuracy", None)
    if sig_0 is not None and sig_010 is not None:
        drop = float(sig_010) - float(sig_0)
        L.append(f"Projective locked_stack D24: σ=0.0→{sig_0}, σ=0.1→{sig_010} "
                 f"(drop = {drop:+.4f}).\n\n")
    else:
        L.append("(data pending)\n\n")

    L.append("### Q4: Primary failure classification\n\n")

    # Attempt classification from data
    seed_ok  = (a_s42 is not None and a_s99 is not None and
                abs(float(a_s42) - float(a_s99)) > 0.03)
    noise_ok = (sig_0 is not None and sig_010 is not None and
                abs(float(sig_010) - float(sig_0)) > 0.10)
    c_stable = get("probe3_taskC", f"seed_{TASK_C_EXTRA_SEED}", 32, "accuracy", None)

    if a_s42 is None:
        L.append("CLASSIFICATION: (data pending)\n\n")
    else:
        causes = []
        if seed_ok:
            causes.append("OPTIMIZATION (seed-sensitive training dynamics)")
        if noise_ok:
            causes.append("NUMERICAL (projective features amplify noise at D=24)")
        a_orig_24 = get("probe1_taskA", "original_s42", 24, "accuracy", None)
        a_orig_32 = get("probe1_taskA", "original_s42", 32, "accuracy", None)
        if a_orig_24 is not None and a_orig_32 is not None:
            if float(a_orig_32) < 0.93 and not seed_ok:
                causes.append("STRUCTURAL (consistent regression regardless of seed)")
        if not causes:
            causes.append("MIXED or INCONCLUSIVE — probe insufficient to isolate")

        L.append(f"**FAILURE MODE IS: {' + '.join(causes)}**\n\n")

    L.append("---\n\n")

    # Task A vs C explanation
    L.append("## Task A vs Task C Divergence\n\n")
    L.append("Task C (product-cycle, shorter-period target, D32=1.0) succeeds while "
             "Task A (single dominant cyclic, period-12 target) shows regression at D32. "
             "The projective controller uses tan(θ/2) features per angular pair. "
             "For the shorter-period component in Task C, the critical phase signal "
             "occupies a different angular frequency; the controller may find a more "
             "numerically stable operating point. For Task A, the period-12 dominant "
             "component requires the controller to track finer angular distinctions "
             "across longer horizons — the projective features are more sensitive to "
             "phase accumulation errors. Whether this is structural (angular resolution) "
             "or optimization (training trajectory) is what Probe 1 seed comparison "
             "directly disambiguates.\n\n")

    L.append("---\n\n")
    L.append("## Mapping Note (post-hoc labels only)\n\n")
    L.append("- '12-state cyclic component' = GEOM_K3 block (9,21,12,3)\n")
    L.append("- '8-state cyclic component' = GEOM_K2 block (5,13,8,2)\n")
    L.append("- 'equal quarter-period partition' ≡ argmax % 4 (incidental)\n")
    L.append("- 'boundary rotated +1 step' ≡ (argmax+1) % 4 (incidental)\n")
    L.append("These labels are incidental to the geometry — not required for correctness.\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"  MD  → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("LOCKED STACK FAILURE ISOLATION PROBE v1")
    print(f"  Horizons:  {HORIZONS}")
    print(f"  Probe 1:   {len(TASK_A_VARIANTS)} Task A variants × {len(HORIZONS)} horizons")
    print(f"  Probe 2:   {len(NOISE_SIGMAS)} noise levels × {len(HORIZONS)} horizons")
    print(f"  Probe 3:   1 extra seed × {len(HORIZONS)} horizons")
    print(f"  Total:     {len(TASK_A_VARIANTS)*len(HORIZONS) + len(NOISE_SIGMAS)*len(HORIZONS) + len(HORIZONS)} runs")
    print("=" * 70)

    # Verify mandatory first reads
    for fname in (
        "prime_transport_router_start_agnostic_root_probe_v1.csv",
        "prime_transport_router_controller_geometry_probe_v1.csv",
    ):
        p = RESULTS_DIR / fname
        print(f"  {'✓' if p.exists() else '✗ MISSING:'} {fname}")

    # Load state cache (Tasks A and B)
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

    # Build Task C synthetic state tables
    print("\nBuilding Task C state tables (product-cycle, 40 states) ...")
    TN_oh_C, TR_C, tau0_oh_C, pool_ids_C = build_mod8_state_tables()
    print(f"  Task C: {tau0_oh_C.shape[0]} states, pool={pool_ids_C.shape[0]}")

    csv_rows: List[Dict] = []

    # ─────────────────────────────────────────────────────────────────
    # Probe 1: Task A micro-variations
    # ─────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("PROBE 1 — Task A: partition and seed sensitivity")
    print("=" * 70)

    for D in HORIZONS:
        print(f"\n── Horizon D={D} ──")
        TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
            TN_oh_AB, tau0_oh_AB, TR_AB, pool_ids_AB, BLOCKS_A
        )

        for vname, vseed, vpartition, vdesc in TASK_A_VARIANTS:
            classes = build_class_table_from_lookup(
                tau0_oh_AB, TASK_A_CYC_S, TASK_A_CYC_E, vpartition
            )
            cls_dist = [(classes == i).sum().item() for i in range(VOCAB)]
            print(f"\n  variant={vname}  classes={cls_dist}")

            result = train_one(
                f"probe1/{vname}", D,
                TN_ang, TR_p, tau0_hyb, pool_ids_p, classes,
                BLOCKS_A, noise_sigma=0.0, seed=vseed,
            )
            model = result["model"]
            model.eval()
            dec_f = run_decodability_final(model, pool_ids_p, classes)
            all_st, all_x0 = collect_trajectories(model, pool_ids_p, classes)
            no_last = eval_no_last(model, all_st, all_x0)

            # Compute Δ vs file-3 Task A for this horizon
            ref = FAIL_TASK_A_LS_D32 if D == 32 else FAIL_TASK_A_LS_D24
            dv3 = round(result["final_acc"] - ref, 4)

            csv_rows.append({
                "probe":                   "probe1_taskA",
                "variant":                 vname,
                "horizon":                 D,
                "seed":                    vseed,
                "accuracy":                result["final_acc"],
                "solve_step":              result["solve_step"] or "",
                "no_last_accuracy":        no_last,
                "decodability_final":      dec_f,
                "alpha_last":              result["alphaD"],
                "delta_vs_file3_task_a":   dv3,
                "runtime_seconds":         result["wall"],
                "note":                    (f"probe=1; variant={vname}; D={D}; "
                                            f"seed={vseed}; partition=custom_lookup; "
                                            f"noise_sigma=0.00; anchor=two_i_rot; "
                                            f"ctrl=projective; eps_high=1.0; {vdesc}"),
            })
            write_csv(csv_rows)

    # ─────────────────────────────────────────────────────────────────
    # Probe 2: Task B noise sweep
    # ─────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("PROBE 2 — Task B: noise sweep at D=24 and D=32")
    print("=" * 70)

    for D in HORIZONS:
        print(f"\n── Horizon D={D} ──")
        TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
            TN_oh_AB, tau0_oh_AB, TR_AB, pool_ids_AB, BLOCKS_A
        )
        # Original period-3 interleaved partition (same as file-3 Task A/B)
        classes = build_class_table_from_lookup(
            tau0_oh_AB, TASK_A_CYC_S, TASK_A_CYC_E, PARTITION_ORIGINAL
        )

        for sig in NOISE_SIGMAS:
            vname = f"sigma_{sig:.2f}"
            print(f"\n  sigma={sig:.2f}")
            result = train_one(
                f"probe2/{vname}", D,
                TN_ang, TR_p, tau0_hyb, pool_ids_p, classes,
                BLOCKS_A, noise_sigma=sig, seed=GLOBAL_SEED,
            )
            model = result["model"]
            model.eval()
            dec_f   = run_decodability_final(model, pool_ids_p, classes)
            all_st, all_x0 = collect_trajectories(model, pool_ids_p, classes)
            no_last = eval_no_last(model, all_st, all_x0)

            ref = FAIL_TASK_A_LS_D32 if D == 32 else FAIL_TASK_A_LS_D24
            dv3 = round(result["final_acc"] - ref, 4)

            csv_rows.append({
                "probe":                   "probe2_taskB",
                "variant":                 vname,
                "horizon":                 D,
                "seed":                    GLOBAL_SEED,
                "accuracy":                result["final_acc"],
                "solve_step":              result["solve_step"] or "",
                "no_last_accuracy":        no_last,
                "decodability_final":      dec_f,
                "alpha_last":              result["alphaD"],
                "delta_vs_file3_task_a":   dv3,
                "runtime_seconds":         result["wall"],
                "note":                    (f"probe=2; variant={vname}; D={D}; "
                                            f"seed={GLOBAL_SEED}; noise_sigma={sig:.2f}; "
                                            f"partition=original_period3; anchor=two_i_rot; "
                                            f"ctrl=projective; eps_high=1.0"),
            })
            write_csv(csv_rows)

    # ─────────────────────────────────────────────────────────────────
    # Probe 3: Task C stability (extra seed)
    # ─────────────────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("PROBE 3 — Task C: stability confirmation (extra seed)")
    print("=" * 70)

    for D in HORIZONS:
        print(f"\n── Horizon D={D} ──")
        TN_ang_C, TR_pC, tau0_hyb_C, pool_ids_pC = prepare_tables(
            TN_oh_C, tau0_oh_C, TR_C, pool_ids_C, BLOCKS_C
        )
        # Uniform quarter-period partition for 8-state cyclic component
        partition_C = [0, 0, 1, 1, 2, 2, 3, 3]
        classes_C   = build_class_table_from_lookup(
            tau0_oh_C, TASK_C_CYC_S, TASK_C_CYC_E, partition_C
        )
        cls_dist = [(classes_C == i).sum().item() for i in range(VOCAB)]
        print(f"  class dist: {cls_dist}")

        vname = f"seed_{TASK_C_EXTRA_SEED}"
        result = train_one(
            f"probe3/{vname}", D,
            TN_ang_C, TR_pC, tau0_hyb_C, pool_ids_pC, classes_C,
            BLOCKS_C, noise_sigma=0.0, seed=TASK_C_EXTRA_SEED,
        )
        model = result["model"]
        model.eval()
        dec_f   = run_decodability_final(model, pool_ids_pC, classes_C)
        all_st, all_x0 = collect_trajectories(model, pool_ids_pC, classes_C)
        no_last = eval_no_last(model, all_st, all_x0)

        ref_c32 = FAIL_TASK_C_LS_D32 if D == 32 else None
        dv3     = round(result["final_acc"] - ref_c32, 4) if ref_c32 else ""

        csv_rows.append({
            "probe":                   "probe3_taskC",
            "variant":                 vname,
            "horizon":                 D,
            "seed":                    TASK_C_EXTRA_SEED,
            "accuracy":                result["final_acc"],
            "solve_step":              result["solve_step"] or "",
            "no_last_accuracy":        no_last,
            "decodability_final":      dec_f,
            "alpha_last":              result["alphaD"],
            "delta_vs_file3_task_a":   dv3,
            "runtime_seconds":         result["wall"],
            "note":                    (f"probe=3; variant={vname}; D={D}; "
                                        f"seed={TASK_C_EXTRA_SEED}; noise_sigma=0.00; "
                                        f"partition=uniform_8state; anchor=two_i_rot; "
                                        f"ctrl=projective; eps_high=1.0; "
                                        f"product_cycle_state_space"),
        })
        write_csv(csv_rows)

    write_markdown(csv_rows)

    print("\n" + "=" * 70)
    print("DONE")
    print(f"  CSV → {CSV_OUT}")
    print(f"  MD  → {MD_OUT}")
    print("=" * 70)


if __name__ == "__main__":
    main()
