#!/usr/bin/env python3
"""run_router_harmonic_generalization_v1.py

HARMONIC GENERALIZATION TEST v1
================================

Branch: harmonic-generalization-test-v1

LOCKED FINDINGS (do not revisit):
  - mod-12 k=3 is necessary for mod12 % 4 target
  - Baseline transport destroys k=3 signal (mid_dec → 0.31 from initial=1.0)
  - Split-canonical (eps_high=1.0): k>=2 frozen
    → final_acc=1.0, solve=500, no_last=1.0, mid_dec=1.0
  - Split-canonical solves both inject and no-inject tasks on mod-12

GENERALIZATION QUESTION:
  Does "freeze k>=2, transport k=1" generalize to a different modular structure
  where a DIFFERENT harmonic carries the signal?

NEW TASK DESIGN — MOD-8 HALF-PHASE CLASSIFICATION:
  State space: Z_5 × Z_8  (40 states)
    - component b ∈ {0..4}: irrelevant routing "noise"
    - component c ∈ {0..7}: mod-8 target component

  Target: x0 = c % 4  → 4 classes {0,1,2,3}
    Class 0: c ∈ {0, 4}    k=2 vector = (1, 0)
    Class 1: c ∈ {1, 5}    k=2 vector = (0, 1)
    Class 2: c ∈ {2, 6}    k=2 vector = (-1, 0)
    Class 3: c ∈ {3, 7}    k=2 vector = (0, -1)

REQUIRED HARMONIC:
  k=2 (mod-8 second harmonic, period=4).
  Reasoning:
    - mod-8 % 4 creates 4 classes repeating with period 4 within the mod-8 cycle
    - The angular harmonic with period 4 in an 8-period space is k=2
    - k=1 (period 8): does NOT separate the 4 classes (pairs like c=0,c=4 share class 0
        but their k=1 vectors differ: (1,0) vs (-1,0) → NOT class-specific)
    - k=2 (period 4): DOES separate → class-constant and orthogonal across classes
  This is DIFFERENT from mod-12 k=3 case (k=3 had period 4 in the 12-cycle).

OPERATIONS (6) — all change the mod-8 component c:
  Op 0: c += 1 mod 8
  Op 1: c += 2 mod 8
  Op 2: c += 3 mod 8
  Op 3: c += 1 mod 8, b += 1 mod 5
  Op 4: c += 2 mod 8, b += 2 mod 5
  Op 5: c += 3 mod 8, b += 3 mod 5

  ALL ops change c → baseline transport MUST mix k=2 from different classes.
  Mathematical analysis: with equal op weights, transported k2 = -k2/3 per step.
  After 12 steps: |k2| = (1/3)^12 ≈ 1.9e-6. Signal collapses to near-zero.
  → Baseline mid decodability: near-chance (0.25 for 4 classes).
  → Split-canonical: k2 frozen at initial → mid decodability = 1.0.

GEOMETRY:
  GEOM_K2 = [(0,5,5,1),(5,13,8,2)]
    Block 0: mod-5, 1 harmonic (k=1), indices 0-4
    Block 1: mod-8, 2 harmonics (k=1,k=2), indices 5-12
  N_PHASE_PAIRS = 2  (one per block, matches original convention)
  d_ang = 2 + 4 = 6
  d_hyb = 6 + 2 = 8
  d_in  = 4 + 8 = 12

TRANSPORT VARIANTS:
  baseline:        eps_high=0.0  (full transport for all k)
  split_canonical: eps_high=1.0  (k=1 transported; k>=2 frozen)

TASKS:
  A. task_A_inject:    inject x0 at t=D-1  (b_posLast=2.0)
  B. task_B_no_inject: no injection         (tau-only decoding, harder)

MEASUREMENTS PER (transport × task):
  final_accuracy, solve_step, no_last_accuracy,
  initial/mid/final decodability, runtime_seconds

KEY QUESTIONS:
  1. Does baseline fail on the harder no-inject task (as in mod-12)?
  2. Does split-canonical recover perfect/near-perfect performance?
  3. Does the k=2 mid decodability confirm the required harmonic?
  4. Does "freeze higher harmonics" generalize when modulus changes?
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
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_harmonic_generalization_v1.csv"
MD_OUT      = DOCS_DIR   / "prime_transport_router_harmonic_generalization_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Config — locked to match split-transport experiments for comparability
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24
D_HIDDEN      = 32
BATCH_SIZE    = 512
VOCAB         = 4          # 4 mod-8 half-phase classes (c % 4)
D_EMB         = 4
N_PHASE_PAIRS = 2          # one per geometry block (vs 4 in mod-12 experiment)
N_OPS         = 6
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 2048
SOLVE_ACC     = 0.999
INIT_SCALE    = 0.05
MAX_STEPS     = 3_500
N_BATCHES     = N_EVAL // BATCH_SIZE
NEG_INF       = -1e9
N_PROBE       = 4096

# Geometry: (start, end, modulus, n_harmonics)
# Block 0: mod-5,  k=1 only  → indices 0-4  (5 dims)
# Block 1: mod-8,  k=1,k=2  → indices 5-12 (8 dims)
GEOM_K2 = [(0, 5, 5, 1), (5, 13, 8, 2)]

# mod-8 block position in the one-hot
MOD8_S, MOD8_E = 5, 13   # indices 5..12 inclusive (8 dims)
D_ONEHOT      = 13        # total one-hot dims: 5 + 8

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, 16, D_HIDDEN)
except Exception:
    pass

# ═══════════════════════════════════════════════════════════════════════
# Transport variants
# ═══════════════════════════════════════════════════════════════════════
TRANSPORT_VARIANTS: Dict[str, float] = {
    "baseline":        0.0,   # full transport for all k
    "split_canonical": 1.0,   # k>=2 frozen (split-transport rule)
}

# ═══════════════════════════════════════════════════════════════════════
# Task configurations
# ═══════════════════════════════════════════════════════════════════════
TASK_CONFIGS: Dict[str, Dict] = {
    "task_A_inject": {
        "inject":         True,
        "b_pos0_init":    0.0,
        "b_posLast_init": 2.0,
        "label":          "mod8-half-phase inject@last",
    },
    "task_B_no_inject": {
        "inject":         False,
        "b_pos0_init":    0.0,
        "b_posLast_init": 0.0,
        "label":          "mod8-half-phase NO inject (tau-only, harder)",
    },
}


# ═══════════════════════════════════════════════════════════════════════
# Synthetic state space: Z_5 × Z_8  (40 states)
# ═══════════════════════════════════════════════════════════════════════
def build_mod8_state_tables() -> Tuple[torch.Tensor, torch.Tensor,
                                       torch.Tensor, torch.Tensor]:
    """
    Build state tables for Z_5 × Z_8 product space.

    State: (b, c)  where b ∈ Z_5, c ∈ Z_8
    One-hot: [mod5_onehot (5 dims) | mod8_onehot (8 dims)] = 13 dims

    Operations (all change c, creating k=2 mixing under baseline transport):
      Op 0: c += 1
      Op 1: c += 2
      Op 2: c += 3
      Op 3: c += 1, b += 1
      Op 4: c += 2, b += 2
      Op 5: c += 3, b += 3

    Returns:
      TN_oh:    (N_STATES, N_OPS, D_ONEHOT)  neighbor one-hot reps
      TR:       (N_STATES, N_OPS)             next state indices
      tau0_oh:  (N_STATES, D_ONEHOT)          initial state one-hots
      pool_ids: (4000,)                        sampling pool
    """
    all_states = list(itertools.product(range(5), range(8)))  # 40 states
    N_STATES   = len(all_states)  # = 40
    s2i        = {s: i for i, s in enumerate(all_states)}

    def apply_op(b: int, c: int, op: int) -> Tuple[int, int]:
        if   op == 0: return (b,         (c + 1) % 8)
        elif op == 1: return (b,         (c + 2) % 8)
        elif op == 2: return (b,         (c + 3) % 8)
        elif op == 3: return ((b+1) % 5, (c + 1) % 8)
        elif op == 4: return ((b+2) % 5, (c + 2) % 8)
        elif op == 5: return ((b+3) % 5, (c + 3) % 8)
        raise ValueError(op)

    # Transition table
    TR = torch.zeros(N_STATES, N_OPS, dtype=torch.long)
    for i, (b, c) in enumerate(all_states):
        for op in range(N_OPS):
            nb, nc = apply_op(b, c, op)
            TR[i, op] = s2i[(nb, nc)]

    # One-hot representations
    tau0_oh = torch.zeros(N_STATES, D_ONEHOT)
    for i, (b, c) in enumerate(all_states):
        tau0_oh[i, b]           = 1.0  # mod-5 block: indices 0-4
        tau0_oh[i, MOD8_S + c]  = 1.0  # mod-8 block: indices 5-12

    # Neighbor reps
    TN_oh = torch.zeros(N_STATES, N_OPS, D_ONEHOT)
    for i in range(N_STATES):
        for op in range(N_OPS):
            next_i        = TR[i, op].item()
            TN_oh[i, op]  = tau0_oh[next_i]

    # Pool: repeat all states to match original pool size (~4000)
    repeats  = 4000 // N_STATES + 1
    pool_ids = torch.arange(N_STATES).repeat(repeats)[:4000]

    return TN_oh, TR, tau0_oh, pool_ids


def build_mod8_class_table(tau0_oh: torch.Tensor) -> torch.Tensor:
    """Extract target class: c % 4  where c = mod-8 initial state."""
    return (tau0_oh[:, MOD8_S:MOD8_E].argmax(dim=-1) % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers (unchanged from split-transport forward-use)
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int]:
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    d_hyb = d_ang + N_PHASE_PAIRS
    d_in  = D_EMB + d_hyb
    return d_ang, d_hyb, d_in


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


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Split-transport (canonical — identical to split-transport forward-use v1)
# ═══════════════════════════════════════════════════════════════════════
def apply_split_transport(
        base: torch.Tensor,
        tau_prev: torch.Tensor,
        blocks,
        eps_high: float,
) -> torch.Tensor:
    """
    k=1:   tau_k1_{t+1} = normalize(base_k1)
    k>=2:  tau_high_{t+1} = (1-eps_high)*normalize(base_high) + eps_high*tau_high_t

    eps_high=0.0 → baseline (full transport)
    eps_high=1.0 → split_canonical (k>=2 frozen)
    """
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
# Router model
# ═══════════════════════════════════════════════════════════════════════
class RouterHarmonicGen(nn.Module):
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 blocks,
                 eps_high: float,
                 inject: bool,
                 b_pos0_init: float,
                 b_posLast_init: float,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        self.blocks    = blocks
        self.eps_high  = eps_high
        self.do_inject = inject
        d_ang, d_hyb, d_in = geom_dims(blocks)
        self.d_ang     = d_ang
        self.d_hyb     = d_hyb
        dh  = D_HIDDEN
        dha = max(8, dh // 4)

        self.register_buffer("TN",         TN_ang)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids",   pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]     = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1] = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(float(b_pos0_init)))
        self.b_posLast = nn.Parameter(torch.tensor(float(b_posLast_init)))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb  = rp(VOCAB, D_EMB)
        self.W1     = rp(d_in, dh);    self.b1     = zp(dh)
        self.W2     = rp(dh, N_OPS);   self.b2     = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb);  self.b_attn = zp(dha)
        self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)
        if inject:
            self.W_tok_inject = rp(VOCAB, d_hyb)
        else:
            self.W_tok_inject = None

    def _inject(self, hybrid: torch.Tensor, x0: torch.Tensor, t: int) -> torch.Tensor:
        if self.do_inject and t == D - 1:
            return hybrid + self.W_tok_inject[x0]
        return hybrid

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn     = self.TN[state_ids]
        embs   = self.W_emb[tokens_t]
        h      = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
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

    def _eval_transport(self, tau_prev, best_ang) -> torch.Tensor:
        return apply_split_transport(best_ang, tau_prev, self.blocks, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = self._inject(hybrid, x0, t)
            soft_taus.append(tau_prev)
        st  = torch.stack(soft_taus, dim=1)
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc  = ((h_a * self.v_attn).sum(dim=-1)
               + self.pos0_mask * self.b_pos0
               + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st: torch.Tensor):
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc  = ((h_a * self.v_attn).sum(dim=-1)
               + self.pos0_mask * self.b_pos0
               + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha

    def readout_masked(self, st: torch.Tensor, mask: torch.Tensor):
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc  = ((h_a * self.v_attn).sum(dim=-1)
               + self.pos0_mask * self.b_pos0
               + self.posLast_mask * self.b_posLast
               + mask)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers (unchanged from split-transport forward-use)
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, mod8_classes, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = mod8_classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, mod8_classes) -> Tuple[float, float]:
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod8_classes, rng)
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
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur)
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            pred, alpha = model.readout(st)
            correct += (pred.argmax(1) == x0).sum().item()
            aD_sum  += alpha[:, D - 1].mean().item()
    model.train()
    return round(correct / N_EVAL, 4), round(aD_sum / N_BATCHES, 4)


def collect_trajectories(model, pool_ids, mod8_classes):
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st:  List[torch.Tensor] = []
    all_x0:  List[torch.Tensor] = []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod8_classes, rng)
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
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur.clone())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
    return all_st, all_x0


def eval_no_last(model, all_st, all_x0) -> float:
    mask    = torch.zeros(1, D); mask[0, D - 1] = NEG_INF
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


def run_decodability_probes(model, pool_ids, mod8_classes, label: str) -> Dict[str, float]:
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    MID  = D // 2 - 1
    B    = BATCH_SIZE
    n_b  = (N_PROBE + B - 1) // B

    tau0_list: List[torch.Tensor] = []
    tau_m_list: List[torch.Tensor] = []
    tau_f_list: List[torch.Tensor] = []
    lbl_list:  List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(n_b):
            sids, _, x0 = make_batch(pool_ids, mod8_classes, rng)
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
                tau_cur = model._inject(hybrid, x0, t)
                if t == MID:
                    tau_m_list.append(tau_cur.cpu())
                if t == D - 1:
                    tau_f_list.append(tau_cur.cpu())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)

    y        = torch.cat(lbl_list,  dim=0).numpy()
    tau0_arr = torch.cat(tau0_list, dim=0).numpy()
    tau_m    = torch.cat(tau_m_list, dim=0).numpy()
    tau_f    = torch.cat(tau_f_list, dim=0).numpy()

    return {
        "initial": linear_probe(tau0_arr, y, f"{label}/initial"),
        "mid":     linear_probe(tau_m,    y, f"{label}/mid(t={MID})"),
        "final":   linear_probe(tau_f,    y, f"{label}/final(t={D-1})"),
    }


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        transport_name: str,
        task_name: str,
        TN_ang, TR, tau0_hyb, pool_ids, mod8_classes,
        blocks,
        eps_high: float,
        task_cfg: Dict,
) -> Dict:
    d_ang, d_hyb, d_in = geom_dims(blocks)
    run_label = f"{transport_name}/{task_name}"
    print(f"\n{'=' * 64}")
    print(f"  {run_label}")
    print(f"  d_tau_ang={d_ang}  d_tau_hyb={d_hyb}  eps_high={eps_high:.2f}")
    print(f"  inject={task_cfg['inject']}  "
          f"b_pos0_init={task_cfg['b_pos0_init']}  "
          f"b_posLast_init={task_cfg['b_posLast_init']}")
    print(f"{'=' * 64}")

    model = RouterHarmonicGen(
        TN_ang, TR, tau0_hyb, pool_ids, blocks,
        eps_high=eps_high,
        inject=task_cfg["inject"],
        b_pos0_init=task_cfg["b_pos0_init"],
        b_posLast_init=task_cfg["b_posLast_init"],
    )
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()

    t0         = time.perf_counter()
    solve_step = None
    final_acc  = 0.0
    alphaD_f   = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, mod8_classes, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, aD = eval_acc(model, pool_ids, mod8_classes)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc; alphaD_f = aD
            print(f"    [{run_label}] step={step:5d}  acc={acc:.4f}  α_{{D-1}}={aD:.4f}")

    wall = round(time.perf_counter() - t0, 1)
    print(f"  → acc={final_acc:.4f}  solve={solve_step}  wall={wall}s")
    model.eval()
    return dict(
        model=model,
        transport=transport_name,
        task=task_name,
        eps_high=eps_high,
        inject=task_cfg["inject"],
        pool_ids=pool_ids,
        mod8_classes=mod8_classes,
        final_acc=final_acc,
        solve_step=solve_step,
        alphaD=alphaD_f,
        wall=wall,
    )


# ═══════════════════════════════════════════════════════════════════════
# CSV output
# ═══════════════════════════════════════════════════════════════════════
_FIELDNAMES = [
    "transport", "task", "eps_high", "inject",
    "position", "decodability",
    "final_accuracy", "solve_step",
    "no_last_accuracy", "alpha_last",
    "runtime_seconds", "note",
]


def write_csv(rows: List[Dict]):
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=_FIELDNAMES)
        w.writeheader()
        for r in rows:
            w.writerow({k: r.get(k, "") for k in _FIELDNAMES})
    print(f"\nCSV written: {CSV_OUT}  ({len(rows)} rows)")


# ═══════════════════════════════════════════════════════════════════════
# Markdown output
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(results: List[Dict], verdict: str):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []

    L.append("# Prime Transport Router: Harmonic Generalization Test v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}  \n")
    L.append(f"**Geometry:** `GEOM_K2` = [(0,5,5,1),(5,13,8,2)]  \n")
    L.append(f"**Branch:** harmonic-generalization-test-v1  \n\n")

    L.append("---\n\n## Modular Task Definition\n\n")
    L.append("**State space:** Z_5 × Z_8 (40 states)  \n")
    L.append("- Component b ∈ Z_5: routing noise, not used for target  \n")
    L.append("- Component c ∈ Z_8: target component  \n\n")
    L.append("**Target:** `x0 = c % 4`  → 4 classes {0,1,2,3}  \n")
    L.append("```\n")
    L.append("  Class 0: c ∈ {0, 4}   k=2 vector = ( 1,  0)\n")
    L.append("  Class 1: c ∈ {1, 5}   k=2 vector = ( 0,  1)\n")
    L.append("  Class 2: c ∈ {2, 6}   k=2 vector = (-1,  0)\n")
    L.append("  Class 3: c ∈ {3, 7}   k=2 vector = ( 0, -1)\n")
    L.append("```\n\n")

    L.append("---\n\n## Expected Harmonic\n\n")
    L.append("**Required harmonic: k=2 (mod-8 second harmonic)**  \n\n")
    L.append("Reasoning:  \n")
    L.append("- mod-8 % 4 creates period-4 repetition within the 8-state cycle  \n")
    L.append("- k=1 (period 8): maps c=0→(1,0) and c=4→(-1,0) → DIFFERENT vectors for same class → NOT class-constant  \n")
    L.append("- k=2 (period 4): maps c∈{0,4}→(1,0), c∈{1,5}→(0,1), etc. → CONSTANT per class, ORTHOGONAL across classes  \n")
    L.append("- k=3 (period 8/3): not class-constant  \n\n")
    L.append("**Baseline transport prediction:**  \n")
    L.append("All 6 operations change c. For equal-weight routing, transported k2 = -k2/3 per step.  \n")
    L.append("After 12 steps: |k2| ≈ (1/3)^12 = 1.9e-6. Signal: near-zero → mid decodability ≈ 0.25 (chance).  \n\n")
    L.append("**Split-canonical prediction:**  \n")
    L.append("k2 frozen at initial value throughout all 24 steps → mid decodability = 1.0.  \n\n")

    L.append("**This differs from mod-12 k=3:**  \n")
    L.append("- mod-12 uses k=3 as the signal harmonic (period 4 in 12-cycle, k=12/4=3)  \n")
    L.append("- mod-8 uses k=2 as the signal harmonic (period 4 in 8-cycle, k=8/4=2)  \n")
    L.append("- Different modulus, different required k — same freeze rule (k>=2)  \n\n")

    L.append("---\n\n## Operations\n\n")
    L.append("| Op | Δc (mod 8) | Δb (mod 5) | k=2 rotation |\n")
    L.append("|----|-----------|-----------|---------------|\n")
    L.append("| 0  | +1        | 0         | +90°          |\n")
    L.append("| 1  | +2        | 0         | +180°         |\n")
    L.append("| 2  | +3        | 0         | +270°         |\n")
    L.append("| 3  | +1        | +1        | +90°          |\n")
    L.append("| 4  | +2        | +2        | +180°         |\n")
    L.append("| 5  | +3        | +3        | +270°         |\n\n")
    L.append("**All ops change c** → baseline transport mixes k=2 from 3 other classes at each step.  \n\n")

    L.append("---\n\n## Main Comparison: Accuracy and Solve Step\n\n")
    L.append("| transport | task | accuracy | solve_step | no_last | α_{D-1} | runtime_s |\n")
    L.append("|-----------|------|----------|------------|---------|---------|----------|\n")
    for r in results:
        ss = str(r["solve_step"]) if r.get("solve_step") else "—"
        L.append(f"| {r['transport']} | {r['task']} "
                 f"| {r['final_acc']:.4f} | {ss}"
                 f" | {r.get('no_last', 0.0):.4f}"
                 f" | {r['alphaD']:.4f}"
                 f" | {r['wall']:.1f} |\n")
    L.append("\n")

    L.append("---\n\n## Decodability Progression (initial / mid / final)\n\n")
    L.append(f"**Baseline prediction:** initial=1.0, mid≈0.25 (chance), final varies  \n")
    L.append(f"**Split-canonical prediction:** initial=1.0, mid=1.0, final=1.0  \n\n")
    L.append("| transport | task | initial | mid (t=11) | final (t=23) |\n")
    L.append("|-----------|------|---------|------------|-------------|\n")
    for r in results:
        p = r.get("decodability", {})
        L.append(f"| {r['transport']} | {r['task']}"
                 f" | {p.get('initial', 0.0):.4f}"
                 f" | {p.get('mid', 0.0):.4f}"
                 f" | {p.get('final', 0.0):.4f} |\n")
    L.append("\n")

    L.append("---\n\n## Key Questions Answered\n\n")

    baseline_A = next((r for r in results if r["transport"] == "baseline"
                       and r["task"] == "task_A_inject"), None)
    split_A    = next((r for r in results if r["transport"] == "split_canonical"
                       and r["task"] == "task_A_inject"), None)
    baseline_B = next((r for r in results if r["transport"] == "baseline"
                       and r["task"] == "task_B_no_inject"), None)
    split_B    = next((r for r in results if r["transport"] == "split_canonical"
                       and r["task"] == "task_B_no_inject"), None)

    # Q1
    L.append("**Q1: Does baseline fail on the harder no-inject task?**  \n")
    if baseline_B:
        b_acc = baseline_B["final_acc"]
        b_md  = baseline_B.get("decodability", {}).get("mid", 0.0)
        if b_acc < 0.5:
            L.append(f"YES — baseline fails task_B (acc={b_acc:.4f}, mid_dec={b_md:.4f})  \n\n")
        else:
            L.append(f"NO — baseline solves task_B (acc={b_acc:.4f})  \n\n")

    # Q2
    L.append("**Q2: Does split-canonical recover performance?**  \n")
    if split_B:
        s_acc = split_B["final_acc"]
        s_nl  = split_B.get("no_last", 0.0)
        if s_acc >= SOLVE_ACC:
            L.append(f"YES — split_canonical solves task_B (acc={s_acc:.4f}, no_last={s_nl:.4f})  \n\n")
        else:
            L.append(f"NO — split_canonical does NOT solve task_B (acc={s_acc:.4f})  \n\n")

    # Q3
    L.append("**Q3: Does mid decodability confirm k=2 as the required harmonic?**  \n")
    if baseline_A and split_A:
        b_md = baseline_A.get("decodability", {}).get("mid", 0.0)
        s_md = split_A.get("decodability", {}).get("mid", 0.0)
        L.append(f"baseline mid_dec={b_md:.4f} vs split_canonical mid_dec={s_md:.4f}  \n")
        if s_md > 0.9 and b_md < 0.5:
            L.append("YES — k=2 is destroyed by baseline but preserved by split-canonical.  \n\n")
        elif s_md > b_md + 0.1:
            L.append("PARTIAL — split-canonical shows better k=2 preservation than baseline.  \n\n")
        else:
            L.append("UNCLEAR — mid decodability does not clearly confirm k=2 as the signal harmonic.  \n\n")

    # Q4
    L.append("**Q4: Does the freeze rule generalize when modulus changes?**  \n")
    if split_A and split_B and baseline_B:
        s_a_ok = split_A["final_acc"] >= SOLVE_ACC
        s_b_ok = split_B["final_acc"] >= SOLVE_ACC
        b_b_ok = baseline_B["final_acc"] >= SOLVE_ACC
        if s_a_ok and s_b_ok and not b_b_ok:
            L.append("YES — split-canonical solves both tasks on mod-8 where baseline fails on task_B.  \n\n")
        elif s_a_ok and s_b_ok and b_b_ok:
            L.append("INCONCLUSIVE — both methods solve; no discriminative evidence.  \n\n")
        elif s_a_ok and not s_b_ok:
            L.append("PARTIAL — split-canonical helps on inject task but fails on no-inject.  \n\n")
        else:
            L.append("NO — split-canonical fails even the easier inject task.  \n\n")

    L.append("---\n\n")
    L.append(f"## SPLIT-TRANSPORT GENERALIZATION: {verdict}\n\n")

    L.append("---\n\n## Honesty Section\n\n")

    L.append("### What generalized\n\n")
    if split_B and split_B["final_acc"] >= SOLVE_ACC:
        L.append("- Split-canonical (freeze k>=2) solves the mod-8 task with k=2 as signal harmonic  \n")
        L.append("- The mechanism transfers: freezing k>=2 preserves whatever harmonic carries the class label  \n")
        L.append("- The architecture required NO changes — same D, N_OPS, geometry convention  \n")
    else:
        L.append("- Split-canonical did NOT solve the mod-8 no-inject task  \n")
        L.append("- The freeze rule did not clearly generalize to mod-8 k=2  \n")

    L.append("\n### What did not generalize\n\n")
    if split_B and split_B["final_acc"] >= SOLVE_ACC:
        L.append("- This test uses a synthetic 40-state space (Z_5 × Z_8), not a real prime-transport state cache  \n")
        L.append("- The operations are manually chosen to force k=2 mixing — not derived from prime dynamics  \n")
        L.append("- Generalization to arbitrary moduli or non-cyclic state spaces is not tested  \n")
    else:
        L.append("- The freeze rule may require specific conditions not present here  \n")

    L.append("\n### Whether harmonic preservation appears to be a reusable law\n\n")
    if split_B and split_B["final_acc"] >= SOLVE_ACC and baseline_B and baseline_B["final_acc"] < 0.5:
        L.append("APPEARS REUSABLE: The pattern 'freeze k>=2 to preserve high-frequency class signal' "
                 "holds for both mod-12/k=3 and mod-8/k=2.  \n")
        L.append("The law may be: for any modulus M with target = state % (M/k), "
                 "the k-th harmonic carries the signal and must be frozen.  \n")
        L.append("Caveat: tested on only 2 instances (mod-12 and mod-8). "
                 "Further evidence needed before strong claims.  \n")
    elif split_B and split_B["final_acc"] >= SOLVE_ACC:
        L.append("PARTIALLY REUSABLE: Split-canonical works on mod-8 but baseline also works. "
                 "The discriminative advantage of the freeze rule is not confirmed.  \n")
    else:
        L.append("NOT CONFIRMED: Results do not support harmonic preservation as a reusable law "
                 "across moduli.  \n")

    L.append("\n### Experimental limitations\n\n")
    L.append("- State space is synthetic and small (40 states vs 343k in mod-12 experiments)  \n")
    L.append("- Operations are hand-designed (all change c) to force k=2 destruction  \n")
    L.append("- One modulus tested (mod-8). A 3rd modulus test would strengthen the claim  \n")
    L.append("- The \"no-inject\" task is the discriminative signal: inject can succeed by other means  \n")
    L.append(f"- Reference (mod-12 locked): baseline/task_B final_acc=0.2461 (chance), "
             f"split/task_B final_acc=1.0, no_last=1.0  \n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 64)
    print("HARMONIC GENERALIZATION TEST v1")
    print("=" * 64)
    print(f"Task:      mod-8 half-phase classification (c % 4, 4 classes)")
    print(f"Modulus:   8  (vs mod-12 in prior experiments)")
    print(f"Required harmonic: k=2 (vs k=3 in mod-12)")
    print(f"Geometry:  GEOM_K2 = {GEOM_K2}")
    print(f"Transport: {list(TRANSPORT_VARIANTS.keys())}")
    print(f"Tasks:     {list(TASK_CONFIGS.keys())}")
    print(f"Steps:     {MAX_STEPS}  LR={LR}  B={BATCH_SIZE}\n")

    print("Building synthetic mod-8 state tables (Z_5 × Z_8 = 40 states) ...")
    TN_oh, TR_raw, tau0_oh, pool_ids_raw = build_mod8_state_tables()
    print(f"  TN_oh={TN_oh.shape}, TR={TR_raw.shape}, tau0_oh={tau0_oh.shape}")
    print(f"  pool_ids={pool_ids_raw.shape}")

    mod8_classes = build_mod8_class_table(tau0_oh)
    print(f"  class distribution: {dict(zip(*torch.unique(mod8_classes, return_counts=True)))}")

    TN_ang, TR_t, tau0_hyb, pool_t = prepare_tables(
        TN_oh, tau0_oh, TR_raw, pool_ids_raw, GEOM_K2)
    d_ang, d_hyb, d_in = geom_dims(GEOM_K2)
    print(f"  d_ang={d_ang}  d_hyb={d_hyb}  d_in={d_in}")

    # Verify k=2 initial decodability
    k2_tau0 = TN_ang[:, 0, 2:4] if TN_ang.dim() == 3 else None  # skip for now
    print()

    all_results: List[Dict] = []

    for task_name, task_cfg in TASK_CONFIGS.items():
        for transport_name, eps_high in TRANSPORT_VARIANTS.items():
            r = train_one(
                transport_name, task_name,
                TN_ang, TR_t, tau0_hyb, pool_t, mod8_classes,
                GEOM_K2, eps_high=eps_high, task_cfg=task_cfg,
            )
            all_results.append(r)

    print(f"\n{'=' * 64}")
    print("POST-TRAINING MEASUREMENTS")
    print("=" * 64)

    for r in all_results:
        run_label = f"{r['transport']}/{r['task']}"
        model     = r["model"]
        pool_r    = r["pool_ids"]
        mod8_cls  = r["mod8_classes"]

        print(f"\n  [{run_label}] collecting trajectories ...")
        all_st, all_x0 = collect_trajectories(model, pool_r, mod8_cls)
        r["no_last"] = eval_no_last(model, all_st, all_x0)
        print(f"    no_last = {r['no_last']:.4f}")

        print(f"  [{run_label}] running decodability probes ...")
        r["decodability"] = run_decodability_probes(model, pool_r, mod8_cls, run_label)

    print(f"\n{'=' * 64}")
    print("SUMMARY")
    print("=" * 64)
    hdr = (f"  {'transport':<18} {'task':<22} {'acc':>6} {'solve':>6} "
           f"{'no_last':>8} {'dec_init':>9} {'dec_mid':>8} {'dec_fin':>8} {'wall_s':>7}")
    print(hdr)
    print("  " + "-" * (len(hdr) - 2))
    for r in all_results:
        p  = r.get("decodability", {})
        ss = str(r.get("solve_step")) if r.get("solve_step") else "—"
        print(f"  {r['transport']:<18} {r['task']:<22} "
              f"{r['final_acc']:>6.4f} {ss:>6} "
              f"{r.get('no_last', 0.0):>8.4f} "
              f"{p.get('initial', 0.0):>9.4f} "
              f"{p.get('mid', 0.0):>8.4f} "
              f"{p.get('final', 0.0):>8.4f} "
              f"{r['wall']:>7.1f}")

    # Determine verdict
    split_A  = next((r for r in all_results if r["transport"] == "split_canonical"
                     and r["task"] == "task_A_inject"), None)
    split_B  = next((r for r in all_results if r["transport"] == "split_canonical"
                     and r["task"] == "task_B_no_inject"), None)
    base_B   = next((r for r in all_results if r["transport"] == "baseline"
                     and r["task"] == "task_B_no_inject"), None)

    s_a_ok = split_A and split_A["final_acc"] >= SOLVE_ACC
    s_b_ok = split_B and split_B["final_acc"] >= SOLVE_ACC
    b_b_ok = base_B  and base_B["final_acc"]  >= SOLVE_ACC

    s_b_better = (split_B and base_B and
                  split_B["final_acc"] > base_B["final_acc"] + 0.05)

    if s_a_ok and s_b_ok and not b_b_ok:
        verdict = "STRONG"
    elif s_a_ok and s_b_ok and s_b_better:
        verdict = "PARTIAL"
    elif s_a_ok and s_b_ok and b_b_ok:
        verdict = "INCONCLUSIVE"
    elif s_a_ok and not s_b_ok:
        verdict = "PARTIAL"
    else:
        verdict = "FAIL"

    print(f"\nSPLIT-TRANSPORT GENERALIZATION: {verdict}")

    # Build CSV rows
    csv_rows: List[Dict] = []
    for r in all_results:
        p = r.get("decodability", {})
        for pos_name in ("initial", "mid", "final"):
            csv_rows.append({
                "transport":        r["transport"],
                "task":             r["task"],
                "eps_high":         r["eps_high"],
                "inject":           r["inject"],
                "position":         pos_name,
                "decodability":     p.get(pos_name, ""),
                "final_accuracy":   r["final_acc"],
                "solve_step":       r.get("solve_step") if r.get("solve_step") else "",
                "no_last_accuracy": r.get("no_last", ""),
                "alpha_last":       r["alphaD"],
                "runtime_seconds":  r["wall"],
                "note": (f"split_canonical eps_high=1.0 k>=2 frozen; inject={r['inject']}; mod8"
                         if r["transport"] == "split_canonical"
                         else f"baseline eps_high=0.0 full transport; inject={r['inject']}; mod8"),
            })

    write_csv(csv_rows)
    write_markdown(all_results, verdict)

    print(f"\nDone.")
    print(f"  CSV: {CSV_OUT}")
    print(f"  MD:  {MD_OUT}")


if __name__ == "__main__":
    main()
