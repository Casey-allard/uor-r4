#!/usr/bin/env python3
"""run_router_mod12_signal_preservation_v1.py

MOD-12 HARMONIC SIGNAL PRESERVATION IN THE TRAINABLE DELAYED REGIME v1
=======================================================================

Branch: MOD-12 HARMONIC SIGNAL PRESERVATION IN THE TRAINABLE DELAYED REGIME

Locked context
--------------
1. Pure representation-capacity result (LOCKED):
   - reduced_k1 (mod-12 k=1 only):  linear accuracy = 0.264  (NOT separable)
   - fuller_k3  (mod-12 k=1,2,3):   linear accuracy = 1.000  (trivially separable)
   - MOD-12 HARMONIC CAPACITY IS: K1 INSUFFICIENT / K3 NECESSARY

2. Delayed-injection trainable regime (LOCKED):
   - inject_position="last", b_posLast=2.0 → acc=1.0, solve@2500
   - tau0_direct=0.259 (random x0 → NOT encoded in τ₀)
   - last_only=1.000, no_last=0.260: last position is load-bearing
   - trajectory is genuinely necessary (no_tau0=1.000)

3. Previous r4_retest result:
   - Both reduced and fuller regimes achieve ~1.0 on standard random-x0 task
   - Neither strictly better → "fuller geometry irrelevant" for random task
   - BUT: random-x0 task does NOT require mod-12 signal preservation

Objective
---------
Test whether the delayed trainable regime can PRESERVE and USE the mod-12
k=3 harmonic signal end-to-end, on a task that specifically requires it.

Task
----
  target = mod12_initial_state % 4
  x0 = tau0_oh[s, 9:21].argmax() % 4   (derived from initial state, NOT random)

  Class 0: j ∈ {0, 4,  8}    Class 1: j ∈ {1, 5,  9}
  Class 2: j ∈ {2, 6, 10}    Class 3: j ∈ {3, 7, 11}

  This is the SAME task proven necessary for k=3 at representation level.

Design
------
  Regime A — reduced_k1:
    GEOM = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,1)]   d_hyb=12  d_in=16

  Regime B — fuller_k3:
    GEOM = [(0,2,2,1),(2,7,5,1),(7,9,2,1),(9,21,12,3)]   d_hyb=16  d_in=20

  Both use: inject@last, b_posLast_init=2.0, same training budget (3500 steps).
  ONLY the mod-12 block varies; mod-5 kept at k=1 to isolate mod-12 signal.

Measurements
------------
  1. training accuracy / final accuracy
  2. solve step if achieved
  3. last_only accuracy
  4. no_last accuracy        ← KEY: routing-only signal preservation
  5. tau0_direct accuracy    ← st[:,0,:] @ W_pred (after step 0, no inj)
  6. alpha_{D-1}
  7. runtime

  Decodability probes (each regime, after training):
    Linear decodability of mod-12 class from τ at:
      a. initial representation  (τ₀ = tau0_table[sids], before any routing)
      b. middle trajectory       (τ at t=D//2-1, mid-point, no injection)
      c. final position          (τ at t=D-1, after injection)
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_mod12_signal_preservation_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_mod12_signal_preservation_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED    = 42
D              = 24
D_HIDDEN       = 32
BATCH_SIZE     = 512
VOCAB          = 4       # 4 mod-12 quarter-phase classes
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
B_POSLAST_INIT = 2.0     # confirmed rescuer from bootstrap probe
N_PROBE        = 4096    # states for decodability probes
MOD12_S        = 9       # mod-12 onehot block start index
MOD12_E        = 21      # mod-12 onehot block end index

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, 16, D_HIDDEN)   # conservative: reduced_k1 d_in
except Exception:
    pass

# ═══════════════════════════════════════════════════════════════════════
# Geometry definitions — ONLY mod-12 block varies
# ═══════════════════════════════════════════════════════════════════════
# Each entry: (start_in_onehot, end_in_onehot, modulus, n_harmonics)
GEOM_K1 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 1)]
GEOM_K3 = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]


def geom_dims(blocks) -> Tuple[int, int, int]:
    """Return (d_tau_ang, d_tau_hyb, d_in_hyb)."""
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    d_hyb = d_ang + N_PHASE_PAIRS
    d_in  = D_EMB + d_hyb
    return d_ang, d_hyb, d_in


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion (multi-harmonic, from r4_retest)
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
    """mod12_classes[i] = tau0_oh[i, 9:21].argmax() % 4  ∈ {0,1,2,3}."""
    mod12_idx = tau0_oh[:, MOD12_S:MOD12_E].argmax(dim=-1)
    return (mod12_idx % VOCAB).long()


# ═══════════════════════════════════════════════════════════════════════
# Per-block normalization helper (shared)
# ═══════════════════════════════════════════════════════════════════════
def _normalize_ang_and_radial(ang: torch.Tensor, blocks) -> torch.Tensor:
    """Normalize per-block using fundamental magnitude; compute radial."""
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
    dirn   = torch.cat(ang_parts, dim=1)
    radial = torch.cat(mags,      dim=1)
    return torch.cat([dirn, radial], dim=1)


# ═══════════════════════════════════════════════════════════════════════
# Router (geometry-agnostic, inject@last, b_posLast)
# ═══════════════════════════════════════════════════════════════════════
class RouterMod12Signal(nn.Module):
    """
    Delayed-injection router for mod-12 signal preservation.

    Fixed: inject_position='last', b_pos0=0.0, b_posLast_init=2.0.
    Geometry is parameterized by 'blocks'.
    x0 is the mod-12 quarter-phase class of the initial state (NOT random).
    """

    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 blocks,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        self.blocks = blocks
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
        """One routing step → (hybrid, new_state_ids)."""
        tn   = self.TN[state_ids]
        embs = self.W_emb[tokens_t]
        h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
        logits = h @ self.W2 + self.b2
        if self.training:
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)
        B    = state_ids.shape[0]
        base = torch.einsum("bi,bij->bj", w, tn)
        hybrid   = _normalize_ang_and_radial(base, self.blocks)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

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
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha

    def readout_masked(self, st: torch.Tensor, attn_mask: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast
                 + attn_mask)
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, mod12_classes, rng):
    """x0 = mod12_class of the sampled initial state (NOT random)."""
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = mod12_classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, mod12_classes) -> Tuple[float, float, float]:
    """Returns (accuracy, mean_alpha_0, mean_alpha_{D-1})."""
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; a0_sum = 0.0; aD_sum = 0.0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            logits = model(sids, toks, x0, 0.05)
            correct += (logits.argmax(1) == x0).sum().item()
            # Hard-geom alpha tracking
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            B = BATCH_SIZE
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = _normalize_ang_and_radial(best_ang, model.blocks)
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur)
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            _, alpha = model.readout(st)
            a0_sum += alpha[:, 0].mean().item()
            aD_sum += alpha[:, D - 1].mean().item()
    model.train()
    return (round(correct / N_EVAL, 4),
            round(a0_sum / N_BATCHES, 4),
            round(aD_sum / N_BATCHES, 4))


def collect_hard_geom_trajectories(
        model, pool_ids, mod12_classes
) -> Tuple[List[torch.Tensor], List[torch.Tensor], List[torch.Tensor]]:
    """Collect (stacked_taus, tau0_initial, x0_labels) for N_EVAL states.

    Returns:
        all_st     — list of (B, D, d_hyb) trajectory tensors
        all_tau0   — list of (B, d_hyb) initial tau tensors (pre-routing)
        all_x0     — list of (B,) mod-12 class label tensors
    """
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st: List[torch.Tensor]   = []
    all_tau0: List[torch.Tensor] = []
    all_x0: List[torch.Tensor]   = []
    B = BATCH_SIZE
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            # Store initial tau0 (before any routing)
            tau0_init = model.tau0_table[sids].clone()
            all_tau0.append(tau0_init)
            all_x0.append(x0)

            # Hard-geom trajectory
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
                hybrid   = _normalize_ang_and_radial(best_ang, model.blocks)
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur.clone())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
    return all_st, all_tau0, all_x0


# ═══════════════════════════════════════════════════════════════════════
# Ablations
# ═══════════════════════════════════════════════════════════════════════
ABLATION_DEFS = [
    ("full",       "full trajectory"),
    ("tau0_direct", "τ after step-0 → W_pred (no injection; like r4_retest def)"),
    ("no_tau0",    "position 0 masked"),
    ("last_only",  f"only t={D - 1}"),
    ("no_last",    f"t={D - 1} masked  ← KEY: routing-only signal"),
]


def apply_ablation(
        model,
        all_st:   List[torch.Tensor],
        all_x0:   List[torch.Tensor],
        ablation: str,
) -> Tuple[float, float, float, float]:
    """Returns (accuracy, mean_alpha_0, mean_alpha_{D-1}, wall_s)."""
    t_start = time.perf_counter()
    correct = 0; a0_sum = 0.0; aD_sum = 0.0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            B = st.shape[0]
            if ablation == "full":
                pred, alpha = model.readout(st)
            elif ablation == "tau0_direct":
                # Predict from tau at position 0 (after step 0, consistent with r4_retest)
                pred  = st[:, 0, :] @ model.W_pred + model.b_pred
                alpha = torch.zeros(B, D); alpha[:, 0] = 1.0
            elif ablation == "no_tau0":
                mask = torch.zeros(1, D); mask[0, 0] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif ablation == "last_only":
                mask = torch.full((1, D), NEG_INF); mask[0, D - 1] = 0.0
                pred, alpha = model.readout_masked(st, mask)
            elif ablation == "no_last":
                mask = torch.zeros(1, D); mask[0, D - 1] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            else:
                raise ValueError(ablation)
            correct += (pred.argmax(1) == x0).sum().item()
            a0_sum  += alpha[:, 0].mean().item()
            aD_sum  += alpha[:, D - 1].mean().item()
    wall = round(time.perf_counter() - t_start, 4)
    return (round(correct / N_EVAL, 4),
            round(a0_sum / N_BATCHES, 4),
            round(aD_sum / N_BATCHES, 4),
            wall)


def _interpret_ablation(ablation: str, acc: float, full_acc: float) -> str:
    drop = round(acc - full_acc, 3)
    if ablation == "full":
        return "reference"
    if ablation == "tau0_direct":
        if acc >= 0.999:  return "τ₀ encodes answer linearly (k=3 capacity)"
        if acc < 0.35:    return "τ₀ does NOT encode answer (k=1 capacity) ✓"
        return f"partial: {drop:+.3f}"
    if ablation == "no_tau0":
        if acc >= 0.999:  return "trajectory sufficient without τ₀ ✓"
        if acc < 0.35:    return "τ₀ is critical (collapse)"
        return f"{drop:+.3f}"
    if ablation == "last_only":
        if acc >= 0.999:  return "final position carries full answer ✓"
        if acc > 0.5:     return f"partial signal at t=D-1: {drop:+.3f}"
        return f"{drop:+.3f}"
    if ablation == "no_last":
        if acc >= 0.999:  return "routing preserves signal end-to-end ✓"
        if acc < 0.35:    return "t=D-1 injection critical; routing insufficient ✗"
        return f"routing partial: {drop:+.3f}"
    return f"{drop:+.3f}"


# ═══════════════════════════════════════════════════════════════════════
# Decodability probes
# ═══════════════════════════════════════════════════════════════════════
def linear_probe(X: np.ndarray, y: np.ndarray, label: str) -> float:
    """Logistic regression accuracy for classifying mod-12 class from tau."""
    scaler = StandardScaler()
    Xs     = scaler.fit_transform(X)
    clf    = LogisticRegression(
        max_iter=2000, C=1.0, solver="lbfgs", multi_class="multinomial")
    clf.fit(Xs, y)
    acc = float((clf.predict(Xs) == y).mean())
    print(f"      [{label}] decodability={acc:.4f}  (n={X.shape[0]}, d={X.shape[1]})")
    return round(acc, 4)


def run_decodability_probes(
        model,
        pool_ids,
        mod12_classes,
        regime_label: str,
) -> Dict[str, float]:
    """
    Probe linear decodability of mod-12 class at:
      - initial:  tau0_table[sids]   (before any routing)
      - mid:      taus at t=D//2-1   (mid-trajectory, no injection)
      - final:    taus at t=D-1      (after routing + injection at t=D-1)

    Returns dict of {position_name: linear_accuracy}.
    """
    print(f"\n  Running decodability probes for [{regime_label}] ...")
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    N   = N_PROBE
    n_batches = (N + BATCH_SIZE - 1) // BATCH_SIZE
    B   = BATCH_SIZE
    MID = D // 2 - 1  # approx middle (index 11 for D=24)

    tau0_list:  List[torch.Tensor] = []
    tau_mid_list:   List[torch.Tensor] = []
    tau_final_list: List[torch.Tensor] = []
    labels_list:    List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(n_batches):
            sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng)
            # Initial representation (pre-routing)
            tau0_list.append(model.tau0_table[sids].cpu())
            labels_list.append(x0.cpu())

            # Hard-geom trajectory to get mid and final taus
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid  = _normalize_ang_and_radial(best_ang, model.blocks)
                tau_cur = model._inject(hybrid, x0, t)
                if t == MID:
                    tau_mid_list.append(tau_cur.cpu())
                if t == D - 1:
                    tau_final_list.append(tau_cur.cpu())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)

    tau0_arr  = torch.cat(tau0_list,      dim=0).numpy()
    tau_m_arr = torch.cat(tau_mid_list,   dim=0).numpy()
    tau_f_arr = torch.cat(tau_final_list, dim=0).numpy()
    y         = torch.cat(labels_list,    dim=0).numpy()

    acc_init  = linear_probe(tau0_arr,  y, f"{regime_label}/initial")
    acc_mid   = linear_probe(tau_m_arr, y, f"{regime_label}/mid(t={MID})")
    acc_final = linear_probe(tau_f_arr, y, f"{regime_label}/final(t={D-1})")

    return {"initial": acc_init, "mid": acc_mid, "final": acc_final}


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_regime(
        blocks, regime_label: str,
        TN_ang, TR, tau0_hyb, pool_ids,
        mod12_classes,
) -> Tuple:
    d_ang, d_hyb, d_in = geom_dims(blocks)
    print(f"\n{'=' * 60}")
    print(f"REGIME: {regime_label}")
    print(f"  geometry: d_tau_ang={d_ang}  d_tau_hyb={d_hyb}  d_in={d_in}")
    print(f"  inject=last  b_posLast_init={B_POSLAST_INIT}  b_pos0=0.0  radial=dynamic")
    print(f"  task: x0 = mod12_initial_state % 4  (NOT random)")
    print(f"{'=' * 60}")

    model = RouterMod12Signal(TN_ang, TR, tau0_hyb, pool_ids, blocks=blocks)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()

    t0         = time.perf_counter()
    solve_step = None; final_acc = 0.0; alpha0_f = 0.0; alphaD_f = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, mod12_classes, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, a0, aD = eval_acc(model, pool_ids, mod12_classes)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc; alpha0_f = a0; alphaD_f = aD
            print(f"    [{regime_label}] step={step:5d}  acc={acc:.4f}"
                  f"  α₀={a0:.4f}  α_{{D-1}}={aD:.4f}")

    wall = round(time.perf_counter() - t0, 1)
    sps  = round(MAX_STEPS / wall, 1)
    bL   = round(float(model.b_posLast.item()), 4)
    print(f"  {regime_label}: acc={final_acc:.4f}  solve={solve_step}"
          f"  sps={sps}  b_posLast={bL}"
          f"  α₀={alpha0_f:.4f}  α_{{D-1}}={alphaD_f:.4f}")
    model.eval()
    return model, final_acc, solve_step, sps, wall, alpha0_f, alphaD_f, bL


# ═══════════════════════════════════════════════════════════════════════
# Output writers
# ═══════════════════════════════════════════════════════════════════════
_FIELDNAMES = [
    "regime", "position_probe", "linear_decodability",
    "final_accuracy", "solve_step",
    "tau0_direct_accuracy", "alpha_last",
    "runtime_seconds", "note",
]


def _row(regime, ablation_name, acc, solve_step, tau0_direct, aD, wall, note,
         position_probe="", lin_dec="") -> Dict:
    return {
        "regime":               regime,
        "position_probe":       position_probe,
        "linear_decodability":  lin_dec,
        "final_accuracy":       acc,
        "solve_step":           solve_step if solve_step is not None else "",
        "tau0_direct_accuracy": tau0_direct,
        "alpha_last":           aD,
        "runtime_seconds":      wall,
        "note":                 note,
    }


def write_csv(rows: List[Dict]):
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=_FIELDNAMES)
        w.writeheader()
        for r in rows:
            w.writerow({k: r.get(k, "") for k in _FIELDNAMES})
    print(f"\nCSV written: {CSV_OUT}  ({len(rows)} rows)")


def write_markdown(
        regimes: List[Dict],
        ablation_rows: List[Dict],
        decodability: Dict[str, Dict[str, float]],
        conclusion: str,
):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []

    L.append("# Prime Transport Router: Mod-12 Harmonic Signal Preservation v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}, "
             f"b_posLast_init={B_POSLAST_INIT}  \n\n")
    L.append("**Task:** `target = mod12_initial_state % 4`  "
             "(state-specific x0; NOT random)  \n\n")

    # ── Definition table
    L.append("---\n\n## Regime Definitions\n\n")
    L.append("| component | reduced_k1 | fuller_k3 |\n")
    L.append("|-----------|------------|----------|\n")
    L.append("| mod-12 harmonics | k=1 only → 2 dims | k=1,2,3 → 6 dims |\n")
    L.append("| mod-5  harmonics | k=1 only (same) | k=1 only (same) |\n")
    L.append("| d_tau_hyb | 12 | 16 |\n")
    L.append("| d_in_hyb  | 16 | 20 |\n")
    L.append("| inject_position | last | last |\n")
    L.append("| b_posLast_init  | 2.0  | 2.0  |\n\n")
    L.append("**Only the mod-12 block varies.** "
             "Mod-5 kept at k=1 to isolate the mod-12 harmonic effect.  \n\n")
    L.append("**Known representation capacity (locked):**  \n")
    L.append("- reduced_k1: linear accuracy = 0.264 (k=1 cannot separate 4 interleaved triangles)  \n")
    L.append("- fuller_k3:  linear accuracy = 1.000 (k=3 collapses each class to orthogonal unit vector)  \n\n")

    # ── Training results
    L.append("---\n\n## Training Results\n\n")
    L.append("| regime | d_hyb | accuracy | solve_step | α_{D-1} | b_posLast | runtime_s |\n")
    L.append("|--------|-------|----------|------------|---------|-----------|----------|\n")
    for r in regimes:
        ss = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['regime']} | {r['d_hyb']} | {r['acc']:.4f} | {ss}"
                 f" | {r['aD']:.4f} | {r['bL']:.4f} | {r['wall']:.1f} |\n")
    L.append("\n")

    # ── Ablation results
    L.append("---\n\n## Ablation Results\n\n")
    L.append("| regime | ablation | accuracy | interpretation |\n")
    L.append("|--------|----------|----------|----------------|\n")
    for r in ablation_rows:
        L.append(f"| {r['regime']} | {r['ablation']} | {r['acc']:.4f} | {r['interp']} |\n")
    L.append("\n")

    # ── Decodability probes
    L.append("---\n\n## Decodability Probes\n\n")
    L.append("Linear decodability of `mod12_class` from τ at three trajectory positions "
             "(logistic regression, train-fit accuracy).  \n\n")
    L.append("| regime | position | linear_decodability | interpretation |\n")
    L.append("|--------|----------|---------------------|----------------|\n")
    for regime_label, probes in decodability.items():
        for pos_name, acc in probes.items():
            if pos_name == "initial":
                interp = "pre-routing repr capacity (locked baseline)"
            elif pos_name == "mid":
                interp = "mid-trajectory preservation (routing dynamics)"
            else:
                interp = "final position (includes injection at t=D-1)"
            L.append(f"| {regime_label} | {pos_name} | {acc:.4f} | {interp} |\n")
    L.append("\n")

    # ── Key questions
    L.append("---\n\n## Answers to Key Questions\n\n")

    # Q1: fuller_k3 outperforms?
    r_k1 = next(r for r in regimes if r["regime"] == "reduced_k1")
    r_k3 = next(r for r in regimes if r["regime"] == "fuller_k3")
    nolast_k1 = next((r for r in ablation_rows
                      if r["regime"] == "reduced_k1" and r["ablation"] == "no_last"), None)
    nolast_k3 = next((r for r in ablation_rows
                      if r["regime"] == "fuller_k3"  and r["ablation"] == "no_last"), None)

    full_comparable = abs(r_k1["acc"] - r_k3["acc"]) < 0.05
    q1 = ("Both regimes reach similar full accuracy (both solve the injection task). "
          "The key difference appears in the no_last ablation.  "
          if full_comparable else
          f"Accuracy difference: reduced_k1={r_k1['acc']:.4f}, fuller_k3={r_k3['acc']:.4f}.  ")

    L.append(f"**Q1: Does fuller_k3 outperform reduced_k1 in the full delayed trainable regime?**  \n"
             f"{q1}\n\n")

    # Q2: k3 trajectory better preserved?
    mid_k1 = decodability.get("reduced_k1", {}).get("mid", 0.0)
    mid_k3 = decodability.get("fuller_k3",  {}).get("mid", 0.0)
    q2_adv = mid_k3 - mid_k1
    q2 = (f"Mid-trajectory decodability: reduced_k1={mid_k1:.4f}, fuller_k3={mid_k3:.4f}.  "
          f"k=3 advantage at mid-trajectory: {q2_adv:+.4f}.  "
          + ("The k=3 signal IS better preserved through trajectory dynamics.  "
             if q2_adv > 0.1 else
             "The k=3 signal advantage does NOT survive the trajectory dynamics.  "))
    L.append(f"**Q2: Does the delayed trajectory preserve k=3 signal better than k=1?**  \n{q2}\n\n")

    # Q3: only representational or survives dynamics?
    nolast_k3_acc = nolast_k3["acc"] if nolast_k3 else 0.0
    q3 = (f"no_last accuracy for fuller_k3 = {nolast_k3_acc:.4f}.  "
          + ("The k=3 advantage survives THROUGH dynamics (>0.6 without injection).  "
             if nolast_k3_acc > 0.6 else
             "The k=3 advantage is ONLY representational; routing dynamics do not preserve it.  "))
    L.append(f"**Q3: Is k=3 advantage only representational, or does it survive dynamics?**  \n{q3}\n\n")

    # Q4: overturns earlier conclusion?
    nolast_k1_acc = nolast_k1["acc"] if nolast_k1 else 0.0
    if nolast_k3_acc > nolast_k1_acc + 0.1:
        q4 = ("YES — on the mod-12-specific task, fuller_k3 significantly outperforms reduced_k1 "
              "in the no_last ablation. The earlier 'fuller geometry irrelevant' conclusion "
              "was specific to the random-x0 task; it does NOT hold for mod-12 signal preservation.  ")
    elif abs(nolast_k3_acc - nolast_k1_acc) < 0.05:
        q4 = ("NO — even on the mod-12-specific task, fuller_k3 and reduced_k1 perform similarly "
              "in the no_last ablation. The earlier 'fuller geometry irrelevant' conclusion "
              "is NOT overturned.  ")
    else:
        q4 = (f"PARTIAL — fuller_k3 no_last={nolast_k3_acc:.4f} vs "
              f"reduced_k1 no_last={nolast_k1_acc:.4f}. "
              "A moderate advantage exists but is not conclusive.  ")
    L.append(f"**Q4: Does this overturn the 'fuller geometry irrelevant' conclusion?**  \n{q4}\n\n")

    # ── Honesty section
    L.append("---\n\n## Honesty Section\n\n")
    L.append("**What is now fairly tested:**  \n")
    L.append("- Both regimes trained under identical conditions (inject@last, b_posLast=2.0, "
             "3500 steps, same task, same seed).  \n")
    L.append("- The task specifically requires mod-12 quarter-phase classification "
             "(the exact task proven necessary for k=3 at representation level).  \n")
    L.append("- Decodability probes directly measure whether mod-12 class information "
             "survives routing at three trajectory depths.  \n")
    L.append("- The no_last ablation tests routing-only signal preservation without "
             "the injection shortcut.  \n\n")

    L.append("**What still remains unresolved:**  \n")
    L.append("- Whether a longer training budget (>3500 steps) would change the dynamics result.  \n")
    L.append("- Whether different routing architectures (e.g. attention-based routing rather "
             "than angular selection) would better preserve harmonic structure.  \n")
    L.append("- Whether the result generalises to other modular structure (mod-5, etc.).  \n\n")

    # 12-tick intuition
    init_k3 = decodability.get("fuller_k3", {}).get("initial", 0.0)
    mid_k3_ = decodability.get("fuller_k3", {}).get("mid",     0.0)
    L.append("**Whether the 12-tick intuition survives dynamic testing:**  \n")
    L.append(f"The 12-tick intuition holds at the REPRESENTATION level "
             f"(initial decodability: fuller_k3={init_k3:.4f} vs "
             f"reduced_k1={decodability.get('reduced_k1', {}).get('initial', 0.0):.4f}).  \n")
    L.append(f"At mid-trajectory the decodability for fuller_k3 = {mid_k3_:.4f}.  \n")
    if mid_k3_ > 0.7:
        L.append("The 12-tick (k=3) advantage PERSISTS through the dynamic routing, "
                 "suggesting the routing trajectory does not erase modular harmonic structure.  \n\n")
    elif mid_k3_ > 0.4:
        L.append("The 12-tick (k=3) advantage is PARTIALLY PRESERVED through dynamics. "
                 "Routing does not fully destroy the harmonic structure but also does not "
                 "maintain it at representation-level fidelity.  \n\n")
    else:
        L.append("The 12-tick (k=3) advantage is LOST through dynamics. "
                 "Routing erases the high-frequency harmonic information.  \n\n")

    # ── Conclusion
    L.append("---\n\n")
    L.append(f"## MOD-12 K3 THROUGH DYNAMICS IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("MOD-12 HARMONIC SIGNAL PRESERVATION v1")
    print("=" * 60)
    d_k1 = geom_dims(GEOM_K1)
    d_k3 = geom_dims(GEOM_K3)
    print(f"reduced_k1: d_tau_ang={d_k1[0]}  d_tau_hyb={d_k1[1]}  d_in={d_k1[2]}")
    print(f"fuller_k3:  d_tau_ang={d_k3[0]}  d_tau_hyb={d_k3[1]}  d_in={d_k3[2]}")
    print(f"Both: inject=last  b_posLast_init={B_POSLAST_INIT}  steps={MAX_STEPS}")
    print(f"Task: x0 = mod12_initial_state % 4  (NOT random)\n")

    print("Loading cache ...")
    t0_load = time.perf_counter()
    data    = torch.load(CACHE_PATH, weights_only=False)
    TN_oh   = data["TN_oh"];   tau0_oh = data["tau0_oh"]
    TR      = data["TR"];      pool_ids = data["pool_ids"]
    print(f"  {TN_oh.shape[0]:,} states in {time.perf_counter() - t0_load:.3f}s")

    # Build mod-12 class labels for ALL states
    mod12_classes = build_mod12_class_table(tau0_oh)
    class_counts  = [(mod12_classes[pool_ids] == c).sum().item() for c in range(VOCAB)]
    print(f"  Pool class distribution: {class_counts}  "
          f"(should be roughly balanced)\n")

    # Build tables for each geometry
    TN_k1, TR_k1, tau0_k1, pid_k1 = prepare_tables(TN_oh, tau0_oh, TR, pool_ids, GEOM_K1)
    TN_k3, TR_k3, tau0_k3, pid_k3 = prepare_tables(TN_oh, tau0_oh, TR, pool_ids, GEOM_K3)

    # ── Train both regimes ────────────────────────────────────────────
    model_k1, acc_k1, solve_k1, sps_k1, wall_k1, a0_k1, aD_k1, bL_k1 = train_regime(
        GEOM_K1, "reduced_k1", TN_k1, TR_k1, tau0_k1, pid_k1, mod12_classes)
    model_k3, acc_k3, solve_k3, sps_k3, wall_k3, a0_k3, aD_k3, bL_k3 = train_regime(
        GEOM_K3, "fuller_k3", TN_k3, TR_k3, tau0_k3, pid_k3, mod12_classes)

    regimes_summary = [
        dict(regime="reduced_k1", d_hyb=d_k1[1], acc=acc_k1, solve_step=solve_k1,
             a0=a0_k1, aD=aD_k1, bL=bL_k1, wall=wall_k1),
        dict(regime="fuller_k3",  d_hyb=d_k3[1], acc=acc_k3, solve_step=solve_k3,
             a0=a0_k3, aD=aD_k3, bL=bL_k3, wall=wall_k3),
    ]

    # ── Ablations ────────────────────────────────────────────────────
    print(f"\n{'=' * 60}")
    print("ABLATIONS")
    print("=" * 60)

    ablation_rows: List[Dict] = []
    tau0_direct_by_regime: Dict[str, float] = {}

    for regime_label, model, blocks, TN_ang, TR_t, tau0_hyb, pool_t, acc_train in [
        ("reduced_k1", model_k1, GEOM_K1, TN_k1, TR_k1, tau0_k1, pid_k1, acc_k1),
        ("fuller_k3",  model_k3, GEOM_K3, TN_k3, TR_k3, tau0_k3, pid_k3, acc_k3),
    ]:
        print(f"\n  Collecting trajectories for [{regime_label}] ...")
        all_st, all_tau0, all_x0 = collect_hard_geom_trajectories(
            model, pool_t, mod12_classes)
        full_acc_here = None
        for abl_name, abl_note in ABLATION_DEFS:
            acc_abl, a0_abl, aD_abl, w_abl = apply_ablation(
                model, all_st, all_x0, abl_name)
            if abl_name == "full":
                full_acc_here = acc_abl
            if abl_name == "tau0_direct":
                tau0_direct_by_regime[regime_label] = acc_abl
            interp = _interpret_ablation(abl_name, acc_abl, full_acc_here or acc_train)
            delta  = round(acc_abl - (full_acc_here or acc_train), 4)
            sign   = "+" if delta >= 0 else ""
            print(f"    [{abl_name:<12}] acc={acc_abl:.4f}  Δ={sign}{delta:.4f}"
                  f"  α₀={a0_abl:.4f}  α_{{D-1}}={aD_abl:.4f}  — {interp}")
            ablation_rows.append(dict(
                regime=regime_label, ablation=abl_name, acc=acc_abl,
                aD=aD_abl, interp=interp, note=abl_note, wall=w_abl,
            ))

    # ── Decodability probes ──────────────────────────────────────────
    print(f"\n{'=' * 60}")
    print("DECODABILITY PROBES")
    print("=" * 60)

    decodability: Dict[str, Dict[str, float]] = {}
    decodability["reduced_k1"] = run_decodability_probes(
        model_k1, pid_k1, mod12_classes, "reduced_k1")
    decodability["fuller_k3"]  = run_decodability_probes(
        model_k3, pid_k3, mod12_classes, "fuller_k3")

    # ── Summary ──────────────────────────────────────────────────────
    print(f"\n{'=' * 60}")
    print("SUMMARY")
    print("=" * 60)
    for r in regimes_summary:
        ss = f"  ✓ solve@{r['solve_step']}" if r["solve_step"] else ""
        print(f"  {r['regime']:<12} acc={r['acc']:.4f}  α_{{D-1}}={r['aD']:.4f}  "
              f"b_posLast={r['bL']:.3f}{ss}")

    print("\n  Decodability:")
    for regime_label, probes in decodability.items():
        for pos, acc in probes.items():
            print(f"    {regime_label}/{pos:<8} = {acc:.4f}")

    nolast_k1 = next((r["acc"] for r in ablation_rows
                      if r["regime"] == "reduced_k1" and r["ablation"] == "no_last"), 0.0)
    nolast_k3 = next((r["acc"] for r in ablation_rows
                      if r["regime"] == "fuller_k3"  and r["ablation"] == "no_last"), 0.0)
    mid_k3    = decodability.get("fuller_k3", {}).get("mid", 0.0)

    if mid_k3 > 0.7 and nolast_k3 > 0.6:
        conclusion = "PRESERVED"
    elif mid_k3 > 0.4 or nolast_k3 > 0.4:
        conclusion = "PARTIALLY PRESERVED"
    else:
        conclusion = "LOST"

    print(f"\nMOD-12 K3 THROUGH DYNAMICS IS: {conclusion}")

    # ── Build CSV rows ────────────────────────────────────────────────
    csv_rows: List[Dict] = []

    # Training summary rows
    for r in regimes_summary:
        tau0d = tau0_direct_by_regime.get(r["regime"], "")
        csv_rows.append({
            "regime":               r["regime"],
            "position_probe":       "training_summary",
            "linear_decodability":  "",
            "final_accuracy":       r["acc"],
            "solve_step":           r["solve_step"] if r["solve_step"] else "",
            "tau0_direct_accuracy": tau0d,
            "alpha_last":           r["aD"],
            "runtime_seconds":      r["wall"],
            "note":                 "training result",
        })

    # Ablation rows
    for r in ablation_rows:
        tau0d = tau0_direct_by_regime.get(r["regime"], "")
        csv_rows.append({
            "regime":               r["regime"],
            "position_probe":       r["ablation"],
            "linear_decodability":  "",
            "final_accuracy":       r["acc"],
            "solve_step":           "",
            "tau0_direct_accuracy": tau0d,
            "alpha_last":           r["aD"],
            "runtime_seconds":      r["wall"],
            "note":                 r["note"],
        })

    # Decodability probe rows
    for regime_label, probes in decodability.items():
        tau0d = tau0_direct_by_regime.get(regime_label, "")
        r_sum = next(r for r in regimes_summary if r["regime"] == regime_label)
        for pos_name, lin_dec in probes.items():
            csv_rows.append({
                "regime":               regime_label,
                "position_probe":       pos_name,
                "linear_decodability":  lin_dec,
                "final_accuracy":       r_sum["acc"],
                "solve_step":           r_sum["solve_step"] if r_sum["solve_step"] else "",
                "tau0_direct_accuracy": tau0d,
                "alpha_last":           r_sum["aD"],
                "runtime_seconds":      "",
                "note":                 f"linear decodability at {pos_name} position",
            })

    write_csv(csv_rows)
    write_markdown(regimes_summary, ablation_rows, decodability, conclusion)


if __name__ == "__main__":
    main()
