#!/usr/bin/env python3
"""run_router_angular_radial_hybrid_v1.py

Angular + Radial Hybrid experiment: testing the hypothesis that the
correct training representation is angular direction + preserved radial
magnitude, not pure Euclidean and not pure torus projection.

Motivation:
  - M2 (circular-mean mixture) BROKE learning: discarding resultant
    magnitude removed a useful training signal.
  - The model uses off-torus magnitude as a confidence/concentration signal.
  - Therefore the correct representation may be S^1 x R+  per phase pair,
    not S^1 alone and not R^2 alone.

Four-way comparison:
  1. BASELINE_ONEHOT   -- one-hot tau (D_TAU=21), Euclidean mixture
  2. ANGULAR_EUC       -- angular tau (D_TAU=8),  Euclidean mixture
  3. ANGULAR_CIRC      -- angular tau (D_TAU=8),  circular-mean (reference)
  4. ANGULAR_HYBRID    -- angular direction(8) + magnitude(4) = (D_TAU=12)

Plus: radial-law measurement -- magnitude statistics by checkpoint,
per-phase-pair distribution, correlation with accuracy/alpha0.

Same operators, task, D=24, training setup, seeds, temperature schedule.
"""
from __future__ import annotations

import csv
import importlib.util
import math
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_angular_radial_hybrid_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_angular_radial_hybrid_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked hyperparameters
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
D_TAU_HYB = 12;  D_IN_HYB = D_EMB + D_TAU_HYB   # 16  (8 direction + 4 magnitude)

PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
N_PHASE_PAIRS = 4
_TRANSPORT_TH = 3

CHECKPOINTS = [0, 250, 500, 1000, 1500, 2000, 2500, 3000]

# Thread policy: adaptive, not hard-locked. Applied in main().
# At current regime (D_HIDDEN=32, B=256), auto-selects 1 thread.
# At larger regimes, auto-scales to multi-thread.
from thread_policy import select_threads as _select_threads


# ═══════════════════════════════════════════════════════════════════════
# Load v6 torch base
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
# Angular conversion
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


def _circ_norm(tau: torch.Tensor) -> torch.Tensor:
    """Per-phase-pair normalization to unit circle."""
    B = tau.shape[0]
    p = tau.view(B, N_PHASE_PAIRS, 2)
    n = (p * p).sum(dim=2, keepdim=True).sqrt().clamp(min=1e-8)
    return (p / n).view(B, D_TAU_ANG)


