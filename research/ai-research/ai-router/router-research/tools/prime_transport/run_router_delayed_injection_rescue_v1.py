#!/usr/bin/env python3
"""run_router_delayed_injection_rescue_v1.py

DELAYED-INJECTION TRAINING RESCUE PROBE v1

Objective: Determine whether inject_position="last" (restored regime) can be
made trainable without changing the architectural question.

Prior result (r4_restoration_probe_v1):
  - inject@last definitively breaks τ₀-sufficiency (tau0_direct=0.256≈chance)
  - BUT model failed to train: acc=0.266≈chance under SGD/LR=0.02/3500 steps
  - Root cause: W_tok_inject[x0] at t=D-1 receives ~1/D diluted gradients
    (attention alpha_{D-1}≈1/24 initially; no b_posLast bias)

Rescue sweep (architecture UNCHANGED):
  optimizer : SGD (required) + Adam (one bounded alternative; justified by
              gradient-magnitude normalisation resolving 1/D dilution)
  LR        : {0.02, 0.01, 0.005}  (SGD)  |  {0.001, 0.0003}  (Adam)
  steps     : {3500, 7000, 12000}   (SGD)  |  {3500, 7000}     (Adam)

Ablations per run (key subset only):
  full       — standard readout (reference)
  tau0_direct — bypass attention; pred = τ₀ @ W_pred + b_pred
  no_tau0    — mask position 0
  last_only  — mask all except t=D-1
  no_last    — mask position D-1
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_delayed_injection_rescue_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_delayed_injection_rescue_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked architectural config (unchanged from restoration probe)
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
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 2048
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
N_BATCHES     = N_EVAL // BATCH_SIZE

# Fixed restored regime toggles
INJECT_POSITION = "last"
B_POS0_INIT     = 0.0
RADIAL_MODE     = "dynamic"

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
# Router (fixed restored architecture)
# ═══════════════════════════════════════════════════════════════════════
class RouterRestored(nn.Module):
    """Fixed restored architecture: inject_position='last', b_pos0_init=0.0."""

    def __init__(self, TN_ang, TR, tau0_hyb, pool_ids,
                 init_scale: float = INIT_SCALE,
                 seed: int = GLOBAL_SEED):
        super().__init__()
        dh = D_HIDDEN; dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, D); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(B_POS0_INIT))
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
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        pred  = torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred
        return pred, alpha

    def readout_masked(self, st: torch.Tensor, attn_mask: torch.Tensor):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0 + attn_mask
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


def eval_acc(model, pool_ids) -> float:
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            correct += (model(sids, toks, x0, 0.05).argmax(1) == x0).sum().item()
    model.train()
    return correct / N_EVAL


def train_model(TN_ang, TR, tau0_hyb, pool_ids,
                max_steps: int, lr: float, optimizer_name: str,
                label: str) -> Tuple:
    model = RouterRestored(TN_ang, TR, tau0_hyb, pool_ids)
    if optimizer_name == "sgd":
        opt = torch.optim.SGD(model.parameters(), lr=lr)
    else:  # adam
        opt = torch.optim.Adam(model.parameters(), lr=lr)

    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    model.train()
    t0 = time.perf_counter()
    solve_step = None; final_acc = 0.0; first_above_chance = None

    for step in range(1, max_steps + 1):
        frac = step / max(max_steps - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_t)
        loss = F.cross_entropy(model(sids, toks, x0, temp), x0)
        opt.zero_grad(); loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()
        if step % EVAL_EVERY == 0:
            acc = eval_acc(model, pool_ids)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            if acc > 0.35 and first_above_chance is None:
                first_above_chance = (step, acc)
            final_acc = acc
            print(f"    [{label}] step={step:6d}  acc={acc:.4f}")

    wall = time.perf_counter() - t0
    sps  = round(max_steps / wall, 1)
    print(f"  {label}: acc={final_acc:.4f}  solve={solve_step}  "
          f"first_above_chance={first_above_chance}  sps={sps}  "
          f"b_pos0={model.b_pos0.item():.3f}")
    model.eval()
    return model, final_acc, solve_step, sps, round(wall, 1), first_above_chance


# ═══════════════════════════════════════════════════════════════════════
# Hard geom trajectory collection (dynamic radial — restored mode)
# ═══════════════════════════════════════════════════════════════════════
def collect_hard_geom_trajectories(model, pool_ids) -> Tuple[List, List]:
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    all_st: List[torch.Tensor] = []
    all_x0: List[torch.Tensor] = []

    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = BATCH_SIZE
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []

            for t in range(D):
                tn      = model.TN[sids_loop]                          # (B,6,8)
                cur_dir = tau_prev[:, :D_TAU_ANG]                      # (B,8)
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)      # (B,6)
                best_op = ang_sim.argmax(dim=1)                        # (B,)
                best_ang = tn.gather(
                    1, best_op.view(B,1,1).expand(B,1,D_TAU_ANG)
                ).squeeze(1)                                            # (B,8)
                best_sim = ang_sim.gather(1, best_op.unsqueeze(1)).squeeze(1)
                radial   = (best_sim + 2.0).clamp(0.0, 4.0).unsqueeze(1).expand(
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
NEG_INF = -1e9

ABLATION_DEFS = [
    ("full",        "none — full trajectory"),
    ("tau0_direct", "attention bypassed; pred = τ₀ @ W_pred + b_pred"),
    ("no_tau0",     "position 0 masked (alpha[0]→0)"),
    ("last_only",   f"all positions except t={D-1} masked"),
    ("no_last",     f"position t={D-1} masked"),
]


def apply_ablation(model, all_st, all_x0, variant: str) -> Tuple[float, float, float, float]:
    t_start = time.perf_counter()
    correct = 0; alpha0_sum = 0.0; alphaD_sum = 0.0

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
            else:
                raise ValueError(variant)

            correct   += (pred.argmax(1) == x0).sum().item()
            alpha0_sum += alpha[:, 0].mean().item()
            alphaD_sum += alpha[:, D-1].mean().item()

    wall = round(time.perf_counter() - t_start, 4)
    acc  = round(correct / N_EVAL, 4)
    a0   = round(alpha0_sum / N_BATCHES, 4)
    aD   = round(alphaD_sum / N_BATCHES, 4)
    return acc, a0, aD, wall


# ═══════════════════════════════════════════════════════════════════════
# Sweep definition
# ═══════════════════════════════════════════════════════════════════════
SGD_RUNS = [
    dict(optimizer="sgd",  lr=0.02,   max_steps=3_500),
    dict(optimizer="sgd",  lr=0.02,   max_steps=7_000),
    dict(optimizer="sgd",  lr=0.02,   max_steps=12_000),
    dict(optimizer="sgd",  lr=0.01,   max_steps=3_500),
    dict(optimizer="sgd",  lr=0.01,   max_steps=7_000),
    dict(optimizer="sgd",  lr=0.01,   max_steps=12_000),
    dict(optimizer="sgd",  lr=0.005,  max_steps=3_500),
    dict(optimizer="sgd",  lr=0.005,  max_steps=7_000),
    dict(optimizer="sgd",  lr=0.005,  max_steps=12_000),
]
ADAM_RUNS = [
    # Adam justified: gradient-magnitude normalisation resolves 1/D dilution
    # at D=24 the effective gradient to W_tok_inject is scaled by alpha_{D-1}≈1/24
    # Adam updates by sign, not magnitude, bypassing this dilution
    dict(optimizer="adam", lr=0.001,  max_steps=3_500),
    dict(optimizer="adam", lr=0.001,  max_steps=7_000),
    dict(optimizer="adam", lr=0.0003, max_steps=3_500),
    dict(optimizer="adam", lr=0.0003, max_steps=7_000),
]
ALL_RUNS = SGD_RUNS + ADAM_RUNS


# ═══════════════════════════════════════════════════════════════════════
# Markdown writer
# ═══════════════════════════════════════════════════════════════════════
def _write_markdown(all_rows: List[Dict], conclusion: str):
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L: List[str] = []
    L.append("# Prime Transport Router: Delayed-Injection Training Rescue Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B_train={BATCH_SIZE}, N_eval={N_EVAL}  \n\n")

    L.append("## Context\n\n")
    L.append("Prior probe (r4_restoration_probe_v1) established:\n\n")
    L.append("- `inject_position='last'` breaks τ₀-sufficiency (tau0_direct=0.256≈chance)  \n")
    L.append("- BUT model failed to train: acc=0.266≈chance  \n")
    L.append("- Root cause: W_tok_inject at t=D-1 receives ~α_{D-1}≈1/24 diluted gradients;  \n")
    L.append("  no b_posLast bias to bootstrap attention toward position D-1  \n\n")
    L.append("**Architecture unchanged.** Only optimizer/LR/steps are varied.  \n\n")
    L.append("**Adam justification:** Adam normalises gradient magnitudes, "
             "resolving the 1/D dilution without any model change.  \n\n")

    L.append("---\n\n")
    L.append("## Sweep Results\n\n")
    L.append("| variant | optimizer | lr | steps | accuracy | solve_step | "
             "tau0_direct | last_only | alpha0 | alphaD | runtime_s | note |\n")
    L.append("|---------|-----------|-----|-------|----------|------------|"
             "-------------|-----------|--------|--------|-----------|------|\n")
    for r in all_rows:
        if r["ablation"] != "full":
            continue
        # Find tau0_direct and last_only for same variant
        t0d = next((x for x in all_rows if x["variant"] == r["variant"]
                    and x["ablation"] == "tau0_direct"), {})
        lo  = next((x for x in all_rows if x["variant"] == r["variant"]
                    and x["ablation"] == "last_only"), {})
        above = "✓" if r["accuracy"] > 0.35 else ""
        solved = f"{r['solve_step']}" if r["solve_step"] else "—"
        L.append(f"| {r['variant']} | {r['optimizer']} | {r['learning_rate']} "
                 f"| {r['steps']} | {r['accuracy']:.4f}{above} | {solved} "
                 f"| {t0d.get('accuracy', 0):.4f} | {lo.get('accuracy', 0):.4f} "
                 f"| {r['alpha0']:.4f} | {r['alphaD']:.4f} "
                 f"| {r['runtime_seconds']:.1f} | {r['note']} |\n")

    L.append("\n---\n\n")
    L.append("## Explicit Answers\n\n")

    # Find best run
    full_rows = [r for r in all_rows if r["ablation"] == "full"]
    best = max(full_rows, key=lambda r: r["accuracy"])
    best_t0d = next((r for r in all_rows if r["variant"] == best["variant"]
                     and r["ablation"] == "tau0_direct"), {})
    best_lo  = next((r for r in all_rows if r["variant"] == best["variant"]
                     and r["ablation"] == "last_only"), {})
    best_nl  = next((r for r in all_rows if r["variant"] == best["variant"]
                     and r["ablation"] == "no_last"), {})

    any_trained = any(r["accuracy"] > 0.35 for r in full_rows)
    any_solved  = any(r["solve_step"] is not None for r in full_rows)
    first_above = min((r for r in full_rows if r["accuracy"] > 0.35),
                      key=lambda r: r["steps"], default=None)

    L.append("**1. Can delayed injection learn at all?**  \n")
    if any_trained:
        L.append(f"YES — best accuracy={best['accuracy']:.4f} "
                 f"({best['optimizer']}, lr={best['learning_rate']}, steps={best['steps']})  \n\n")
    else:
        L.append("NO — all runs remain at chance level (acc≤0.35).  \n\n")

    L.append("**2. At what training budget does it first rise meaningfully above chance?**  \n")
    if first_above:
        L.append(f"Steps={first_above['steps']}, optimizer={first_above['optimizer']}, "
                 f"lr={first_above['learning_rate']}, acc={first_above['accuracy']:.4f}  \n\n")
    else:
        L.append("Never — no run exceeded 0.35 accuracy.  \n\n")

    L.append("**3. If it learns, does τ₀ remain non-sufficient?**  \n")
    if any_trained and best_t0d:
        if best_t0d.get("accuracy", 1) < 0.4:
            L.append(f"YES — tau0_direct={best_t0d['accuracy']:.4f}≈chance on best run. "
                     f"τ₀ remains non-sufficient. Restoration is intact.  \n\n")
        else:
            L.append(f"NO — tau0_direct={best_t0d['accuracy']:.4f} on best run. "
                     f"τ₀ has recovered some answer signal.  \n\n")
    else:
        L.append("N/A — no run learned the task.  \n\n")

    L.append("**4. Does any evidence appear that trajectory becomes load-bearing?**  \n")
    if any_trained:
        lo_acc  = best_lo.get("accuracy", 0)
        nl_acc  = best_nl.get("accuracy", 0)
        t0d_acc = best_t0d.get("accuracy", 1)
        if t0d_acc < 0.4 and lo_acc > 0.5 and nl_acc < 0.5:
            L.append(f"YES — last_only={lo_acc:.4f}, no_last={nl_acc:.4f}, "
                     f"tau0_direct={t0d_acc:.4f}. "
                     f"The routing trajectory is load-bearing: position D-1 carries the answer.  \n\n")
        elif t0d_acc < 0.4 and lo_acc > 0.35:
            L.append(f"PARTIALLY — last_only={lo_acc:.4f}, tau0_direct={t0d_acc:.4f}. "
                     f"Some trajectory structure is emerging.  \n\n")
        else:
            L.append(f"NO CLEAR EVIDENCE — last_only={lo_acc:.4f}, "
                     f"tau0_direct={t0d_acc:.4f}.  \n\n")
    else:
        L.append("NO — no run learned the task.  \n\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")

    L.append("**What improved:**  \n")
    if any_trained:
        L.append(f"- Best run ({best['optimizer']}, lr={best['learning_rate']}, "
                 f"steps={best['steps']}) achieved acc={best['accuracy']:.4f}.  \n")
    if any_solved:
        s = next(r for r in full_rows if r["solve_step"] is not None)
        L.append(f"- Task solved (acc≥0.999) by {s['optimizer']}, lr={s['learning_rate']}, "
                 f"steps={s['steps']}.  \n")
    if not any_trained:
        L.append("- Nothing improved — all runs remain at chance.  \n")
    L.append("\n")

    L.append("**What remained broken:**  \n")
    if not any_solved:
        L.append("- No run reached solve threshold (acc≥0.999).  \n")
    if not any_trained:
        L.append("- No run exceeded 0.35 accuracy.  \n")
    L.append("- Architecture is unchanged; the 1/D gradient dilution persists for SGD without a positional prior at D-1.  \n")
    L.append("\n")

    L.append("**Whether the next issue is still training or truly architectural:**  \n")
    if any_solved:
        L.append("Training issue resolved — Adam finds the solution. "
                 "The delayed-injection architecture is viable with adaptive optimizers.  \n\n")
    elif any_trained:
        L.append("Mixed — partial learning observed. Further steps or adaptive LR scheduling "
                 "may bridge the gap. The architecture is not fundamentally broken.  \n\n")
    else:
        L.append("Possibly architectural — no optimizer/LR/step combination in this sweep "
                 "produced meaningful learning. Consider positional bias toward D-1 or "
                 "curriculum injection starting from an intermediate position.  \n\n")

    L.append("---\n\n")
    L.append(f"## DELAYED-INJECTION REGIME IS: {conclusion}\n\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 60)
    print("DELAYED-INJECTION TRAINING RESCUE PROBE v1")
    print("=" * 60)
    print(f"Architecture: inject_position={INJECT_POSITION}  "
          f"b_pos0_init={B_POS0_INIT}  radial={RADIAL_MODE}")
    print(f"Sweep: {len(ALL_RUNS)} runs "
          f"(SGD×LR×steps + Adam×LR×steps)")

    print("\nLoading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    print(f"  {data['TN_oh'].shape[0]:,} states in {time.perf_counter()-t0:.3f}s")
    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )

    all_csv_rows: List[Dict] = []
    total = len(ALL_RUNS)

    for i, spec in enumerate(ALL_RUNS):
        opt_name  = spec["optimizer"]
        lr        = spec["lr"]
        max_steps = spec["max_steps"]
        label     = f"{opt_name}_lr{lr}_s{max_steps}"

        print(f"\n{'='*60}")
        print(f"RUN {i+1}/{total}: {label}")
        print(f"{'='*60}")

        model, train_acc, solve_step, sps, wall, first_above = train_model(
            TN_ang, TR, tau0_hyb, pool_ids, max_steps, lr, opt_name, label
        )
        b_pos0_val = float(model.b_pos0.item())

        print(f"  Collecting hard geom trajectories ...")
        all_st, all_x0 = collect_hard_geom_trajectories(model, pool_ids)

        print(f"  Applying ablations ...")
        for variant, desc in ABLATION_DEFS:
            acc, a0, aD, abl_wall = apply_ablation(model, all_st, all_x0, variant)
            print(f"    [{variant:<12}] acc={acc:.4f}  α₀={a0:.4f}  α_{{D-1}}={aD:.4f}")
            first_above_str = f"{first_above[0]}@{first_above[1]:.4f}" if first_above else "never"
            all_csv_rows.append({
                "variant":             label,
                "ablation":            variant,
                "steps":               max_steps,
                "learning_rate":       lr,
                "optimizer":           opt_name,
                "accuracy":            acc,
                "solve_step":          solve_step,
                "tau0_direct_accuracy": "",   # filled below
                "alpha0":              a0,
                "alphaD":              aD,
                "b_pos0_trained":      round(b_pos0_val, 4),
                "runtime_seconds":     wall,
                "first_above_chance":  first_above_str,
                "ablated_component":   desc,
                "note":                ("" if acc > 0.35 else "chance"),
            })

    # Fill tau0_direct_accuracy column (cross-reference)
    for r in all_csv_rows:
        t0d = next((x["accuracy"] for x in all_csv_rows
                    if x["variant"] == r["variant"] and x["ablation"] == "tau0_direct"), "")
        r["tau0_direct_accuracy"] = t0d

    # ── Determine conclusion ─────────────────────────────────────────────
    full_rows  = [r for r in all_csv_rows if r["ablation"] == "full"]
    any_solved = any(r["solve_step"] is not None for r in full_rows)
    best_acc   = max(r["accuracy"] for r in full_rows)
    any_trained = best_acc > 0.35

    if any_solved:
        conclusion = "TRAINABLE"
    elif any_trained:
        conclusion = "PARTIALLY TRAINABLE"
    else:
        conclusion = "NOT TRAINABLE"

    print(f"\n{'='*60}")
    print(f"SWEEP SUMMARY")
    print(f"  best accuracy:  {best_acc:.4f}")
    print(f"  any solved:     {any_solved}")
    print(f"  any >chance:    {any_trained}")
    print(f"\nDELAYED-INJECTION REGIME IS: {conclusion}")

    # ── CSV ───────────────────────────────────────────────────────────────
    fieldnames = [
        "variant", "ablation", "steps", "learning_rate", "optimizer",
        "accuracy", "solve_step", "tau0_direct_accuracy",
        "alpha0", "alphaD", "b_pos0_trained",
        "runtime_seconds", "first_above_chance",
        "ablated_component", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(all_csv_rows)
    print(f"\nCSV written: {CSV_OUT}  ({len(all_csv_rows)} rows)")

    _write_markdown(all_csv_rows, conclusion)
    print(f"Markdown written: {MD_OUT}")


if __name__ == "__main__":
    main()
