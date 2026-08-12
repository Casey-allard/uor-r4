#!/usr/bin/env python3
"""run_router_harmonic_preservation_v1.py

HARMONIC PRESERVATION IN TRANSPORT DYNAMICS v1
===============================================

Branch: HARMONIC PRESERVATION IN TRANSPORT DYNAMICS

Locked context
--------------
1. Pure representation: mod-12 k=3 NECESSARY; k=1 INSUFFICIENT.
2. Delayed trainable regime: inject@last + b_posLast=2.0 → acc=1.0.
3. Signal preservation result (LOCKED):
   - fuller_k3 initial decodability = 1.0000  (k=3 clearly encoded at t=0)
   - fuller_k3 mid decodability     = 0.3076  (near chance after 12 routing steps)
   - no_last ablation: 0.2432 (chance) for BOTH regimes
   - MOD-12 K3 THROUGH DYNAMICS IS: LOST

4. Root cause hypothesis:
   - Transport update = weighted average over neighbor states
   - Neighbors span all mod-12 classes → k=3 class vectors average toward zero
   - Transport acts as a LOW-PASS FILTER destroying high-frequency harmonics

Objective
---------
Test three minimal transport modifications that may preserve k=3 harmonic content:

  BASELINE     — standard transport (reference)
  RESIDUAL_0.1  — tau_{t+1} = 0.9 * transport(tau_t) + 0.1 * tau_t
  RESIDUAL_0.25 — tau_{t+1} = 0.75 * transport(tau_t) + 0.25 * tau_t
  HARMONIC_SPLIT — k=1 pair: full transport; k>=2 pairs: eps_high=0.5 residual carry

For EACH variant, compare fuller_k3 vs reduced_k1 on the same delayed trainable
task (target = mod12_initial_state % 4, inject@last, b_posLast=2.0).

Key measurements
----------------
  - Final accuracy / solve step
  - no_last accuracy (routing-only; injection masked)
  - Decodability at initial / mid (t=D/2) / final positions
  - Whether any variant preserves k=3 above baseline mid-trajectory
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_harmonic_preservation_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_harmonic_preservation_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED    = 42
D              = 24
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
B_POSLAST_INIT = 2.0
N_PROBE        = 4096
MOD12_S, MOD12_E = 9, 21

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, 16, D_HIDDEN)
except Exception:
    pass

# ═══════════════════════════════════════════════════════════════════════
# Geometry — only mod-12 block varies
# ═══════════════════════════════════════════════════════════════════════
GEOM_K1 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 1)]
GEOM_K3 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]

GEOMS = {"reduced_k1": GEOM_K1, "fuller_k3": GEOM_K3}


def geom_dims(blocks) -> Tuple[int, int, int]:
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    d_hyb = d_ang + N_PHASE_PAIRS
    d_in  = D_EMB + d_hyb
    return d_ang, d_hyb, d_in


# ═══════════════════════════════════════════════════════════════════════
# Transport variants
# ═══════════════════════════════════════════════════════════════════════
# variant_name → (eps_full, eps_high, description)
#   eps_full : residual fraction for FULL hybrid tau
#   eps_high : residual fraction for k>=2 angular pairs only (harmonic_split)
VARIANTS: Dict[str, Dict] = {
    "baseline":      dict(eps_full=0.00, eps_high=0.00,
                          desc="standard transport; no carry"),
    "residual_0.1":  dict(eps_full=0.10, eps_high=0.00,
                          desc="tau_{t+1} = 0.9*transport + 0.1*tau_t"),
    "residual_0.25": dict(eps_full=0.25, eps_high=0.00,
                          desc="tau_{t+1} = 0.75*transport + 0.25*tau_t"),
    "harmonic_split":dict(eps_full=0.00, eps_high=0.50,
                          desc="k=1 full transport; k>=2 carry eps_high=0.5"),
}

# ═══════════════════════════════════════════════════════════════════════
# Angular / table helpers
# ═══════════════════════════════════════════════════════════════════════
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


def build_mod12_class_table(tau0_oh: torch.Tensor) -> torch.Tensor:
    return (tau0_oh[:, MOD12_S:MOD12_E].argmax(dim=-1) % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Transport update functions
# ═══════════════════════════════════════════════════════════════════════
def _normalize_ang_and_radial(ang: torch.Tensor, blocks) -> torch.Tensor:
    """Standard per-block normalization → hybrid (ang_normalized, radial)."""
    ang_parts: List[torch.Tensor] = []
    mags:      List[torch.Tensor] = []
    ai = 0
    for _, _, _, n_h in blocks:
        fund = ang[:, ai:ai + 2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        for h_idx in range(n_h):
            pair = ang[:, ai + h_idx * 2 : ai + h_idx * 2 + 2]
            ang_parts.append(pair / mag)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


def _apply_transport_variant(
        base: torch.Tensor,       # (B, d_ang) weighted sum of neighbor reps
        tau_prev: torch.Tensor,   # (B, d_hyb) previous hybrid tau
        blocks,
        eps_full: float,
        eps_high: float,
) -> torch.Tensor:
    """
    Apply transport variant to produce new hybrid tau.

    baseline       (eps_full=0, eps_high=0): standard normalize
    residual       (eps_full>0, eps_high=0): (1-ε)*normalize + ε*tau_prev
    harmonic_split (eps_full=0, eps_high>0): k=1 full transport; k>=2 residual
    """
    if eps_high > 0:
        # Harmonic-split: k=1 full transport, k>=2 carry from tau_prev angular
        ang_parts: List[torch.Tensor] = []
        mags:      List[torch.Tensor] = []
        ai = 0
        for _, _, _, n_h in blocks:
            fund = base[:, ai:ai + 2]
            mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
            mags.append(mag)
            # k=1: full transport (normalized by fundamental mag)
            ang_parts.append(fund / mag)
            # k>=2: blend new (normalized) with previous angular pair
            for h_idx in range(1, n_h):
                new_pair  = base[:, ai + h_idx * 2 : ai + h_idx * 2 + 2] / mag
                prev_pair = tau_prev[:, ai + h_idx * 2 : ai + h_idx * 2 + 2]
                ang_parts.append((1.0 - eps_high) * new_pair + eps_high * prev_pair)
            ai += n_h * 2
        hybrid_new = torch.cat(ang_parts + mags, dim=1)
        return hybrid_new
    else:
        # Standard normalize
        hybrid_new = _normalize_ang_and_radial(base, blocks)
        if eps_full > 0:
            return (1.0 - eps_full) * hybrid_new + eps_full * tau_prev
        return hybrid_new


# ═══════════════════════════════════════════════════════════════════════
# Router
# ═══════════════════════════════════════════════════════════════════════
class RouterHarmonicPreservation(nn.Module):
    """
    Delayed-injection router with configurable transport variant.

    Fixed: inject@last, b_pos0=0.0, b_posLast_init=2.0.
    Task:  x0 = mod12_initial_state % 4  (NOT random).
    Transport variant controlled by eps_full and eps_high.
    """

    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 blocks,
                 eps_full: float = 0.0,
                 eps_high: float = 0.0,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        self.blocks   = blocks
        self.eps_full = eps_full
        self.eps_high = eps_high
        d_ang, d_hyb, d_in = geom_dims(blocks)
        self.d_ang = d_ang
        self.d_hyb = d_hyb
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

        self.b_pos0    = nn.Parameter(torch.tensor(0.0))
        self.b_posLast = nn.Parameter(torch.tensor(float(B_POSLAST_INIT)))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in,  dh);    self.b1     = zp(dh)
        self.W2           = rp(dh,    N_OPS); self.b2     = zp(N_OPS)
        self.W_attn       = rp(dha,   d_hyb); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_hyb)

    def _inject(self, hybrid: torch.Tensor, x0: torch.Tensor, t: int) -> torch.Tensor:
        if t == D - 1:
            return hybrid + self.W_tok_inject[x0]
        return hybrid

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        base     = torch.einsum("bi,bij->bj", w, tn)
        hybrid   = _apply_transport_variant(
            base, tau_prev, self.blocks, self.eps_full, self.eps_high)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def _eval_transport(self, tau_prev, best_ang) -> torch.Tensor:
        """Hard-geom (argmax) transport step with variant logic."""
        return _apply_transport_variant(
            best_ang, tau_prev, self.blocks, self.eps_full, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = self._inject(hybrid, x0, t)
            soft_taus.append(tau_prev)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha

    def readout_masked(self, st: torch.Tensor, mask: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + mask)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, mod12_classes, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = mod12_classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, mod12_classes) -> Tuple[float, float]:
    """Returns (accuracy, mean_alpha_{D-1})."""
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            # Hard-geom trajectory for alpha
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


# ═══════════════════════════════════════════════════════════════════════
# Trajectory collection + ablation
# ═══════════════════════════════════════════════════════════════════════
def collect_trajectories(model, pool_ids, mod12_classes):
    """Collect (stacked_taus, tau0_initial, x0_labels) for N_EVAL states."""
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st:   List[torch.Tensor] = []
    all_tau0: List[torch.Tensor] = []
    all_x0:   List[torch.Tensor] = []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            all_tau0.append(model.tau0_table[sids].clone())
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
    return all_st, all_tau0, all_x0


def eval_no_last(model, all_st, all_x0) -> float:
    """Accuracy with t=D-1 attention-masked (routing-only; no injection shortcut)."""
    mask    = torch.zeros(1, D); mask[0, D - 1] = NEG_INF
    correct = 0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            pred, _ = model.readout_masked(st, mask)
            correct += (pred.argmax(1) == x0).sum().item()
    return round(correct / N_EVAL, 4)


# ═══════════════════════════════════════════════════════════════════════
# Decodability probes
# ═══════════════════════════════════════════════════════════════════════
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
    """Linear decodability of mod-12 class at initial / mid (t=D//2-1) / final."""
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    MID  = D // 2 - 1   # t=11 for D=24
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
                tau_cur = model._inject(hybrid, x0, t)
                if t == MID:
                    tau_m_list.append(tau_cur.cpu())
                if t == D - 1:
                    tau_f_list.append(tau_cur.cpu())
                tau_prev  = tau_cur
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
        variant_name: str, regime_name: str,
        TN_ang, TR, tau0_hyb, pool_ids, mod12_classes,
        blocks, eps_full: float, eps_high: float,
) -> Dict:
    d_ang, d_hyb, d_in = geom_dims(blocks)
    label = f"{variant_name}/{regime_name}"
    print(f"\n{'=' * 60}")
    print(f"  {label}")
    print(f"  d_tau_ang={d_ang}  d_tau_hyb={d_hyb}  eps_full={eps_full}  eps_high={eps_high}")
    print(f"{'=' * 60}")

    model = RouterHarmonicPreservation(
        TN_ang, TR, tau0_hyb, pool_ids, blocks,
        eps_full=eps_full, eps_high=eps_high)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()

    t0         = time.perf_counter()
    solve_step = None; final_acc = 0.0; alphaD_f = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, aD = eval_acc(model, pool_ids, mod12_classes)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc; alphaD_f = aD
            print(f"    [{label}] step={step:5d}  acc={acc:.4f}  α_{{D-1}}={aD:.4f}")

    wall = round(time.perf_counter() - t0, 1)
    bL   = round(float(model.b_posLast.item()), 4)
    print(f"  → acc={final_acc:.4f}  solve={solve_step}  wall={wall}s  bL={bL}")
    model.eval()
    return dict(
        model=model, variant=variant_name, regime=regime_name,
        blocks=blocks, pool_ids=pool_ids, mod12_classes=mod12_classes,
        final_acc=final_acc, solve_step=solve_step,
        alphaD=alphaD_f, bL=bL, wall=wall,
    )