# ═══════════════════════════════════════════════════════════════════════
# RouterBaseline -- one-hot tau, Euclidean mixture
# ═══════════════════════════════════════════════════════════════════════
class RouterBaseline(nn.Module):
    def __init__(
        self, TN: torch.Tensor, TR: torch.Tensor,
        tau0: torch.Tensor, pool: torch.Tensor,
        d_hidden: int = D_HIDDEN, d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT, sc: float = 0.05, seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        d_tau = D_TAU_OH; d_in = D_IN_OH
        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)
        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))
        self.W_emb = rp(VOCAB, D_EMB)
        self.W1 = rp(d_in, dh);   self.b1 = zp(dh)
        self.W2 = rp(dh, N_OPS);  self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_tau); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_tau, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]; D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            tn = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base = torch.einsum("bi,bij->bj", w, tn)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st = torch.stack(soft_taus, dim=1)
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# RouterAngularEuc -- angular tau, Euclidean mixture
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularEuc(nn.Module):
    def __init__(
        self, TN: torch.Tensor, TR: torch.Tensor,
        tau0: torch.Tensor, pool: torch.Tensor,
        d_hidden: int = D_HIDDEN, d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT, sc: float = 0.05, seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        d_tau = D_TAU_ANG; d_in = D_IN_ANG
        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)
        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))
        self.W_emb = rp(VOCAB, D_EMB)
        self.W1 = rp(d_in, dh);   self.b1 = zp(dh)
        self.W2 = rp(dh, N_OPS);  self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_tau); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_tau, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]; D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            tn = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base = torch.einsum("bi,bij->bj", w, tn)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st = torch.stack(soft_taus, dim=1)
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# RouterAngularCirc -- angular tau, circular-mean mixture (failed ref)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularCirc(nn.Module):
    def __init__(
        self, TN: torch.Tensor, TR: torch.Tensor,
        tau0: torch.Tensor, pool: torch.Tensor,
        d_hidden: int = D_HIDDEN, d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT, sc: float = 0.05, seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        d_tau = D_TAU_ANG; d_in = D_IN_ANG
        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)
        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))
        self.W_emb = rp(VOCAB, D_EMB)
        self.W1 = rp(d_in, dh);   self.b1 = zp(dh)
        self.W2 = rp(dh, N_OPS);  self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_tau); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_tau, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]; D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            tn = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]
            h = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base = torch.einsum("bi,bij->bj", w, tn)
            _p = base.view(B, 4, 2)
            _n = (_p * _p).sum(dim=2, keepdim=True).sqrt().clamp(min=1e-8)
            base = (_p / _n).view(B, 8)
            if t == 0:
                tau_prev = base + self.W_tok_inject[x0]
                _p2 = tau_prev.view(B, 4, 2)
                _n2 = (_p2 * _p2).sum(dim=2, keepdim=True).sqrt().clamp(min=1e-8)
                tau_prev = (_p2 / _n2).view(B, 8)
            else:
                tau_prev = base
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        st = torch.stack(soft_taus, dim=1)
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# RouterAngularHybrid -- angular direction + preserved radial magnitude
#
# Representation per recurrence step:
#   tau = [direction(8), magnitude(4)]  =  12 dims
#
# direction: (cos theta_bar, sin theta_bar) per phase pair, normalized to S^1
# magnitude: resultant length per phase pair (R+), preserved explicitly
#
# This separates circular geometry (direction) from concentration signal
# (magnitude) so the model can use both.
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularHybrid(nn.Module):
    def __init__(
        self, TN: torch.Tensor, TR: torch.Tensor,
        tau0: torch.Tensor, pool: torch.Tensor,
        d_hidden: int = D_HIDDEN, d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT, sc: float = 0.05, seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        d_tau = D_TAU_HYB; d_in = D_IN_HYB
        self.register_buffer("TN", TN)          # (N, 6, 8) angular
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0) # (N, 12) hybrid
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)
        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))
        self.W_emb = rp(VOCAB, D_EMB)
        self.W1 = rp(d_in, dh);   self.b1 = zp(dh)
        self.W2 = rp(dh, N_OPS);  self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_tau); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_tau, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]; D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]  # (B, 12)
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            tn = self.TN[state_ids]              # (B, 6, 8) angular
            embs = self.W_emb[tokens[:, t]]      # (B, 4)
            h = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)

            base = torch.einsum("bi,bij->bj", w, tn)  # (B, 8) angular mixture

            # ── Decompose into direction + magnitude ──
            _p = base.view(B, 4, 2)
            _mag = (_p * _p).sum(dim=2, keepdim=False).sqrt()  # (B, 4)
            _mag_safe = _mag.clamp(min=1e-8)
            _dir = (_p / _mag_safe.unsqueeze(2)).view(B, 8)    # (B, 8) unit direction
            hybrid = torch.cat([_dir, _mag], dim=1)            # (B, 12)

            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)

            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)

        st = torch.stack(soft_taus, dim=1)  # (B, D, 12)
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


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
# Evaluation with magnitude tracking
#
# mode: "onehot" | "ang_euc" | "ang_circ" | "hybrid"
# ═══════════════════════════════════════════════════════════════════════
def evaluate(
    model: nn.Module,
    pool_ids: torch.Tensor,
    D: int,
    n_eval: int,
    device: str,
    mode: str = "onehot",
) -> Dict:
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

    is_angular = mode != "onehot"
    mag_sum    = torch.zeros(N_PHASE_PAIRS, device=device)
    mag_sq_sum = torch.zeros(N_PHASE_PAIRS, device=device)
    mag_count  = 0

    with torch.no_grad():
        for _ in range(n_batches):
            B_ = min(B_eval, n_eval - total)
            if B_ <= 0:
                break
            idx  = torch.randint(0, len(pool_ids), (B_,), device=device)
            sids = pool_ids[idx]
            toks = torch.randint(0, VOCAB, (B_, D), device=device)
            x0   = torch.randint(0, VOCAB, (B_,),   device=device)
            toks[:, 0] = x0

            tau_prev  = model.tau0_table[sids]
            soft_taus: List[torch.Tensor] = []
            cur_sids  = sids.clone()

            for t in range(D):
                tn   = model.TN[cur_sids]
                embs = model.W_emb[toks[:, t]]
                h_in = torch.cat([embs, tau_prev], dim=1)
                h    = torch.tanh(h_in @ model.W1 + model.b1)
                logits = h @ model.W2 + model.b2
                w    = torch.softmax(logits / 0.05, dim=1)

                pc       = w.clamp(1e-12, 1.0)
                ent_sum += float((-(pc * pc.log()).sum(dim=1)).sum().item())
                k_hard       = w.argmax(dim=1)
                transport_n += int((k_hard >= _TRANSPORT_TH).sum().item())
                total_steps += B_

                base = torch.einsum("bi,bij->bj", w, tn)

                # ── Magnitude tracking (angular variants only) ──
                if is_angular:
                    _bp = base.view(B_, N_PHASE_PAIRS, 2)
                    step_mags = (_bp * _bp).sum(dim=2).sqrt()  # (B_, 4)
                    mag_sum    += step_mags.sum(dim=0)
                    mag_sq_sum += (step_mags * step_mags).sum(dim=0)
                    mag_count  += B_

                # ── Construct tau_prev based on mode ──
                if mode == "onehot" or mode == "ang_euc":
                    if t == 0:
                        tau_prev = base + model.W_tok_inject[x0]
                    else:
                        tau_prev = base

                elif mode == "ang_circ":
                    base = _circ_norm(base)
                    if t == 0:
                        tau_prev = base + model.W_tok_inject[x0]
                        tau_prev = _circ_norm(tau_prev)
                    else:
                        tau_prev = base

                elif mode == "hybrid":
                    _p = base.view(B_, 4, 2)
                    _mag = (_p * _p).sum(dim=2, keepdim=False).sqrt()
                    _mag_safe = _mag.clamp(min=1e-8)
                    _dir = (_p / _mag_safe.unsqueeze(2)).view(B_, 8)
                    hybrid = torch.cat([_dir, _mag], dim=1)
                    if t == 0:
                        tau_prev = hybrid + model.W_tok_inject[x0]
                    else:
                        tau_prev = hybrid

                soft_taus.append(tau_prev)
                cur_sids = model.TR[cur_sids].gather(
                    1, k_hard.unsqueeze(1)).squeeze(1)

            st   = torch.stack(soft_taus, dim=1)
            h_a  = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            a_sc = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(a_sc, dim=1)
            alpha_sum += alpha.sum(dim=0)
            pred = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += int((pred.argmax(1) == x0).sum().item())
            total   += B_

    model.train()

    # Magnitude statistics
    if is_angular and mag_count > 0:
        mean_per_pair = mag_sum / mag_count
        var_per_pair  = (mag_sq_sum / mag_count) - mean_per_pair ** 2
        std_per_pair  = var_per_pair.clamp(min=0).sqrt()
        overall_mean  = float(mean_per_pair.mean().item())
        overall_std   = float(std_per_pair.mean().item())
        pp_mean = [float(x.item()) for x in mean_per_pair]
        pp_std  = [float(x.item()) for x in std_per_pair]
    else:
        overall_mean = -1.0
        overall_std  = -1.0
        pp_mean = [-1.0] * N_PHASE_PAIRS
        pp_std  = [-1.0] * N_PHASE_PAIRS

    return {
        "accuracy":       correct / max(total, 1),
        "route_entropy":  ent_sum / max(total_steps, 1),
        "transport_frac": transport_n / max(total_steps, 1),
        "alpha0":         float((alpha_sum[0] / max(total, 1)).item()),
        "b_pos0":         float(model.b_pos0.item()),
        "mean_mag":       overall_mean,
        "std_mag":        overall_std,
        "pp_mag_mean":    pp_mean,
        "pp_mag_std":     pp_std,
    }


