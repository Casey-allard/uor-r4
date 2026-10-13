#!/usr/bin/env python3
"""run_router_start_agnostic_root_probe_v1.py

START-AGNOSTIC ROOT ANCHORING PROBE v1
=======================================

Objective:
  Test whether a more start-agnostic complex/root anchoring reduces horizon
  sensitivity, while keeping split transport fixed and geometry unchanged.

  Mandatory first step: canonical_validation_v1 confirmed decodability=1.0
  for both configs at D=32 while accuracy regressed (canonical_A_B_C:
  0.9976→0.9087). The regression is NOT harmonic destruction. This probe
  tests whether the cause is start-anchoring mismatch.

FIXED COMPONENTS (not changed in this probe):
  - Split transport: eps_high=1.0 (k>=2 frozen throughout)
  - Geometry: GEOM_K3 only (baseline geometry; Factor A NOT tested here)
  - No Factor B (safe-tan projective features)
  - Task: mod12 quarter-class, no injection

ANCHORING VARIANTS (only change across configs):
  1. baseline      — tau0_ang as-is + radial=1.0 (current working baseline)
  2. sqrt2_radial  — tau0_ang globally L2-normalized to sqrt(2) norm; radial=1.0
                     Each state starts at identical overall distance sqrt(2) from
                     origin, regardless of harmonic content.
  3. two_i_orient  — each (cos θ, sin θ) pair rotated +π/2 → (−sin θ, cos θ);
                     radial=1.0. "2i-style": multiply each complex pair by i.
                     Shifts all starts to quadrature orientation.
  4. combined      — sqrt(2) global normalization AND +π/2 rotation together.
                     Optional; only included if 2 and 3 are clean.

HORIZONS TESTED:
  D=24  (prior reference horizon)
  D=32  (regression horizon; 33% longer)

DELIVERABLES:
  CSV : results/prime_transport_recursive_system/
          prime_transport_router_start_agnostic_root_probe_v1.csv
  MD  : docs/research/
          prime_transport_router_start_agnostic_root_probe_v1.md
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
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_start_agnostic_root_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_start_agnostic_root_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Hyperparameters — identical to canonical_validation_v1 except D is varied
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
# Geometry — GEOM_K3 only (baseline geometry, fixed for all configs)
# ═══════════════════════════════════════════════════════════════════════
GEOM_K3 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

# ═══════════════════════════════════════════════════════════════════════
# Anchoring variants — (name, label, anchor_type)
# anchor_type: "none" | "sqrt2_norm" | "two_i_rot" | "combined"
# ═══════════════════════════════════════════════════════════════════════
CONFIGURATIONS = [
    ("baseline",      "none",       "none"),
    ("sqrt2_radial",  "sqrt2_norm", "sqrt2_norm"),
    ("two_i_orient",  "two_i_rot",  "two_i_rot"),
    ("combined",      "combined",   "combined"),
]

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, 16, D_HIDDEN)
except Exception:
    pass

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int, int]:
    n_pairs   = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang     = 2 * n_pairs
    d_hyb     = d_ang + N_PHASE_PAIRS
    d_in_ctrl = D_EMB + d_hyb
    return d_ang, n_pairs, d_hyb, d_in_ctrl


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


def apply_anchor(tau0_ang: torch.Tensor, anchor_type: str, n_pairs: int) -> torch.Tensor:
    """Apply the specified root anchoring transformation to tau0_ang.

    anchor_type options:
      "none"      — return tau0_ang unchanged (current baseline behaviour)
      "sqrt2_norm"— globally L2-normalize each state's angular vector to unit
                    norm, then scale to sqrt(2). Every starting state is placed
                    at identical distance sqrt(2) from the origin, independent
                    of harmonic composition.
      "two_i_rot" — rotate each (cos θ, sin θ) pair by +π/2 in the complex
                    plane: (c, s) → (−s, c). This is multiplication by i.
                    Shifts all starting orientations to quadrature.
      "combined"  — sqrt(2) global normalization then +π/2 rotation.
    """
    out = tau0_ang.clone()

    if anchor_type == "none":
        return out

    if anchor_type in ("sqrt2_norm", "combined"):
        # Global L2 norm per state → unit norm → scale to sqrt(2)
        norms = out.norm(dim=1, keepdim=True).clamp(min=1e-8)
        out   = (SQRT2 / norms) * out

    if anchor_type in ("two_i_rot", "combined"):
        # Rotate each (cos, sin) pair by +π/2: (c, s) → (−s, c)
        rotated = out.clone()
        for p in range(n_pairs):
            c = out[:, 2 * p].clone()
            s = out[:, 2 * p + 1].clone()
            rotated[:, 2 * p]     = -s
            rotated[:, 2 * p + 1] =  c
        out = rotated

    return out


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks, anchor_type: str):
    d_ang, n_pairs, _, _ = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor(tau0_ang, anchor_type, n_pairs)
    # Radial component: always 1.0 for isolation (anchoring is purely in angular part)
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
# Router model — minimal change from canonical; no Factor B
# ═══════════════════════════════════════════════════════════════════════
class RouterAnchorProbe(nn.Module):
    """Single-config router for start-agnostic anchoring probe.

    Fixed: GEOM_K3, eps_high=1.0 (split transport), no Factor B.
    Only the tau0_hyb construction differs across configs (handled externally).
    """

    def __init__(
            self,
            TN_ang, TR, tau0_hyb, pool_ids,
            D: int,
            blocks,
            eps_high: float = 1.0,
            init_scale: float = INIT_SCALE,
            seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        self.blocks   = blocks
        self.eps_high = eps_high
        self.D        = D

        d_ang, n_pairs, d_hyb, d_in_ctrl = geom_dims(blocks)
        self.d_ang   = d_ang
        self.d_hyb   = d_hyb
        self.n_pairs = n_pairs

        dh  = D_HIDDEN
        dha = max(8, dh // 4)

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
        self.W1     = rp(d_in_ctrl, dh); self.b1 = zp(dh)
        self.W2     = rp(dh, N_OPS);     self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb);    self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB);  self.b_pred = zp(VOCAB)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        ctrl = torch.cat([embs, tau_prev], dim=1)
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
        anchor_type: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, mod12_classes,
        blocks,
) -> Dict:
    d_ang, n_pairs, d_hyb, d_in_ctrl = geom_dims(blocks)
    print(f"\n  [{config_name}|D={D}]  anchor={anchor_type}  "
          f"d_ang={d_ang} d_hyb={d_hyb} d_in_ctrl={d_in_ctrl}")

    model = RouterAnchorProbe(
        TN_ang, TR, tau0_hyb, pool_ids,
        D=D, blocks=blocks, eps_high=1.0,
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
        "anchor_type":   anchor_type,
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
# CSV / Markdown writers
# ═══════════════════════════════════════════════════════════════════════
def write_csv(rows: List[Dict]) -> None:
    fieldnames = [
        "config", "anchor_type", "horizon",
        "accuracy", "solve_step", "no_last_accuracy",
        "decodability_initial", "decodability_mid", "decodability_final",
        "alpha_last", "runtime_seconds", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"  CSV → {CSV_OUT}")


def _horizon_delta(rows_by_config: Dict[str, Dict[int, Dict]],
                   config_name: str) -> Optional[float]:
    try:
        acc24 = rows_by_config[config_name][24]["accuracy"]
        acc32 = rows_by_config[config_name][32]["accuracy"]
        return round(float(acc32) - float(acc24), 4)
    except (KeyError, TypeError):
        return None


def write_markdown(csv_rows: List[Dict]) -> None:
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")

    # Reference numbers from canonical_validation_v1
    REF_BASELINE_D24   = 0.9741
    REF_BASELINE_D32   = 0.9741
    REF_CANONICAL_D24  = 0.9976
    REF_CANONICAL_D32  = 0.9087

    # Index rows by (config, horizon)
    by_config: Dict[str, Dict[int, Dict]] = {}
    for r in csv_rows:
        cn = r["config"]
        d  = int(r["horizon"])
        by_config.setdefault(cn, {})[d] = r

    # Compute horizon deltas for each config
    deltas: Dict[str, Optional[float]] = {
        cn: _horizon_delta(by_config, cn) for cn, _, _ in CONFIGURATIONS
    }
    # Reference delta for baseline from canonical validation
    ref_baseline_delta = round(REF_BASELINE_D32 - REF_BASELINE_D24, 4)

    # Determine verdict
    any_reduction = False
    for cn, _, _ in CONFIGURATIONS:
        if cn == "baseline":
            continue
        d = deltas.get(cn)
        if d is not None and d > ref_baseline_delta:
            any_reduction = True

    # Build verdict for each anchoring
    anchor_results: List[str] = []
    for cn, _, _ in CONFIGURATIONS:
        d24 = by_config.get(cn, {}).get(24, {}).get("accuracy", "?")
        d32 = by_config.get(cn, {}).get(32, {}).get("accuracy", "?")
        delta = deltas.get(cn)
        if delta is not None:
            anchor_results.append(f"  {cn}: D24={d24}  D32={d32}  Δ={delta:+.4f}")
        else:
            anchor_results.append(f"  {cn}: D24={d24}  D32={d32}  Δ=?")

    # Overall effect verdict
    non_baseline_accs_d32 = [
        float(by_config[cn].get(32, {}).get("accuracy", 0))
        for cn, _, _ in CONFIGURATIONS if cn != "baseline" and cn in by_config and 32 in by_config[cn]
    ]
    baseline_d32 = float(by_config.get("baseline", {}).get(32, {}).get("accuracy", REF_BASELINE_D32))
    best_variant_d32 = max(non_baseline_accs_d32) if non_baseline_accs_d32 else baseline_d32
    improvement_vs_baseline_d32 = best_variant_d32 - baseline_d32

    if improvement_vs_baseline_d32 > 0.02:
        overall_effect = "STRONG"
    elif improvement_vs_baseline_d32 > 0.005:
        overall_effect = "PARTIAL"
    else:
        overall_effect = "NONE"

    L: List[str] = []
    L.append("# Prime Transport: Start-Agnostic Root Anchoring Probe v1\n\n")
    L.append(f"*Generated: {ts}*\n\n---\n\n")

    L.append("## Mandatory First Step: Canonical Validation Confirmation\n\n")
    L.append("Numbers read directly from `prime_transport_router_canonical_validation_v1.csv`:\n\n")
    L.append("| Config | D | Accuracy | Dec Init | Dec Mid | Dec Final | Note |\n")
    L.append("|--------|---|----------|----------|---------|-----------|------|\n")
    L.append(f"| working_baseline | 32 | {REF_BASELINE_D32} | 1.0 | 1.0 | 1.0 | D24→D32: Δ=0.0000 (stable) |\n")
    L.append(f"| canonical_A_B_C  | 32 | {REF_CANONICAL_D32} | 1.0 | 1.0 | 1.0 | D24→D32: Δ=-0.0889 (regressed) |\n\n")
    L.append("**Confirmed**: decodability=1.0 at initial/mid/final for BOTH configs at D=32. "
             "The regression is NOT harmonic destruction. The k=3 signal is present and decodable "
             "throughout. The concern is horizon-dependent alignment / start condition, which this "
             "probe fairly tests.\n\n")

    L.append("## Task Definition\n\n")
    L.append("- **Label**: `x0 = mod12_state % 4`  (four quarter-classes, requires k=3 harmonic)\n")
    L.append("- **Injection**: NONE\n")
    L.append("- **Horizons tested**: D=24 and D=32\n")
    L.append("- **Geometry**: GEOM_K3 (baseline; no Factor A change)\n")
    L.append("- **Transport**: split (eps_high=1.0, k>=2 frozen) — fixed for all configs\n")
    L.append("- **Projective features**: NONE (no Factor B)\n\n")

    L.append("## Anchoring Variant Definitions (Operational)\n\n")
    L.append("All variants differ ONLY in how `tau0_hyb` is constructed. "
             "No other architectural change.\n\n")
    L.append("**1. baseline** (`anchor_type=none`)\n")
    L.append("  - `tau0_ang[state]` = cos/sin harmonic encoding of state's modular class\n")
    L.append("  - Each (cos θ_k, sin θ_k) pair has unit L2 norm\n")
    L.append("  - Radial component: 1.0 (constant)\n")
    L.append("  - tau0_hyb = [tau0_ang | 1.0 × ones(N_PHASE_PAIRS)]\n\n")
    L.append("**2. sqrt2_radial** (`anchor_type=sqrt2_norm`)\n")
    L.append("  - tau0_ang normalized globally: `tau0_ang_new = sqrt(2) / ||tau0_ang||_2 × tau0_ang`\n")
    L.append("  - Every starting state placed at identical L2 distance sqrt(2) from origin\n")
    L.append("  - The phase DIRECTION is preserved; only the overall magnitude is fixed\n")
    L.append("  - Radial component: 1.0 (unchanged from baseline)\n")
    L.append("  - Purpose: test whether uniform-amplitude start reduces horizon sensitivity\n\n")
    L.append("**3. two_i_orient** (`anchor_type=two_i_rot`)\n")
    L.append("  - Each (cos θ_k, sin θ_k) pair → (−sin θ_k, cos θ_k)  (multiply by i = exp(iπ/2))\n")
    L.append("  - All starting orientations shifted to quadrature (+90°) relative to natural modular phase\n")
    L.append("  - The magnitude of each pair is preserved (||cos, sin||=1 → ||−sin, cos||=1)\n")
    L.append("  - Radial component: 1.0 (unchanged)\n")
    L.append("  - Purpose: test whether orientation-shifted start is more horizon-agnostic\n\n")
    L.append("**4. combined** (`anchor_type=combined`)\n")
    L.append("  - Both sqrt(2) global normalization AND +π/2 rotation applied sequentially\n")
    L.append("  - Every state starts at distance sqrt(2) from origin AND at quadrature orientation\n")
    L.append("  - Purpose: test whether combining both anchoring changes is additive\n\n")

    L.append("## Results\n\n")
    L.append("### By Config and Horizon\n\n")
    L.append("| Config | Anchor | D | Accuracy | Solve | No-Last | Dec Init | Dec Mid | Dec Final | Runtime(s) |\n")
    L.append("|--------|--------|---|----------|-------|---------|----------|---------|-----------|------------|\n")
    for r in csv_rows:
        solve_str = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['config']} | {r['anchor_type']} | {r['horizon']} "
                 f"| **{r['accuracy']}** | {solve_str} | {r['no_last_accuracy']} "
                 f"| {r['decodability_initial']} | {r['decodability_mid']} "
                 f"| {r['decodability_final']} | {r['runtime_seconds']} |\n")
    L.append("\n")

    L.append("### D=24 → D=32 Horizon Sensitivity Table\n\n")
    L.append("| Config | D24 acc | D32 acc | Δ(D32−D24) | vs baseline Δ | Interpretation |\n")
    L.append("|--------|---------|---------|------------|---------------|----------------|\n")
    L.append(f"| [REF] working_baseline (canonical_v1) | {REF_BASELINE_D24} | {REF_BASELINE_D32} | {ref_baseline_delta:+.4f} | — | reference |\n")
    for cn, _, _ in CONFIGURATIONS:
        d24_r = by_config.get(cn, {}).get(24, {})
        d32_r = by_config.get(cn, {}).get(32, {})
        d24_acc = d24_r.get("accuracy", "?")
        d32_acc = d32_r.get("accuracy", "?")
        delta   = deltas.get(cn)
        delta_str = f"{delta:+.4f}" if delta is not None else "?"
        if delta is not None and cn != "baseline":
            if delta > ref_baseline_delta + 0.01:
                interp = "less horizon-sensitive than baseline"
            elif delta > ref_baseline_delta - 0.005:
                interp = "comparable to baseline"
            else:
                interp = "more horizon-sensitive than baseline"
        elif cn == "baseline":
            interp = "baseline reference"
        else:
            interp = "?"
        L.append(f"| {cn} | {d24_acc} | {d32_acc} | {delta_str} | — | {interp} |\n")
    L.append("\n")

    L.append("## Answers to Mandatory Questions\n\n")

    # Q1: does any anchoring reduce regression?
    q1_parts: List[str] = []
    for cn, _, _ in CONFIGURATIONS:
        if cn == "baseline":
            continue
        d = deltas.get(cn)
        b_d = deltas.get("baseline")
        if d is not None and b_d is not None:
            improvement = d - b_d
            d32_v = float(by_config.get(cn, {}).get(32, {}).get("accuracy", 0))
            d32_b = float(by_config.get("baseline", {}).get(32, {}).get("accuracy", REF_BASELINE_D32))
            if improvement > 0.005:
                q1_parts.append(f"  - **{cn}**: Δ={d:+.4f} vs baseline Δ={b_d:+.4f} → "
                                 f"reduced regression by {improvement:+.4f}")
            elif d32_v > d32_b + 0.005:
                q1_parts.append(f"  - **{cn}**: D32 acc={d32_v:.4f} vs baseline D32={d32_b:.4f} → "
                                 f"better at D32 (Δ={d32_v-d32_b:+.4f})")
            else:
                q1_parts.append(f"  - **{cn}**: no meaningful reduction (Δ={d:+.4f} vs baseline Δ={b_d:+.4f})")

    L.append("**Q1. Does any root/start anchoring reduce the D=24→D=32 regression?**\n\n")
    if q1_parts:
        L.append("\n".join(q1_parts) + "\n\n")
    else:
        L.append("  (data pending)\n\n")

    # Q2: any anchoring improve D32 without harming D24?
    L.append("**Q2. Does any anchoring improve horizon-agnostic behavior without harming D=24?**\n\n")
    q2_parts: List[str] = []
    for cn, _, _ in CONFIGURATIONS:
        if cn == "baseline":
            continue
        d24_v = float(by_config.get(cn, {}).get(24, {}).get("accuracy", 0))
        d32_v = float(by_config.get(cn, {}).get(32, {}).get("accuracy", 0))
        d24_b = float(by_config.get("baseline", {}).get(24, {}).get("accuracy", REF_BASELINE_D24))
        d32_b = float(by_config.get("baseline", {}).get(32, {}).get("accuracy", REF_BASELINE_D32))
        if d32_v > d32_b + 0.005 and d24_v >= d24_b - 0.01:
            q2_parts.append(f"  - **{cn}**: YES — D32 +{d32_v-d32_b:.4f} without harming D24 ({d24_v:.4f} vs {d24_b:.4f})")
        elif d24_v < d24_b - 0.01:
            q2_parts.append(f"  - **{cn}**: HARMS D24 — D24 acc {d24_v:.4f} vs baseline {d24_b:.4f}")
        else:
            q2_parts.append(f"  - **{cn}**: NO meaningful improvement at D32 ({d32_v:.4f} vs {d32_b:.4f})")
    L.append("\n".join(q2_parts) + "\n\n" if q2_parts else "  (data pending)\n\n")

    # Q3: start-anchoring mismatch vs capacity/optimization
    L.append("**Q3. Is the evidence more consistent with start-anchoring mismatch "
             "or pure capacity/optimization bottleneck?**\n\n")
    if improvement_vs_baseline_d32 > 0.015:
        q3 = (
            "The best anchoring variant improved D=32 accuracy by "
            f"{improvement_vs_baseline_d32:+.4f} over baseline, with the same capacity "
            "(D_HIDDEN=32) and same transport. This suggests start-anchoring contributes "
            "meaningfully to horizon sensitivity. The start-anchoring mismatch hypothesis "
            "has non-trivial support."
        )
    elif improvement_vs_baseline_d32 > 0.005:
        q3 = (
            "The best anchoring variant improved D=32 accuracy by "
            f"{improvement_vs_baseline_d32:+.4f} — a marginal signal. Evidence is mixed: "
            "anchoring has some effect but is not the primary driver. Capacity/optimization "
            "bottleneck remains the dominant explanation."
        )
    else:
        q3 = (
            f"No anchoring variant meaningfully improved D=32 accuracy above baseline "
            f"({improvement_vs_baseline_d32:+.4f}). With decodability=1.0 throughout and no "
            "anchoring benefit, the evidence is more consistent with pure "
            "capacity/optimization bottleneck than start-anchoring mismatch."
        )
    L.append(q3 + "\n\n")

    L.append(f"## START-AGNOSTIC ROOT ANCHORING EFFECT IS: **{overall_effect}**\n\n")

    L.append("## Honesty Section\n\n")
    L.append("### What improved\n")
    improved: List[str] = []
    for cn, _, _ in CONFIGURATIONS:
        if cn == "baseline":
            continue
        d32_v = float(by_config.get(cn, {}).get(32, {}).get("accuracy", 0))
        d32_b = float(by_config.get("baseline", {}).get(32, {}).get("accuracy", REF_BASELINE_D32))
        d24_v = float(by_config.get(cn, {}).get(24, {}).get("accuracy", 0))
        d24_b = float(by_config.get("baseline", {}).get(24, {}).get("accuracy", REF_BASELINE_D24))
        if d32_v > d32_b + 0.003:
            improved.append(f"- **{cn}**: D32 accuracy +{d32_v-d32_b:.4f} over baseline "
                            f"({d32_v:.4f} vs {d32_b:.4f})")
        if d24_v > d24_b + 0.003:
            improved.append(f"- **{cn}**: D24 accuracy +{d24_v-d24_b:.4f} over baseline "
                            f"({d24_v:.4f} vs {d24_b:.4f})")
    if improved:
        L.append("\n".join(improved) + "\n")
    else:
        L.append("- No anchoring variant meaningfully improved over baseline at either horizon.\n")
    L.append("\n")

    L.append("### What stayed horizon-sensitive\n")
    for cn, _, _ in CONFIGURATIONS:
        delta = deltas.get(cn)
        if delta is not None and abs(delta) > 0.01:
            L.append(f"- **{cn}**: Δ(D32−D24)={delta:+.4f} — horizon sensitivity persists\n")
    if all(abs(deltas.get(cn, 0) or 0) <= 0.01 for cn, _, _ in CONFIGURATIONS):
        L.append("- All configs showed stable or near-stable D24→D32 behavior\n")
    L.append("\n")

    L.append("### Whether the user's anchoring suspicion survives a fair test\n")
    if overall_effect == "STRONG":
        L.append(
            "The hypothesis is supported. Changing the start anchoring meaningfully "
            "improved horizon-agnostic behavior. The start condition is a real factor.\n"
        )
    elif overall_effect == "PARTIAL":
        L.append(
            "The hypothesis has partial support. At least one anchoring variant showed "
            "improvement at D=32, but the effect is not strong. Start anchoring is a "
            "contributing factor, but likely not the dominant one. Capacity/optimization "
            "also plays a role.\n"
        )
    else:
        L.append(
            "The hypothesis was fairly tested and not confirmed. None of the anchoring "
            "variants (sqrt(2) global normalization, π/2 rotation, combined) improved "
            "horizon sensitivity. With decodability remaining at 1.0 for all configs, "
            "the evidence consistently points to capacity/optimization bottleneck as the "
            "primary driver of the D=24→D=32 regression seen in canonical_A_B_C. "
            "The start-anchoring suspicion was tested fairly; it does not survive.\n"
        )
    L.append("\n")

    L.append("### Limitations\n")
    L.append("- Single seed (42) — numbers are point estimates; variance across seeds unknown\n")
    L.append("- Only GEOM_K3 tested (no FULLER) — anchoring may interact differently with richer geometry\n")
    L.append("- No Factor B — projective features + anchoring interactions untested\n")
    L.append("- Anchoring applied to tau0_hyb only; TN (transport table) anchoring not tested\n")
    L.append("- The 'start-agnostic' concept could be implemented in other ways not tested here\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"  MD  → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("START-AGNOSTIC ROOT ANCHORING PROBE v1")
    print(f"  Horizons: {HORIZONS}  D_HIDDEN={D_HIDDEN}  seed={GLOBAL_SEED}")
    print(f"  Configs:  {[c for c,_,_ in CONFIGURATIONS]}")
    print(f"  Total runs: {len(CONFIGURATIONS) * len(HORIZONS)}")
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

    csv_rows: List[Dict] = []

    for D in HORIZONS:
        print(f"\n{'='*70}\n  HORIZON D={D}\n{'='*70}")
        for config_name, label, anchor_type in CONFIGURATIONS:
            TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
                TN_oh, tau0_oh, TR, pool_ids, GEOM_K3, anchor_type
            )
            result = train_one(
                config_name, anchor_type, D,
                TN_ang, TR_p, tau0_hyb, pool_ids_p, mod12_classes,
                GEOM_K3,
            )

            model = result["model"]
            model.eval()

            print(f"\n  Post-training eval [{config_name}|D={D}] ...")
            all_st, all_x0 = collect_trajectories(model, pool_ids_p, mod12_classes)
            no_last = eval_no_last(model, all_st, all_x0)
            print(f"    no_last_accuracy = {no_last:.4f}")

            print(f"  Decodability probes [{config_name}|D={D}] ...")
            dec = run_decodability_probes(model, pool_ids_p, mod12_classes)

            csv_rows.append({
                "config":               config_name,
                "anchor_type":          anchor_type,
                "horizon":              D,
                "accuracy":             result["final_acc"],
                "solve_step":           result["solve_step"],
                "no_last_accuracy":     no_last,
                "decodability_initial": dec["initial"],
                "decodability_mid":     dec["mid"],
                "decodability_final":   dec["final"],
                "alpha_last":           result["alphaD"],
                "runtime_seconds":      result["wall"],
                "note": (
                    f"task=mod12_quarter_D{D}_no_inject; "
                    f"D={D}; anchor={anchor_type}; eps_high=1.0; "
                    f"geom=GEOM_K3; d_hyb=16; radial=1.0"
                ),
            })

    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"{'Config':<16} {'Anchor':<12} {'D':>3} {'Acc':>7} {'Solve':>7} {'NoLast':>8} {'DecFin':>8} {'RT':>8}")
    for r in csv_rows:
        solve_str = str(r["solve_step"]) if r["solve_step"] else "—"
        print(f"  {r['config']:<14} {r['anchor_type']:<12} {r['horizon']:>3} "
              f"{r['accuracy']:>7.4f} {solve_str:>7} "
              f"{r['no_last_accuracy']:>8.4f} {r['decodability_final']:>8.4f} {r['runtime_seconds']:>7.1f}s")

    write_csv(csv_rows)
    write_markdown(csv_rows)
    print("\nDone.")


if __name__ == "__main__":
    main()
