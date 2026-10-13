#!/usr/bin/env python3
"""run_router_fanout_reduction_probe_v1.py

GEOMETRY-GUIDED FAN-OUT REDUCTION PROBE v1

Tests whether reducing the number of candidates scored/blended per routing
decision — via angular proximity geometry — affects accuracy or runtime.

GEOMETRIC PRUNING RULE:
  Angular similarity = dot product of current tau direction (tau_prev[:, :8])
  with each candidate tau-next direction (TN[state_ids, :, :8]).
  TN entries are unit angular vectors per phase pair (S^1 x S^1 x S^1 x S^1).
  Low-similarity candidates are masked to -inf before softmax.
  This filters out candidates that are geometrically far from the current
  phase trajectory — without using future-state lookahead.

VARIANTS:
  k6_baseline   — all 6 candidates, standard forward pass (exact baseline)
  k4_geom       — top-4 by angular proximity, remaining MLP-scored + soft-blended
  k3_geom       — top-3 by angular proximity, remaining MLP-scored + soft-blended
  k2_geom       — top-2 by angular proximity, remaining MLP-scored + soft-blended
  k1_geom_hard  — top-1 by angular proximity, hard tau (no MLP, no blend)

NOTE on k=1 gradient structure:
  For k=4,3,2: gradient flows through all remaining MLP logits via Gumbel-softmax.
  For k=1 hard (geom): MLP is bypassed entirely. Only W_tok_inject, W_attn,
  W_pred, and b_pos0 receive gradients. W1/W2 are not updated. This variant
  is included for completeness but expected to degrade.

LOCKED CONFIG: D=24, D_HIDDEN=32, B=256, cache-loaded, CPU.
"""
from __future__ import annotations

import csv
import math
import sys
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
CACHE_DIR   = RESULTS_DIR / "state_cache"
CACHE_PATH  = CACHE_DIR / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_fanout_reduction_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_fanout_reduction_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked config
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
DEVICE        = "cpu"
D_HIDDEN      = 32
BATCH_SIZE    = 256
D             = 24
B0_INIT       = 2.0
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS  # 12
D_IN_HYB      = D_EMB + D_TAU_HYB           # 16
N_OPS         = 6
TRANSPORT_TH  = 3   # ops >= this count as "transport"
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
MAX_STEPS     = 3000
EVAL_EVERY    = 500
N_EVAL        = 1000
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
N_TIMING      = 100   # forward calls for timing measurement

