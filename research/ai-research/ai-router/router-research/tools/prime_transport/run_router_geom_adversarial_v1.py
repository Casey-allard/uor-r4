#!/usr/bin/env python3
"""run_router_geom_adversarial_v1.py

ADVERSARIAL ROBUSTNESS OF HARD GEOMETRIC INFERENCE v1

Attempts to BREAK k=1 hard geometric inference under three attack categories.
No fixes. No heuristics. Measure-only.

PHASE A — GEOMETRIC NOISE
  Angular direction perturbations injected into cur_dir before ang_sim.
  noise_std in radians: 0→π. Compared against soft inference under same tau noise.

PHASE B — DISTRIBUTION SHIFT
  B1: D extrapolation — D=64 trained model evaluated at D={80,96,128}.
      pos0_mask zero-padded; OOD sequence length; no retraining.
  B2: Adversarial pool — uniform random pool from all 343K states
      instead of the training-distribution 4K pool.

PHASE C — AMBIGUITY / TIE STRESS
  C1: Margin distribution — characterize natural top1-vs-top2 ang_sim gaps.
  C2: Random tie-breaking — when margin < threshold, pick randomly among tied candidates.
  C3: Targeted swap attack — inject noise calibrated to current margin (forces ties).
  C4: Clustered-state subset — evaluate only on states where all 6 TN candidates
      are geometrically similar (high pairwise cosine similarity = inherently ambiguous).

TRAINS:
  - D=24 model (3000 steps) — used for phases A, B2, C
  - D=64 model (5000 steps) — used for phase B1 extrapolation

LOCKED: do not retrain to fix failures. Measure and report only.
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

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_geom_adversarial_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_geom_adversarial_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D_HIDDEN      = 32
BATCH_SIZE    = 256
B0_INIT       = 2.0
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
TRANSPORT_TH  = 3
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 1024
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
FAILURE_TH    = 0.99    # drop below this = failure
DEGRADE_TH    = 0.001   # absolute drop from clean = any degradation

# Attack parameters
ANGULAR_NOISE_LEVELS = [0.0, 0.05, 0.1, 0.2, 0.5, 1.0, 1.57, 3.14]   # radians std
D_EXTRAP_LEVELS      = [64, 80, 96, 128]
TIE_THRESHOLDS       = [0.5, 0.2, 0.1, 0.05, 0.01]   # fraction of max possible margin
SWAP_FACTORS         = [0.25, 0.5, 1.0, 2.0]           # noise = factor × margin
N_TRIALS             = 3    # repeat stochastic tests

# Training budget
MAX_STEPS_D24 = 3_000
MAX_STEPS_D64 = 5_000

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _select_threads
    _select_threads(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion + hybrid tables (locked)
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG, dtype=onehot.dtype)
    ai = 0
    for s, e, m in PHASE_BLOCKS:
        k = onehot[..., s:e].argmax(dim=-1).float()
        angle = 2.0 * math.pi * k / float(m)
        out[..., ai]     = torch.cos(angle)
        out[..., ai + 1] = torch.sin(angle)
        ai += 2
    return out


def prepare_hybrid_tables(TN_oh, tau0_oh, TR, pool_ids):
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Model (locked baseline, d_context parameterised)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularHybrid(nn.Module):
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 d_hidden=D_HIDDEN, d_context=24,
                 b0_init=B0_INIT, init_scale=INIT_SCALE, seed=GLOBAL_SEED):
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))
        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);    self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);         self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU_HYB);   self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(D_TAU_HYB, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU_HYB)

    def forward(self, state_ids, tokens, x0, temp):
        B = state_ids.shape[0]; D_len = tokens.shape[1]
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D_len):
            tn   = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base  = torch.einsum("bi,bij->bj", w, tn)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()
            dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
            hybrid = torch.cat([dirn, mag], dim=1)
            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Utilities
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, rng, d):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng)
    toks = torch.randint(VOCAB, (BATCH_SIZE, d), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def perturb_direction(cur_dir: torch.Tensor, noise_std: float) -> torch.Tensor:
    """Rotate each phase pair (cos θ, sin θ) by δ ~ N(0, noise_std) radians."""
    if noise_std == 0.0:
        return cur_dir
    B = cur_dir.shape[0]
    pairs = cur_dir.view(B, N_PHASE_PAIRS, 2)
    delta = torch.randn(B, N_PHASE_PAIRS) * noise_std
    cos_d = torch.cos(delta); sin_d = torch.sin(delta)
    c = pairs[:, :, 0]; s = pairs[:, :, 1]
    new_c = c * cos_d - s * sin_d
    new_s = s * cos_d + c * sin_d
    return torch.stack([new_c, new_s], dim=2).view(B, D_TAU_ANG)


def train_model(TN_ang, TR, tau0_hyb, pool_ids, d, max_steps, label):
    model = RouterAngularHybrid(TN_ang, TR, tau0_hyb, pool_ids, d_context=d)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    rng_e = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    model.train()
    t0 = time.perf_counter()
    solve_step = None
    final_acc  = 0.0
    for step in range(1, max_steps + 1):
        frac   = step / max(max_steps - 1, 1)
        temp   = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t, d)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc = _eval_soft(model, pool_ids, d, GLOBAL_SEED + 200)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc
            print(f"    [{label}] step={step:5d}  acc={acc:.4f}")
    wall = time.perf_counter() - t0
    print(f"  {label}: acc={final_acc:.4f}  solve={solve_step}  "
          f"sps={max_steps/wall:.1f}  wall={wall:.1f}s")
    model.eval()
    return model, final_acc, solve_step


def _eval_soft(model, pool_ids, d, seed=GLOBAL_SEED + 200):
    model.eval()
    rng = torch.Generator().manual_seed(seed)
    correct = 0
    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / (N_EVAL // BATCH_SIZE * BATCH_SIZE)


def _eval_hard_geom_core(model, pool_ids, d, seed,
                         noise_std=0.0,
                         tie_threshold=0.0,
                         swap_factor=0.0,
                         pos0_mask_override=None,
                         collect_margins=False):
    """
    Unified hard geometry eval with optional attack parameters.

    noise_std        — angular direction perturbation (Phase A)
    tie_threshold    — fraction of max-possible margin; if actual margin < this,
                       pick randomly among top-2 (Phase C2)
    swap_factor      — inject noise = swap_factor * actual_margin (Phase C3)
    pos0_mask_override — extended mask for D extrapolation (Phase B1)
    collect_margins  — if True, also return list of per-step margins
    """
    model.eval()
    rng = torch.Generator().manual_seed(seed)
    correct = 0
    total = 0
    margins_all = []

    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            B = sids.shape[0]
            tau_prev  = model.tau0_table[sids]
            soft_taus = []
            sids_loop = sids.clone()

            for t in range(d):
                tn      = model.TN[sids_loop]                     # (B, 6, 8)
                cur_dir = tau_prev[:, :D_TAU_ANG]                 # (B, 8)

                # Phase A: angular direction noise
                if noise_std > 0.0:
                    cur_dir = perturb_direction(cur_dir, noise_std)

                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)  # (B, 6)

                # Track margins (top1 - top2 similarity)
                top2_vals, _ = ang_sim.topk(2, dim=1)
                margin = (top2_vals[:, 0] - top2_vals[:, 1])      # (B,)
                if collect_margins:
                    margins_all.append(margin.cpu())

                # Phase C3: targeted swap — noise calibrated to current margin
                if swap_factor > 0.0:
                    swap_std = (margin * swap_factor).unsqueeze(1).expand(B, N_PHASE_PAIRS)
                    delta = torch.randn(B, N_PHASE_PAIRS) * swap_std
                    pairs = cur_dir.view(B, N_PHASE_PAIRS, 2)
                    cos_d = torch.cos(delta); sin_d = torch.sin(delta)
                    c = pairs[:, :, 0]; s = pairs[:, :, 1]
                    new_c = c * cos_d - s * sin_d
                    new_s = s * cos_d + c * sin_d
                    cur_dir = torch.stack([new_c, new_s], dim=2).view(B, D_TAU_ANG)
                    ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                    top2_vals2, _ = ang_sim.topk(2, dim=1)
                    margin = top2_vals2[:, 0] - top2_vals2[:, 1]

                # Phase C2: random tie-breaking when margin < threshold
                if tie_threshold > 0.0:
                    tied = margin < tie_threshold                   # (B,) bool mask
                    best_geom = ang_sim.argmax(dim=1).clone()
                    if tied.any():
                        rand_choice = torch.randint(N_OPS, (B,))
                        best_geom[tied] = rand_choice[tied]
                else:
                    best_geom = ang_sim.argmax(dim=1)              # (B,)

                # Hard tau: exact TN entry for best candidate
                best_ang = tn.gather(
                    1, best_geom.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                ).squeeze(1)
                hybrid = torch.cat(
                    [best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1
                )
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids_loop = model.TR[sids_loop].gather(
                    1, best_geom.unsqueeze(1)).squeeze(1)

            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            mask  = pos0_mask_override if pos0_mask_override is not None else model.pos0_mask
            sc    = (h_a * model.v_attn).sum(dim=-1) + mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            pred  = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += (pred.argmax(1) == x0).sum().item()
            total   += B

    model.train()
    acc = round(correct / total, 4)
    if collect_margins:
        all_m = torch.cat(margins_all)
        return acc, all_m
    return acc


def _eval_soft_noisy(model, pool_ids, d, noise_std, seed, pos0_mask_override=None):
    """Soft inference with tau_prev direction noise at each step."""
    model.eval()
    rng = torch.Generator().manual_seed(seed)
    correct = 0
    total   = 0

    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            B = sids.shape[0]
            tau_prev  = model.tau0_table[sids]
            soft_taus = []
            sids_loop = sids.clone()

            for t in range(d):
                tn   = model.TN[sids_loop]
                # Inject noise into the direction part of tau_prev
                if noise_std > 0.0:
                    dir_noisy = perturb_direction(tau_prev[:, :D_TAU_ANG], noise_std)
                    tau_noisy = torch.cat([dir_noisy, tau_prev[:, D_TAU_ANG:]], dim=1)
                else:
                    tau_noisy = tau_prev
                embs = model.W_emb[toks[:, t]]
                h    = torch.tanh(torch.cat([embs, tau_noisy], dim=1) @ model.W1 + model.b1)
                w    = torch.softmax((h @ model.W2 + model.b2) / 0.05, dim=1)
                base = torch.einsum("bi,bij->bj", w, tn)
                pairs = base.view(B, N_PHASE_PAIRS, 2)
                mag  = (pairs * pairs).sum(dim=2).sqrt()
                dirn = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
                hybrid   = torch.cat([dirn, mag], dim=1)
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids_loop = model.TR[sids_loop].gather(
                    1, w.argmax(dim=1).unsqueeze(1)).squeeze(1)

            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            mask  = pos0_mask_override if pos0_mask_override is not None else model.pos0_mask
            sc    = (h_a * model.v_attn).sum(dim=-1) + mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            pred  = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += (pred.argmax(1) == x0).sum().item()
            total   += B

    model.train()
    return round(correct / total, 4)


# ═══════════════════════════════════════════════════════════════════════
# Phase A — Angular Noise
# ═══════════════════════════════════════════════════════════════════════
def run_phase_a(model_24, pool_ids) -> List[Dict]:
    print("\n" + "="*60)
    print("PHASE A: ANGULAR DIRECTION NOISE")
    print("="*60)
    rows = []
    d = 24

    # Baseline (noise=0) — both modes
    base_hard = _eval_hard_geom_core(model_24, pool_ids, d, GLOBAL_SEED + 888)
    base_soft = _eval_soft(model_24, pool_ids, d, GLOBAL_SEED + 200)
    print(f"  Baseline: hard={base_hard:.4f}  soft={base_soft:.4f}")

    for noise_std in ANGULAR_NOISE_LEVELS:
        hard_accs = []
        soft_accs = []
        for trial in range(N_TRIALS):
            seed = GLOBAL_SEED + 888 + trial * 100
            h = _eval_hard_geom_core(model_24, pool_ids, d, seed, noise_std=noise_std)
            s = _eval_soft_noisy(model_24, pool_ids, d, noise_std, seed)
            hard_accs.append(h)
            soft_accs.append(s)

        hard_mean = round(sum(hard_accs) / N_TRIALS, 4)
        hard_std  = round((sum((x - hard_mean)**2 for x in hard_accs) / N_TRIALS)**0.5, 4)
        soft_mean = round(sum(soft_accs) / N_TRIALS, 4)
        soft_std  = round((sum((x - soft_mean)**2 for x in soft_accs) / N_TRIALS)**0.5, 4)

        hard_deg  = round(base_hard - hard_mean, 4)
        soft_deg  = round(base_soft - soft_mean, 4)
        hard_fail = hard_mean < FAILURE_TH
        soft_fail = soft_mean < FAILURE_TH

        noise_deg = round(math.degrees(noise_std), 1) if noise_std > 0 else 0.0
        print(f"  noise={noise_std:.3f}rad ({noise_deg:5.1f}°) "
              f"hard={hard_mean:.4f}±{hard_std:.4f}(Δ{hard_deg:+.4f})"
              f"  soft={soft_mean:.4f}±{soft_std:.4f}(Δ{soft_deg:+.4f})"
              + (" ← HARD FAIL" if hard_fail else "")
              + (" ← SOFT FAIL" if soft_fail else ""))

        rows.append({
            "phase": "A_angular_noise", "attack": f"noise_std={noise_std:.3f}",
            "D": d, "infer_mode": "hard_geom",
            "accuracy": hard_mean, "accuracy_std": hard_std,
            "delta_from_clean": -hard_deg, "failure": hard_fail,
            "noise_std_rad": noise_std, "noise_deg": noise_deg,
            "note": f"{noise_deg:.1f}° angular rotation noise on cur_dir",
        })
        rows.append({
            "phase": "A_angular_noise", "attack": f"noise_std={noise_std:.3f}",
            "D": d, "infer_mode": "soft_mlp",
            "accuracy": soft_mean, "accuracy_std": soft_std,
            "delta_from_clean": -soft_deg, "failure": soft_fail,
            "noise_std_rad": noise_std, "noise_deg": noise_deg,
            "note": f"{noise_deg:.1f}° angular rotation noise on tau_prev",
        })

    return rows


# ═══════════════════════════════════════════════════════════════════════
# Phase B — Distribution Shift
# ═══════════════════════════════════════════════════════════════════════
def run_phase_b1(model_64, pool_ids) -> List[Dict]:
    """B1: D extrapolation — test D=64 model at D > 64."""
    print("\n" + "="*60)
    print("PHASE B1: D EXTRAPOLATION (D=64 model, OOD sequence lengths)")
    print("="*60)
    rows = []
    d_train = 64

    for d_eval in D_EXTRAP_LEVELS:
        # Build extended pos0_mask (zero-padded beyond d_train)
        if d_eval != d_train:
            ext_mask = torch.zeros(1, d_eval)
            ext_mask[0, 0] = model_64.pos0_mask[0, 0].item()  # keep pos-0 bias
        else:
            ext_mask = None  # use model's own mask

        hard_acc = _eval_hard_geom_core(model_64, pool_ids, d_eval,
                                         GLOBAL_SEED + 888,
                                         pos0_mask_override=ext_mask)
        soft_acc = _eval_soft_noisy(model_64, pool_ids, d_eval, 0.0,
                                     GLOBAL_SEED + 200,
                                     pos0_mask_override=ext_mask)

        flag_h = " ← HARD FAIL" if hard_acc < FAILURE_TH else ""
        flag_s = " ← SOFT FAIL" if soft_acc < FAILURE_TH else ""
        ood = " (OOD)" if d_eval > d_train else ""
        print(f"  D_eval={d_eval:3d}{ood}  hard={hard_acc:.4f}{flag_h}"
              f"  soft={soft_acc:.4f}{flag_s}")

        for mode, acc in [("hard_geom", hard_acc), ("soft_mlp", soft_acc)]:
            rows.append({
                "phase": "B1_d_extrapolation", "attack": f"d_eval={d_eval}",
                "D": d_eval, "infer_mode": mode,
                "accuracy": acc, "accuracy_std": 0.0,
                "delta_from_clean": 0.0, "failure": acc < FAILURE_TH,
                "noise_std_rad": 0.0, "noise_deg": 0.0,
                "note": f"D=64 model tested at D={d_eval}"
                        + (" (OOD extrapolation)" if d_eval > d_train else " (in-distribution)"),
            })

    return rows


def run_phase_b2(model_24, pool_ids, TN_ang, TR, tau0_hyb, n_states) -> List[Dict]:
    """B2: Adversarial pool — random states from full 343K space."""
    print("\n" + "="*60)
    print("PHASE B2: ADVERSARIAL POOL (random initial states)")
    print("="*60)
    rows = []
    d = 24

    # Clean baseline with training pool
    base_hard = _eval_hard_geom_core(model_24, pool_ids, d, GLOBAL_SEED + 888)
    base_soft = _eval_soft(model_24, pool_ids, d, GLOBAL_SEED + 200)

    # Adversarial pool: uniform random from all 343K states
    rng_pool = torch.Generator().manual_seed(GLOBAL_SEED + 5555)
    adv_pool = torch.randperm(n_states, generator=rng_pool)[:pool_ids.shape[0]]

    adv_hard = _eval_hard_geom_core(model_24, adv_pool, d, GLOBAL_SEED + 888)
    adv_soft = _eval_soft_noisy(model_24, adv_pool, d, 0.0, GLOBAL_SEED + 200)

    print(f"  Training pool:    hard={base_hard:.4f}  soft={base_soft:.4f}")
    print(f"  Adversarial pool: hard={adv_hard:.4f}  soft={adv_soft:.4f}"
          + (" ← HARD FAIL" if adv_hard < FAILURE_TH else "")
          + (" ← SOFT FAIL" if adv_soft < FAILURE_TH else ""))

    for pool_type, hard_acc, soft_acc in [
        ("training_pool",    base_hard, base_soft),
        ("adversarial_pool", adv_hard,  adv_soft),
    ]:
        for mode, acc in [("hard_geom", hard_acc), ("soft_mlp", soft_acc)]:
            rows.append({
                "phase": "B2_adv_pool", "attack": pool_type,
                "D": d, "infer_mode": mode,
                "accuracy": acc, "accuracy_std": 0.0,
                "delta_from_clean": 0.0, "failure": acc < FAILURE_TH,
                "noise_std_rad": 0.0, "noise_deg": 0.0,
                "note": pool_type,
            })

    return rows


# ═══════════════════════════════════════════════════════════════════════
# Phase C — Tie / Ambiguity Stress
# ═══════════════════════════════════════════════════════════════════════
def run_phase_c(model_24, pool_ids, TN_ang) -> List[Dict]:
    print("\n" + "="*60)
    print("PHASE C: TIE / AMBIGUITY STRESS")
    print("="*60)
    rows = []
    d = 24

    # ── C1: Margin distribution analysis ───────────────────────────────
    print("\n  C1: Collecting margin distribution from 2000 steps ...")
    rng_m = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    acc_c1, all_margins = _eval_hard_geom_core(
        model_24, pool_ids, d, GLOBAL_SEED + 888, collect_margins=True
    )
    m_mean  = round(all_margins.mean().item(), 4)
    m_std   = round(all_margins.std().item(), 4)
    m_p10   = round(all_margins.quantile(0.10).item(), 4)
    m_p25   = round(all_margins.quantile(0.25).item(), 4)
    m_p50   = round(all_margins.quantile(0.50).item(), 4)
    m_p90   = round(all_margins.quantile(0.90).item(), 4)
    n_tight = int((all_margins < 0.01).sum().item())
    print(f"    margin stats: mean={m_mean}  std={m_std}")
    print(f"    p10={m_p10}  p25={m_p25}  p50={m_p50}  p90={m_p90}")
    print(f"    steps with margin < 0.01: {n_tight}/{len(all_margins)} "
          f"({100*n_tight/len(all_margins):.1f}%)")

    rows.append({
        "phase": "C1_margin_analysis", "attack": "margin_distribution",
        "D": d, "infer_mode": "hard_geom",
        "accuracy": acc_c1, "accuracy_std": 0.0,
        "delta_from_clean": 0.0, "failure": False,
        "noise_std_rad": 0.0, "noise_deg": 0.0,
        "note": f"margin_mean={m_mean} std={m_std} p10={m_p10} p50={m_p50} "
                f"p90={m_p90} tight_frac={100*n_tight/len(all_margins):.1f}%",
    })

    # ── C2: Random tie-breaking ─────────────────────────────────────────
    print("\n  C2: Random tie-breaking (threshold on absolute ang_sim margin) ...")
    base_hard = acc_c1  # already have this

    for threshold in TIE_THRESHOLDS:
        accs = []
        for trial in range(N_TRIALS):
            seed = GLOBAL_SEED + 777 + trial * 50
            acc = _eval_hard_geom_core(model_24, pool_ids, d, seed,
                                        tie_threshold=threshold)
            accs.append(acc)
        acc_mean = round(sum(accs) / N_TRIALS, 4)
        acc_std  = round((sum((x-acc_mean)**2 for x in accs) / N_TRIALS)**0.5, 4)
        frac_affected = round(float((all_margins < threshold).float().mean()), 4)
        print(f"    threshold={threshold:.3f}  acc={acc_mean:.4f}±{acc_std:.4f}  "
              f"frac_steps_affected={frac_affected:.4f}"
              + (" ← FAIL" if acc_mean < FAILURE_TH else ""))
        rows.append({
            "phase": "C2_random_tie", "attack": f"tie_thr={threshold}",
            "D": d, "infer_mode": "hard_geom",
            "accuracy": acc_mean, "accuracy_std": acc_std,
            "delta_from_clean": round(base_hard - acc_mean, 4),
            "failure": acc_mean < FAILURE_TH,
            "noise_std_rad": 0.0, "noise_deg": 0.0,
            "note": f"random tie-break when margin<{threshold}; "
                    f"{100*frac_affected:.1f}% steps affected",
        })

    # ── C3: Targeted swap attack ────────────────────────────────────────
    print("\n  C3: Targeted swap attack (noise = swap_factor × margin) ...")
    for swap_factor in SWAP_FACTORS:
        accs = []
        for trial in range(N_TRIALS):
            seed = GLOBAL_SEED + 999 + trial * 50
            acc = _eval_hard_geom_core(model_24, pool_ids, d, seed,
                                        swap_factor=swap_factor)
            accs.append(acc)
        acc_mean = round(sum(accs) / N_TRIALS, 4)
        acc_std  = round((sum((x-acc_mean)**2 for x in accs) / N_TRIALS)**0.5, 4)
        print(f"    swap_factor={swap_factor:.2f}  acc={acc_mean:.4f}±{acc_std:.4f}"
              + (" ← FAIL" if acc_mean < FAILURE_TH else ""))
        rows.append({
            "phase": "C3_targeted_swap", "attack": f"swap_factor={swap_factor}",
            "D": d, "infer_mode": "hard_geom",
            "accuracy": acc_mean, "accuracy_std": acc_std,
            "delta_from_clean": round(base_hard - acc_mean, 4),
            "failure": acc_mean < FAILURE_TH,
            "noise_std_rad": 0.0, "noise_deg": 0.0,
            "note": f"targeted swap noise = {swap_factor}×margin per step",
        })

    # ── C4: Clustered-state subset ──────────────────────────────────────
    print("\n  C4: Clustered-state subset (high pairwise TN similarity) ...")

    # Compute per-state pairwise TN similarity
    pool_TN = TN_ang[pool_ids]                                    # (P, 6, 8)
    pairwise = torch.einsum("nid,njd->nij", pool_TN, pool_TN)    # (P, 6, 6)
    triu_mask = torch.triu(torch.ones(N_OPS, N_OPS), diagonal=1).bool()
    clustering = pairwise[:, triu_mask].mean(dim=1)               # (P,) mean pairwise sim

    c_mean = round(clustering.mean().item(), 4)
    c_std  = round(clustering.std().item(), 4)
    c_p75  = round(clustering.quantile(0.75).item(), 4)
    c_p90  = round(clustering.quantile(0.90).item(), 4)
    print(f"    Clustering stats: mean={c_mean}  std={c_std}  "
          f"p75={c_p75}  p90={c_p90}")

    for pct, label_c in [(50, "top50pct"), (25, "top25pct"), (10, "top10pct")]:
        threshold_c = clustering.quantile(1.0 - pct/100.0).item()
        clustered_mask = clustering >= threshold_c
        clustered_pool = pool_ids[clustered_mask]
        n_clustered = clustered_pool.shape[0]

        hard_acc = _eval_hard_geom_core(model_24, clustered_pool, d, GLOBAL_SEED + 888)
        soft_acc = _eval_soft_noisy(model_24, clustered_pool, d, 0.0, GLOBAL_SEED + 200)
        print(f"    {label_c} (n={n_clustered}, cluster≥{threshold_c:.3f}): "
              f"hard={hard_acc:.4f}  soft={soft_acc:.4f}"
              + (" ← HARD FAIL" if hard_acc < FAILURE_TH else ""))

        for mode, acc in [("hard_geom", hard_acc), ("soft_mlp", soft_acc)]:
            rows.append({
                "phase": "C4_clustered_states", "attack": label_c,
                "D": d, "infer_mode": mode,
                "accuracy": acc, "accuracy_std": 0.0,
                "delta_from_clean": round(base_hard - acc, 4) if mode == "hard_geom" else 0.0,
                "failure": acc < FAILURE_TH,
                "noise_std_rad": 0.0, "noise_deg": 0.0,
                "note": f"{pct}% most-clustered states (n={n_clustered}); "
                        f"mean_cluster_sim≥{threshold_c:.3f}",
            })

    return rows, m_mean, m_std, m_p10, m_p50, m_p90, n_tight, len(all_margins)


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(all_rows, margin_stats, base_hard_24, base_soft_24):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    m_mean, m_std, m_p10, m_p50, m_p90, n_tight, n_total = margin_stats

    # Determine overall verdict
    any_hard_fail = any(r["failure"] and r["infer_mode"] == "hard_geom" for r in all_rows)
    first_hard_fail = next(
        (r for r in all_rows if r["failure"] and r["infer_mode"] == "hard_geom"), None
    )

    # Count fails by phase
    fails_by_phase: Dict[str, List] = {}
    for r in all_rows:
        if r["failure"] and r["infer_mode"] == "hard_geom":
            p = r["phase"]
            fails_by_phase.setdefault(p, []).append(r)

    L: List[str] = []
    L.append("# Prime Transport Router: Adversarial Robustness of Hard Geometric Inference v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, CPU  \n")
    L.append(f"**Baseline:** hard_geom={base_hard_24:.4f}, soft_mlp={base_soft_24:.4f} (D=24, clean)  \n\n")

    L.append("---\n\n")
    L.append("## Phase A — Angular Direction Noise\n\n")
    L.append("Angular perturbation δ ~ N(0, σ) radians per phase pair applied to cur_dir "
             "before ang_sim.  \nSame perturbation applied to tau_prev direction for soft "
             "inference (fair comparison).\n\n")
    L.append("| noise_std (rad) | noise (°) | hard_acc | hard_std | Δ_hard | "
             "soft_acc | soft_std | Δ_soft | hard_fail? | soft_fail? |\n")
    L.append("|-----------------|-----------|----------|----------|--------|"
             "---------|---------|--------|-----------|----------|\n")
    a_rows_h = [r for r in all_rows if r["phase"] == "A_angular_noise" and r["infer_mode"] == "hard_geom"]
    a_rows_s = [r for r in all_rows if r["phase"] == "A_angular_noise" and r["infer_mode"] == "soft_mlp"]
    for rh, rs in zip(a_rows_h, a_rows_s):
        L.append(f"| {rh['noise_std_rad']:.3f} | {rh['noise_deg']:.1f} "
                 f"| {rh['accuracy']:.4f} | {rh['accuracy_std']:.4f} "
                 f"| {rh['delta_from_clean']:+.4f} "
                 f"| {rs['accuracy']:.4f} | {rs['accuracy_std']:.4f} "
                 f"| {rs['delta_from_clean']:+.4f} "
                 f"| {'YES ✗' if rh['failure'] else 'no'} "
                 f"| {'YES ✗' if rs['failure'] else 'no'} |\n")
    L.append("\n")
    # First A failure
    first_a_hard = next((r for r in a_rows_h if r["failure"]), None)
    if first_a_hard:
        L.append(f"**First hard geom failure:** noise_std={first_a_hard['noise_std_rad']:.3f}rad "
                 f"({first_a_hard['noise_deg']:.1f}°), acc={first_a_hard['accuracy']:.4f}  \n")
    else:
        L.append("**Hard geom failure:** NONE across all tested noise levels.  \n")
    first_a_soft = next((r for r in a_rows_s if r["failure"]), None)
    if first_a_soft:
        L.append(f"**First soft failure:** noise_std={first_a_soft['noise_std_rad']:.3f}rad "
                 f"({first_a_soft['noise_deg']:.1f}°), acc={first_a_soft['accuracy']:.4f}  \n")
    else:
        L.append("**Soft failure:** NONE across all tested noise levels.  \n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Phase B1 — D Extrapolation\n\n")
    L.append("D=64 trained model evaluated at D > 64. "
             "pos0_mask zero-padded to new length. No retraining.\n\n")
    L.append("| D_eval | OOD? | hard_acc | soft_acc | hard_fail? | soft_fail? |\n")
    L.append("|--------|------|----------|----------|-----------|----------|\n")
    b1_rows_h = [r for r in all_rows if r["phase"] == "B1_d_extrapolation" and r["infer_mode"] == "hard_geom"]
    b1_rows_s = [r for r in all_rows if r["phase"] == "B1_d_extrapolation" and r["infer_mode"] == "soft_mlp"]
    for rh, rs in zip(b1_rows_h, b1_rows_s):
        ood = "yes" if rh["D"] > 64 else "no"
        L.append(f"| {rh['D']} | {ood} | {rh['accuracy']:.4f} | {rs['accuracy']:.4f} "
                 f"| {'YES ✗' if rh['failure'] else 'no'} "
                 f"| {'YES ✗' if rs['failure'] else 'no'} |\n")
    L.append("\n")
    first_b1_hard = next((r for r in b1_rows_h if r["failure"]), None)
    if first_b1_hard:
        L.append(f"**First hard geom failure at D_eval={first_b1_hard['D']}**  \n\n")
    else:
        L.append("**No failure observed in D extrapolation range.**  \n\n")

    L.append("---\n\n")
    L.append("## Phase B2 — Adversarial Pool\n\n")
    L.append("Uniform random pool from all 343K states vs training-distribution pool.\n\n")
    L.append("| pool | hard_acc | soft_acc | hard_fail? |\n")
    L.append("|------|----------|----------|-----------|\n")
    b2_h = [r for r in all_rows if r["phase"] == "B2_adv_pool" and r["infer_mode"] == "hard_geom"]
    b2_s = [r for r in all_rows if r["phase"] == "B2_adv_pool" and r["infer_mode"] == "soft_mlp"]
    for rh, rs in zip(b2_h, b2_s):
        L.append(f"| {rh['attack']} | {rh['accuracy']:.4f} | {rs['accuracy']:.4f} "
                 f"| {'YES ✗' if rh['failure'] else 'no'} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Phase C — Tie / Ambiguity Stress\n\n")

    L.append("### C1: Margin Distribution\n\n")
    L.append(f"Natural top-1 vs top-2 angular similarity gap during clean hard geom inference:  \n")
    L.append(f"- mean = {m_mean}  std = {m_std}  \n")
    L.append(f"- p10 = {m_p10}  p50 = {m_p50}  p90 = {m_p90}  \n")
    L.append(f"- steps with margin < 0.01: {n_tight}/{n_total} "
             f"({100*n_tight/n_total:.1f}%)  \n\n")

    L.append("### C2: Random Tie-Breaking\n\n")
    L.append("| tie_threshold | acc | std | Δ | frac_affected | fail? |\n")
    L.append("|---------------|-----|-----|---|---------------|-------|\n")
    c2_rows = [r for r in all_rows if r["phase"] == "C2_random_tie"]
    for r in c2_rows:
        L.append(f"| {r['attack']} | {r['accuracy']:.4f} | {r['accuracy_std']:.4f} "
                 f"| {r['delta_from_clean']:+.4f} | (see note) "
                 f"| {'YES ✗' if r['failure'] else 'no'} |\n")
    L.append("\n")
    for r in c2_rows:
        L.append(f"- {r['attack']}: {r['note']}  \n")
    L.append("\n")

    L.append("### C3: Targeted Swap Attack\n\n")
    L.append("Noise calibrated to current margin magnitude at each step.\n\n")
    L.append("| swap_factor | acc | std | Δ | fail? |\n")
    L.append("|-------------|-----|-----|---|-------|\n")
    c3_rows = [r for r in all_rows if r["phase"] == "C3_targeted_swap"]
    for r in c3_rows:
        L.append(f"| {r['attack']} | {r['accuracy']:.4f} | {r['accuracy_std']:.4f} "
                 f"| {r['delta_from_clean']:+.4f} | {'YES ✗' if r['failure'] else 'no'} |\n")
    L.append("\n")

    L.append("### C4: Clustered-State Subset\n\n")
    L.append("Eval restricted to states where TN candidates are most geometrically similar.\n\n")
    L.append("| subset | hard_acc | soft_acc | hard_fail? |\n")
    L.append("|--------|----------|----------|-----------|\n")
    c4_h = [r for r in all_rows if r["phase"] == "C4_clustered_states" and r["infer_mode"] == "hard_geom"]
    c4_s = [r for r in all_rows if r["phase"] == "C4_clustered_states" and r["infer_mode"] == "soft_mlp"]
    for rh, rs in zip(c4_h, c4_s):
        L.append(f"| {rh['attack']} | {rh['accuracy']:.4f} | {rs['accuracy']:.4f} "
                 f"| {'YES ✗' if rh['failure'] else 'no'} |\n")
    L.append("\n")
    for r in c4_h:
        L.append(f"- {r['attack']}: {r['note']}  \n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Failure Summary\n\n")
    if any_hard_fail:
        L.append("Hard geometric inference **FAILED** under the following conditions:\n\n")
        for phase, fail_list in fails_by_phase.items():
            for r in fail_list:
                L.append(f"- [{phase}] {r['attack']}: acc={r['accuracy']:.4f}  \n")
        L.append("\n")
        # Determine worst failure
        worst = min(fails_by_phase.values(), key=lambda lst: lst[0]["accuracy"])
        worst_r = worst[0]
        L.append(f"**Worst failure:** [{worst_r['phase']}] {worst_r['attack']} "
                 f"→ acc={worst_r['accuracy']:.4f}  \n\n")
        # Compare to soft
        soft_failures = [r for r in all_rows if r["failure"] and r["infer_mode"] == "soft_mlp"]
        if soft_failures:
            L.append("Soft inference also failed under some of the same conditions:  \n")
            for r in soft_failures[:3]:
                L.append(f"- [{r['phase']}] {r['attack']}: soft acc={r['accuracy']:.4f}  \n")
        else:
            L.append("Soft inference did **NOT** fail under any of the same conditions.  \n")
        L.append("\n")
    else:
        L.append("Hard geometric inference survived all adversarial conditions tested.  \n")
        L.append("No accuracy below {:.2f} was observed in any phase.  \n\n".format(FAILURE_TH))

    soft_only_fails = [r for r in all_rows if r["failure"] and r["infer_mode"] == "soft_mlp"]
    if soft_only_fails and not any_hard_fail:
        L.append("> Note: Soft MLP inference failed under some conditions where "
                 "hard geometry did not. Hard geometry may be more noise-robust "
                 "due to its discretisation of the routing decision.\n\n")
    elif soft_only_fails:
        L.append("> Note: Soft MLP inference also showed failures under some conditions.\n\n")

    L.append("---\n\n")

    # Verdict
    n_hard_fails = sum(1 for r in all_rows if r["failure"] and r["infer_mode"] == "hard_geom")
    total_hard = sum(1 for r in all_rows if r["infer_mode"] == "hard_geom")
    fail_rate = n_hard_fails / total_hard if total_hard > 0 else 0

    if n_hard_fails == 0:
        verdict = "ROBUST"
    elif fail_rate < 0.15:
        verdict = "PARTIALLY ROBUST"
    else:
        verdict = "FRAGILE"

    L.append(f"## HARD GEOMETRIC INFERENCE IS: {verdict}\n\n")
    L.append(f"({n_hard_fails}/{total_hard} tested conditions resulted in failure "
             f"[acc < {FAILURE_TH}])  \n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("ADVERSARIAL ROBUSTNESS OF HARD GEOMETRIC INFERENCE v1")
    print("=" * 60)

    # Load cache
    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    t_load = time.perf_counter() - t0
    n_states = data["TN_oh"].shape[0]
    print(f"  {n_states:,} states in {t_load:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )

    # Train D=24 model (phases A, B2, C)
    print("\nTraining D=24 model (for phases A, B2, C) ...")
    model_24, acc_24, solve_24 = train_model(
        TN_ang, TR, tau0_hyb, pool_ids, d=24, max_steps=MAX_STEPS_D24, label="D=24"
    )
    base_hard_24 = _eval_hard_geom_core(model_24, pool_ids, 24, GLOBAL_SEED + 888)
    base_soft_24 = _eval_soft(model_24, pool_ids, 24, GLOBAL_SEED + 200)
    print(f"  Baseline D=24: hard={base_hard_24:.4f}  soft={base_soft_24:.4f}")

    # Train D=64 model (phase B1)
    print("\nTraining D=64 model (for phase B1 extrapolation) ...")
    model_64, acc_64, solve_64 = train_model(
        TN_ang, TR, tau0_hyb, pool_ids, d=64, max_steps=MAX_STEPS_D64, label="D=64"
    )

    all_rows: List[Dict] = []

    # Phase A
    rows_a = run_phase_a(model_24, pool_ids)
    all_rows.extend(rows_a)

    # Phase B1
    rows_b1 = run_phase_b1(model_64, pool_ids)
    all_rows.extend(rows_b1)

    # Phase B2
    rows_b2 = run_phase_b2(model_24, pool_ids, TN_ang, TR, tau0_hyb, n_states)
    all_rows.extend(rows_b2)

    # Phase C
    result_c = run_phase_c(model_24, pool_ids, TN_ang)
    rows_c   = result_c[0]
    margin_stats = result_c[1:]   # m_mean, m_std, m_p10, m_p50, m_p90, n_tight, n_total
    all_rows.extend(rows_c)

    # Summary
    print("\n" + "="*60)
    print("ADVERSARIAL SUMMARY")
    print("="*60)
    hard_failures = [r for r in all_rows if r["failure"] and r["infer_mode"] == "hard_geom"]
    soft_failures = [r for r in all_rows if r["failure"] and r["infer_mode"] == "soft_mlp"]
    print(f"  Hard geom failures: {len(hard_failures)} / "
          f"{sum(1 for r in all_rows if r['infer_mode']=='hard_geom')} conditions")
    print(f"  Soft MLP failures:  {len(soft_failures)} / "
          f"{sum(1 for r in all_rows if r['infer_mode']=='soft_mlp')} conditions")
    if hard_failures:
        for r in hard_failures:
            print(f"    HARD FAIL: [{r['phase']}] {r['attack']} → acc={r['accuracy']:.4f}")
    else:
        print("  No hard geom failures detected.")

    # CSV
    fieldnames = [
        "phase", "attack", "D", "infer_mode",
        "accuracy", "accuracy_std", "delta_from_clean", "failure",
        "noise_std_rad", "noise_deg", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(all_rows)
    print(f"\nCSV written: {CSV_OUT}")

    # Markdown
    _write_markdown(all_rows, (margin_stats[0], margin_stats[1], margin_stats[2],
                               margin_stats[3], margin_stats[4], margin_stats[5],
                               margin_stats[6]),
                    base_hard_24, base_soft_24)
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()