# ═══════════════════════════════════════════════════════════════════════
# Training + checkpoint evaluation
# ═══════════════════════════════════════════════════════════════════════
def train_and_evaluate(
    variant_name: str,
    model: nn.Module,
    pool_ids: torch.Tensor,
    device: str,
    mode: str = "onehot",
) -> Tuple[List[dict], float]:
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

    ckpt_set = set(CHECKPOINTS)
    rows: List[dict] = []

    def _record(ckpt: int, elapsed: float, loss_v: float) -> None:
        sd = {k: v for k, v in scripted.state_dict().items()
              if k not in {"TN", "TR", "tau0_table", "pool_ids", "pos0_mask"}}
        model.load_state_dict(sd, strict=False)
        met = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device, mode=mode)
        solved = "solved" if met["accuracy"] >= 0.90 else ""
        rows.append({
            "variant": variant_name, "checkpoint": ckpt,
            "accuracy": round(met["accuracy"], 4),
            "alpha0": round(met["alpha0"], 4),
            "route_entropy": round(met["route_entropy"], 4),
            "transport_frac": round(met["transport_frac"], 4),
            "mean_mag": round(met["mean_mag"], 5),
            "std_mag": round(met["std_mag"], 5),
            "pp_mag_mean": [round(x, 5) for x in met["pp_mag_mean"]],
            "pp_mag_std":  [round(x, 5) for x in met["pp_mag_std"]],
            "b_pos0": round(met["b_pos0"], 3),
            "runtime_seconds": round(elapsed, 2),
            "loss": round(loss_v, 4),
            "d_tau": d_tau, "d_in": d_in, "n_params": n_params,
            "jit": "yes" if jit_ok else "no",
            "note": solved,
        })
        if ckpt > 0:
            print(f"    step {ckpt:>5}: acc={met['accuracy']:.4f}  "
                  f"alpha0={met['alpha0']:.4f}  "
                  f"loss={loss_v:.4f}  "
                  f"mag={met['mean_mag']:.4f}  "
                  f"t={elapsed:.1f}s  {solved}", flush=True)

    if 0 in ckpt_set:
        _record(0, 0.0, -1.0)
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
            _record(step_num, elapsed, loss_val)
            scripted.train()

    runtime = time.perf_counter() - t0
    return rows, runtime