# Thread policy (non-blocking)
try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _select_threads
    _select_threads(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion utilities (locked — must match baseline exactly)
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


def prepare_hybrid_tables(
    TN_oh: torch.Tensor,
    tau0_oh: torch.Tensor,
    TR: torch.Tensor,
    pool_ids: torch.Tensor,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# RouterFanoutVariant
# ═══════════════════════════════════════════════════════════════════════
class RouterFanoutVariant(nn.Module):
    """Geometry-guided fan-out reduction variant of RouterAngularHybrid.

    fanout_k controls how many candidates are blended per step:
      k=6         → standard baseline (no geometric masking)
      k=4,3,2     → geometric mask eliminates (6-k) most-distant candidates
                    before softmax; MLP scores surviving k; soft-blend those k.
      k=1 (hard)  → use_geom_only=True: angular proximity selects 1 candidate,
                    MLP bypassed, hard tau update (no blend).

    Geometric criterion: angular similarity = dot(tau_dir, tn_i) for each i.
    tau_dir = tau_prev[:, :D_TAU_ANG] (the direction component of hybrid tau).
    TN entries are unit angular vectors → similarity in [-1, 1].
    """

    def __init__(
        self,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        tau0_hyb: torch.Tensor,
        pool_ids: torch.Tensor,
        fanout_k: int,
        use_geom_only: bool = False,
        d_hidden: int = D_HIDDEN,
        d_context: int = D,
        b0_init: float = B0_INIT,
        init_scale: float = INIT_SCALE,
        seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        self.fanout_k = fanout_k
        self.use_geom_only = use_geom_only  # True → k=1 pure geometric, no MLP in selection

        dh = d_hidden
        dha = max(8, dh // 4)
        d_tau = D_TAU_HYB

        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, d_context)
        m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))

        gen = torch.Generator().manual_seed(seed)

        def rp(*sh):
            return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))

        def zp(*sh):
            return nn.Parameter(torch.zeros(*sh))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);   self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);        self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau);       self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB);     self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B = state_ids.shape[0]
        D_len = tokens.shape[1]
        k = self.fanout_k

        tau_prev = self.tau0_table[state_ids]        # (B, 12)
        soft_taus: List[torch.Tensor] = []

        for t in range(D_len):
            tn = self.TN[state_ids]                  # (B, 6, 8) angular candidates

            if self.use_geom_only:
                # ── k=1 PURE GEOMETRIC HARD ────────────────────────────────────
                # Angular proximity selects best candidate. MLP not involved.
                # Hard tau update: exact TN entry for best geometric op.
                # W1, W2 receive no gradient here — see module docstring.
                cur_dir = tau_prev[:, :D_TAU_ANG]    # (B, 8)
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)  # (B, 6)
                best_op = ang_sim.argmax(dim=1)       # (B,)

                # Gather the single best angular tau-next
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                ).squeeze(1)                          # (B, 8)

                # TN entries are unit angular vectors (cos/sin) so magnitude = 1.0
                hybrid = torch.cat(
                    [best_ang, torch.ones(B, N_PHASE_PAIRS, device=best_ang.device)], dim=1
                )                                     # (B, 12)
                tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                state_ids = self.TR[state_ids].gather(
                    1, best_op.unsqueeze(1)
                ).squeeze(1)

            else:
                # ── MLP-SCORED PATH (k=6,4,3,2 and k=1_mlp_hard) ─────────────
                embs   = self.W_emb[tokens[:, t]]    # (B, 4)
                h      = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
                logits = h @ self.W2 + self.b2        # (B, 6)

                if k < N_OPS:
                    # Geometric mask: drop the (N_OPS - k) most-distant candidates
                    cur_dir = tau_prev[:, :D_TAU_ANG]
                    ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)  # (B, 6)
                    _, bot_idx = ang_sim.topk(N_OPS - k, dim=1, largest=False)
                    geom_mask = torch.zeros_like(logits)
                    geom_mask.scatter_(1, bot_idx, float("-inf"))
                    logits = logits + geom_mask

                if self.training:
                    u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                    w = torch.softmax(
                        (logits - torch.log(-torch.log(u))) / temp, dim=1
                    )
                else:
                    w = torch.softmax(logits / 0.05, dim=1)

                base  = torch.einsum("bi,bij->bj", w, tn)  # (B, 8) soft angular mixture
                pairs = base.view(B, N_PHASE_PAIRS, 2)
                mag   = (pairs * pairs).sum(dim=2).sqrt()   # (B, 4)
                mag_s = mag.clamp(min=1e-8)
                dirn  = (pairs / mag_s.unsqueeze(2)).view(B, D_TAU_ANG)  # (B, 8)
                hybrid = torch.cat([dirn, mag], dim=1)       # (B, 12)
                tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                state_ids = self.TR[state_ids].gather(
                    1, torch.argmax(w, dim=1).unsqueeze(1)
                ).squeeze(1)

        st  = torch.stack(soft_taus, dim=1)              # (B, D, 12)
        h_a = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc  = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Batch / Eval helpers (locked — identical to baseline)
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids: torch.Tensor, rng_gen: torch.Generator):
    B = BATCH_SIZE
    idx  = torch.randint(pool_ids.shape[0], (B,), generator=rng_gen)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (B,), generator=rng_gen)
    toks = torch.randint(VOCAB, (B, D), generator=rng_gen)
    toks[:, 0] = x0
    return sids, toks, x0


def evaluate(model: nn.Module, pool_ids: torch.Tensor, rng_gen: torch.Generator) -> float:
    model.eval()
    correct = 0
    total = 0
    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng_gen)
            logits = model(sids, toks, x0, 0.05)
            preds  = logits.argmax(dim=1)
            correct += (preds == x0).sum().item()
            total   += sids.shape[0]
    model.train()
    return correct / total if total > 0 else 0.0


# ═══════════════════════════════════════════════════════════════════════
# k=1 geometric inference on any trained model
# ═══════════════════════════════════════════════════════════════════════
def eval_k1_geom_on_model(
    model: "RouterFanoutVariant",
    pool_ids: torch.Tensor,
) -> Tuple[float, float]:
    """Evaluate a TRAINED model using k=1 geometric routing at inference time.

    Regardless of what fanout_k the model was trained with, force k=1 angular
    proximity selection during the eval forward pass. This directly tests whether
    the trained manifold geometry is sufficient for single-candidate routing.

    Returns: (accuracy, geom_mlp_agreement_rate)
      accuracy            — fraction correctly predicted using geom-k1 routing
      geom_mlp_agreement  — fraction of steps where geom-k1 == MLP argmax
                            (only meaningful for k>1 trained models)
    """
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 888)
    correct = 0
    total = 0
    agree_steps = 0
    total_steps = 0

    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = sids.shape[0]
            tau_prev = model.tau0_table[sids]
            soft_taus = []

            for t in range(D):
                tn = model.TN[sids]                       # (B, 6, 8)
                cur_dir = tau_prev[:, :D_TAU_ANG]         # (B, 8)
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)  # (B, 6)
                best_geom = ang_sim.argmax(dim=1)         # (B,) geom-k1 selection

                # Also compute MLP argmax for agreement measurement
                # (skip MLP for use_geom_only models since W1/W2 are untrained)
                if not model.use_geom_only:
                    embs   = model.W_emb[toks[:, t]]
                    h      = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                    logits = h @ model.W2 + model.b2
                    best_mlp = logits.argmax(dim=1)
                    agree_steps += (best_geom == best_mlp).sum().item()
                    total_steps += B

                # Hard tau from geometric best candidate
                best_ang = tn.gather(
                    1, best_geom.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                ).squeeze(1)
                hybrid = torch.cat(
                    [best_ang, torch.ones(B, N_PHASE_PAIRS, device=best_ang.device)], dim=1
                )
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids = model.TR[sids].gather(1, best_geom.unsqueeze(1)).squeeze(1)

            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            logits_pred = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            preds = logits_pred.argmax(dim=1)
            correct += (preds == x0).sum().item()
            total   += B

    model.train()
    acc   = round(correct / total, 4) if total > 0 else 0.0
    agree = round(agree_steps / total_steps, 4) if total_steps > 0 else -1.0
    return acc, agree


