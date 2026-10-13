#!/usr/bin/env python3
"""run_router_circular_mean_mixture_v1.py

Circular-Mean Mixture Alignment experiment: direct test of mismatch M2.

The soft operator mixture currently performs a Euclidean weighted average
of angular (cos theta, sin theta) pairs.  The result lies INSIDE the
unit circle, not ON it, because |sum w_i u_i| <= 1 for unit vectors u_i.

This experiment adds per-phase-pair renormalization after each mixture
step, projecting back to S^1 -- implementing a proper circular mean.

Three-way comparison:
  BASELINE_ONEHOT  -- one-hot tau (D_TAU=21), Euclidean mixture
  ANGULAR_EUC      -- angular tau (D_TAU=8),  Euclidean mixture
  ANGULAR_CIRC     -- angular tau (D_TAU=8),  circular-mean mixture

Same operators, task, D=24, training setup, seeds, temperature schedule.
"""
from __future__ import annotations

import csv
import importlib.util
import math
import os
import time
from pathlib import Path
from typing import Dict, List, Tuple

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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_circular_mean_mixture_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_circular_mean_mixture_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked hyperparameters (canonical v6 baseline)
# ═══════════════════════════════════════════════════════════════════════
VOCAB        = 4
D_EMB        = 4
N_OPS        = 6
LR           = 0.02
TEMP_START   = 2.0
TEMP_END     = 0.1
GLOBAL_SEED  = 42
B0_INIT      = 2.0
D_CONTEXT    = 24
BATCH_SIZE   = 256
D_HIDDEN     = 32
BENCH_BUDGET = 3000
N_EVAL       = 1000

# Encoding dimensions
D_TAU_OH  = 21;  D_IN_OH  = D_EMB + D_TAU_OH    # 25
D_TAU_ANG = 8;   D_IN_ANG = D_EMB + D_TAU_ANG   # 12

# Phase block structure: (start_onehot, end_onehot, modulus)
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
N_PHASE_PAIRS = 4
_TRANSPORT_TH = 3

CHECKPOINTS = [0, 250, 500, 1000, 1500, 2000, 2500, 3000]

torch.set_num_threads(1)

# ═══════════════════════════════════════════════════════════════════════
# Load v6 torch base module
# ═══════════════════════════════════════════════════════════════════════
def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch_base",
        str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"),
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ═══════════════════════════════════════════════════════════════════════
# Angular encoding conversion
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    """Convert (..., 21) one-hot tau -> (..., 8) angular (cos/sin pairs)."""
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


# ═══════════════════════════════════════════════════════════════════════
# Circular mean helper: per-phase-pair normalization to unit circle
# ═══════════════════════════════════════════════════════════════════════
def _circ_norm(tau: torch.Tensor) -> torch.Tensor:
    """Project each (cos, sin) pair to the unit circle independently.

    Standard circular mean: direction of the resultant vector.
    tau shape: (B, 8)  ->  view as (B, 4, 2)  ->  normalize  ->  (B, 8)
    """
    B = tau.shape[0]
    pairs = tau.view(B, N_PHASE_PAIRS, 2)
    norms = (pairs * pairs).sum(dim=2, keepdim=True).sqrt().clamp(min=1e-8)
    return (pairs / norms).view(B, D_TAU_ANG)


