#!/usr/bin/env python3
"""run_router_coupled_restoration_interaction_v1.py

COUPLED RESTORATION INTERACTION SCREEN v1
==========================================

Branch: coupled restoration interaction screen

LOCKED FINDINGS (do not revisit):
  - k=3 harmonic is necessary for mod-12 quarter-class representation
  - Baseline transport destroys higher harmonics
  - Split transport (eps_high=1.0, k>=2 frozen) solves both mod-12/k=3 and mod-8/k=2 no-inject tasks
  - Baseline transport fails on both harder no-inject tasks
  - Some remaining factors may only be useful in combination

OBJECTIVE:
  Test whether remaining candidate factors A/B/C/D show useful interaction
  effects when combined, rather than being incorrectly dismissed individually.

FACTORS (exact operational definitions):
  A — Fuller 4-factor geometry:
        Switch from GEOM_K3 [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]
        to GEOM_FULLER  [(0,2,2,1),(2,7,5,2),(7,9,2,1),(9,21,12,3)].
        Adds k=2 harmonic for the mod=5 block.
        d_ang: 12 → 14; d_hyb: 16 → 18; d_in: 20 → 22.

  B — Tangent/projective feature layer (controller input only):
        For each harmonic pair (cos θ, sin θ) in tau_ang, compute
          t_k = sin θ_k / (|cos θ_k| + 0.1)   [safe tangent / projective coord]
        and append to the controller input (W1 only).
        Does NOT change transport, tau trajectory, or readout dimensions.
        Adds n_ang_pairs extra scalar features (6 for K3 geom, 7 for FULLER).

  C — Root-style initialization law:
        Initialize tau0_hyb radial magnitudes as sqrt(2) instead of 1.0.
        Radial magnitudes are the N_PHASE_PAIRS values appended to tau0_ang.
        No change to transport formula or weight init scale.

  D — Full transport (contested alternative):
        eps_high = 0.0 → all harmonics transported at every step (baseline rule).
        Replace split transport (k>=2 frozen, eps_high=1.0) with full transport.

CONFIGURATIONS (6 total):
  1. working_baseline  → none of A/B/C/D
  2. A_only            → A
  3. B_only            → B
  4. A_B               → A + B
  5. A_B_C             → A + B + C
  6. A_B_C_D           → A + B + C + D

TASK (single, best for trajectory dependence):
  task_B_no_inject: x0 = mod12_initial_state % 4, NO injection.
  - Forces all signal to flow through the tau trajectory (no shortcut)
  - This task distinguishes split vs baseline transport most sharply
  - D=24 transport steps

METRICS PER CONFIG:
  final_accuracy, solve_step, no_last_accuracy,
  decodability (initial/mid/final), runtime_seconds
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_coupled_restoration_interaction_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_coupled_restoration_interaction_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config — identical to split_transport_forward_use_v1 baseline
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED    = 42
D              = 24
D_HIDDEN       = 32
BATCH_SIZE     = 512
VOCAB          = 4
D_EMB          = 4
N_PHASE_PAIRS  = 4       # one per geometry block
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

# Task: no injection, b_posLast=0 (forces all signal through tau trajectory)
TASK_INJECT   = False
B_POS0_INIT   = 0.0
B_POSLAST_INIT = 0.0

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, 16, D_HIDDEN)
except Exception:
    pass

# ═══════════════════════════════════════════════════════════════════════
# Geometry definitions — Factor A controls which is active
# ═══════════════════════════════════════════════════════════════════════
# Each tuple: (start_in_onehot, end_in_onehot, modulus, n_harmonics)
GEOM_K3     = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]  # baseline
GEOM_FULLER = [(0, 2, 2, 1), (2, 7, 5, 2), (7, 9, 2, 1), (9, 21, 12, 3)]  # Factor A

# ═══════════════════════════════════════════════════════════════════════
# 6 configurations: (config_name, factors_enabled, use_A, use_B, use_C, use_D)
# ═══════════════════════════════════════════════════════════════════════
CONFIGURATIONS = [
    ("working_baseline", "none",    False, False, False, False),
    ("A_only",           "A",       True,  False, False, False),
    ("B_only",           "B",       False, True,  False, False),
    ("A_B",              "A+B",     True,  True,  False, False),
    ("A_B_C",            "A+B+C",   True,  True,  True,  False),
    ("A_B_C_D",          "A+B+C+D", True,  True,  True,  True),
]

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks, use_B: bool) -> Tuple[int, int, int, int]:
    """Return (d_ang, n_pairs, d_hyb, d_in_ctrl).
    d_in_ctrl includes projective features if use_B=True.
    """
    n_pairs = sum(n_h for (_, _, _, n_h) in blocks)  # total harmonic pairs
    d_ang   = 2 * n_pairs
    d_hyb   = d_ang + N_PHASE_PAIRS
    d_in_ctrl = D_EMB + d_hyb + (n_pairs if use_B else 0)
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


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks, radial_init: float):
    """Build multi-harmonic hybrid tables.
    radial_init: initial radial magnitude for tau0 (1.0 = standard, sqrt(2) = Factor C).
    """
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_hyb = torch.cat(
        [tau0_ang, radial_init * torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1
    )
    return TN_ang, TR, tau0_hyb, pool_ids


def build_mod12_class_table(tau0_oh: torch.Tensor) -> torch.Tensor:
    return (tau0_oh[:, MOD12_S:MOD12_E].argmax(dim=-1) % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Split-transport function (shared; eps_high=0.0 gives full transport)
# ═══════════════════════════════════════════════════════════════════════
def apply_split_transport(
        base: torch.Tensor,
        tau_prev: torch.Tensor,
        blocks,
        eps_high: float,
) -> torch.Tensor:
    """
    eps_high=1.0: split transport (k>=2 frozen) — working baseline
    eps_high=0.0: full transport (baseline rule)  — Factor D restores this
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
# Safe-tangent projective features (Factor B)
# ═══════════════════════════════════════════════════════════════════════
def compute_projective_features(tau_ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    """
    For each harmonic pair (c, s) in tau_ang compute t = s / (|c| + 0.1).
    Returns tensor of shape (..., n_pairs).
    Appended to controller input only — does not affect transport.
    """
    parts = []
    for i in range(n_pairs):
        c = tau_ang[..., 2 * i]
        s = tau_ang[..., 2 * i + 1]
        t = s / (c.abs() + 0.1)
        parts.append(t.unsqueeze(-1))
    return torch.cat(parts, dim=-1)


# ═══════════════════════════════════════════════════════════════════════
# Router model — parameterised by all factor flags
# ═══════════════════════════════════════════════════════════════════════
class RouterInteraction(nn.Module):
    """
    Router supporting all 6 configurations.
    Task: task_B_no_inject (no injection at any position).

    use_A: GEOM_FULLER instead of GEOM_K3
    use_B: safe-tan projective features appended to controller input
    use_C: tau0 radial = sqrt(2) (baked into tau0_hyb at table prep time)
    use_D: eps_high=0.0 (full transport); False → eps_high=1.0 (split canonical)
    """

    def __init__(
            self,
            TN_ang, TR, tau0_hyb, pool_ids,
            blocks,
            use_B: bool,
            eps_high: float,
            init_scale: float = INIT_SCALE,
            seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        self.blocks   = blocks
        self.eps_high = eps_high
        self.use_B    = use_B

        d_ang, n_pairs, d_hyb, d_in_ctrl = geom_dims(blocks, use_B)
        self.d_ang    = d_ang
        self.d_hyb    = d_hyb
        self.n_pairs  = n_pairs

        dh  = D_HIDDEN
        dha = max(8, dh // 4)

        self.register_buffer("TN",          TN_ang)
        self.register_buffer("TR",          TR)
        self.register_buffer("tau0_table",  tau0_hyb)
        self.register_buffer("pool_ids",    pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]     = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1] = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(float(B_POS0_INIT)))
        self.b_posLast = nn.Parameter(torch.tensor(float(B_POSLAST_INIT)))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb   = rp(VOCAB, D_EMB)
        self.W1      = rp(d_in_ctrl, dh); self.b1 = zp(dh)
        self.W2      = rp(dh, N_OPS);     self.b2 = zp(N_OPS)
        self.W_attn  = rp(dha, d_hyb);    self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred  = rp(d_hyb, VOCAB);  self.b_pred = zp(VOCAB)
        # No injection (task_B_no_inject): no W_tok_inject

    def _ctrl_input(self, embs: torch.Tensor, tau_prev: torch.Tensor) -> torch.Tensor:
        if self.use_B:
            tau_ang  = tau_prev[:, :self.d_ang]
            proj     = compute_projective_features(tau_ang, self.n_pairs)
            return torch.cat([embs, tau_prev, proj], dim=1)
        return torch.cat([embs, tau_prev], dim=1)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        ctrl = self._ctrl_input(embs, tau_prev)
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
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = hybrid  # no injection
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
def make_batch(pool_ids, mod12_classes, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = mod12_classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, mod12_classes) -> Tuple[float, float]:
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
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
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st:  List[torch.Tensor] = []
    all_x0:  List[torch.Tensor] = []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
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


def run_decodability_probes(model, pool_ids, mod12_classes, label: str) -> Dict[str, float]:
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    MID  = D // 2 - 1
    B    = BATCH_SIZE
    n_b  = (N_PROBE + B - 1) // B

    tau0_list:  List[torch.Tensor] = []
    tau_m_list: List[torch.Tensor] = []
    tau_f_list: List[torch.Tensor] = []
    lbl_list:   List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(n_b):
            sids, _, x0 = make_batch(pool_ids, mod12_classes, rng)
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

    y        = torch.cat(lbl_list,   dim=0).numpy()
    tau0_arr = torch.cat(tau0_list,  dim=0).numpy()
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
        config_name: str,
        factors_enabled: str,
        TN_ang, TR, tau0_hyb, pool_ids, mod12_classes,
        blocks,
        use_B: bool,
        eps_high: float,
) -> Dict:
    d_ang, n_pairs, d_hyb, d_in_ctrl = geom_dims(blocks, use_B)
    print(f"\n  [{config_name}]  factors={factors_enabled}  "
          f"d_ang={d_ang} d_hyb={d_hyb} d_in_ctrl={d_in_ctrl}  "
          f"eps_high={eps_high}")

    model = RouterInteraction(
        TN_ang, TR, tau0_hyb, pool_ids,
        blocks, use_B=use_B, eps_high=eps_high,
    )
    opt = torch.optim.Adam(model.parameters(), lr=LR)

    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 1)
    t0    = time.perf_counter()

    final_acc  = 0.0
    solve_step: Optional[int] = None
    alphaD     = 0.0

    for step in range(1, MAX_STEPS + 1):
        temp = TEMP_START * (TEMP_END / TEMP_START) ** (step / MAX_STEPS)
        sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng_t)
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
        "config":          config_name,
        "factors_enabled": factors_enabled,
        "blocks":          blocks,
        "use_B":           use_B,
        "eps_high":        eps_high,
        "model":           model,
        "pool_ids":        pool_ids,
        "mod12_classes":   mod12_classes,
        "final_acc":       final_acc,
        "solve_step":      solve_step,
        "alphaD":          alphaD,
        "wall":            round(wall, 1),
    }