# ═══════════════════════════════════════════════════════════════════════
# Markdown report writer
# ═══════════════════════════════════════════════════════════════════════
def write_md(
    bl_rows: List[dict], ae_rows: List[dict],
    ac_rows: List[dict], ah_rows: List[dict],
    rt_bl: float, rt_ae: float, rt_ac: float, rt_ah: float,
) -> None:
    L: List[str] = []
    A = L.append

    A("# Angular + Radial Hybrid — Experiment v1")
    A("")
    A("## Purpose")
    A("")
    A("Test whether the correct training representation is **angular direction +")
    A("preserved radial magnitude** (S¹ × R⁺ per phase pair), rather than pure")
    A("Euclidean (R²) or pure torus projection (S¹).")
    A("")
    A("Motivated by the circular-mean experiment failure: discarding resultant")
    A("magnitude broke learning because the model uses it as a")
    A("confidence/concentration signal.")
    A("")

    A("## Hybrid Representation")
    A("")
    A("After the einsum mixture `base = Σ wᵢ · (cos θᵢ, sin θᵢ)`:")
    A("")
    A("1. **Resultant magnitude** per phase pair: r = ‖(Σ wᵢ cos θᵢ, Σ wᵢ sin θᵢ)‖")
    A("2. **Unit direction** per phase pair: d̂ = (cos θ̄, sin θ̄) = base_pair / r")
    A("3. **Hybrid tau**: `[d̂₀, d̂₁, d̂₂, d̂₃, r₀, r₁, r₂, r₃]` = 12 dims")
    A("")
    A("| Component | Dims | Interpretation |")
    A("|-----------|------|----------------|")
    A("| direction | 8 | circular mean direction on S¹ per phase pair |")
    A("| magnitude | 4 | resultant length (concentration/confidence) per phase pair |")
    A("| **total** | **12** | **S¹ × R⁺ per phase pair** |")
    A("")
    A("Direction is normalized to the unit circle (same as circular-mean).")
    A("Magnitude is preserved as a separate explicit feature (what circular-mean discarded).")
    A("The MLP sees both and can learn to use each independently.")
    A("")
    A("Initial states (on-torus): all magnitudes = 1.0.")
    A("During training: magnitudes vary as the softmax distributes weight across operators.")
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

    A("## Four Variants")
    A("")
    A("| Variant | Tau Encoding | D_TAU | D_IN | Mixture | Params |")
    A("|---------|-------------|-------|------|---------|--------|")
    bl_np = bl_rows[-1].get("n_params", "?") if bl_rows else "?"
    ae_np = ae_rows[-1].get("n_params", "?") if ae_rows else "?"
    ac_np = ac_rows[-1].get("n_params", "?") if ac_rows else "?"
    ah_np = ah_rows[-1].get("n_params", "?") if ah_rows else "?"
    A(f"| baseline_onehot | one-hot | {D_TAU_OH} | {D_IN_OH} | Euclidean | {bl_np} |")
    A(f"| angular_euclidean | cos/sin | {D_TAU_ANG} | {D_IN_ANG} | Euclidean | {ae_np} |")
    A(f"| angular_circular | cos/sin | {D_TAU_ANG} | {D_IN_ANG} | Circular mean | {ac_np} |")
    A(f"| angular_hybrid | dir+mag | {D_TAU_HYB} | {D_IN_HYB} | Direction+Magnitude | {ah_np} |")
    A("")

    # Convergence comparison
    A("## Convergence Comparison")
    A("")
    A("| Step | BL Acc | AE Acc | AC Acc | AH Acc | BL α₀ | AE α₀ | AC α₀ | AH α₀ |")
    A("|------|--------|--------|--------|--------|--------|--------|--------|--------|")
    bl_ck = {r["checkpoint"]: r for r in bl_rows}
    ae_ck = {r["checkpoint"]: r for r in ae_rows}
    ac_ck = {r["checkpoint"]: r for r in ac_rows}
    ah_ck = {r["checkpoint"]: r for r in ah_rows}
    all_ckpts = sorted(set(
        list(bl_ck) + list(ae_ck) + list(ac_ck) + list(ah_ck)))
    for ck in all_ckpts:
        vals = []
        for d in [bl_ck, ae_ck, ac_ck, ah_ck]:
            r = d.get(ck, {})
            vals.append(f"{r.get('accuracy', -1):.4f}" if r else "—")
        for d in [bl_ck, ae_ck, ac_ck, ah_ck]:
            r = d.get(ck, {})
            vals.append(f"{r.get('alpha0', -1):.4f}" if r else "—")
        A(f"| {ck} | {' | '.join(vals)} |")
    A("")

    # Final summary
    bl_f = bl_ck.get(BENCH_BUDGET, {})
    ae_f = ae_ck.get(BENCH_BUDGET, {})
    ac_f = ac_ck.get(BENCH_BUDGET, {})
    ah_f = ah_ck.get(BENCH_BUDGET, {})

    sps_bl = BENCH_BUDGET / rt_bl if rt_bl > 0 else 0
    sps_ae = BENCH_BUDGET / rt_ae if rt_ae > 0 else 0
    sps_ac = BENCH_BUDGET / rt_ac if rt_ac > 0 else 0
    sps_ah = BENCH_BUDGET / rt_ah if rt_ah > 0 else 0

    def _first_solve(rows: List[dict]) -> str:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return str(r["checkpoint"])
        return "never"

    bl_ss = _first_solve(bl_rows)
    ae_ss = _first_solve(ae_rows)
    ac_ss = _first_solve(ac_rows)
    ah_ss = _first_solve(ah_rows)

    A("## Final Summary")
    A("")
    A("| Metric | Baseline | Ang+Euc | Ang+Circ | **Ang+Hybrid** |")
    A("|--------|----------|---------|----------|----------------|")
    def _fv(d: dict, k: str, fmt: str = ".4f") -> str:
        v = d.get(k, -1); return f"{v:{fmt}}" if v >= 0 else "—"
    A(f"| Final accuracy | {_fv(bl_f,'accuracy')} | {_fv(ae_f,'accuracy')} | {_fv(ac_f,'accuracy')} | **{_fv(ah_f,'accuracy')}** |")
    A(f"| Final α₀ | {_fv(bl_f,'alpha0')} | {_fv(ae_f,'alpha0')} | {_fv(ac_f,'alpha0')} | **{_fv(ah_f,'alpha0')}** |")
    A(f"| Route entropy | {_fv(bl_f,'route_entropy')} | {_fv(ae_f,'route_entropy')} | {_fv(ac_f,'route_entropy')} | **{_fv(ah_f,'route_entropy')}** |")
    A(f"| Transport frac | {_fv(bl_f,'transport_frac')} | {_fv(ae_f,'transport_frac')} | {_fv(ac_f,'transport_frac')} | **{_fv(ah_f,'transport_frac')}** |")
    A(f"| First solve step | {bl_ss} | {ae_ss} | {ac_ss} | **{ah_ss}** |")
    A(f"| Parameters | {bl_np} | {ae_np} | {ac_np} | **{ah_np}** |")
    A(f"| Training time (s) | {rt_bl:.1f} | {rt_ae:.1f} | {rt_ac:.1f} | **{rt_ah:.1f}** |")
    A(f"| Steps/sec | {sps_bl:.1f} | {sps_ae:.1f} | {sps_ac:.1f} | **{sps_ah:.1f}** |")
    A("")

    # ── Magnitude analysis ──
    A("## Radial-Law Measurement")
    A("")
    A("### Resultant magnitude by checkpoint (angular variants)")
    A("")
    A("| Step | Ang+Euc mag | Ang+Circ mag | **Ang+Hybrid mag** | Ang+Euc std | Ang+Hybrid std |")
    A("|------|-------------|--------------|--------------------|-----------  |----------------|")
    for ck in all_ckpts:
        ae_r = ae_ck.get(ck, {}); ac_r = ac_ck.get(ck, {}); ah_r = ah_ck.get(ck, {})
        ae_m = f"{ae_r.get('mean_mag',-1):.5f}" if ae_r.get('mean_mag',-1)>=0 else "—"
        ac_m = f"{ac_r.get('mean_mag',-1):.5f}" if ac_r.get('mean_mag',-1)>=0 else "—"
        ah_m = f"{ah_r.get('mean_mag',-1):.5f}" if ah_r.get('mean_mag',-1)>=0 else "—"
        ae_s = f"{ae_r.get('std_mag',-1):.5f}" if ae_r.get('std_mag',-1)>=0 else "—"
        ah_s = f"{ah_r.get('std_mag',-1):.5f}" if ah_r.get('std_mag',-1)>=0 else "—"
        A(f"| {ck} | {ae_m} | {ac_m} | **{ah_m}** | {ae_s} | **{ah_s}** |")
    A("")

    A("### Per-phase-pair magnitude at final checkpoint")
    A("")
    A("| Phase Pair | Factor | Ang+Euc | Ang+Hybrid |")
    A("|------------|--------|---------|------------|")
    pair_names = ["swap (Z/2)", "coupled (Z/5)", "twist (Z/2)", "lift (Z/12)"]
    ae_pp = ae_f.get("pp_mag_mean", [-1]*4)
    ah_pp = ah_f.get("pp_mag_mean", [-1]*4)
    for i, pn in enumerate(pair_names):
        aev = f"{ae_pp[i]:.5f}" if ae_pp[i] >= 0 else "—"
        ahv = f"{ah_pp[i]:.5f}" if ah_pp[i] >= 0 else "—"
        A(f"| {i} | {pn} | {aev} | {ahv} |")
    A("")

    # Correlation analysis
    A("### Magnitude–accuracy correlation across checkpoints")
    A("")
    for vname, ckdata in [("Ang+Euc", ae_ck), ("Ang+Hybrid", ah_ck)]:
        accs = []; mags = []
        for ck in all_ckpts:
            r = ckdata.get(ck, {})
            a = r.get("accuracy", -1)
            m = r.get("mean_mag", -1)
            if a >= 0 and m >= 0:
                accs.append(a); mags.append(m)
        if len(accs) >= 3:
            n = len(accs)
            mx = sum(mags)/n; ax = sum(accs)/n
            cov = sum((m-mx)*(a-ax) for m,a in zip(mags,accs)) / n
            vm = sum((m-mx)**2 for m in mags) / n
            va = sum((a-ax)**2 for a in accs) / n
            r_val = cov / max((vm * va) ** 0.5, 1e-12)
            A(f"- **{vname}**: Pearson r(magnitude, accuracy) = {r_val:.4f}")
        else:
            A(f"- **{vname}**: insufficient data for correlation")
    A("")

    # Magnitude trajectory interpretation
    A("### Magnitude trajectory interpretation")
    A("")
    ae_mags = [ae_ck.get(ck, {}).get("mean_mag", -1) for ck in all_ckpts]
    ah_mags = [ah_ck.get(ck, {}).get("mean_mag", -1) for ck in all_ckpts]
    ae_valid = [m for m in ae_mags if m >= 0]
    ah_valid = [m for m in ah_mags if m >= 0]

    if ae_valid:
        m0 = ae_valid[0]; mf = ae_valid[-1]
        trend = "growing" if mf > m0 * 1.05 else ("shrinking" if mf < m0 * 0.95 else "stable")
        A(f"- **Ang+Euc**: magnitude {trend} ({m0:.4f} → {mf:.4f})")
    if ah_valid:
        m0 = ah_valid[0]; mf = ah_valid[-1]
        trend = "growing" if mf > m0 * 1.05 else ("shrinking" if mf < m0 * 0.95 else "stable")
        A(f"- **Ang+Hybrid**: magnitude {trend} ({m0:.4f} → {mf:.4f})")
    A("")

    # Interpretation
    A("## Interpretation")
    A("")
    ah_acc = ah_f.get("accuracy", 0)
    ae_acc = ae_f.get("accuracy", 0)
    ac_acc = ac_f.get("accuracy", 0)

    if ah_acc >= 0.9 and ah_ss != "never":
        ah_s_int = int(ah_ss)
        ae_s_int = int(ae_ss) if ae_ss != "never" else 99999
        if ah_s_int < ae_s_int:
            A("**Hybrid is the fastest-converging angular variant.** Explicit direction+magnitude")
            A("separation provides both circular geometry AND concentration signal, allowing the")
            A("model to learn faster than flat Euclidean angular encoding.")
            A("This confirms: the correct representation is S¹ × R⁺, not S¹ alone or R² alone.")
        elif ah_s_int == ae_s_int:
            A("**Hybrid converges at the same rate as Euclidean angular.** The explicit")
            A("direction/magnitude separation does not help beyond what the MLP already extracts")
            A("from the raw Euclidean angular vectors.")
        else:
            A("**Hybrid converges slower than Euclidean angular.** The decomposition into")
            A("direction+magnitude may be introducing overhead or disrupting gradients compared")
            A("to the raw Euclidean angular representation.")
    elif ah_acc >= 0.9:
        A("**Hybrid solves D=24** but convergence speed comparison is ambiguous.")
    else:
        A("**Hybrid does NOT solve D=24.** The direction+magnitude decomposition may be")
        A("interfering with learning, similar to (but potentially less severely than)")
        A("the circular-mean variant.")
    A("")
    if ac_acc < 0.9 and ah_acc >= 0.9:
        A("Key comparison: circular-mean FAILS (discards magnitude) while hybrid SUCCEEDS")
        A("(preserves magnitude). This directly confirms that radial magnitude is the")
        A("critical missing signal.")
        A("")
    A("")

    A("## Honesty Section")
    A("")
    A("### What Improved")
    A("")
    improvements = []
    if ah_ss != "never" and ac_ss == "never":
        improvements.append("- Hybrid SOLVES while circular-mean does NOT (magnitude preservation is critical)")
    if ah_ss != "never" and ae_ss != "never" and int(ah_ss) < int(ae_ss):
        improvements.append(f"- Hybrid converges {int(ae_ss)-int(ah_ss)} steps earlier than Euclidean angular")
    if ah_f.get("accuracy", 0) > ae_f.get("accuracy", 0) + 0.005:
        improvements.append(f"- Accuracy improved: {ah_f.get('accuracy',0):.4f} vs {ae_f.get('accuracy',0):.4f}")
    if not improvements:
        improvements.append("- No material improvement over Euclidean angular encoding")
    for line in improvements: A(line)
    A("")

    A("### What Did Not Improve")
    A("")
    no_improv = []
    if ah_ss != "never" and ae_ss != "never" and int(ah_ss) >= int(ae_ss):
        no_improv.append(f"- Convergence: solve step {ah_ss} vs {ae_ss} (not faster)")
    if sps_ah < sps_ae * 0.95:
        no_improv.append(f"- Runtime: {sps_ah:.1f} vs {sps_ae:.1f} sps (overhead from decomposition)")
    if not no_improv:
        no_improv.append("- All measured metrics improved or stayed constant")
    for line in no_improv: A(line)
    A("")

    A("### Whether M4 is Now the Next Binding Mismatch")
    A("")
    A("If hybrid representation resolves M2 fully, the remaining mismatches are:")
    A("- **M3**: SGD flat metric ignores torus curvature")
    A("- **M4**: Euclidean backward adjoint ≠ geometric adjoint of transport")
    A("- **M5**: No gradient distinction between transport/non-transport operators")
    A("")
    A("M4 (geometric adjoint) is the most likely next binding mismatch because it")
    A("affects the backward pass — the dominant runtime component — and the current")
    A("backward pass treats all operators' gradients identically regardless of their")
    A("geometric transport structure.")
    A("")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(L) + "\n")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main() -> None:
    print("=" * 70)
    print("Angular + Radial Hybrid Experiment v1")
    print("=" * 70)

    # Adaptive thread policy
    _select_threads(BATCH_SIZE, D_IN_HYB, D_HIDDEN)

    print(f"  4 variants: baseline, ang_euc, ang_circ, ang_hybrid")
    print(f"  D={D_CONTEXT}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
          f"budget={BENCH_BUDGET}")
    print(f"  Checkpoints: {CHECKPOINTS}")
    print(f"  torch: {torch.__version__}, threads: {torch.get_num_threads()}")
    print()

    device = "cpu"

    # ── Load state tables ──
    print("Loading state tables (one-time BFS)...")
    import random as _pyrand
    v6  = _load_v6torch()
    rng = _pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_states = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN_oh, TR, tau0_oh, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    TN_oh    = TN_oh.to(device)
    TR       = TR.to(device)
    tau0_oh  = tau0_oh.to(device)
    pool_ids = pool_ids.to(device)
    print(f"  {n_states:,} states")
    print()

    # ── Angular conversion ──
    print("Converting to angular encoding...")
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    print(f"  TN:   {TN_oh.shape} → {TN_ang.shape}")
    print(f"  tau0: {tau0_oh.shape} → {tau0_ang.shape}")

    # ── Hybrid conversion: append unit magnitudes ──
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    print(f"  tau0_hybrid: {tau0_hyb.shape}  (direction + magnitude=1.0)")

    # Verify decomposition round-trip
    sample = TN_ang[42, 0:1]  # (1, 8)
    _sp = sample.view(1, 4, 2)
    _sm = (_sp * _sp).sum(dim=2).sqrt()
    _sd = (_sp / _sm.clamp(min=1e-8).unsqueeze(2)).view(1, 8)
    _hyb = torch.cat([_sd, _sm], dim=1)
    print(f"  Decomposition check (state=42, op=0):")
    print(f"    angular:   {[round(float(x), 4) for x in sample[0]]}")
    print(f"    direction: {[round(float(x), 4) for x in _sd[0]]}")
    print(f"    magnitude: {[round(float(x), 4) for x in _sm[0]]}")
    print(f"    hybrid:    {[round(float(x), 4) for x in _hyb[0]]}")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 1: Baseline one-hot
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 1: baseline_onehot (D_TAU=21, Euclidean)")
    print("=" * 50)
    model_bl = RouterBaseline(TN_oh, TR, tau0_oh, pool_ids)
    print(f"  Parameters: {sum(p.numel() for p in model_bl.parameters())}")
    bl_rows, rt_bl = train_and_evaluate(
        "baseline_onehot", model_bl, pool_ids, device, mode="onehot")
    print(f"  Total: {rt_bl:.1f}s, {BENCH_BUDGET/rt_bl:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 2: Angular + Euclidean
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 2: angular_euclidean (D_TAU=8, Euclidean)")
    print("=" * 50)
    model_ae = RouterAngularEuc(TN_ang, TR, tau0_ang, pool_ids)
    print(f"  Parameters: {sum(p.numel() for p in model_ae.parameters())}")
    ae_rows, rt_ae = train_and_evaluate(
        "angular_euclidean", model_ae, pool_ids, device, mode="ang_euc")
    print(f"  Total: {rt_ae:.1f}s, {BENCH_BUDGET/rt_ae:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 3: Angular + Circular (failed reference)
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 3: angular_circular (D_TAU=8, circ-mean ref)")
    print("=" * 50)
    model_ac = RouterAngularCirc(TN_ang, TR, tau0_ang, pool_ids)
    print(f"  Parameters: {sum(p.numel() for p in model_ac.parameters())}")
    ac_rows, rt_ac = train_and_evaluate(
        "angular_circular", model_ac, pool_ids, device, mode="ang_circ")
    print(f"  Total: {rt_ac:.1f}s, {BENCH_BUDGET/rt_ac:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Variant 4: Angular + Radial Hybrid
    # ══════════════════════════════════════════════════════════════════
    print("=" * 50)
    print("VARIANT 4: angular_hybrid (D_TAU=12, dir+mag)")
    print("=" * 50)
    model_ah = RouterAngularHybrid(TN_ang, TR, tau0_hyb, pool_ids)
    print(f"  Parameters: {sum(p.numel() for p in model_ah.parameters())}")
    ah_rows, rt_ah = train_and_evaluate(
        "angular_hybrid", model_ah, pool_ids, device, mode="hybrid")
    print(f"  Total: {rt_ah:.1f}s, {BENCH_BUDGET/rt_ah:.1f} sps")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Write CSV
    # ══════════════════════════════════════════════════════════════════
    all_rows = bl_rows + ae_rows + ac_rows + ah_rows
    csv_fields = [
        "variant", "checkpoint", "accuracy", "alpha0",
        "route_entropy", "transport_fraction",
        "mean_resultant_magnitude", "magnitude_std",
        "pp_mag_swap", "pp_mag_coupled", "pp_mag_twist", "pp_mag_lift",
        "runtime_seconds", "steps_per_second",
        "loss", "d_tau", "n_params", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=csv_fields)
        w.writeheader()
        for r in all_rows:
            ckpt = r["checkpoint"]
            rt = r["runtime_seconds"]
            pp = r.get("pp_mag_mean", [-1]*4)
            w.writerow({
                "variant":                    r["variant"],
                "checkpoint":                 ckpt,
                "accuracy":                   r["accuracy"],
                "alpha0":                     r["alpha0"],
                "route_entropy":              r["route_entropy"],
                "transport_fraction":         r["transport_frac"],
                "mean_resultant_magnitude":   r["mean_mag"],
                "magnitude_std":              r["std_mag"],
                "pp_mag_swap":                pp[0],
                "pp_mag_coupled":             pp[1],
                "pp_mag_twist":               pp[2],
                "pp_mag_lift":                pp[3],
                "runtime_seconds":            rt,
                "steps_per_second":           round(ckpt / rt, 1) if rt > 0 else 0,
                "loss":                       r["loss"],
                "d_tau":                      r["d_tau"],
                "n_params":                   r["n_params"],
                "note":                       r["note"],
            })
    print(f"CSV → {CSV_OUT}")

    # ══════════════════════════════════════════════════════════════════
    # Write markdown
    # ══════════════════════════════════════════════════════════════════
    write_md(bl_rows, ae_rows, ac_rows, ah_rows, rt_bl, rt_ae, rt_ac, rt_ah)
    print(f"MD  → {MD_OUT}")
    print()

    # ══════════════════════════════════════════════════════════════════
    # Console summary
    # ══════════════════════════════════════════════════════════════════
    print("=" * 72)
    print("SUMMARY")
    print("=" * 72)

    bl_f = {r["checkpoint"]: r for r in bl_rows}.get(BENCH_BUDGET, {})
    ae_f = {r["checkpoint"]: r for r in ae_rows}.get(BENCH_BUDGET, {})
    ac_f = {r["checkpoint"]: r for r in ac_rows}.get(BENCH_BUDGET, {})
    ah_f = {r["checkpoint"]: r for r in ah_rows}.get(BENCH_BUDGET, {})

    hdr = f"{'':>20s} {'Baseline':>9s} {'Ang+Euc':>9s} {'Ang+Circ':>9s} {'Hybrid':>9s}"
    print(hdr)

    def _pr(label: str, k: str, fmt: str = ".4f") -> None:
        bv = bl_f.get(k, -1); ev = ae_f.get(k, -1)
        cv = ac_f.get(k, -1); hv = ah_f.get(k, -1)
        print(f"{label:>20s} {bv:>9{fmt}} {ev:>9{fmt}} {cv:>9{fmt}} {hv:>9{fmt}}")

    _pr("Accuracy", "accuracy")
    _pr("Alpha0", "alpha0")
    _pr("Route entropy", "route_entropy")
    _pr("Transport frac", "transport_frac")
    _pr("Mean magnitude", "mean_mag", ".5f")

    def _fsolve(rows: List[dict]) -> int:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return r["checkpoint"]
        return -1

    print(f"{'Solve step':>20s} {_fsolve(bl_rows):>9d} {_fsolve(ae_rows):>9d} "
          f"{_fsolve(ac_rows):>9d} {_fsolve(ah_rows):>9d}")
    n_bl = sum(p.numel() for p in model_bl.parameters())
    n_ae = sum(p.numel() for p in model_ae.parameters())
    n_ac = sum(p.numel() for p in model_ac.parameters())
    n_ah = sum(p.numel() for p in model_ah.parameters())
    print(f"{'Parameters':>20s} {n_bl:>9d} {n_ae:>9d} {n_ac:>9d} {n_ah:>9d}")
    print(f"{'Steps/sec':>20s} {BENCH_BUDGET/rt_bl:>9.1f} {BENCH_BUDGET/rt_ae:>9.1f} "
          f"{BENCH_BUDGET/rt_ac:>9.1f} {BENCH_BUDGET/rt_ah:>9.1f}")
    print("=" * 72)


if __name__ == "__main__":
    main()