# ═══════════════════════════════════════════════════════════════════════
# Alpha0 + transport_fraction measurement
# ═══════════════════════════════════════════════════════════════════════
def measure_alpha0_transport(
    model: RouterFanoutVariant,
    pool_ids: torch.Tensor,
) -> Tuple[float, float]:
    """Measure alpha0 (mean pos-0 attention weight) and transport_fraction."""
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 999)
    alpha0_sum = 0.0
    transport_sum = 0.0
    n_decisions = 0
    n_batches = N_EVAL // BATCH_SIZE

    with torch.no_grad():
        for _ in range(n_batches):
            sids, toks, x0 = make_batch(pool_ids, rng)
            B = sids.shape[0]
            tau_prev = model.tau0_table[sids]
            transport_ops = 0
            total_ops = 0

            for t in range(D):
                tn = model.TN[sids]

                if model.use_geom_only:
                    cur_dir = tau_prev[:, :D_TAU_ANG]
                    ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                    best_op = ang_sim.argmax(dim=1)
                    transport_ops += (best_op >= TRANSPORT_TH).sum().item()
                    total_ops += B
                    best_ang = tn.gather(
                        1, best_op.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                    ).squeeze(1)
                    hybrid = torch.cat(
                        [best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1
                    )
                    tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                    sids = model.TR[sids].gather(1, best_op.unsqueeze(1)).squeeze(1)
                else:
                    embs   = model.W_emb[toks[:, t]]
                    h      = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                    logits = h @ model.W2 + model.b2
                    if model.fanout_k < N_OPS:
                        cur_dir = tau_prev[:, :D_TAU_ANG]
                        ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                        _, bot_idx = ang_sim.topk(N_OPS - model.fanout_k, dim=1, largest=False)
                        geom_mask = torch.zeros_like(logits)
                        geom_mask.scatter_(1, bot_idx, float("-inf"))
                        logits = logits + geom_mask
                    w = torch.softmax(logits / 0.05, dim=1)
                    k_hard = w.argmax(dim=1)
                    transport_ops += (k_hard >= TRANSPORT_TH).sum().item()
                    total_ops += B
                    base   = torch.einsum("bi,bij->bj", w, tn)
                    pairs  = base.view(B, N_PHASE_PAIRS, 2)
                    mag    = (pairs * pairs).sum(dim=2).sqrt()
                    mag_s  = mag.clamp(min=1e-8)
                    dirn   = (pairs / mag_s.unsqueeze(2)).view(B, D_TAU_ANG)
                    hybrid = torch.cat([dirn, mag], dim=1)
                    tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                    sids = model.TR[sids].gather(1, k_hard.unsqueeze(1)).squeeze(1)

            # Compute alpha0 for this batch via a clean forward pass
            model.eval()
            sids_eval, toks_eval, x0_eval = make_batch(pool_ids, rng)
            # Reconstruct soft_taus and alpha via a forward-like computation
            tau2 = model.tau0_table[sids_eval]
            soft_taus2 = []
            sids2 = sids_eval
            for t in range(D):
                tn2 = model.TN[sids2]
                if model.use_geom_only:
                    cur_dir2 = tau2[:, :D_TAU_ANG]
                    ang2 = torch.einsum("bi,bji->bj", cur_dir2, tn2)
                    bop2 = ang2.argmax(dim=1)
                    ba2 = tn2.gather(
                        1, bop2.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                    ).squeeze(1)
                    hyb2 = torch.cat([ba2, torch.ones(B, N_PHASE_PAIRS)], dim=1)
                    tau2 = (hyb2 + model.W_tok_inject[x0_eval]) if t == 0 else hyb2
                    sids2 = model.TR[sids2].gather(1, bop2.unsqueeze(1)).squeeze(1)
                else:
                    emb2   = model.W_emb[toks_eval[:, t]]
                    h2     = torch.tanh(torch.cat([emb2, tau2], dim=1) @ model.W1 + model.b1)
                    lg2    = h2 @ model.W2 + model.b2
                    if model.fanout_k < N_OPS:
                        cd2 = tau2[:, :D_TAU_ANG]
                        as2 = torch.einsum("bi,bji->bj", cd2, tn2)
                        _, bi2 = as2.topk(N_OPS - model.fanout_k, dim=1, largest=False)
                        gm2 = torch.zeros_like(lg2)
                        gm2.scatter_(1, bi2, float("-inf"))
                        lg2 = lg2 + gm2
                    w2 = torch.softmax(lg2 / 0.05, dim=1)
                    bs2 = torch.einsum("bi,bij->bj", w2, tn2)
                    pr2 = bs2.view(B, N_PHASE_PAIRS, 2)
                    mg2 = (pr2 * pr2).sum(dim=2).sqrt().clamp(min=1e-8)
                    dn2 = (pr2 / mg2.unsqueeze(2)).view(B, D_TAU_ANG)
                    hyb2 = torch.cat([dn2, (pr2*pr2).sum(dim=2).sqrt()], dim=1)
                    tau2 = (hyb2 + model.W_tok_inject[x0_eval]) if t == 0 else hyb2
                    sids2 = model.TR[sids2].gather(1, w2.argmax(dim=1).unsqueeze(1)).squeeze(1)
                soft_taus2.append(tau2)

            st2 = torch.stack(soft_taus2, dim=1)
            ha2 = torch.tanh(st2 @ model.W_attn.t() + model.b_attn)
            sc2 = (ha2 * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha2 = torch.softmax(sc2, dim=1)
            alpha0_sum += alpha2[:, 0].mean().item()
            n_decisions += 1

            transport_sum += transport_ops / max(total_ops, 1)

    model.train()
    alpha0 = alpha0_sum / max(n_decisions, 1)
    transport_frac = transport_sum / max(n_batches, 1)
    return round(alpha0, 4), round(transport_frac, 4)


# ═══════════════════════════════════════════════════════════════════════
# Forward-pass timing
# ═══════════════════════════════════════════════════════════════════════
def time_forward_pass(
    model: RouterFanoutVariant,
    pool_ids: torch.Tensor,
    n_calls: int = N_TIMING,
) -> float:
    """Return mean forward-pass time in milliseconds (eval mode, no grad)."""
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    model.eval()
    times = []
    with torch.no_grad():
        for _ in range(n_calls + 5):  # 5 warmup calls
            sids, toks, x0 = make_batch(pool_ids, rng)
            t0 = time.perf_counter()
            _ = model(sids, toks, x0, 0.05)
            times.append((time.perf_counter() - t0) * 1000.0)
    model.train()
    return round(sum(times[5:]) / n_calls, 3)  # skip warmup


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_variant(
    model: RouterFanoutVariant,
    pool_ids: torch.Tensor,
    label: str,
) -> Dict:
    opt = torch.optim.SGD(model.parameters(), lr=LR)
    rng_train = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    rng_eval  = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    model.train()

    solve_step = None
    final_acc  = 0.0
    t0 = time.perf_counter()
    eval_log = []

    for step in range(1, MAX_STEPS + 1):
        frac = step / max(MAX_STEPS - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_train)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()

        if step % EVAL_EVERY == 0:
            acc = evaluate(model, pool_ids, rng_eval)
            eval_log.append((step, round(acc, 4)))
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc
            print(f"    [{label}] step={step:4d}  acc={acc:.4f}")

    wall = time.perf_counter() - t0
    sps  = MAX_STEPS / wall
    return {
        "final_acc": round(final_acc, 4),
        "solve_step": solve_step if solve_step else "DNF",
        "wall_time_s": round(wall, 2),
        "steps_per_sec": round(sps, 1),
        "eval_log": eval_log,
    }


# ═══════════════════════════════════════════════════════════════════════
# Variant definitions
# ═══════════════════════════════════════════════════════════════════════
VARIANTS = [
    {
        "label":        "k6_baseline",
        "fanout_k":     6,
        "use_geom_only": False,
        "note":         "All 6 candidates — standard forward pass, exact baseline",
    },
    {
        "label":        "k4_geom",
        "fanout_k":     4,
        "use_geom_only": False,
        "note":         "Top-4 by angular proximity; 2 most-distant masked to -inf",
    },
    {
        "label":        "k3_geom",
        "fanout_k":     3,
        "use_geom_only": False,
        "note":         "Top-3 by angular proximity; 3 most-distant masked to -inf",
    },
    {
        "label":        "k2_geom",
        "fanout_k":     2,
        "use_geom_only": False,
        "note":         "Top-2 by angular proximity; 4 most-distant masked to -inf",
    },
    {
        "label":        "k1_geom_hard",
        "fanout_k":     1,
        "use_geom_only": True,
        "note":         "Top-1 by angular proximity; hard tau; MLP bypassed; W1/W2 no-grad",
    },
]


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("GEOMETRY-GUIDED FAN-OUT REDUCTION PROBE v1")
    print("=" * 70)
    print(f"Config: D={D}  D_HIDDEN={D_HIDDEN}  B={BATCH_SIZE}  MAX_STEPS={MAX_STEPS}")
    print(f"Geometric rule: angular similarity (dot product in S^1 x S^1 x S^1 x S^1)")
    print(f"Variants: {[v['label'] for v in VARIANTS]}")
    print()

    # ──────────────────────────────────────────────────────────────────
    # Load cache
    # ──────────────────────────────────────────────────────────────────
    print(f"Loading cache: {CACHE_PATH}")
    t_load_start = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    t_load = time.perf_counter() - t_load_start
    n_states = data["TN_oh"].shape[0]
    print(f"  {n_states:,} states loaded in {t_load:.3f}s")

    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )
    print(f"  TN_ang: {tuple(TN_ang.shape)}  TR: {tuple(TR.shape)}")
    print(f"  pool_ids: {pool_ids.shape[0]:,} states\n")

    # ──────────────────────────────────────────────────────────────────
    # Run all variants
    # ──────────────────────────────────────────────────────────────────
    rows: List[Dict] = []
    baseline_fwd_ms = None

    for vi, vcfg in enumerate(VARIANTS):
        label        = vcfg["label"]
        fanout_k     = vcfg["fanout_k"]
        use_geom_only = vcfg["use_geom_only"]

        print(f"[{vi+1}/{len(VARIANTS)}] {label}  (k={fanout_k}, geom_only={use_geom_only})")
        print(f"  Note: {vcfg['note']}")

        model = RouterFanoutVariant(
            TN_ang, TR, tau0_hyb, pool_ids,
            fanout_k=fanout_k,
            use_geom_only=use_geom_only,
        )
        n_params = sum(p.numel() for p in model.parameters())
        print(f"  Parameters: {n_params:,}")

        train_result = train_variant(model, pool_ids, label)
        fwd_ms = time_forward_pass(model, pool_ids)
        alpha0, transport_frac = measure_alpha0_transport(model, pool_ids)

        if baseline_fwd_ms is None:
            baseline_fwd_ms = fwd_ms

        fwd_speedup = round(baseline_fwd_ms / fwd_ms, 3) if fwd_ms > 0 else "N/A"

        candidate_work_pct = round(100.0 * (1.0 - fwd_ms / baseline_fwd_ms), 1) if vi > 0 else 47.6

        # KEY TEST: evaluate this trained model using k=1 geometric routing.
        # If the manifold is carrying the weight, k1_geom_acc should ≈ trained acc.
        k1_geom_acc, geom_mlp_agree = eval_k1_geom_on_model(model, pool_ids)
        print(f"  → k1_geom_eval: acc={k1_geom_acc:.4f}  geom_mlp_agreement={geom_mlp_agree:.4f}")

        row = {
            "variant":               label,
            "candidate_fanout":      fanout_k,
            "geom_only":             use_geom_only,
            "accuracy":              train_result["final_acc"],
            "solve_step":            train_result["solve_step"],
            "steps_per_second":      train_result["steps_per_sec"],
            "wall_time_s":           train_result["wall_time_s"],
            "forward_time_ms":       fwd_ms,
            "fwd_speedup_vs_k6":     fwd_speedup,
            "candidate_work_pct_of_fwd": candidate_work_pct if vi == 0 else f"Δ{candidate_work_pct:+.1f}% vs k6",
            "alpha0":                alpha0,
            "transport_fraction":    transport_frac,
            "k1_geom_eval_acc":      k1_geom_acc,
            "geom_mlp_agreement":    geom_mlp_agree,
            "note":                  vcfg["note"],
        }
        rows.append(row)

        print(f"  → acc={train_result['final_acc']:.4f}  solve={train_result['solve_step']}"
              f"  sps={train_result['steps_per_sec']}  wall={train_result['wall_time_s']}s")
        print(f"  → fwd={fwd_ms:.3f}ms  speedup_vs_k6={fwd_speedup}x"
              f"  alpha0={alpha0}  transport={transport_frac}\n")

    # ──────────────────────────────────────────────────────────────────
    # Analysis
    # ──────────────────────────────────────────────────────────────────
    baseline_acc = rows[0]["accuracy"]
    baseline_solve = rows[0]["solve_step"]
    baseline_fwd   = rows[0]["forward_time_ms"]

    # Find minimum safe fanout (first k where accuracy drops below 0.99 or solve fails)
    min_safe_k = rows[0]["candidate_fanout"]
    for r in rows[1:]:
        acc  = r["accuracy"]
        solv = r["solve_step"]
        if acc >= 0.990 and solv != "DNF":
            min_safe_k = r["candidate_fanout"]
        else:
            break

    # Degrade finding
    first_degrade = None
    for r in rows[1:]:
        if r["accuracy"] < 0.990 or r["solve_step"] == "DNF":
            first_degrade = r
            break

    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"Baseline (k=6): acc={baseline_acc}  solve={baseline_solve}  fwd={baseline_fwd}ms")
    for r in rows[1:]:
        delta = round(r["accuracy"] - baseline_acc, 4)
        print(f"  {r['variant']:20s}  acc={r['accuracy']:.4f} (Δ{delta:+.4f})"
              f"  solve={str(r['solve_step']):6s}  fwd={r['forward_time_ms']:.3f}ms")
    print()
    if first_degrade:
        print(f"First degradation: {first_degrade['variant']} (acc={first_degrade['accuracy']:.4f})")
    print(f"MINIMUM SAFE FAN-OUT: {min_safe_k}")

    # ──────────────────────────────────────────────────────────────────
    # Write CSV
    # ──────────────────────────────────────────────────────────────────
    fieldnames = [
        "variant", "candidate_fanout", "geom_only",
        "accuracy", "solve_step", "steps_per_second", "wall_time_s",
        "forward_time_ms", "fwd_speedup_vs_k6", "candidate_work_pct_of_fwd",
        "alpha0", "transport_fraction",
        "k1_geom_eval_acc", "geom_mlp_agreement",
        "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"\nCSV written: {CSV_OUT}")

    # ──────────────────────────────────────────────────────────────────
    # Write Markdown
    # ──────────────────────────────────────────────────────────────────
    _write_markdown(rows, baseline_acc, baseline_solve, baseline_fwd,
                    min_safe_k, first_degrade, t_load, n_states)
    print(f"Markdown written: {MD_OUT}")


def _write_markdown(
    rows, baseline_acc, baseline_solve, baseline_fwd,
    min_safe_k, first_degrade, t_load, n_states
):
    import datetime
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")

    baseline_fwd_float = float(rows[0]["forward_time_ms"])

    lines = []
    lines.append("# Prime Transport Router: Fan-Out Reduction Probe v1\n")
    lines.append(f"**Generated:** {ts}  \n")
    lines.append(f"**Config:** D={D}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
                 f"MAX_STEPS={MAX_STEPS}, SOLVE_ACC={SOLVE_ACC}  \n")
    lines.append(f"**Cache:** {n_states:,} states loaded in {t_load:.3f}s  \n\n")

    lines.append("---\n\n")
    lines.append("## Geometric Pruning Rule\n\n")
    lines.append("**Angular similarity** between current tau direction and each candidate:\n\n")
    lines.append("```\n")
    lines.append("cur_dir  = tau_prev[:, :8]           # direction component of hybrid tau (B, 8)\n")
    lines.append("ang_sim  = einsum('bi,bij->bj', cur_dir, TN[state_ids])  # (B, 6) cosine-like sim\n")
    lines.append("# TN entries are unit angular vectors (cos θ, sin θ) per phase pair\n")
    lines.append("# So ang_sim[i] ∈ [-1, 1] for each candidate operator i\n")
    lines.append("# Bottom (6-k) operators by ang_sim are masked to -inf before softmax\n")
    lines.append("```\n\n")
    lines.append("This is purely local: no future-state lookahead, no learned gate, no randomness.\n\n")

    lines.append("---\n\n")
    lines.append("## Results Table\n\n")
    header = ("| variant | k | accuracy | solve_step | sps | fwd_ms | speedup "
              "| k1_geom_eval | geom_mlp_agree | alpha0 | transport_frac |\n")
    sep    = ("|---------|---|----------|------------|-----|--------|---------|"
              "--------------|----------------|--------|----------------|\n")
    lines.append(header)
    lines.append(sep)
    for r in rows:
        delta = round(r["accuracy"] - baseline_acc, 4)
        delta_str = f"({delta:+.4f})"
        spd = r["fwd_speedup_vs_k6"]
        lines.append(
            f"| {r['variant']} | {r['candidate_fanout']} "
            f"| {r['accuracy']:.4f} {delta_str} "
            f"| {r['solve_step']} "
            f"| {r['steps_per_second']} "
            f"| {r['forward_time_ms']:.3f} "
            f"| {spd}x "
            f"| {r['k1_geom_eval_acc']:.4f} "
            f"| {r['geom_mlp_agreement']} "
            f"| {r['alpha0']} "
            f"| {r['transport_fraction']} |\n"
        )
    lines.append("\n")

    lines.append("---\n\n")
    lines.append("## Phase Timeline\n\n")
    lines.append(f"- **CACHE_LOAD:** {t_load:.3f}s → {n_states:,} states\n")
    lines.append(f"- **PREPARE_HYBRID_TABLES:** vectorized (4 phase pairs)\n")
    for r in rows:
        lines.append(f"- **TRAIN [{r['variant']}]:** {r['wall_time_s']}s "
                     f"→ acc={r['accuracy']:.4f}, solve={r['solve_step']}\n")
    lines.append("\n")

    lines.append("---\n\n")
    lines.append("## Forward-Pass Timing\n\n")
    lines.append(f"Baseline k=6: {baseline_fwd:.3f}ms per call (B=256, D=24, eval mode, 100 calls)\n\n")
    lines.append("| variant | fwd_ms | speedup vs k=6 | note |\n")
    lines.append("|---------|--------|-----------------|------|\n")
    for r in rows:
        lines.append(f"| {r['variant']} | {r['forward_time_ms']:.3f}ms "
                     f"| {r['fwd_speedup_vs_k6']}x "
                     f"| {r['note']} |\n")
    lines.append("\n")
    lines.append("> **candidate_work_pct_of_fwd (k=6 baseline):** 47.6%  \n")
    lines.append("> Source: lookahead probe v1 (CAND_GEN 12.4% + CAND_SCORE 23.5% + CAND_COMB 11.7%)\n\n")

    lines.append("---\n\n")
    lines.append("## Geometry Hypothesis Test\n\n")
    lines.append("> **Hypothesis:** If the model has learned the correct manifold geometry,\n")
    lines.append("> k=1 geometric routing (angular proximity alone) should be sufficient.\n")
    lines.append("> The MLP and soft blend are differentiable training scaffolding —\n")
    lines.append("> the radial angle/length representation should carry the routing weight.\n\n")
    lines.append("### k=1 Geometric Inference on Each Trained Model\n\n")
    lines.append("*(Each model was evaluated with k=1 angular-proximity routing,\n")
    lines.append("regardless of what fanout_k it was trained with.)*\n\n")
    lines.append("| trained_variant | trained_acc | k1_geom_eval_acc | drop | geom_mlp_agreement |\n")
    lines.append("|-----------------|-------------|------------------|------|--------------------|\n")
    for r in rows:
        drop = round(r["accuracy"] - r["k1_geom_eval_acc"], 4)
        lines.append(
            f"| {r['variant']} "
            f"| {r['accuracy']:.4f} "
            f"| {r['k1_geom_eval_acc']:.4f} "
            f"| {drop:+.4f} "
            f"| {r['geom_mlp_agreement']} |\n"
        )
    lines.append("\n")
    # Interpretation
    k6_row = rows[0]
    k6_geom_eval = k6_row["k1_geom_eval_acc"]
    k6_acc = k6_row["accuracy"]
    k6_agree = k6_row["geom_mlp_agreement"]
    drop_k6 = round(k6_acc - k6_geom_eval, 4)
    lines.append("**Interpretation (trained k=6 model):**\n\n")
    lines.append(f"- Trained accuracy: {k6_acc:.4f}\n")
    lines.append(f"- k=1 geom eval accuracy: {k6_geom_eval:.4f}  (drop: {drop_k6:+.4f})\n")
    lines.append(f"- geom/MLP argmax agreement: {k6_agree:.4f}\n\n")
    if k6_agree >= 0.90:
        lines.append("→ **HIGH AGREEMENT**: The MLP argmax and geometric nearest-neighbor agree\n")
        lines.append(f"  on ≥{k6_agree*100:.0f}% of routing steps. The manifold geometry is\n")
        lines.append("  carrying the routing weight. The soft blend and MLP are converging\n")
        lines.append("  toward the same answer as geometric proximity.\n\n")
    elif k6_agree >= 0.70:
        lines.append("→ **MODERATE AGREEMENT**: MLP and geometry agree on most but not all steps.\n")
        lines.append("  The manifold is partially encoding routing but the MLP is adding information\n")
        lines.append("  that geometric proximity alone misses.\n\n")
    else:
        lines.append("→ **LOW AGREEMENT**: MLP routing and geometric proximity diverge significantly.\n")
        lines.append("  The manifold geometry alone is NOT sufficient for routing — the MLP is\n")
        lines.append("  learning something genuinely different from nearest-neighbor geometry.\n\n")

    if drop_k6 <= 0.005:
        lines.append("→ **GEOMETRY HYPOTHESIS: CONFIRMED** — k=1 geometric inference on the trained\n")
        lines.append(f"  model achieves {k6_geom_eval:.4f} accuracy (drop ≤ 0.005 from trained {k6_acc:.4f}).\n")
        lines.append("  The manifold and radial angle/length are carrying the routing weight.\n\n")
    elif drop_k6 <= 0.02:
        lines.append("→ **GEOMETRY HYPOTHESIS: PARTIAL** — k=1 geometric inference achieves\n")
        lines.append(f"  {k6_geom_eval:.4f} (drop {drop_k6:.4f}). The geometry carries most of the\n")
        lines.append("  routing weight but the soft blend contributes a measurable margin.\n\n")
    else:
        lines.append("→ **GEOMETRY HYPOTHESIS: NOT CONFIRMED** — k=1 geometric inference drops\n")
        lines.append(f"  {drop_k6:.4f} from trained accuracy. The soft 6-way blend is doing real work\n")
        lines.append("  that geometry alone cannot replicate at inference time.\n\n")

    lines.append("---\n\n")
    lines.append("## First Degradation Point\n\n")
    if first_degrade:
        lines.append(f"First fan-out level where performance degrades meaningfully: "
                     f"**{first_degrade['variant']}** (k={first_degrade['candidate_fanout']})\n\n")
        lines.append(f"- accuracy={first_degrade['accuracy']:.4f} "
                     f"(Δ{first_degrade['accuracy'] - baseline_acc:+.4f} vs baseline)\n")
        lines.append(f"- solve_step={first_degrade['solve_step']}\n\n")
    else:
        lines.append("No meaningful degradation observed across tested fan-out levels.\n\n")

    lines.append("---\n\n")
    lines.append("## Honesty Section\n\n")
    lines.append("### What improved\n\n")

    # Compute which variants are strictly faster
    improved_fwd = [r for r in rows[1:] if float(r["forward_time_ms"]) < baseline_fwd_float]
    if improved_fwd:
        for r in improved_fwd:
            lines.append(f"- {r['variant']}: forward-pass time {r['forward_time_ms']:.3f}ms "
                         f"vs baseline {baseline_fwd_float:.3f}ms "
                         f"(speedup {r['fwd_speedup_vs_k6']}x)\n")
    else:
        lines.append("- No variants showed forward-pass speedup over baseline.\n")
        lines.append("  The added ang_sim einsum (same cost as CAND_COMB) offsets the blend reduction.\n")
    lines.append("\n")

    lines.append("### What degraded\n\n")
    degraded = [r for r in rows[1:] if r["accuracy"] < baseline_acc - 0.005 or r["solve_step"] == "DNF"]
    if degraded:
        for r in degraded:
            lines.append(f"- {r['variant']}: acc={r['accuracy']:.4f} "
                         f"(Δ{r['accuracy'] - baseline_acc:+.4f}), "
                         f"solve={r['solve_step']}\n")
    else:
        lines.append("- No variants showed meaningful accuracy degradation (< 0.005 delta).\n")
    lines.append("\n")

    lines.append("### Whether geometry is doing real pruning work\n\n")
    lines.append("The angular similarity criterion identifies which of the 6 candidate tau-nexts\n")
    lines.append("are most aligned with the current trajectory direction in S^1×S^1×S^1×S^1.\n")
    lines.append("Pruning to top-k removes candidates that are geometrically anti-aligned.\n\n")

    lines.append("Key limitation: the ang_sim computation itself requires fetching all 6 TN entries\n")
    lines.append("and computing a (B,6) dot product — similar cost to the einsum it replaces.\n")
    lines.append("Net timing benefit from blend reduction is therefore partially offset by the\n")
    lines.append("added geometric scoring overhead.\n\n")

    lines.append("For k=1 geom_hard: W1/W2 receive no gradient (MLP bypassed). This variant\n")
    lines.append("is structurally unable to learn routing weights. Any accuracy shown is from\n")
    lines.append("residual learning in W_tok_inject, W_attn, W_pred only.\n\n")

    lines.append("---\n\n")
    lines.append("## Questions Answered\n\n")
    lines.append("**1. How low can fan-out go before performance degrades?**\n\n")
    lines.append(f"Minimum safe fan-out based on acc ≥ 0.990 and solve before DNF: **k={min_safe_k}**\n\n")

    lines.append("**2. Does geometry-guided pruning materially reduce runtime?**\n\n")
    if improved_fwd:
        lines.append("Yes — variants with k<6 show measurable forward-pass speedup. "
                     "However, the ang_sim overhead partially offsets gains.\n\n")
    else:
        lines.append("No — the ang_sim overhead (one extra (B,6) dot product per step) "
                     "offsets the blend reduction. Net speedup is negligible.\n\n")

    lines.append("**3. Does the soft tau update actually need all 6 candidates?**\n\n")

    # Check if k4,k3,k2 all solve
    soft_k_solves = [r for r in rows[1:4] if r["solve_step"] != "DNF" and r["accuracy"] >= 0.990]
    if soft_k_solves:
        min_soft_k = min(r["candidate_fanout"] for r in soft_k_solves)
        lines.append(f"No — the soft tau update can use as few as k={min_soft_k} candidates "
                     f"without meaningful accuracy loss.\n\n")
    else:
        lines.append("Uncertain — reduction below k=4 may degrade accuracy based on measured results.\n\n")

    lines.append("---\n\n")
    lines.append(f"## MINIMUM SAFE FAN-OUT: {min_safe_k}\n")

    with open(MD_OUT, "w") as f:
        f.writelines(lines)


if __name__ == "__main__":
    main()