# ═══════════════════════════════════════════════════════════════════════
# RouterBaseline -- one-hot tau, Euclidean mixture
# ═══════════════════════════════════════════════════════════════════════
class RouterBaseline(nn.Module):
    """One-hot tau baseline: D_TAU=21, D_IN=25, Euclidean einsum mixture."""

    def __init__(
        self,
        TN: torch.Tensor,
        TR: torch.Tensor,
        tau0: torch.Tensor,
        pool: torch.Tensor,
        d_hidden: int = D_HIDDEN,
        d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT,
        sc: float = 0.05,
        seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh  = d_hidden
        dha = max(8, dh // 4)
        d_tau = D_TAU_OH
        d_in  = D_IN_OH

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context)
        m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)

        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            embs     = self.W_emb[tokens[:, t]]
            h_in     = torch.cat([embs, tau_prev], dim=1)
            h        = torch.tanh(h_in @ self.W1 + self.b1)
            logits   = h @ self.W2 + self.b2

            if self.training:
                u  = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gn = -torch.log(-torch.log(u))
                w  = torch.softmax((logits + gn) / temp, dim=1)
            else:
                w  = torch.softmax(logits / 0.05, dim=1)

            base     = torch.einsum("bi,bij->bj", w, tn_batch)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)

            k_hard    = torch.argmax(w, dim=1)
            tr_rows   = self.TR[state_ids]
            state_ids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

        soft_taus_stack = torch.stack(soft_taus, dim=1)
        h_attn       = torch.tanh(
            soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        return pooled @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# RouterAngularEuc -- angular tau, Euclidean mixture (no normalization)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularEuc(nn.Module):
    """Angular (cos/sin) tau with standard Euclidean einsum mixture."""

    def __init__(
        self,
        TN: torch.Tensor,
        TR: torch.Tensor,
        tau0: torch.Tensor,
        pool: torch.Tensor,
        d_hidden: int = D_HIDDEN,
        d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT,
        sc: float = 0.05,
        seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh  = d_hidden
        dha = max(8, dh // 4)
        d_tau = D_TAU_ANG
        d_in  = D_IN_ANG

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context)
        m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)

        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            embs     = self.W_emb[tokens[:, t]]
            h_in     = torch.cat([embs, tau_prev], dim=1)
            h        = torch.tanh(h_in @ self.W1 + self.b1)
            logits   = h @ self.W2 + self.b2

            if self.training:
                u  = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gn = -torch.log(-torch.log(u))
                w  = torch.softmax((logits + gn) / temp, dim=1)
            else:
                w  = torch.softmax(logits / 0.05, dim=1)

            base     = torch.einsum("bi,bij->bj", w, tn_batch)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)

            k_hard    = torch.argmax(w, dim=1)
            tr_rows   = self.TR[state_ids]
            state_ids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

        soft_taus_stack = torch.stack(soft_taus, dim=1)
        h_attn       = torch.tanh(
            soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        return pooled @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# RouterAngularCirc -- angular tau, CIRCULAR-MEAN mixture
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularCirc(nn.Module):
    """Angular (cos/sin) tau with per-phase-pair circular mean normalization.

    After each einsum mixture and after step-0 injection, each (cos, sin)
    pair is renormalized to unit length.  This projects the mixture result
    back to S^1, implementing a proper circular mean for each phase factor.
    """

    def __init__(
        self,
        TN: torch.Tensor,
        TR: torch.Tensor,
        tau0: torch.Tensor,
        pool: torch.Tensor,
        d_hidden: int = D_HIDDEN,
        d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT,
        sc: float = 0.05,
        seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh  = d_hidden
        dha = max(8, dh // 4)
        d_tau = D_TAU_ANG
        d_in  = D_IN_ANG

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context)
        m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)

        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            embs     = self.W_emb[tokens[:, t]]
            h_in     = torch.cat([embs, tau_prev], dim=1)
            h        = torch.tanh(h_in @ self.W1 + self.b1)
            logits   = h @ self.W2 + self.b2

            if self.training:
                u  = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gn = -torch.log(-torch.log(u))
                w  = torch.softmax((logits + gn) / temp, dim=1)
            else:
                w  = torch.softmax(logits / 0.05, dim=1)

            base = torch.einsum("bi,bij->bj", w, tn_batch)

            # ── Circular mean: project each phase pair back to S¹ ──
            _p = base.view(B, 4, 2)
            _n = (_p * _p).sum(dim=2, keepdim=True).sqrt().clamp(min=1e-8)
            base = (_p / _n).view(B, 8)

            if t == 0:
                tau_prev = base + self.W_tok_inject[x0]
                # Re-normalize after injection to stay on torus
                _p2 = tau_prev.view(B, 4, 2)
                _n2 = (_p2 * _p2).sum(dim=2, keepdim=True).sqrt().clamp(min=1e-8)
                tau_prev = (_p2 / _n2).view(B, 8)
            else:
                tau_prev = base

            soft_taus.append(tau_prev)

            k_hard    = torch.argmax(w, dim=1)
            tr_rows   = self.TR[state_ids]
            state_ids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

        soft_taus_stack = torch.stack(soft_taus, dim=1)
        # Final pooling is NOT circularly normalized -- goes to prediction head
        h_attn       = torch.tanh(
            soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        return pooled @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Sampling
# ═══════════════════════════════════════════════════════════════════════
def sample_batch(
    pool_ids: torch.Tensor, D: int, B: int, device: str,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx       = torch.randint(0, len(pool_ids), (B,), device=device)
    state_ids = pool_ids[idx]
    tokens    = torch.randint(0, VOCAB, (B, D), device=device)
    x0        = torch.randint(0, VOCAB, (B,),   device=device)
    tokens[:, 0] = x0
    return state_ids, tokens, x0


# ═══════════════════════════════════════════════════════════════════════
# Evaluation (handles all 3 variants via circular_mean flag)
# ═══════════════════════════════════════════════════════════════════════
def evaluate(
    model: nn.Module,
    pool_ids: torch.Tensor,
    D: int,
    n_eval: int,
    device: str,
    circular_mean: bool = False,
) -> Dict[str, float]:
    model.eval()
    torch.manual_seed(GLOBAL_SEED + D + 13)

    d_tau       = model.tau0_table.shape[1]
    B_eval      = 256
    n_batches   = max(1, min(4, (n_eval + B_eval - 1) // B_eval))
    total       = 0
    correct     = 0
    ent_sum     = 0.0
    transport_n = 0
    total_steps = 0
    alpha_sum   = torch.zeros(D, device=device)

    with torch.no_grad():
        for _ in range(n_batches):
            B_ = min(B_eval, n_eval - total)
            if B_ <= 0:
                break
            idx       = torch.randint(0, len(pool_ids), (B_,), device=device)
            state_ids = pool_ids[idx]
            tokens    = torch.randint(0, VOCAB, (B_, D), device=device)
            x0        = torch.randint(0, VOCAB, (B_,),   device=device)
            tokens[:, 0] = x0

            tau_prev  = model.tau0_table[state_ids]
            soft_taus: List[torch.Tensor] = []
            sids      = state_ids.clone()

            for t in range(D):
                tn_batch = model.TN[sids]
                embs     = model.W_emb[tokens[:, t]]
                h_in     = torch.cat([embs, tau_prev], dim=1)
                h        = torch.tanh(h_in @ model.W1 + model.b1)
                logits   = h @ model.W2 + model.b2
                w        = torch.softmax(logits / 0.05, dim=1)

                pc       = w.clamp(1e-12, 1.0)
                ent_sum += float((-(pc * pc.log()).sum(dim=1)).sum().item())

                k_hard       = w.argmax(dim=1)
                transport_n += int((k_hard >= _TRANSPORT_TH).sum().item())
                total_steps += B_

                base = torch.einsum("bi,bij->bj", w, tn_batch)

                if circular_mean:
                    base = _circ_norm(base)

                if t == 0:
                    tau_prev = base + model.W_tok_inject[x0]
                    if circular_mean:
                        tau_prev = _circ_norm(tau_prev)
                else:
                    tau_prev = base

                soft_taus.append(tau_prev)
                tr_rows  = model.TR[sids]
                sids     = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

            soft_taus_stack = torch.stack(soft_taus, dim=1)
            h_attn   = torch.tanh(
                soft_taus_stack @ model.W_attn.t() + model.b_attn)
            a_raw    = (h_attn * model.v_attn).sum(dim=-1)
            a_scores = a_raw + model.pos0_mask * model.b_pos0
            alpha    = torch.softmax(a_scores, dim=1)
            alpha_sum += alpha.sum(dim=0)

            pred    = (torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
                       @ model.W_pred + model.b_pred)
            correct += int((pred.argmax(1) == x0).sum().item())
            total   += B_

    model.train()
    return {
        "accuracy":       correct / max(total, 1),
        "route_entropy":  ent_sum / max(total_steps, 1),
        "transport_frac": transport_n / max(total_steps, 1),
        "alpha0":         float((alpha_sum[0] / max(total, 1)).item()),
        "b_pos0":         float(model.b_pos0.item()),
    }


# ═══════════════════════════════════════════════════════════════════════
# Training + checkpoint evaluation
# ═══════════════════════════════════════════════════════════════════════
def train_and_evaluate(
    variant_name: str,
    model: nn.Module,
    pool_ids: torch.Tensor,
    device: str,
    circular_mean: bool = False,
) -> Tuple[List[dict], float]:
    """Train for BENCH_BUDGET steps, evaluating at each checkpoint."""
    try:
        scripted = torch.jit.script(model)
        jit_ok = True
    except Exception as e:
        print(f"    JIT failed ({e}), using eager", flush=True)
        scripted = model
        jit_ok = False

    scripted.train()
    optimizer = torch.optim.SGD(scripted.parameters(), lr=LR)
    torch.manual_seed(GLOBAL_SEED)

    n_params = sum(p.numel() for p in model.parameters())
    d_tau    = model.tau0_table.shape[1]
    d_in     = D_EMB + d_tau

    ckpt_set   = set(CHECKPOINTS)
    rows: List[dict] = []

    # Step 0 evaluation
    if 0 in ckpt_set:
        sd = {k: v for k, v in scripted.state_dict().items()
              if k not in {"TN", "TR", "tau0_table", "pool_ids", "pos0_mask"}}
        model.load_state_dict(sd, strict=False)
        metrics = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device,
                           circular_mean=circular_mean)
        rows.append({
            "variant": variant_name,
            "checkpoint": 0,
            "accuracy": round(metrics["accuracy"], 4),
            "alpha0": round(metrics["alpha0"], 4),
            "route_entropy": round(metrics["route_entropy"], 4),
            "transport_frac": round(metrics["transport_frac"], 4),
            "b_pos0": round(metrics["b_pos0"], 3),
            "runtime_seconds": 0.0,
            "loss": -1.0,
            "d_tau": d_tau,
            "d_in": d_in,
            "n_params": n_params,
            "jit": "yes" if jit_ok else "no",
            "note": "pre-training",
        })
        scripted.train()

    loss_val = -1.0
    t0 = time.perf_counter()

    for step in range(BENCH_BUDGET):
        frac = step / max(BENCH_BUDGET - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)

        sids, toks, x0 = sample_batch(pool_ids, D_CONTEXT, BATCH_SIZE, device)
        pred = scripted(sids, toks, x0, temp)
        loss = F.cross_entropy(pred, x0)
        loss_val = float(loss.item())

        optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(scripted.parameters(), max_norm=1.0)
        optimizer.step()

        step_num = step + 1
        if step_num in ckpt_set:
            elapsed = time.perf_counter() - t0
            sd = {k: v for k, v in scripted.state_dict().items()
                  if k not in {"TN", "TR", "tau0_table", "pool_ids", "pos0_mask"}}
            model.load_state_dict(sd, strict=False)
            metrics = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device,
                               circular_mean=circular_mean)
            solved = "solved" if metrics["accuracy"] >= 0.90 else ""
            rows.append({
                "variant": variant_name,
                "checkpoint": step_num,
                "accuracy": round(metrics["accuracy"], 4),
                "alpha0": round(metrics["alpha0"], 4),
                "route_entropy": round(metrics["route_entropy"], 4),
                "transport_frac": round(metrics["transport_frac"], 4),
                "b_pos0": round(metrics["b_pos0"], 3),
                "runtime_seconds": round(elapsed, 2),
                "loss": round(loss_val, 4),
                "d_tau": d_tau,
                "d_in": d_in,
                "n_params": n_params,
                "jit": "yes" if jit_ok else "no",
                "note": solved,
            })
            print(f"    step {step_num:>5}: acc={metrics['accuracy']:.4f}  "
                  f"alpha0={metrics['alpha0']:.4f}  "
                  f"loss={loss_val:.4f}  "
                  f"ent={metrics['route_entropy']:.4f}  "
                  f"t={elapsed:.1f}s  {solved}", flush=True)
            scripted.train()

    runtime = time.perf_counter() - t0
    return rows, runtime


# ═══════════════════════════════════════════════════════════════════════
# Markdown report writer
# ═══════════════════════════════════════════════════════════════════════
def write_md(
    bl_rows: List[dict],
    ae_rows: List[dict],
    ac_rows: List[dict],
    rt_bl: float,
    rt_ae: float,
    rt_ac: float,
) -> None:
    L: List[str] = []
    A = L.append

    A("# Circular-Mean Mixture — Experiment v1")
    A("")
    A("## Purpose")
    A("")
    A("Direct test of Training-Rule Alignment Analysis v1, mismatch M2:")
    A("the soft operator mixture performs a **Euclidean weighted average** of")
    A("angular (cos θ, sin θ) pairs. The result lies inside the unit circle,")
    A("not on it, because |Σ wᵢ uᵢ| ≤ 1 for unit vectors uᵢ.")
    A("This discards the circular geometry of each phase factor.")
    A("")

    A("## Circular-Mean Mixture Definition")
    A("")
    A("For each cyclic phase factor Z/m represented as (cos θ, sin θ):")
    A("")
    A("**Euclidean mixture** (current baseline):")
    A("```")
    A("τ̃ = Σᵢ wᵢ · (cos θᵢ, sin θᵢ)")
    A("```")
    A("Result: |τ̃| < 1 when weights are spread across operators.")
    A("Direction encodes the angular mean, but magnitude mixes with")
    A("concentration information.  Not on the torus.")
    A("")
    A("**Circular mean** (this experiment):")
    A("```")
    A("τ̃ = Σᵢ wᵢ · (cos θᵢ, sin θᵢ)")
    A("τ  = τ̃ / |τ̃|     (per-phase-pair, independently)")
    A("```")
    A("Result: |τ| = 1 always.  Each phase pair lives on S¹.")
    A("This is the standard circular mean (direction of the resultant vector).")
    A("")

    A("## Per-Phase-Pair Renormalization")
    A("")
    A("Normalization applied independently to each of the 4 phase pairs:")
    A("")
    A("| Pair | Phase Factor | Modulus | After Normalization |")
    A("|------|-------------|---------|---------------------|")
    A("| 0 | swap_phase | Z/2 | ‖(cos,sin)‖ = 1 |")
    A("| 1 | coupled_phase | Z/5 | ‖(cos,sin)‖ = 1 |")
    A("| 2 | twist_phase | Z/2 | ‖(cos,sin)‖ = 1 |")
    A("| 3 | lift_phase | Z/12 | ‖(cos,sin)‖ = 1 |")
    A("")
    A("Applied after:")
    A("1. The einsum mixture of TN entries (every recurrence step)")
    A("2. The step-0 token injection (keeps injected tau on torus)")
    A("")
    A("**NOT** applied to final attention pooling (goes to prediction head,")
    A("not fed back into recurrence).")
    A("")
    A("Implementation: `tau.view(B, 4, 2)` → normalize each pair → `view(B, 8)`")
    A("Differentiable: ∂(x/‖x‖)/∂x = (I - x̂x̂ᵀ)/‖x‖")
    A("")

    A("## Locked Configuration")
    A("")
    A("| Item | Value |")
    A("|------|-------|")
    A(f"| device | cpu |")
    A(f"| D_HIDDEN | {D_HIDDEN} |")
    A(f"| batch_size | {BATCH_SIZE} |")
    A(f"| D (delay) | {D_CONTEXT} |")
    A(f"| training budget | {BENCH_BUDGET} steps |")
    A(f"| optimizer | SGD, lr={LR} |")
    A(f"| temperature | {TEMP_START} → {TEMP_END} (exponential) |")
    A(f"| grad clip | 1.0 |")
    A(f"| pos0 bias init | {B0_INIT} |")
    A(f"| seed | {GLOBAL_SEED} |")
    A("")

    A("## Three Variants")
    A("")
    A("| Variant | Tau Encoding | D_TAU | D_IN | Mixture | Params |")
    A("|---------|-------------|-------|------|---------|--------|")
    bl_np = bl_rows[-1].get("n_params", "?") if bl_rows else "?"
    ae_np = ae_rows[-1].get("n_params", "?") if ae_rows else "?"
    ac_np = ac_rows[-1].get("n_params", "?") if ac_rows else "?"
    A(f"| baseline_onehot | one-hot | {D_TAU_OH} | {D_IN_OH} | Euclidean | {bl_np} |")
    A(f"| angular_euclidean | cos/sin | {D_TAU_ANG} | {D_IN_ANG} | Euclidean | {ae_np} |")
    A(f"| angular_circular | cos/sin | {D_TAU_ANG} | {D_IN_ANG} | Circular mean | {ac_np} |")
    A("")

    # Convergence table
    A("## Convergence Comparison")
    A("")
    A("| Step | BL Acc | AE Acc | AC Acc | BL α₀ | AE α₀ | AC α₀ | BL Ent | AE Ent | AC Ent |")
    A("|------|--------|--------|--------|--------|--------|--------|--------|--------|--------|")

    bl_ck = {r["checkpoint"]: r for r in bl_rows}
    ae_ck = {r["checkpoint"]: r for r in ae_rows}
    ac_ck = {r["checkpoint"]: r for r in ac_rows}
    all_ckpts = sorted(set(list(bl_ck.keys()) + list(ae_ck.keys()) + list(ac_ck.keys())))

    for ck in all_ckpts:
        vals = []
        for d in [bl_ck, ae_ck, ac_ck]:
            r = d.get(ck, {})
            vals.append(f"{r.get('accuracy', -1):.4f}" if r else "—")
        for d in [bl_ck, ae_ck, ac_ck]:
            r = d.get(ck, {})
            vals.append(f"{r.get('alpha0', -1):.4f}" if r else "—")
        for d in [bl_ck, ae_ck, ac_ck]:
            r = d.get(ck, {})
            vals.append(f"{r.get('route_entropy', -1):.4f}" if r else "—")
        A(f"| {ck} | {' | '.join(vals)} |")
    A("")

    # Final summary
    bl_f = bl_ck.get(BENCH_BUDGET, {})
    ae_f = ae_ck.get(BENCH_BUDGET, {})
    ac_f = ac_ck.get(BENCH_BUDGET, {})

    def _v(d: dict, k: str, fmt: str = ".4f") -> str:
        v = d.get(k, -1)
        if v == -1:
            return "—"
        return f"{v:{fmt}}"

    sps_bl = BENCH_BUDGET / rt_bl if rt_bl > 0 else 0
    sps_ae = BENCH_BUDGET / rt_ae if rt_ae > 0 else 0
    sps_ac = BENCH_BUDGET / rt_ac if rt_ac > 0 else 0

    # Find first solve checkpoint for each variant
    def _first_solve(rows: List[dict]) -> str:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return str(r["checkpoint"])
        return "never"

    bl_solve = _first_solve(bl_rows)
    ae_solve = _first_solve(ae_rows)
    ac_solve = _first_solve(ac_rows)

    A("## Final Summary")
    A("")
    A("| Metric | Baseline | Ang+Euc | Ang+Circ | Δ(Circ−Euc) |")
    A("|--------|----------|---------|----------|-------------|")
    A(f"| Final accuracy | {_v(bl_f, 'accuracy')} | {_v(ae_f, 'accuracy')} | {_v(ac_f, 'accuracy')} | {ac_f.get('accuracy', 0) - ae_f.get('accuracy', 0):+.4f} |")
    A(f"| Final α₀ | {_v(bl_f, 'alpha0')} | {_v(ae_f, 'alpha0')} | {_v(ac_f, 'alpha0')} | {ac_f.get('alpha0', 0) - ae_f.get('alpha0', 0):+.4f} |")
    A(f"| Final route entropy | {_v(bl_f, 'route_entropy')} | {_v(ae_f, 'route_entropy')} | {_v(ac_f, 'route_entropy')} | {ac_f.get('route_entropy', 0) - ae_f.get('route_entropy', 0):+.4f} |")
    A(f"| Final transport frac | {_v(bl_f, 'transport_frac')} | {_v(ae_f, 'transport_frac')} | {_v(ac_f, 'transport_frac')} | {ac_f.get('transport_frac', 0) - ae_f.get('transport_frac', 0):+.4f} |")
    A(f"| First solve step | {bl_solve} | {ae_solve} | {ac_solve} | — |")
    A(f"| Parameters | {bl_np} | {ae_np} | {ac_np} | 0 |")
    A(f"| Training time (s) | {rt_bl:.1f} | {rt_ae:.1f} | {rt_ac:.1f} | {rt_ac - rt_ae:+.1f} |")
    A(f"| Steps/sec | {sps_bl:.1f} | {sps_ae:.1f} | {sps_ac:.1f} | {sps_ac - sps_ae:+.1f} |")
    A("")

    # Key questions
    A("## Key Questions")
    A("")
    bl_acc = bl_f.get("accuracy", 0)
    ae_acc = ae_f.get("accuracy", 0)
    ac_acc = ac_f.get("accuracy", 0)

    A(f"**Did all three solve D=24?** "
      f"Baseline: {'YES' if bl_acc >= 0.9 else 'NO'}, "
      f"Ang+Euc: {'YES' if ae_acc >= 0.9 else 'NO'}, "
      f"Ang+Circ: {'YES' if ac_acc >= 0.9 else 'NO'}")
    A("")

    A(f"**First solve step:** Baseline: {bl_solve}, Ang+Euc: {ae_solve}, "
      f"Ang+Circ: {ac_solve}")
    A("")

    # Circular mean vs Euclidean comparison
    delta_acc = ac_acc - ae_acc
    A("**Does circular-mean mixture improve accuracy over Euclidean angular?**")
    if delta_acc > 0.005:
        A(f"YES — +{delta_acc:.4f}")
    elif delta_acc < -0.005:
        A(f"NO — worse by {-delta_acc:.4f}")
    else:
        A(f"MARGINAL — difference is {delta_acc:+.4f} (within noise)")
    A("")

    if ac_solve != "never" and ae_solve != "never":
        ac_s = int(ac_solve)
        ae_s = int(ae_solve)
        A("**Does circular-mean converge faster?**")
        if ac_s < ae_s:
            A(f"YES — {ae_s - ac_s} steps earlier than Euclidean angular")
        elif ac_s > ae_s:
            A(f"NO — {ac_s - ae_s} steps later than Euclidean angular")
        else:
            A("SAME — identical solve checkpoint")
        A("")

    A("**Does circular-mean affect runtime?**")
    if sps_ac > sps_ae * 1.05:
        A(f"FASTER — {sps_ac/sps_ae:.2f}× ({sps_ac:.1f} vs {sps_ae:.1f} sps)")
    elif sps_ac < sps_ae * 0.95:
        A(f"SLOWER — {sps_ae/sps_ac:.2f}× ({sps_ac:.1f} vs {sps_ae:.1f} sps)")
    else:
        A(f"SIMILAR — {sps_ac:.1f} vs {sps_ae:.1f} sps")
    A("")

    # Interpretation
    A("## Interpretation")
    A("")
    if ac_acc >= 0.9 and ae_acc >= 0.9:
        if ac_solve != "never" and ae_solve != "never":
            ac_s = int(ac_solve)
            ae_s = int(ae_solve)
            if ac_s < ae_s:
                A("Circular-mean normalization **improves convergence** over Euclidean angular.")
                A("This supports M2 as a real binding mismatch: the Euclidean mixture was")
                A("producing off-torus intermediate states that the MLP had to compensate for.")
                A("Making the mixture geometrically correct on S¹ helps the optimizer.")
            elif ac_s == ae_s:
                A("Circular-mean normalization achieves **identical convergence** to Euclidean angular.")
                A("M2 does not appear to be a binding mismatch at D=24. The MLP learns to")
                A("handle off-torus intermediates without difficulty at this scale.")
            else:
                A("Circular-mean normalization converges **slower** than Euclidean angular.")
                A("The normalization may be discarding useful gradient magnitude information")
                A("(the resultant length carries concentration/certainty signal that the MLP")
                A("may be using). M2 is not the binding mismatch at this scale.")
        else:
            A("Both angular variants solve D=24. See convergence details above.")
    elif ac_acc < 0.9 and ae_acc >= 0.9:
        A("**Circular-mean normalization BREAKS learning.** The normalization discards")
        A("magnitude information that the MLP needs. The resultant length of the")
        A("mixture encodes operator weight concentration, which is a useful training signal.")
        A("M2 is real but the fix direction is wrong: the MLP USES the off-torus signal.")
    else:
        A("Neither angular variant solved D=24 reliably.")
    A("")

    # Honesty
    A("## Honesty Section")
    A("")
    A("### What Improved")
    A("")
    improvements = []
    if ac_solve != "never" and ae_solve != "never" and int(ac_solve) < int(ae_solve):
        improvements.append(f"- Convergence: {int(ae_solve) - int(ac_solve)} steps faster")
    if delta_acc > 0.005:
        improvements.append(f"- Accuracy: +{delta_acc:.4f}")
    ac_tf = ac_f.get("transport_frac", 0)
    ae_tf = ae_f.get("transport_frac", 0)
    if ac_tf > ae_tf + 0.01:
        improvements.append(f"- Transport fraction: +{ac_tf - ae_tf:.4f}")
    if not improvements:
        improvements.append("- No material improvement from circular-mean normalization")
    for line in improvements:
        A(line)
    A("")

    A("### What Did Not Improve")
    A("")
    no_improve = []
    if abs(delta_acc) <= 0.005:
        no_improve.append("- Final accuracy: no material change")
    if ac_solve != "never" and ae_solve != "never" and int(ac_solve) >= int(ae_solve):
        no_improve.append(f"- Convergence speed: not faster (solve at step {ac_solve} vs {ae_solve})")
    if sps_ac < sps_ae * 0.95:
        no_improve.append(f"- Runtime: slower ({sps_ac:.1f} vs {sps_ae:.1f} sps)")
    if not no_improve:
        no_improve.append("- All measured metrics improved or stayed constant")
    for line in no_improve:
        A(line)
    A("")

    A("### What Remains the Next Mismatch")
    A("")
    A("If M2 circular mean does not materially help:")
    A("- **M3**: SGD flat metric ignores torus curvature — optimizer steps in R^n, not on T⁴")
    A("- **M4**: Euclidean backward adjoint ≠ geometric adjoint of transport operators")
    A("- **M5**: No gradient distinction between transport and non-transport operators")
    A("")
    A("If M2 helps but is insufficient:")
    A("- The resultant-length (magnitude before normalization) carries concentration")
    A("  information. A hybrid approach could preserve magnitude as an auxiliary feature")
    A("  while still normalizing the direction.")
    A("")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(L) + "\n")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main() -> None:
    print("=" * 70)
    print("Circular-Mean Mixture Experiment v1")
    print("=" * 70)
    print(f"  3 variants: baseline_onehot, angular_euclidean, angular_circular")
    print(f"  D={D_CONTEXT}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
          f"budget={BENCH_BUDGET}")
    print(f"  Checkpoints: {CHECKPOINTS}")
    print(f"  torch: {torch.__version__}, threads: {torch.get_num_threads()}")
    print()

    device = "cpu"

    # ── Load state tables (one-time BFS) ──
    print("Loading state tables (one-time BFS)...")
    import random as _pyrand
    v6  = _load_v6torch()
    rng = _pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_states = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN, TR, tau0, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    TN   = TN.to(device)
    TR   = TR.to(device)
    tau0 = tau0.to(device)
    pool_ids = pool_ids.to(device)

    tn_mb = TN.element_size() * TN.nelement() / 1e6
    tr_mb = TR.element_size() * TR.nelement() / 1e6
    print(f"  {n_states:,} states, "
          f"TN={TN.shape}, TR={TR.shape}, "
          f"mem={tn_mb:.1f} MB TN + {tr_mb:.1f} MB TR")
    print()

    # ── Convert to angular encoding ──
    print("Converting state tables to angular encoding...")
    TN_ang   = convert_onehot_to_angular(TN)
    tau0_ang = convert_onehot_to_angular(tau0)
    print(f"  TN:   {TN.shape} → {TN_ang.shape}")
    print(f"  tau0: {tau0.shape} → {tau0_ang.shape}")

    # Verify angular encoding
    sample_oh = TN[42, 0]
    sample_an = TN_ang[42, 0]
    print(f"  Sample verification (state=42, op=0):")
    print(f"    one-hot: {[round(float(x), 4) for x in sample_oh]}")
    print(f"    angular: {[round(float(x), 4) for x in sample_an]}")

    # Verify circular normalization
    sample_tau = TN_ang[42, 0:1]  # (1, 8)
    normed = _circ_norm(sample_tau)
    pairs = normed.view(1, N_PHASE_PAIRS, 2)
    norms = (pairs * pairs).sum(dim=2).sqrt()
    print(f"  Circ-norm verification: pair norms = "
          f"{[round(float(x), 6) for x in norms[0]]}")

    # Test with a non-unit mixture to show normalization effect
    mixed = 0.5 * TN_ang[42, 0:1] + 0.5 * TN_ang[42, 1:2]
    mixed_pairs = mixed.view(1, N_PHASE_PAIRS, 2)
    mixed_norms = (mixed_pairs * mixed_pairs).sum(dim=2).sqrt()
    normed_mixed = _circ_norm(mixed)
    normed_pairs = normed_mixed.view(1, N_PHASE_PAIRS, 2)
    normed_norms = (normed_pairs * normed_pairs).sum(dim=2).sqrt()
    print(f"  50/50 mix pre-norm norms:  {[round(float(x), 4) for x in mixed_norms[0]]}")
    print(f"  50/50 mix post-norm norms: {[round(float(x), 6) for x in normed_norms[0]]}")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 1: Baseline one-hot
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 1: baseline_onehot (D_TAU=21, Euclidean)")
    print("=" * 50)
    model_bl = RouterBaseline(TN, TR, tau0, pool_ids)
    n_bl = sum(p.numel() for p in model_bl.parameters())
    print(f"  Parameters: {n_bl}")
    bl_rows, rt_bl = train_and_evaluate(
        "baseline_onehot", model_bl, pool_ids, device, circular_mean=False)
    sps_bl = BENCH_BUDGET / rt_bl
    print(f"  Total: {rt_bl:.1f}s, {sps_bl:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 2: Angular + Euclidean
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 2: angular_euclidean (D_TAU=8, Euclidean)")
    print("=" * 50)
    model_ae = RouterAngularEuc(TN_ang, TR, tau0_ang, pool_ids)
    n_ae = sum(p.numel() for p in model_ae.parameters())
    print(f"  Parameters: {n_ae}")
    ae_rows, rt_ae = train_and_evaluate(
        "angular_euclidean", model_ae, pool_ids, device, circular_mean=False)
    sps_ae = BENCH_BUDGET / rt_ae
    print(f"  Total: {rt_ae:.1f}s, {sps_ae:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 3: Angular + Circular Mean
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 3: angular_circular (D_TAU=8, circular mean)")
    print("=" * 50)
    model_ac = RouterAngularCirc(TN_ang, TR, tau0_ang, pool_ids)
    n_ac = sum(p.numel() for p in model_ac.parameters())
    print(f"  Parameters: {n_ac}")
    ac_rows, rt_ac = train_and_evaluate(
        "angular_circular", model_ac, pool_ids, device, circular_mean=True)
    sps_ac = BENCH_BUDGET / rt_ac
    print(f"  Total: {rt_ac:.1f}s, {sps_ac:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Write CSV
    # ══════════════════════════════════════════════════════════════════
    all_rows = bl_rows + ae_rows + ac_rows

    # Compute solve_step per variant
    def _solve_step(rows: List[dict]) -> int:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return r["checkpoint"]
        return -1

    bl_ss = _solve_step(bl_rows)
    ae_ss = _solve_step(ae_rows)
    ac_ss = _solve_step(ac_rows)
    solve_map = {
        "baseline_onehot": bl_ss,
        "angular_euclidean": ae_ss,
        "angular_circular": ac_ss,
    }

    csv_fields = [
        "variant", "checkpoint", "accuracy", "alpha0",
        "solve_step", "wall_time_seconds", "steps_per_second",
        "route_entropy", "transport_fraction", "loss",
        "d_tau", "n_params", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=csv_fields)
        w.writeheader()
        for r in all_rows:
            ckpt = r["checkpoint"]
            rt = r["runtime_seconds"]
            w.writerow({
                "variant":            r["variant"],
                "checkpoint":         ckpt,
                "accuracy":           r["accuracy"],
                "alpha0":             r["alpha0"],
                "solve_step":         solve_map.get(r["variant"], -1),
                "wall_time_seconds":  rt,
                "steps_per_second":   round(ckpt / rt, 1) if rt > 0 else 0,
                "route_entropy":      r["route_entropy"],
                "transport_fraction": r["transport_frac"],
                "loss":               r["loss"],
                "d_tau":              r["d_tau"],
                "n_params":           r["n_params"],
                "note":               r["note"],
            })
    print(f"CSV → {CSV_OUT}")

    # ══════════════════════════════════════════════════════════════════
    # Write markdown
    # ══════════════════════════════════════════════════════════════════
    write_md(bl_rows, ae_rows, ac_rows, rt_bl, rt_ae, rt_ac)
    print(f"MD  → {MD_OUT}")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Console summary
    # ══════════════════════════════════════════════════════════════════
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)

    bl_f = {r["checkpoint"]: r for r in bl_rows}.get(BENCH_BUDGET, {})
    ae_f = {r["checkpoint"]: r for r in ae_rows}.get(BENCH_BUDGET, {})
    ac_f = {r["checkpoint"]: r for r in ac_rows}.get(BENCH_BUDGET, {})

    hdr = f"{'':>22s} {'Baseline':>10s} {'Ang+Euc':>10s} {'Ang+Circ':>10s} {'Δ(C-E)':>10s}"
    print(hdr)

    def _row(label: str, bv: float, ev: float, cv: float, fmt: str = ".4f") -> str:
        d = cv - ev
        return (f"{label:>22s} {bv:>10{fmt}} {ev:>10{fmt}} "
                f"{cv:>10{fmt}} {d:>+10{fmt}}")

    print(_row("Accuracy",
               bl_f.get("accuracy", 0), ae_f.get("accuracy", 0),
               ac_f.get("accuracy", 0)))
    print(_row("Alpha0",
               bl_f.get("alpha0", 0), ae_f.get("alpha0", 0),
               ac_f.get("alpha0", 0)))
    print(_row("Route entropy",
               bl_f.get("route_entropy", 0), ae_f.get("route_entropy", 0),
               ac_f.get("route_entropy", 0)))
    print(_row("Transport frac",
               bl_f.get("transport_frac", 0), ae_f.get("transport_frac", 0),
               ac_f.get("transport_frac", 0)))

    print(f"{'Solve step':>22s} {'':>4s}{bl_ss:>6d} {'':>4s}{ae_ss:>6d} "
          f"{'':>4s}{ac_ss:>6d} {'':>10s}")
    print(f"{'Parameters':>22s} {'':>4s}{n_bl:>6d} {'':>4s}{n_ae:>6d} "
          f"{'':>4s}{n_ac:>6d} {'':>4s}{n_ac - n_ae:>+6d}")
    print(_row("Steps/sec", sps_bl, sps_ae, sps_ac, ".1f"))
    print("=" * 70)


if __name__ == "__main__":
    main()
