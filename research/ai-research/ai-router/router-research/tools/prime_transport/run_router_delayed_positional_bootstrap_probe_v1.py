#!/usr/bin/env python3
"""run_router_delayed_positional_bootstrap_probe_v1.py

DELAYED-INJECTION POSITIONAL BOOTSTRAP PROBE v1

Objective: Test whether a minimal trainable b_posLast scalar (positional
attention bias at t=D-1, mirroring the b_pos0 used in the baseline) is
sufficient to make the delayed-injection (inject@last) regime trainable.

LOCKED FINDINGS:
  1. inject@last breaks tau0-sufficiency (confirmed)
  2. inject@last fails to train without positional prior (acc≈0.27, confirmed)
  3. Phase/frame alignment is CONSISTENT — frame mismatch is NOT the cause
  4. Root cause: no positional prior at t=D-1 → alpha_{D-1} ≈ 1/24 throughout
     early training → W_tok_inject and W_pred cannot bootstrap each other
  5. Baseline uses b_pos0=2.0 → attention bootstraps toward t=0 from step 1

MINIMAL FIX (single scalar):
  posLast_mask: (1, D) buffer with 1 at position D-1 (mirrors pos0_mask)
  b_posLast: trainable scalar (mirrors b_pos0)
  Attention: sc += posLast_mask * b_posLast   (additive, no other change)

VARIANTS:
  no_prior          b_posLast_init=0.0  (baseline delayed, should fail)
  bLast_0.5         b_posLast_init=0.5
  bLast_1.0         b_posLast_init=1.0
  bLast_2.0         b_posLast_init=2.0  (mirrors baseline b_pos0=2.0)

All variants: inject_position='last', b_pos0_init=0.0, radial=dynamic
Training: SGD/LR=0.02, 3500 steps (same as successful baseline)

Ablations (on any variant that achieves acc≥0.999):
  full, tau0_direct, no_tau0, last_only, no_last, random_t_gt0
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_delayed_positional_bootstrap_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_delayed_positional_bootstrap_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
D             = 24
D_HIDDEN      = 32
BATCH_SIZE    = 512
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 2048
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
MAX_STEPS     = 3_500
N_BATCHES     = N_EVAL // BATCH_SIZE
NEG_INF       = -1e9

try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _st
    _st(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion + hybrid tables
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG)
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
# Router with optional b_posLast
# ═══════════════════════════════════════════════════════════════════════
class RouterDelayedBootstrap(nn.Module):
    """
    Delayed-injection router with optional b_posLast positional prior.

    Fixed: inject_position='last', b_pos0_init=0.0, radial=dynamic

    b_posLast_init controls the last-position attention bias:
      0.0 → no prior (original failing model)
      2.0 → strong prior mirroring baseline b_pos0=2.0
    """
    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 b_posLast_init: float = 0.0,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        dh = D_HIDDEN; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        # pos0_mask: bias at position 0 (kept at 0.0 for delayed regime)
        m0 = torch.zeros(1, D); m0[0, 0] = 1.0
        self.register_buffer("pos0_mask", m0)
        # posLast_mask: bias at position D-1 (the minimal positional prior)
        mL = torch.zeros(1, D); mL[0, D-1] = 1.0
        self.register_buffer("posLast_mask", mL)
        self.b_pos0    = nn.Parameter(torch.tensor(0.0))         # fixed at 0 for delayed
        self.b_posLast = nn.Parameter(torch.tensor(b_posLast_init))
        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))
        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);    self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);        self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU_HYB);  self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(D_TAU_HYB, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU_HYB)

    def _inject(self, hybrid: torch.Tensor, x0: torch.Tensor, t: int) -> torch.Tensor:
        if t == D - 1:
            return hybrid + self.W_tok_inject[x0]
        return hybrid

    def forward(self, state_ids, tokens, x0, temp):
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            tn   = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            B     = state_ids.shape[0]
            base  = torch.einsum("bi,bij->bj", w, tn)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()
            dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
            hybrid   = torch.cat([dirn, mag], dim=1)
            tau_prev = self._inject(hybrid, x0, t)
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
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
# Utilities
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng)
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids) -> Tuple[float, float, float]:
    """Returns (accuracy, mean_alpha_0, mean_alpha_{D-1})."""
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; a0_sum = 0.0; aD_sum = 0.0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            logits = model(sids, toks, x0, 0.05)
            correct += (logits.argmax(1) == x0).sum().item()
            # Recompute alpha for tracking
            tau_prev = model.tau0_table[sids]
            taus = []
            sids_l = sids.clone()
            for t in range(D):
                tn = model.TN[sids_l]
                cur_dir = tau_prev[:, :D_TAU_ANG]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(BATCH_SIZE,1,1).expand(BATCH_SIZE,1,D_TAU_ANG)
                ).squeeze(1)
                best_sim = ang_sim.gather(1, best_op.unsqueeze(1)).squeeze(1)
                radial = (best_sim + 2.0).clamp(0,4).unsqueeze(1).expand(
                    BATCH_SIZE, N_PHASE_PAIRS) / 2.0
                hybrid = torch.cat([best_ang, radial], dim=1)
                tau_cur = model._inject(hybrid, x0, t)
                taus.append(tau_cur)
                tau_prev = tau_cur
                sids_l = model.TR[sids_l].gather(1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            _, alpha = model.readout(st)
            a0_sum += alpha[:, 0].mean().item()
            aD_sum += alpha[:, D-1].mean().item()
    model.train()
    return (round(correct / N_EVAL, 4),
            round(a0_sum / N_BATCHES, 4),
            round(aD_sum / N_BATCHES, 4))


def train_model(TN_ang, TR, tau0_hyb, pool_ids,
                b_posLast_init: float, label: str):
    model = RouterDelayedBootstrap(
        TN_ang, TR, tau0_hyb, pool_ids, b_posLast_init=b_posLast_init)
    opt   = torch.optim.SGD(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter()
    solve_step = None; final_acc = 0.0; alpha0_final = 0.0; alphaD_final = 0.0

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc, a0, aD = eval_acc(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None: solve_step = step
            final_acc = acc; alpha0_final = a0; alphaD_final = aD
            print(f"    [{label}] step={step:5d}  acc={acc:.4f}  "
                  f"α₀={a0:.4f}  α_{{D-1}}={aD:.4f}")

    wall = time.perf_counter() - t0
    sps  = round(MAX_STEPS / wall, 1)
    bL   = float(model.b_posLast.item())
    print(f"  {label}: acc={final_acc:.4f}  solve={solve_step}  sps={sps}  "
          f"b_posLast={bL:.3f}  α₀={alpha0_final:.4f}  α_{{D-1}}={alphaD_final:.4f}")
    model.eval()
    return model, final_acc, solve_step, sps, round(wall, 1), alpha0_final, alphaD_final


# ═══════════════════════════════════════════════════════════════════════
# Hard geom trajectory collector (inject@last, dynamic radial)
# ═══════════════════════════════════════════════════════════════════════
def collect_hard_geom_trajectories(model, pool_ids):
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st, all_x0 = [], []
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = BATCH_SIZE
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                cur_dir = tau_prev[:, :D_TAU_ANG]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,D_TAU_ANG)).squeeze(1)
                best_sim = ang_sim.gather(1, best_op.unsqueeze(1)).squeeze(1)
                radial   = (best_sim + 2.0).clamp(0,4).unsqueeze(1).expand(
                    B, N_PHASE_PAIRS) / 2.0
                hybrid   = torch.cat([best_ang, radial], dim=1)
                tau_cur  = model._inject(hybrid, x0, t)
                taus.append(tau_cur.clone())
                tau_prev  = tau_cur
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            all_st.append(torch.stack(taus, dim=1))
            all_x0.append(x0)
    return all_st, all_x0


# ═══════════════════════════════════════════════════════════════════════
# Ablations
# ═══════════════════════════════════════════════════════════════════════
ABLATION_DEFS = [
    ("full",         "full trajectory"),
    ("tau0_direct",  "attention bypassed; pred = τ₀ @ W_pred + b_pred"),
    ("no_tau0",      "position 0 masked"),
    ("last_only",    f"only t={D-1}"),
    ("no_last",      f"t={D-1} masked"),
    ("random_t_gt0", "τ_t (t>0) replaced with random table entries; τ₀ intact"),
]

# Pre-sample random taus once (reused across ablations)
_rand_taus_cache: Optional[torch.Tensor] = None


def _get_rand_taus(tau0_hyb, pool_ids) -> torch.Tensor:
    global _rand_taus_cache
    if _rand_taus_cache is None:
        rng = torch.Generator().manual_seed(GLOBAL_SEED + 77777)
        rand_idx = torch.randint(pool_ids.shape[0], (BATCH_SIZE, D-1), generator=rng)
        _rand_taus_cache = tau0_hyb[pool_ids[rand_idx]]
    return _rand_taus_cache


def apply_ablation(model, all_st, all_x0, variant: str,
                   rand_taus: Optional[torch.Tensor] = None):
    t_start = time.perf_counter()
    correct = 0; a0_sum = 0.0; aD_sum = 0.0
    with torch.no_grad():
        for st, x0 in zip(all_st, all_x0):
            B = st.shape[0]
            if variant == "full":
                pred, alpha = model.readout(st)
            elif variant == "tau0_direct":
                pred  = st[:, 0, :] @ model.W_pred + model.b_pred
                alpha = torch.zeros(B, D); alpha[:, 0] = 1.0
            elif variant == "no_tau0":
                mask = torch.zeros(1, D); mask[0, 0] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "last_only":
                mask = torch.full((1, D), NEG_INF); mask[0, D-1] = 0.0
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "no_last":
                mask = torch.zeros(1, D); mask[0, D-1] = NEG_INF
                pred, alpha = model.readout_masked(st, mask)
            elif variant == "random_t_gt0":
                st_abl = st.clone()
                if rand_taus is not None:
                    st_abl[:, 1:, :] = rand_taus[:B, :, :]
                pred, alpha = model.readout(st_abl)
            else:
                raise ValueError(variant)
            correct   += (pred.argmax(1) == x0).sum().item()
            a0_sum    += alpha[:, 0].mean().item()
            aD_sum    += alpha[:, D-1].mean().item()
    wall = round(time.perf_counter() - t_start, 4)
    acc  = round(correct / N_EVAL, 4)
    a0   = round(a0_sum / N_BATCHES, 4)
    aD   = round(aD_sum / N_BATCHES, 4)
    return acc, a0, aD, wall


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(variant_results: List[Dict], ablation_results: List[Dict],
                    conclusion: str):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: Delayed-Injection Positional Bootstrap Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, "
             f"N_eval={N_EVAL}, steps={MAX_STEPS}, LR={LR}  \n\n")

    L.append("## Diagnosis\n\n")
    L.append("The delayed-injection (inject@last) regime fails to train because "
             "no positional prior focuses attention on τ_{D-1} during early training.  \n")
    L.append(f"Without b_posLast: α_{{D-1}} ≈ 1/{D} = {1/D:.3f} throughout early training.  \n")
    L.append("**Minimal fix:** add trainable `b_posLast` scalar with "
             "`posLast_mask` (1 at t=D-1, 0 elsewhere), mirroring `b_pos0`.  \n\n")
    L.append("**No other change to geometry, routing, or task.**  \n\n")

    L.append("---\n\n")
    L.append("## Head-to-Head Comparison\n\n")
    L.append("| variant | b_posLast_init | b_posLast_trained | accuracy | solve_step | "
             "alpha0 | alpha_{D-1} | runtime_s |\n")
    L.append("|---------|---------------|-------------------|----------|------------|"
             "--------|-------------|----------|\n")
    for r in variant_results:
        solved = str(r["solve_step"]) if r["solve_step"] else "—"
        L.append(f"| {r['label']} | {r['b_posLast_init']} | {r['b_posLast_trained']:.3f} "
                 f"| {r['accuracy']:.4f} | {solved} "
                 f"| {r['alpha0']:.4f} | {r['alphaD']:.4f} | {r['runtime_s']:.1f} |\n")
    L.append("\n")

    if ablation_results:
        L.append("---\n\n")
        L.append("## Ablation Suite (solved variants only)\n\n")
        L.append("| variant | ablation | accuracy | Δ_vs_full | alpha0 | alpha_{D-1} | "
                 "interpretation |\n")
        L.append("|---------|----------|----------|-----------|--------|-------------|"
                 "----------------|\n")
        for r in ablation_results:
            delta = round(r["accuracy"] - r["full_acc"], 4)
            sign  = "+" if delta >= 0 else ""
            L.append(f"| {r['variant']} | {r['ablation']} | {r['accuracy']:.4f} "
                     f"| {sign}{delta:.4f} | {r['alpha0']:.4f} | {r['alphaD']:.4f} "
                     f"| {r['interpretation']} |\n")
        L.append("\n")

    L.append("---\n\n")
    L.append("## Explicit Answers\n\n")

    best = max(variant_results, key=lambda r: r["accuracy"])
    any_solved = any(r["solve_step"] is not None for r in variant_results)

    L.append("**1. Does b_posLast make the delayed regime trainable?**  \n")
    if any_solved:
        s = next(r for r in variant_results if r["solve_step"] is not None)
        L.append(f"YES — {s['label']} (b_posLast_init={s['b_posLast_init']}) "
                 f"reaches acc={s['accuracy']:.4f} at step {s['solve_step']}.  \n\n")
    else:
        L.append(f"NOT YET — best acc={best['accuracy']:.4f} "
                 f"({best['label']}, b_posLast_init={best['b_posLast_init']}).  \n\n")

    L.append("**2. Does attention concentrate on the last position?**  \n")
    best_aD = max(r["alphaD"] for r in variant_results)
    best_aD_var = max(variant_results, key=lambda r: r["alphaD"])
    L.append(f"Max α_{{D-1}} achieved: {best_aD:.4f} "
             f"({best_aD_var['label']}, b_posLast_init={best_aD_var['b_posLast_init']}).  \n")
    if best_aD > 0.5:
        L.append("YES — attention is dominated by the last position.  \n\n")
    elif best_aD > 0.2:
        L.append("PARTIALLY — attention concentration is above uniform but not dominant.  \n\n")
    else:
        L.append("NO — attention remains near-uniform despite positional prior.  \n\n")

    # tau0 non-sufficiency
    tau0d_for_best = next(
        (r["accuracy"] for r in ablation_results
         if r["variant"] == best["label"] and r["ablation"] == "tau0_direct"), None)
    L.append("**3. Is tau0 still non-sufficient (injection at last preserved)?**  \n")
    if tau0d_for_best is not None:
        if tau0d_for_best < 0.4:
            L.append(f"YES — tau0_direct={tau0d_for_best:.4f}≈chance. "
                     f"τ₀ does NOT encode the answer.  \n\n")
        else:
            L.append(f"NO — tau0_direct={tau0d_for_best:.4f}. "
                     f"τ₀ has some answer content.  \n\n")
    else:
        L.append("N/A — no solved variant to ablate.  \n\n")

    # Trajectory load-bearing
    notau_acc  = next((r["accuracy"] for r in ablation_results
                       if r["variant"] == best["label"] and r["ablation"] == "no_tau0"), None)
    nolast_acc = next((r["accuracy"] for r in ablation_results
                       if r["variant"] == best["label"] and r["ablation"] == "no_last"), None)
    L.append("**4. Is the trajectory genuinely load-bearing?**  \n")
    if notau_acc is not None and nolast_acc is not None:
        if notau_acc >= 0.99 and nolast_acc < 0.5:
            L.append(f"YES — no_tau0={notau_acc:.4f} (no collapse without τ₀); "
                     f"no_last={nolast_acc:.4f} (collapses without τ_{{D-1}}). "
                     f"The routing trajectory carries the answer to t=D-1.  \n\n")
        elif notau_acc < 0.5 and nolast_acc < 0.5:
            L.append(f"PARTIALLY — both τ₀ and τ_{{D-1}} matter: "
                     f"no_tau0={notau_acc:.4f}, no_last={nolast_acc:.4f}.  \n\n")
        else:
            L.append(f"no_tau0={notau_acc:.4f}  no_last={nolast_acc:.4f}.  \n\n")
    else:
        L.append("N/A — no solved variant to ablate.  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")

    L.append("**What improved:**  \n")
    no_prior = next((r for r in variant_results if r["b_posLast_init"] == 0.0), None)
    if no_prior and best["accuracy"] > no_prior["accuracy"] + 0.1:
        L.append(f"- b_posLast raised accuracy from {no_prior['accuracy']:.4f} "
                 f"(no prior) to {best['accuracy']:.4f} ({best['label']}).  \n")
    if any_solved:
        L.append("- Task is solved. The cold-start bootstrapping hypothesis is confirmed.  \n")
    if tau0d_for_best is not None and tau0d_for_best < 0.4:
        L.append("- τ₀ remains non-sufficient (tau0_direct≈chance). "
                 "The injection frame is correct at t=D-1.  \n")
    L.append("\n")

    L.append("**What remained broken / limitations:**  \n")
    if not any_solved:
        L.append("- No variant reached solve threshold.  \n")
    L.append("- b_posLast is an architectural addition (one scalar), "
             "but it is the minimal possible change consistent with the diagnosis.  \n")
    if nolast_acc is not None and nolast_acc < 0.5 and notau_acc is not None and notau_acc < 0.9:
        L.append("- no_tau0 accuracy suggests τ₀ still plays some role. "
                 "The trajectory may not be fully load-bearing end-to-end.  \n")
    L.append("\n")

    L.append("**Next bottleneck if this still fails:**  \n")
    if not any_solved:
        L.append("- If b_posLast does not rescue training, the next hypothesis is "
                 "that 3500 steps is insufficient even with bootstrapping. "
                 "Extended training or Adam optimizer would be the next probe.  \n")
    else:
        L.append("- The system now trains with inject@last. "
                 "The next question is whether it generalises to larger D "
                 "and whether the routing trajectory carries meaningful structure "
                 "beyond just the last-position injection signal.  \n")
    L.append("\n")

    L.append("---\n\n")
    L.append(f"## DELAYED-INJECTION WITH POSITIONAL PRIOR IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


def _interpret(ablation: str, acc: float, full_acc: float) -> str:
    drop = round(acc - full_acc, 3)
    if ablation == "full":
        return "reference"
    elif ablation == "tau0_direct":
        if acc >= 0.999:   return "τ₀ encodes answer (shortcut present)"
        elif acc < 0.35:   return "τ₀ does NOT encode answer ✓"
        else:              return f"partial: {drop:+.3f}"
    elif ablation == "no_tau0":
        if acc >= 0.999:   return "trajectory sufficient without τ₀ ✓"
        elif acc < 0.35:   return "τ₀ is critical (collapse)"
        else:              return f"{drop:+.3f}"
    elif ablation == "last_only":
        if acc >= 0.999:   return "final position carries full answer ✓"
        elif acc > 0.5:    return f"partial signal at t=D-1: {drop:+.3f}"
        else:              return f"{drop:+.3f}"
    elif ablation == "no_last":
        if acc < 0.35:     return "t=D-1 critical (collapse without it) ✓"
        elif acc >= 0.999: return "t=D-1 not critical"
        else:              return f"{drop:+.3f}"
    elif ablation == "random_t_gt0":
        if acc >= 0.999:   return "t>0 taus irrelevant (only τ₀ matters)"
        elif acc < 0.5:    return "t>0 taus necessary ✓"
        else:              return f"{drop:+.3f}"
    return f"{drop:+.3f}"


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("DELAYED-INJECTION POSITIONAL BOOTSTRAP PROBE v1")
    print("=" * 60)
    print(f"Fixed: inject_position=last, b_pos0_init=0.0, radial=dynamic")
    print(f"Sweep: b_posLast_init ∈ {{0.0, 0.5, 1.0, 2.0}}, {MAX_STEPS} steps each")

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    print(f"  {data['TN_oh'].shape[0]:,} states in {time.perf_counter()-t0:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )
    rand_taus = _get_rand_taus(tau0_hyb, pool_ids)

    VARIANTS = [
        dict(label="no_prior",   b_posLast_init=0.0),
        dict(label="bLast_0.5",  b_posLast_init=0.5),
        dict(label="bLast_1.0",  b_posLast_init=1.0),
        dict(label="bLast_2.0",  b_posLast_init=2.0),
    ]

    variant_results: List[Dict] = []
    all_csv_rows:    List[Dict] = []
    solved_models:   List[Tuple] = []   # (label, model, b_posLast_init)

    for spec in VARIANTS:
        label = spec["label"]
        bL_init = spec["b_posLast_init"]
        print(f"\n{'='*60}")
        print(f"VARIANT: {label}  (b_posLast_init={bL_init})")
        print(f"{'='*60}")
        model, acc, solve_step, sps, wall, a0, aD = train_model(
            TN_ang, TR, tau0_hyb, pool_ids, bL_init, label)
        bL_trained = float(model.b_posLast.item())
        vr = dict(label=label, b_posLast_init=bL_init, b_posLast_trained=bL_trained,
                  accuracy=acc, solve_step=solve_step, sps=sps,
                  alpha0=a0, alphaD=aD, runtime_s=wall)
        variant_results.append(vr)
        all_csv_rows.append({
            "variant": label, "ablation": "training_summary",
            "b_posLast_init": bL_init, "b_posLast_trained": round(bL_trained,4),
            "steps": MAX_STEPS, "accuracy": acc, "solve_step": solve_step or "",
            "alpha0": a0, "alphaD": aD, "tau0_direct_accuracy": "",
            "runtime_seconds": wall, "note": "training result",
        })
        if solve_step is not None:
            solved_models.append((label, model, bL_init))

    # ── Ablations on solved variants ─────────────────────────────────────
    ablation_results: List[Dict] = []
    print(f"\n{'='*60}")
    print(f"ABLATIONS (solved variants: {[s[0] for s in solved_models]})")
    print(f"{'='*60}")

    # Also ablate the best unsolved variant if nothing solved
    if not solved_models:
        best_var = max(variant_results, key=lambda r: r["accuracy"])
        best_model_label = best_var["label"]
        best_bL = best_var["b_posLast_init"]
        print(f"  No solved variant — ablating best: {best_model_label}")
        # Retrain to get model object
        m, _, _, _, _, _, _ = train_model(
            TN_ang, TR, tau0_hyb, pool_ids, best_bL, best_model_label + "_reabl")
        solved_models.append((best_model_label, m, best_bL))

    for abl_label, model, bL_init in solved_models:
        print(f"\n  Collecting trajectories for {abl_label} ...")
        all_st, all_x0 = collect_hard_geom_trajectories(model, pool_ids)
        full_acc_here = None
        for variant_name, desc in ABLATION_DEFS:
            acc, a0, aD, wall = apply_ablation(
                model, all_st, all_x0, variant_name, rand_taus)
            if variant_name == "full":
                full_acc_here = acc
            delta = round(acc - (full_acc_here or acc), 4)
            sign  = "+" if delta >= 0 else ""
            interp = _interpret(variant_name, acc, full_acc_here or acc)
            print(f"    [{variant_name:<14}] acc={acc:.4f}  Δ={sign}{delta:.4f}  "
                  f"α₀={a0:.4f}  α_{{D-1}}={aD:.4f}  — {interp}")
            ablation_results.append({
                "variant": abl_label, "ablation": variant_name,
                "accuracy": acc, "full_acc": full_acc_here or acc,
                "alpha0": a0, "alphaD": aD, "interpretation": interp,
            })
            all_csv_rows.append({
                "variant": abl_label, "ablation": variant_name,
                "b_posLast_init": bL_init,
                "b_posLast_trained": round(float(model.b_posLast.item()), 4),
                "steps": MAX_STEPS, "accuracy": acc, "solve_step": "",
                "alpha0": a0, "alphaD": aD, "tau0_direct_accuracy": "",
                "runtime_seconds": wall, "note": desc,
            })

    # Fill tau0_direct_accuracy cross-reference
    for r in all_csv_rows:
        t0d = next((x["accuracy"] for x in all_csv_rows
                    if x["variant"] == r["variant"] and x["ablation"] == "tau0_direct"), "")
        r["tau0_direct_accuracy"] = t0d

    # ── Determine conclusion ─────────────────────────────────────────────
    any_solved = any(r["solve_step"] is not None for r in variant_results)
    best = max(variant_results, key=lambda r: r["accuracy"])

    if any_solved:
        conclusion = "TRAINABLE"
    elif best["accuracy"] > 0.5:
        conclusion = "PARTIALLY TRAINABLE"
    else:
        conclusion = "NOT TRAINABLE"

    print(f"\n{'='*60}")
    print(f"SUMMARY")
    for r in variant_results:
        solved_str = f"  ✓ solve@{r['solve_step']}" if r["solve_step"] else ""
        print(f"  {r['label']:<12} acc={r['accuracy']:.4f}  "
              f"α_{{D-1}}={r['alphaD']:.4f}  b_posLast={r['b_posLast_trained']:.3f}"
              f"{solved_str}")
    print(f"\nDELAYED-INJECTION WITH POSITIONAL PRIOR IS: {conclusion}")

    # ── CSV ───────────────────────────────────────────────────────────────
    fieldnames = [
        "variant", "ablation", "b_posLast_init", "b_posLast_trained", "steps",
        "accuracy", "solve_step", "alpha0", "alphaD",
        "tau0_direct_accuracy", "runtime_seconds", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(all_csv_rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(all_csv_rows)} rows)")

    _write_markdown(variant_results, ablation_results, conclusion)
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()