# ═══════════════════════════════════════════════════════════════════════
# CSV output
# ═══════════════════════════════════════════════════════════════════════
def write_csv(rows: List[Dict]) -> None:
    fieldnames = [
        "config", "factors_enabled",
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
# Markdown output
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(all_results: List[Dict], csv_rows: List[Dict]) -> None:
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%d %H:%M UTC")
    L: List[str] = []
    L.append(f"# Prime Transport: Coupled Restoration Interaction Screen v1\n\n")
    L.append(f"*Generated: {ts}*\n\n")
    L.append("---\n\n")

    # Factor definitions
    L.append("## Factor Definitions\n\n")
    L.append("| Factor | Name | Operational Definition |\n")
    L.append("|--------|------|------------------------|\n")
    L.append("| A | Fuller geometry | Switch GEOM_K3→GEOM_FULLER: adds k=2 harmonic for mod=5 block. d_ang: 12→14, d_hyb: 16→18. |\n")
    L.append("| B | Tangent/projective feature | Append safe-tan features `s/(|c|+0.1)` per harmonic pair to controller input only (W1). Does not change tau trajectory or readout. |\n")
    L.append("| C | Root-style init | tau0 radial magnitudes initialized to sqrt(2)≈1.414 instead of 1.0. |\n")
    L.append("| D | Full transport | eps_high=0.0 (all harmonics transported); replaces split transport (eps_high=1.0, k>=2 frozen). |\n")
    L.append("\n")

    # Task description
    L.append("## Task\n\n")
    L.append("**task_B_no_inject**: x0 = mod12_initial_state % 4, NO token injection at any position.\n")
    L.append("Forces all signal to flow through the tau trajectory over D=24 transport steps.\n")
    L.append("This task distinguishes split vs baseline transport most sharply per locked findings.\n\n")

    # Configurations
    L.append("## Configurations\n\n")
    L.append("| # | Config | Factors |\n")
    L.append("|---|--------|---------|\n")
    for i, (name, factors, *_) in enumerate(CONFIGURATIONS, 1):
        L.append(f"| {i} | {name} | {factors} |\n")
    L.append("\n")

    # Results table
    L.append("## Results\n\n")
    L.append("| Config | Factors | Accuracy | Solve Step | No-Last Acc | Dec Init | Dec Mid | Dec Final | Runtime(s) |\n")
    L.append("|--------|---------|----------|------------|-------------|----------|---------|-----------|------------|\n")
    for r in all_results:
        p  = r.get("decodability", {})
        ss = str(r.get("solve_step", "—")) if r.get("solve_step") else "—"
        L.append(
            f"| {r['config']} | {r['factors_enabled']} "
            f"| **{r['final_acc']:.4f}** | {ss} "
            f"| {r.get('no_last', 0.0):.4f} "
            f"| {p.get('initial', 0.0):.4f} "
            f"| {p.get('mid', 0.0):.4f} "
            f"| {p.get('final', 0.0):.4f} "
            f"| {r['wall']:.1f} |\n"
        )
    L.append("\n")

    # Answer the 5 questions
    baseline = next((r for r in all_results if r["config"] == "working_baseline"), None)
    a_only   = next((r for r in all_results if r["config"] == "A_only"), None)
    b_only   = next((r for r in all_results if r["config"] == "B_only"), None)
    ab       = next((r for r in all_results if r["config"] == "A_B"), None)
    abc      = next((r for r in all_results if r["config"] == "A_B_C"), None)
    abcd     = next((r for r in all_results if r["config"] == "A_B_C_D"), None)

    def acc(r): return r["final_acc"] if r else float("nan")
    def ss(r):  return r.get("solve_step") if r else None

    L.append("## Analysis\n\n")
    L.append("### Q1: Does any single factor help on top of the working baseline?\n\n")
    a_delta = acc(a_only) - acc(baseline)
    b_delta = acc(b_only) - acc(baseline)
    L.append(f"- A alone: {acc(a_only):.4f} vs baseline {acc(baseline):.4f} (Δ={a_delta:+.4f})\n")
    L.append(f"- B alone: {acc(b_only):.4f} vs baseline {acc(baseline):.4f} (Δ={b_delta:+.4f})\n\n")

    L.append("### Q2: Does A+B show synergy beyond A-only and B-only?\n\n")
    L.append(f"- A_only: {acc(a_only):.4f},  B_only: {acc(b_only):.4f},  A+B: {acc(ab):.4f}\n")
    ab_expected = max(acc(a_only), acc(b_only))
    ab_synergy  = acc(ab) - ab_expected
    L.append(f"- A+B vs max(A,B): Δ={ab_synergy:+.4f}\n\n")

    L.append("### Q3: Does adding C materially help?\n\n")
    L.append(f"- A+B: {acc(ab):.4f},  A+B+C: {acc(abc):.4f}  (Δ={acc(abc)-acc(ab):+.4f})\n\n")

    L.append("### Q4: Does turning on D (full transport) help or hurt once A+B+C present?\n\n")
    L.append(f"- A+B+C: {acc(abc):.4f},  A+B+C+D: {acc(abcd):.4f}  (Δ={acc(abcd)-acc(abc):+.4f})\n\n")

    L.append("### Q5: Evidence that factors only work as a coupled package?\n\n")
    isolated_max = max(acc(a_only), acc(b_only))
    coupled_acc  = acc(abcd)
    coupled_gain = coupled_acc - isolated_max
    L.append(f"- max isolated: {isolated_max:.4f},  full coupled (A+B+C+D): {coupled_acc:.4f}  (Δ={coupled_gain:+.4f})\n\n")

    # Determine overall coupled restoration effect
    base_acc = acc(baseline)
    any_single_helps = (a_delta > 0.02 or b_delta > 0.02)
    ab_synergy_strong = ab_synergy > 0.02
    full_coupled_best = (acc(abcd) > base_acc + 0.02 and acc(abcd) >= max(acc(a_only), acc(b_only)) + 0.02)
    only_coupled_wins = (not any_single_helps and acc(abc) > base_acc + 0.02)

    if only_coupled_wins or full_coupled_best or ab_synergy_strong:
        effect_label = "STRONG"
    elif any_single_helps or acc(abc) > base_acc + 0.01:
        effect_label = "PARTIAL"
    else:
        effect_label = "NONE"

    L.append(f"## COUPLED RESTORATION EFFECT IS: {effect_label}\n\n")

    # Honesty section
    L.append("## Honesty Section\n\n")

    # What worked alone
    worked_alone = []
    if a_delta > 0.02:
        worked_alone.append(f"A (Δ={a_delta:+.4f})")
    if b_delta > 0.02:
        worked_alone.append(f"B (Δ={b_delta:+.4f})")
    L.append(f"**Worked alone (Δ>0.02 over baseline)**: "
             f"{', '.join(worked_alone) if worked_alone else 'none'}\n\n")

    # What only worked in combination
    only_combo = []
    if not any_single_helps and acc(ab) > base_acc + 0.02:
        only_combo.append("A+B combination (neither A nor B alone sufficient)")
    if acc(abc) > base_acc + 0.02 and acc(ab) <= base_acc + 0.02:
        only_combo.append("A+B+C (C unlocked A+B)")
    if acc(abcd) > base_acc + 0.02 and acc(abc) <= base_acc + 0.02:
        only_combo.append("A+B+C+D (D unlocked A+B+C)")
    L.append(f"**Only worked in combination**: "
             f"{', '.join(only_combo) if only_combo else 'none observed'}\n\n")

    # Ambiguous
    ambiguous = []
    if abs(acc(abc) - acc(ab)) < 0.01:
        ambiguous.append("C contribution ambiguous (< 0.01 change on A+B)")
    if abs(acc(abcd) - acc(abc)) < 0.01:
        ambiguous.append("D contribution ambiguous (< 0.01 change on A+B+C)")
    L.append(f"**Remains ambiguous**: "
             f"{'; '.join(ambiguous) if ambiguous else 'none'}\n\n")

    # One-at-a-time false rejection risk
    L.append("**Would one-at-a-time testing have falsely rejected a useful direction?**\n\n")
    if only_coupled_wins or full_coupled_best:
        L.append(
            "YES. Isolated tests of A and B would have shown no improvement over baseline "
            "individually, leading to incorrect rejection of both. The useful behaviour "
            "only emerges in combination.\n\n"
        )
    elif any_single_helps:
        L.append(
            "PARTIALLY. At least one factor showed benefit in isolation, "
            "so one-at-a-time would not have falsely rejected everything. "
            "However, some combination effects may still be underdetected.\n\n"
        )
    else:
        L.append(
            "UNCLEAR. No strong combination benefit observed in this run. "
            "One-at-a-time testing would have correctly concluded no clear gain.\n\n"
        )

    # Geometry dims reference
    L.append("## Implementation Notes\n\n")
    L.append("### Geometry dimensions by config\n\n")
    L.append("| Config | Geom | d_ang | d_hyb | d_in_ctrl |\n")
    L.append("|--------|------|-------|-------|-----------|\n")
    for name, factors, use_A, use_B, use_C, use_D in CONFIGURATIONS:
        blocks = GEOM_FULLER if use_A else GEOM_K3
        d_ang, n_pairs, d_hyb, d_in_ctrl = geom_dims(blocks, use_B)
        L.append(f"| {name} | {'FULLER' if use_A else 'K3'} | {d_ang} | {d_hyb} | {d_in_ctrl} |\n")
    L.append("\n")
    L.append("Note: d_in_ctrl = D_EMB + d_hyb + (n_pairs if use_B else 0).\n")
    L.append("Factor C is baked into tau0_hyb radial values at table-prep time (no structural change).\n")
    L.append("Factor D changes eps_high from 1.0 (split) to 0.0 (full transport).\n\n")
    L.append(f"Task: task_B_no_inject | D={D} | seed={GLOBAL_SEED} | max_steps={MAX_STEPS} | "
             f"LR={LR} | batch={BATCH_SIZE}\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"  MD  → {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 64)
    print("COUPLED RESTORATION INTERACTION SCREEN v1")
    print("=" * 64)
    print(f"Task:      task_B_no_inject (mod12 %4, no injection)")
    print(f"Steps:     {MAX_STEPS}  LR={LR}  B={BATCH_SIZE}")
    print(f"Configs:   {[c[0] for c in CONFIGURATIONS]}\n")

    print("Loading state cache ...")
    t_load = time.perf_counter()
    data   = torch.load(CACHE_PATH, weights_only=False)
    TN_oh  = data["TN_oh"]; tau0_oh = data["tau0_oh"]
    TR     = data["TR"];    pool_ids = data["pool_ids"]
    print(f"  {TN_oh.shape[0]:,} states loaded in {time.perf_counter()-t_load:.3f}s")

    mod12_classes = build_mod12_class_table(tau0_oh)

    # Pre-build both geometry tables (C factor baked into radial_init)
    tables: Dict[str, Dict] = {}
    for use_A in (False, True):
        for use_C in (False, True):
            blocks      = GEOM_FULLER if use_A else GEOM_K3
            radial_init = math.sqrt(2) if use_C else 1.0
            key         = (use_A, use_C)
            TN_ang, TR_t, tau0_hyb, pool_t = prepare_tables(
                TN_oh, tau0_oh, TR, pool_ids, blocks, radial_init
            )
            tables[key] = {
                "blocks":    blocks,
                "TN_ang":    TN_ang,
                "TR":        TR_t,
                "tau0_hyb":  tau0_hyb,
                "pool_ids":  pool_t,
            }

    # ── Train all 6 configurations ────────────────────────────────
    all_results: List[Dict] = []

    for config_name, factors, use_A, use_B, use_C, use_D in CONFIGURATIONS:
        eps_high = 0.0 if use_D else 1.0
        t        = tables[(use_A, use_C)]
        r = train_one(
            config_name     = config_name,
            factors_enabled = factors,
            TN_ang          = t["TN_ang"],
            TR              = t["TR"],
            tau0_hyb        = t["tau0_hyb"],
            pool_ids        = t["pool_ids"],
            mod12_classes   = mod12_classes,
            blocks          = t["blocks"],
            use_B           = use_B,
            eps_high        = eps_high,
        )
        all_results.append(r)

    # ── Post-training measurements ────────────────────────────────
    print(f"\n{'=' * 64}")
    print("POST-TRAINING MEASUREMENTS")
    print("=" * 64)

    for r in all_results:
        label = r["config"]
        model = r["model"]
        pool  = r["pool_ids"]
        mc    = r["mod12_classes"]

        print(f"\n  [{label}] collecting trajectories ...")
        all_st, all_x0 = collect_trajectories(model, pool, mc)
        r["no_last"]    = eval_no_last(model, all_st, all_x0)
        print(f"    no_last = {r['no_last']:.4f}")

        print(f"  [{label}] running decodability probes ...")
        r["decodability"] = run_decodability_probes(model, pool, mc, label)

    # ── Summary table ─────────────────────────────────────────────
    print(f"\n{'=' * 64}")
    print("SUMMARY")
    print("=" * 64)
    hdr = f"  {'config':<22} {'factors':<10} {'acc':>6} {'solve':>6} {'no_last':>8} {'dec_init':>9} {'dec_mid':>8} {'dec_fin':>8} {'wall_s':>7}"
    print(hdr)
    print("  " + "-" * (len(hdr) - 2))
    for r in all_results:
        p  = r.get("decodability", {})
        ss = str(r.get("solve_step")) if r.get("solve_step") else "—"
        print(f"  {r['config']:<22} {r['factors_enabled']:<10} "
              f"{r['final_acc']:>6.4f} {ss:>6} "
              f"{r.get('no_last', 0.0):>8.4f} "
              f"{p.get('initial', 0.0):>9.4f} "
              f"{p.get('mid', 0.0):>8.4f} "
              f"{p.get('final', 0.0):>8.4f} "
              f"{r['wall']:>7.1f}")

    # ── Build CSV rows ────────────────────────────────────────────
    csv_rows: List[Dict] = []
    for r in all_results:
        p    = r.get("decodability", {})
        _, _, d_hyb, _ = geom_dims(r["blocks"], r["use_B"])
        note = (
            f"factors={r['factors_enabled']}; "
            f"eps_high={r['eps_high']}; "
            f"d_hyb={d_hyb}; "
            f"inject=False (task_B_no_inject)"
        )
        csv_rows.append({
            "config":                r["config"],
            "factors_enabled":       r["factors_enabled"],
            "accuracy":              r["final_acc"],
            "solve_step":            r.get("solve_step") if r.get("solve_step") else "",
            "no_last_accuracy":      r.get("no_last", ""),
            "decodability_initial":  p.get("initial", ""),
            "decodability_mid":      p.get("mid", ""),
            "decodability_final":    p.get("final", ""),
            "alpha_last":            r["alphaD"],
            "runtime_seconds":       r["wall"],
            "note":                  note,
        })

    write_csv(csv_rows)
    write_markdown(all_results, csv_rows)

    print(f"\nDone.")
    print(f"  CSV: {CSV_OUT}")
    print(f"  MD:  {MD_OUT}")


if __name__ == "__main__":
    main()