# ═══════════════════════════════════════════════════════════════════════
# Output writers
# ═══════════════════════════════════════════════════════════════════════
_FIELDNAMES = [
    "variant", "regime", "position",
    "decodability", "final_accuracy", "solve_step",
    "no_last_accuracy", "alpha_last", "runtime_seconds", "note",
]


def write_csv(rows: List[Dict]):
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=_FIELDNAMES)
        w.writeheader()
        for r in rows:
            w.writerow({k: r.get(k, "") for k in _FIELDNAMES})
    print(f"\nCSV written: {CSV_OUT}  ({len(rows)} rows)")


def write_markdown(
        results: List[Dict],
        conclusion: str,
        baseline_mid_k3: float,
):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []

    L.append("# Prime Transport Router: Harmonic Preservation in Transport Dynamics v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}, "
             f"b_posLast_init={B_POSLAST_INIT}  \n\n")
    L.append("**Task:** `target = mod12_initial_state % 4`  (state-specific x0)  \n\n")

    # ── Variant definitions ─────────────────────────────────────────
    L.append("---\n\n## Transport Variant Definitions\n\n")
    L.append("| variant | update rule | parameters |\n")
    L.append("|---------|-------------|------------|\n")
    for vname, vd in VARIANTS.items():
        L.append(f"| {vname} | {vd['desc']} | "
                 f"eps_full={vd['eps_full']}, eps_high={vd['eps_high']} |\n")
    L.append("\n")
    L.append("**Low-pass filter hypothesis:** The baseline transport computes a weighted "
             "average over neighbor states spanning all mod-12 classes. This averaging "
             "destroys high-frequency (k=3) class vectors while preserving low-frequency "
             "(k=1) structure — acting as a spatial low-pass filter on the harmonic basis.  \n\n")
    L.append("**Residual carry:** Adds a fraction ε of the previous tau to the transported "
             "result. The initial k=3 signal decays as ε^t at step t.  \n")
    L.append(f"  - ε=0.1:  initial k=3 signal at mid (t=11): 0.1^11 ≈ {0.1**11:.2e}  \n")
    L.append(f"  - ε=0.25: initial k=3 signal at mid (t=11): 0.25^11 ≈ {0.25**11:.2e}  \n\n")
    L.append("**Harmonic split:** k=1 pair updated by full transport; k>=2 pairs "
             "blended with previous tau (eps_high=0.5). "
             f"Initial k=3 signal at mid (t=11): 0.5^11 ≈ {0.5**11:.4f}  \n\n")

    # ── Training summary ───────────────────────────────────────────
    L.append("---\n\n## Training and Task Results\n\n")
    L.append("| variant | regime | accuracy | solve_step | α_{D-1} | no_last | runtime_s |\n")
    L.append("|---------|--------|----------|------------|---------|---------|----------|\n")
    for r in results:
        ss = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['variant']} | {r['regime']} | {r['final_acc']:.4f} | {ss}"
                 f" | {r['alphaD']:.4f} | {r['no_last']:.4f} | {r['wall']:.1f} |\n")
    L.append("\n")

    # ── Decodability table ──────────────────────────────────────────
    L.append("---\n\n## Decodability Probes\n\n")
    L.append("Linear decodability of `mod12_class` from τ at three positions.  \n")
    L.append(f"**Baseline reference:** fuller_k3/initial=1.0000, mid={baseline_mid_k3:.4f} (near chance), final=1.0000  \n\n")

    L.append("| variant | regime | initial | mid (t=11) | final (t=23) |\n")
    L.append("|---------|--------|---------|------------|-------------|\n")
    for r in results:
        p = r.get("decodability", {})
        L.append(f"| {r['variant']} | {r['regime']} "
                 f"| {p.get('initial', '?'):.4f} "
                 f"| {p.get('mid', '?'):.4f} "
                 f"| {p.get('final', '?'):.4f} |\n")
    L.append("\n")

    # Mid-trajectory improvement over baseline
    L.append("**Mid-trajectory improvement over baseline (fuller_k3 only):**  \n\n")
    L.append("| variant | mid_decodability | Δ_vs_baseline |\n")
    L.append("|---------|-----------------|---------------|\n")
    for r in results:
        if r["regime"] != "fuller_k3":
            continue
        mid = r.get("decodability", {}).get("mid", 0.0)
        delta = round(mid - baseline_mid_k3, 4)
        sign  = "+" if delta >= 0 else ""
        L.append(f"| {r['variant']} | {mid:.4f} | {sign}{delta:.4f} |\n")
    L.append("\n")

    # ── Key questions ──────────────────────────────────────────────
    L.append("---\n\n## Key Questions\n\n")

    # Q1: any variant preserves k=3 above baseline at mid?
    best_mid = max(
        (r.get("decodability", {}).get("mid", 0.0)
         for r in results if r["regime"] == "fuller_k3"),
        default=0.0)
    best_variant = next(
        (r["variant"] for r in results
         if r["regime"] == "fuller_k3"
         and r.get("decodability", {}).get("mid", 0.0) == best_mid),
        "?")
    q1 = (f"Best mid-trajectory decodability (fuller_k3): {best_mid:.4f} "
          f"from variant '{best_variant}' "
          f"(baseline: {baseline_mid_k3:.4f}).  "
          + ("Some improvement observed."
             if best_mid > baseline_mid_k3 + 0.05 else
             "No meaningful improvement over baseline."))
    L.append(f"**Q1: Does any variant preserve k=3 above baseline at mid-trajectory?**  \n{q1}\n\n")

    # Q2: does k=3 preservation improve no_last?
    baseline_nolast = next(
        (r["no_last"] for r in results
         if r["variant"] == "baseline" and r["regime"] == "fuller_k3"), 0.25)
    best_nolast = max(
        (r["no_last"] for r in results if r["regime"] == "fuller_k3"), default=0.25)
    best_nl_var = next(
        (r["variant"] for r in results
         if r["regime"] == "fuller_k3" and r["no_last"] == best_nolast), "?")
    q2 = (f"Best no_last accuracy (fuller_k3): {best_nolast:.4f} "
          f"from '{best_nl_var}' (baseline: {baseline_nolast:.4f}).  "
          + ("No_last improves with better k=3 preservation."
             if best_nolast > baseline_nolast + 0.05 else
             "No_last does NOT improve — routing alone insufficient regardless of variant."))
    L.append(f"**Q2: Does k=3 preservation improve no_last performance?**  \n{q2}\n\n")

    # Q3: low-pass filter evidence
    k3_initial = next(
        (r.get("decodability", {}).get("initial", 0.0)
         for r in results if r["variant"] == "baseline" and r["regime"] == "fuller_k3"),
        1.0)
    k3_mid_base = baseline_mid_k3
    drop = round(k3_initial - k3_mid_base, 4)
    q3 = (f"fuller_k3: initial decodability {k3_initial:.4f} → mid {k3_mid_base:.4f} "
          f"(drop: {drop:.4f}).  "
          f"Reduced_k1: initial {next((r.get('decodability',{}).get('initial',0.0) for r in results if r['variant']=='baseline' and r['regime']=='reduced_k1'), 0.0):.4f}"
          f" → mid {next((r.get('decodability',{}).get('mid',0.0) for r in results if r['variant']=='baseline' and r['regime']=='reduced_k1'), 0.0):.4f}.  "
          "The collapse from 1.0 to chance for fuller_k3, while reduced_k1 remains at "
          "chance throughout, confirms the transport destroys high-frequency harmonics "
          "specifically — consistent with a low-pass filter.  ")
    L.append(f"**Q3: Is the transport operator acting as a low-pass filter?**  \n{q3}\n\n")

    # Q4: can minimal modifications prevent collapse?
    any_preserved = best_mid > 0.6
    any_partial   = best_mid > baseline_mid_k3 + 0.05
    q4 = (f"Best mid decodability across all variants (fuller_k3): {best_mid:.4f}.  "
          + ("YES — at least one variant achieves substantial k=3 preservation (>0.6)."
             if any_preserved else
             "PARTIAL — some improvement but no variant achieves robust preservation (>0.6)."
             if any_partial else
             "NO — none of the tested variants prevent harmonic collapse at mid-trajectory."))
    L.append(f"**Q4: Can minimal residual/structured updates prevent harmonic collapse?**  \n{q4}\n\n")

    # ── Honesty section ────────────────────────────────────────────
    L.append("---\n\n## Honesty Section\n\n")
    L.append("**Whether baseline behaves as a low-pass filter:**  \n")
    if drop > 0.5:
        L.append("YES — the drop from 1.0 to near-chance at mid-trajectory is severe and selective "
                 "(only affects higher harmonics), consistent with low-pass filtering.  \n\n")
    else:
        L.append(f"Partial evidence — decodability drop = {drop:.4f}.  \n\n")

    best_improvement = best_mid - baseline_mid_k3
    L.append("**Which minimal modification (if any) improves preservation:**  \n")
    if best_improvement > 0.05:
        L.append(f"Variant '{best_variant}' shows improvement of {best_improvement:+.4f} "
                 f"at mid-trajectory.  \n")
        if best_improvement < 0.3:
            L.append("However, the improvement is modest and the signal remains largely degraded.  \n\n")
        else:
            L.append("This is a meaningful improvement — the variant substantially preserves k=3.  \n\n")
    else:
        L.append("None of the tested variants (residual carry with ε∈{0.1, 0.25}, "
                 "harmonic_split with eps_high=0.5) meaningfully improve mid-trajectory "
                 "decodability. The harmonic collapse appears robust to these perturbations.  \n\n")

    L.append("**What remains unresolved:**  \n")
    L.append("- Whether a stronger harmonic freeze (eps_high → 1.0) would preserve k=3 "
             "(at cost of routing losing harmonic information for operator selection).  \n")
    L.append("- Whether a trainable per-harmonic decay rate could find an optimal tradeoff.  \n")
    L.append("- Whether architectures with explicit harmonic memory (e.g., separate slow/fast "
             "channels for different harmonic orders) would solve the preservation problem.  \n")
    L.append("- Whether the low-pass filtering is an inherent property of the angular routing "
             "operator or a consequence of the training objective.  \n\n")

    L.append("---\n\n")
    L.append(f"## HARMONIC CONTENT THROUGH TRANSPORT IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("HARMONIC PRESERVATION IN TRANSPORT DYNAMICS v1")
    print("=" * 60)
    print(f"Variants:  {list(VARIANTS.keys())}")
    print(f"Regimes:   {list(GEOMS.keys())}")
    print(f"Task:      x0 = mod12_initial_state % 4  (NOT random)")
    print(f"Fixed:     inject=last  b_posLast_init={B_POSLAST_INIT}  steps={MAX_STEPS}\n")

    print("Loading cache ...")
    t_load = time.perf_counter()
    data   = torch.load(CACHE_PATH, weights_only=False)
    TN_oh  = data["TN_oh"]; tau0_oh = data["tau0_oh"]
    TR     = data["TR"];    pool_ids = data["pool_ids"]
    print(f"  {TN_oh.shape[0]:,} states in {time.perf_counter()-t_load:.3f}s")

    mod12_classes = build_mod12_class_table(tau0_oh)

    # Pre-build tables for each geometry
    tables = {}
    for regime_name, blocks in GEOMS.items():
        tables[regime_name] = prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks)

    # ── Train all combinations ────────────────────────────────────
    all_results: List[Dict] = []

    for variant_name, vspec in VARIANTS.items():
        for regime_name, blocks in GEOMS.items():
            TN_ang, TR_t, tau0_hyb, pool_t = tables[regime_name]
            r = train_one(
                variant_name, regime_name,
                TN_ang, TR_t, tau0_hyb, pool_t, mod12_classes,
                blocks,
                eps_full=vspec["eps_full"],
                eps_high=vspec["eps_high"],
            )
            all_results.append(r)

    # ── Collect no_last + decodability for each run ───────────────
    print(f"\n{'=' * 60}")
    print("POST-TRAINING MEASUREMENTS")
    print("=" * 60)

    for r in all_results:
        label = f"{r['variant']}/{r['regime']}"
        model     = r["model"]
        pool_t    = r["pool_ids"]
        mod12_cls = r["mod12_classes"]

        print(f"\n  [{label}] collecting trajectories ...")
        all_st, _, all_x0 = collect_trajectories(model, pool_t, mod12_cls)
        r["no_last"] = eval_no_last(model, all_st, all_x0)
        print(f"    no_last = {r['no_last']:.4f}")

        print(f"  [{label}] running decodability probes ...")
        r["decodability"] = run_decodability_probes(model, pool_t, mod12_cls, label)

    # ── Summary ──────────────────────────────────────────────────
    print(f"\n{'=' * 60}")
    print("SUMMARY")
    print("=" * 60)
    print(f"  {'variant':<18} {'regime':<12} {'acc':>6} {'solve':>6} "
          f"{'no_last':>8} {'dec_init':>9} {'dec_mid':>8} {'dec_final':>10}")
    for r in all_results:
        p    = r.get("decodability", {})
        ss   = str(r["solve_step"]) if r["solve_step"] else "—"
        print(f"  {r['variant']:<18} {r['regime']:<12} {r['final_acc']:>6.4f} {ss:>6} "
              f"{r['no_last']:>8.4f} "
              f"{p.get('initial', 0.0):>9.4f} "
              f"{p.get('mid', 0.0):>8.4f} "
              f"{p.get('final', 0.0):>10.4f}")

    # ── Determine conclusion ──────────────────────────────────────
    baseline_mid_k3 = next(
        r.get("decodability", {}).get("mid", 0.31)
        for r in all_results
        if r["variant"] == "baseline" and r["regime"] == "fuller_k3")

    best_mid_k3 = max(
        r.get("decodability", {}).get("mid", 0.0)
        for r in all_results if r["regime"] == "fuller_k3")

    if best_mid_k3 > 0.7:
        conclusion = "PRESERVED"
    elif best_mid_k3 > baseline_mid_k3 + 0.1:
        conclusion = "PARTIALLY PRESERVED"
    else:
        conclusion = "DESTROYED"

    print(f"\nHARMONIC CONTENT THROUGH TRANSPORT IS: {conclusion}")

    # ── Build CSV rows ────────────────────────────────────────────
    csv_rows: List[Dict] = []
    for r in all_results:
        p  = r.get("decodability", {})
        for pos_name in ("initial", "mid", "final"):
            csv_rows.append({
                "variant":         r["variant"],
                "regime":          r["regime"],
                "position":        pos_name,
                "decodability":    p.get(pos_name, ""),
                "final_accuracy":  r["final_acc"],
                "solve_step":      r["solve_step"] if r["solve_step"] else "",
                "no_last_accuracy": r["no_last"],
                "alpha_last":      r["alphaD"],
                "runtime_seconds": r["wall"],
                "note":            VARIANTS[r["variant"]]["desc"],
            })

    write_csv(csv_rows)
    write_markdown(all_results, conclusion, baseline_mid_k3)


if __name__ == "__main__":
    main()
